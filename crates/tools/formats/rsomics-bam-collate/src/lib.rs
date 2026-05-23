//! In-memory QNAME collation, a Rust port of `samtools collate` (MIT).
//!
//! Collation makes every record sharing a QNAME contiguous in the output so a
//! downstream `fixmate`/`markdup` pipeline sees mates adjacent. It is not a
//! coordinate sort and not a name sort: the inter-group order is unconstrained,
//! only intra-group adjacency matters.
//!
//! samtools collate (`bamshuf.c`, `main_bamshuf`) achieves this by hashing every
//! record's name to one of N temporary files on disk, then reading each temp
//! file back and shuffling its records by hash key — a full disk round-trip of
//! the input. That cost is justified for inputs larger than RAM; for an input
//! that fits in memory it is pure overhead. This crate groups entirely in
//! memory: one read pass builds the groups, one write pass emits them, no temp
//! files. Groups are emitted in first-seen-QNAME order — itself a valid
//! collation (all same-QNAME records contiguous) and deterministic for a given
//! input, so the output bytes are reproducible across runs. The inter-group
//! order therefore differs from samtools' hash order; both satisfy the only
//! contract collation has.
//!
//! Records pass through byte-for-byte via the [`rsomics_bamio::raw`] path —
//! seq/qual/cigar/name are never decoded, only the name is read to key the
//! grouping.

use std::collections::HashMap;
use std::io::Write;
use std::num::NonZero;
use std::path::Path;

use noodles::bam;
use noodles::bgzf;
use rsomics_bamio::raw::{self, RawRecord};
use rsomics_common::{Result, RsomicsError};

#[derive(Debug, Clone, Default)]
pub struct CollateOpts {
    /// Write uncompressed BGZF (`-u`). Skips deflate so the write pass is
    /// cheaper when the output feeds straight into another tool.
    pub uncompressed: bool,
}

/// Read every record from `input`, group by first-seen QNAME, and write the
/// groups out so all records sharing a QNAME are contiguous. `output_path` of
/// `None` writes BAM to stdout (always compressed; `-u` only applies to file
/// output, matching the way a stdout consumer expects a BAM stream).
pub fn collate(
    input: &Path,
    output_path: Option<&Path>,
    opts: &CollateOpts,
    workers: NonZero<usize>,
) -> Result<u64> {
    let mut reader = rsomics_bamio::open_with_workers(input, workers)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let groups = read_groups(&mut reader)?;
    let total: u64 = groups.iter().map(|g| g.len() as u64).sum();

    match output_path {
        Some(path) => {
            if opts.uncompressed {
                let file = std::fs::File::create(path).map_err(|e| {
                    RsomicsError::InvalidInput(format!("creating {}: {e}", path.display()))
                })?;
                let inner = bgzf::io::multithreaded_writer::Builder::default()
                    .set_compression_level(bgzf::io::writer::CompressionLevel::NONE)
                    .set_worker_count(workers)
                    .build_from_writer(file);
                let mut writer = bam::io::Writer::from(inner);
                write_groups(&mut writer, &header, &groups)?;
            } else {
                let mut writer = rsomics_bamio::create_with_workers(path, workers)?;
                write_groups(&mut writer, &header, &groups)?;
            }
        }
        None => {
            let mut writer = bam::io::Writer::new(std::io::stdout().lock());
            write_groups(&mut writer, &header, &groups)?;
        }
    }

    Ok(total)
}

/// One read pass: bucket every record into a per-QNAME group, preserving the
/// order in which each QNAME was first seen. The group index map keys on the
/// raw name bytes so no name decode/allocation beyond the key copy occurs.
fn read_groups<R: std::io::Read>(reader: &mut bam::io::Reader<R>) -> Result<Vec<Vec<RawRecord>>> {
    let mut groups: Vec<Vec<RawRecord>> = Vec::new();
    let mut index: HashMap<Vec<u8>, usize> = HashMap::new();
    let mut rec = RawRecord::default();

    while raw::read_record(reader.get_mut(), &mut rec)? != 0 {
        let record = std::mem::take(&mut rec);
        let slot = index.get(record.name()).copied();
        match slot {
            Some(i) => groups[i].push(record),
            None => {
                index.insert(record.name().to_vec(), groups.len());
                groups.push(vec![record]);
            }
        }
    }
    Ok(groups)
}

/// One write pass: header, then every group's records in first-seen order.
/// Within a group records keep their input order, so a name-grouped but
/// otherwise input-preserving stream is emitted.
fn write_groups<W: Write>(
    writer: &mut bam::io::Writer<W>,
    header: &noodles::sam::Header,
    groups: &[Vec<RawRecord>],
) -> Result<()> {
    writer.write_header(header).map_err(RsomicsError::Io)?;
    for group in groups {
        for record in group {
            raw::write_record(writer.get_mut(), record)?;
        }
    }
    Ok(())
}
