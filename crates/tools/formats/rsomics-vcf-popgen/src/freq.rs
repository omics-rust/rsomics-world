//! Allele frequency per site — equivalent to `vcftools --freq`.
//!
//! Output: CHROM, POS, N_ALLELES, N_CHR, followed by `allele:freq` pairs.
//! Monomorphic sites and sites with no non-missing genotypes are skipped.

use std::path::Path;

use anyhow::Result;

use crate::vcf::VcfReader;

pub struct FreqRecord {
    pub chrom: String,
    pub pos: u64,
    pub n_alleles: usize,
    pub n_chr: u64,                       // called (non-missing) allele copies
    pub allele_freqs: Vec<(String, f64)>, // (allele, frequency)
}

pub fn freq(path: &Path) -> Result<Vec<FreqRecord>> {
    let reader = VcfReader::open(path)?;
    let mut out = Vec::new();

    for rec in reader {
        let rec = rec?;
        if rec.alt_alleles.is_empty() {
            continue;
        }

        let n_alleles = rec.alt_alleles.len() + 1; // ref + alts
        // Count allele observations.
        let mut counts = vec![0u64; n_alleles];
        let mut n_chr = 0u64;

        for gt in &rec.gts {
            for a in [gt.a1, gt.a2] {
                use crate::vcf::Allele;
                match a {
                    Allele::Ref => {
                        counts[0] += 1;
                        n_chr += 1;
                    }
                    Allele::Alt(idx) => {
                        let i = idx as usize;
                        if i < counts.len() {
                            counts[i] += 1;
                        }
                        n_chr += 1;
                    }
                    Allele::Missing => {}
                }
            }
        }

        if n_chr == 0 {
            continue;
        }

        let mut allele_freqs = Vec::with_capacity(n_alleles);
        allele_freqs.push((rec.ref_allele.clone(), counts[0] as f64 / n_chr as f64));
        for (i, alt) in rec.alt_alleles.iter().enumerate() {
            allele_freqs.push((alt.clone(), counts[i + 1] as f64 / n_chr as f64));
        }

        out.push(FreqRecord {
            chrom: rec.chrom,
            pos: rec.pos,
            n_alleles,
            n_chr,
            allele_freqs,
        });
    }

    Ok(out)
}

pub fn print_freq(records: &[FreqRecord]) {
    println!("CHROM\tPOS\tN_ALLELES\tN_CHR\t{{ALLELE:FREQ}}");
    for r in records {
        let pairs: Vec<String> = r
            .allele_freqs
            .iter()
            .map(|(a, f)| format!("{a}:{f:.4}"))
            .collect();
        println!(
            "{}\t{}\t{}\t{}\t{}",
            r.chrom,
            r.pos,
            r.n_alleles,
            r.n_chr,
            pairs.join("\t")
        );
    }
}
