use std::collections::HashMap;
use std::io::{BufRead, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub struct ClassifyResult {
    pub total_reads: u64,
    pub classified: u64,
    pub unclassified: u64,
}

#[allow(clippy::implicit_hasher)]
pub fn classify_reads(
    reads: &Path,
    db: &HashMap<u64, u32>,
    k: usize,
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

        let mut tax_hits: HashMap<u32, u32> = HashMap::new();
        for kmer in iter.flatten() {
            if let Some(&taxid) = db.get(&kmer) {
                *tax_hits.entry(taxid).or_insert(0) += 1;
            }
        }

        if let Some((best_tax, best_count)) = tax_hits.iter().max_by_key(|(_, v)| *v) {
            writeln!(out, "C\t{name}\t{best_tax}\t{best_count}").map_err(RsomicsError::Io)?;
            classified += 1;
        } else {
            writeln!(out, "U\t{name}\t0\t0").map_err(RsomicsError::Io)?;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(ClassifyResult {
        total_reads: total,
        classified,
        unclassified: total - classified,
    })
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
