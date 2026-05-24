//! Infer RNA-seq library strand protocol from a BAM file and a BED12 gene model.
//!
//! Mirrors the algorithm of `RSeQC` `infer_experiment.py` (LGPL):
//!   - reads up to `sample_size` mapped, non-duplicate, non-secondary,
//!     non-QC-fail reads with MAPQ ≥ `mapq_cut`;
//!   - for each read, finds overlapping genes in the BED12 model;
//!   - classifies the read by `(read_id, map_strand, gene_strand)`;
//!   - emits forward (sp1), reverse (sp2), and undetermined fractions.
//!
//! ## Origin
//!
//! This crate is an independent Rust reimplementation based on:
//! - `RSeQC`: `infer_experiment.py` (LGPL-2.1+), Wang et al. 2012
//!   <https://doi.org/10.1093/bioinformatics/bts356>
//! - The SAM/BAM format specification (MIT)
//! - BED12 format specification
//! - Black-box behaviour testing against `RSeQC` 5.0.4
//!
//! License: MIT OR Apache-2.0.
//! Upstream credit: `RSeQC` <https://rseqc.sourceforge.net/> (LGPL-2.1+).

#![allow(clippy::cast_precision_loss)]

use std::collections::HashMap;
use std::io::Write;
use std::num::NonZero;
use std::path::Path;

use coitrees::{COITree, Interval as CoiInterval, IntervalTree};
use rsomics_bamio::raw::{self, RawRecord};
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

// BAM flag bits (from the SAM spec).
const FLAG_PAIRED: u16 = 0x0001;
const FLAG_QCFAIL: u16 = 0x0200;
const FLAG_DUPLICATE: u16 = 0x0400;
const FLAG_SECONDARY: u16 = 0x0100;
const FLAG_UNMAPPED: u16 = 0x0004;
const FLAG_REVERSE: u16 = 0x0010;
const FLAG_READ1: u16 = 0x0040;

/// Strand of a gene ('+' or '-').
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeneStrand {
    Plus,
    Minus,
}

/// Per-chromosome interval tree, mapping genomic positions → gene strands.
///
/// Multiple overlapping genes at the same locus store all their strands;
/// the query returns the set of distinct strands hit.
pub struct GeneIndex {
    /// Per-chromosome `COITree`; metadata is index into `strands`.
    trees: HashMap<String, COITree<usize, u32>>,
    strands: Vec<GeneStrand>,
}

impl GeneIndex {
    /// Parse a BED12 file and build per-chromosome interval trees.
    ///
    /// Lines beginning with `#`, `track`, or `browser` are skipped.
    /// Lines with fewer than 6 fields are skipped with a warning to stderr.
    pub fn from_bed12(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| RsomicsError::Io(std::io::Error::other(format!("reading BED12: {e}"))))?;

        let mut strands: Vec<GeneStrand> = Vec::new();
        let mut raw: HashMap<String, Vec<CoiInterval<usize>>> = HashMap::new();

        for line in content.lines() {
            if line.starts_with('#') || line.starts_with("track") || line.starts_with("browser") {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 6 {
                eprintln!("[NOTE:input bed must be 12-column] skipped this line: {line}");
                continue;
            }
            let chrom = fields[0];
            let Ok(tx_start) = fields[1].parse::<i32>() else {
                eprintln!("[NOTE:input bed must be 12-column] skipped this line: {line}");
                continue;
            };
            let Ok(tx_end) = fields[2].parse::<i32>() else {
                eprintln!("[NOTE:input bed must be 12-column] skipped this line: {line}");
                continue;
            };
            let strand = match fields[5] {
                "+" => GeneStrand::Plus,
                "-" => GeneStrand::Minus,
                _ => {
                    eprintln!("[NOTE:input bed must be 12-column] skipped this line: {line}");
                    continue;
                }
            };

            let id = strands.len();
            strands.push(strand);
            // coitrees uses end-inclusive [first, last] intervals.
            // BED is half-open [tx_start, tx_end), so store [tx_start, tx_end-1].
            raw.entry(chrom.to_string())
                .or_default()
                .push(CoiInterval::new(tx_start, tx_end - 1, id));
        }

        let trees = raw
            .into_iter()
            .map(|(chrom, ivs)| (chrom, COITree::new(&ivs)))
            .collect();

        Ok(Self { trees, strands })
    }

    /// Returns the set of distinct gene strands overlapping `[start, end)`.
    ///
    /// `start` is 0-based, `end` is `start + seq_len` (half-open, BAM convention).
    /// Matching `RSeQC`: `readEnd = readStart + qlen` (query sequence length).
    fn overlapping_strands(&self, chrom: &str, start: i32, end: i32) -> SmallStrandSet {
        let mut set = SmallStrandSet::empty();
        let Some(tree) = self.trees.get(chrom) else {
            return set;
        };
        // Convert half-open [start, end) to coitrees' end-inclusive [start, end-1].
        tree.query(start, end - 1, |node| {
            set.insert(self.strands[*node.metadata]);
        });
        set
    }
}

/// A tiny set of up to two distinct strands (Plus, Minus, or both).
#[derive(Clone, Copy, Default)]
struct SmallStrandSet {
    has_plus: bool,
    has_minus: bool,
}

impl SmallStrandSet {
    fn empty() -> Self {
        Self::default()
    }

    fn insert(&mut self, s: GeneStrand) {
        match s {
            GeneStrand::Plus => self.has_plus = true,
            GeneStrand::Minus => self.has_minus = true,
        }
    }

    fn is_empty(self) -> bool {
        !self.has_plus && !self.has_minus
    }

    /// Returns `Some(strand)` if exactly one strand is present, `None` otherwise.
    fn unique(self) -> Option<GeneStrand> {
        match (self.has_plus, self.has_minus) {
            (true, false) => Some(GeneStrand::Plus),
            (false, true) => Some(GeneStrand::Minus),
            _ => None,
        }
    }
}

/// Whether the BAM contains paired-end or single-end reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Protocol {
    PairEnd,
    SingleEnd,
    /// Both paired and unpaired reads seen — ambiguous input.
    Mixture,
    /// No usable reads reached the gene model.
    Unknown,
}

/// Result of strandedness inference.
#[derive(Debug, Clone, Serialize)]
pub struct StrandednessResult {
    pub protocol: Protocol,
    /// Fraction explained by `"1++,1--,2+-,2-+"` (FR / forward-stranded) or `"++,--"`.
    pub spec1: f64,
    /// Fraction explained by `"1+-,1-+,2++,2--"` (RF / reverse-stranded) or `"+-,-+"`.
    pub spec2: f64,
    /// Fraction that could not be classified.
    pub other: f64,
    /// Total usable reads sampled.
    pub sampled: u64,
}

impl StrandednessResult {
    /// Emit the exact text format `RSeQC` `infer_experiment.py` prints to stdout.
    pub fn write_rseqc<W: Write>(&self, mut out: W) -> std::io::Result<()> {
        match self.protocol {
            Protocol::PairEnd => {
                writeln!(out)?;
                writeln!(out)?;
                writeln!(out, "This is PairEnd Data")?;
                writeln!(
                    out,
                    "Fraction of reads failed to determine: {:.4}",
                    self.other
                )?;
                writeln!(
                    out,
                    "Fraction of reads explained by \"1++,1--,2+-,2-+\": {:.4}",
                    self.spec1
                )?;
                writeln!(
                    out,
                    "Fraction of reads explained by \"1+-,1-+,2++,2--\": {:.4}",
                    self.spec2
                )?;
            }
            Protocol::SingleEnd => {
                writeln!(out)?;
                writeln!(out)?;
                writeln!(out, "This is SingleEnd Data")?;
                writeln!(
                    out,
                    "Fraction of reads failed to determine: {:.4}",
                    self.other
                )?;
                writeln!(
                    out,
                    "Fraction of reads explained by \"++,--\": {:.4}",
                    self.spec1
                )?;
                writeln!(
                    out,
                    "Fraction of reads explained by \"+-,-+\": {:.4}",
                    self.spec2
                )?;
            }
            _ => {
                writeln!(out, "Unknown Data type")?;
            }
        }
        Ok(())
    }
}

/// Infer strandedness from `bam_path` using the gene model at `bed_path`.
///
/// Mirrors `configure_experiment` from `RSeQC`'s `SAM.py`:
/// - iterates forward from the start of the BAM (no index seek);
/// - stops after `sample_size` usable reads (reads that land in a gene);
/// - filters: QC-fail, duplicate, secondary, unmapped, MAPQ < `mapq_cut`.
///
/// `readEnd` is computed as `readStart + seq_len` (query/sequence length), matching
/// `RSeQC` source (`readEnd = readStart + aligned_read.qlen`).
pub fn infer_strandedness(
    bam_path: &Path,
    bed_path: &Path,
    sample_size: u64,
    mapq_cut: u8,
    workers: NonZero<usize>,
) -> Result<StrandednessResult> {
    eprintln!("Reading reference gene model {} ...", bed_path.display());
    let gene_index = GeneIndex::from_bed12(bed_path)?;
    eprintln!("Done");

    eprintln!("Loading SAM/BAM file ...");

    let mut reader = rsomics_bamio::open_with_workers(bam_path, workers)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    // Build tid → chrom name lookup from the header.
    let ref_names: Vec<String> = header
        .reference_sequences()
        .keys()
        .map(|k| String::from_utf8_lossy(k.as_ref()).into_owned())
        .collect();

    let (p_strandness, s_strandness, count) = scan_reads(
        reader.get_mut(),
        &ref_names,
        &gene_index,
        sample_size,
        mapq_cut,
    )?;

    eprintln!("Finished");
    eprintln!("Total {count} usable reads were sampled");

    Ok(compute_result(&p_strandness, &s_strandness, count))
}

/// Counters from the BAM scan: (paired counts, single-end counts, total usable reads).
type ScanCounts = (HashMap<&'static str, u64>, HashMap<&'static str, u64>, u64);

/// Inner read-scanning loop, separated to keep `infer_strandedness` within lint limits.
fn scan_reads<R: std::io::Read>(
    reader: &mut R,
    ref_names: &[String],
    gene_index: &GeneIndex,
    sample_size: u64,
    mapq_cut: u8,
) -> Result<ScanCounts> {
    // Paired-end strandness counters keyed by static `"(read_id)(map_strand)(gene_strand)"`.
    // Keys: "1++", "1+-", "1-+", "1--", "2++", "2+-", "2-+", "2--", "ambig".
    // Ambiguous-locus hits (both + and - gene) go to "ambig", contributing to other.
    let mut p_strandness: HashMap<&'static str, u64> = HashMap::new();
    // Single-end: "++", "+-", "-+", "--", "ambig".
    let mut s_strandness: HashMap<&'static str, u64> = HashMap::new();
    let mut count: u64 = 0;
    let mut rec = RawRecord::default();

    loop {
        let bytes_read = raw::read_record(reader, &mut rec)?;
        if bytes_read == 0 {
            break;
        }

        let flags = rec.flags();
        if flags & (FLAG_QCFAIL | FLAG_DUPLICATE | FLAG_SECONDARY | FLAG_UNMAPPED) != 0 {
            continue;
        }
        if rec.mapping_quality() < mapq_cut {
            continue;
        }

        let tid = rec.reference_sequence_id();
        if tid < 0 {
            continue;
        }
        #[allow(clippy::cast_sign_loss)]
        let Some(chrom) = ref_names.get(tid as usize) else {
            continue;
        };

        // RSeQC: readStart = aligned_read.pos (0-based), readEnd = readStart + qlen.
        let read_start = rec.alignment_start(); // raw BAM pos field is 0-based
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let seq_len = rec.sequence_len() as i32;
        let read_end = read_start + seq_len;

        let strand_set = gene_index.overlapping_strands(chrom, read_start, read_end);
        if strand_set.is_empty() {
            continue;
        }

        let map_strand = if flags & FLAG_REVERSE != 0 { '-' } else { '+' };

        if flags & FLAG_PAIRED != 0 {
            let read_id = if flags & FLAG_READ1 != 0 { '1' } else { '2' };
            let key = if let Some(gs) = strand_set.unique() {
                paired_key(
                    read_id,
                    map_strand,
                    if gs == GeneStrand::Plus { '+' } else { '-' },
                )
            } else {
                "ambig"
            };
            *p_strandness.entry(key).or_insert(0) += 1;
        } else {
            let key = if let Some(gs) = strand_set.unique() {
                single_key(map_strand, if gs == GeneStrand::Plus { '+' } else { '-' })
            } else {
                "ambig"
            };
            *s_strandness.entry(key).or_insert(0) += 1;
        }

        count += 1;
        if count >= sample_size {
            break;
        }
    }

    Ok((p_strandness, s_strandness, count))
}

/// Compute `StrandednessResult` from raw strandness counter maps.
fn compute_result(
    p_strandness: &HashMap<&'static str, u64>,
    s_strandness: &HashMap<&'static str, u64>,
    count: u64,
) -> StrandednessResult {
    let has_paired = p_strandness.values().any(|&v| v > 0);
    let has_single = s_strandness.values().any(|&v| v > 0);

    let (protocol, spec1, spec2, other) = match (has_paired, has_single) {
        (true, false) => {
            let total = p_strandness.values().sum::<u64>() as f64;
            let sp1 = (p_strandness.get("1++").copied().unwrap_or(0)
                + p_strandness.get("1--").copied().unwrap_or(0)
                + p_strandness.get("2+-").copied().unwrap_or(0)
                + p_strandness.get("2-+").copied().unwrap_or(0)) as f64
                / total;
            let sp2 = (p_strandness.get("1+-").copied().unwrap_or(0)
                + p_strandness.get("1-+").copied().unwrap_or(0)
                + p_strandness.get("2++").copied().unwrap_or(0)
                + p_strandness.get("2--").copied().unwrap_or(0)) as f64
                / total;
            let ot = (1.0 - sp1 - sp2).max(0.0);
            (Protocol::PairEnd, sp1, sp2, ot)
        }
        (false, true) => {
            let total = s_strandness.values().sum::<u64>() as f64;
            let sp1 = (s_strandness.get("++").copied().unwrap_or(0)
                + s_strandness.get("--").copied().unwrap_or(0)) as f64
                / total;
            let sp2 = (s_strandness.get("+-").copied().unwrap_or(0)
                + s_strandness.get("-+").copied().unwrap_or(0)) as f64
                / total;
            let ot = (1.0 - sp1 - sp2).max(0.0);
            (Protocol::SingleEnd, sp1, sp2, ot)
        }
        (true, true) => (Protocol::Mixture, 0.0, 0.0, 0.0),
        (false, false) => (Protocol::Unknown, 0.0, 0.0, 0.0),
    };

    StrandednessResult {
        protocol,
        spec1,
        spec2,
        other,
        sampled: count,
    }
}

/// Map `(read_id, map_strand, gene_strand)` → static key string for paired-end.
const fn paired_key(read_id: char, map_strand: char, gene_strand: char) -> &'static str {
    match (read_id, map_strand, gene_strand) {
        ('1', '+', '+') => "1++",
        ('1', '+', '-') => "1+-",
        ('1', '-', '+') => "1-+",
        ('1', '-', '-') => "1--",
        ('2', '+', '+') => "2++",
        ('2', '+', '-') => "2+-",
        ('2', '-', '+') => "2-+",
        ('2', '-', '-') => "2--",
        _ => "ambig",
    }
}

/// Map `(map_strand, gene_strand)` → static key string for single-end.
const fn single_key(map_strand: char, gene_strand: char) -> &'static str {
    match (map_strand, gene_strand) {
        ('+', '+') => "++",
        ('+', '-') => "+-",
        ('-', '+') => "-+",
        ('-', '-') => "--",
        _ => "ambig",
    }
}
