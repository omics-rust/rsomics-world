#![allow(clippy::cast_precision_loss)]

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use noodles::bam;
use noodles::sam;
use noodles::sam::alignment::Record as _;
use rsomics_common::{Result, RsomicsError};

#[derive(Debug, Clone)]
pub struct DepthOpts {
    pub min_mapq: u8,
    pub min_baseq: u8,
    pub max_depth: u32,
    pub skip_flags: u16,
}

impl Default for DepthOpts {
    fn default() -> Self {
        Self {
            min_mapq: 0,
            min_baseq: 0,
            max_depth: 8000,
            skip_flags: 0x704, // unmapped + secondary + supplementary + dup
        }
    }
}

pub fn compute_depth(
    input: &Path,
    output: &mut dyn Write,
    opts: &DepthOpts,
) -> Result<u64> {
    let mut reader = File::open(input)
        .map(bam::io::Reader::new)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let header = reader.read_header().map_err(RsomicsError::Io)?;
    let ref_seqs = header.reference_sequences();

    let ref_names: Vec<String> = ref_seqs
        .keys()
        .map(|k| k.to_string())
        .collect();

    // Per-reference depth accumulator: ref_idx → position → depth
    let mut depth_map: BTreeMap<usize, BTreeMap<usize, u32>> = BTreeMap::new();

    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;
        let flags = record.flags().map_err(RsomicsError::Io)?;

        if (flags.bits() & opts.skip_flags) != 0 {
            continue;
        }

        let mq = record
            .mapping_quality()
            .and_then(|r| r.ok())
            .map_or(0, |q| q.get());
        if mq < opts.min_mapq {
            continue;
        }

        let tid = match record.reference_sequence_id().transpose() {
            Ok(Some(id)) => id,
            _ => continue,
        };

        let start = match record.alignment_start().transpose() {
            Ok(Some(pos)) => pos.get(),
            _ => continue,
        };

        let end = match record.alignment_end().transpose() {
            Ok(Some(pos)) => pos.get(),
            _ => start + 1,
        };

        let ref_depths = depth_map.entry(tid).or_default();
        for pos in start..=end {
            let d = ref_depths.entry(pos).or_insert(0);
            if *d < opts.max_depth {
                *d += 1;
            }
        }
    }

    let mut out = BufWriter::with_capacity(256 * 1024, output);
    let mut lines: u64 = 0;

    for (tid, positions) in &depth_map {
        let name = ref_names.get(*tid).map_or("*", |n| n.as_str());
        for (&pos, &depth) in positions {
            writeln!(out, "{name}\t{pos}\t{depth}").map_err(RsomicsError::Io)?;
            lines += 1;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(lines)
}
