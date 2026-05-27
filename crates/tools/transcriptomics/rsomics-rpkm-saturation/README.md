# rsomics-rpkm-saturation

Subsample-based RPKM saturation analysis. Subsamples the aligned reads at
configurable fractions and computes per-gene RPKM at each fraction, producing
a saturation curve that reveals whether sequencing depth is sufficient to
detect all expressed genes.

## Usage

```
rsomics-rpkm-saturation -i input.bam -r genes.bed12 -o out_prefix [OPTIONS]
```

### Options

| Flag | Long | Default | Description |
|---|---|---|---|
| `-i` | `--input-file` | required | Input BAM (coordinate-sorted, indexed) |
| `-r` | `--refgene` | required | Reference gene model in BED12 format |
| `-o` | `--out-prefix` | required | Output file prefix |
| `-l` | `--percentile-floor` | 5 | Lowest sampling percentile |
| `-u` | `--percentile-ceiling` | 100 | Highest sampling percentile |
| `-s` | `--percentile-step` | 5 | Step between percentiles |
| | `--mapq` | 30 | Minimum mapping quality |
| `-t` | `--threads` | auto | Worker threads for BGZF decode |
| `--seed` | | random | RNG seed for deterministic sampling |

## Output files

- `<prefix>.eRPKM.xls` — Tab-separated RPKM per gene per fraction. Columns:
  `#chr`, `start`, `end`, `name`, `score`, `strand`, `5%`, `10%`, ..., `100%`.
- `<prefix>.rawCount.xls` — Same layout, raw read counts instead of RPKM.

## Algorithm

1. Load all primary, non-duplicate mapped reads passing `--mapq`.
2. Parse BED12 genes; expand each gene's exon blocks.
3. For each fraction `F` in `[lower..upper]` step `S`:
   - Sample ⌊F% × total_reads⌋ read indices without replacement.
   - Assign each sampled read to all genes whose exons it overlaps.
   - Compute RPKM = raw_count / (gene_length_kb × total_mapped_millions).
4. Write `.eRPKM.xls` and `.rawCount.xls`.

Fractions are processed in parallel via rayon. A deterministic RNG
(ChaCha12) seeded per-fraction from `--seed` ensures reproducible output.

## Determinism vs RSeQC

RSeQC's `RPKM_saturation.py` uses Python's `random` module without a fixed
seed, so its output is non-deterministic across runs. This tool uses a
seeded ChaCha12 RNG and is fully deterministic given the same `--seed`.
Compat tests compare: (a) gene order and BED6 prefix columns exactly;
(b) RPKM at the 100% fraction within 1% relative tolerance; (c) raw counts
at 100% exactly. At sub-100% fractions, structural sanity is verified.

## Origin

This crate is an independent Rust reimplementation of `RPKM_saturation.py`
(part of RSeQC) based on:

- The RSeQC documentation: <https://rseqc.sourceforge.net/#rpkm-saturation-py>
- The public BED12 file-format specification
- Black-box behaviour testing against the upstream binary

No source code from the GPL upstream was used as reference during
implementation. Test fixtures are independently generated from publicly
available RNA-seq data (HG002 / 1000 Genomes project subsets).

License: MIT OR Apache-2.0  
Upstream credit: RSeQC <https://github.com/MonashBioinformatics/RSeQC> (GPL-2.0)
