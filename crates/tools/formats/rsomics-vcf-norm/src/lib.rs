use std::io;
use std::path::Path;

use noodles::vcf;
use noodles::vcf::variant::io::Write as VariantWrite;
use noodles::vcf::variant::record::AlternateBases as _;
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
            let alts = record.alternate_bases();

            if alts.len() > 1 {
                let mut buf = vcf::variant::RecordBuf::try_from_variant_record(&header, &record)
                    .map_err(|e| {
                        RsomicsError::InvalidInput(format!("converting record: {e}"))
                    })?;

                let all_alts: Vec<String> = buf
                    .alternate_bases()
                    .as_ref()
                    .to_vec();

                for alt in &all_alts {
                    *buf.alternate_bases_mut() =
                        vcf::variant::record_buf::AlternateBases::from(vec![alt.clone()]);
                    writer
                        .write_variant_record(&header, &buf)
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
