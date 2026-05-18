#![allow(clippy::cast_precision_loss)]

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use noodles::bam;
use noodles::sam;
use noodles::sam::alignment::record::Flags;
use rsomics_common::{Result, RsomicsError};

#[derive(Debug, Clone)]
pub struct ViewFilter {
    pub require_flags: u16,
    pub exclude_flags: u16,
    pub min_mapq: u8,
    pub count_only: bool,
    pub with_header: bool,
    pub output_bam: bool,
}

impl Default for ViewFilter {
    fn default() -> Self {
        Self {
            require_flags: 0,
            exclude_flags: 0,
            min_mapq: 0,
            count_only: false,
            with_header: true,
            output_bam: false,
        }
    }
}

fn passes_filter(flags: Flags, mapq: Option<io::Result<sam::alignment::record::MappingQuality>>, filter: &ViewFilter) -> bool {
    let bits = flags.bits();
    if filter.require_flags != 0 && (bits & filter.require_flags) != filter.require_flags {
        return false;
    }
    if filter.exclude_flags != 0 && (bits & filter.exclude_flags) != 0 {
        return false;
    }
    if filter.min_mapq > 0 {
        let mq = mapq
            .and_then(|r| r.ok())
            .map_or(0, |q| q.get());
        if mq < filter.min_mapq {
            return false;
        }
    }
    true
}

pub fn view_bam(input: &Path, output: &mut dyn Write, filter: &ViewFilter) -> Result<u64> {
    let mut reader = File::open(input)
        .map(bam::io::Reader::new)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let header = reader.read_header().map_err(RsomicsError::Io)?;

    if filter.output_bam {
        return view_bam_to_bam(&mut reader, &header, output, filter);
    }

    let mut out = BufWriter::with_capacity(256 * 1024, output);

    if filter.with_header && !filter.count_only {
        write!(out, "{header}").map_err(RsomicsError::Io)?;
    }

    let mut count: u64 = 0;
    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;
        let flags = record.flags().map_err(RsomicsError::Io)?;
        let mapq = record.mapping_quality();

        if !passes_filter(flags, mapq, filter) {
            continue;
        }
        count += 1;

        if !filter.count_only {
            let mut buf = Vec::new();
            sam::io::Writer::new(&mut buf)
                .write_alignment_record(&header, &record)
                .map_err(RsomicsError::Io)?;
            out.write_all(&buf).map_err(RsomicsError::Io)?;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}

fn view_bam_to_bam(
    reader: &mut bam::io::Reader<File>,
    header: &sam::Header,
    output: &mut dyn Write,
    filter: &ViewFilter,
) -> Result<u64> {
    let mut writer = bam::io::Writer::new(output);
    writer.write_header(header).map_err(RsomicsError::Io)?;

    let mut count: u64 = 0;
    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;
        let flags = record.flags().map_err(RsomicsError::Io)?;
        let mapq = record.mapping_quality();

        if !passes_filter(flags, mapq, filter) {
            continue;
        }
        count += 1;

        writer
            .write_record(header, &record)
            .map_err(RsomicsError::Io)?;
    }
    Ok(count)
}
