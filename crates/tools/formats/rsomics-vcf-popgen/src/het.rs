//! Per-individual heterozygosity — equivalent to `vcftools --het`.
//!
//! Output: INDV, O(HOM), E(HOM), N_SITES, F
//! where F is Wright's inbreeding coefficient = (O_hom - E_hom) / (N_sites - E_hom).
//! Follows vcftools semantics: sites with any missing genotype in the individual are
//! excluded from that individual's count.

use std::path::Path;

use anyhow::Result;

use crate::vcf::VcfReader;

pub struct HetRecord {
    pub indv: String,
    pub o_hom: u64,   // observed homozygous genotypes
    pub e_hom: f64,   // expected homozygous genotypes under HWE
    pub n_sites: u64, // non-missing sites for this individual
    pub f: f64,       // inbreeding coefficient
}

pub fn het(path: &Path) -> Result<Vec<HetRecord>> {
    let reader = VcfReader::open(path)?;
    let samples = reader.samples.clone();
    let n = samples.len();

    // Per-individual accumulators.
    let mut o_hom = vec![0u64; n];
    let mut e_hom = vec![0.0f64; n];
    let mut n_sites = vec![0u64; n];

    for rec in reader {
        let rec = rec?;
        if rec.alt_alleles.is_empty() {
            continue;
        }

        // Site-level AF from all non-missing genotypes.
        let (n_chr, n_alt) = rec.allele_count();
        if n_chr == 0 {
            continue;
        }
        let p = (n_chr - n_alt) as f64 / n_chr as f64; // ref freq
        let q = n_alt as f64 / n_chr as f64; // alt freq (biallelic approx)
        let e_hom_site = p * p + q * q; // HWE expected hom freq

        for (i, gt) in rec.gts.iter().enumerate() {
            if gt.is_missing() {
                continue;
            }
            n_sites[i] += 1;
            if !gt.is_het() {
                o_hom[i] += 1;
            }
            e_hom[i] += e_hom_site;
        }
    }

    let records = samples
        .into_iter()
        .enumerate()
        .map(|(i, name)| {
            let ns = n_sites[i];
            let oh = o_hom[i];
            let eh = e_hom[i];
            let f = if ns as f64 - eh > 0.0 {
                (oh as f64 - eh) / (ns as f64 - eh)
            } else {
                f64::NAN
            };
            HetRecord {
                indv: name,
                o_hom: oh,
                e_hom: eh,
                n_sites: ns,
                f,
            }
        })
        .collect();

    Ok(records)
}

pub fn print_het(records: &[HetRecord]) {
    println!("INDV\tO(HOM)\tE(HOM)\tN_SITES\tF");
    for r in records {
        let f = if r.f.is_nan() {
            "nan".to_string()
        } else {
            format!("{:.6}", r.f)
        };
        println!(
            "{}\t{}\t{:.2}\t{}\t{}",
            r.indv, r.o_hom, r.e_hom, r.n_sites, f
        );
    }
}
