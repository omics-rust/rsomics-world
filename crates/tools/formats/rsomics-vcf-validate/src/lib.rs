use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub struct VcfValidation {
    pub variants: u64,
    pub errors: Vec<String>,
    pub is_valid: bool,
    pub has_header: bool,
}

pub fn validate_vcf(input: &Path) -> Result<VcfValidation> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut variants: u64 = 0;
    let mut errors: Vec<String> = Vec::new();
    let mut line_num: u64 = 0;
    let mut has_header = false;
    let mut has_column_line = false;

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        line_num += 1;

        if line.starts_with("##") {
            has_header = true;
            continue;
        }

        if line.starts_with("#CHROM") {
            has_column_line = true;
            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() < 8 {
                errors.push(format!(
                    "line {line_num}: column header has {} fields, need >=8",
                    cols.len()
                ));
            }
            continue;
        }

        if line.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 8 {
            errors.push(format!(
                "line {line_num}: variant has {} fields, need >=8",
                fields.len()
            ));
            variants += 1;
            continue;
        }

        if fields[1].parse::<u64>().is_err() {
            errors.push(format!("line {line_num}: POS is not a valid integer"));
        }

        let ref_allele = fields[3];
        if ref_allele.is_empty() || ref_allele == "." {
            errors.push(format!("line {line_num}: REF allele is empty or '.'"));
        }

        variants += 1;
    }

    if !has_column_line {
        errors.push("#CHROM column header line not found".to_string());
    }

    let is_valid = errors.is_empty();
    Ok(VcfValidation {
        variants,
        errors,
        is_valid,
        has_header,
    })
}
