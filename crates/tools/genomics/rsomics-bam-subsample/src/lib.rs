use std::fs::File;
use std::io::Write;
use std::path::Path;

use noodles::bam;
use noodles::sam::alignment::io::Write as AlnWrite;
use rand::SeedableRng;
use rand::distributions::{Distribution, Uniform};
use rand::rngs::StdRng;
use rsomics_common::{Result, RsomicsError};

pub fn subsample_bam(
    input: &Path,
    output: &mut dyn Write,
    fraction: f64,
    seed: u64,
) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let mut reader = bam::io::Reader::new(file);
    let header = reader.read_header().map_err(|e| {
        RsomicsError::InvalidInput(format!("reading header from {}: {e}", input.display()))
    })?;

    let mut writer = bam::io::Writer::new(output);
    writer.write_header(&header).map_err(RsomicsError::Io)?;

    let mut rng = StdRng::seed_from_u64(seed);
    let dist = Uniform::new(0.0f64, 1.0);
    let mut kept: u64 = 0;

    for result in reader.records() {
        let record = result.map_err(|e| {
            RsomicsError::InvalidInput(format!("reading record from {}: {e}", input.display()))
        })?;
        if dist.sample(&mut rng) < fraction {
            writer
                .write_alignment_record(&header, &record)
                .map_err(RsomicsError::Io)?;
            kept += 1;
        }
    }

    Ok(kept)
}
