#![allow(clippy::cast_precision_loss)]

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use needletail::parse_fastx_file;
use rsomics_common::{Result, RsomicsError};

pub fn bed_nuc(bed_path: &Path, fasta_path: &Path, output: &mut dyn Write) -> Result<u64> {
    let seqs = load_fasta(fasta_path)?;

    let file = File::open(bed_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", bed_path.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);

    writeln!(out, "#chrom\tstart\tend\tA\tC\tG\tT\tN\tother\tlen\tGC_pct")
        .map_err(RsomicsError::Io)?;

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

        let bc = if let Some(seq) = seqs.get(chrom) {
            let s = start.min(seq.len());
            let e = end.min(seq.len());
            count_bases(&seq[s..e])
        } else {
            BaseCounts {
                adenine: 0,
                cytosine: 0,
                guanine: 0,
                thymine: 0,
                ambiguous: 0,
                other: 0,
            }
        };

        let len = bc.adenine + bc.cytosine + bc.guanine + bc.thymine + bc.ambiguous + bc.other;
        let gc_pct = if len > 0 {
            (bc.guanine + bc.cytosine) as f64 / len as f64 * 100.0
        } else {
            0.0
        };

        writeln!(
            out,
            "{chrom}\t{start}\t{end}\t{}\t{}\t{}\t{}\t{}\t{}\t{len}\t{gc_pct:.2}",
            bc.adenine, bc.cytosine, bc.guanine, bc.thymine, bc.ambiguous, bc.other
        )
        .map_err(RsomicsError::Io)?;
        count += 1;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}

struct BaseCounts {
    adenine: u64,
    cytosine: u64,
    guanine: u64,
    thymine: u64,
    ambiguous: u64,
    other: u64,
}

fn count_bases(seq: &[u8]) -> BaseCounts {
    let mut bc = BaseCounts {
        adenine: 0,
        cytosine: 0,
        guanine: 0,
        thymine: 0,
        ambiguous: 0,
        other: 0,
    };
    for &base in seq {
        match base.to_ascii_uppercase() {
            b'A' => bc.adenine += 1,
            b'C' => bc.cytosine += 1,
            b'G' => bc.guanine += 1,
            b'T' => bc.thymine += 1,
            b'N' => bc.ambiguous += 1,
            _ => bc.other += 1,
        }
    }
    bc
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
