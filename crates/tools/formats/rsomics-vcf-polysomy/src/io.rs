/// VCF/BCF reader that extracts per-chromosome BAF distributions.
///
/// Reads the FORMAT/BAF float field exactly as bcftools polysomy does.
/// Supports plain VCF and bgzf/gzip-compressed VCF (via flate2).
use crate::histogram::BafHistogram;
use crate::model::PolysomyArgs;
use flate2::read::MultiGzDecoder;
use rsomics_common::{Result, RsomicsError};
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Open a VCF/BCF path and return per-chromosome BAF histograms.
///
/// Only the FORMAT/BAF field is read; GT/AD/DP are ignored.
/// If the VCF has multiple samples, `-s` must have been specified.
pub fn read_baf_distributions(path: &Path, args: &PolysomyArgs) -> Result<Vec<BafHistogram>> {
    let raw = std::fs::File::open(path)?;
    let is_gz = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("gz") || e.eq_ignore_ascii_case("bgz"))
        .unwrap_or(false);

    if is_gz {
        let dec = MultiGzDecoder::new(raw);
        parse_vcf(BufReader::new(dec), args)
    } else {
        parse_vcf(BufReader::new(raw), args)
    }
}

fn parse_vcf(reader: impl BufRead, args: &PolysomyArgs) -> Result<Vec<BafHistogram>> {
    let mut dists: Vec<BafHistogram> = Vec::new();
    let mut sample_col: Option<usize> = None;
    let mut prev_chrom = String::new();

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("##") {
            continue;
        }
        if line.starts_with('#') {
            // Header: #CHROM POS ID REF ALT QUAL FILTER INFO FORMAT sample1 …
            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() < 9 {
                continue;
            }
            let samples: Vec<&str> = cols[9..].to_vec();
            if samples.is_empty() {
                return Err(RsomicsError::InvalidInput(
                    "VCF has no sample columns".into(),
                ));
            }
            sample_col = Some(match &args.sample {
                Some(s) => samples
                    .iter()
                    .position(|&n| n == s.as_str())
                    .ok_or_else(|| {
                        RsomicsError::InvalidInput(format!("sample '{}' not found", s))
                    })?,
                None => {
                    if samples.len() > 1 {
                        return Err(RsomicsError::InvalidInput(format!(
                            "VCF has {} samples; specify one with -s/--sample",
                            samples.len()
                        )));
                    }
                    0
                }
            });
            continue;
        }

        // Data line.
        let scol = match sample_col {
            Some(c) => c,
            None => return Err(RsomicsError::InvalidInput("data line before header".into())),
        };

        // Split fields: CHROM POS ID REF ALT QUAL FILTER INFO FORMAT sample…
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 10 {
            continue;
        }
        let chrom = cols[0].to_string();
        let fmt_field = cols[8];

        // Locate BAF in FORMAT.
        let baf_idx = match fmt_field.split(':').position(|f| f == "BAF") {
            Some(i) => i,
            None => continue,
        };

        // Extract the target sample's data.
        let sample_str = match cols.get(9 + scol) {
            Some(s) => *s,
            None => continue,
        };
        let baf_str = sample_str.split(':').nth(baf_idx).unwrap_or(".");
        if baf_str == "." {
            continue;
        }
        let baf: f32 = match baf_str.parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        if !baf.is_finite() {
            continue;
        }

        // New chromosome → new histogram.
        if chrom != prev_chrom {
            dists.push(BafHistogram::new(&chrom));
            prev_chrom = chrom;
        }
        dists.last_mut().unwrap().push(baf);
    }

    Ok(dists)
}
