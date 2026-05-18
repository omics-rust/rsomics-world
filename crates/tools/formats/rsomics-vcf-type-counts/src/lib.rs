use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn vcf_type_counts(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let (mut snps, mut indels, mut mnps, mut other) = (0u64, 0, 0, 0);
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 5 {
            continue;
        }
        let rlen = f[3].len();
        for alt in f[4].split(',') {
            if alt == "*" || alt == "." {
                continue;
            }
            if rlen == 1 && alt.len() == 1 {
                snps += 1;
            } else if rlen == alt.len() && rlen > 1 {
                mnps += 1;
            } else if rlen != alt.len() {
                indels += 1;
            } else {
                other += 1;
            }
        }
    }
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    writeln!(out, "SNPs\t{snps}").map_err(RsomicsError::Io)?;
    writeln!(out, "indels\t{indels}").map_err(RsomicsError::Io)?;
    writeln!(out, "MNPs\t{mnps}").map_err(RsomicsError::Io)?;
    writeln!(out, "other\t{other}").map_err(RsomicsError::Io)?;
    out.flush().map_err(RsomicsError::Io)?;
    Ok(snps + indels + mnps + other)
}
