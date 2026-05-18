use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use noodles::bam;
use noodles::sam;
use noodles::sam::alignment::Record as _;
use rsomics_common::{Result, RsomicsError};

fn read_group(record: &bam::Record) -> Option<String> {
    let data = record.data();
    let rg = data.get(&sam::alignment::record::data::field::Tag::READ_GROUP);
    rg.and_then(|r| r.ok()).and_then(|v| {
        if let sam::alignment::record::data::field::Value::String(s) = v {
            Some(s.to_string())
        } else {
            None
        }
    })
}

pub fn split_bam(input: &Path, output_prefix: &Path) -> Result<HashMap<String, u64>> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let mut reader = bam::io::Reader::new(file);
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let mut writers: HashMap<String, (bam::io::Writer<File>, u64)> = HashMap::new();
    let unassigned_key = String::from("unassigned");

    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;
        let rg = read_group(&record).unwrap_or_else(|| unassigned_key.clone());

        let (writer, count) = if let Some(entry) = writers.get_mut(&rg) {
            entry
        } else {
            let path = output_path(output_prefix, &rg);
            let out_file = File::create(&path)
                .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
            let mut w = bam::io::Writer::new(out_file);
            w.write_header(&header).map_err(RsomicsError::Io)?;
            writers.insert(rg.clone(), (w, 0));
            writers.get_mut(&rg).unwrap()
        };

        writer
            .write_record(&header, &record)
            .map_err(RsomicsError::Io)?;
        *count += 1;
    }

    Ok(writers.into_iter().map(|(k, (_, c))| (k, c)).collect())
}

fn output_path(prefix: &Path, rg: &str) -> PathBuf {
    let safe_rg = rg.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_");
    let stem = prefix
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let parent = prefix.parent().unwrap_or(Path::new("."));
    parent.join(format!("{stem}.{safe_rg}.bam"))
}
