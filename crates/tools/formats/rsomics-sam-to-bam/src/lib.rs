use std::fs::File;
use std::io::{self, BufReader, Write};
use std::path::Path;

use noodles::bam;
use noodles::sam;
use rsomics_common::{Result, RsomicsError};

pub fn convert(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let reader: Box<dyn io::BufRead> = if input == Path::new("-") {
        Box::new(BufReader::new(io::stdin().lock()))
    } else {
        let file = File::open(input)
            .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
        Box::new(BufReader::new(file))
    };

    let mut sam_reader = sam::io::Reader::new(reader);
    let header = sam_reader.read_header().map_err(RsomicsError::Io)?;

    let mut writer = bam::io::Writer::new(output);
    writer.write_header(&header).map_err(RsomicsError::Io)?;

    let mut count: u64 = 0;
    for result in sam_reader.records() {
        let record = result.map_err(RsomicsError::Io)?;
        writer
            .write_record(&header, &record)
            .map_err(RsomicsError::Io)?;
        count += 1;
    }

    Ok(count)
}
