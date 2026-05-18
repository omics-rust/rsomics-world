# rsomics-infercnv

Infer copy-number variations from single-cell RNA-seq expression data.

## Origin

This crate is an independent Rust reimplementation of `inferCNV` based on:
- The published method: Patel et al., "inferCNV of the Trinity CTAT Project" (Broad Institute)
- The public algorithm description and documentation
- Black-box behavior testing against the upstream R package output

No source code from the upstream was used as reference during implementation.
Test fixtures are independently generated synthetic data.

License: MIT OR Apache-2.0.
Upstream credit: [inferCNV](https://github.com/broadinstitute/inferCNV) (BSD-3-Clause).

## Usage

```bash
rsomics-infercnv --matrix counts.tsv --gtf genes.gtf --ref-cells normals.txt -o cnv.tsv
```

### Inputs

- `--matrix`: Dense expression matrix (TSV). First column = gene names, first row = cell barcodes.
- `--gtf`: Gene annotation (GTF/GFF) for genomic ordering.
- `--ref-cells`: File listing reference (normal) cell barcodes, one per line (optional).
- `--window`: Sliding window size for smoothing (default: 101 genes).

### Algorithm

1. Parse gene positions from GTF and reorder expression matrix by genomic coordinate.
2. Log-normalize expression values: ln(x + 1).
3. Apply sliding-window mean smoothing along the genome axis per cell.
4. Subtract the mean signal of reference (normal) cells.
5. Output the residual CNV signal matrix.
