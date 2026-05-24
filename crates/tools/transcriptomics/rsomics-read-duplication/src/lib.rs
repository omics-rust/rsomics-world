//! Sequence-based and position-based read duplication rate.
//!
//! Mirrors the algorithm of `RSeQC` `read_duplication.py` (LGPL-2.1+):
//! - Iterates all mapped reads; skips unmapped, QC-fail, and MAPQ < threshold.
//! - Sequence-based: groups reads by exact (uppercased) sequence; builds
//!   occurrence-count histogram.
//! - Position-based: groups reads by `chrom:start:exon_boundary` key derived
//!   from CIGAR exon blocks; builds occurrence-count histogram.
//! - Writes `<prefix>.seq.DupRate.xls` and `<prefix>.pos.DupRate.xls`, each
//!   a two-column TSV (`Occurrence\tUniqReadNumber`) sorted by occurrence.
//!
//! ## Origin
//!
//! This crate is an independent Rust reimplementation of `RSeQC`
//! `read_duplication.py` based on:
//! - The published method: Wang et al. 2012 <https://doi.org/10.1093/bioinformatics/bts356>
//! - The public SAM/BAM format specification
//! - Reading the LGPL-2.1+ `RSeQC` 5.0.4 source (`SAM.py::readDupRate`,
//!   `bam_cigar.py::fetch_exon`) to derive exact key semantics and
//!   filter logic (LGPL allows reading; implementation is independent Rust)
//! - Black-box behaviour testing against `RSeQC` 5.0.4
//!
//! License: MIT OR Apache-2.0.
//! Upstream credit: `RSeQC` <https://rseqc.sourceforge.net/> (LGPL-2.1+).

use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::num::NonZero;
use std::path::Path;

use rsomics_bamio::raw::{self, RawRecord};
use rsomics_common::{Result, RsomicsError};

// SAM flag bits used by readDupRate.
const FLAG_UNMAPPED: u16 = 0x0004;
const FLAG_QCFAIL: u16 = 0x0200;

// BAM CIGAR op codes relevant to exon boundary calculation.
const OP_MATCH: u8 = 0; // M — consumed from both query and reference
const OP_DEL: u8 = 2; // D — deletion from reference (advances ref pos)
const OP_REF_SKIP: u8 = 3; // N — intron / reference skip (advances ref pos)
const OP_SOFT_CLIP: u8 = 4; // S — soft clipping (advances ref pos in fetch_exon)

/// Lookup table: BAM 4-bit base encoding → ASCII byte (upper-case).
///
/// Encoding: 0=`=`, 1=A, 2=C, 3=M, 4=G, 5=R, 6=S, 7=V, 8=T, 9=W, 10=Y, 11=H, 12=K, 13=D, 14=B, 15=N.
const NIBBLE_TO_BASE: [u8; 16] = *b"=ACMGRSVTWYHKDBN";

/// Decode the BAM nibble-packed sequence into an ASCII uppercase `String`.
fn decode_seq(rec: &RawRecord) -> String {
    let n = rec.sequence_len();
    let mut out = vec![0u8; n];
    for (i, b) in out.iter_mut().enumerate() {
        *b = NIBBLE_TO_BASE[rec.seq_nibble(i) as usize];
    }
    // All values come from NIBBLE_TO_BASE which is pure ASCII; unwrap is infallible.
    String::from_utf8(out).unwrap()
}

/// Mirrors `bam_cigar.fetch_exon`: returns exon blocks as `(start, end)` pairs.
///
/// `st` is the 0-based alignment start. Soft clips advance `chrom_st` (matching
/// `RSeQC` source `fetch_exon` lines `elif c==4: chrom_st += s`).
fn fetch_exon(st: i64, cigar_ops: impl Iterator<Item = (u8, u32)>) -> Vec<(i64, i64)> {
    let mut blocks = Vec::new();
    let mut chrom_st = st;
    for (op, len) in cigar_ops {
        let len = i64::from(len);
        match op {
            OP_MATCH => {
                blocks.push((chrom_st, chrom_st + len));
                chrom_st += len;
            }
            OP_DEL | OP_REF_SKIP | OP_SOFT_CLIP => {
                chrom_st += len;
            }
            _ => {}
        }
    }
    blocks
}

/// Build the position key matching `RSeQC` `readDupRate`:
/// `"{chrom}:{hit_st}:{s1}-{e1}:{s2}-{e2}:..."`.
fn pos_key(chrom: &str, hit_st: i64, cigar_ops: impl Iterator<Item = (u8, u32)>) -> String {
    let blocks = fetch_exon(hit_st, cigar_ops);
    let mut key = format!("{chrom}:{hit_st}:");
    for (s, e) in blocks {
        write!(key, "{s}-{e}:").unwrap();
    }
    key
}

/// Occurrence-count histograms for sequence-based and position-based duplication.
pub struct DupHistograms {
    /// Occurrence count → number of distinct sequences with that occurrence.
    pub seq: HashMap<u64, u64>,
    /// Occurrence count → number of distinct position keys with that occurrence.
    pub pos: HashMap<u64, u64>,
}

/// Scan `bam_path` and compute duplication histograms.
///
/// Filters applied (matching `RSeQC` `readDupRate`):
/// - Skip unmapped reads (FLAG 0x0004).
/// - Skip QC-fail reads (FLAG 0x0200).
/// - Skip reads with MAPQ < `mapq_cut`.
///
/// No secondary or PCR-duplicate flag filter — `readDupRate` does not apply those.
pub fn compute_duplication(
    bam_path: &Path,
    mapq_cut: u8,
    workers: NonZero<usize>,
) -> Result<DupHistograms> {
    let mut reader = rsomics_bamio::open_with_workers(bam_path, workers)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let ref_names: Vec<String> = header
        .reference_sequences()
        .keys()
        .map(|k| String::from_utf8_lossy(k.as_ref()).into_owned())
        .collect();

    let inner = reader.get_mut();
    let mut rec = RawRecord::default();

    let mut seq_dup: HashMap<String, u64> = HashMap::new();
    let mut pos_dup: HashMap<String, u64> = HashMap::new();

    loop {
        let n = raw::read_record(inner, &mut rec)?;
        if n == 0 {
            break;
        }

        let flags = rec.flags();
        if flags & (FLAG_UNMAPPED | FLAG_QCFAIL) != 0 {
            continue;
        }
        if rec.mapping_quality() < mapq_cut {
            continue;
        }

        // Sequence-based: key is the full uppercased read sequence.
        let seq = decode_seq(&rec);
        *seq_dup.entry(seq).or_insert(0) += 1;

        // Position-based: key encodes chrom, start, and CIGAR exon blocks.
        let tid = rec.reference_sequence_id();
        if tid < 0 {
            continue;
        }
        #[allow(clippy::cast_sign_loss)]
        let Some(chrom) = ref_names.get(tid as usize) else {
            continue;
        };
        let hit_st = i64::from(rec.alignment_start());
        let key = pos_key(chrom, hit_st, rec.cigar_ops());
        *pos_dup.entry(key).or_insert(0) += 1;
    }

    // Invert: occurrence count → number of distinct sequences/positions.
    let mut seq_hist: HashMap<u64, u64> = HashMap::new();
    for &count in seq_dup.values() {
        *seq_hist.entry(count).or_insert(0) += 1;
    }
    let mut pos_hist: HashMap<u64, u64> = HashMap::new();
    for &count in pos_dup.values() {
        *pos_hist.entry(count).or_insert(0) += 1;
    }

    Ok(DupHistograms {
        seq: seq_hist,
        pos: pos_hist,
    })
}

/// Write a duplication histogram as a two-column TSV matching `RSeQC`'s `.xls` format.
///
/// Header line: `Occurrence\tUniqReadNumber`. Rows sorted by occurrence ascending.
pub fn write_xls<S: std::hash::BuildHasher>(
    hist: &HashMap<u64, u64, S>,
    path: &Path,
) -> Result<()> {
    let f = File::create(path).map_err(RsomicsError::Io)?;
    let mut w = BufWriter::new(f);
    writeln!(w, "Occurrence\tUniqReadNumber").map_err(RsomicsError::Io)?;
    let mut keys: Vec<u64> = hist.keys().copied().collect();
    keys.sort_unstable();
    for k in keys {
        writeln!(w, "{k}\t{}", hist[&k]).map_err(RsomicsError::Io)?;
    }
    Ok(())
}

/// Run the full duplication analysis and write output files.
pub fn run_duplication(
    bam_path: &Path,
    out_prefix: &Path,
    mapq_cut: u8,
    workers: NonZero<usize>,
) -> Result<DupHistograms> {
    eprintln!("Load BAM file ...");
    let hists = compute_duplication(bam_path, mapq_cut, workers)?;
    eprintln!("Done");

    let prefix_str = out_prefix
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let dir = out_prefix.parent().unwrap_or(Path::new("."));

    eprintln!("report duplicate rate based on sequence ...");
    let seq_path = dir.join(format!("{prefix_str}.seq.DupRate.xls"));
    write_xls(&hists.seq, &seq_path)?;

    eprintln!("report duplicate rate based on mapping ...");
    let pos_path = dir.join(format!("{prefix_str}.pos.DupRate.xls"));
    write_xls(&hists.pos, &pos_path)?;

    Ok(hists)
}
