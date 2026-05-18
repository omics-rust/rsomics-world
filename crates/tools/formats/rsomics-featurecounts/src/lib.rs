#![allow(clippy::cast_precision_loss, clippy::implicit_hasher)]

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use noodles::bam;
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct Feature {
    pub gene_id: String,
    pub chrom: String,
    pub start: u64,
    pub end: u64,
    pub strand: char,
}

#[derive(Debug, Clone)]
pub struct CountOpts {
    pub feature_type: String,
    pub attribute: String,
    pub strand_specific: u8,
    pub min_mapq: u8,
}

impl Default for CountOpts {
    fn default() -> Self {
        Self {
            feature_type: "exon".to_string(),
            attribute: "gene_id".to_string(),
            strand_specific: 0,
            min_mapq: 0,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct CountSummary {
    pub assigned: u64,
    pub unassigned_no_features: u64,
    pub unassigned_ambiguity: u64,
    pub total_reads: u64,
}

pub fn load_features(gff_path: &Path, opts: &CountOpts) -> Result<Vec<Feature>> {
    let file = File::open(gff_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", gff_path.display())))?;
    let reader = BufReader::new(file);
    let mut features = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 9 || fields[2] != opts.feature_type {
            continue;
        }
        let chrom = fields[0].to_string();
        let start: u64 = fields[3].parse().unwrap_or(0);
        let end: u64 = fields[4].parse().unwrap_or(0);
        let strand = fields[6].chars().next().unwrap_or('.');
        let gene_id = extract_attr(fields[8], &opts.attribute).unwrap_or_default();
        if gene_id.is_empty() {
            continue;
        }
        features.push(Feature {
            gene_id,
            chrom,
            start,
            end,
            strand,
        });
    }

    Ok(features)
}

fn extract_attr(attrs: &str, key: &str) -> Option<String> {
    for part in attrs.split(';') {
        let part = part.trim();
        let Some(rest) = part.strip_prefix(key) else {
            continue;
        };
        if rest.starts_with('=') || rest.starts_with(' ') {
            let val = rest[1..].trim().trim_matches('"');
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

pub fn count_reads(
    bam_path: &Path,
    features: &[Feature],
    opts: &CountOpts,
) -> Result<(HashMap<String, u64>, CountSummary)> {
    let file = File::open(bam_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", bam_path.display())))?;
    let mut reader = bam::io::Reader::new(file);
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let ref_names: Vec<String> = header
        .reference_sequences()
        .keys()
        .map(ToString::to_string)
        .collect();

    let mut chrom_features: HashMap<String, Vec<&Feature>> = HashMap::new();
    for f in features {
        chrom_features.entry(f.chrom.clone()).or_default().push(f);
    }

    let mut counts: HashMap<String, u64> = HashMap::new();
    let mut summary = CountSummary::default();

    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;
        let flags = record.flags();
        if flags.is_unmapped() || flags.is_secondary() || flags.is_supplementary() {
            continue;
        }

        summary.total_reads += 1;

        let mq = record.mapping_quality().map_or(0, |q| q.get());
        if mq < opts.min_mapq {
            summary.unassigned_no_features += 1;
            continue;
        }

        let Some(tid) = record.reference_sequence_id().transpose().ok().flatten() else {
            summary.unassigned_no_features += 1;
            continue;
        };
        let Some(chrom) = ref_names.get(tid) else {
            summary.unassigned_no_features += 1;
            continue;
        };
        let Some(start) = record.alignment_start().transpose().ok().flatten() else {
            summary.unassigned_no_features += 1;
            continue;
        };
        let read_start = start.get() as u64;
        let read_end = read_start + record.sequence().len() as u64;

        let Some(chr_feats) = chrom_features.get(chrom.as_str()) else {
            summary.unassigned_no_features += 1;
            continue;
        };

        let mut hits: Vec<&str> = Vec::new();
        for f in chr_feats {
            if read_start < f.end && read_end > f.start {
                hits.push(&f.gene_id);
            }
        }

        match hits.len() {
            0 => summary.unassigned_no_features += 1,
            1 => {
                *counts.entry(hits[0].to_string()).or_insert(0) += 1;
                summary.assigned += 1;
            }
            _ => {
                hits.dedup();
                if hits.len() == 1 {
                    *counts.entry(hits[0].to_string()).or_insert(0) += 1;
                    summary.assigned += 1;
                } else {
                    summary.unassigned_ambiguity += 1;
                }
            }
        }
    }

    Ok((counts, summary))
}

pub fn write_counts(
    counts: &HashMap<String, u64>,
    features: &[Feature],
    output: &mut dyn Write,
) -> Result<()> {
    let mut out = BufWriter::with_capacity(256 * 1024, output);

    let mut gene_order: Vec<&str> = features.iter().map(|f| f.gene_id.as_str()).collect();
    gene_order.dedup();

    writeln!(out, "Geneid\tCount").map_err(RsomicsError::Io)?;
    for gene in &gene_order {
        let count = counts.get(*gene).copied().unwrap_or(0);
        writeln!(out, "{gene}\t{count}").map_err(RsomicsError::Io)?;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(())
}
