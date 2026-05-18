use rsomics_common::{Result, RsomicsError};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn gff_introns(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut transcripts: BTreeMap<String, Vec<(String, u64, u64, String)>> = BTreeMap::new();
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 9 || fields[2] != "exon" {
            continue;
        }
        let chrom = fields[0].to_string();
        let start: u64 = fields[3].parse().unwrap_or(0);
        let end: u64 = fields[4].parse().unwrap_or(0);
        let strand = fields[6].to_string();
        let tid = fields[8]
            .split(';')
            .find(|s| s.contains("transcript_id"))
            .map(|s| {
                s.split(|c| c == '=' || c == ' ' || c == '"')
                    .filter(|p| !p.is_empty())
                    .nth(1)
                    .unwrap_or("?")
            })
            .unwrap_or("?")
            .to_string();
        transcripts
            .entry(tid)
            .or_default()
            .push((chrom, start, end, strand));
    }
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut count: u64 = 0;
    for (tid, mut exons) in transcripts {
        exons.sort_by_key(|(_, s, _, _)| *s);
        for w in exons.windows(2) {
            let (chrom, _, prev_end, strand) = &w[0];
            let (_, next_start, _, _) = &w[1];
            writeln!(out, "{chrom}\t{prev_end}\t{next_start}\t{tid}\t.\t{strand}")
                .map_err(RsomicsError::Io)?;
            count += 1;
        }
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
