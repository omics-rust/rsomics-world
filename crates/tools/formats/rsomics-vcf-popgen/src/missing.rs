//! Missingness statistics — equivalent to `vcftools --missing-site` and `--missing-indv`.
//!
//! Per-site output: CHROM, POS, N_DATA, N_GENOTYPE_FILTERED, N_MISS, F_MISS
//! Per-indv output: INDV, N_DATA, N_GENOTYPE_FILTERED, N_MISS, F_MISS

use std::path::Path;

use anyhow::Result;

use crate::vcf::VcfReader;

pub struct MissingSite {
    pub chrom: String,
    pub pos: u64,
    pub n_data: u64, // total genotype calls expected (= n_samples)
    pub n_miss: u64,
    pub f_miss: f64,
}

pub struct MissingIndv {
    pub indv: String,
    pub n_data: u64, // total sites seen
    pub n_miss: u64,
    pub f_miss: f64,
}

pub fn missing_site(path: &Path) -> Result<Vec<MissingSite>> {
    let reader = VcfReader::open(path)?;
    let n_samples = reader.n_samples() as u64;
    let mut out = Vec::new();

    for rec in reader {
        let rec = rec?;
        let n_miss = rec.gts.iter().filter(|g| g.is_missing()).count() as u64;
        let f_miss = if n_samples > 0 {
            n_miss as f64 / n_samples as f64
        } else {
            0.0
        };
        out.push(MissingSite {
            chrom: rec.chrom,
            pos: rec.pos,
            n_data: n_samples,
            n_miss,
            f_miss,
        });
    }

    Ok(out)
}

pub fn missing_indv(path: &Path) -> Result<Vec<MissingIndv>> {
    let reader = VcfReader::open(path)?;
    let samples = reader.samples.clone();
    let n = samples.len();
    let mut n_data = vec![0u64; n];
    let mut n_miss = vec![0u64; n];

    for rec in reader {
        let rec = rec?;
        for (i, gt) in rec.gts.iter().enumerate() {
            n_data[i] += 1;
            if gt.is_missing() {
                n_miss[i] += 1;
            }
        }
    }

    let records = samples
        .into_iter()
        .enumerate()
        .map(|(i, name)| {
            let nd = n_data[i];
            let nm = n_miss[i];
            let f = if nd > 0 { nm as f64 / nd as f64 } else { 0.0 };
            MissingIndv {
                indv: name,
                n_data: nd,
                n_miss: nm,
                f_miss: f,
            }
        })
        .collect();

    Ok(records)
}

pub fn print_missing_site(records: &[MissingSite]) {
    println!("CHROM\tPOS\tN_DATA\tN_GENOTYPE_FILTERED\tN_MISS\tF_MISS");
    for r in records {
        println!(
            "{}\t{}\t{}\t0\t{}\t{:.4}",
            r.chrom, r.pos, r.n_data, r.n_miss, r.f_miss
        );
    }
}

pub fn print_missing_indv(records: &[MissingIndv]) {
    println!("INDV\tN_DATA\tN_GENOTYPE_FILTERED\tN_MISS\tF_MISS");
    for r in records {
        println!("{}\t{}\t0\t{}\t{:.4}", r.indv, r.n_data, r.n_miss, r.f_miss);
    }
}
