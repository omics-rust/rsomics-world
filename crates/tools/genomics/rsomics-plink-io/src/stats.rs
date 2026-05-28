//! Site-level statistics from a loaded PLINK fileset.
//!
//! These stats match plink --freq / --missing / --hardy output format.

use rsomics_pgen::{Genotype, Pgen};

pub struct FreqRecord {
    pub chrom: String,
    pub snp: String,
    pub a1: String,   // minor allele (a1 in PLINK convention)
    pub a2: String,   // major allele
    pub maf: f64,     // minor allele frequency
    pub nchrobs: u64, // non-missing allele copies
}

pub struct MissingRecord {
    pub chrom: String,
    pub snp: String,
    pub n_miss: u64,
    pub n_geno: u64, // total called genotypes
    pub f_miss: f64,
}

pub struct HweRecord {
    pub chrom: String,
    pub snp: String,
    pub test: String, // "ALL", "AFF", "UNAFF"
    pub geno: String, // OBS(HOM1/HET/HOM2)
    pub o_het: u64,
    pub e_het: f64,
    pub p: f64,
}

/// Allele frequency per variant, matching plink --freq output.
pub fn allele_freq(pgen: &Pgen) -> Vec<FreqRecord> {
    let n_samples = pgen.n_samples();
    pgen.variants
        .iter()
        .enumerate()
        .map(|(v, var)| {
            let mut n_a1 = 0u64; // A1 allele count (PLINK A1 = minor/coded)
            let mut n_obs = 0u64;
            for s in 0..n_samples {
                match pgen.get(v, s) {
                    Genotype::HomA1 => {
                        n_a1 += 2;
                        n_obs += 2;
                    }
                    Genotype::Het => {
                        n_a1 += 1;
                        n_obs += 2;
                    }
                    Genotype::HomA2 => {
                        n_obs += 2;
                    }
                    Genotype::Missing => {}
                    _ => {}
                }
            }
            let maf = if n_obs > 0 {
                n_a1 as f64 / n_obs as f64
            } else {
                0.0
            };
            FreqRecord {
                chrom: var.chrom.clone(),
                snp: var.id.clone(),
                a1: var.a1.clone(),
                a2: var.a2.clone(),
                maf,
                nchrobs: n_obs,
            }
        })
        .collect()
}

/// Missingness per variant, matching plink --missing --variant-only output.
pub fn missingness(pgen: &Pgen) -> Vec<MissingRecord> {
    let n_samples = pgen.n_samples();
    pgen.variants
        .iter()
        .enumerate()
        .map(|(v, var)| {
            let n_miss = (0..n_samples)
                .filter(|&s| pgen.get(v, s) == Genotype::Missing)
                .count() as u64;
            let n_geno = n_samples as u64 - n_miss;
            let f_miss = if n_samples > 0 {
                n_miss as f64 / n_samples as f64
            } else {
                0.0
            };
            MissingRecord {
                chrom: var.chrom.clone(),
                snp: var.id.clone(),
                n_miss,
                n_geno,
                f_miss,
            }
        })
        .collect()
}

/// Per-variant HWE p-value, matching plink --hardy (exact test, Wigginton 2005).
pub fn hwe_stats(pgen: &Pgen) -> Vec<HweRecord> {
    let n_samples = pgen.n_samples();
    pgen.variants
        .iter()
        .enumerate()
        .map(|(v, var)| {
            let mut n_hom1 = 0u64;
            let mut n_het = 0u64;
            let mut n_hom2 = 0u64;
            for s in 0..n_samples {
                match pgen.get(v, s) {
                    Genotype::HomA1 => n_hom1 += 1,
                    Genotype::Het => n_het += 1,
                    Genotype::HomA2 => n_hom2 += 1,
                    Genotype::Missing => {}
                    _ => {}
                }
            }
            let n = n_hom1 + n_het + n_hom2;
            let p_val = exact_hwe_p(n_hom1, n_het, n_hom2);
            let geno = format!("{n_hom1}/{n_het}/{n_hom2}");
            let e_het = if n > 0 {
                let p = (2 * n_hom1 + n_het) as f64 / (2 * n) as f64;
                2.0 * p * (1.0 - p) * n as f64
            } else {
                0.0
            };
            HweRecord {
                chrom: var.chrom.clone(),
                snp: var.id.clone(),
                test: "ALL".to_string(),
                geno,
                o_het: n_het,
                e_het,
                p: p_val,
            }
        })
        .collect()
}

/// Wigginton et al. 2005 exact HWE p-value.
fn exact_hwe_p(obs_hom1: u64, obs_het: u64, obs_hom2: u64) -> f64 {
    let n = obs_hom1 + obs_het + obs_hom2;
    if n == 0 {
        return f64::NAN;
    }
    let n_rare_a = (2 * obs_hom1 + obs_het).min(2 * obs_hom2 + obs_het);
    let n_chr = 2 * n;
    let max_het = n_rare_a.min(n_chr - n_rare_a);
    let table_len = (max_het / 2 + 1) as usize;
    let mut probs = vec![0.0f64; table_len];
    let max_idx = table_len - 1;
    probs[max_idx] = 1.0;
    for t in (0..max_idx).rev() {
        let k = (max_het - 2 * (max_idx - t - 1) as u64) as f64;
        let denom = (n_rare_a as f64 - k + 2.0) * (n_chr as f64 - n_rare_a as f64 - k + 2.0);
        if denom == 0.0 {
            break;
        }
        probs[t] = probs[t + 1] * k * (k - 1.0) / denom;
    }
    let sum: f64 = probs.iter().sum();
    if sum == 0.0 {
        return f64::NAN;
    }
    let obs_t = if obs_het <= max_het && obs_het % 2 == max_het % 2 {
        Some(max_idx - ((max_het - obs_het) / 2) as usize)
    } else {
        None
    };
    let obs_p = obs_t.map_or(0.0, |t| probs[t] / sum);
    let p_hwe: f64 = probs.iter().filter(|&&p| p <= obs_p + 1e-15).sum::<f64>() / sum;
    p_hwe.clamp(0.0, 1.0)
}

pub fn print_freq(records: &[FreqRecord]) {
    println!("CHR\tSNP\tA1\tA2\tMAF\tNCHROBS");
    for r in records {
        println!(
            "{}\t{}\t{}\t{}\t{:.6}\t{}",
            r.chrom, r.snp, r.a1, r.a2, r.maf, r.nchrobs
        );
    }
}

pub fn print_missing(records: &[MissingRecord]) {
    println!("CHR\tSNP\tN_MISS\tN_GENO\tF_MISS");
    for r in records {
        println!(
            "{}\t{}\t{}\t{}\t{:.4}",
            r.chrom, r.snp, r.n_miss, r.n_geno, r.f_miss
        );
    }
}

pub fn print_hwe(records: &[HweRecord]) {
    println!("CHR\tSNP\tTEST\tGENO\tO(HET)\tE(HET)\tP");
    for r in records {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{:.3}\t{:.6}",
            r.chrom, r.snp, r.test, r.geno, r.o_het, r.e_het, r.p
        );
    }
}
