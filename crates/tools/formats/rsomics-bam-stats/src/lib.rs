#![allow(clippy::cast_precision_loss)]

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use noodles::bam;
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

#[derive(Debug, Default, Clone, Serialize)]
pub struct BamStats {
    pub total_reads: u64,
    pub mapped_reads: u64,
    pub unmapped_reads: u64,
    pub paired_reads: u64,
    pub properly_paired: u64,
    pub secondary: u64,
    pub supplementary: u64,
    pub duplicates: u64,
    pub total_bases: u64,
    pub mapped_bases: u64,
    pub average_length: f64,
    pub average_mapq: f64,
    pub bases_q20: u64,
    pub bases_q30: u64,
}

pub fn compute_stats(input: &Path) -> Result<BamStats> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let mut reader = bam::io::Reader::new(file);
    let _header = reader.read_header().map_err(RsomicsError::Io)?;

    let mut s = BamStats::default();
    let mut mapq_sum: u64 = 0;
    let mut len_sum: u64 = 0;

    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;
        let flags = record.flags();

        s.total_reads += 1;

        if flags.is_unmapped() {
            s.unmapped_reads += 1;
        } else {
            s.mapped_reads += 1;
        }
        if flags.is_secondary() {
            s.secondary += 1;
        }
        if flags.is_supplementary() {
            s.supplementary += 1;
        }
        if flags.is_duplicate() {
            s.duplicates += 1;
        }
        if flags.is_segmented() {
            s.paired_reads += 1;
            if flags.is_properly_segmented() {
                s.properly_paired += 1;
            }
        }

        let seq_len = record.sequence().len();
        len_sum += seq_len as u64;
        s.total_bases += seq_len as u64;

        if !flags.is_unmapped() {
            s.mapped_bases += seq_len as u64;
            let mq = record.mapping_quality().map_or(0, |q| q.get());
            mapq_sum += u64::from(mq);
        }

        let quals = record.quality_scores();
        for q in quals.as_ref() {
            if *q >= 20 {
                s.bases_q20 += 1;
            }
            if *q >= 30 {
                s.bases_q30 += 1;
            }
        }
    }

    if s.total_reads > 0 {
        s.average_length = len_sum as f64 / s.total_reads as f64;
    }
    if s.mapped_reads > 0 {
        s.average_mapq = mapq_sum as f64 / s.mapped_reads as f64;
    }

    Ok(s)
}

pub fn write_stats(stats: &BamStats, output: &mut dyn Write) -> Result<()> {
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    writeln!(out, "SN\ttotal reads:\t{}", stats.total_reads).map_err(RsomicsError::Io)?;
    writeln!(out, "SN\tmapped reads:\t{}", stats.mapped_reads).map_err(RsomicsError::Io)?;
    writeln!(out, "SN\tunmapped reads:\t{}", stats.unmapped_reads).map_err(RsomicsError::Io)?;
    writeln!(out, "SN\tpaired reads:\t{}", stats.paired_reads).map_err(RsomicsError::Io)?;
    writeln!(out, "SN\tproperly paired:\t{}", stats.properly_paired).map_err(RsomicsError::Io)?;
    writeln!(out, "SN\tsecondary:\t{}", stats.secondary).map_err(RsomicsError::Io)?;
    writeln!(out, "SN\tsupplementary:\t{}", stats.supplementary).map_err(RsomicsError::Io)?;
    writeln!(out, "SN\tduplicates:\t{}", stats.duplicates).map_err(RsomicsError::Io)?;
    writeln!(out, "SN\ttotal bases:\t{}", stats.total_bases).map_err(RsomicsError::Io)?;
    writeln!(out, "SN\tmapped bases:\t{}", stats.mapped_bases).map_err(RsomicsError::Io)?;
    writeln!(out, "SN\taverage length:\t{:.1}", stats.average_length).map_err(RsomicsError::Io)?;
    writeln!(out, "SN\taverage mapq:\t{:.1}", stats.average_mapq).map_err(RsomicsError::Io)?;
    writeln!(out, "SN\tbases >= Q20:\t{}", stats.bases_q20).map_err(RsomicsError::Io)?;
    writeln!(out, "SN\tbases >= Q30:\t{}", stats.bases_q30).map_err(RsomicsError::Io)?;
    out.flush().map_err(RsomicsError::Io)?;
    Ok(())
}
