# rsomics-bam-bedcov

Per-BED-region read depth — for each region in a BED file, the total number
of read bases overlapping it, summed across one or more BAM inputs. Rust
reimplementation of `samtools bedcov`.

## Install

```
cargo install rsomics-bam-bedcov
```

Single binary. Each input BAM must be coordinate-sorted and indexed
(`samtools index`).

## Usage

```
rsomics-bam-bedcov regions.bed aln.bam
rsomics-bam-bedcov -Q 20 panel.bed sample1.bam sample2.bam
```

One output line per BED region, the original BED columns passed through
verbatim, then one appended coverage column per input BAM — exactly the
`samtools bedcov` output, in input BED order. `-Q/--min-mq` sets the minimum
mapping quality (default 0).

## Strategy

bedcov is two workloads in one. The dominant case — capture panels, exomes,
genome-wide windows — is tens of thousands to hundreds of thousands of
regions; the sparse case is a handful of spot-check regions. They want
opposite I/O shapes, so the tool picks per call:

- **Linear sweep** (many regions). The BAM is read exactly once: each BGZF
  block is inflated a single time, records are read through the
  `rsomics-bamio` raw path (refID/pos/CIGAR at fixed offsets, no full record
  decode), and a moving cursor over the start-sorted regions attributes each
  read's reference span to every region it overlaps — O(reads + overlap-hits).
  Results are emitted in input BED order.

- **Indexed query** (few regions). Each region is served from the BAM index
  with a direct BGZF seek, decoding only the reads that overlap it — cheaper
  than reading the whole file when there are few regions.

The crossover is file-size-aware: the sweep's cost is `file_bytes / inflate_rate`
(flat in region count), the indexed path's is `n_regions × per_region_seek`,
so they meet at `n_regions ≈ file_bytes / 384 KiB`, with an absolute floor of
256 regions below which a genuinely sparse query never inflates the whole BAM.
On an Apple-M2 / 170 MB coord-sorted BAM the sweep beats `samtools bedcov`
from roughly 8k regions upward, widening to ~6× at 50k regions.

## Coverage semantics

Matches `samtools bedcov` with no `-j`: each read contributes, to every BED
region it overlaps, the length of the intersection of its reference span
`[start, start+span)` with the region `[beg, end)`. The reference span counts
CIGAR M/D/N/=/X — deletion (D) and ref-skip (N) positions count as covered,
because the default `samtools bedcov` pileup includes them (only `-j` excludes
them). Output is byte-identical to `samtools bedcov` on sorted and unsorted
BEDs alike, verified by the compat tests over both the sweep and indexed paths.

## Origin

Independent Rust reimplementation of `samtools bedcov` based on:

- The published method: Li H et al., *The Sequence Alignment/Map format and
  SAMtools*, Bioinformatics 25:2078 (2009),
  [doi:10.1093/bioinformatics/btp352].
- The SAM/BAM specification v1.6 (record layout, CIGAR semantics) and the BED
  format (0-based half-open intervals).
- Black-box behaviour testing against `samtools bedcov` (v1.23.1).

samtools is MIT-licensed; its `bedcov.c` was consulted for the exact coverage
semantics (the default-vs-`-j` deletion/ref-skip handling, the `0x704` default
flag filter, the per-base pileup count, multi-BAM column ordering).

License: MIT OR Apache-2.0.
Upstream credit: [samtools](https://github.com/samtools/samtools) (MIT).

### External-dep quadrant classification

- `noodles-bgzf` with the `libdeflate` feature — **Quadrant ②** (FFI wrapper
  over bundled-C libdeflate). This is the same inflate `samtools` uses; it
  switches every BGZF block (sweep read path and indexed-query path alike) off
  the pure-Rust zlib-rs onto libdeflate, which wins single-threaded. Portable
  bundled C, no system library. A future pass may pure-Rust replace it.
- `noodles` (noodles-bam, noodles-sam, noodles-core) — **Quadrant ①** (pure
  Rust, spec-tracking SAM/BAM library). Used for the header and the indexed
  CSI query.
- `rsomics-bamio` — Layer-A shared BAM raw-record read path (pure Rust over
  the libdeflate BGZF reader; the sweep's hot loop).
- `rsomics-common`, `rsomics-help`, `clap`, `serde`, `serde_json`, `itoa` —
  **Quadrant ④** (edge utilities).

No Quadrant ③ (single-threaded-in-hot-path) dependencies.

## Performance

The release contract: strictly faster wall-clock than `samtools bedcov` on the
representative many-region fixture. Measured on Apple M2, `samtools 1.23.1`
(libdeflate), 50k regions over a 170 MB coord-sorted BAM: ours ~0.67 s vs
samtools ~4.46 s → ~6×. For sparse region sets (tens of regions) both finish
in well under 100 ms; the adaptive heuristic keeps the tool from inflating a
whole BAM for a few windows.

[doi:10.1093/bioinformatics/btp352]: https://doi.org/10.1093/bioinformatics/btp352
