use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn view_vcf(
    input: &Path,
    output: &mut dyn Write,
    header_only: bool,
    no_header: bool,
    count_only: bool,
    region: Option<&str>,
) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut count: u64 = 0;

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;

        if line.starts_with('#') {
            if !no_header && !count_only {
                writeln!(out, "{line}").map_err(RsomicsError::Io)?;
            }
            continue;
        }

        if header_only {
            break;
        }

        if let Some(region) = region {
            let chrom = line.split('\t').next().unwrap_or("");
            if chrom != region {
                continue;
            }
        }

        count += 1;
        if !count_only {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
        }
    }

    if count_only {
        writeln!(out, "{count}").map_err(RsomicsError::Io)?;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
