# rsomics-plink-pca

PCA and GRM computation from PLINK1 binary filesets.

## Usage

```
rsomics-plink-pca --plink <prefix> [--out <prefix>] [--pcs <k>]
```

Reads `<prefix>.bed`, `<prefix>.bim`, `<prefix>.fam` and writes:
- `<out>.eigenvec` — tab-separated FID, IID, PC1, PC2, …
- `<out>.eigenval` — eigenvalues in descending order, one per line

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `-p` / `--plink` | — | Input PLINK1 prefix |
| `-o` / `--out` | `pca` | Output prefix |
| `-k` / `--pcs` | `10` | Number of principal components |

## Install

```bash
cargo install rsomics-plink-pca
```

## Algorithm

1. Build the standardized genotype matrix X (samples × variants):
   each variant column is centered at `2p` and scaled by `sqrt(2p(1-p))`.
   Missing genotypes are mean-imputed (PLINK2 default).

2. Compute the n×n Genetic Relationship Matrix (GRM): `G = (1/M) * X * X^T`.

3. Eigendecompose G using faer's self-adjoint EVD (LAPACK dsyevd-equivalent).
   Eigenvalues sorted in descending order; top-k retained.

4. Emit PC scores in PLINK2 `.eigenvec` format.

## Origin

This crate is an independent Rust reimplementation of PLINK2's `--pca`
operation based on:

- Purcell et al. 2007, AJHG (PLINK v1.07)
- Chang et al. 2015, GigaScience (PLINK2 GRM and randomized PCA approach)
- Public PLINK2 documentation and output format specification

No source code from the GPL-incompatible PLINK binaries was used as reference
during implementation. Test fixtures are independently generated synthetic data.

License: MIT OR Apache-2.0.
Upstream credit: PLINK2 (https://www.cog-genomics.org/plink/2.0/) (GPL-3).
