use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn vcf_singletons(input: &Path, output: &mut dyn Write) -> Result<(u64, u64)> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let (mut total, mut singletons) = (0u64, 0u64);
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
            continue;
        }
        total += 1;
        let is_singleton = line
            .split('\t')
            .nth(7)
            .is_some_and(|info| info.split(';').any(|kv| kv == "AC=1"));
        if is_singleton {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
            singletons += 1;
        }
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok((total, singletons))
}
