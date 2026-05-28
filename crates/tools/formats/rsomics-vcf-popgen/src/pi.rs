//! Nucleotide diversity (π) in sliding windows — equivalent to `vcftools --window-pi`.
//!
//! π per window = (sum of pairwise differences) / (n_pairs * window_size)
//! where pairwise difference at a site = 2 * p * (1-p) for allele freq p.
//!
//! Output: CHROM, BIN_START, BIN_END, N_VARIANTS, PI

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use crate::vcf::VcfReader;

pub struct PiWindow {
    pub chrom: String,
    pub bin_start: u64, // 1-based, inclusive
    pub bin_end: u64,   // 1-based, inclusive
    pub n_variants: u64,
    pub pi: f64,
}

pub fn pi_windows(path: &Path, window_size: u64, step_size: u64) -> Result<Vec<PiWindow>> {
    // Collect per-site (chrom, pos) → pi_site.
    // pi_site = 2*p*(1-p) where p = ref freq among called alleles.
    let reader = VcfReader::open(path)?;
    let mut sites: HashMap<String, Vec<(u64, f64)>> = HashMap::new();

    for rec in reader {
        let rec = rec?;
        if rec.alt_alleles.is_empty() {
            continue;
        }
        let (n_chr, n_alt) = rec.allele_count();
        if n_chr < 2 {
            continue;
        }
        let p = (n_chr - n_alt) as f64 / n_chr as f64;
        let q = n_alt as f64 / n_chr as f64;
        // Unbiased estimator: multiply by n/(n-1) where n = n_chr/2 = n_individuals.
        let n_ind = n_chr / 2;
        let scale = if n_ind > 1 {
            n_ind as f64 / (n_ind - 1) as f64
        } else {
            1.0
        };
        let pi_site = 2.0 * p * q * scale;
        sites.entry(rec.chrom).or_default().push((rec.pos, pi_site));
    }

    // For each chromosome, assign sites to windows.
    let mut out = Vec::new();

    let mut chroms: Vec<String> = sites.keys().cloned().collect();
    chroms.sort();

    for chrom in chroms {
        let mut site_vec = sites.remove(&chrom).unwrap();
        site_vec.sort_by_key(|(p, _)| *p);

        if site_vec.is_empty() {
            continue;
        }
        let max_pos = site_vec.last().unwrap().0;

        let mut win_start = 1u64;
        while win_start <= max_pos {
            let win_end = win_start + window_size - 1;
            let mut pi_sum = 0.0f64;
            let mut n_variants = 0u64;
            for &(pos, pi_site) in &site_vec {
                if pos >= win_start && pos <= win_end {
                    pi_sum += pi_site;
                    n_variants += 1;
                }
                if pos > win_end {
                    break;
                }
            }
            if n_variants > 0 {
                out.push(PiWindow {
                    chrom: chrom.clone(),
                    bin_start: win_start,
                    bin_end: win_end,
                    n_variants,
                    pi: pi_sum / window_size as f64,
                });
            }
            win_start += step_size;
        }
    }

    out.sort_by(|a, b| a.chrom.cmp(&b.chrom).then(a.bin_start.cmp(&b.bin_start)));
    Ok(out)
}

pub fn print_pi(records: &[PiWindow]) {
    println!("CHROM\tBIN_START\tBIN_END\tN_VARIANTS\tPI");
    for r in records {
        println!(
            "{}\t{}\t{}\t{}\t{:.6}",
            r.chrom, r.bin_start, r.bin_end, r.n_variants, r.pi
        );
    }
}
