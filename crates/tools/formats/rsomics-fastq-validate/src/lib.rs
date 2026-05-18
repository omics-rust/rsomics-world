use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub struct ValidationResult {
    pub records: u64,
    pub errors: Vec<String>,
    pub is_valid: bool,
}

pub fn validate_fastq(input: &Path) -> Result<ValidationResult> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut records: u64 = 0;
    let mut errors: Vec<String> = Vec::new();
    let mut line_num: u64 = 0;

    loop {
        let header = match lines.next() {
            Some(Ok(l)) => {
                line_num += 1;
                l
            }
            Some(Err(e)) => {
                errors.push(format!("line {line_num}: IO error: {e}"));
                break;
            }
            None => break,
        };

        if header.is_empty() {
            continue;
        }

        if !header.starts_with('@') {
            errors.push(format!("line {line_num}: header doesn't start with @"));
        }

        let seq = match lines.next() {
            Some(Ok(l)) => {
                line_num += 1;
                l
            }
            _ => {
                errors.push(format!("line {line_num}: truncated (missing sequence)"));
                break;
            }
        };

        let plus = match lines.next() {
            Some(Ok(l)) => {
                line_num += 1;
                l
            }
            _ => {
                errors.push(format!("line {line_num}: truncated (missing + line)"));
                break;
            }
        };

        if !plus.starts_with('+') {
            errors.push(format!("line {line_num}: separator doesn't start with +"));
        }

        let qual = match lines.next() {
            Some(Ok(l)) => {
                line_num += 1;
                l
            }
            _ => {
                errors.push(format!("line {line_num}: truncated (missing quality)"));
                break;
            }
        };

        if seq.len() != qual.len() {
            errors.push(format!(
                "record {}: seq len {} != qual len {}",
                records + 1,
                seq.len(),
                qual.len()
            ));
        }

        records += 1;
    }

    let is_valid = errors.is_empty();
    Ok(ValidationResult {
        records,
        errors,
        is_valid,
    })
}
