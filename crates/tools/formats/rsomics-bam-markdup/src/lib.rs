use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use noodles::bam;
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

#[derive(Debug, Default, Clone, Serialize)]
pub struct MarkdupStats {
    pub total: u64,
    pub duplicates_marked: u64,
    pub duplicates_removed: u64,
    pub optical_duplicates: u64,
}

#[derive(Debug, Clone, Default)]
pub struct MarkdupOpts {
    pub remove: bool,
}

#[derive(Hash, PartialEq, Eq)]
struct DupKey {
    tid: usize,
    pos: usize,
    mate_tid: usize,
    mate_pos: usize,
    is_reverse: bool,
}

fn dup_key(record: &bam::Record) -> Option<DupKey> {
    let flags = record.flags();
    if flags.is_unmapped() || flags.is_secondary() || flags.is_supplementary() {
        return None;
    }

    let tid = record.reference_sequence_id().transpose().ok().flatten()?;
    let pos = record
        .alignment_start()
        .transpose()
        .ok()
        .flatten()
        .map(|p| p.get())?;

    let mate_tid = record
        .mate_reference_sequence_id()
        .transpose()
        .ok()
        .flatten()
        .unwrap_or(0);
    let mate_pos = record
        .mate_alignment_start()
        .transpose()
        .ok()
        .flatten()
        .map_or(0, |p| p.get());

    Some(DupKey {
        tid,
        pos,
        mate_tid,
        mate_pos,
        is_reverse: flags.is_reverse_complemented(),
    })
}

fn sum_quals(record: &bam::Record) -> u64 {
    record
        .quality_scores()
        .as_ref()
        .iter()
        .map(|&q| u64::from(q))
        .sum()
}

pub fn markdup(input: &Path, output: &mut dyn Write, opts: &MarkdupOpts) -> Result<MarkdupStats> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let mut reader = bam::io::Reader::new(file);
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let mut records: Vec<bam::Record> = Vec::new();
    for result in reader.records() {
        records.push(result.map_err(RsomicsError::Io)?);
    }

    let mut seen: HashMap<DupKey, usize> = HashMap::new();
    let mut is_dup: Vec<bool> = vec![false; records.len()];

    for (i, record) in records.iter().enumerate() {
        let Some(key) = dup_key(record) else {
            continue;
        };
        if let Some(&best_idx) = seen.get(&key) {
            if sum_quals(record) > sum_quals(&records[best_idx]) {
                is_dup[best_idx] = true;
                seen.insert(key, i);
            } else {
                is_dup[i] = true;
            }
        } else {
            seen.insert(key, i);
        }
    }

    let mut writer = bam::io::Writer::new(output);
    writer.write_header(&header).map_err(RsomicsError::Io)?;

    let mut stats = MarkdupStats {
        total: records.len() as u64,
        ..Default::default()
    };

    for (i, record) in records.iter().enumerate() {
        if is_dup[i] {
            if opts.remove {
                stats.duplicates_removed += 1;
                continue;
            }
            stats.duplicates_marked += 1;
        }
        writer
            .write_record(&header, record)
            .map_err(RsomicsError::Io)?;
    }

    Ok(stats)
}
