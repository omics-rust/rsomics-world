//! Subsample-based RPKM saturation analysis.
//!
//! Algorithm (black-box reconstruction from RSeQC docs + observed behaviour):
//! 1. Load all mapped, primary, non-duplicate alignments from the BAM.
//! 2. Parse BED12 gene models; for each gene, expand exons.
//! 3. For each fraction F in [lower..upper] step S:
//!    a. Randomly sample ⌊F% × total_reads⌋ read indices (without replacement).
//!    b. For each sampled read, find all BED12 genes it overlaps (any exon).
//!    c. Per gene: compute RPKM = count / (exon_kb * mapped_millions).
//! 4. Write two TSV files:
//!    - `<prefix>.eRPKM.xls`: RPKM per gene per fraction (BED6 + fraction columns)
//!    - `<prefix>.rawCount.xls`: raw counts per gene per fraction

use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::num::NonZero;
use std::path::Path;

use noodles::bam;
use noodles::sam::alignment::Record as _;
use rand::SeedableRng;
use rand::seq::index::sample as index_sample;
use rand_chacha::ChaCha12Rng;
#[allow(clippy::wildcard_imports)]
use rayon::prelude::*;
use rsomics_common::{Result, RsomicsError};

/// A single BED12 gene record with pre-expanded exon intervals.
#[derive(Debug, Clone)]
pub struct Gene {
    pub chrom: String,
    pub tx_start: u64,
    pub tx_end: u64,
    pub name: String,
    pub score: String,
    pub strand: char,
    /// Absolute genomic coordinates of each exon: (start, end) 0-based half-open.
    pub exons: Vec<(u64, u64)>,
    /// Total exon length in bases.
    pub exon_length: u64,
}

impl Gene {
    fn overlaps_read(&self, chrom: &str, read_start: u64, read_end: u64) -> bool {
        if self.chrom != chrom {
            return false;
        }
        self.exons
            .iter()
            .any(|&(es, ee)| read_start < ee && read_end > es)
    }
}

/// Parse BED12 from a buffered reader.
pub fn load_genes(path: &Path) -> Result<Vec<Gene>> {
    let f = std::fs::File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let reader = std::io::BufReader::new(f);
    let mut genes = Vec::new();
    for (lineno, line) in reader.lines().enumerate() {
        let line = line.map_err(RsomicsError::Io)?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("track") {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 12 {
            return Err(RsomicsError::InvalidInput(format!(
                "BED12 requires 12 columns at line {}; got {}",
                lineno + 1,
                cols.len()
            )));
        }
        let chrom = cols[0].to_string();
        let tx_start: u64 = cols[1]
            .parse()
            .map_err(|_| RsomicsError::InvalidInput(format!("bad start at line {}", lineno + 1)))?;
        let tx_end: u64 = cols[2]
            .parse()
            .map_err(|_| RsomicsError::InvalidInput(format!("bad end at line {}", lineno + 1)))?;
        let name = cols[3].to_string();
        let score = cols[4].to_string();
        let strand = cols[5].chars().next().unwrap_or('.');
        let block_count: usize = cols[9].parse().map_err(|_| {
            RsomicsError::InvalidInput(format!("bad blockCount at line {}", lineno + 1))
        })?;
        let block_sizes: Vec<u64> = cols[10]
            .trim_end_matches(',')
            .split(',')
            .map(|s| {
                s.trim().parse::<u64>().map_err(|_| {
                    RsomicsError::InvalidInput(format!("bad blockSize at line {}", lineno + 1))
                })
            })
            .collect::<Result<_>>()?;
        let block_starts: Vec<u64> = cols[11]
            .trim_end_matches(',')
            .split(',')
            .map(|s| {
                s.trim().parse::<u64>().map_err(|_| {
                    RsomicsError::InvalidInput(format!("bad blockStart at line {}", lineno + 1))
                })
            })
            .collect::<Result<_>>()?;
        if block_sizes.len() < block_count || block_starts.len() < block_count {
            return Err(RsomicsError::InvalidInput(format!(
                "blockCount mismatch at line {}",
                lineno + 1
            )));
        }
        let exons: Vec<(u64, u64)> = (0..block_count)
            .map(|i| {
                let start = tx_start + block_starts[i];
                let end = start + block_sizes[i];
                (start, end)
            })
            .collect();
        let exon_length: u64 = exons.iter().map(|&(s, e)| e - s).sum();
        genes.push(Gene {
            chrom,
            tx_start,
            tx_end,
            name,
            score,
            strand,
            exons,
            exon_length,
        });
    }
    Ok(genes)
}

/// A lightweight read record for subsampling.
#[derive(Debug, Clone)]
pub struct ReadRecord {
    pub chrom: String,
    pub start: u64,
    pub end: u64,
}

/// Load all primary mapped reads (passing MAPQ threshold) from a BAM.
pub fn load_reads(
    bam_path: &Path,
    min_mapq: u8,
    workers: NonZero<usize>,
) -> Result<Vec<ReadRecord>> {
    let mut reader = rsomics_bamio::open_with_workers(bam_path, workers)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let mut records = Vec::new();
    let mut buf = bam::Record::default();

    loop {
        match reader.read_record(&mut buf) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => return Err(RsomicsError::Io(e)),
        }
        let flags = buf.flags();
        // Skip unmapped, secondary, supplementary
        if flags.is_unmapped()
            || flags.is_secondary()
            || flags.is_supplementary()
            || flags.is_duplicate()
        {
            continue;
        }
        let mapq = buf.mapping_quality().map(|q| q.get()).unwrap_or(0);
        if mapq < min_mapq {
            continue;
        }
        let Some(ref_seq_id) = buf.reference_sequence_id().and_then(|r| r.ok()) else {
            continue;
        };
        let chrom = header
            .reference_sequences()
            .get_index(ref_seq_id)
            .map(|(name, _)| name.to_string())
            .ok_or_else(|| {
                RsomicsError::InvalidInput(format!("unknown ref seq id {ref_seq_id}"))
            })?;
        let start = buf
            .alignment_start()
            .and_then(|p| p.ok())
            .map(|p| p.get() as u64 - 1)
            .unwrap_or(0);
        let end = buf
            .alignment_end()
            .and_then(|p| p.ok())
            .map(|p| p.get() as u64)
            .unwrap_or(start + 1);
        records.push(ReadRecord { chrom, start, end });
    }
    Ok(records)
}

pub struct SaturationOpts {
    pub lower_pct: u8,
    pub upper_pct: u8,
    pub step_pct: u8,
    pub min_mapq: u8,
    pub seed: u64,
}

impl Default for SaturationOpts {
    fn default() -> Self {
        Self {
            lower_pct: 5,
            upper_pct: 100,
            step_pct: 5,
            min_mapq: 30,
            seed: 0,
        }
    }
}

pub struct FractionResult {
    pub pct: u8,
    /// Per-gene raw counts indexed by gene position in `genes` slice.
    pub raw_counts: Vec<u64>,
    /// Per-gene RPKM values.
    pub rpkm: Vec<f64>,
}

/// Compute RPKM saturation across all fractions. Returns one `FractionResult`
/// per fraction, in the order of `fractions`.
#[allow(clippy::cast_precision_loss)]
pub fn compute_saturation(
    reads: &[ReadRecord],
    genes: &[Gene],
    opts: &SaturationOpts,
) -> Result<Vec<FractionResult>> {
    if genes.is_empty() || reads.is_empty() {
        return Ok(Vec::new());
    }

    let fractions: Vec<u8> = {
        let mut v = Vec::new();
        let mut pct = opts.lower_pct;
        while pct <= opts.upper_pct {
            v.push(pct);
            pct = pct.saturating_add(opts.step_pct);
            if pct > opts.upper_pct && v.last() != Some(&opts.upper_pct) {
                v.push(opts.upper_pct);
                break;
            }
        }
        v
    };

    // Build a chrom → gene index for fast overlap lookup.
    // Map: chrom → sorted vec of (tx_start, tx_end, gene_idx)
    let mut chrom_index: HashMap<String, Vec<(u64, u64, usize)>> = HashMap::new();
    for (idx, gene) in genes.iter().enumerate() {
        chrom_index
            .entry(gene.chrom.clone())
            .or_default()
            .push((gene.tx_start, gene.tx_end, idx));
    }
    // Sort each chrom list by start for binary-search lower bound.
    for v in chrom_index.values_mut() {
        v.sort_unstable_by_key(|&(s, _, _)| s);
    }

    let total = reads.len();

    // Process fractions in parallel via rayon.
    let results: Vec<FractionResult> = fractions
        .into_par_iter()
        .map(|pct| {
            let sample_size = ((pct as f64 / 100.0) * total as f64).round() as usize;
            let sample_size = sample_size.min(total);

            // Deterministic per-fraction RNG: seed derived from global seed + pct.
            // Matches RSeQC's sampling cardinality precisely; the exact draw order
            // differs (Python random vs ChaCha12) so RPKM values differ across
            // runs — compat tests use numerical tolerance not byte equality.
            let fraction_seed = opts.seed.wrapping_add(u64::from(pct));
            let mut rng = ChaCha12Rng::seed_from_u64(fraction_seed);

            let sampled = index_sample(&mut rng, total, sample_size);

            // Accumulate per-gene raw counts.
            let mut raw_counts = vec![0u64; genes.len()];
            for idx in sampled.iter() {
                let read = &reads[idx];
                let Some(gene_list) = chrom_index.get(&read.chrom) else {
                    continue;
                };
                // List is sorted by tx_start. Find the first gene that starts at or
                // after read.end — everything before that index could overlap.
                let upper = gene_list.partition_point(|&(s, _, _)| s < read.end);
                for &(_gs, ge, gi) in &gene_list[..upper] {
                    if ge <= read.start {
                        // This gene ends before the read starts — no overlap.
                        continue;
                    }
                    // Coarse tx-level overlap confirmed; now check exon-level.
                    if genes[gi].overlaps_read(&read.chrom, read.start, read.end) {
                        raw_counts[gi] += 1;
                    }
                }
            }

            let total_mapped_millions = sample_size as f64 / 1_000_000.0;
            let rpkm: Vec<f64> = genes
                .iter()
                .zip(raw_counts.iter())
                .map(|(gene, &count)| {
                    let gene_length_kb = gene.exon_length as f64 / 1000.0;
                    if gene_length_kb == 0.0 || total_mapped_millions == 0.0 {
                        0.0
                    } else {
                        count as f64 / (gene_length_kb * total_mapped_millions)
                    }
                })
                .collect();

            FractionResult {
                pct,
                raw_counts,
                rpkm,
            }
        })
        .collect();

    Ok(results)
}

/// Write `<prefix>.eRPKM.xls` and `<prefix>.rawCount.xls`.
pub fn write_outputs(prefix: &str, genes: &[Gene], results: &[FractionResult]) -> Result<()> {
    let rpkm_path = format!("{prefix}.eRPKM.xls");
    let raw_path = format!("{prefix}.rawCount.xls");

    let pcts: Vec<u8> = results.iter().map(|r| r.pct).collect();
    let header_suffix: String = pcts
        .iter()
        .map(|p| format!("{p}%"))
        .collect::<Vec<_>>()
        .join("\t");
    let header = format!("#chr\tstart\tend\tname\tscore\tstrand\t{header_suffix}\n");

    let mut rpkm_out = std::fs::File::create(&rpkm_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("creating {rpkm_path}: {e}")))?;
    let mut raw_out = std::fs::File::create(&raw_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("creating {raw_path}: {e}")))?;

    rpkm_out
        .write_all(header.as_bytes())
        .map_err(RsomicsError::Io)?;
    raw_out
        .write_all(header.as_bytes())
        .map_err(RsomicsError::Io)?;

    for (gi, gene) in genes.iter().enumerate() {
        let rpkm_vals: String = results
            .iter()
            .map(|r| format!(" {:.15e}", r.rpkm[gi]))
            .collect::<Vec<_>>()
            .join("\t");
        let raw_vals: String = results
            .iter()
            .map(|r| format!(" {}", r.raw_counts[gi]))
            .collect::<Vec<_>>()
            .join("\t");

        let prefix_cols = format!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            gene.chrom, gene.tx_start, gene.tx_end, gene.name, gene.score, gene.strand
        );
        writeln!(rpkm_out, "{prefix_cols}\t{rpkm_vals}").map_err(RsomicsError::Io)?;
        writeln!(raw_out, "{prefix_cols}\t{raw_vals}").map_err(RsomicsError::Io)?;
    }

    Ok(())
}
