use std::io;
use std::path::Path;

use noodles::vcf;
use rsomics_common::{Result, RsomicsError};

pub struct NormStats {
    pub total: u64,
    pub split: u64,
    pub realigned: u64,
}

pub fn normalize_vcf(
    input: &Path,
    output: &mut dyn io::Write,
    split_multiallelic: bool,
) -> Result<NormStats> {
    let mut reader = vcf::io::reader::Builder::default()
        .build_from_path(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let header = reader
        .read_header()
        .map_err(|e| RsomicsError::InvalidInput(format!("reading VCF header: {e}")))?;

    let mut writer = vcf::io::Writer::new(output);
    writer
        .write_header(&header)
        .map_err(|e| RsomicsError::InvalidInput(format!("writing header: {e}")))?;

    let mut stats = NormStats {
        total: 0,
        split: 0,
        realigned: 0,
    };

    for result in reader.records() {
        let record =
            result.map_err(|e| RsomicsError::InvalidInput(format!("reading VCF record: {e}")))?;
        stats.total += 1;

        if split_multiallelic {
            let alts: Vec<_> = record
                .alternate_bases()
                .iter()
                .collect();

            if alts.len() > 1 {
                for alt in &alts {
                    let mut split_rec = record.clone();
                    let new_alts = vcf::record::AlternateBases::from(vec![(*alt).clone()]);
                    *split_rec.alternate_bases_mut() = new_alts;
                    writer
                        .write_record(&header, &split_rec)
                        .map_err(|e| {
                            RsomicsError::InvalidInput(format!("writing record: {e}"))
                        })?;
                    stats.split += 1;
                }
                continue;
            }
        }

        writer
            .write_record(&header, &record)
            .map_err(|e| RsomicsError::InvalidInput(format!("writing record: {e}")))?;
    }

    Ok(stats)
}
