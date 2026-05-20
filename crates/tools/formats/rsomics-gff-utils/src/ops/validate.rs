use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub struct GffValidation {
    pub records: u64,
    pub errors: Vec<String>,
    pub is_valid: bool,
}

pub fn validate_gff(input: &Path) -> Result<GffValidation> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut records: u64 = 0;
    let mut errors: Vec<String> = Vec::new();
    let mut line_num: u64 = 0;

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        line_num += 1;

        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 9 {
            errors.push(format!(
                "line {line_num}: expected 9 tab-separated fields, got {}",
                fields.len()
            ));
            continue;
        }

        if fields[3].parse::<u64>().is_err() {
            errors.push(format!("line {line_num}: start is not a valid integer"));
        }
        if fields[4].parse::<u64>().is_err() {
            errors.push(format!("line {line_num}: end is not a valid integer"));
        }

        let strand = fields[6];
        if strand != "+" && strand != "-" && strand != "." {
            errors.push(format!("line {line_num}: invalid strand '{strand}'"));
        }

        records += 1;
    }

    let is_valid = errors.is_empty();
    Ok(GffValidation {
        records,
        errors,
        is_valid,
    })
}
