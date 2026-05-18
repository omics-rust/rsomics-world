#![allow(clippy::cast_precision_loss)]

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use noodles::bam;
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

#[derive(Debug, Default, Clone, Serialize)]
pub struct RefCoverage {
    pub name: String,
    pub length: u64,
    pub mapped_reads: u64,
    pub covered_bases: u64,
    pub mean_depth: f64,
    pub coverage_pct: f64,
}

pub fn compute_coverage(input: &Path) -> Result<Vec<RefCoverage>> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let mut reader = bam::io::Reader::new(file);
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let ref_seqs = header.reference_sequences();
    let ref_names: Vec<String> = ref_seqs.keys().map(ToString::to_string).collect();
    let ref_lens: Vec<u64> = ref_seqs.values().map(|rs| rs.length() as u64).collect();

    let mut per_ref_reads: BTreeMap<usize, u64> = BTreeMap::new();
    let mut per_ref_bases: BTreeMap<usize, u64> = BTreeMap::new();

    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;
        let flags = record.flags();
        if flags.is_unmapped() || flags.is_secondary() || flags.is_supplementary() {
            continue;
        }

        let Some(tid) = record.reference_sequence_id().transpose().ok().flatten() else {
            continue;
        };
        let seq_len = record.sequence().len() as u64;

        *per_ref_reads.entry(tid).or_insert(0) += 1;
        *per_ref_bases.entry(tid).or_insert(0) += seq_len;
    }

    let mut result = Vec::with_capacity(ref_names.len());
    for (i, name) in ref_names.iter().enumerate() {
        let length = ref_lens[i];
        let mapped_reads = per_ref_reads.get(&i).copied().unwrap_or(0);
        let total_bases = per_ref_bases.get(&i).copied().unwrap_or(0);
        let mean_depth = if length > 0 {
            total_bases as f64 / length as f64
        } else {
            0.0
        };
        let covered_bases = if mean_depth > 0.0 {
            length.min(total_bases)
        } else {
            0
        };
        let coverage_pct = if length > 0 {
            covered_bases as f64 / length as f64 * 100.0
        } else {
            0.0
        };
        result.push(RefCoverage {
            name: name.clone(),
            length,
            mapped_reads,
            covered_bases,
            mean_depth,
            coverage_pct,
        });
    }

    Ok(result)
}

pub fn write_coverage(cov: &[RefCoverage], output: &mut dyn Write) -> Result<()> {
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    writeln!(
        out,
        "#rname\tstartpos\tendpos\tnumreads\tcovbases\tcoverage\tmeandepth"
    )
    .map_err(RsomicsError::Io)?;
    for r in cov {
        writeln!(
            out,
            "{}\t1\t{}\t{}\t{}\t{:.4}\t{:.2}",
            r.name, r.length, r.mapped_reads, r.covered_bases, r.coverage_pct, r.mean_depth
        )
        .map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(())
}
