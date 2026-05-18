use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn vcf_het_hom(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut samples: Vec<String> = Vec::new();
    let mut het: Vec<u64> = Vec::new();
    let mut hom_alt: Vec<u64> = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with("#CHROM") {
            let fields: Vec<&str> = line.split('\t').collect();
            for s in fields.iter().skip(9) {
                samples.push(s.to_string());
            }
            het.resize(samples.len(), 0);
            hom_alt.resize(samples.len(), 0);
            continue;
        }
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        for (i, gt_field) in fields.iter().skip(9).enumerate() {
            let gt = gt_field.split(':').next().unwrap_or(".");
            if gt.contains('/') || gt.contains('|') {
                let alleles: Vec<&str> = gt.split(|c| c == '/' || c == '|').collect();
                if alleles.len() == 2 && alleles[0] != "." && alleles[1] != "." {
                    if alleles[0] != alleles[1] {
                        if i < het.len() {
                            het[i] += 1;
                        }
                    } else if alleles[0] != "0" {
                        if i < hom_alt.len() {
                            hom_alt[i] += 1;
                        }
                    }
                }
            }
        }
    }
    writeln!(out, "sample\thet\thom_alt").map_err(RsomicsError::Io)?;
    for (i, s) in samples.iter().enumerate() {
        writeln!(
            out,
            "{s}\t{}\t{}",
            het.get(i).unwrap_or(&0),
            hom_alt.get(i).unwrap_or(&0)
        )
        .map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(samples.len() as u64)
}
