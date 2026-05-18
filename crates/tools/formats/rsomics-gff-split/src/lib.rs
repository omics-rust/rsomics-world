use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn split_gff(input: &Path, prefix: &Path) -> Result<HashMap<String, u64>> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);

    let mut header_lines: Vec<String> = Vec::new();
    let mut writers: HashMap<String, (BufWriter<File>, u64)> = HashMap::new();

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;

        if line.starts_with('#') {
            header_lines.push(line);
            continue;
        }
        if line.is_empty() {
            continue;
        }

        let chrom = line.split('\t').next().unwrap_or("unknown").to_string();

        let (writer, count) = if let Some(entry) = writers.get_mut(&chrom) {
            entry
        } else {
            let path = output_path(prefix, &chrom);
            let out_file = File::create(&path)
                .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
            let mut w = BufWriter::new(out_file);
            for h in &header_lines {
                writeln!(w, "{h}").map_err(RsomicsError::Io)?;
            }
            writers.insert(chrom.clone(), (w, 0));
            writers.get_mut(&chrom).unwrap()
        };

        writeln!(writer, "{line}").map_err(RsomicsError::Io)?;
        *count += 1;
    }

    for (_, (w, _)) in &mut writers {
        w.flush().map_err(RsomicsError::Io)?;
    }

    Ok(writers.into_iter().map(|(k, (_, c))| (k, c)).collect())
}

fn output_path(prefix: &Path, chrom: &str) -> std::path::PathBuf {
    let safe = chrom.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_");
    let stem = prefix
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let parent = prefix.parent().unwrap_or(Path::new("."));
    parent.join(format!("{stem}.{safe}.gff"))
}
