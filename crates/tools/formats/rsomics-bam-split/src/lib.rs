use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

use noodles::bam;
use rsomics_common::{Result, RsomicsError};

pub fn split_by_reference(input: &Path, output_prefix: &Path) -> Result<HashMap<String, u64>> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let mut reader = bam::io::Reader::new(file);
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let ref_names: Vec<String> = header
        .reference_sequences()
        .keys()
        .map(ToString::to_string)
        .collect();

    let mut writers: HashMap<String, (bam::io::Writer<noodles::bgzf::io::Writer<File>>, u64)> =
        HashMap::new();

    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;
        let flags = record.flags();
        if flags.is_unmapped() {
            continue;
        }

        let Some(tid) = record.reference_sequence_id().transpose().ok().flatten() else {
            continue;
        };
        let name = ref_names
            .get(tid)
            .cloned()
            .unwrap_or_else(|| format!("tid{tid}"));

        let (writer, count) = if let Some(entry) = writers.get_mut(&name) {
            entry
        } else {
            let path = output_path(output_prefix, &name);
            let out_file = File::create(&path)
                .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
            let mut w = bam::io::Writer::new(out_file);
            w.write_header(&header).map_err(RsomicsError::Io)?;
            writers.insert(name.clone(), (w, 0));
            writers.get_mut(&name).unwrap()
        };

        writer
            .write_record(&header, &record)
            .map_err(RsomicsError::Io)?;
        *count += 1;
    }

    Ok(writers.into_iter().map(|(k, (_, c))| (k, c)).collect())
}

fn output_path(prefix: &Path, ref_name: &str) -> PathBuf {
    let safe = ref_name.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_");
    let stem = prefix
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let parent = prefix.parent().unwrap_or(Path::new("."));
    parent.join(format!("{stem}.{safe}.bam"))
}
