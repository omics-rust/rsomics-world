use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

struct Annotation {
    start: u64,
    end: u64,
    label: String,
}

pub fn annotate_vcf(
    vcf_path: &Path,
    bed_path: &Path,
    output: &mut dyn Write,
    tag: &str,
) -> Result<u64> {
    let annotations = load_annotations(bed_path)?;

    let file = File::open(vcf_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", vcf_path.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut count: u64 = 0;

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 8 {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
            continue;
        }

        let chrom = fields[0];
        let pos: u64 = fields[1].parse().unwrap_or(0);

        let label = find_annotation(&annotations, chrom, pos);

        if let Some(label) = label {
            let info = if fields[7] == "." {
                format!("{tag}={label}")
            } else {
                format!("{};{tag}={label}", fields[7])
            };
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    write!(out, "\t").map_err(RsomicsError::Io)?;
                }
                if i == 7 {
                    write!(out, "{info}").map_err(RsomicsError::Io)?;
                } else {
                    write!(out, "{field}").map_err(RsomicsError::Io)?;
                }
            }
            writeln!(out).map_err(RsomicsError::Io)?;
        } else {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
        }
        count += 1;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}

fn find_annotation(
    annotations: &BTreeMap<String, Vec<Annotation>>,
    chrom: &str,
    pos: u64,
) -> Option<String> {
    let anns = annotations.get(chrom)?;
    for ann in anns {
        if pos >= ann.start && pos <= ann.end {
            return Some(ann.label.clone());
        }
    }
    None
}

fn load_annotations(path: &Path) -> Result<BTreeMap<String, Vec<Annotation>>> {
    let file = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let reader = BufReader::new(file);
    let mut by_chrom: BTreeMap<String, Vec<Annotation>> = BTreeMap::new();

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 4 {
            continue;
        }
        let chrom = fields[0].to_string();
        let start: u64 = fields[1].parse().unwrap_or(0);
        let end: u64 = fields[2].parse().unwrap_or(0);
        let label = fields[3].to_string();
        by_chrom
            .entry(chrom)
            .or_default()
            .push(Annotation { start, end, label });
    }

    for anns in by_chrom.values_mut() {
        anns.sort_by_key(|a| a.start);
    }

    Ok(by_chrom)
}
