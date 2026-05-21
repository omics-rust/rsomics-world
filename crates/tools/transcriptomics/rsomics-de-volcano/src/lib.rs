use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub fn annotate_de(
    input: &Path,
    pval_col: &str,
    lfc_col: &str,
    pval_thresh: f64,
    lfc_thresh: f64,
    output: &mut dyn Write,
) -> Result<(u64, u64, u64)> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::new(output);
    let mut lines = reader.lines();

    let header = lines
        .next()
        .ok_or_else(|| RsomicsError::InvalidInput("empty".into()))?
        .map_err(RsomicsError::Io)?;
    let cols: Vec<&str> = header.split('\t').collect();
    let pi = cols
        .iter()
        .position(|c| *c == pval_col)
        .ok_or_else(|| RsomicsError::InvalidInput(format!("column '{pval_col}' not found")))?;
    let li = cols
        .iter()
        .position(|c| *c == lfc_col)
        .ok_or_else(|| RsomicsError::InvalidInput(format!("column '{lfc_col}' not found")))?;

    writeln!(out, "{header}\tcategory").map_err(RsomicsError::Io)?;
    let (mut up, mut down, mut ns) = (0u64, 0u64, 0u64);

    for line in lines {
        let line = line.map_err(RsomicsError::Io)?;
        let fields: Vec<&str> = line.split('\t').collect();
        let pval: f64 = fields.get(pi).and_then(|s| s.parse().ok()).unwrap_or(1.0);
        let lfc: f64 = fields.get(li).and_then(|s| s.parse().ok()).unwrap_or(0.0);

        let cat = if pval <= pval_thresh && lfc >= lfc_thresh {
            up += 1;
            "UP"
        } else if pval <= pval_thresh && lfc <= -lfc_thresh {
            down += 1;
            "DOWN"
        } else {
            ns += 1;
            "NS"
        };
        writeln!(out, "{line}\t{cat}").map_err(RsomicsError::Io)?;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok((up, down, ns))
}
