//! FASTQ random subsampling — Bernoulli (fraction) and reservoir (exact count) modes.
//!
//! ## Modes
//!
//! - **Fraction mode** (`-p`): Bernoulli trial per record with probability `p`. Output size is
//!   approximately `p × input_size`. Single-pass, O(1) memory, identical to `seqtk sample` with
//!   `-f` or `seqkit sample -p`.
//!
//! - **Exact-count mode** (`-n`): Reservoir sampling (Vitter's Algorithm R). Output size is
//!   exactly `n` records (or all records if input has fewer than `n`). Requires loading `n` records
//!   into memory; input is streamed single-pass.
//!
//! Both modes accept paired-end reads: supply R1 and R2 paths; the same random decisions apply to
//! both files so mate-pair correspondence is preserved.

use std::io::{BufWriter, Write};
use std::path::Path;

use needletail::parse_fastx_file;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rand::seq::index::sample as reservoir_sample;

use rsomics_common::{Result, RsomicsError};

/// How many records to include in the output.
#[derive(Debug, Clone, Copy)]
pub enum SampleMode {
    /// Include each record independently with probability `p` (Bernoulli trial).
    Fraction(f64),
    /// Include exactly `n` records using reservoir sampling (fewer if input < n).
    Exact(u64),
}

/// Open a writer that gzip-compresses if the path ends with `.gz`, otherwise plain.
fn open_writer(path: &Path) -> Result<Box<dyn Write>> {
    let raw = std::fs::File::create(path).map_err(RsomicsError::Io)?;
    if path.extension().and_then(|e| e.to_str()) == Some("gz") {
        Ok(Box::new(BufWriter::new(flate2::write::GzEncoder::new(
            raw,
            flate2::Compression::default(),
        ))))
    } else {
        Ok(Box::new(BufWriter::new(raw)))
    }
}

/// Fraction-mode subsampling for a single FASTQ stream.
///
/// Writes records to `out` with independent Bernoulli probability `p`.
/// Returns `(total, kept)`.
fn sample_fraction_single(
    input: &Path,
    p: f64,
    rng: &mut SmallRng,
    out: &mut impl Write,
) -> Result<(u64, u64)> {
    let mut reader = parse_fastx_file(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("cannot open {}: {e}", input.display())))?;
    let (mut total, mut kept) = (0u64, 0u64);
    while let Some(record) = reader.next() {
        let rec = record
            .map_err(|e| RsomicsError::InvalidInput(format!("malformed FASTQ record: {e}")))?;
        total += 1;
        if rng.random::<f64>() < p {
            rec.write(&mut *out, None)
                .map_err(|e| RsomicsError::InvalidInput(e.to_string()))?;
            kept += 1;
        }
    }
    Ok((total, kept))
}

/// Fraction-mode for paired-end: R1 and R2 use the same Bernoulli decisions.
fn sample_fraction_paired(
    r1: &Path,
    r2: &Path,
    p: f64,
    rng: &mut SmallRng,
    out1: &mut impl Write,
    out2: &mut impl Write,
) -> Result<(u64, u64)> {
    let mut reader1 = parse_fastx_file(r1)
        .map_err(|e| RsomicsError::InvalidInput(format!("cannot open {}: {e}", r1.display())))?;
    let mut reader2 = parse_fastx_file(r2)
        .map_err(|e| RsomicsError::InvalidInput(format!("cannot open {}: {e}", r2.display())))?;
    let (mut total, mut kept) = (0u64, 0u64);
    loop {
        let rec1 = reader1.next();
        let rec2 = reader2.next();
        match (rec1, rec2) {
            (None, None) => break,
            (Some(r1), Some(r2)) => {
                let r1 = r1.map_err(|e| {
                    RsomicsError::InvalidInput(format!("malformed R1 FASTQ record: {e}"))
                })?;
                let r2 = r2.map_err(|e| {
                    RsomicsError::InvalidInput(format!("malformed R2 FASTQ record: {e}"))
                })?;
                total += 1;
                if rng.random::<f64>() < p {
                    r1.write(&mut *out1, None)
                        .map_err(|e| RsomicsError::InvalidInput(e.to_string()))?;
                    r2.write(&mut *out2, None)
                        .map_err(|e| RsomicsError::InvalidInput(e.to_string()))?;
                    kept += 1;
                }
            }
            _ => {
                return Err(RsomicsError::InvalidInput(
                    "R1 and R2 have different record counts".into(),
                ));
            }
        }
    }
    Ok((total, kept))
}

/// Buffer that stores raw FASTQ record bytes for reservoir sampling.
struct RawRecord(Vec<u8>);

impl RawRecord {
    fn from_needletail(rec: &needletail::parser::SequenceRecord) -> Result<Self> {
        let mut buf = Vec::with_capacity(256);
        rec.write(&mut buf, None)
            .map_err(|e| RsomicsError::InvalidInput(e.to_string()))?;
        Ok(Self(buf))
    }
}

/// Exact-count reservoir sampling for a single FASTQ stream.
///
/// Reads the entire input to collect a reservoir of `n` records, then writes them in
/// input order (sorted by original position so the output order is stable).
fn sample_exact_single(
    input: &Path,
    n: u64,
    rng: &mut SmallRng,
    out: &mut impl Write,
) -> Result<(u64, u64)> {
    let mut reader = parse_fastx_file(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("cannot open {}: {e}", input.display())))?;

    let n_usize = n as usize;

    // Phase 1: fill reservoir with first n records.
    let mut reservoir: Vec<(usize, RawRecord)> = Vec::with_capacity(n_usize);
    let mut total = 0usize;

    loop {
        match reader.next() {
            None => break,
            Some(rec) => {
                let rec = rec.map_err(|e| {
                    RsomicsError::InvalidInput(format!("malformed FASTQ record: {e}"))
                })?;
                if total < n_usize {
                    reservoir.push((total, RawRecord::from_needletail(&rec)?));
                } else {
                    // Vitter's Algorithm R: replace a random slot with probability n/i.
                    let j = rng.random_range(0..=total);
                    if j < n_usize {
                        reservoir[j] = (total, RawRecord::from_needletail(&rec)?);
                    }
                }
                total += 1;
            }
        }
    }

    // Sort by original position for stable output order.
    reservoir.sort_unstable_by_key(|(pos, _)| *pos);

    let kept = reservoir.len() as u64;
    for (_, raw) in reservoir {
        out.write_all(&raw.0).map_err(RsomicsError::Io)?;
    }
    Ok((total as u64, kept))
}

/// Exact-count reservoir sampling for paired-end reads.
///
/// Collects indices into a reservoir, then writes R1/R2 pairs for those indices.
/// Requires two passes over the input (one for index selection, one for output).
fn sample_exact_paired(
    r1: &Path,
    r2: &Path,
    n: u64,
    rng: &mut SmallRng,
    out1: &mut impl Write,
    out2: &mut impl Write,
) -> Result<(u64, u64)> {
    // Count total records (first pass on R1 only — R2 should match).
    let mut reader1_count = parse_fastx_file(r1)
        .map_err(|e| RsomicsError::InvalidInput(format!("cannot open {}: {e}", r1.display())))?;
    let mut total = 0u64;
    while reader1_count.next().is_some() {
        total += 1;
    }

    // Select the indices to keep using reservoir_sample from rand.
    let n_keep = n.min(total) as usize;
    let mut selected: std::collections::HashSet<usize> =
        reservoir_sample(rng, total as usize, n_keep)
            .into_iter()
            .collect();

    // Second pass: write selected pairs.
    let mut reader1 = parse_fastx_file(r1)
        .map_err(|e| RsomicsError::InvalidInput(format!("cannot open {}: {e}", r1.display())))?;
    let mut reader2 = parse_fastx_file(r2)
        .map_err(|e| RsomicsError::InvalidInput(format!("cannot open {}: {e}", r2.display())))?;

    let mut idx = 0usize;
    let mut kept = 0u64;
    loop {
        let rec1 = reader1.next();
        let rec2 = reader2.next();
        match (rec1, rec2) {
            (None, None) => break,
            (Some(r1), Some(r2)) => {
                let r1 = r1.map_err(|e| {
                    RsomicsError::InvalidInput(format!("malformed R1 FASTQ record: {e}"))
                })?;
                let r2 = r2.map_err(|e| {
                    RsomicsError::InvalidInput(format!("malformed R2 FASTQ record: {e}"))
                })?;
                if selected.remove(&idx) {
                    r1.write(&mut *out1, None)
                        .map_err(|e| RsomicsError::InvalidInput(e.to_string()))?;
                    r2.write(&mut *out2, None)
                        .map_err(|e| RsomicsError::InvalidInput(e.to_string()))?;
                    kept += 1;
                }
                idx += 1;
            }
            _ => {
                return Err(RsomicsError::InvalidInput(
                    "R1 and R2 have different record counts".into(),
                ));
            }
        }
    }
    Ok((total, kept))
}

/// Result of a subsampling operation.
pub struct SampleResult {
    pub total: u64,
    pub kept: u64,
}

/// Single-end subsampling.
pub fn run_se(input: &Path, output: &Path, mode: SampleMode, seed: u64) -> Result<SampleResult> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut out = open_writer(output)?;
    let (total, kept) = match mode {
        SampleMode::Fraction(p) => sample_fraction_single(input, p, &mut rng, &mut out)?,
        SampleMode::Exact(n) => sample_exact_single(input, n, &mut rng, &mut out)?,
    };
    out.flush().map_err(RsomicsError::Io)?;
    Ok(SampleResult { total, kept })
}

/// Paired-end subsampling.
pub fn run_pe(
    r1: &Path,
    r2: &Path,
    out1: &Path,
    out2: &Path,
    mode: SampleMode,
    seed: u64,
) -> Result<SampleResult> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut writer1 = open_writer(out1)?;
    let mut writer2 = open_writer(out2)?;
    let (total, kept) = match mode {
        SampleMode::Fraction(p) => {
            sample_fraction_paired(r1, r2, p, &mut rng, &mut writer1, &mut writer2)?
        }
        SampleMode::Exact(n) => {
            sample_exact_paired(r1, r2, n, &mut rng, &mut writer1, &mut writer2)?
        }
    };
    writer1.flush().map_err(RsomicsError::Io)?;
    writer2.flush().map_err(RsomicsError::Io)?;
    Ok(SampleResult { total, kept })
}
