use needletail::parse_fastx_file;
use rsomics_common::{Result, RsomicsError};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn bed_to_fasta(bed_path: &Path, fasta_path: &Path, output: &mut dyn Write) -> Result<u64> {
    let seqs = load_fasta(fasta_path)?;
    let file = File::open(bed_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", bed_path.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(256 * 1024, output);
    let mut count: u64 = 0;
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 3 {
            continue;
        }
        let chrom = fields[0];
        let start: usize = fields[1].parse().unwrap_or(0);
        let end: usize = fields[2].parse().unwrap_or(0);
        let name = fields.get(3).unwrap_or(&".");
        if let Some(seq) = seqs.get(chrom) {
            let s = start.min(seq.len());
            let e = end.min(seq.len());
            writeln!(out, ">{chrom}:{start}-{end}_{name}").map_err(RsomicsError::Io)?;
            out.write_all(&seq[s..e]).map_err(RsomicsError::Io)?;
            writeln!(out).map_err(RsomicsError::Io)?;
            count += 1;
        }
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
fn load_fasta(path: &Path) -> Result<HashMap<String, Vec<u8>>> {
    if std::fs::metadata(path).is_ok_and(|m| m.len() == 0) {
        return Err(RsomicsError::InvalidInput("empty FASTA".into()));
    }
    let mut reader = parse_fastx_file(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let mut seqs = HashMap::new();
    while let Some(record) = reader.next() {
        let record = record.map_err(|e| RsomicsError::InvalidInput(format!("reading: {e}")))?;
        let id = std::str::from_utf8(record.id())
            .unwrap_or("unknown")
            .to_string();
        seqs.insert(id, record.seq().to_vec());
    }
    Ok(seqs)
}
