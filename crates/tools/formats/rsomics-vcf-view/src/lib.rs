use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub struct ViewOpts {
    pub header_only: bool,
    pub no_header: bool,
    pub count_only: bool,
    pub region: Option<String>,
}

pub fn view_vcf(input: &Path, output: &mut dyn Write, opts: &ViewOpts) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut count: u64 = 0;

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;

        if line.starts_with('#') {
            if !opts.no_header && !opts.count_only {
                writeln!(out, "{line}").map_err(RsomicsError::Io)?;
            }
            continue;
        }

        if opts.header_only {
            break;
        }

        if let Some(ref region) = opts.region {
            let chrom = line.split('\t').next().unwrap_or("");
            if chrom != region {
                continue;
            }
        }

        count += 1;
        if !opts.count_only {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
        }
    }

    if opts.count_only {
        writeln!(out, "{count}").map_err(RsomicsError::Io)?;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
