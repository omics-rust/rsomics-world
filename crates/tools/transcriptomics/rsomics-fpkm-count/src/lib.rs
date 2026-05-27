//! FPKM computation from BAM + BED12 gene model.
//!
//! Formula: FPKM = (Frag_count × 10⁹) / (mRNA_size × total_fragments)
//!          FPM  = (Frag_count / total_fragments) × 10⁶
//!
//! where mRNA_size = sum of exon block lengths from BED12 columns 10/11,
//! and total_fragments is the count of SE reads plus read1 of PE pairs.
//!
//! Fragment-to-gene assignment follows RSeQC's counting convention:
//! - Single-end reads: the full read span is checked against each exon interval.
//! - Paired-end reads: only read1 is processed; read2 is skipped entirely.
//!   Overlap is tested as a point check on the read1 start position (1 bp), not
//!   the full span — this matches FPKM_count.py's `exon_ranges.find(pos, pos+1)`.
//!
//! Reads/pairs hitting 2+ distinct genes are discarded as ambiguous.

#![allow(clippy::cast_precision_loss)]

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use noodles::bam;
use rsomics_common::{Result, RsomicsError};

/// One BED12 transcript record with exon intervals pre-computed from block geometry.
#[derive(Debug, Clone)]
pub struct Gene {
    pub chrom: String,
    pub tx_start: u64,
    pub tx_end: u64,
    pub name: String,
    pub strand: char,
    /// Sum of BED12 block sizes — the effective transcript length for the FPKM denominator.
    pub mrna_size: u64,
    /// Exon intervals in BED half-open [start, end) coordinates.
    pub exons: Vec<(u64, u64)>,
}

/// Strand specificity rules parsed from RSeQC's `-d` format.
///
/// Each token is two or three characters:
/// - Optional leading `1` (read1) or `2` (read2); absent → applies to both.
/// - Read strand: `+` = forward, `-` = reverse.
/// - Gene strand: `+` = forward, `-` = reverse.
///
/// Examples: `"1++,1--,2+-,2-+"`, `"++,--"`, `"+-,-+"`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StrandRules {
    /// Bitmask: bit index = (is_read2 as u8)*4 + (read_reverse as u8)*2 + (gene_minus as u8).
    mask: u8,
}

impl StrandRules {
    const fn bit(is_read2: bool, read_reverse: bool, gene_minus: bool) -> u8 {
        1 << ((is_read2 as u8) * 4 + (read_reverse as u8) * 2 + gene_minus as u8)
    }

    pub fn allows(&self, is_read2: bool, read_reverse: bool, gene_strand: char) -> bool {
        self.mask & Self::bit(is_read2, read_reverse, gene_strand == '-') != 0
    }

    pub fn parse(s: &str) -> Result<Option<Self>> {
        let s = s.trim();
        if s.is_empty() {
            return Ok(None);
        }
        let mut rules = Self::default();
        for token in s.split(',') {
            let token = token.trim();
            let (r2, rest) = if let Some(t) = token.strip_prefix('1') {
                (Some(false), t)
            } else if let Some(t) = token.strip_prefix('2') {
                (Some(true), t)
            } else {
                (None, token)
            };
            if rest.len() != 2 {
                return Err(RsomicsError::InvalidInput(format!(
                    "strand rule token {token:?}: expected 2-char strand pair after optional read number"
                )));
            }
            let read_fwd = rest.as_bytes()[0] == b'+';
            let gene_fwd = rest.as_bytes()[1] == b'+';
            match r2 {
                Some(is_r2) => {
                    rules.mask |= Self::bit(is_r2, !read_fwd, !gene_fwd);
                }
                None => {
                    rules.mask |= Self::bit(false, !read_fwd, !gene_fwd);
                    rules.mask |= Self::bit(true, !read_fwd, !gene_fwd);
                }
            }
        }
        Ok(Some(rules))
    }
}

pub struct FpkmOpts {
    /// Minimum MAPQ to be considered uniquely mapped.
    pub min_mapq: u8,
    /// Skip reads with MAPQ below `min_mapq` (mirrors `-u`).
    pub unique_only: bool,
    /// Strand specificity rules; `None` = unstranded.
    pub strand: Option<StrandRules>,
}

impl Default for FpkmOpts {
    fn default() -> Self {
        Self {
            min_mapq: 30,
            unique_only: false,
            strand: None,
        }
    }
}

/// Load a BED12 file and return one [`Gene`] per transcript record.
pub fn load_bed12(path: &Path) -> Result<Vec<Gene>> {
    let f = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let mut genes = Vec::new();
    for (i, line) in BufReader::new(f).lines().enumerate() {
        let line = line.map_err(RsomicsError::Io)?;
        let s = line.trim_end();
        if s.is_empty() || s.starts_with('#') || s.starts_with("track") || s.starts_with("browser")
        {
            continue;
        }
        genes.push(parse_bed12_line(s, i + 1)?);
    }
    Ok(genes)
}

fn parse_bed12_line(line: &str, lineno: usize) -> Result<Gene> {
    let mut f = line.split('\t');
    macro_rules! field {
        ($name:expr) => {
            f.next().ok_or_else(|| {
                RsomicsError::InvalidInput(format!("BED12 line {lineno}: missing {}", $name))
            })?
        };
    }
    let chrom = field!("chrom").to_string();
    let tx_start: u64 = field!("chromStart")
        .parse()
        .map_err(|e| RsomicsError::InvalidInput(format!("BED12 line {lineno}: chromStart: {e}")))?;
    let tx_end: u64 = field!("chromEnd")
        .parse()
        .map_err(|e| RsomicsError::InvalidInput(format!("BED12 line {lineno}: chromEnd: {e}")))?;
    let name = field!("name").to_string();
    let _score = field!("score");
    let strand = field!("strand").chars().next().unwrap_or('.');
    let _thick_start = field!("thickStart");
    let _thick_end = field!("thickEnd");
    let _item_rgb = field!("itemRgb");
    let block_count: usize = field!("blockCount")
        .parse()
        .map_err(|e| RsomicsError::InvalidInput(format!("BED12 line {lineno}: blockCount: {e}")))?;
    let sizes_raw = field!("blockSizes");
    let starts_raw = field!("blockStarts");

    let sizes = comma_u64(sizes_raw, block_count, lineno, "blockSizes")?;
    let starts = comma_u64(starts_raw, block_count, lineno, "blockStarts")?;

    let mut exons = Vec::with_capacity(block_count);
    let mut mrna_size = 0u64;
    for i in 0..block_count {
        let es = tx_start + starts[i];
        let ee = es + sizes[i];
        mrna_size += sizes[i];
        exons.push((es, ee));
    }

    Ok(Gene {
        chrom,
        tx_start,
        tx_end,
        name,
        strand,
        mrna_size,
        exons,
    })
}

fn comma_u64(s: &str, expected: usize, lineno: usize, field: &str) -> Result<Vec<u64>> {
    let mut v = Vec::with_capacity(expected);
    for p in s.split(',') {
        let p = p.trim();
        if p.is_empty() {
            continue;
        }
        v.push(p.parse::<u64>().map_err(|e| {
            RsomicsError::InvalidInput(format!("BED12 line {lineno}: {field} value {p:?}: {e}"))
        })?);
    }
    if v.len() != expected {
        return Err(RsomicsError::InvalidInput(format!(
            "BED12 line {lineno}: {field} has {} values, expected {expected}",
            v.len()
        )));
    }
    Ok(v)
}

/// Results of counting fragments from a BAM against a gene model.
pub struct CountResult {
    /// Fragment count per gene, parallel to the input `genes` slice.
    pub frag_counts: Vec<f64>,
    /// Total unique aligned fragments (pairs deduplicated by QNAME).
    pub total_fragments: f64,
}

/// Count fragments per gene from `bam_path`.
pub fn count_bam(bam_path: &Path, genes: &[Gene], opts: &FpkmOpts) -> Result<CountResult> {
    let exon_by_chrom = build_exon_index(genes);

    let file = File::open(bam_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", bam_path.display())))?;
    let mut reader = bam::io::Reader::new(file);
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let ref_names: Vec<String> = header
        .reference_sequences()
        .keys()
        .map(ToString::to_string)
        .collect();

    let mut frag_counts = vec![0_f64; genes.len()];
    let mut total_fragments: f64 = 0.0;

    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;
        let flags = record.flags();

        if flags.is_unmapped() || flags.is_secondary() || flags.is_supplementary() {
            continue;
        }

        let mq = record.mapping_quality().map_or(0, |q| q.get());
        if opts.unique_only && mq < opts.min_mapq {
            continue;
        }

        let is_paired = flags.is_segmented();
        let is_read2 = flags.is_last_segment();

        // RSeQC counts total_fragments as: SE reads + read1 of PE pairs.
        // read2 is skipped entirely for both total counting and gene assignment.
        if is_read2 {
            continue;
        }

        total_fragments += 1.0;

        let Some(tid) = record.reference_sequence_id().transpose().ok().flatten() else {
            continue;
        };
        let Some(chrom) = ref_names.get(tid) else {
            continue;
        };
        let Some(pos) = record.alignment_start().transpose().ok().flatten() else {
            continue;
        };
        let read_start = pos.get() as u64 - 1;
        // SE: check full read span; PE read1: point check on read start (RSeQC convention).
        let overlap_end = if is_paired {
            read_start + 1
        } else {
            read_start + record.sequence().len() as u64
        };
        let read_reverse = flags.is_reverse_complemented();

        let hits = exon_gene_hits(
            chrom,
            read_start,
            overlap_end,
            is_read2,
            read_reverse,
            genes,
            &exon_by_chrom,
            opts,
        );

        if hits.len() == 1 {
            frag_counts[hits[0]] += 1.0;
        }
    }

    Ok(CountResult {
        frag_counts,
        total_fragments,
    })
}

fn build_exon_index(genes: &[Gene]) -> HashMap<String, Vec<(u64, u64, usize)>> {
    let mut m: HashMap<String, Vec<(u64, u64, usize)>> = HashMap::new();
    for (gi, gene) in genes.iter().enumerate() {
        for &(es, ee) in &gene.exons {
            m.entry(gene.chrom.clone()).or_default().push((es, ee, gi));
        }
    }
    m
}

#[allow(clippy::too_many_arguments)]
fn exon_gene_hits(
    chrom: &str,
    read_start: u64,
    read_end: u64,
    is_read2: bool,
    read_reverse: bool,
    genes: &[Gene],
    exon_by_chrom: &HashMap<String, Vec<(u64, u64, usize)>>,
    opts: &FpkmOpts,
) -> Vec<usize> {
    let Some(exons) = exon_by_chrom.get(chrom) else {
        return Vec::new();
    };
    let mut hits: Vec<usize> = Vec::new();
    for &(es, ee, gi) in exons {
        if read_start >= ee || read_end <= es {
            continue;
        }
        if hits.contains(&gi) {
            continue;
        }
        if let Some(rules) = opts.strand
            && !rules.allows(is_read2, read_reverse, genes[gi].strand)
        {
            continue;
        }
        hits.push(gi);
    }
    hits
}

/// Format a float in Python-compatible notation: integer values include the `.0` suffix.
///
/// RSeQC uses Python's `str(float)` which renders `2.0` not `2`; this matches that format
/// so that downstream parsers expecting the `.0` suffix don't choke.
fn py_float(v: f64) -> String {
    if v.fract() == 0.0 && v.is_finite() {
        format!("{v}.0")
    } else {
        format!("{v}")
    }
}

/// Write the FPKM table to `output`.
///
/// Tab-separated with a `#`-prefixed header line:
/// `#chrom\tst\tend\taccession\tmRNA_size\tgene_strand\tFrag_count\tFPM\tFPKM`
pub fn write_fpkm<W: Write>(genes: &[Gene], result: &CountResult, output: W) -> Result<()> {
    let mut w = BufWriter::with_capacity(256 * 1024, output);
    writeln!(
        w,
        "#chrom\tst\tend\taccession\tmRNA_size\tgene_strand\tFrag_count\tFPM\tFPKM"
    )
    .map_err(RsomicsError::Io)?;

    let total = result.total_fragments;
    for (gene, &count) in genes.iter().zip(result.frag_counts.iter()) {
        let fpm = if total > 0.0 {
            count / total * 1_000_000.0
        } else {
            0.0
        };
        let fpkm = if total > 0.0 && gene.mrna_size > 0 {
            count * 1_000_000_000.0 / (gene.mrna_size as f64 * total)
        } else {
            0.0
        };
        writeln!(
            w,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            gene.chrom,
            gene.tx_start,
            gene.tx_end,
            gene.name,
            py_float(gene.mrna_size as f64),
            gene.strand,
            py_float(count),
            py_float(fpm),
            py_float(fpkm),
        )
        .map_err(RsomicsError::Io)?;
    }

    w.flush().map_err(RsomicsError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bed12_parse_simple() {
        let line = "chr1\t100\t500\tGENE1\t0\t+\t100\t500\t0\t2\t100,150,\t0,250,";
        let gene = parse_bed12_line(line, 1).unwrap();
        assert_eq!(gene.name, "GENE1");
        assert_eq!(gene.tx_start, 100);
        assert_eq!(gene.tx_end, 500);
        assert_eq!(gene.mrna_size, 250);
        assert_eq!(gene.exons, vec![(100, 200), (350, 500)]);
    }

    #[test]
    fn strand_rules_parse_paired_end() {
        let r = StrandRules::parse("1++,1--,2+-,2-+").unwrap().unwrap();
        assert!(r.allows(false, false, '+'));
        assert!(r.allows(false, true, '-'));
        assert!(r.allows(true, false, '-'));
        assert!(r.allows(true, true, '+'));
        assert!(!r.allows(false, false, '-'));
        assert!(!r.allows(true, false, '+'));
    }

    #[test]
    fn strand_rules_parse_single_end() {
        let r = StrandRules::parse("++,--").unwrap().unwrap();
        assert!(r.allows(false, false, '+'));
        assert!(r.allows(false, true, '-'));
        assert!(!r.allows(false, false, '-'));
    }

    #[test]
    fn strand_rules_none_on_empty() {
        assert!(StrandRules::parse("").unwrap().is_none());
    }

    #[test]
    fn fpkm_formula() {
        // Frag_count=2, total=4, mrna_size=250
        // FPM = 2/4 * 1e6 = 500000; FPKM = 2e9/(250*4) = 2000000
        let genes = vec![Gene {
            chrom: "chr1".into(),
            tx_start: 100,
            tx_end: 500,
            name: "G1".into(),
            strand: '+',
            mrna_size: 250,
            exons: vec![(100, 200), (350, 500)],
        }];
        let result = CountResult {
            frag_counts: vec![2.0],
            total_fragments: 4.0,
        };
        let mut buf = Vec::new();
        write_fpkm(&genes, &result, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("500000"), "{s}");
        assert!(s.contains("2000000"), "{s}");
    }
}
