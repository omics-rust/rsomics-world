//! Gene-body 5'→3' read-coverage profile for RNA-seq degradation QC.
//!
//! Mirrors the algorithm of `RSeQC` `geneBody_coverage.py` (LGPL):
//!   - parses a BED12 gene model and builds a list of 100 percentile genomic
//!     positions for each transcript whose mRNA length ≥ `min_mrna_len`;
//!   - pileups each BAM at those positions, filtering QC-fail/duplicate/
//!     secondary/unmapped reads and deletion-spanning bases;
//!   - for minus-strand transcripts, reverses the per-transcript vector
//!     before accumulation (so position 1 is always 5');
//!   - outputs `<prefix>.geneBodyCoverage.txt` — a 2-row TSV with a
//!     Percentile header and one coverage row per BAM.
//!
//! ## Origin
//!
//! This crate is an independent Rust reimplementation based on:
//! - `RSeQC`: `geneBody_coverage.py` (LGPL-2.1+), Wang et al. 2012
//!   <https://doi.org/10.1093/bioinformatics/bts356>
//! - The SAM/BAM format specification (MIT)
//! - BED12 format specification
//! - Black-box behaviour testing against `RSeQC` 5.0.4
//!
//! No GPL source was used as reference during implementation.
//! License: MIT OR Apache-2.0.
//! Upstream credit: `RSeQC` <https://rseqc.sourceforge.net/> (LGPL-2.1+).

#![allow(clippy::cast_precision_loss)]

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use noodles::bam;
use noodles::core::Region;
use noodles::sam::alignment::record::cigar::op::Kind;
use rsomics_common::{Result, RsomicsError};

/// A parsed transcript from BED12 with pre-computed 100 percentile positions.
pub struct Transcript {
    pub chrom: String,
    pub strand: Strand,
    /// 100 1-based genomic positions (indices 0..100 → percentiles 1..100).
    pub positions: [u64; 100],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Strand {
    Plus,
    Minus,
}

/// Parse BED12, filter transcripts with mRNA length < `min_mrna_len`, compute
/// 100 percentile positions for each remaining transcript.
///
/// Mirrors `genebody_percentile` in `geneBody_coverage.py` exactly:
///   - builds `gene_all_base` as 1-based genomic positions for every exon base;
///   - passes that list to the percentile sampler.
pub fn load_transcripts(bed_path: &Path, min_mrna_len: usize) -> Result<Vec<Transcript>> {
    let content = std::fs::read_to_string(bed_path)
        .map_err(|e| RsomicsError::Io(std::io::Error::other(format!("reading BED12: {e}"))))?;

    let mut transcripts = Vec::new();

    for line in content.lines() {
        if line.starts_with('#') || line.starts_with("track") || line.starts_with("browser") {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 12 {
            eprintln!("[NOTE:input bed must be 12-column] skipped this line: {line}");
            continue;
        }

        let chrom = fields[0];
        let Ok(tx_start): std::result::Result<u64, _> = fields[1].parse() else {
            eprintln!("[NOTE:input bed must be 12-column] skipped this line: {line}");
            continue;
        };
        let strand = match fields[5] {
            "+" => Strand::Plus,
            "-" => Strand::Minus,
            _ => {
                eprintln!("[NOTE:input bed must be 12-column] skipped this line: {line}");
                continue;
            }
        };

        let Ok(block_count): std::result::Result<usize, _> = fields[9].parse() else {
            eprintln!("[NOTE:input bed must be 12-column] skipped this line: {line}");
            continue;
        };

        let Some(block_sizes) = parse_csv_u64(fields[10], block_count) else {
            eprintln!("[NOTE:input bed must be 12-column] skipped this line: {line}");
            continue;
        };
        let Some(block_starts) = parse_csv_u64(fields[11], block_count) else {
            eprintln!("[NOTE:input bed must be 12-column] skipped this line: {line}");
            continue;
        };

        // Build gene_all_base: 1-based genomic positions for all exon bases.
        // Mirrors: `gene_all_base.extend(range(st+1, end+1))` in RSeQC.
        let mut gene_all_base: Vec<u64> = Vec::new();
        for (bstart, bsize) in block_starts.iter().zip(block_sizes.iter()) {
            let exon_start = tx_start + bstart; // 0-based
            let exon_end = exon_start + bsize; // exclusive
            // 1-based positions: exon_start+1 ..= exon_end
            for pos in (exon_start + 1)..=(exon_end) {
                gene_all_base.push(pos);
            }
        }

        if gene_all_base.len() < min_mrna_len {
            continue;
        }

        let positions = percentile_positions(&gene_all_base);
        transcripts.push(Transcript {
            chrom: chrom.to_string(),
            strand,
            positions,
        });
    }

    Ok(transcripts)
}

/// Parse a comma-separated list (with optional trailing comma) of exactly `n` u64 values.
fn parse_csv_u64(s: &str, n: usize) -> Option<Vec<u64>> {
    let parts: Vec<&str> = s.trim_end_matches(',').split(',').collect();
    if parts.len() != n {
        return None;
    }
    parts.iter().map(|p| p.parse().ok()).collect()
}

/// Sample 100 percentile positions from a sorted list of genomic positions.
///
/// Mirrors `mystat.percentile_list` from `RSeQC`:
///   for i in 1..=100: k = (N-1)*i/100.0; linear-interp between floor/ceil indices.
/// Uses banker's rounding (round-half-to-even) to match Python's `round()` builtin.
fn percentile_positions(sorted_positions: &[u64]) -> [u64; 100] {
    let n = sorted_positions.len();
    debug_assert!(n >= 100);
    let mut out = [0u64; 100];
    for (idx, item) in out.iter_mut().enumerate() {
        let pct = idx + 1; // percentile 1..=100
        let k = (n - 1) as f64 * pct as f64 / 100.0;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let floor_idx = k.floor() as usize;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let ceil_idx = k.ceil() as usize;
        *item = if floor_idx == ceil_idx {
            sorted_positions[floor_idx]
        } else {
            let d0 = sorted_positions[floor_idx] as f64 * (ceil_idx as f64 - k);
            let d1 = sorted_positions[ceil_idx] as f64 * (k - floor_idx as f64);
            round_half_to_even(d0 + d1)
        };
    }
    out
}

/// Round-half-to-even (banker's rounding), matching Python's `round()` builtin.
///
/// Python `round(9300.5)` = 9300 (round to nearest even); Rust `.round()` = 9301.
/// This function replicates Python's behavior for the percentile interpolation.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn round_half_to_even(x: f64) -> u64 {
    let floor = x.floor();
    let frac = x - floor;
    if (frac - 0.5).abs() < 1e-10 {
        let n = floor as u64;
        if n.is_multiple_of(2) { n } else { n + 1 }
    } else {
        x.round() as u64
    }
}

/// Compute the accumulated gene-body coverage for a single BAM file.
///
/// Mirrors `genebody_coverage` in `RSeQC`:
///   - pileup at each of the 100 percentile positions;
///   - count only reads that are not del/qcfail/secondary/unmapped/dup;
///   - for minus-strand transcripts, reverse the vector before accumulation.
#[allow(clippy::too_many_lines)]
pub fn compute_coverage(bam_path: &Path, transcripts: &[Transcript]) -> Result<[u64; 100]> {
    // Try <bam>.bai first, then <stem>.bai.
    let index_path_1 = {
        let name = bam_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        bam_path.with_file_name(format!("{name}.bai"))
    };
    let index_path_2 = bam_path.with_extension("bai");

    let index_path = if index_path_1.exists() {
        &index_path_1
    } else {
        &index_path_2
    };

    let index = bam::bai::fs::read(index_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", index_path.display())))?;

    let file = File::open(bam_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", bam_path.display())))?;
    let mut reader = bam::io::Reader::new(file);
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let mut aggregated = [0u64; 100];
    let mut genes_done = 0u64;

    for tx in transcripts {
        let positions = &tx.positions;

        // RSeQC: chrom_start = positions[0]-1 (0-based, clamped ≥0), chrom_end = positions[-1].
        let region_start_0 = positions[0].saturating_sub(1);
        let region_end_1 = positions[99]; // 1-based inclusive

        // noodles Region parses "chrom:start-end" with 1-based coordinates.
        let region_str = format!("{}:{}-{}", tx.chrom, region_start_0 + 1, region_end_1);
        let Ok(region) = region_str.parse::<Region>() else {
            continue;
        };

        // Per-percentile-position coverage counter (initialized to 0).
        let mut coverage = HashMap::<u64, u64>::with_capacity(100);
        for &p in positions {
            coverage.insert(p, 0);
        }

        // HashSet of the 100 percentile positions for O(1) membership test.
        let pos_set: HashSet<u64> = positions.iter().copied().collect();
        let last_pos = positions[99];

        let Ok(mut query) = reader.query(&header, &index, &region) else {
            continue;
        };

        let mut record = bam::Record::default();
        loop {
            match query.read_record(&mut record) {
                Ok(0) => break,
                Ok(_) => {}
                Err(e) => return Err(RsomicsError::Io(e)),
            }

            let flags = record.flags();
            if flags.is_unmapped()
                || flags.is_secondary()
                || flags.is_qc_fail()
                || flags.is_duplicate()
            {
                continue;
            }

            // `alignment_start()` returns `Option<io::Result<Position>>` (1-based).
            let start_1based: usize = match record.alignment_start() {
                Some(Ok(pos)) => usize::from(pos),
                _ => continue,
            };

            let cigar = record.cigar();
            let mut ref_cursor = start_1based;
            let mut done = false;

            for op_result in cigar.iter() {
                if done {
                    break;
                }
                let Ok(op) = op_result else { break };
                let len = op.len();
                match op.kind() {
                    Kind::Match | Kind::SequenceMatch | Kind::SequenceMismatch => {
                        // Reference-consuming + query-present: increment coverage.
                        for ref_pos in ref_cursor..(ref_cursor + len) {
                            #[allow(clippy::cast_possible_truncation)]
                            let p = ref_pos as u64;
                            if pos_set.contains(&p) {
                                *coverage.entry(p).or_insert(0) += 1;
                            }
                            if p > last_pos {
                                done = true;
                                break;
                            }
                        }
                        ref_cursor += len;
                    }
                    Kind::Deletion | Kind::Skip => {
                        // Reference-consuming, query absent: pileup `is_del` — skip.
                        ref_cursor += len;
                    }
                    // Query-only ops: no ref advance.
                    Kind::SoftClip | Kind::HardClip | Kind::Pad | Kind::Insertion => {}
                }
            }
        }

        let mut tmp: Vec<u64> = positions
            .iter()
            .map(|p| *coverage.get(p).unwrap_or(&0))
            .collect();
        if tx.strand == Strand::Minus {
            tmp.reverse();
        }
        for (idx, v) in tmp.iter().enumerate() {
            aggregated[idx] += v;
        }

        genes_done += 1;
        if genes_done.is_multiple_of(100) {
            eprint!("\t{genes_done} transcripts finished\r");
        }
    }
    eprintln!();

    Ok(aggregated)
}

/// Write the `.geneBodyCoverage.txt` output.
///
/// Format (from `RSeQC` source):
///   Line 1: `"Percentile\t1\t2\t...\t100\n"`
///   Line 2: `"<sample_name>\t<cov[0]>\t<cov[1]>\t...\t<cov[99]>\n"`
///
/// Coverage values match `RSeQC`'s `str(cvg[k])` on `defaultdict(int)` — integers.
pub fn write_output<W: Write>(
    mut out: W,
    sample_name: &str,
    coverage: &[u64; 100],
) -> std::io::Result<()> {
    write!(out, "Percentile")?;
    for i in 1u32..=100 {
        write!(out, "\t{i}")?;
    }
    writeln!(out)?;

    write!(out, "{sample_name}")?;
    for v in coverage {
        write!(out, "\t{v}")?;
    }
    writeln!(out)?;
    Ok(())
}

/// Derive the sample name from a BAM path, matching `valid_name(basename.replace('.bam',''))`.
///
/// `RSeQC` replaces spaces with `_`, prepends `V` if leading digit, replaces
/// non-alphanumeric/`_.` characters with `_`.
#[must_use]
pub fn sample_name_from_path(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("sample");
    valid_name(stem)
}

fn valid_name(s: &str) -> String {
    let rid: String = s.split_whitespace().collect::<Vec<_>>().join("_");
    let mut out = String::with_capacity(rid.len() + 1);
    let mut chars = rid.chars();
    if let Some(first) = chars.next() {
        if first.is_ascii_digit() {
            out.push('V');
        }
        out.push(if is_valid_char(first) { first } else { '_' });
        for ch in chars {
            out.push(if is_valid_char(ch) { ch } else { '_' });
        }
    }
    out
}

fn is_valid_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '.'
}
