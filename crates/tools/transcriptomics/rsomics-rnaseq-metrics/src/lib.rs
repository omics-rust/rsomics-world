//! RNA-seq QC metrics: base-region classification + transcript-coverage bias.
//!
//! Mirrors Picard `CollectRnaSeqMetrics` 3.4.0 (MIT licence — source-readable port).
//!
//! Algorithm (from Picard source):
//! - Each aligned, passing-filter base is classified into exactly one region with priority
//!   RIBOSOMAL > CODING > UTR > INTRONIC > INTERGENIC.
//! - CODING: base overlaps an exon AND falls within `[cdsStart, cdsEnd)`.
//! - UTR: base overlaps an exon AND falls outside `[cdsStart, cdsEnd)`.
//! - INTRONIC: base within `[txStart, txEnd)` but not in any exon.
//! - INTERGENIC: base outside all gene spans.
//! - Bias block: top-1000 most-expressed transcripts (by total coverage depth), each
//!   normalized to 100-position percentile bins; medians of CV, 5′/3′ bias reported.
//!
//! ## Origin
//!
//! This crate is a Rust port based on the MIT-licensed Picard 3.4.0 source:
//! - `src/main/java/picard/analysis/CollectRnaSeqMetrics.java`
//! - `src/main/java/picard/analysis/RnaSeqMetrics.java`
//! - Black-box validation against Picard 3.4.0
//!
//! License: MIT OR Apache-2.0.
//! Upstream credit: Picard <https://github.com/broadinstitute/picard> (MIT).

#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;

use noodles::bam;
use noodles::sam::alignment::record::cigar::op::Kind;
use rsomics_common::{Result, RsomicsError};

// ── Strand specificity ─────────────────────────────────────────────────────

/// Strand specificity mode, matching Picard's `StrandSpecificity` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrandSpecificity {
    None,
    /// Read 1 is sense (same strand as transcript).
    FirstReadTranscriptionStrand,
    /// Read 2 is sense (same strand as transcript).
    SecondReadTranscriptionStrand,
}

impl std::str::FromStr for StrandSpecificity {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, String> {
        match s.to_uppercase().as_str() {
            "NONE" => Ok(Self::None),
            "FIRST_READ_TRANSCRIPTION_STRAND" => Ok(Self::FirstReadTranscriptionStrand),
            "SECOND_READ_TRANSCRIPTION_STRAND" => Ok(Self::SecondReadTranscriptionStrand),
            other => Err(format!("unknown strand specificity: {other}")),
        }
    }
}

// ── Gene model ─────────────────────────────────────────────────────────────

/// A parsed gene model from refFlat.
///
/// All coordinates are 0-based half-open, matching refFlat conventions.
#[derive(Debug)]
pub struct Gene {
    pub chrom: String,
    /// Gene strand ('+' or '-').
    pub strand: char,
    /// Transcript start (0-based, inclusive).
    pub tx_start: u64,
    /// Transcript end (0-based, exclusive).
    pub tx_end: u64,
    /// CDS start (0-based, inclusive). Equals `tx_end` for non-coding transcripts.
    pub cds_start: u64,
    /// CDS end (0-based, exclusive). Equals `tx_start` for non-coding transcripts.
    pub cds_end: u64,
    /// Sorted exon intervals `[start, end)` in 0-based half-open.
    pub exons: Vec<(u64, u64)>,
    /// Cached union of exon bases as sorted non-overlapping intervals.
    pub exon_union: Vec<(u64, u64)>,
}

impl Gene {
    fn new(
        chrom: String,
        strand: char,
        tx_start: u64,
        tx_end: u64,
        cds_start: u64,
        cds_end: u64,
        exons: Vec<(u64, u64)>,
    ) -> Self {
        let exon_union = merge_intervals(&exons);
        Self {
            chrom,
            strand,
            tx_start,
            tx_end,
            cds_start,
            cds_end,
            exons,
            exon_union,
        }
    }

    /// True if 0-based position `p` falls in any exon.
    #[must_use]
    pub fn in_exon(&self, p: u64) -> bool {
        self.exon_union
            .binary_search_by(|&(s, e)| {
                if p < s {
                    std::cmp::Ordering::Greater
                } else if p >= e {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .is_ok()
    }

    /// Total exon (mRNA) length in bases.
    #[must_use]
    pub fn mrna_len(&self) -> u64 {
        self.exon_union.iter().map(|(s, e)| e - s).sum()
    }
}

// ── refFlat parsing ────────────────────────────────────────────────────────

/// Parse a refFlat file into a list of [`Gene`] records.
///
/// refFlat column order (0-based, tab-separated):
///   0 geneName, 1 name (transcript), 2 chrom, 3 strand, 4 txStart, 5 txEnd,
///   6 cdsStart, 7 cdsEnd, 8 exonCount, 9 exonStarts, 10 exonEnds.
/// All coordinates 0-based half-open.
pub fn load_refflat(path: &Path) -> Result<Vec<Gene>> {
    let f = std::fs::File::open(path)
        .map_err(|e| RsomicsError::Io(std::io::Error::other(format!("{}: {e}", path.display()))))?;
    let reader = BufReader::new(f);
    let mut genes = Vec::new();
    for (lineno, line) in reader.lines().enumerate() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 11 {
            return Err(RsomicsError::InvalidInput(format!(
                "{}:{}: expected ≥11 tab-separated fields, got {}",
                path.display(),
                lineno + 1,
                fields.len()
            )));
        }
        let chrom = fields[2].to_string();
        let strand = fields[3].chars().next().unwrap_or('+');
        let tx_start: u64 = parse_u64(fields[4], path, lineno, "txStart")?;
        let tx_end: u64 = parse_u64(fields[5], path, lineno, "txEnd")?;
        let cds_start: u64 = parse_u64(fields[6], path, lineno, "cdsStart")?;
        let cds_end: u64 = parse_u64(fields[7], path, lineno, "cdsEnd")?;
        let exon_count: usize = fields[8].parse().map_err(|_| {
            RsomicsError::InvalidInput(format!("{}: bad exonCount", path.display()))
        })?;
        let starts = parse_csv_u64(fields[9], exon_count).ok_or_else(|| {
            RsomicsError::InvalidInput(format!("{}: bad exonStarts", path.display()))
        })?;
        let ends = parse_csv_u64(fields[10], exon_count).ok_or_else(|| {
            RsomicsError::InvalidInput(format!("{}: bad exonEnds", path.display()))
        })?;
        let exons: Vec<(u64, u64)> = starts.into_iter().zip(ends).collect();
        genes.push(Gene::new(
            chrom, strand, tx_start, tx_end, cds_start, cds_end, exons,
        ));
    }
    Ok(genes)
}

fn parse_u64(s: &str, path: &Path, lineno: usize, field: &str) -> Result<u64> {
    s.parse().map_err(|_| {
        RsomicsError::InvalidInput(format!(
            "{}:{}: bad {field}: {s:?}",
            path.display(),
            lineno + 1
        ))
    })
}

fn parse_csv_u64(s: &str, n: usize) -> Option<Vec<u64>> {
    let parts: Vec<&str> = s.trim_end_matches(',').split(',').collect();
    if parts.len() != n {
        return None;
    }
    parts.iter().map(|p| p.parse().ok()).collect()
}

/// Merge a list of (potentially overlapping) intervals into sorted non-overlapping intervals.
fn merge_intervals(ivs: &[(u64, u64)]) -> Vec<(u64, u64)> {
    let mut sorted: Vec<(u64, u64)> = ivs.to_vec();
    sorted.sort_unstable();
    let mut merged: Vec<(u64, u64)> = Vec::new();
    for (s, e) in sorted {
        if let Some(last) = merged.last_mut()
            && s <= last.1
        {
            last.1 = last.1.max(e);
        } else {
            merged.push((s, e));
        }
    }
    merged
}

// ── rRNA interval_list ─────────────────────────────────────────────────────

/// Parse a Picard `interval_list` for rRNA regions.
///
/// Returns a map from chrom → sorted Vec of `(start, end)` 0-based half-open intervals.
/// `interval_list` format: SAM header lines (`@HD`, `@SQ`, …), then data lines
/// `chrom\tstart\tend\tstrand\tname` with 1-based inclusive coordinates.
pub fn load_rrna_intervals(path: &Path) -> Result<HashMap<String, Vec<(u64, u64)>>> {
    let f = std::fs::File::open(path)
        .map_err(|e| RsomicsError::Io(std::io::Error::other(format!("{}: {e}", path.display()))))?;
    let reader = BufReader::new(f);
    let mut map: HashMap<String, Vec<(u64, u64)>> = HashMap::new();
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('@') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 3 {
            continue;
        }
        let chrom = fields[0].to_string();
        // 1-based inclusive → 0-based half-open.
        let start: u64 = fields[1]
            .parse::<u64>()
            .map_err(|_| RsomicsError::InvalidInput(format!("bad interval start: {}", fields[1])))?
            .saturating_sub(1);
        let end: u64 = fields[2]
            .parse()
            .map_err(|_| RsomicsError::InvalidInput(format!("bad interval end: {}", fields[2])))?;
        map.entry(chrom).or_default().push((start, end));
    }
    for ivs in map.values_mut() {
        *ivs = merge_intervals(ivs);
    }
    Ok(map)
}

/// Check if 0-based position `p` overlaps any interval in a sorted, merged list.
fn in_intervals(ivs: &[(u64, u64)], p: u64) -> bool {
    ivs.binary_search_by(|&(s, e)| {
        if p < s {
            std::cmp::Ordering::Greater
        } else if p >= e {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    })
    .is_ok()
}

// ── Metrics accumulator ────────────────────────────────────────────────────

/// Accumulated RNA-seq metrics (base counts).
#[derive(Debug, Default)]
pub struct RnaSeqMetrics {
    pub pf_bases: u64,
    pub pf_aligned_bases: u64,
    pub ribosomal_bases: u64,
    pub coding_bases: u64,
    pub utr_bases: u64,
    pub intronic_bases: u64,
    pub intergenic_bases: u64,
    pub ignored_reads: u64,
    pub correct_strand_reads: u64,
    pub incorrect_strand_reads: u64,
    /// R1 reads overlapping any transcript.
    pub num_r1_transcript_strand_reads: u64,
    /// R2 reads overlapping any transcript.
    pub num_r2_transcript_strand_reads: u64,
    /// Reads overlapping a transcript but neither first nor last segment.
    pub num_unexplained_reads: u64,
}

impl RnaSeqMetrics {
    #[must_use]
    pub fn pct_ribosomal_bases(&self) -> f64 {
        if self.pf_aligned_bases == 0 {
            return 0.0;
        }
        self.ribosomal_bases as f64 / self.pf_aligned_bases as f64
    }

    #[must_use]
    pub fn pct_coding_bases(&self) -> f64 {
        if self.pf_aligned_bases == 0 {
            return 0.0;
        }
        self.coding_bases as f64 / self.pf_aligned_bases as f64
    }

    #[must_use]
    pub fn pct_utr_bases(&self) -> f64 {
        if self.pf_aligned_bases == 0 {
            return 0.0;
        }
        self.utr_bases as f64 / self.pf_aligned_bases as f64
    }

    #[must_use]
    pub fn pct_intronic_bases(&self) -> f64 {
        if self.pf_aligned_bases == 0 {
            return 0.0;
        }
        self.intronic_bases as f64 / self.pf_aligned_bases as f64
    }

    #[must_use]
    pub fn pct_intergenic_bases(&self) -> f64 {
        if self.pf_aligned_bases == 0 {
            return 0.0;
        }
        self.intergenic_bases as f64 / self.pf_aligned_bases as f64
    }

    /// `PCT_MRNA_BASES` = `(CODING + UTR) / ALIGNED`
    #[must_use]
    pub fn pct_mrna_bases(&self) -> f64 {
        if self.pf_aligned_bases == 0 {
            return 0.0;
        }
        (self.coding_bases + self.utr_bases) as f64 / self.pf_aligned_bases as f64
    }

    /// `PCT_USABLE_BASES` = mRNA bases / `PF_BASES` (denominator is total, not aligned).
    #[must_use]
    pub fn pct_usable_bases(&self) -> f64 {
        if self.pf_bases == 0 {
            return 0.0;
        }
        (self.coding_bases + self.utr_bases) as f64 / self.pf_bases as f64
    }

    #[must_use]
    pub fn pct_correct_strand_reads(&self) -> f64 {
        let total = self.correct_strand_reads + self.incorrect_strand_reads;
        if total == 0 {
            return 0.0;
        }
        self.correct_strand_reads as f64 / total as f64
    }

    /// `PCT_R1_TRANSCRIPT_STRAND_READS`
    #[must_use]
    pub fn pct_r1_transcript_strand_reads(&self) -> f64 {
        let total = self.num_r1_transcript_strand_reads
            + self.num_r2_transcript_strand_reads
            + self.num_unexplained_reads;
        if total == 0 {
            return 0.0;
        }
        self.num_r1_transcript_strand_reads as f64 / total as f64
    }

    /// `PCT_R2_TRANSCRIPT_STRAND_READS`
    #[must_use]
    pub fn pct_r2_transcript_strand_reads(&self) -> f64 {
        let total = self.num_r1_transcript_strand_reads
            + self.num_r2_transcript_strand_reads
            + self.num_unexplained_reads;
        if total == 0 {
            return 0.0;
        }
        self.num_r2_transcript_strand_reads as f64 / total as f64
    }
}

/// Bias metrics computed from the top-N transcripts.
#[derive(Debug)]
pub struct BiasMetrics {
    pub median_cv_coverage: f64,
    pub median_5prime_bias: f64,
    pub median_3prime_bias: f64,
    pub median_5prime_to_3prime_bias: f64,
}

// ── Gene index ─────────────────────────────────────────────────────────────

/// Spatial index for fast gene-overlap queries.
pub struct GeneIndex {
    genes: Vec<Gene>,
    /// chrom → indices into `genes` sorted by `tx_start`.
    chrom_map: HashMap<String, Vec<usize>>,
}

impl GeneIndex {
    #[must_use]
    pub fn new(genes: Vec<Gene>) -> Self {
        let mut chrom_map: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, g) in genes.iter().enumerate() {
            chrom_map.entry(g.chrom.clone()).or_default().push(i);
        }
        for v in chrom_map.values_mut() {
            v.sort_unstable_by_key(|&i| genes[i].tx_start);
        }
        Self { genes, chrom_map }
    }

    /// Iterate over genes whose `[tx_start, tx_end)` overlaps 0-based position `p` on `chrom`.
    pub fn overlapping<'a>(&'a self, chrom: &str, p: u64) -> impl Iterator<Item = &'a Gene> {
        let indices = self
            .chrom_map
            .get(chrom)
            .map_or(&[] as &[usize], Vec::as_slice);
        let upper = indices.partition_point(|&i| self.genes[i].tx_start <= p);
        let genes = &self.genes;
        indices[..upper]
            .iter()
            .rev()
            .take_while(move |&&i| genes[i].tx_end > p)
            .map(move |&i| &genes[i])
    }

    #[must_use]
    pub fn genes(&self) -> &[Gene] {
        &self.genes
    }
}

// ── Base classification ────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum Region {
    Coding,
    Utr,
    Intronic,
    Intergenic,
}

fn classify_base(chrom: &str, p: u64, gene_index: &GeneIndex) -> Region {
    let mut coding = false;
    let mut utr = false;
    let mut intronic = false;
    let mut in_gene_span = false;

    for gene in gene_index.overlapping(chrom, p) {
        in_gene_span = true;
        if gene.in_exon(p) {
            if p >= gene.cds_start && p < gene.cds_end {
                coding = true;
            } else {
                utr = true;
            }
        } else {
            intronic = true;
        }
    }

    if coding {
        Region::Coding
    } else if utr {
        Region::Utr
    } else if intronic || in_gene_span {
        Region::Intronic
    } else {
        Region::Intergenic
    }
}

// ── Strand determination ───────────────────────────────────────────────────

/// Determine if a read is on the correct transcript strand.
///
/// Returns `Some(true)` = correct, `Some(false)` = incorrect, `None` = not counted.
///
/// Picard logic: for `FIRST_READ_TRANSCRIPTION_STRAND`, R1 on same strand as gene = correct;
/// for `SECOND_READ_TRANSCRIPTION_STRAND`, R2 on same strand as gene = correct.
fn strand_correct(
    flags: noodles::sam::alignment::record::Flags,
    gene_strand: char,
    strand_specificity: StrandSpecificity,
) -> Option<bool> {
    if strand_specificity == StrandSpecificity::None {
        return None;
    }
    let is_r1 = !flags.is_supplementary() && !flags.is_secondary() && !flags.is_last_segment();
    let is_reverse = flags.is_reverse_complemented();
    let read_on_plus = match strand_specificity {
        StrandSpecificity::FirstReadTranscriptionStrand => {
            if is_r1 {
                !is_reverse
            } else {
                is_reverse
            }
        }
        StrandSpecificity::SecondReadTranscriptionStrand => {
            if is_r1 {
                is_reverse
            } else {
                !is_reverse
            }
        }
        StrandSpecificity::None => unreachable!(),
    };
    Some(read_on_plus == (gene_strand == '+'))
}

// ── BAM processing ─────────────────────────────────────────────────────────

/// Compute the number of aligned bases in a read that overlap with rRNA intervals.
///
/// Picard computes this as the fraction of the READ (not aligned bases) that overlaps
/// rRNA. A read with >= `RRNA_FRAGMENT_PERCENTAGE` (default 0.8) overlap is classified
/// entirely as ribosomal — its bases are counted in `RIBOSOMAL_BASES` and it is skipped
/// for all other region classification.
fn rrna_overlap_bases(
    cigar: &noodles::bam::record::Cigar<'_>,
    aln_start_0: u64,
    rrna_ivs: &[(u64, u64)],
) -> u64 {
    let mut overlap = 0u64;
    let mut ref_cursor = aln_start_0;
    for op_result in cigar.iter() {
        let Ok(op) = op_result else { break };
        let len = op.len() as u64;
        match op.kind() {
            Kind::Match | Kind::SequenceMatch | Kind::SequenceMismatch => {
                for dp in 0..len {
                    if in_intervals(rrna_ivs, ref_cursor + dp) {
                        overlap += 1;
                    }
                }
                ref_cursor += len;
            }
            Kind::Deletion | Kind::Skip => {
                ref_cursor += len;
            }
            Kind::SoftClip | Kind::HardClip | Kind::Pad | Kind::Insertion => {}
        }
    }
    overlap
}

/// Process a single BAM file and accumulate RNA-seq metrics.
///
/// rRNA classification is READ-LEVEL (matching Picard): a read with ≥ 80% of its bases
/// overlapping rRNA intervals is classified entirely as `RIBOSOMAL_BASES`; its bases are
/// not classified for region. Reads below the threshold are classified base-by-base.
#[allow(clippy::implicit_hasher)]
pub fn collect_metrics(
    bam_path: &Path,
    gene_index: &GeneIndex,
    rrna: &HashMap<String, Vec<(u64, u64)>>,
    strand_spec: StrandSpecificity,
) -> Result<RnaSeqMetrics> {
    let f = std::fs::File::open(bam_path).map_err(|e| {
        RsomicsError::Io(std::io::Error::other(format!(
            "{}: {e}",
            bam_path.display()
        )))
    })?;
    let mut reader = bam::io::Reader::new(f);
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let mut metrics = RnaSeqMetrics::default();
    let mut record = bam::Record::default();
    loop {
        match reader.read_record(&mut record) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => return Err(RsomicsError::Io(e)),
        }

        let flags = record.flags();
        // Secondary/supplementary reads are silently skipped (not in PF_BASES, not in IGNORED_READS).
        if flags.is_secondary() || flags.is_supplementary() {
            continue;
        }

        let seq_len = record.sequence().len() as u64;
        metrics.pf_bases += seq_len;

        if flags.is_unmapped() || flags.is_qc_fail() {
            continue;
        }

        let ref_id = record
            .reference_sequence_id()
            .ok_or_else(|| RsomicsError::InvalidInput("missing reference id".into()))??;
        let chrom = header
            .reference_sequences()
            .get_index(ref_id)
            .map(|(name, _)| name.to_string())
            .ok_or_else(|| RsomicsError::InvalidInput("unknown reference id".into()))?;

        let aln_start_1: u64 = match record.alignment_start() {
            Some(Ok(pos)) => usize::from(pos) as u64,
            _ => continue,
        };
        let aln_start_0 = aln_start_1 - 1;
        let cigar = record.cigar();

        process_read(
            &cigar,
            flags,
            &chrom,
            aln_start_0,
            seq_len,
            gene_index,
            rrna,
            strand_spec,
            &mut metrics,
        );
    }

    Ok(metrics)
}

// Default rRNA fragment percentage from Picard.
const RRNA_FRAGMENT_PCT: f64 = 0.8;

/// Process one mapped read into metrics.
#[allow(clippy::too_many_arguments)]
fn process_read(
    cigar: &noodles::bam::record::Cigar<'_>,
    flags: noodles::sam::alignment::record::Flags,
    chrom: &str,
    aln_start_0: u64,
    seq_len: u64,
    gene_index: &GeneIndex,
    rrna: &HashMap<String, Vec<(u64, u64)>>,
    strand_spec: StrandSpecificity,
    metrics: &mut RnaSeqMetrics,
) {
    if let Some(rrna_ivs) = rrna.get(chrom) {
        let overlap = rrna_overlap_bases(cigar, aln_start_0, rrna_ivs);
        if overlap as f64 / seq_len as f64 >= RRNA_FRAGMENT_PCT {
            metrics.ribosomal_bases += seq_len;
            metrics.pf_aligned_bases += seq_len;
            return;
        }
    }

    let mut ref_cursor = aln_start_0;
    let mut any_transcript_gene: Option<char> = None;

    for op_result in cigar.iter() {
        let Ok(op) = op_result else { break };
        let len = op.len() as u64;
        match op.kind() {
            Kind::Match | Kind::SequenceMatch | Kind::SequenceMismatch => {
                for dp in 0..len {
                    let p = ref_cursor + dp;
                    metrics.pf_aligned_bases += 1;
                    match classify_base(chrom, p, gene_index) {
                        Region::Coding => metrics.coding_bases += 1,
                        Region::Utr => metrics.utr_bases += 1,
                        Region::Intronic => metrics.intronic_bases += 1,
                        Region::Intergenic => metrics.intergenic_bases += 1,
                    }
                }
                if any_transcript_gene.is_none() {
                    any_transcript_gene = gene_index
                        .overlapping(chrom, ref_cursor)
                        .next()
                        .map(|g| g.strand);
                }
                ref_cursor += len;
            }
            Kind::Deletion | Kind::Skip => {
                ref_cursor += len;
            }
            Kind::SoftClip | Kind::HardClip | Kind::Pad | Kind::Insertion => {}
        }
    }

    if let Some(gene_strand) = any_transcript_gene {
        if let Some(correct) = strand_correct(flags, gene_strand, strand_spec) {
            if correct {
                metrics.correct_strand_reads += 1;
            } else {
                metrics.incorrect_strand_reads += 1;
            }
        }
        // Picard uses SAM flag bits 0x40 (READ1) and 0x80 (READ2) to classify reads.
        // Reads without either bit (including unpaired fragment reads) are UNEXPLAINED.
        if flags.is_first_segment() {
            metrics.num_r1_transcript_strand_reads += 1;
        } else if flags.is_last_segment() {
            metrics.num_r2_transcript_strand_reads += 1;
        } else {
            metrics.num_unexplained_reads += 1;
        }
    }
}

// ── Bias computation ───────────────────────────────────────────────────────

/// Compute transcript-coverage bias over a second BAM pass.
///
/// For each transcript with mRNA length ≥ `min_length`, collects coverage at
/// 100 percentile positions, selects top 1000 by total coverage, then computes
/// medians of CV, 5′ bias, 3′ bias, and 5′/3′ ratio.
///
/// Picard percentile binning formula: `bin_i = floor(i * (length-1) / 99)` for `i` in 0..99.
pub fn compute_bias(
    bam_path: &Path,
    gene_index: &GeneIndex,
    min_length: u64,
) -> Result<BiasMetrics> {
    let zero_bias = BiasMetrics {
        median_cv_coverage: 0.0,
        median_5prime_bias: 0.0,
        median_3prime_bias: 0.0,
        median_5prime_to_3prime_bias: 0.0,
    };

    let tx_samples = build_tx_samples(gene_index, min_length);
    if tx_samples.is_empty() {
        return Ok(zero_bias);
    }

    let pos_to_bins = build_pos_index(&tx_samples, gene_index);
    let (coverages, total_cov) = accumulate_coverage(bam_path, &pos_to_bins, tx_samples.len())?;

    let mut ranked: Vec<usize> = (0..tx_samples.len())
        .filter(|&i| total_cov[i] > 0)
        .collect();
    ranked.sort_unstable_by(|&a, &b| total_cov[b].cmp(&total_cov[a]));
    ranked.truncate(1000);

    if ranked.is_empty() {
        return Ok(zero_bias);
    }

    Ok(compute_bias_stats(&coverages, &ranked))
}

fn build_tx_samples(gene_index: &GeneIndex, min_length: u64) -> Vec<(usize, Vec<u64>)> {
    gene_index
        .genes()
        .iter()
        .enumerate()
        .filter(|(_, g)| g.mrna_len() >= min_length)
        .map(|(i, g)| {
            let mut positions: Vec<u64> = g.exon_union.iter().flat_map(|&(s, e)| s..e).collect();
            if g.strand == '-' {
                positions.reverse();
            }
            let sample_pos: Vec<u64> = picard_percentile_indices(positions.len())
                .into_iter()
                .map(|idx| positions[idx])
                .collect();
            (i, sample_pos)
        })
        .collect()
}

fn build_pos_index(tx_samples: &[(usize, Vec<u64>)], gene_index: &GeneIndex) -> PosIndex {
    let mut pos_to_bins: PosIndex = HashMap::new();
    for (tx_vec_idx, (gene_idx, sample_pos)) in tx_samples.iter().enumerate() {
        let chrom = &gene_index.genes()[*gene_idx].chrom;
        let chrom_map = pos_to_bins.entry(chrom.clone()).or_default();
        for (bin, &gpos) in sample_pos.iter().enumerate() {
            chrom_map.entry(gpos).or_default().push((tx_vec_idx, bin));
        }
    }
    pos_to_bins
}

type PosIndex = HashMap<String, HashMap<u64, Vec<(usize, usize)>>>;

#[allow(clippy::implicit_hasher)]
fn accumulate_coverage(
    bam_path: &Path,
    pos_to_bins: &PosIndex,
    num_tx: usize,
) -> Result<(Vec<[u64; 100]>, Vec<u64>)> {
    let mut coverages: Vec<[u64; 100]> = vec![[0u64; 100]; num_tx];
    let mut total_cov: Vec<u64> = vec![0u64; num_tx];

    let f = std::fs::File::open(bam_path).map_err(|e| {
        RsomicsError::Io(std::io::Error::other(format!(
            "{}: {e}",
            bam_path.display()
        )))
    })?;
    let mut reader = bam::io::Reader::new(f);
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let mut record = bam::Record::default();
    loop {
        match reader.read_record(&mut record) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => return Err(RsomicsError::Io(e)),
        }

        let flags = record.flags();
        if flags.is_unmapped()
            || flags.is_secondary()
            || flags.is_supplementary()
            || flags.is_qc_fail()
            || flags.is_duplicate()
        {
            continue;
        }

        let Some(Ok(ref_id)) = record.reference_sequence_id() else {
            continue;
        };
        let Some((chrom_name, _)) = header.reference_sequences().get_index(ref_id) else {
            continue;
        };
        let chrom = chrom_name.to_string();

        let Some(chrom_map) = pos_to_bins.get(&chrom) else {
            continue;
        };

        let aln_start_1: u64 = match record.alignment_start() {
            Some(Ok(pos)) => usize::from(pos) as u64,
            _ => continue,
        };

        let cigar = record.cigar();
        let mut ref_cursor = aln_start_1 - 1; // 0-based

        for op_result in cigar.iter() {
            let Ok(op) = op_result else { break };
            let len = op.len() as u64;
            match op.kind() {
                Kind::Match | Kind::SequenceMatch | Kind::SequenceMismatch => {
                    for dp in 0..len {
                        let p = ref_cursor + dp;
                        if let Some(bins) = chrom_map.get(&p) {
                            for &(tx_idx, bin) in bins {
                                coverages[tx_idx][bin] += 1;
                                total_cov[tx_idx] += 1;
                            }
                        }
                    }
                    ref_cursor += len;
                }
                Kind::Deletion | Kind::Skip => {
                    ref_cursor += len;
                }
                Kind::SoftClip | Kind::HardClip | Kind::Pad | Kind::Insertion => {}
            }
        }
    }

    Ok((coverages, total_cov))
}

// END_BIAS_BASES default = 100 in Picard.
// With a 100-bin vector, end_bins=100 means both 5' and 3' bias = global mean = 1.0.
const END_BINS: usize = 100;

fn compute_bias_stats(coverages: &[[u64; 100]], ranked: &[usize]) -> BiasMetrics {
    let mut cv_acc: Vec<f64> = Vec::with_capacity(ranked.len());
    let mut prime5_acc: Vec<f64> = Vec::with_capacity(ranked.len());
    let mut prime3_acc: Vec<f64> = Vec::with_capacity(ranked.len());
    let mut ratio_acc: Vec<f64> = Vec::with_capacity(ranked.len());

    for &tx_idx in ranked {
        let cov = &coverages[tx_idx];
        let mean = cov.iter().sum::<u64>() as f64 / 100.0;
        if mean == 0.0 {
            continue;
        }
        let variance = cov
            .iter()
            .map(|&c| {
                let d = c as f64 - mean;
                d * d
            })
            .sum::<f64>()
            / 100.0;
        cv_acc.push(variance.sqrt() / mean);

        let end_bins = END_BINS.min(100);
        let mean_5 = cov[..end_bins].iter().sum::<u64>() as f64 / end_bins as f64;
        let mean_3 = cov[(100 - end_bins)..].iter().sum::<u64>() as f64 / end_bins as f64;
        let b5 = mean_5 / mean;
        let b3 = mean_3 / mean;
        prime5_acc.push(b5);
        prime3_acc.push(b3);
        if b3 > 0.0 {
            ratio_acc.push(b5 / b3);
        }
    }

    BiasMetrics {
        median_cv_coverage: median_f64(&mut cv_acc),
        median_5prime_bias: median_f64(&mut prime5_acc),
        median_3prime_bias: median_f64(&mut prime3_acc),
        median_5prime_to_3prime_bias: median_f64(&mut ratio_acc),
    }
}

/// Compute 100 Picard-style percentile indices into a positions list of length `n`.
///
/// Picard formula: `index_i = floor(i * (n-1) / 99)` for `i` in 0..99.
fn picard_percentile_indices(n: usize) -> Vec<usize> {
    (0..100)
        .map(|i| {
            if n <= 1 {
                0
            } else {
                (i * (n - 1) / 99).min(n - 1)
            }
        })
        .collect()
}

fn median_f64(vals: &mut [f64]) -> f64 {
    if vals.is_empty() {
        return 0.0;
    }
    vals.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = vals.len();
    if n % 2 == 1 {
        vals[n / 2]
    } else {
        let lo = vals[n / 2 - 1];
        let hi = vals[n / 2];
        lo + (hi - lo) / 2.0
    }
}

// ── Output formatting ──────────────────────────────────────────────────────

/// Format a float metric value as Picard does: strip trailing zeros, no scientific notation.
#[must_use]
pub fn fmt_pct(v: f64) -> String {
    if v == 0.0 {
        return "0".to_string();
    }
    let s = format!("{v:.6}");
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

/// Format the Picard-style METRICS data block (header + data row).
///
/// The `## htsjdk` preamble lines are written by `cli.rs` and excluded here
/// since they contain invocation-specific timestamps and command-line strings.
#[must_use]
pub fn format_metrics(metrics: &RnaSeqMetrics, bias: &BiasMetrics, has_rrna: bool) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    writeln!(
        out,
        "PF_BASES\tPF_ALIGNED_BASES\tRIBOSOMAL_BASES\tCODING_BASES\tUTR_BASES\t\
         INTRONIC_BASES\tINTERGENIC_BASES\tIGNORED_READS\tCORRECT_STRAND_READS\t\
         INCORRECT_STRAND_READS\tNUM_R1_TRANSCRIPT_STRAND_READS\t\
         NUM_R2_TRANSCRIPT_STRAND_READS\tNUM_UNEXPLAINED_READS\t\
         PCT_R1_TRANSCRIPT_STRAND_READS\tPCT_R2_TRANSCRIPT_STRAND_READS\t\
         PCT_RIBOSOMAL_BASES\tPCT_CODING_BASES\tPCT_UTR_BASES\tPCT_INTRONIC_BASES\t\
         PCT_INTERGENIC_BASES\tPCT_MRNA_BASES\tPCT_USABLE_BASES\t\
         PCT_CORRECT_STRAND_READS\tMEDIAN_CV_COVERAGE\tMEDIAN_5PRIME_BIAS\t\
         MEDIAN_3PRIME_BIAS\tMEDIAN_5PRIME_TO_3PRIME_BIAS\tSAMPLE\tLIBRARY\tREAD_GROUP"
    )
    .unwrap();

    let rrna_str = if has_rrna {
        metrics.ribosomal_bases.to_string()
    } else {
        String::new()
    };
    let pct_rrna_str = if has_rrna {
        fmt_pct(metrics.pct_ribosomal_bases())
    } else {
        String::new()
    };

    writeln!(
        out,
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t\t\t",
        metrics.pf_bases,
        metrics.pf_aligned_bases,
        rrna_str,
        metrics.coding_bases,
        metrics.utr_bases,
        metrics.intronic_bases,
        metrics.intergenic_bases,
        metrics.ignored_reads,
        metrics.correct_strand_reads,
        metrics.incorrect_strand_reads,
        metrics.num_r1_transcript_strand_reads,
        metrics.num_r2_transcript_strand_reads,
        metrics.num_unexplained_reads,
        fmt_pct(metrics.pct_r1_transcript_strand_reads()),
        fmt_pct(metrics.pct_r2_transcript_strand_reads()),
        pct_rrna_str,
        fmt_pct(metrics.pct_coding_bases()),
        fmt_pct(metrics.pct_utr_bases()),
        fmt_pct(metrics.pct_intronic_bases()),
        fmt_pct(metrics.pct_intergenic_bases()),
        fmt_pct(metrics.pct_mrna_bases()),
        fmt_pct(metrics.pct_usable_bases()),
        fmt_pct(metrics.pct_correct_strand_reads()),
        fmt_pct(bias.median_cv_coverage),
        fmt_pct(bias.median_5prime_bias),
        fmt_pct(bias.median_3prime_bias),
        fmt_pct(bias.median_5prime_to_3prime_bias),
    )
    .unwrap();

    out
}
