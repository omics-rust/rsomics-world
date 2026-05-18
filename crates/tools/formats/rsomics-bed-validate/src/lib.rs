use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub struct BedValidation {
    pub records: u64,
    pub errors: Vec<String>,
    pub is_valid: bool,
}

pub fn validate_bed(input: &Path) -> Result<BedValidation> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut records: u64 = 0;
    let mut errors: Vec<String> = Vec::new();
    let mut line_num: u64 = 0;

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        line_num += 1;

        if line.starts_with('#') || line.starts_with("track") || line.starts_with("browser") {
            continue;
        }
        if line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 3 {
            errors.push(format!(
                "line {line_num}: need >= 3 fields, got {}",
                fields.len()
            ));
            continue;
        }

        let start = fields[1].parse::<u64>();
        let end = fields[2].parse::<u64>();

        if start.is_err() {
            errors.push(format!("line {line_num}: start is not a valid integer"));
        }
        if end.is_err() {
            errors.push(format!("line {line_num}: end is not a valid integer"));
        }
        if let (Ok(s), Ok(e)) = (start, end)
            && s > e
        {
            errors.push(format!("line {line_num}: start ({s}) > end ({e})"));
        }

        records += 1;
    }

    let is_valid = errors.is_empty();
    Ok(BedValidation {
        records,
        errors,
        is_valid,
    })
}
