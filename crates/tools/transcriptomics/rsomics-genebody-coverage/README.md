# rsomics-genebody-coverage

Gene-body 5'→3' read-coverage profile for RNA-seq degradation / 3'-bias QC.

Rust port of `RSeQC` `geneBody_coverage.py`.

## Usage

```
rsomics-genebody-coverage -i sample.bam -r genes.bed12 -o prefix
```

Produces `prefix.geneBodyCoverage.txt` — a 2-row TSV identical in format
to `RSeQC`, suitable for downstream visualisation with the companion `.r` file
or any plotting tool.

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `-i` / `--input` | required | Sorted, indexed BAM file(s); comma-separated for multiple |
| `-r` / `--refgene` | required | Reference gene model in BED12 format |
| `-l` / `--minimum-length` | 100 | Skip transcripts with mRNA length < this value |
| `-o` / `--out-prefix` | required | Prefix for output file |
| `-t` / `--threads` | auto | Number of BGZF decode threads (passed to bamio) |

## Output format

```
Percentile	1	2	…	100
sample_name	<cov1>	<cov2>	…	<cov100>
```

Coverage values are integers summed across all qualifying transcripts.

## Algorithm

For each transcript in the BED12 whose total exon length ≥ `min_mrna_len`:

1. Build the ordered list of 1-based genomic positions for all exon bases.
2. Sample 100 percentile positions using linear interpolation with
   banker's rounding (matching `mystat.percentile_list` in RSeQC exactly).
3. Query the indexed BAM over `[positions[0]-1, positions[99]]`.
4. For each read passing the filter (not QC-fail/duplicate/secondary/unmapped),
   walk the CIGAR and increment coverage at any percentile position the read
   covers (skipping deletion/intron-skip ops, matching pysam `is_del`).
5. For minus-strand transcripts, reverse the 100-element vector before
   accumulation (so index 0 is always the 5' end).
6. Accumulate across all transcripts.

## Origin

This crate is an independent Rust reimplementation based on:
- `RSeQC`: `geneBody_coverage.py` (LGPL-2.1+), Wang et al. 2012
  <https://doi.org/10.1093/bioinformatics/bts356>
- The SAM/BAM format specification (MIT)
- BED12 format specification
- Black-box behaviour testing against `RSeQC` 5.0.4

No source code from the LGPL upstream was used as reference beyond
the algorithm description and format specification. Test fixtures are
independently generated.

License: MIT OR Apache-2.0.
Upstream credit: `RSeQC` <https://rseqc.sourceforge.net/> (LGPL-2.1+).
