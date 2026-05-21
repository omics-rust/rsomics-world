use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub struct TaxEntry {
    pub pct: f64,
    pub reads_clade: u64,
    pub reads_direct: u64,
    pub rank: String,
    pub taxid: u64,
    pub name: String,
}

pub fn parse_report(input: &Path) -> Result<Vec<TaxEntry>> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 6 {
            continue;
        }
        let pct: f64 = parts[0].trim().parse().unwrap_or(0.0);
        let reads_clade: u64 = parts[1].trim().parse().unwrap_or(0);
        let reads_direct: u64 = parts[2].trim().parse().unwrap_or(0);
        let rank = parts[3].trim().to_string();
        let taxid: u64 = parts[4].trim().parse().unwrap_or(0);
        let name = parts[5..].join("\t").trim().to_string();
        entries.push(TaxEntry {
            pct,
            reads_clade,
            reads_direct,
            rank,
            taxid,
            name,
        });
    }
    Ok(entries)
}

pub fn top_taxa(entries: &[TaxEntry], rank: &str, n: usize, output: &mut dyn Write) -> Result<()> {
    let mut out = BufWriter::new(output);
    writeln!(out, "pct\treads\ttaxid\tname").map_err(RsomicsError::Io)?;

    let mut filtered: Vec<&TaxEntry> = entries
        .iter()
        .filter(|e| rank.is_empty() || e.rank == rank)
        .collect();
    filtered.sort_by_key(|e| std::cmp::Reverse(e.reads_clade));

    for e in filtered.iter().take(n) {
        writeln!(
            out,
            "{:.2}\t{}\t{}\t{}",
            e.pct, e.reads_clade, e.taxid, e.name
        )
        .map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(())
}
