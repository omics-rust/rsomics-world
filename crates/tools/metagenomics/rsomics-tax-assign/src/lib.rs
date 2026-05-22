use std::collections::HashMap;
use std::io::{BufRead, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};
use rsomics_taxonomy::Taxonomy;

pub struct ClassifyResult {
    pub total_reads: u64,
    pub classified: u64,
    pub unclassified: u64,
}

/// Classify reads using k-mer LCA (Kraken2-style).
///
/// For each read: extract canonical k-mers, look up taxid in DB,
/// collect all hit taxids, compute their LCA via the taxonomy tree.
/// Falls back to majority-vote if no taxonomy provided.
#[allow(clippy::implicit_hasher)]
pub fn classify_reads(
    reads: &Path,
    db: &HashMap<u64, u32>,
    k: usize,
    taxonomy: Option<&Taxonomy>,
    output: &mut dyn Write,
) -> Result<ClassifyResult> {
    let mut reader = needletail::parse_fastx_file(reads)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", reads.display())))?;

    let mut out = BufWriter::new(output);
    let mut total = 0u64;
    let mut classified = 0u64;

    while let Some(result) = reader.next() {
        let record =
            result.map_err(|e| RsomicsError::InvalidInput(format!("reading record: {e}")))?;
        let name = std::str::from_utf8(record.id())
            .map_err(|e| RsomicsError::InvalidInput(format!("non-UTF8 name: {e}")))?;
        let seq = record.seq();
        total += 1;

        let iter = rsomics_kmer::KmerIter::new(&seq, k, true)
            .map_err(|e| RsomicsError::InvalidInput(format!("kmer: {e}")))?;

        let mut hit_taxids: Vec<u32> = Vec::new();
        for kmer in iter.flatten() {
            if let Some(&taxid) = db.get(&kmer) {
                hit_taxids.push(taxid);
            }
        }

        if hit_taxids.is_empty() {
            writeln!(out, "U\t{name}\t0\t0").map_err(RsomicsError::Io)?;
        } else {
            let n_hits = hit_taxids.len();
            let assigned = if let Some(tax) = taxonomy {
                tax.lca(&hit_taxids).unwrap_or(0)
            } else {
                majority_vote(&hit_taxids)
            };
            writeln!(out, "C\t{name}\t{assigned}\t{n_hits}").map_err(RsomicsError::Io)?;
            classified += 1;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(ClassifyResult {
        total_reads: total,
        classified,
        unclassified: total - classified,
    })
}

fn majority_vote(taxids: &[u32]) -> u32 {
    let mut counts: HashMap<u32, u32> = HashMap::new();
    for &t in taxids {
        *counts.entry(t).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .map_or(0, |(t, _)| t)
}

pub fn load_kmer_db(path: &Path) -> Result<HashMap<u64, u32>> {
    let file = std::fs::File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let reader = std::io::BufReader::new(file);
    let mut db = HashMap::new();

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            let kmer: u64 = parts[0]
                .parse()
                .map_err(|e| RsomicsError::InvalidInput(format!("bad kmer hash: {e}")))?;
            let taxid: u32 = parts[1]
                .parse()
                .map_err(|e| RsomicsError::InvalidInput(format!("bad taxid: {e}")))?;
            db.insert(kmer, taxid);
        }
    }
    Ok(db)
}
