use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub struct SampleEntry {
    pub sample_id: String,
    pub r1: String,
    pub r2: Option<String>,
    pub valid: bool,
    pub errors: Vec<String>,
}

pub fn validate_sample_sheet(input: &Path, output: &mut dyn Write) -> Result<Vec<SampleEntry>> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut out = BufWriter::new(output);

    writeln!(out, "sample_id\tr1\tr2\tstatus\terrors").map_err(RsomicsError::Io)?;

    for (line_num, line) in reader.lines().enumerate() {
        let line = line.map_err(RsomicsError::Io)?;
        let line = line.trim().to_string();
        if line.is_empty() || line.starts_with('#') || line_num == 0 && line.contains("sample") {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.is_empty() {
            continue;
        }

        let sample_id = parts[0].to_string();
        let r1 = parts.get(1).unwrap_or(&"").to_string();
        let r2 = parts
            .get(2)
            .map(ToString::to_string)
            .filter(|s| !s.is_empty());

        let mut errors = Vec::new();
        if sample_id.is_empty() {
            errors.push("empty sample_id".to_string());
        }
        if r1.is_empty() {
            errors.push("missing R1 path".to_string());
        } else if !Path::new(&r1).exists() {
            errors.push(format!("R1 not found: {r1}"));
        }
        if let Some(ref r2_path) = r2 && !Path::new(r2_path).exists() {
            errors.push(format!("R2 not found: {r2_path}"));
        }

        let valid = errors.is_empty();
        let status = if valid { "OK" } else { "ERROR" };
        let err_str = errors.join("; ");
        let r2_str = r2.as_deref().unwrap_or("");
        writeln!(out, "{sample_id}\t{r1}\t{r2_str}\t{status}\t{err_str}")
            .map_err(RsomicsError::Io)?;

        entries.push(SampleEntry {
            sample_id,
            r1,
            r2,
            valid,
            errors,
        });
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(entries)
}
