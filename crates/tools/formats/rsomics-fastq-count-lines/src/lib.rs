use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn count_fastq(input: &Path) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let lines = reader.lines().count() as u64;
    Ok(lines / 4)
}
