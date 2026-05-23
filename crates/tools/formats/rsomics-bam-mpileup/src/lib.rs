//! `samtools mpileup` port over the [`rsomics_pileup`] engine.
//!
//! mpileup walks a coordinate-sorted BAM column by column and prints, per
//! reference position, the read bases / base qualities (and optional map
//! qualities / read positions / etc.) over the covering reads. The column
//! construction — which reads cover a position, each read's `qpos` / `is_del` /
//! `indel` / `is_head` / `is_tail`, and overlapping-mate quality removal — is the
//! shared [`rsomics_pileup`] engine; this crate only encodes each column's text.
//!
//! The encoding mirrors `samtools` `bam_plcmd.c` (`pileup_seq` + the column
//! output loop): the `.,ACGTN`/`=` base alphabet, `^<mapqchar>` / `$` read
//! start/end markers, `+N<seq>` / `-N<refseq>` indel notation, `*` for a deleted
//! position, and the per-base `min_baseQ` (default 13) filter applied before a
//! base is counted. See the per-function comments for the cited behaviour.
//!
//! Defaults follow `samtools mpileup`: `min_baseQ = 13`, `min_mapq = 0`,
//! `rflag_filter = UNMAP|SECONDARY|QCFAIL|DUP`, orphan filtering and overlap
//! removal ON. Base Alignment Quality (BAQ, `sam_prob_realn`) — the one default
//! that mutates qualities when `-f` is given — is **not** implemented; pass `-B`
//! (no BAQ) for byte-exact reference-aware output, matching `samtools mpileup -B`.

use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::num::NonZero;
use std::path::Path;

use rsomics_bamio::raw::RawRecord;
use rsomics_common::{Result, RsomicsError};
use rsomics_pileup::{Column, PileupEngine, PileupOpts, PileupRead};
use serde::Serialize;

const FLAG_REVERSE: u16 = 0x10;

/// The pileup base alphabet (`pileup_seq`'s `seq_nt_str_uc`/`seq_nt_str_lc`),
/// indexed by the 4-bit `seq_nt16` code. Uppercase = forward strand, lowercase =
/// reverse. Slot 0 is `.`/`,` — a base equal to the reference; the literal `=`
/// base never reaches here because a `=` query base is itself encoded as the
/// matched-reference dot only when a reference is supplied.
const SEQ_NT_UC: &[u8; 16] = b".ACMGRSVTWYHKDBN";
const SEQ_NT_LC: &[u8; 16] = b",acmgrsvtwyhkdbn";

/// htslib `seq_nt16_str`: the raw `seq_nt16` → base alphabet (slot 0 = `=`).
/// Insertion bases are emitted through this verbatim (then upper/lower-cased by
/// strand), unlike the column base which maps slot 0 to the matched-ref dot.
const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";

/// htslib `seq_nt16_table`: ASCII reference base → 4-bit `seq_nt16` code. Only
/// the entries the encoder consults (ACGTN + lowercase + ambiguity) need to be
/// correct; everything else maps to N (15).
fn nt16_of_ref(b: u8) -> u8 {
    match b.to_ascii_uppercase() {
        b'=' => 0,
        b'A' => 1,
        b'C' => 2,
        b'M' => 3,
        b'G' => 4,
        b'R' => 5,
        b'S' => 6,
        b'V' => 7,
        b'T' => 8,
        b'W' => 9,
        b'Y' => 10,
        b'H' => 11,
        b'K' => 12,
        b'D' => 13,
        b'B' => 14,
        _ => 15,
    }
}

#[derive(Debug, Clone)]
pub struct MpileupOpts {
    pub min_baseq: u8,
    pub min_mapq: u8,
    pub max_depth: u32,
    pub no_overlaps: bool,
    pub no_orphan_filter: bool,
    pub rflag_filter: u16,
    pub rflag_require: u16,
    /// Output all positions, including zero-coverage (`-a`); `-aa` (output all
    /// references too) sets `output_all = 2`.
    pub output_all: u8,
    /// `-B`: disable BAQ. We never apply BAQ, so this is informational — but we
    /// reject default `-f` (BAQ on) loudly so a user is not silently given
    /// non-samtools output.
    pub no_baq: bool,
}

impl Default for MpileupOpts {
    fn default() -> Self {
        Self {
            min_baseq: 13,
            min_mapq: 0,
            max_depth: 8000,
            no_overlaps: false,
            no_orphan_filter: false,
            rflag_filter: 0x704,
            rflag_require: 0,
            output_all: 0,
            no_baq: false,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct MpileupStats {
    pub positions: u64,
}

/// A loaded reference: contig name → sequence bytes (as stored in the FASTA,
/// case preserved — samtools prints the reference base in the column verbatim).
struct Reference {
    contigs: HashMap<String, Vec<u8>>,
}

impl Reference {
    /// Parse a FASTA into memory. Multi-line records are concatenated; the name
    /// is the first whitespace-delimited token of the header.
    fn load(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path)
            .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
        let mut contigs: HashMap<String, Vec<u8>> = HashMap::new();
        let mut name: Option<String> = None;
        let mut seq: Vec<u8> = Vec::new();
        for line in bytes.split(|&b| b == b'\n') {
            let line = line.strip_suffix(b"\r").unwrap_or(line);
            if line.first() == Some(&b'>') {
                if let Some(n) = name.take() {
                    contigs.insert(n, std::mem::take(&mut seq));
                }
                let header = &line[1..];
                let end = header
                    .iter()
                    .position(u8::is_ascii_whitespace)
                    .unwrap_or(header.len());
                name = Some(String::from_utf8_lossy(&header[..end]).into_owned());
            } else {
                seq.extend_from_slice(line);
            }
        }
        if let Some(n) = name.take() {
            contigs.insert(n, seq);
        }
        Ok(Self { contigs })
    }

    /// The reference base at 0-based `pos` on `contig`, or `b'N'` when off the
    /// end / contig absent (matching htslib's `pos < ref_len ? ref[pos] : 'N'`).
    fn base(&self, contig: &str, pos: i64) -> u8 {
        self.contigs
            .get(contig)
            .and_then(|s| usize::try_from(pos).ok().and_then(|p| s.get(p)).copied())
            .unwrap_or(b'N')
    }
}

#[allow(clippy::too_many_arguments)]
pub fn mpileup(
    input: &Path,
    fasta: Option<&Path>,
    output: &mut dyn Write,
    opts: &MpileupOpts,
    workers: NonZero<usize>,
) -> Result<MpileupStats> {
    if fasta.is_some() && !opts.no_baq {
        return Err(RsomicsError::InvalidInput(
            "reference-aware mpileup with BAQ (the samtools default for -f) is not implemented; \
             pass -B to disable BAQ for byte-exact samtools-compatible output"
                .into(),
        ));
    }

    let reference = fasta.map(Reference::load).transpose()?;

    let mut reader = rsomics_bamio::open_with_workers(input, workers)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;
    let ref_names: Vec<String> = header
        .reference_sequences()
        .keys()
        .map(ToString::to_string)
        .collect();
    let ref_lens: Vec<i64> = header
        .reference_sequences()
        .values()
        .map(|s| i64::try_from(usize::from(s.length())).unwrap_or(i64::MAX))
        .collect();

    let engine_opts = PileupOpts {
        smart_overlaps: !opts.no_overlaps,
        no_orphan: !opts.no_orphan_filter,
        min_mapq: opts.min_mapq,
        rflag_filter: opts.rflag_filter,
        rflag_require: opts.rflag_require,
    };
    let mut engine = PileupEngine::new(engine_opts);

    let mut out = BufWriter::with_capacity(256 * 1024, output);
    let mut line: Vec<u8> = Vec::with_capacity(256);
    let mut stats = MpileupStats::default();

    // `-a`/`-aa` fills gaps between emitted columns with empty-coverage lines.
    // Track the last emitted (tid, pos) so the gap-filler runs in coordinate
    // order, mirroring bam_plcmd.c's last_tid/last_pos walk.
    let mut last_tid: i32 = -1;
    let mut last_pos: i64 = -1;

    let reader_mut = reader.get_mut();
    let mut record = RawRecord::default();

    rsomics_pileup::run(
        &mut engine,
        || -> Result<Option<RawRecord>> {
            let n = rsomics_bamio::raw::read_record(reader_mut, &mut record)?;
            if n == 0 {
                Ok(None)
            } else {
                Ok(Some(record.clone()))
            }
        },
        |col: &Column| -> Result<()> {
            if opts.output_all > 0 {
                emit_gaps(
                    &mut out,
                    &mut line,
                    &ref_names,
                    &ref_lens,
                    reference.as_ref(),
                    opts,
                    col.tid,
                    col.pos,
                    &mut last_tid,
                    &mut last_pos,
                    &mut stats,
                )?;
            }
            encode_column(&mut line, &ref_names, reference.as_ref(), opts, col);
            out.write_all(&line).map_err(RsomicsError::Io)?;
            stats.positions += 1;
            last_tid = col.tid;
            last_pos = col.pos;
            Ok(())
        },
    )?;

    // `-a`/`-aa`: emit trailing empty positions after the last column
    // (bam_plcmd.c:880). Single `-a` fills the last covered reference to its
    // full length and stops; `-aa` continues through every remaining reference.
    if opts.output_all > 0 {
        emit_trailing(
            &mut out,
            &mut line,
            &ref_names,
            &ref_lens,
            reference.as_ref(),
            opts,
            &mut last_tid,
            &mut last_pos,
            &mut stats,
        )?;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(stats)
}

/// Encode one pileup column into `line` (cleared first), terminated by `\n`.
/// Layout (5-column default): `chrom \t pos1 \t refbase \t depth \t bases \t quals`.
fn encode_column(
    line: &mut Vec<u8>,
    ref_names: &[String],
    reference: Option<&Reference>,
    opts: &MpileupOpts,
    col: &Column,
) {
    line.clear();
    let chrom = ref_names.get(col.tid as usize).map_or("*", String::as_str);
    let ref_base = reference.map_or(b'N', |r| r.base(chrom, col.pos));

    line.extend_from_slice(chrom.as_bytes());
    line.push(b'\t');
    push_int(line, col.pos + 1);
    line.push(b'\t');
    line.push(ref_base);

    let mut bases: Vec<u8> = Vec::with_capacity(col.reads.len() * 2);
    let mut quals: Vec<u8> = Vec::with_capacity(col.reads.len());
    let mut depth = 0u32;

    for read in &col.reads {
        let rec = col.records[read.read_index];
        // min_baseQ gates per base (bam_plcmd.c: a base below the threshold is
        // not counted and contributes neither a base nor a quality char). At a
        // deletion qpos points at the last real base; htslib uses qual 0 when
        // qpos is past the sequence, which always passes a 0 threshold and fails
        // any positive one — replicate via the raw qual lookup.
        let q = base_qual(rec, read);
        if q < opts.min_baseq {
            continue;
        }
        encode_base(&mut bases, rec, read, ref_base, reference, chrom, col.pos);
        quals.push(qual_char(q));
        depth += 1;
    }

    line.push(b'\t');
    push_int(line, i64::from(depth));
    line.push(b'\t');
    if bases.is_empty() {
        line.push(b'*');
    } else {
        line.extend_from_slice(&bases);
    }
    line.push(b'\t');
    if quals.is_empty() {
        line.push(b'*');
    } else {
        line.extend_from_slice(&quals);
    }
    line.push(b'\n');
}

/// The base quality samtools tests against `min_baseQ` for this read at this
/// column (`bam_plcmd.c`: `qpos < l_qseq ? qual[qpos] : 0`).
fn base_qual(rec: &RawRecord, read: &PileupRead) -> u8 {
    let quals = rec.quality_scores();
    if read.qpos < quals.len() {
        quals[read.qpos]
    } else {
        0
    }
}

/// Phred quality → printable char (`q + 33`, capped at 126), `bam_plcmd.c`.
fn qual_char(q: u8) -> u8 {
    let c = u16::from(q) + 33;
    if c > 126 { 126 } else { c as u8 }
}

/// Encode a single read's contribution to the bases column (`pileup_seq`):
/// optional `^<mapqchar>` head marker, the base (or `*`/`<`/`>` for a deletion /
/// ref-skip), optional `+N<seq>` / `-N<refseq>` indel, optional `$` tail marker.
fn encode_base(
    out: &mut Vec<u8>,
    rec: &RawRecord,
    read: &PileupRead,
    ref_base: u8,
    reference: Option<&Reference>,
    chrom: &str,
    pos: i64,
) {
    let is_rev = rec.flags() & FLAG_REVERSE != 0;

    if read.is_head {
        out.push(b'^');
        let mapq = rec.mapping_quality();
        out.push(if mapq > 93 { 126 } else { mapq + 33 });
    }

    if !read.is_del {
        // seq_nt16 code of this read base; off the end of seq → N (15).
        let mut c = if read.qpos < rec.sequence_len() {
            rec.seq_nibble(read.qpos)
        } else {
            15
        };
        if reference.is_some() {
            let rb = nt16_of_ref(ref_base);
            if c == rb {
                c = 0; // "=", rendered as . or , by the alphabet's 0 slot.
            }
        }
        let table = if is_rev { SEQ_NT_LC } else { SEQ_NT_UC };
        out.push(table[c as usize]);
    } else {
        // Deletion → `*` (`#` with -rev-del, not enabled); ref-skip → `<`/`>`.
        out.push(if read.is_refskip {
            if is_rev { b'<' } else { b'>' }
        } else {
            b'*'
        });
    }

    if read.indel > 0 {
        // Insertion: `+N` then the N inserted read bases (case by strand). The
        // inserted bases start at qpos+1 in the query (pileup_seq /
        // bam_plp_insertion). Insertion bases are read bases, never `=`.
        out.push(b'+');
        push_uint(out, read.indel as u64);
        let del_off = usize::from(read.is_del);
        for j in 1..=read.indel as usize {
            // htslib `bam_plp_insertion`: inserted base index is qpos+j-is_del.
            let qp = read.qpos + j - del_off;
            let base = if qp < rec.sequence_len() {
                SEQ_NT16_STR[rec.seq_nibble(qp) as usize]
            } else {
                b'N'
            };
            out.push(if is_rev {
                base.to_ascii_lowercase()
            } else {
                base
            });
        }
    } else if read.indel < 0 {
        // Deletion: `-N` then the N reference bases following this position
        // (case by strand), or `N` when no reference (pileup_seq's del_len loop:
        // `(ref && pos+j < ref_len) ? ref[pos+j] : 'N'`).
        let del_len = -read.indel;
        out.push(b'-');
        push_uint(out, del_len as u64);
        for j in 1..=del_len {
            let rb = reference.map_or(b'N', |r| r.base(chrom, pos + j));
            out.push(if is_rev {
                rb.to_ascii_lowercase()
            } else {
                rb.to_ascii_uppercase()
            });
        }
    }

    if read.is_tail {
        out.push(b'$');
    }
}

fn push_int(out: &mut Vec<u8>, v: i64) {
    let mut buf = itoa::Buffer::new();
    out.extend_from_slice(buf.format(v).as_bytes());
}

fn push_uint(out: &mut Vec<u8>, v: u64) {
    let mut buf = itoa::Buffer::new();
    out.extend_from_slice(buf.format(v).as_bytes());
}

/// `-a`/`-aa`: emit empty-coverage lines for positions between the last emitted
/// column and the column about to be emitted at `(tid, pos)`.
#[allow(clippy::too_many_arguments)]
fn emit_gaps(
    out: &mut dyn Write,
    line: &mut Vec<u8>,
    ref_names: &[String],
    ref_lens: &[i64],
    reference: Option<&Reference>,
    opts: &MpileupOpts,
    tid: i32,
    pos: i64,
    last_tid: &mut i32,
    last_pos: &mut i64,
    stats: &mut MpileupStats,
) -> Result<()> {
    // Missing portions of previous references (bam_plcmd.c:612). The previous
    // `last_tid` is filled to its full length for any `-a`; single `-a` then
    // advances one reference and stops, `-aa` continues through intermediate
    // empty references too.
    while tid > *last_tid {
        if *last_tid >= 0 {
            let len = ref_lens.get(*last_tid as usize).copied().unwrap_or(0);
            while *last_pos + 1 < len {
                *last_pos += 1;
                emit_empty(out, line, ref_names, reference, *last_tid, *last_pos, stats)?;
            }
        }
        *last_tid += 1;
        *last_pos = -1;
        if opts.output_all < 2 {
            break;
        }
    }
    if *last_tid != tid {
        *last_tid = tid;
        *last_pos = -1;
    }
    // Missing portion of the current reference up to the column (bam_plcmd.c:644).
    while *last_pos + 1 < pos {
        *last_pos += 1;
        emit_empty(out, line, ref_names, reference, tid, *last_pos, stats)?;
    }
    Ok(())
}

/// `-a`/`-aa`: emit trailing empty positions after the last covered column
/// (bam_plcmd.c:880-909). Single `-a` fills the last covered reference and stops;
/// `-aa` continues through every remaining reference. On a wholly-empty input
/// `-aa` starts at reference 0.
#[allow(clippy::too_many_arguments)]
fn emit_trailing(
    out: &mut dyn Write,
    line: &mut Vec<u8>,
    ref_names: &[String],
    ref_lens: &[i64],
    reference: Option<&Reference>,
    opts: &MpileupOpts,
    last_tid: &mut i32,
    last_pos: &mut i64,
    stats: &mut MpileupStats,
) -> Result<()> {
    // -aa on a blank file: start from reference 0 (bam_plcmd.c:886).
    if *last_tid < 0 && opts.output_all >= 2 {
        *last_tid = 0;
    }
    let n = ref_names.len() as i32;
    while *last_tid >= 0 && *last_tid < n {
        let len = ref_lens.get(*last_tid as usize).copied().unwrap_or(0);
        while *last_pos + 1 < len {
            *last_pos += 1;
            emit_empty(out, line, ref_names, reference, *last_tid, *last_pos, stats)?;
        }
        *last_tid += 1;
        *last_pos = -1;
        if opts.output_all < 2 {
            break;
        }
    }
    Ok(())
}

fn emit_empty(
    out: &mut dyn Write,
    line: &mut Vec<u8>,
    ref_names: &[String],
    reference: Option<&Reference>,
    tid: i32,
    pos: i64,
    stats: &mut MpileupStats,
) -> Result<()> {
    line.clear();
    let chrom = ref_names.get(tid as usize).map_or("*", String::as_str);
    let ref_base = reference.map_or(b'N', |r| r.base(chrom, pos));
    line.extend_from_slice(chrom.as_bytes());
    line.push(b'\t');
    push_int(line, pos + 1);
    line.push(b'\t');
    line.push(ref_base);
    line.extend_from_slice(b"\t0\t*\t*\n");
    out.write_all(line).map_err(RsomicsError::Io)?;
    stats.positions += 1;
    Ok(())
}
