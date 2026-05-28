//! Export PLINK binary data to VCF and 012-matrix formats.

use std::io::{self, Write};

use rsomics_pgen::{Genotype, Pgen};

/// Write a minimal VCF to `out` from a loaded PLINK fileset.
///
/// No per-sample FORMAT fields beyond GT are emitted. Genotype encoding:
/// - HomA1 → 0/0 (A1 is the reference/coded allele in PLINK)
/// - Het    → 0/1
/// - HomA2  → 1/1
/// - Missing → ./.
pub fn to_vcf(pgen: &Pgen, out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "##fileformat=VCFv4.2")?;
    writeln!(
        out,
        "##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">"
    )?;
    let sample_names: Vec<String> = pgen
        .samples
        .iter()
        .map(|s| format!("{}_{}", s.fid, s.iid))
        .collect();
    write!(out, "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT")?;
    for name in &sample_names {
        write!(out, "\t{name}")?;
    }
    writeln!(out)?;

    for (v, var) in pgen.variants.iter().enumerate() {
        // PLINK A1 = coded/minor allele → VCF ALT; A2 = major → VCF REF.
        write!(
            out,
            "{}\t{}\t{}\t{}\t{}\t.\tPASS\t.\tGT",
            var.chrom, var.pos, var.id, var.a2, var.a1
        )?;
        for s in 0..pgen.n_samples() {
            let gt = match pgen.get(v, s) {
                Genotype::HomA1 => "1/1",
                Genotype::Het => "0/1",
                Genotype::HomA2 => "0/0",
                Genotype::Missing => "./.",
                _ => "./.",
            };
            write!(out, "\t{gt}")?;
        }
        writeln!(out)?;
    }
    Ok(())
}

/// Write a 012-matrix (sample × variant) to `out`.
///
/// 0 = homozygous A2, 1 = heterozygous, 2 = homozygous A1, -1 = missing.
/// Row = one sample, column = one variant, space-separated.
/// Header line: variant IDs.
pub fn to_012(pgen: &Pgen, out: &mut impl Write) -> io::Result<()> {
    // Header: sample FID_IID as first column, then variant IDs.
    write!(out, "FID_IID")?;
    for var in &pgen.variants {
        write!(out, "\t{}", var.id)?;
    }
    writeln!(out)?;

    for s in 0..pgen.n_samples() {
        let sample = &pgen.samples[s];
        write!(out, "{}_{}", sample.fid, sample.iid)?;
        for v in 0..pgen.n_variants() {
            let val = match pgen.get(v, s) {
                Genotype::HomA2 => 0i8,
                Genotype::Het => 1,
                Genotype::HomA1 => 2,
                Genotype::Missing => -1,
                _ => -1,
            };
            write!(out, "\t{val}")?;
        }
        writeln!(out)?;
    }
    Ok(())
}
