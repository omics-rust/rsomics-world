# rsomics-bbi

Pure-Rust bigWig/BBI reader. Layer A library: opens a bigWig, enumerates its
chromosomes, and answers per-base value queries over an interval with
pyBigWig-compatible NaN semantics (NaN where the file carries no data).

Shared by deeptools-port tool crates that read bigWig signal —
`rsomics-compute-matrix` (`computeMatrix`) and `rsomics-bigwig-compare`
(`bigwigCompare`).

The reader covers the BBI on-disk layout: the 64-byte header, the chromosome
B-tree, the R-tree ("cir") interval index, and the three data-section flavours
(bedGraph, varStep, fixedStep). Block inflation uses `flate2` with the zlib-rs
backend (pure Rust).

## API

```rust
use std::path::Path;
use rsomics_bbi::BigWig;

let mut bw = BigWig::open(Path::new("signal.bw"))?;

// chromosome enumeration for genome-wide tiling
for (name, len) in bw.chroms() {
    println!("{name}\t{len}");
}
let len = bw.chrom_len("chr1");          // Option<u32>

// per-base values; NaN where the file has no data, None if chrom absent
let vals = bw.values("chr1", 1000, 2000)?; // Option<Vec<f32>>
```

## Origin

The bigWig/BBI on-disk layout (header at offset 0, chromosome B-tree, R-tree
"cir" index, zlib-compressed data sections in bedGraph/varStep/fixedStep
flavours) was read from the bigtools 0.5.6 source (`src/bbi/bbiread.rs`,
`src/bbi/bigwigread.rs`, MIT, Jack Huey) and Jim Kent's published BBI format.

We carry our own reader rather than depend on bigtools because bigtools pins
`libdeflater = "0.13"` whose `libdeflate-sys` (C FFI, `links = "libdeflate"`)
collides with the workspace's `rsomics-fqgz` (`libdeflater = "1"`); cargo
forbids two crates linking the same native library. Block inflation here uses
the workspace `flate2` zlib-rs backend (pure Rust), so this crate is
Quadrant ① (pure Rust + explicit, simple decode).

License: MIT OR Apache-2.0.
Upstream credit: [bigtools](https://github.com/jackh726/bigtools) (MIT),
UCSC bigWig/BBI format (Jim Kent).
