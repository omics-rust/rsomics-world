//! `samtools ampliconclip` port: clip amplicon primer regions off aligned reads.
//!
//! Given a BED of primer coordinates, each mapped read is matched against the
//! primer list for its reference and, on a hit, the overlapping primer bases are
//! removed from the read's 5' end (default) — soft-clipped (CIGAR rewrite,
//! leading/trailing `S`) by default, or hard-clipped (`--hard-clip`: SEQ/QUAL
//! removed, POS updated for a 5' cut, CIGAR recomputed). `--both-ends` clips both
//! the 5' and 3' ends; `--strand` restricts each primer to its BED strand.
//!
//! Semantics mirror samtools `bam_ampliconclip.c` (`bam_clip` /
//! `matching_clip_site` / `bam_trim_left` / `bam_trim_right`); see the cited
//! per-function behaviour in [`clip`] and [`bed`].
//!
//! Records are streamed and edited on their raw BAM payload bytes (the same
//! layout htslib's `bam1_t` holds), so the clip is a tight CIGAR/SEQ/QUAL byte
//! surgery with no decode/re-encode round-trip through a noodles `RecordBuf`.
//! Only the header is parsed via noodles (to flip `SO:coordinate` → `SO:unknown`
//! and add the `@PG` line). Output BGZF goes through the `rsomics-bamio` parallel
//! writer (libdeflate), which is where the throughput win over samtools comes
//! from on top of the tight clip loop.

mod bed;
mod clip;

use std::io::Write;
use std::num::NonZero;
use std::path::Path;

use noodles::bam;
use noodles::sam::Header;
use noodles::sam::header::record::value::Map;
use noodles::sam::header::record::value::map::Program;
use noodles::sam::header::record::value::map::header::tag::SORT_ORDER;
use noodles::sam::header::record::value::map::program::tag as program_tag;
use rsomics_bamio::raw::{self, RawRecord};
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

use bed::PrimerBed;
use clip::{RawBam, trim_left, trim_right};

pub use clip::Clipping;

// SAM/BAM FLAG bits the clip path inspects (SAMv1 §1.4).
const FLAG_UNMAPPED: u16 = 0x4;
const FLAG_REVERSE: u16 = 0x10;
const FLAG_QCFAIL: u16 = 0x200;

/// Per-run counters, mirroring samtools' stats block.
#[derive(Debug, Default, Clone, Serialize)]
pub struct ClipStats {
    pub total: u64,
    pub clipped: u64,
    pub forward_clipped: u64,
    pub reverse_clipped: u64,
    pub both_clipped: u64,
    pub not_clipped: u64,
    pub excluded: u64,
    pub filtered: u64,
    pub failed: u64,
    pub written: u64,
}

#[derive(Debug, Clone)]
pub struct ClipOpts {
    pub clipping: Clipping,
    pub both_ends: bool,
    pub use_strand: bool,
    pub tolerance: i64,
    pub mark_fail: bool,
    pub write_clipped: bool,
    pub no_excluded: bool,
    pub fail_len: i64,
    pub filter_len: i64,
    pub unmap_len: i64,
    pub add_pg: bool,
    /// Keep the NM/MD tags on clipped reads. samtools deletes them by default
    /// (`del_tag = 1`) because clipping invalidates the edit-distance/mismatch
    /// strings; `--keep-tag` turns that off.
    pub keep_tag: bool,
}

impl Default for ClipOpts {
    fn default() -> Self {
        // samtools defaults (cl_param_t init): soft clip, tolerance 5, add @PG,
        // length filters off (-1), unmap-len 0.
        ClipOpts {
            clipping: Clipping::Soft,
            both_ends: false,
            use_strand: false,
            tolerance: 5,
            mark_fail: false,
            write_clipped: false,
            no_excluded: false,
            fail_len: -1,
            filter_len: -1,
            unmap_len: 0,
            add_pg: true,
            keep_tag: false,
        }
    }
}

/// Run ampliconclip on `input` against `bedfile`, writing to `output_path`
/// (`None` = stdout). `arg_list` is the command-line string for the `@PG` CL tag.
pub fn ampliconclip(
    input: &Path,
    output_path: Option<&Path>,
    bedfile: &Path,
    opts: &ClipOpts,
    arg_list: &str,
    workers: NonZero<usize>,
) -> Result<ClipStats> {
    let primers = bed::load(bedfile, opts.use_strand)?;

    let mut reader = rsomics_bamio::open_with_workers(input, workers)?;
    let mut header = reader.read_header().map_err(RsomicsError::Io)?;
    rewrite_header(&mut header, opts, arg_list)?;

    match output_path {
        Some(path) => {
            let mut writer = rsomics_bamio::create_with_workers(path, workers)?;
            run(&mut reader, &mut writer, &header, primers, opts)
        }
        None => {
            let mut writer = bam::io::Writer::new(std::io::stdout().lock());
            run(&mut reader, &mut writer, &header, primers, opts)
        }
    }
}

/// samtools changes `SO:coordinate` to `SO:unknown` (clipping shifts POS, ruining
/// the sort) and adds an `@PG` line unless `--no-PG`.
fn rewrite_header(header: &mut Header, opts: &ClipOpts, arg_list: &str) -> Result<()> {
    if let Some(hd) = header.header_mut().as_mut() {
        let is_coord = hd
            .other_fields()
            .get(&SORT_ORDER)
            .map(|so| AsRef::<[u8]>::as_ref(so) == b"coordinate")
            .unwrap_or(false);
        if is_coord {
            hd.other_fields_mut().insert(SORT_ORDER, "unknown".into());
        }
    }

    if opts.add_pg {
        let program = Map::<Program>::builder()
            .insert(program_tag::NAME, "rsomics-bam-ampliconclip")
            .insert(program_tag::VERSION, env!("CARGO_PKG_VERSION"))
            .insert(program_tag::COMMAND_LINE, arg_list)
            .build()
            .map_err(|e| RsomicsError::InvalidInput(format!("building @PG: {e}")))?;
        header
            .programs_mut()
            .add("rsomics-bam-ampliconclip", program)
            .map_err(RsomicsError::Io)?;
    }

    Ok(())
}

fn run<R, W>(
    reader: &mut bam::io::Reader<R>,
    writer: &mut bam::io::Writer<W>,
    header: &Header,
    mut primers: PrimerBed,
    opts: &ClipOpts,
) -> Result<ClipStats>
where
    R: std::io::Read,
    W: Write,
{
    writer.write_header(header).map_err(RsomicsError::Io)?;

    // Map BAM tid → reference name → primer list, resolved lazily as the tid
    // changes (samtools caches `sites`/`ref_found` across same-tid runs).
    let ref_names: Vec<String> = header
        .reference_sequences()
        .keys()
        .map(|k| String::from_utf8_lossy(k.as_ref()).into_owned())
        .collect();

    let mut stats = ClipStats::default();
    let mut raw = RawRecord::default();
    let mut last_tid: i32 = i32::MIN;
    let mut current_ref: Option<String> = None;

    while raw::read_record(reader.get_mut(), &mut raw)? != 0 {
        stats.total += 1;
        let mut rec = RawBam {
            bytes: raw.as_bytes().to_vec(),
        };

        let tid = rec.reference_sequence_id();
        if tid != last_tid {
            last_tid = tid;
            current_ref = (tid >= 0)
                .then(|| ref_names.get(tid as usize).cloned())
                .flatten()
                .filter(|name| primers.by_ref.contains_key(name));
        }

        let ref_found = current_ref.is_some();
        let excluded = rec.flags() & (FLAG_UNMAPPED | FLAG_QCFAIL) != 0;

        let mut filter = false;

        if !excluded && ref_found {
            let ref_name = current_ref.as_ref().unwrap();
            let list = primers.by_ref.get_mut(ref_name).unwrap();
            let mut been_clipped = false;

            if !opts.both_ends {
                let is_rev = rec.flags() & FLAG_REVERSE != 0;
                let pos = if is_rev { rec.endpos() } else { rec.pos() };
                let size =
                    bed::matching_clip_site(list, pos, is_rev, opts.use_strand, opts.tolerance);
                if size > 0 {
                    if is_rev {
                        rec = trim_right(&rec, size as u32, opts.clipping);
                        stats.reverse_clipped += 1;
                    } else {
                        rec = trim_left(&rec, size as u32, opts.clipping);
                        stats.forward_clipped += 1;
                    }
                    been_clipped = true;
                } else {
                    if opts.mark_fail {
                        rec.set_flag_bits(FLAG_QCFAIL);
                    }
                    stats.not_clipped += 1;
                }
            } else {
                // both-ends: left (forward) first, then right (reverse), each
                // re-reading the (possibly already left-clipped) read's end.
                let mut left = false;
                let mut right = false;

                let lsize = bed::matching_clip_site(
                    list,
                    rec.pos(),
                    false,
                    opts.use_strand,
                    opts.tolerance,
                );
                if lsize > 0 {
                    rec = trim_left(&rec, lsize as u32, opts.clipping);
                    stats.forward_clipped += 1;
                    left = true;
                    been_clipped = true;
                }

                let rsize = bed::matching_clip_site(
                    list,
                    rec.endpos(),
                    true,
                    opts.use_strand,
                    opts.tolerance,
                );
                if rsize > 0 {
                    rec = trim_right(&rec, rsize as u32, opts.clipping);
                    stats.reverse_clipped += 1;
                    right = true;
                    been_clipped = true;
                }

                if left && right {
                    stats.both_clipped += 1;
                } else if !left && !right {
                    if opts.mark_fail {
                        rec.set_flag_bits(FLAG_QCFAIL);
                    }
                    stats.not_clipped += 1;
                }
            }

            // samtools' default `del_tag`: a clip invalidates the NM (edit
            // distance) and MD (mismatch) strings, so they are removed from a
            // clipped read unless `--keep-tag`.
            if been_clipped && !opts.keep_tag {
                rec.remove_aux(*b"NM");
                rec.remove_aux(*b"MD");
            }

            // Length filters operate on the post-clip active query length.
            if opts.fail_len >= 0 || opts.filter_len >= 0 || opts.unmap_len >= 0 {
                let aql = rec.active_query_len();
                if opts.fail_len >= 0 && aql <= opts.fail_len {
                    rec.set_flag_bits(FLAG_QCFAIL);
                }
                if opts.filter_len >= 0 && aql <= opts.filter_len {
                    filter = true;
                }
                if opts.unmap_len >= 0 && aql <= opts.unmap_len {
                    rec = rec.unmap();
                }
            }

            if rec.flags() & FLAG_QCFAIL != 0 {
                stats.failed += 1;
            }

            if opts.write_clipped && !been_clipped {
                filter = true;
            }
        } else {
            stats.excluded += 1;
            if opts.no_excluded {
                filter = true;
            }
        }

        if !filter {
            write_raw(writer.get_mut(), &rec.bytes)?;
            stats.written += 1;
        } else {
            stats.filtered += 1;
        }
    }

    // samtools' `TOTAL CLIPPED: f_count + r_count`; a both-ends read clipped at
    // both ends contributes to both counters.
    stats.clipped = stats.forward_clipped + stats.reverse_clipped;

    Ok(stats)
}

/// Write one raw record payload: `block_size` (u32 LE) then the bytes, exactly as
/// [`raw::write_record`] does, but from an owned byte buffer the clip produced.
fn write_raw<W: Write>(writer: &mut W, bytes: &[u8]) -> Result<()> {
    let block_size = u32::try_from(bytes.len())
        .map_err(|e| RsomicsError::InvalidInput(format!("record too large: {e}")))?;
    writer
        .write_all(&block_size.to_le_bytes())
        .map_err(RsomicsError::Io)?;
    writer.write_all(bytes).map_err(RsomicsError::Io)?;
    Ok(())
}
