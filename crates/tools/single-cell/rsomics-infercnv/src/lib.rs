#![allow(clippy::cast_precision_loss, clippy::needless_range_loop)]

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

/// Gene position annotation: chromosome, start, end, name.
#[derive(Debug, Clone)]
pub struct GenePos {
    pub chrom: String,
    pub start: u64,
    pub name: String,
}

/// Load gene ordering from a GTF/GFF — extracts gene-level features sorted by genomic position
pub fn load_gene_order(gtf_path: &Path) -> Result<Vec<GenePos>> {
    let file = File::open(gtf_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", gtf_path.display())))?;
    let reader = BufReader::new(file);
    let mut genes: Vec<GenePos> = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 9 || fields[2] != "gene" {
            continue;
        }
        let chrom = fields[0].to_string();
        let start: u64 = fields[3].parse().unwrap_or(0);
        let name = extract_attr(fields[8], "gene_name")
            .or_else(|| extract_attr(fields[8], "gene_id"))
            .unwrap_or_else(|| format!("{chrom}:{start}"));
        genes.push(GenePos { chrom, start, name });
    }

    genes.sort_by(|a, b| a.chrom.cmp(&b.chrom).then(a.start.cmp(&b.start)));
    Ok(genes)
}

fn extract_attr(attrs: &str, key: &str) -> Option<String> {
    for part in attrs.split(';') {
        let part = part.trim();
        let Some(rest) = part.strip_prefix(key) else {
            continue;
        };
        if rest.starts_with('=') || rest.starts_with(' ') {
            let val = rest[1..].trim().trim_matches('"');
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

type ExpressionMatrix = (Vec<String>, Vec<String>, Vec<Vec<f64>>);

/// Load a dense expression matrix (TSV: genes as rows, cells as columns).
/// First column = gene name, first row = cell barcodes.
pub fn load_matrix(path: &Path) -> Result<ExpressionMatrix> {
    let file = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let header = lines
        .next()
        .ok_or_else(|| RsomicsError::InvalidInput("empty matrix".into()))?
        .map_err(RsomicsError::Io)?;
    let cells: Vec<String> = header.split('\t').skip(1).map(String::from).collect();

    let mut genes: Vec<String> = Vec::new();
    let mut data: Vec<Vec<f64>> = Vec::new();

    for line in lines {
        let line = line.map_err(RsomicsError::Io)?;
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.is_empty() {
            continue;
        }
        genes.push(fields[0].to_string());
        let row: Vec<f64> = fields[1..]
            .iter()
            .map(|s| s.parse().unwrap_or(0.0))
            .collect();
        data.push(row);
    }

    Ok((genes, cells, data))
}

/// Core inferCNV algorithm:
/// 1. Reorder genes by genomic position
/// 2. Log-normalize expression
/// 3. Per-cell sliding-window smoothing
/// 4. Subtract reference (mean of normal cells)
/// 5. Output per-cell CNV signal matrix
pub fn infer_cnv(
    gene_names: &[String],
    cell_names: &[String],
    expr: &[Vec<f64>],
    gene_order: &[GenePos],
    ref_cell_indices: &[usize],
    window_size: usize,
) -> Result<(Vec<String>, Vec<Vec<f64>>)> {
    let n_genes = gene_names.len();
    let n_cells = cell_names.len();

    if n_genes == 0 || n_cells == 0 {
        return Err(RsomicsError::InvalidInput("empty matrix".into()));
    }

    // Map gene names to their genomic-order index
    let order_map: BTreeMap<&str, usize> = gene_order
        .iter()
        .enumerate()
        .map(|(i, g)| (g.name.as_str(), i))
        .collect();

    // Find genes present in both matrix and annotation
    let mut ordered_indices: Vec<(usize, usize)> = Vec::new(); // (matrix_idx, order_idx)
    for (mi, name) in gene_names.iter().enumerate() {
        if let Some(&oi) = order_map.get(name.as_str()) {
            ordered_indices.push((mi, oi));
        }
    }
    ordered_indices.sort_by_key(|&(_, oi)| oi);

    let n_ordered = ordered_indices.len();
    if n_ordered == 0 {
        return Err(RsomicsError::InvalidInput(
            "no genes matched between matrix and annotation".into(),
        ));
    }

    // Build reordered + log-normalized matrix (genes_ordered × cells)
    let mut norm: Vec<Vec<f64>> = vec![vec![0.0; n_cells]; n_ordered];
    for (gi, &(mi, _)) in ordered_indices.iter().enumerate() {
        for ci in 0..n_cells {
            let val = if mi < expr.len() && ci < expr[mi].len() {
                expr[mi][ci]
            } else {
                0.0
            };
            norm[gi][ci] = (val + 1.0).ln();
        }
    }

    // Sliding-window smoothing per cell
    let half = window_size / 2;
    let mut smoothed: Vec<Vec<f64>> = vec![vec![0.0; n_cells]; n_ordered];
    for ci in 0..n_cells {
        for gi in 0..n_ordered {
            let start = gi.saturating_sub(half);
            let end = (gi + half + 1).min(n_ordered);
            let sum: f64 = (start..end).map(|i| norm[i][ci]).sum();
            smoothed[gi][ci] = sum / (end - start) as f64;
        }
    }

    // Compute reference mean (from normal cells)
    let mut ref_mean: Vec<f64> = vec![0.0; n_ordered];
    if !ref_cell_indices.is_empty() {
        for gi in 0..n_ordered {
            let sum: f64 = ref_cell_indices.iter().map(|&ci| smoothed[gi][ci]).sum();
            ref_mean[gi] = sum / ref_cell_indices.len() as f64;
        }
    }

    // Subtract reference → CNV signal
    let mut cnv: Vec<Vec<f64>> = vec![vec![0.0; n_cells]; n_ordered];
    for gi in 0..n_ordered {
        for ci in 0..n_cells {
            cnv[gi][ci] = smoothed[gi][ci] - ref_mean[gi];
        }
    }

    let ordered_gene_names: Vec<String> = ordered_indices
        .iter()
        .map(|&(mi, _)| gene_names[mi].clone())
        .collect();

    Ok((ordered_gene_names, cnv))
}

/// Write CNV matrix as TSV
pub fn write_cnv(
    gene_names: &[String],
    cell_names: &[String],
    cnv: &[Vec<f64>],
    output: &mut dyn Write,
) -> Result<()> {
    let mut out = BufWriter::with_capacity(256 * 1024, output);

    // Header
    out.write_all(b"gene").map_err(RsomicsError::Io)?;
    for c in cell_names {
        write!(out, "\t{c}").map_err(RsomicsError::Io)?;
    }
    writeln!(out).map_err(RsomicsError::Io)?;

    // Data
    for (gi, gene) in gene_names.iter().enumerate() {
        out.write_all(gene.as_bytes()).map_err(RsomicsError::Io)?;
        if gi < cnv.len() {
            for &val in &cnv[gi] {
                write!(out, "\t{val:.4}").map_err(RsomicsError::Io)?;
            }
        }
        writeln!(out).map_err(RsomicsError::Io)?;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(())
}
