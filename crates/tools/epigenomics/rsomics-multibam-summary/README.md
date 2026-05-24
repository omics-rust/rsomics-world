# rsomics-multibam-summary

Multi-BAM read-count **matrix** — a Rust port of deeptools `multiBamSummary`.

Produces, across several BAM files sharing one reference, a matrix of read
counts (rows = genome bins or BED regions, columns = BAM samples). This matrix
is the input to `plotCorrelation` / `plotPCA` for ChIP/ATAC sample-correlation
and clustering QC.

## Usage

```sh
# bins mode — fixed-width genome bins (deeptools default 10 kb)
rsomics-multibam-summary -b a.bam b.bam c.bam -o counts.tab --bin-size 10000

# BED-file mode — count per supplied region (e.g. called peaks)
rsomics-multibam-summary -b a.bam b.bam --bed peaks.bed -o counts.tab
```

`-o/--out-raw-counts` is the value-exact, human-readable matrix
(deeptools `--outRawCounts`). `-` writes to stdout.

## Modes

- **bins** (default): tile each chromosome into `--bin-size` bins and count
  reads per bin per BAM. Bins span `[i*binSize, (i+1)*binSize)`; the partial
  last bin per chromosome is retained (`n_bins = ceil(chrom_len / binSize)`,
  matching deeptools' `(end-start)//tile + (1 if remainder)`). A read
  contributes +1 to every bin its reference span overlaps. The per-BAM binning
  reuses the Layer-A `rsomics-coverage-core` primitive shared with
  `bamCoverage` / `bamCompare`.
- **BED-file** (`--bed`): count reads per supplied BED region per BAM, one row
  per BED line. A read contributes +1 to every region its reference span overlaps
  (deeptools collapses a read's per-block increments to a single +1 per region).
  Rows are emitted sorted by chromosome (BAM-header order) then ascending
  position, matching deeptools — not BED declaration order.

### Read filter (deeptools defaults)

deeptools' defaults keep almost everything: `minMappingQuality=None`,
`samFlag_exclude=None`, `ignoreDuplicates=False`. Only **unmapped** reads are
dropped — secondary, supplementary and duplicate reads are counted. Override
with `--min-mapq` and `--skip-flags` (e.g. `--skip-flags 0x400` to drop
duplicates).

## `--outRawCounts` format

Header line with single-quoted column names, then plain tab-separated data
rows; counts print as deeptools' float64 (`5.0`, `0.0`):

```
#'chr'	'start'	'end'	'a.bam'	'b.bam'
chr1	0	10000	4.0	2.0
chr1	10000	20000	1.0	0.0
```

Column labels default to each BAM's basename, matching deeptools
`[os.path.basename(x) for x in args.bamfiles]`.

## Scope

The numpy `.npz` matrix that deeptools also emits is **out of scope**: it is an
opaque archive consumed only by the downstream `plotCorrelation` / `plotPCA`
tools, whereas `--outRawCounts` is the value-exact, tool-agnostic oracle. This
crate emits only `--outRawCounts`.

All BAMs must share the first BAM's reference sequence set and lengths (the
common-reference case multiBamSummary targets); a mismatch fails loud. deeptools'
`--region`, `--smartLabels`, fragment extension and blacklist options are not
implemented.

## Origin

This crate is an independent Rust reimplementation of deeptools
`multiBamSummary` based on:

- The published method (Ramírez et al., *deepTools2*, Nucleic Acids Research
  2016, DOI: 10.1093/nar/gkw257)
- The public SAM/BAM and BED file-format specs
- Reading the upstream MIT-licensed source (`multiBamSummary.py`,
  `countReadsPerBin.py`) to match exact binning, counting and `--outRawCounts`
  formatting semantics
- Byte-exact behaviour testing against the upstream binary (`tests/compat.rs`,
  verified against deeptools 3.5.6 for both `bins` and `BED-file` modes)

deeptools is MIT-licensed, so reading and citing its source is permitted.

License: MIT OR Apache-2.0.
Upstream credit: deeptools <https://github.com/deeptools/deepTools> (MIT).
