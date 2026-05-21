use std::fs::File;
use std::io::Write;
use std::path::Path;

use noodles::bam;
use noodles::core::Region;
use noodles::sam::alignment::io::Write as AlnWrite;
use rsomics_common::{Result, RsomicsError};

pub fn extract_region(
    bam_path: &Path,
    region_str: &str,
    output: &mut dyn Write,
    count_only: bool,
) -> Result<u64> {
    let index_path = bam_path.with_extension("bam.bai");
    let index = bam::bai::fs::read(&index_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", index_path.display())))?;

    let file = File::open(bam_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", bam_path.display())))?;
    let mut reader = bam::io::Reader::new(file);
    let header = reader
        .read_header()
        .map_err(|e| RsomicsError::InvalidInput(format!("reading header: {e}")))?;

    let region: Region = region_str
        .parse()
        .map_err(|e| RsomicsError::InvalidInput(format!("invalid region '{region_str}': {e}")))?;

    let mut query = reader
        .query(&header, &index, &region)
        .map_err(|e| RsomicsError::InvalidInput(format!("query failed: {e}")))?;

    let mut record = bam::Record::default();
    let mut count: u64 = 0;

    if count_only {
        while query
            .read_record(&mut record)
            .map_err(|e| RsomicsError::InvalidInput(format!("reading record: {e}")))?
            != 0
        {
            count += 1;
        }
        println!("{count}");
    } else {
        let mut writer = bam::io::Writer::new(output);
        writer.write_header(&header).map_err(RsomicsError::Io)?;

        while query
            .read_record(&mut record)
            .map_err(|e| RsomicsError::InvalidInput(format!("reading record: {e}")))?
            != 0
        {
            writer
                .write_alignment_record(&header, &record)
                .map_err(RsomicsError::Io)?;
            count += 1;
        }
    }

    Ok(count)
}
