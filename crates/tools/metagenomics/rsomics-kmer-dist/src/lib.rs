use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub enum DistMetric {
    Jaccard,
    BrayCurtis,
    Cosine,
}

pub fn kmer_profile(path: &Path, k: usize) -> Result<HashMap<u64, u64>> {
    let mut counts: HashMap<u64, u64> = HashMap::new();
    let mut reader = needletail::parse_fastx_file(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;

    while let Some(result) = reader.next() {
        let record =
            result.map_err(|e| RsomicsError::InvalidInput(format!("reading record: {e}")))?;
        let seq = record.seq();
        let iter = rsomics_kmer::KmerIter::new(&seq, k, true)
            .map_err(|e| RsomicsError::InvalidInput(format!("kmer iter: {e}")))?;
        for kmer in iter.flatten() {
            *counts.entry(kmer).or_insert(0) += 1;
        }
    }
    Ok(counts)
}

#[allow(clippy::cast_precision_loss, clippy::implicit_hasher)]
#[must_use]
pub fn compute_distance(a: &HashMap<u64, u64>, b: &HashMap<u64, u64>, metric: &DistMetric) -> f64 {
    match metric {
        DistMetric::Jaccard => {
            let keys_a: std::collections::HashSet<&u64> = a.keys().collect();
            let keys_b: std::collections::HashSet<&u64> = b.keys().collect();
            let intersection = keys_a.intersection(&keys_b).count();
            let union = keys_a.union(&keys_b).count();
            if union == 0 {
                0.0
            } else {
                1.0 - (intersection as f64 / union as f64)
            }
        }
        DistMetric::BrayCurtis => {
            let mut sum_min = 0u64;
            let mut sum_a = 0u64;
            let mut sum_b = 0u64;
            for (&k, &va) in a {
                let vb = b.get(&k).copied().unwrap_or(0);
                sum_min += va.min(vb);
                sum_a += va;
            }
            for &vb in b.values() {
                sum_b += vb;
            }
            let denom = sum_a + sum_b;
            if denom == 0 {
                0.0
            } else {
                1.0 - (2.0 * sum_min as f64 / denom as f64)
            }
        }
        DistMetric::Cosine => {
            let mut dot = 0.0f64;
            let mut norm_a = 0.0f64;
            let mut norm_b = 0.0f64;
            for (&k, &va) in a {
                let vb = b.get(&k).copied().unwrap_or(0) as f64;
                let va = va as f64;
                dot += va * vb;
                norm_a += va * va;
            }
            for &vb in b.values() {
                norm_b += (vb as f64) * (vb as f64);
            }
            let denom = norm_a.sqrt() * norm_b.sqrt();
            if denom == 0.0 {
                1.0
            } else {
                1.0 - (dot / denom)
            }
        }
    }
}

pub fn pairwise_distances(
    files: &[&Path],
    k: usize,
    metric: &DistMetric,
    output: &mut dyn Write,
) -> Result<()> {
    let profiles: Vec<HashMap<u64, u64>> = files
        .iter()
        .map(|f| kmer_profile(f, k))
        .collect::<Result<Vec<_>>>()?;

    for (i, fi) in files.iter().enumerate() {
        for (j, fj) in files.iter().enumerate() {
            if j <= i {
                continue;
            }
            let d = compute_distance(&profiles[i], &profiles[j], metric);
            let ni = fi.file_name().unwrap_or_default().to_string_lossy();
            let nj = fj.file_name().unwrap_or_default().to_string_lossy();
            writeln!(output, "{ni}\t{nj}\t{d:.6}").map_err(RsomicsError::Io)?;
        }
    }
    Ok(())
}
