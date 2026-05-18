# rsomics-featurecounts

Count reads overlapping genomic features from BAM + GFF annotation.

## Origin

This crate is an independent Rust reimplementation of `featureCounts` based on:
- The published method: Liao et al., "featureCounts: an efficient general purpose
  program for assigning sequence reads to genomic features", Bioinformatics 2014
  (DOI: 10.1093/bioinformatics/btt656)
- The public algorithm description and documentation
- Black-box behavior testing against the upstream binary

No source code from the GPL upstream was used as reference during implementation.
Test fixtures are independently generated synthetic data.

License: MIT OR Apache-2.0.
Upstream credit: [featureCounts (Subread)](http://subread.sourceforge.net/) (GPL-3.0).

## Usage

```bash
rsomics-featurecounts -a genes.gtf input.bam -o counts.tsv
```

### Algorithm

1. Parse GFF/GTF annotation, extract features matching `--feature-type` (default: exon).
2. Group features by `--attribute` (default: gene_id).
3. For each mapped BAM record, find overlapping features by coordinate intersection.
4. Assign read to gene: unique overlap → count, multiple genes → ambiguous, no overlap → unassigned.
5. Output gene × count matrix.
