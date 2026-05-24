//! Classify mapped reads into genomic regions from a BAM file + BED12 gene model.
//!
//! Mirrors the algorithm of `RSeQC` `read_distribution.py` (LGPL):
//!   - for each mapped, non-QC-fail, non-duplicate, non-secondary, non-unmapped read,
//!     split the read into exonic blocks using CIGAR (M/D/N/S ops);
//!   - classify each block by the midpoint of that block;
//!   - assign to the highest-priority overlapping feature (CDS > 5'UTR > 3'UTR > Intron
//!     > TSS up > TES down > unassigned);
//!   - emit "Total Reads / Total Tags / Total Assigned Tags" + feature table.
//!
//! ## Region construction (from BED12)
//!
//! CDS exons, 5'UTR exons, 3'UTR exons, introns are extracted per-transcript and
//! then union-merged + mutually subtracted in the same priority order `RSeQC` uses:
//!   1. CDS exons = merged CDS
//!   2. 5'UTR = merged 5'UTR − CDS
//!   3. 3'UTR = merged 3'UTR − CDS
//!   4. Introns = merged introns − CDS − 5'UTR − 3'UTR
//!   5. `TSS_up`/{`TES_down`} in 1/5/10 kb windows: each window minus all exonic/intronic features
//!
//! ## Origin
//!
//! This crate is an independent Rust reimplementation based on:
//! - `RSeQC`: `read_distribution.py` (LGPL-2.1+), Wang et al. 2012
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
const FLAG_QCFAIL: u16 = 0x0200;
const FLAG_DUPLICATE: u16 = 0x0400;
const FLAG_SECONDARY: u16 = 0x0100;
const FLAG_UNMAPPED: u16 = 0x0004;

/// CIGAR operation codes from the BAM spec.
const CIGAR_MATCH: u32 = 0; // M
const CIGAR_DEL: u32 = 2; // D
const CIGAR_REF_SKIP: u32 = 3; // N (intron in RNA-seq)
const CIGAR_SOFT_CLIP: u32 = 4; // S

/// One genomic interval [start, end) (half-open, 0-based).
#[derive(Clone, Debug)]
struct Iv {
    chrom: String,
    start: i32,
    end: i32,
}

/// Per-chromosome set of merged intervals, built as a `COITree`.
struct RegionIndex {
    /// Per-chromosome interval tree (end-inclusive, coitrees convention).
    trees: HashMap<String, COITree<(), u32>>,
    /// Total base count across all intervals in this set.
    total_bases: i64,
}

impl RegionIndex {
    fn from_intervals(mut ivs: Vec<Iv>) -> Self {
        // Sort + merge to union, then build trees.
        let merged = union_merge(&mut ivs);
        let total_bases: i64 = merged.iter().map(|iv| i64::from(iv.end - iv.start)).sum();

        let mut raw: HashMap<String, Vec<CoiInterval<()>>> = HashMap::new();
        for iv in &merged {
            raw.entry(iv.chrom.clone())
                .or_default()
                .push(CoiInterval::new(iv.start, iv.end - 1, ()));
        }
        let trees = raw
            .into_iter()
            .map(|(chrom, intervals)| (chrom, COITree::new(&intervals)))
            .collect();

        Self { trees, total_bases }
    }

    /// Returns `true` if `point` (0-based) overlaps any interval in this set.
    fn contains(&self, chrom: &str, point: i32) -> bool {
        let Some(tree) = self.trees.get(chrom) else {
            return false;
        };
        let mut found = false;
        tree.query(point, point, |_node| {
            found = true;
        });
        found
    }
}

/// Sort and merge overlapping/adjacent intervals in-place; returns merged list.
fn union_merge(ivs: &mut [Iv]) -> Vec<Iv> {
    if ivs.is_empty() {
        return Vec::new();
    }
    ivs.sort_unstable_by(|a, b| a.chrom.cmp(&b.chrom).then(a.start.cmp(&b.start)));
    let mut merged: Vec<Iv> = Vec::with_capacity(ivs.len());
    for iv in ivs.iter() {
        if let Some(last) = merged.last_mut()
            && last.chrom == iv.chrom
            && iv.start <= last.end
        {
            last.end = last.end.max(iv.end);
            continue;
        }
        merged.push(iv.clone());
    }
    merged
}

/// Subtract `minus` from `base` (set difference of interval lists).
///
/// Both inputs are assumed already merged (sorted, non-overlapping).
fn subtract_sorted(base: Vec<Iv>, minus: &[Iv]) -> Vec<Iv> {
    if minus.is_empty() {
        return base;
    }
    let mut result = Vec::with_capacity(base.len());
    for iv in base {
        let relevant: Vec<&Iv> = minus
            .iter()
            .filter(|m| m.chrom == iv.chrom && m.end > iv.start && m.start < iv.end)
            .collect();
        if relevant.is_empty() {
            result.push(iv);
            continue;
        }
        let mut cursor = iv.start;
        for m in &relevant {
            if m.start > cursor {
                result.push(Iv {
                    chrom: iv.chrom.clone(),
                    start: cursor,
                    end: m.start.min(iv.end),
                });
            }
            cursor = cursor.max(m.end);
            if cursor >= iv.end {
                break;
            }
        }
        if cursor < iv.end {
            result.push(Iv {
                chrom: iv.chrom.clone(),
                start: cursor,
                end: iv.end,
            });
        }
    }
    result
}

/// All feature-region indexes built from a BED12 gene model.
pub struct FeatureIndex {
    cds: RegionIndex,
    utr5: RegionIndex,
    utr3: RegionIndex,
    intron: RegionIndex,
    tss_up_1kb: RegionIndex,
    tss_up_5kb: RegionIndex,
    tss_up_10kb: RegionIndex,
    tes_down_1kb: RegionIndex,
    tes_down_5kb: RegionIndex,
    tes_down_10kb: RegionIndex,
}

impl FeatureIndex {
    /// Parse a BED12 file and build all feature indexes.
    #[allow(clippy::similar_names, clippy::too_many_lines)]
    pub fn from_bed12(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| RsomicsError::Io(std::io::Error::other(format!("reading BED12: {e}"))))?;

        let mut raw_cds: Vec<Iv> = Vec::new();
        let mut raw_utr5: Vec<Iv> = Vec::new();
        let mut raw_utr3: Vec<Iv> = Vec::new();
        let mut raw_intron: Vec<Iv> = Vec::new();
        // For TSS/TES: each gene contributes one upstream/downstream window.
        let mut raw_tss_up_1kb: Vec<Iv> = Vec::new();
        let mut raw_tss_up_5kb: Vec<Iv> = Vec::new();
        let mut raw_tss_up_10kb: Vec<Iv> = Vec::new();
        let mut raw_tes_down_1kb: Vec<Iv> = Vec::new();
        let mut raw_tes_down_5kb: Vec<Iv> = Vec::new();
        let mut raw_tes_down_10kb: Vec<Iv> = Vec::new();

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
            let Ok(tx_start) = fields[1].parse::<i32>() else {
                eprintln!("[NOTE:input bed must be 12-column] skipped this line: {line}");
                continue;
            };
            let Ok(tx_end) = fields[2].parse::<i32>() else {
                eprintln!("[NOTE:input bed must be 12-column] skipped this line: {line}");
                continue;
            };
            let strand = fields[5];
            let Ok(cds_start) = fields[6].parse::<i32>() else {
                continue;
            };
            let Ok(cds_end) = fields[7].parse::<i32>() else {
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
                .filter_map(|s| s.parse::<i32>().ok())
                .map(|o| o + tx_start)
                .collect();

            if block_sizes.len() < block_count || block_starts.len() < block_count {
                continue;
            }

            let exon_starts = &block_starts[..block_count];
            let exon_ends: Vec<i32> = exon_starts
                .iter()
                .zip(block_sizes.iter())
                .map(|(&s, &sz)| s + sz)
                .collect();

            // CDS exons: intersection of each exon with [cds_start, cds_end).
            for (&es, &ee) in exon_starts.iter().zip(exon_ends.iter()) {
                if ee <= cds_start || es >= cds_end {
                    continue;
                }
                let cds_exon_start = es.max(cds_start);
                let cds_exon_end = ee.min(cds_end);
                raw_cds.push(Iv {
                    chrom: chrom.to_uppercase(),
                    start: cds_exon_start,
                    end: cds_exon_end,
                });
            }

            // UTRs: per RSeQC getUTR logic (strand-aware).
            // For '+': 5'UTR = exon parts < cdsStart; 3'UTR = exon parts > cdsEnd
            // For '-': 3'UTR = exon parts < cdsStart; 5'UTR = exon parts > cdsEnd
            for (&es, &ee) in exon_starts.iter().zip(exon_ends.iter()) {
                // 5' side (5'UTR for '+', 3'UTR for '-')
                if es < cds_start {
                    let utr_st = es;
                    let utr_end = ee.min(cds_start);
                    let iv = Iv {
                        chrom: chrom.to_uppercase(),
                        start: utr_st,
                        end: utr_end,
                    };
                    if strand == "+" {
                        raw_utr5.push(iv);
                    } else {
                        raw_utr3.push(iv);
                    }
                }
                // 3' side (3'UTR for '+', 5'UTR for '-')
                if ee > cds_end {
                    let utr_st = es.max(cds_end);
                    let utr_end = ee;
                    let iv = Iv {
                        chrom: chrom.to_uppercase(),
                        start: utr_st,
                        end: utr_end,
                    };
                    if strand == "+" {
                        raw_utr3.push(iv);
                    } else {
                        raw_utr5.push(iv);
                    }
                }
            }

            // Introns: gaps between consecutive exons (only when block_count > 1).
            if block_count > 1 {
                for i in 0..(block_count - 1) {
                    let intron_st = exon_ends[i];
                    let intron_end = exon_starts[i + 1];
                    if intron_st < intron_end {
                        raw_intron.push(Iv {
                            chrom: chrom.to_uppercase(),
                            start: intron_st,
                            end: intron_end,
                        });
                    }
                }
            }

            // TSS upstream / TES downstream windows.
            // Per RSeQC getIntergenic: "up" = upstream of TSS; "down" = downstream of TES.
            // For '+': TSS = tx_start, TES = tx_end
            //   upstream window: [max(0, tx_start - size), tx_start)
            //   downstream window: [tx_end, tx_end + size)
            // For '-': TSS = tx_end, TES = tx_start
            //   upstream window: [tx_end, tx_end + size)
            //   downstream window: [max(0, tx_start - size), tx_start)
            for &size in &[1000i32, 5000, 10000] {
                let (up_st, up_end, down_st, down_end) = if strand == "-" {
                    (tx_end, tx_end + size, 0i32.max(tx_start - size), tx_start)
                } else {
                    (0i32.max(tx_start - size), tx_start, tx_end, tx_end + size)
                };
                let up_iv = Iv {
                    chrom: chrom.to_uppercase(),
                    start: up_st,
                    end: up_end,
                };
                let down_iv = Iv {
                    chrom: chrom.to_uppercase(),
                    start: down_st,
                    end: down_end,
                };
                match size {
                    1000 => {
                        raw_tss_up_1kb.push(up_iv);
                        raw_tes_down_1kb.push(down_iv);
                    }
                    5000 => {
                        raw_tss_up_5kb.push(up_iv);
                        raw_tes_down_5kb.push(down_iv);
                    }
                    _ => {
                        raw_tss_up_10kb.push(up_iv);
                        raw_tes_down_10kb.push(down_iv);
                    }
                }
            }
        }

        // Merge each feature set.
        let cds_merged = union_merge(&mut raw_cds);
        let utr5_merged = union_merge(&mut raw_utr5);
        let utr3_merged = union_merge(&mut raw_utr3);
        let intron_merged = union_merge(&mut raw_intron);
        let tss_up_1kb_merged = union_merge(&mut raw_tss_up_1kb);
        let tss_up_5kb_merged = union_merge(&mut raw_tss_up_5kb);
        let tss_up_10kb_merged = union_merge(&mut raw_tss_up_10kb);
        let tes_down_1kb_merged = union_merge(&mut raw_tes_down_1kb);
        let tes_down_5kb_merged = union_merge(&mut raw_tes_down_5kb);
        let tes_down_10kb_merged = union_merge(&mut raw_tes_down_10kb);

        // Subtract higher-priority regions from lower-priority ones.
        // Priority: CDS > 5'UTR > 3'UTR > Intron > TSS/TES windows.
        let utr5_clean = subtract_sorted(utr5_merged.clone(), &cds_merged);
        let utr3_clean = subtract_sorted(utr3_merged.clone(), &cds_merged);
        let intron_clean = {
            let tmp = subtract_sorted(intron_merged.clone(), &cds_merged);
            let tmp = subtract_sorted(tmp, &utr5_merged);
            subtract_sorted(tmp, &utr3_merged)
        };

        // All genic regions combined for subtracting from TSS/TES windows.
        let mut all_genic: Vec<Iv> = Vec::new();
        all_genic.extend_from_slice(&cds_merged);
        all_genic.extend_from_slice(&utr5_merged);
        all_genic.extend_from_slice(&utr3_merged);
        all_genic.extend_from_slice(&intron_merged);
        let all_genic_merged = union_merge(&mut all_genic);

        let clean_window = |mut ivs: Vec<Iv>| -> Vec<Iv> {
            let merged = union_merge(&mut ivs);
            subtract_sorted(merged, &all_genic_merged)
        };

        let tss_up_1kb_clean = clean_window(tss_up_1kb_merged);
        let tss_up_5kb_clean = clean_window(tss_up_5kb_merged);
        let tss_up_10kb_clean = clean_window(tss_up_10kb_merged);
        let tes_down_1kb_clean = clean_window(tes_down_1kb_merged);
        let tes_down_5kb_clean = clean_window(tes_down_5kb_merged);
        let tes_down_10kb_clean = clean_window(tes_down_10kb_merged);

        Ok(Self {
            cds: RegionIndex::from_intervals(cds_merged),
            utr5: RegionIndex::from_intervals(utr5_clean),
            utr3: RegionIndex::from_intervals(utr3_clean),
            intron: RegionIndex::from_intervals(intron_clean),
            tss_up_1kb: RegionIndex::from_intervals(tss_up_1kb_clean),
            tss_up_5kb: RegionIndex::from_intervals(tss_up_5kb_clean),
            tss_up_10kb: RegionIndex::from_intervals(tss_up_10kb_clean),
            tes_down_1kb: RegionIndex::from_intervals(tes_down_1kb_clean),
            tes_down_5kb: RegionIndex::from_intervals(tes_down_5kb_clean),
            tes_down_10kb: RegionIndex::from_intervals(tes_down_10kb_clean),
        })
    }
}

/// Counters for each genomic region.
#[derive(Debug, Default, Serialize)]
pub struct ReadDistResult {
    pub total_reads: u64,
    pub total_tags: u64,
    pub unassigned_tags: u64,

    pub cds_exons_bases: i64,
    pub cds_exons_tags: u64,
    pub utr5_bases: i64,
    pub utr5_tags: u64,
    pub utr3_bases: i64,
    pub utr3_tags: u64,
    pub intron_bases: i64,
    pub intron_tags: u64,

    pub tss_up_1kb_bases: i64,
    pub tss_up_1kb_tags: u64,
    pub tss_up_5kb_bases: i64,
    pub tss_up_5kb_tags: u64,
    pub tss_up_10kb_bases: i64,
    pub tss_up_10kb_tags: u64,

    pub tes_down_1kb_bases: i64,
    pub tes_down_1kb_tags: u64,
    pub tes_down_5kb_bases: i64,
    pub tes_down_5kb_tags: u64,
    pub tes_down_10kb_bases: i64,
    pub tes_down_10kb_tags: u64,
}

impl ReadDistResult {
    #[must_use]
    pub fn assigned_tags(&self) -> u64 {
        self.total_tags - self.unassigned_tags
    }

    /// Emit the exact text format `RSeQC` `read_distribution.py` prints to stdout.
    ///
    /// Format string matches Python `"%-30s%d"` for header lines and
    /// `"%-20s%-20d%-20d%-18.2f"` for table rows (with `+1` denominator
    /// to avoid division by zero, matching `RSeQC` source exactly).
    pub fn write_rseqc<W: Write>(&self, mut out: W) -> std::io::Result<()> {
        writeln!(out, "{:<30}{}", "Total Reads", self.total_reads)?;
        writeln!(out, "{:<30}{}", "Total Tags", self.total_tags)?;
        writeln!(out, "{:<30}{}", "Total Assigned Tags", self.assigned_tags())?;
        writeln!(
            out,
            "====================================================================="
        )?;
        writeln!(
            out,
            "{:<20}{:<20}{:<20}{:<20}",
            "Group", "Total_bases", "Tag_count", "Tags/Kb"
        )?;
        let row = |out: &mut W, name: &str, bases: i64, tags: u64| -> std::io::Result<()> {
            let tags_per_kb = tags as f64 * 1000.0 / (bases + 1) as f64;
            writeln!(out, "{name:<20}{bases:<20}{tags:<20}{tags_per_kb:<18.2}")
        };
        row(
            &mut out,
            "CDS_Exons",
            self.cds_exons_bases,
            self.cds_exons_tags,
        )?;
        row(&mut out, "5'UTR_Exons", self.utr5_bases, self.utr5_tags)?;
        row(&mut out, "3'UTR_Exons", self.utr3_bases, self.utr3_tags)?;
        row(&mut out, "Introns", self.intron_bases, self.intron_tags)?;
        row(
            &mut out,
            "TSS_up_1kb",
            self.tss_up_1kb_bases,
            self.tss_up_1kb_tags,
        )?;
        row(
            &mut out,
            "TSS_up_5kb",
            self.tss_up_5kb_bases,
            self.tss_up_5kb_tags,
        )?;
        row(
            &mut out,
            "TSS_up_10kb",
            self.tss_up_10kb_bases,
            self.tss_up_10kb_tags,
        )?;
        row(
            &mut out,
            "TES_down_1kb",
            self.tes_down_1kb_bases,
            self.tes_down_1kb_tags,
        )?;
        row(
            &mut out,
            "TES_down_5kb",
            self.tes_down_5kb_bases,
            self.tes_down_5kb_tags,
        )?;
        row(
            &mut out,
            "TES_down_10kb",
            self.tes_down_10kb_bases,
            self.tes_down_10kb_tags,
        )?;
        writeln!(
            out,
            "====================================================================="
        )?;
        Ok(())
    }
}

/// Exonic blocks for one read derived from the CIGAR string.
///
/// Mirrors `bam_cigar.fetch_exon`: M advances position + emits block;
/// I is consumed (no ref advance); D/N/S advance ref without emitting.
fn fetch_exon_blocks(start: i32, cigar_ops: impl Iterator<Item = (u8, u32)>) -> Vec<(i32, i32)> {
    let mut blocks = Vec::new();
    let mut pos = start;
    for (op, len) in cigar_ops {
        #[allow(clippy::cast_possible_wrap)]
        let len = len as i32;
        match u32::from(op) {
            CIGAR_MATCH => {
                blocks.push((pos, pos + len));
                pos += len;
            }
            CIGAR_DEL | CIGAR_REF_SKIP | CIGAR_SOFT_CLIP => {
                pos += len;
            }
            _ => {}
        }
    }
    blocks
}

/// Run the full read distribution analysis.
pub fn run_read_dist(
    bam_path: &Path,
    bed_path: &Path,
    workers: NonZero<usize>,
) -> Result<ReadDistResult> {
    eprintln!("processing {} ...", bed_path.display());
    let index = FeatureIndex::from_bed12(bed_path)?;
    eprintln!("Done");

    eprintln!("processing {} ...", bam_path.display());

    let mut reader = rsomics_bamio::open_with_workers(bam_path, workers)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let ref_names: Vec<String> = header
        .reference_sequences()
        .keys()
        .map(|k| String::from_utf8_lossy(k.as_ref()).into_owned())
        .collect();

    let mut result = ReadDistResult {
        cds_exons_bases: index.cds.total_bases,
        utr5_bases: index.utr5.total_bases,
        utr3_bases: index.utr3.total_bases,
        intron_bases: index.intron.total_bases,
        tss_up_1kb_bases: index.tss_up_1kb.total_bases,
        tss_up_5kb_bases: index.tss_up_5kb.total_bases,
        tss_up_10kb_bases: index.tss_up_10kb.total_bases,
        tes_down_1kb_bases: index.tes_down_1kb.total_bases,
        tes_down_5kb_bases: index.tes_down_5kb.total_bases,
        tes_down_10kb_bases: index.tes_down_10kb.total_bases,
        ..Default::default()
    };

    let mut rec = RawRecord::default();
    loop {
        let bytes_read = raw::read_record(reader.get_mut(), &mut rec)?;
        if bytes_read == 0 {
            break;
        }

        let flags = rec.flags();
        if flags & (FLAG_QCFAIL | FLAG_DUPLICATE | FLAG_SECONDARY | FLAG_UNMAPPED) != 0 {
            continue;
        }

        result.total_reads += 1;

        let tid = rec.reference_sequence_id();
        if tid < 0 {
            continue;
        }
        #[allow(clippy::cast_sign_loss)]
        let Some(chrom) = ref_names.get(tid as usize) else {
            continue;
        };
        let chrom_upper = chrom.to_uppercase();

        let read_start = rec.alignment_start();
        let blocks = fetch_exon_blocks(read_start, rec.cigar_ops());
        result.total_tags += blocks.len() as u64;

        for (bstart, bend) in blocks {
            // Midpoint classification, matching RSeQC: int(st) + int((int(end)-int(st))/2)
            let mid = bstart + (bend - bstart) / 2;
            classify_tag(mid, &chrom_upper, &index, &mut result);
        }
    }

    eprintln!("Finished");

    Ok(result)
}

/// Classify one tag (exon block midpoint) and increment the appropriate counter.
///
/// Priority order exactly matches `RSeQC` source:
///   CDS → 5'UTR (exclusive) → 3'UTR (exclusive) → both UTR → intron →
///   both TSS+TES up+down 10kb → `TSS_up`/`TES_down` in 1/5/10kb → unassigned
fn classify_tag(mid: i32, chrom: &str, idx: &FeatureIndex, res: &mut ReadDistResult) {
    if idx.cds.contains(chrom, mid) {
        res.cds_exons_tags += 1;
        return;
    }
    let in_utr5 = idx.utr5.contains(chrom, mid);
    let in_utr3 = idx.utr3.contains(chrom, mid);
    if in_utr5 && !in_utr3 {
        res.utr5_tags += 1;
        return;
    }
    if in_utr3 && !in_utr5 {
        res.utr3_tags += 1;
        return;
    }
    if in_utr5 && in_utr3 {
        // Overlapping UTR5 and UTR3 — unassigned per RSeQC.
        res.unassigned_tags += 1;
        return;
    }
    if idx.intron.contains(chrom, mid) {
        res.intron_tags += 1;
        return;
    }
    if idx.tss_up_10kb.contains(chrom, mid) && idx.tes_down_10kb.contains(chrom, mid) {
        // Overlaps both TSS and TES 10kb windows — unassigned per RSeQC.
        res.unassigned_tags += 1;
        return;
    }
    if idx.tss_up_1kb.contains(chrom, mid) {
        res.tss_up_1kb_tags += 1;
        res.tss_up_5kb_tags += 1;
        res.tss_up_10kb_tags += 1;
        return;
    }
    if idx.tss_up_5kb.contains(chrom, mid) {
        res.tss_up_5kb_tags += 1;
        res.tss_up_10kb_tags += 1;
        return;
    }
    if idx.tss_up_10kb.contains(chrom, mid) {
        res.tss_up_10kb_tags += 1;
        return;
    }
    if idx.tes_down_1kb.contains(chrom, mid) {
        res.tes_down_1kb_tags += 1;
        res.tes_down_5kb_tags += 1;
        res.tes_down_10kb_tags += 1;
        return;
    }
    if idx.tes_down_5kb.contains(chrom, mid) {
        res.tes_down_5kb_tags += 1;
        res.tes_down_10kb_tags += 1;
        return;
    }
    if idx.tes_down_10kb.contains(chrom, mid) {
        res.tes_down_10kb_tags += 1;
        return;
    }
    res.unassigned_tags += 1;
}
