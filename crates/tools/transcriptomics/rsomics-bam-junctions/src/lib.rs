//! Annotate splice junctions from spliced BAM reads against a BED12 gene model.
//!
//! Mirrors `RSeQC` `junction_annotation.py` (LGPL-2.1+):
//!   - extracts every `N`-op intron from each read's CIGAR string;
//!   - filters by minimum intron length and MAPQ;
//!   - classifies each `(chrom, intron_start, intron_end)` as **known**
//!     (both splice sites present in the BED12 intron set), **`partial_novel`**
//!     (one site known), or **`complete_novel`** (neither site known);
//!   - counts both per-read **events** and distinct **junctions**.
//!
//! ## Origin
//!
//! This crate is an independent Rust reimplementation based on:
//! - `RSeQC`: `junction_annotation.py` (LGPL-2.1+), Wang et al. 2012
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

#![allow(clippy::cast_precision_loss)]

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::num::NonZero;
use std::path::Path;

use rsomics_bamio::raw::{self, RawRecord};
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

// BAM flag bits (SAM spec §1.4).
const FLAG_QCFAIL: u16 = 0x0200;
const FLAG_DUPLICATE: u16 = 0x0400;
const FLAG_SECONDARY: u16 = 0x0100;
const FLAG_UNMAPPED: u16 = 0x0004;

/// Known splice-site sets built from the BED12 intron boundaries.
///
/// `intron_starts` = set of intron start positions (= exon end positions, 0-based half-open end).
/// `intron_ends`   = set of intron end positions (= next exon start, 0-based).
///
/// `RSeQC` uses chromosome names uppercased as keys. We mirror that.
pub struct KnownSites {
    /// Per-chromosome set of intron start coordinates (exon end, 0-based).
    pub intron_starts: HashMap<String, HashSet<i64>>,
    /// Per-chromosome set of intron end coordinates (next exon start, 0-based).
    pub intron_ends: HashMap<String, HashSet<i64>>,
}

impl KnownSites {
    /// Parse a BED12 file and build the known-intron coordinate sets.
    ///
    /// Skips comment/track/browser lines and genes with a single exon
    /// (`blockCount == 1`), mirroring `RSeQC`'s `if int(fields[9]) == 1: continue`.
    pub fn from_bed12(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| RsomicsError::Io(std::io::Error::other(format!("reading BED12: {e}"))))?;

        let mut intron_starts: HashMap<String, HashSet<i64>> = HashMap::new();
        let mut intron_ends: HashMap<String, HashSet<i64>> = HashMap::new();

        for line in content.lines() {
            if line.starts_with('#') || line.starts_with("track") || line.starts_with("browser") {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 12 {
                eprintln!("[NOTE:input bed must be 12-column] skipped this line: {line}");
                continue;
            }

            let chrom = fields[0].to_uppercase();
            let Ok(tx_start) = fields[1].parse::<i64>() else {
                eprintln!("[NOTE:input bed must be 12-column] skipped this line: {line}");
                continue;
            };

            // Skip single-exon transcripts — no introns to index.
            let Ok(block_count) = fields[9].parse::<usize>() else {
                continue;
            };
            if block_count == 1 {
                continue;
            }

            // Parse block sizes and relative block starts.
            let block_sizes: Vec<i64> = fields[10]
                .trim_end_matches(',')
                .split(',')
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse().ok())
                .collect();
            let block_starts: Vec<i64> = fields[11]
                .trim_end_matches(',')
                .split(',')
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse().ok())
                .collect();

            if block_sizes.len() < 2 || block_starts.len() < 2 {
                continue;
            }

            // Absolute exon starts and ends.
            let exon_starts: Vec<i64> = block_starts.iter().map(|&s| s + tx_start).collect();
            let exon_ends: Vec<i64> = exon_starts
                .iter()
                .zip(block_sizes.iter())
                .map(|(&s, &sz)| s + sz)
                .collect();

            // Intron start = exon end (0-based); intron end = next exon start (0-based).
            for i in 0..exon_ends.len() - 1 {
                let i_st = exon_ends[i];
                let i_end = exon_starts[i + 1];
                intron_starts.entry(chrom.clone()).or_default().insert(i_st);
                intron_ends.entry(chrom.clone()).or_default().insert(i_end);
            }
        }

        Ok(Self {
            intron_starts,
            intron_ends,
        })
    }

    /// Classify an intron by whether its splice sites are known.
    #[must_use]
    pub fn classify(&self, chrom: &str, intron_start: i64, intron_end: i64) -> JunctionClass {
        let chrom_up = chrom.to_uppercase();
        let known_start = self
            .intron_starts
            .get(&chrom_up)
            .is_some_and(|s| s.contains(&intron_start));
        let known_end = self
            .intron_ends
            .get(&chrom_up)
            .is_some_and(|s| s.contains(&intron_end));
        match (known_start, known_end) {
            (true, true) => JunctionClass::Known,
            (false, false) => JunctionClass::CompleteNovel,
            _ => JunctionClass::PartialNovel,
        }
    }
}

/// Classification of a splice junction relative to the gene model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JunctionClass {
    Known,
    PartialNovel,
    CompleteNovel,
}

/// Summary counts produced by junction annotation.
#[derive(Debug, Clone, Default, Serialize)]
pub struct JunctionCounts {
    /// Total N-op occurrences seen across all reads (before min-intron filter).
    pub total_events: u64,
    /// Events that were skipped for being shorter than `min_intron`.
    pub filtered_events: u64,
    /// Per-read-occurrence counts (events) for passing junctions.
    pub known_events: u64,
    pub partial_novel_events: u64,
    pub novel_events: u64,
    /// Distinct-junction counts.
    pub known_junctions: u64,
    pub partial_novel_junctions: u64,
    pub novel_junctions: u64,
}

impl JunctionCounts {
    /// Total distinct junctions.
    #[must_use]
    pub fn total_junctions(&self) -> u64 {
        self.known_junctions + self.partial_novel_junctions + self.novel_junctions
    }

    /// Emit the exact stderr text format that `RSeQC junction_annotation.py` prints.
    ///
    /// `RSeQC` emits two `===` blocks to stderr: first events (total includes
    /// filtered events), then junctions (distinct passing junctions).
    /// The `total = N` line (total events including filtered) goes to stdout.
    pub fn write_rseqc_stderr<W: Write>(&self, mut out: W) -> std::io::Result<()> {
        writeln!(out)?;
        writeln!(
            out,
            "==================================================================="
        )?;
        writeln!(out, "Total splicing  Events:\t{}", self.total_events)?;
        writeln!(out, "Known Splicing Events:\t{}", self.known_events)?;
        writeln!(
            out,
            "Partial Novel Splicing Events:\t{}",
            self.partial_novel_events
        )?;
        writeln!(out, "Novel Splicing Events:\t{}", self.novel_events)?;
        writeln!(out, "Filtered Splicing Events:\t{}", self.filtered_events)?;

        writeln!(out)?;
        writeln!(
            out,
            "Total splicing  Junctions:\t{}",
            self.total_junctions()
        )?;
        writeln!(out, "Known Splicing Junctions:\t{}", self.known_junctions)?;
        writeln!(
            out,
            "Partial Novel Splicing Junctions:\t{}",
            self.partial_novel_junctions
        )?;
        writeln!(out, "Novel Splicing Junctions:\t{}", self.novel_junctions)?;
        writeln!(out)?;
        writeln!(
            out,
            "==================================================================="
        )?;
        Ok(())
    }

    /// Emit `total = N\n` to stdout.
    ///
    /// Matches `RSeQC`'s `print("total = " + str(total_junc))` where
    /// `total_junc` is the count of all N-ops seen, including filtered ones.
    pub fn write_rseqc_stdout<W: Write>(&self, mut out: W) -> std::io::Result<()> {
        writeln!(out, "total = {}", self.total_events)?;
        Ok(())
    }
}

/// Annotate splice junctions from `bam_path` against the BED12 gene model at `bed_path`.
pub fn annotate_junctions(
    bam_path: &Path,
    bed_path: &Path,
    min_intron: i64,
    mapq_cut: u8,
    workers: NonZero<usize>,
) -> Result<JunctionCounts> {
    eprintln!("Reading reference bed file: {} ... ", bed_path.display());
    let known = KnownSites::from_bed12(bed_path)?;
    eprintln!("Done");

    if bam_path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("bam"))
    {
        eprintln!("Load BAM file ... ");
    } else {
        eprintln!("Load SAM file ... ");
    }

    let mut reader = rsomics_bamio::open_with_workers(bam_path, workers)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let ref_names: Vec<String> = header
        .reference_sequences()
        .keys()
        .map(|k| String::from_utf8_lossy(k.as_ref()).into_owned())
        .collect();

    let counts = scan_reads(reader.get_mut(), &ref_names, &known, min_intron, mapq_cut)?;

    eprintln!("Done");
    Ok(counts)
}

/// Inner BAM-scanning loop: extract N-ops, classify, accumulate counts.
fn scan_reads<R: std::io::Read>(
    reader: &mut R,
    ref_names: &[String],
    known: &KnownSites,
    min_intron: i64,
    mapq_cut: u8,
) -> Result<JunctionCounts> {
    // Map (chrom, intron_start, intron_end) → per-read occurrence count (events).
    let mut splicing_events: HashMap<(String, i64, i64), u64> = HashMap::new();

    let mut total_events: u64 = 0;
    let mut filtered_events: u64 = 0;
    // Per-read event counters (before dedup into junctions).
    let mut known_ev: u64 = 0;
    let mut partial_ev: u64 = 0;
    let mut novel_ev: u64 = 0;

    let mut rec = RawRecord::default();

    loop {
        let bytes = raw::read_record(reader, &mut rec)?;
        if bytes == 0 {
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

        // Walk the CIGAR to collect N (intron/skip) blocks.
        let introns = fetch_introns(chrom, i64::from(rec.alignment_start()), rec.cigar_ops());
        if introns.is_empty() {
            continue;
        }

        for (chr, i_st, i_end) in introns {
            total_events += 1;
            if i_end - i_st < min_intron {
                filtered_events += 1;
                continue;
            }
            *splicing_events
                .entry((chr.clone(), i_st, i_end))
                .or_insert(0) += 1;

            // Classify each event occurrence (not just distinct junctions).
            match known.classify(&chr, i_st, i_end) {
                JunctionClass::Known => known_ev += 1,
                JunctionClass::PartialNovel => partial_ev += 1,
                JunctionClass::CompleteNovel => novel_ev += 1,
            }
        }
    }

    // Count distinct junctions.
    let mut known_junc: u64 = 0;
    let mut partial_junc: u64 = 0;
    let mut novel_junc: u64 = 0;

    for (chr, i_st, i_end) in splicing_events.keys() {
        match known.classify(chr, *i_st, *i_end) {
            JunctionClass::Known => known_junc += 1,
            JunctionClass::PartialNovel => partial_junc += 1,
            JunctionClass::CompleteNovel => novel_junc += 1,
        }
    }

    Ok(JunctionCounts {
        total_events,
        filtered_events,
        known_events: known_ev,
        partial_novel_events: partial_ev,
        novel_events: novel_ev,
        known_junctions: known_junc,
        partial_novel_junctions: partial_junc,
        novel_junctions: novel_junc,
    })
}

/// Walk a CIGAR op iterator and collect N-op (intron) spans as
/// `(chrom, intron_start_0based, intron_end_0based)`.
///
/// BAM CIGAR opcodes: 0=M, 1=I, 2=D, 3=N, 4=S, 5=H, 6=P, 7==, 8=X.
/// Reference-consuming ops: M(0), D(2), N(3), =(7), X(8).
/// N(3) = intron skip; produces one output entry.
fn fetch_introns<I>(chrom: &str, alignment_start: i64, ops: I) -> Vec<(String, i64, i64)>
where
    I: Iterator<Item = (u8, u32)>,
{
    let mut result = Vec::new();
    let mut ref_pos = alignment_start;

    for (op_code, op_len) in ops {
        let len = i64::from(op_len);
        match op_code {
            0 | 2 | 7 | 8 => {
                // M, D, =, X — consume reference
                ref_pos += len;
            }
            3 => {
                // N — intron skip
                let i_st = ref_pos;
                let i_end = ref_pos + len;
                result.push((chrom.to_string(), i_st, i_end));
                ref_pos = i_end;
            }
            // I(1), S(4), H(5), P(6) — do not consume reference
            _ => {}
        }
    }
    result
}
