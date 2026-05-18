#![allow(clippy::cast_precision_loss)]

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use noodles::bam;
use rsomics_common::{Result, RsomicsError};

#[derive(Debug, Clone)]
pub struct DepthOpts {
    pub min_mapq: u8,
    pub max_depth: u32,
    pub skip_flags: u16,
}

impl Default for DepthOpts {
    fn default() -> Self {
        Self {
            min_mapq: 0,
            max_depth: 8000,
            skip_flags: 0x704,
        }
    }
}

fn tid(r: &bam::Record) -> Option<usize> {
    r.reference_sequence_id().transpose().ok().flatten()
}

fn pos(r: &bam::Record) -> Option<usize> {
    r.alignment_start().transpose().ok().flatten().map(|p| p.get())
}

pub fn compute_depth(input: &Path, output: &mut dyn Write, opts: &DepthOpts) -> Result<u64> {
    let mut reader = File::open(input)
        .map(bam::io::Reader::new)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let header = reader.read_header().map_err(RsomicsError::Io)?;
    let ref_names: Vec<String> = header
        .reference_sequences()
        .keys()
        .map(ToString::to_string)
        .collect();

    let mut depth_map: BTreeMap<usize, BTreeMap<usize, u32>> = BTreeMap::new();

    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;
        let flags = record.flags();

        if (flags.bits() & opts.skip_flags) != 0 {
            continue;
        }

        let mq = record.mapping_quality().map_or(0, |q| q.get());
        if mq < opts.min_mapq {
            continue;
        }

        let Some(t) = tid(&record) else { continue };
        let Some(start) = pos(&record) else { continue };

        let d = depth_map.entry(t).or_default().entry(start).or_insert(0);
        if *d < opts.max_depth {
            *d += 1;
        }
    }

    let mut out = BufWriter::with_capacity(256 * 1024, output);
    let mut lines: u64 = 0;

    for (t, positions) in &depth_map {
        let name = ref_names.get(*t).map_or("*", |n| n.as_str());
        for (&p, &depth) in positions {
            writeln!(out, "{name}\t{p}\t{depth}").map_err(RsomicsError::Io)?;
            lines += 1;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(lines)
}
