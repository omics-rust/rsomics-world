# rsomics-coverage-core

Genome-binned BAM read-coverage primitive. Layer A library shared by
`rsomics-bam-signal` (deeptools `bamCoverage`) and `rsomics-bam-compare`
(deeptools `bamCompare`).

Tiles each chromosome into fixed-width bins and counts, per bin, the reads whose
reference span (alignment start + reference-consuming CIGAR ops) overlaps it.
Returns per-chromosome bin counts plus two read totals (`total_binned` for
bamCoverage CPM/RPKM/RPGC, `total_mapped` for bamCompare's readCount scale
factor).

This crate holds only the binning + counting. Normalisation, the bedGraph
run-length emit, and two-BAM combination live in the consuming tool crates.

## API

```rust
use std::num::NonZero;
use std::path::Path;
use rsomics_coverage_core::{compute_coverage, BinFilter};

let cov = compute_coverage(
    Path::new("in.bam"),
    50,                        // bin size (deeptools default)
    BinFilter::default(),      // accept all mapped reads
    NonZero::new(1).unwrap(),  // BGZF inflate workers
)?;
// cov.chroms: Vec<ChromBins { name, chrom_len, bins: Vec<u32> }>
// cov.total_binned, cov.total_mapped, cov.total_signal()
```

## Origin

This crate is a Rust reimplementation of the per-bin read-counting core of
`deeptools` (`countReadsPerBin.py`), informed by the deeptools source (MIT
license):

- Ramírez et al., *deepTools2: a next generation web server for deep-sequencing
  data analysis*, NAR 2016. DOI: 10.1093/nar/gkw257
- deeptools source: `countReadsPerBin.py`, `bamCoverage.py`, `bamCompare.py`

deeptools is MIT licensed. Its source was read directly and cited here per the
CONVENTIONS clean-room methodology for MIT upstreams.

License: MIT OR Apache-2.0.
Upstream credit: deeptools <https://github.com/deeptools/deeptools> (MIT).
