use std::io::{self, BufWriter, Write};
use std::path::Path;

use noodles::vcf;
use rsomics_common::{Result, RsomicsError};

pub fn query_vcf(input: &Path, output: &mut dyn io::Write, fields: &[String]) -> Result<u64> {
    let mut reader = vcf::io::reader::Builder::default()
        .build_from_path(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let header = reader
        .read_header()
        .map_err(|e| RsomicsError::InvalidInput(format!("reading VCF header: {e}")))?;

    let mut out = BufWriter::with_capacity(256 * 1024, output);
    let mut count: u64 = 0;

    for result in reader.records() {
        let record =
            result.map_err(|e| RsomicsError::InvalidInput(format!("reading VCF record: {e}")))?;
        count += 1;

        let mut first = true;
        for field in fields {
            if !first {
                out.write_all(b"\t").map_err(RsomicsError::Io)?;
            }
            first = false;

            let val = extract_field(&record, &header, field);
            out.write_all(val.as_bytes()).map_err(RsomicsError::Io)?;
        }
        out.write_all(b"\n").map_err(RsomicsError::Io)?;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}

fn extract_field(
    record: &vcf::Record,
    _header: &vcf::Header,
    field: &str,
) -> String {
    match field.to_uppercase().as_str() {
        "CHROM" => record
            .reference_sequence_name()
            .to_string(),
        "POS" => record
            .variant_start()
            .map_or(".".to_string(), |p| p.get().to_string()),
        "ID" => {
            let ids = record.ids();
            if ids.is_empty() {
                ".".to_string()
            } else {
                ids.to_string()
            }
        }
        "REF" => record.reference_bases().to_string(),
        "ALT" => {
            let alts = record.alternate_bases();
            if alts.is_empty() {
                ".".to_string()
            } else {
                alts.to_string()
            }
        }
        "QUAL" => record
            .quality_score()
            .map_or(".".to_string(), |q| format!("{q}")),
        "FILTER" => {
            let filters = record.filters();
            if filters.is_empty() {
                ".".to_string()
            } else {
                filters.to_string()
            }
        }
        _ => ".".to_string(),
    }
}
