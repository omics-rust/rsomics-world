use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub struct FastaValidation {
    pub sequences: u64,
    pub errors: Vec<String>,
    pub is_valid: bool,
}

pub fn validate_fasta(input: &Path) -> Result<FastaValidation> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut sequences: u64 = 0;
    let mut errors: Vec<String> = Vec::new();
    let mut line_num: u64 = 0;
    let mut in_sequence = false;
    let mut has_bases = false;

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        line_num += 1;

        if line.is_empty() {
            continue;
        }

        if line.starts_with('>') {
            if in_sequence && !has_bases {
                errors.push(format!("line {line_num}: previous sequence has no bases"));
            }
            sequences += 1;
            in_sequence = true;
            has_bases = false;

            if line.len() == 1 {
                errors.push(format!("line {line_num}: empty sequence name"));
            }
        } else if in_sequence {
            has_bases = true;
            for (i, ch) in line.bytes().enumerate() {
                if !ch.is_ascii_alphabetic() && ch != b'*' && ch != b'-' {
                    errors.push(format!(
                        "line {line_num} col {}: invalid character '{}'",
                        i + 1,
                        ch as char
                    ));
                    break;
                }
            }
        } else {
            errors.push(format!(
                "line {line_num}: sequence data before first header"
            ));
        }
    }

    if sequences == 0 {
        errors.push("no sequences found".to_string());
    }

    let is_valid = errors.is_empty();
    Ok(FastaValidation {
        sequences,
        errors,
        is_valid,
    })
}
