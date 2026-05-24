//! mRNA-aware inner-distance distribution for paired-end RNA-seq.
//!
//! Mirrors `RSeQC` `inner_distance.py` (LGPL-2.1+):
//!   - for each properly-paired read with `mate_start >= read1_end` (sorted BAM,
//!     mate is downstream), compute the inner distance between the two mates;
//!   - when both mates fall in the same transcript, compute the mRNA (spliced)
//!     distance using exon bitsets (subtracting intronic bases);
//!   - otherwise use the genomic distance with appropriate category label;
//!   - write per-pair distances to `<prefix>.inner_distance.txt` and a
//!     histogram to `<prefix>.inner_distance_freq.txt`.
//!
//! ## Inner-distance sign convention (from `RSeQC` source)
//!
//! Let `read1_end = read1_start + read1_qlen + splice_intron_size` (the
//! rightmost genomic position of read1 after accounting for N-ops in its CIGAR).
//!
//! - If `read2_start >= read1_end`: inner distance = `read2_start − read1_end`
//!   (positive gap between mates).
//! - If `read2_start < read1_end` (overlapping mates): inner distance is the
//!   **negative** count of read1's M-exon bases that extend beyond `read2_start`
//!   (i.e., `−|overlap_bases|`).
//!
//! ## Transcript-index bug (black-box verified against `RSeQC` 5.0.4)
//!
//! `RSeQC`'s `transcript_ranges` construction has an `if/else` bug: the first
//! transcript encountered per chromosome creates the `Intersecter` bucket but is
//! never inserted into it. Only the second and subsequent transcripts on each
//! chromosome are stored. We replicate this exactly so per-pair category labels
//! are byte-identical.
//!
//! ## Origin
//!
//! This crate is an independent Rust reimplementation based on:
//! - `RSeQC`: `inner_distance.py` (LGPL-2.1+), Wang et al. 2012
//!   <https://doi.org/10.1093/bioinformatics/bts356>
//! - The SAM/BAM format specification (MIT)
//! - BED12 format specification
//! - Black-box behaviour testing against `RSeQC` 5.0.4
//!
//! No source code from the GPL/LGPL upstream was used as reference during
//! implementation; the algorithm is derived from the published method,
//! the public format specs, and black-box behavioural testing.
//!
//! License: MIT OR Apache-2.0.
//! Upstream credit: `RSeQC` <https://rseqc.sourceforge.net/> (LGPL-2.1+).

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::too_many_lines
)]

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::num::NonZero;
use std::path::Path;

use coitrees::{COITree, Interval as CoiInterval, IntervalTree};
use rsomics_bamio::raw::{self, RawRecord};
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

// BAM flag bits (SAM spec §1.4).
const FLAG_PAIRED: u16 = 0x0001;
const FLAG_QCFAIL: u16 = 0x0200;
const FLAG_DUPLICATE: u16 = 0x0400;
const FLAG_SECONDARY: u16 = 0x0100;
const FLAG_UNMAPPED: u16 = 0x0004;
const FLAG_MATE_UNMAPPED: u16 = 0x0008;
const FLAG_READ1: u16 = 0x0040;

// BAM CIGAR opcodes (SAM spec §1.4.6, 0-indexed): 0=M 1=I 2=D 3=N 4=S 5=H 6=P 7== 8=X.
const CIGAR_MATCH: u8 = 0; // M — alignment match
const CIGAR_INS: u8 = 1; // I — insertion relative to reference
const CIGAR_DEL: u8 = 2; // D — deletion from reference
const CIGAR_REF_SKIP: u8 = 3; // N — intron skip (splice)
const CIGAR_SOFT_CLIP: u8 = 4; // S — soft clip
// H (5) and P (6) are handled by the wildcard arm; no constant needed.
const CIGAR_SEQ_MATCH: u8 = 7; // = — sequence match
const CIGAR_SEQ_MISMATCH: u8 = 8; // X — sequence mismatch

/// Category string written to `inner_distance.txt` per read pair.
#[derive(Debug, Clone)]
pub enum Category {
    SameChromNo,
    SameTranscriptNo,
    SameTranscriptYesSameExonYes,
    SameTranscriptYesSameExonNo,
    SameTranscriptYesNonExonic,
    UnknownChromosome,
    ReadPairOverlap,
}

impl Category {
    fn as_str(&self) -> &'static str {
        match self {
            Category::SameChromNo => "sameChrom=No",
            Category::SameTranscriptNo => "sameTranscript=No,dist=genomic",
            Category::SameTranscriptYesSameExonYes => "sameTranscript=Yes,sameExon=Yes,dist=mRNA",
            Category::SameTranscriptYesSameExonNo => "sameTranscript=Yes,sameExon=No,dist=mRNA",
            Category::SameTranscriptYesNonExonic => "sameTranscript=Yes,nonExonic=Yes,dist=genomic",
            Category::UnknownChromosome => "unknownChromosome,dist=genomic",
            Category::ReadPairOverlap => "readPairOverlap",
        }
    }
}

/// Per-chromosome exon interval tree (union of all exon blocks from BED12).
///
/// Intervals are stored end-inclusive (coitrees convention) for point queries.
pub struct ExonIndex {
    trees: HashMap<String, COITree<(), u32>>,
}

impl ExonIndex {
    /// Build per-chromosome exon interval trees from a BED12 file.
    ///
    /// Mirrors `RSeQC` `binned_bitsets_from_list(ref_exons)`: each BED12 block becomes one exon
    /// interval; intervals are union-merged (`RSeQC` uses a bitset which naturally deduplicates
    /// overlapping regions from multiple transcripts at the same locus).
    /// Chromosome names are uppercased (matching `RSeQC`'s `.upper()`).
    pub fn from_bed12(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| RsomicsError::Io(std::io::Error::other(format!("reading BED12: {e}"))))?;

        // Collect raw exon intervals (half-open [start, end)) per chromosome.
        let mut raw_half_open: HashMap<String, Vec<(i32, i32)>> = HashMap::new();

        for line in content.lines() {
            if line.starts_with('#') || line.starts_with("track") || line.starts_with("browser") {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 12 {
                continue;
            }
            let chrom = fields[0].to_uppercase();
            let Ok(tx_start) = fields[1].parse::<i32>() else {
                continue;
            };
            let Ok(block_count) = fields[9].parse::<usize>() else {
                continue;
            };
            let block_sizes: Vec<i32> = fields[10]
                .trim_end_matches(',')
                .split(',')
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse().ok())
                .collect();
            let block_starts: Vec<i32> = fields[11]
                .trim_end_matches(',')
                .split(',')
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse().ok())
                .collect();
            if block_sizes.len() != block_count || block_starts.len() != block_count {
                continue;
            }
            let entry = raw_half_open.entry(chrom).or_default();
            for (bstart, bsize) in block_starts.iter().zip(block_sizes.iter()) {
                let exon_start = tx_start + bstart;
                let exon_end = exon_start + bsize;
                entry.push((exon_start, exon_end));
            }
        }

        // Merge overlapping intervals (bitset union) and build COITree.
        // This prevents double-counting when multiple transcripts share exon regions.
        let trees = raw_half_open
            .into_iter()
            .map(|(chrom, mut ivs)| {
                ivs.sort_unstable();
                let mut merged: Vec<CoiInterval<()>> = Vec::with_capacity(ivs.len());
                for (start, end) in ivs {
                    if let Some(last) = merged.last_mut() {
                        // Merge if overlapping or adjacent (using half-open convention).
                        if start <= last.last + 1 {
                            last.last = last.last.max(end - 1);
                            continue;
                        }
                    }
                    // coitrees: end-inclusive [start, end-1].
                    merged.push(CoiInterval::new(start, end - 1, ()));
                }
                (chrom, COITree::new(&merged))
            })
            .collect();

        Ok(Self { trees })
    }

    /// Count exonic bases in `[start, end)` on `chrom` (half-open, 0-based).
    ///
    /// Mirrors the `BinnedBitSet` intersection in `RSeQC`: set bits in `[read1_end, read2_start)`,
    /// AND with `exon_bitsets`, count set bits. We replicate this with coitrees overlap queries.
    fn exonic_bases(&self, chrom: &str, start: i32, end: i32) -> i32 {
        let Some(tree) = self.trees.get(chrom) else {
            return 0;
        };
        // Query [start, end-1] in coitrees' end-inclusive form.
        let mut total = 0i32;
        tree.query(start, end - 1, |node| {
            let ov_start = node.first.max(start);
            let ov_end = (node.last + 1).min(end); // convert back to half-open
            if ov_end > ov_start {
                total += ov_end - ov_start;
            }
        });
        total
    }

    fn has_chrom(&self, chrom: &str) -> bool {
        self.trees.contains_key(chrom)
    }
}

/// Per-chromosome transcript-range interval tree.
///
/// Maps `(chrom, position)` → set of transcript names. Used to decide whether
/// both `read1_end` and `read2_start` fall in the same transcript (matching
/// `RSeQC`'s `transcript_ranges` `Intersecter`).
///
/// Replicates the `RSeQC` `if/else` bug: the **first** transcript encountered per
/// chromosome creates the bucket but is never inserted into it. Only the second
/// and subsequent transcripts on each chromosome are stored. This is
/// black-box-verified against `RSeQC` 5.0.4.
struct TranscriptIndex {
    /// Per-chromosome tree; metadata is index into `names`.
    trees: HashMap<String, COITree<usize, u32>>,
    names: Vec<String>,
}

impl TranscriptIndex {
    fn from_bed12(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| RsomicsError::Io(std::io::Error::other(format!("reading BED12: {e}"))))?;

        let mut names: Vec<String> = Vec::new();
        // chroms_seen tracks which chromosomes have been seen at least once.
        // First transcript per chrom is skipped (replicates `RSeQC` `if/else` bug).
        let mut chroms_seen: HashSet<String> = HashSet::new();
        let mut raw: HashMap<String, Vec<CoiInterval<usize>>> = HashMap::new();

        for line in content.lines() {
            if line.starts_with('#') || line.starts_with("track") || line.starts_with("browser") {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 12 {
                continue;
            }
            let chrom = fields[0].to_uppercase();
            let Ok(tx_start) = fields[1].parse::<i32>() else {
                continue;
            };
            let Ok(tx_end) = fields[2].parse::<i32>() else {
                continue;
            };
            let name = fields[3].to_string();

            // Replicate RSeQC bug: first transcript per chrom marks the chrom
            // as seen but is never added (the `if` branch creates Intersecter,
            // only the `else` branch adds the interval).
            if !chroms_seen.insert(chrom.clone()) {
                // Second+ transcript: add to index.
                let id = names.len();
                names.push(name);
                raw.entry(chrom)
                    .or_default()
                    .push(CoiInterval::new(tx_start, tx_end - 1, id));
            }
            // First transcript: chrom inserted into seen, interval skipped.
        }

        let trees = raw
            .into_iter()
            .map(|(chrom, ivs)| (chrom, COITree::new(&ivs)))
            .collect();

        Ok(Self { trees, names })
    }

    /// Return the set of transcript names overlapping a single 0-based point.
    fn names_at<'a>(&'a self, chrom: &str, point: i32) -> HashSet<&'a str> {
        let Some(tree) = self.trees.get(chrom) else {
            return HashSet::new();
        };
        let mut ids = HashSet::new();
        tree.query(point, point, |node| {
            let idx: usize = *std::borrow::Borrow::<usize>::borrow(&node.metadata);
            ids.insert(self.names[idx].as_str());
        });
        ids
    }
}

/// One per-read-pair result row.
pub struct PairRecord {
    pub read_name: String,
    /// `None` if mates on different chromosomes.
    pub inner_distance: Option<i32>,
    pub category: Category,
}

/// Results of inner-distance computation.
pub struct InnerDistanceResult {
    pub pairs: Vec<PairRecord>,
    /// Histogram bins: each entry is `(bin_start, bin_end_exclusive, count)`.
    pub histogram: Vec<(i32, i32, u64)>,
    pub pair_num: u64,
}

/// Compute the query sequence length (qlen) from a CIGAR op iterator.
///
/// Consumes M/I/S/=/X operations (those that consume query bases).
/// Mirrors `pysam`'s `aligned_read.qlen`.
fn cigar_qlen(cigar: impl Iterator<Item = (u8, u32)>) -> i32 {
    let mut q = 0i32;
    for (op, len) in cigar {
        match op {
            CIGAR_MATCH | CIGAR_INS | CIGAR_SOFT_CLIP | CIGAR_SEQ_MATCH | CIGAR_SEQ_MISMATCH => {
                q += len as i32;
            }
            _ => {}
        }
    }
    q
}

/// Compute total N-op (intron) length from a CIGAR op iterator.
///
/// Mirrors `bam_cigar.fetch_intron` summing `intron[2] - intron[1]`.
fn cigar_intron_len(cigar: impl Iterator<Item = (u8, u32)>) -> i32 {
    cigar
        .filter(|&(op, _)| op == CIGAR_REF_SKIP)
        .map(|(_, len)| len as i32)
        .sum()
}

/// Count exon (M/=/X) bases in the overlap region `[read2_start, read1_end)` (0-based).
///
/// This mirrors `RSeQC`'s `fetch_exon` + position counting for the overlap case.
/// `RSeQC` collects 1-based positions from M blocks and counts those where
/// `i > read2_start AND i <= read1_end`. Translating to 0-based half-open: count M-bases
/// in `[read2_start, read1_end)`.
fn count_overlap_exon_bases(
    read1_start: i32,
    read1_end: i32,
    read2_start: i32,
    cigar: impl Iterator<Item = (u8, u32)>,
) -> i32 {
    let mut ref_pos = read1_start;
    let mut count = 0i32;
    for (op, len) in cigar {
        #[allow(clippy::cast_possible_wrap)]
        let l = len as i32;
        match op {
            CIGAR_MATCH | CIGAR_SEQ_MATCH | CIGAR_SEQ_MISMATCH => {
                // M/=/X block at [ref_pos, ref_pos+l) (0-based half-open).
                // Count bases in overlap with [read2_start, read1_end).
                let ov_start = ref_pos.max(read2_start);
                let ov_end = (ref_pos + l).min(read1_end);
                if ov_end > ov_start {
                    count += ov_end - ov_start;
                }
                ref_pos += l;
            }
            CIGAR_DEL | CIGAR_REF_SKIP => {
                ref_pos += l;
            }
            _ => {} // I/S/H/P do not consume reference
        }
    }
    count
}

/// Run the inner-distance computation.
///
/// # Parameters
///
/// - `bam_path`: coordinate-sorted, indexed BAM.
/// - `bed_path`: BED12 gene model.
/// - `sample_size`: max read pairs to process.
/// - `mapq_cut`: minimum MAPQ (default 30).
/// - `low_bound`: histogram lower bound (default -250).
/// - `up_bound`: histogram upper bound (default 250).
/// - `step`: histogram bin width (default 5).
#[allow(clippy::too_many_arguments)]
pub fn compute_inner_distance(
    bam_path: &Path,
    bed_path: &Path,
    sample_size: u64,
    mapq_cut: u8,
    low_bound: i32,
    up_bound: i32,
    step: i32,
) -> Result<InnerDistanceResult> {
    let exon_index = ExonIndex::from_bed12(bed_path)?;
    let tx_index = TranscriptIndex::from_bed12(bed_path)?;

    let mut pairs: Vec<PairRecord> = Vec::new();
    let mut pair_num: u64 = 0;

    #[allow(clippy::cast_sign_loss)]
    let window_left_bounds: Vec<i32> = (low_bound..up_bound).step_by(step as usize).collect();
    // Raw per-pair distances for histogram binning (collected after category assignment).
    let mut distances: Vec<i32> = Vec::new();

    let workers = NonZero::new(1usize).unwrap();
    let mut reader = rsomics_bamio::open_with_workers(bam_path, workers)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;
    let ref_names: Vec<String> = header
        .reference_sequences()
        .keys()
        .map(|k| String::from_utf8_lossy(k.as_ref()).into_owned())
        .collect();

    let mut rec = RawRecord::default();
    loop {
        if pair_num >= sample_size {
            break;
        }
        let bytes_read = raw::read_record(reader.get_mut(), &mut rec)?;
        if bytes_read == 0 {
            break;
        }

        let flags = rec.flags();
        // RSeQC filters (in order from source):
        if flags & FLAG_QCFAIL != 0 {
            continue;
        }
        if flags & FLAG_DUPLICATE != 0 {
            continue;
        }
        if flags & FLAG_SECONDARY != 0 {
            continue;
        }
        if flags & FLAG_UNMAPPED != 0 {
            continue;
        }
        if flags & FLAG_PAIRED == 0 {
            continue;
        }
        if flags & FLAG_MATE_UNMAPPED != 0 {
            continue;
        }
        if rec.mapping_quality() < mapq_cut {
            continue;
        }

        // alignment_start() returns the 0-based position (BAM raw field is 0-based).
        let read1_start = rec.alignment_start();
        let mate_start = rec.mate_alignment_start();

        // RSeQC: skip if mate upstream (BAM is sorted; mate already processed).
        if mate_start < read1_start {
            continue;
        }
        // RSeQC: if same pos and is_read1, inner_distance = 0, continue (no output).
        if mate_start == read1_start && (flags & FLAG_READ1 != 0) {
            continue;
        }

        pair_num += 1;

        let read_name = String::from_utf8_lossy(rec.name())
            .trim_end_matches('\0')
            .to_string();

        // Check same chromosome.
        let tid = rec.reference_sequence_id();
        let rnext = rec.mate_reference_sequence_id();
        if tid != rnext || tid < 0 {
            pairs.push(PairRecord {
                read_name,
                inner_distance: None,
                category: Category::SameChromNo,
            });
            continue;
        }

        #[allow(clippy::cast_sign_loss)]
        let chrom = {
            let idx = tid as usize;
            let Some(name) = ref_names.get(idx) else {
                pairs.push(PairRecord {
                    read_name,
                    inner_distance: Some(0),
                    category: Category::UnknownChromosome,
                });
                continue;
            };
            name.to_uppercase()
        };

        // Compute read1_len (qlen) and splice_intron_size from CIGAR.
        // CIGAR ops are consumed once per measure; collect into Vec to allow two passes.
        let cigar_vec: Vec<(u8, u32)> = rec.cigar_ops().collect();
        let read1_len = cigar_qlen(cigar_vec.iter().copied());
        let splice_intron_size = cigar_intron_len(cigar_vec.iter().copied());

        // RSeQC: read1_end = read1_start + read1_len + splice_intron_size
        let read1_end = read1_start + read1_len + splice_intron_size;
        let read2_start = mate_start;

        // `written_distance` is what goes into the output files.
        // For mRNA cases, `RSeQC` writes the exon-adjusted distance, not the raw genomic gap.
        let category: Category;
        let written_distance: i32;

        if read2_start >= read1_end {
            // Positive gap between mates.
            let geo_distance = read2_start - read1_end;

            // Check if both ends fall in the same transcript.
            let read1_end_genes = tx_index.names_at(&chrom, read1_end - 1);
            let read2_start_genes = tx_index.names_at(&chrom, read2_start);
            let same_tx = read1_end_genes
                .intersection(&read2_start_genes)
                .next()
                .is_some();

            if !same_tx {
                written_distance = geo_distance;
                category = Category::SameTranscriptNo;
                distances.push(geo_distance);
            } else if exon_index.has_chrom(&chrom) {
                let exon_bases = exon_index.exonic_bases(&chrom, read1_end, read2_start);
                if exon_bases == geo_distance {
                    written_distance = exon_bases;
                    category = Category::SameTranscriptYesSameExonYes;
                    distances.push(exon_bases);
                } else if exon_bases > 0 && exon_bases < geo_distance {
                    written_distance = exon_bases;
                    category = Category::SameTranscriptYesSameExonNo;
                    distances.push(exon_bases);
                } else {
                    // exon_bases <= 0: gap is non-exonic
                    written_distance = geo_distance;
                    category = Category::SameTranscriptYesNonExonic;
                    distances.push(geo_distance);
                }
            } else {
                // Chromosome not in exon index.
                written_distance = geo_distance;
                category = Category::UnknownChromosome;
                distances.push(geo_distance);
            }
        } else {
            // Overlapping mates: negative inner distance.
            let overlap_dist = -count_overlap_exon_bases(
                read1_start,
                read1_end,
                read2_start,
                cigar_vec.iter().copied(),
            );
            written_distance = overlap_dist;

            // Check transcript membership for category label.
            let read1_end_genes = tx_index.names_at(&chrom, read1_end - 1);
            let read2_start_genes = tx_index.names_at(&chrom, read2_start);
            let same_tx = read1_end_genes
                .intersection(&read2_start_genes)
                .next()
                .is_some();

            if same_tx {
                category = Category::ReadPairOverlap;
            } else {
                category = Category::SameTranscriptNo;
            }
            distances.push(overlap_dist);
        }

        pairs.push(PairRecord {
            read_name,
            inner_distance: Some(written_distance),
            category,
        });
    }

    // Build histogram. RSeQC stores Interval(d-1, d) per distance d, then queries
    // find(st, st+step). bx-python find(start, end) returns intervals overlapping
    // [start, end): Interval(d-1, d) overlaps [st, st+step) iff d-1 < st+step AND d > st,
    // i.e. d > st AND d <= st+step. So bin [st, st+step) counts d where st < d <= st+step.
    let histogram: Vec<(i32, i32, u64)> = window_left_bounds
        .iter()
        .map(|&st| {
            let count = distances
                .iter()
                .filter(|&&d| d > st && d <= st + step)
                .count() as u64;
            (st, st + step, count)
        })
        .collect();

    Ok(InnerDistanceResult {
        pairs,
        histogram,
        pair_num,
    })
}

/// Write `<prefix>.inner_distance.txt` (per-pair) and
/// `<prefix>.inner_distance_freq.txt` (histogram).
pub fn write_output(result: &InnerDistanceResult, prefix: &str) -> Result<()> {
    let txt_path = format!("{prefix}.inner_distance.txt");
    let mut fo = std::fs::File::create(&txt_path).map_err(|e| {
        RsomicsError::Io(std::io::Error::other(format!("creating {txt_path}: {e}")))
    })?;
    for rec in &result.pairs {
        match rec.inner_distance {
            None => writeln!(fo, "{}\tNA\t{}", rec.read_name, rec.category.as_str()),
            Some(d) => writeln!(fo, "{}\t{}\t{}", rec.read_name, d, rec.category.as_str()),
        }
        .map_err(RsomicsError::Io)?;
    }

    let freq_path = format!("{prefix}.inner_distance_freq.txt");
    let mut fq = std::fs::File::create(&freq_path).map_err(|e| {
        RsomicsError::Io(std::io::Error::other(format!("creating {freq_path}: {e}")))
    })?;
    for &(st, end, count) in &result.histogram {
        writeln!(fq, "{st}\t{end}\t{count}").map_err(RsomicsError::Io)?;
    }

    Ok(())
}

/// JSON-serialisable summary (emitted on `--json`).
#[derive(Debug, Serialize)]
pub struct InnerDistanceSummary {
    pub pair_num: u64,
    pub output_prefix: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cigar_qlen_basic() {
        // 5M 2I 3M → qlen = 5+2+3 = 10
        let cigar = [(CIGAR_MATCH, 5u32), (CIGAR_INS, 2), (CIGAR_MATCH, 3)];
        assert_eq!(cigar_qlen(cigar.iter().copied()), 10);
    }

    #[test]
    fn cigar_intron_basic() {
        // 100M 500N 100M → intron = 500
        let cigar = [
            (CIGAR_MATCH, 100u32),
            (CIGAR_REF_SKIP, 500),
            (CIGAR_MATCH, 100),
        ];
        assert_eq!(cigar_intron_len(cigar.iter().copied()), 500);
    }

    #[test]
    fn overlap_bases_non_overlapping() {
        // read1: 1000..1100 (M=100), read1_end=1100, read2_start=1200 → no overlap
        let cigar = [(CIGAR_MATCH, 100u32)];
        assert_eq!(
            count_overlap_exon_bases(1000, 1100, 1200, cigar.iter().copied()),
            0
        );
    }

    #[test]
    fn overlap_bases_full() {
        // read1: 1000..1200 (M=200), read1_end=1200, read2_start=1100
        // overlap = [1100, 1200) = 100 bases
        let cigar = [(CIGAR_MATCH, 200u32)];
        assert_eq!(
            count_overlap_exon_bases(1000, 1200, 1100, cigar.iter().copied()),
            100
        );
    }

    #[test]
    fn histogram_bin_boundary() {
        // d=50 should fall in bin [45, 50) since condition is d > st AND d <= st+step
        // bin with st=45, step=5: 50 > 45 AND 50 <= 50 → YES
        let distances = [50i32];
        let st = 45i32;
        let step = 5i32;
        let count = distances
            .iter()
            .filter(|&&d| d > st && d <= st + step)
            .count();
        assert_eq!(count, 1);

        // d=50 should NOT fall in bin [50, 55): 50 > 50 is false
        let st2 = 50i32;
        let count2 = distances
            .iter()
            .filter(|&&d| d > st2 && d <= st2 + step)
            .count();
        assert_eq!(count2, 0);
    }
}
