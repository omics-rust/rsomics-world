use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

use noodles::bam;
use rsomics_common::{Result, RsomicsError};
use rsomics_intervals::Interval;

pub fn peak_counts(
    bam_path: &Path,
    bed_path: &Path,
    output: &mut dyn Write,
    min_mapq: u8,
) -> Result<u64> {
    let peaks = load_bed(bed_path)?;
    let mut counts: HashMap<(String, u64, u64), u64> = HashMap::new();
    for p in &peaks {
        counts.insert((p.chrom.clone(), p.start, p.end), 0);
    }

    let file = File::open(bam_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", bam_path.display())))?;
    let mut reader = bam::io::Reader::new(file);
    let header = reader
        .read_header()
        .map_err(|e| RsomicsError::InvalidInput(format!("reading header: {e}")))?;

    let refs: Vec<String> = header
        .reference_sequences()
        .keys()
        .map(ToString::to_string)
        .collect();

    for result in reader.records() {
        let record =
            result.map_err(|e| RsomicsError::InvalidInput(format!("reading record: {e}")))?;

        let flags = record.flags();
        if flags.is_unmapped() || flags.is_secondary() || flags.is_supplementary() {
            continue;
        }

        if record.mapping_quality().is_some_and(|q| q.get() < min_mapq) {
            continue;
        }

        let Some(Ok(tid)) = record.reference_sequence_id() else {
            continue;
        };
        let chrom = &refs[tid];

        #[allow(clippy::cast_possible_truncation)]
        let start = match record.alignment_start() {
            Some(Ok(p)) => usize::from(p) as u64,
            _ => 0,
        };

        for p in &peaks {
            if p.chrom == *chrom && start >= p.start && start < p.end {
                *counts.entry((p.chrom.clone(), p.start, p.end)).or_insert(0) += 1;
            }
        }
    }

    let mut out = BufWriter::new(output);
    let mut total = 0u64;
    for p in &peaks {
        let c = counts
            .get(&(p.chrom.clone(), p.start, p.end))
            .copied()
            .unwrap_or(0);
        writeln!(out, "{}\t{}\t{}\t{c}", p.chrom, p.start, p.end).map_err(RsomicsError::Io)?;
        total += c;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(total)
}

fn load_bed(path: &Path) -> Result<Vec<Interval>> {
    let file = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    rsomics_intervals::bed::read(BufReader::new(file))
        .map_err(|e| RsomicsError::InvalidInput(format!("reading BED: {e}")))
}
