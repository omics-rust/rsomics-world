//! Singleton sites — equivalent to `vcftools --singletons`.
//!
//! A singleton is a site where exactly one allele copy is the minor allele.
//! A private doubleton is a site where a single individual is homozygous for
//! the minor allele (two copies from the same individual).
//!
//! Output: CHROM, POS, SINGLETON/DOUBLETON, ALLELE, INDV

use std::path::Path;

use anyhow::Result;

use crate::vcf::{Allele, VcfReader};

pub struct SingletonRecord {
    pub chrom: String,
    pub pos: u64,
    pub singleton_type: String, // "S" or "D"
    pub allele: String,
    pub indv: String,
}

pub fn singletons(path: &Path) -> Result<Vec<SingletonRecord>> {
    let reader = VcfReader::open(path)?;
    let samples = reader.samples.clone();
    let mut out = Vec::new();

    for rec in reader {
        let rec = rec?;
        if rec.alt_alleles.len() != 1 {
            continue; // biallelic only (vcftools singletons is biallelic)
        }

        // Count alt allele copies per individual.
        let mut alt_count_per_indv = vec![0u8; samples.len()];
        let mut total_alt = 0u64;
        let mut n_called = 0u64;

        for (i, gt) in rec.gts.iter().enumerate() {
            if gt.is_missing() {
                continue;
            }
            n_called += 1;
            for a in [gt.a1, gt.a2] {
                if matches!(a, Allele::Alt(_)) {
                    alt_count_per_indv[i] += 1;
                    total_alt += 1;
                }
            }
        }

        if n_called == 0 {
            continue;
        }

        // Use the less-frequent allele as the "singleton" candidate.
        let n_chr = n_called * 2;
        let n_ref = n_chr - total_alt;
        let (minor_count, is_alt_minor) = if total_alt <= n_ref {
            (total_alt, true)
        } else {
            (n_ref, false)
        };

        if minor_count == 0 {
            continue;
        }

        if minor_count == 1 {
            // Singleton: exactly one copy of the minor allele.
            let carrier = samples.iter().enumerate().find(|(i, _)| {
                let gt = &rec.gts[*i];
                if is_alt_minor {
                    alt_count_per_indv[*i] == 1
                } else {
                    // ref singleton: het carrier
                    !gt.is_missing() && alt_count_per_indv[*i] == 1
                }
            });
            if let Some((_, name)) = carrier {
                let allele = if is_alt_minor {
                    rec.alt_alleles[0].clone()
                } else {
                    rec.ref_allele.clone()
                };
                out.push(SingletonRecord {
                    chrom: rec.chrom,
                    pos: rec.pos,
                    singleton_type: "S".to_string(),
                    allele,
                    indv: name.clone(),
                });
            }
        } else if minor_count == 2 {
            // Private doubleton: one individual is hom for the minor allele.
            let carrier = samples.iter().enumerate().find(|(i, _)| {
                let gt = &rec.gts[*i];
                if gt.is_missing() {
                    return false;
                }
                if is_alt_minor {
                    alt_count_per_indv[*i] == 2
                } else {
                    // ref doubleton: hom ref in a sea of alt
                    alt_count_per_indv[*i] == 0
                }
            });
            if let Some((_, name)) = carrier {
                let allele = if is_alt_minor {
                    rec.alt_alleles[0].clone()
                } else {
                    rec.ref_allele.clone()
                };
                out.push(SingletonRecord {
                    chrom: rec.chrom,
                    pos: rec.pos,
                    singleton_type: "D".to_string(),
                    allele,
                    indv: name.clone(),
                });
            }
        }
    }

    Ok(out)
}

pub fn print_singletons(records: &[SingletonRecord]) {
    println!("CHROM\tPOS\tSINGLETON/DOUBLETON\tALLELE\tINDV");
    for r in records {
        println!(
            "{}\t{}\t{}\t{}\t{}",
            r.chrom, r.pos, r.singleton_type, r.allele, r.indv
        );
    }
}
