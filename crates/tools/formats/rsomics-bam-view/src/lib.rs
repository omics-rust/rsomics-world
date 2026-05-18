use std::fs::File;
use std::io::Write;
use std::path::Path;

use noodles::bam;
use noodles::sam;
use rsomics_common::{Result, RsomicsError};

#[derive(Debug, Clone, Default)]
pub struct ViewFilter {
    pub require_flags: u16,
    pub exclude_flags: u16,
    pub min_mapq: u8,
    pub count_only: bool,
}

fn passes(flags: sam::alignment::record::Flags, mapq: Option<u8>, f: &ViewFilter) -> bool {
    let bits = flags.bits();
    if f.require_flags != 0 && (bits & f.require_flags) != f.require_flags {
        return false;
    }
    if f.exclude_flags != 0 && (bits & f.exclude_flags) != 0 {
        return false;
    }
    if f.min_mapq > 0 && mapq.unwrap_or(0) < f.min_mapq {
        return false;
    }
    true
}

pub fn view_bam(input: &Path, output: &mut dyn Write, filter: &ViewFilter) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let mut reader = bam::io::Reader::new(file);
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    if filter.count_only {
        return count_bam(&mut reader, filter);
    }

    let mut writer = bam::io::Writer::new(output);
    writer.write_header(&header).map_err(RsomicsError::Io)?;

    let mut count: u64 = 0;
    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;
        let flags = record.flags();
        let mapq = record.mapping_quality().map(|q| q.get());

        if !passes(flags, mapq, filter) {
            continue;
        }
        count += 1;
        writer
            .write_record(&header, &record)
            .map_err(RsomicsError::Io)?;
    }
    Ok(count)
}

fn count_bam<R: std::io::Read>(
    reader: &mut bam::io::Reader<noodles::bgzf::io::Reader<R>>,
    filter: &ViewFilter,
) -> Result<u64> {
    let mut count: u64 = 0;
    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;
        let flags = record.flags();
        let mapq = record.mapping_quality().map(|q| q.get());
        if passes(flags, mapq, filter) {
            count += 1;
        }
    }
    Ok(count)
}
