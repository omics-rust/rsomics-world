# Compression

> Codecs and CLI tools for the squashed bytes underneath every bioinformatics
> file.

## Scope

Block- and stream-level compression: DEFLATE/gzip, BGZF, zstd, lz4, xz, and
the user-facing CLIs (`bgzip`, `pigz`, `crabz`) that wrap them. Excludes
*record-level* file formats (those live in
[`io-formats.md`](io-formats.md)) and random-access indexes built on top of
BGZF (those live in [`indexing.md`](indexing.md)).

## Design notes

- BGZF is the workhorse beneath BAM, compressed VCF and BCF, and tabix-indexed
  text. CRAM has its own container and codec model. Throughput on the BGZF
  paths translates directly to pipeline wall time.
- Two implementation strategies coexist: pure-Rust DEFLATE via
  [`flate2`](https://github.com/rust-lang/flate2-rs) (with `miniz_oxide`
  or `zlib-ng` backends) and FFI to `libdeflate` via `libdeflater`. For
  block-sized inputs, libdeflate is ~2× faster — `gzp` and `crabz` use it
  by default.
- Multi-threaded compression is where Rust beats single-threaded `gzip` and
  matches `pigz`: see [`gzp`](https://github.com/sstadick/gzp) and
  [`crabz`](https://github.com/sstadick/crabz).
- zstd is a serious contender for *new* file formats (CRAM 3.1 uses it
  internally) but the existing bioinformatics ecosystem is overwhelmingly
  gzip/BGZF, so any zstd-only output needs a fallback path.
- xz/LZMA shows up only in archival contexts (e.g. SRA fasterq-dump
  outputs) and is not worth optimising.

## TODO

- [x] **`flate2`** — DEFLATE/gzip/zlib codec for Rust.
  - Reference impl: `C` · [madler/zlib](https://github.com/madler/zlib) · `Zlib`
  - Existing Rust: [`flate2`](https://github.com/rust-lang/flate2-rs) `1.1.9` (pluggable: `miniz_oxide` pure-Rust default, `zlib-ng` FFI, `cloudflare-zlib`); supplementary [`libflate`](https://github.com/sile/libflate) `2.3.0` (pure-Rust)
  - Existing Rust kind: `pure-port/FFI-wrapper`
  - Existing non-C alternatives: —
  - Parallelism: single-threaded codec (wrap with `gzp` for parallel)
  - SIMD: inherited via backend dep (`miniz_oxide` `simd` feature; or `zlib-ng` hand SIMD when that backend is selected). flate2 itself contains no SIMD.
  - Quadrant: ①+② (① with default `miniz_oxide` + `simd` feature; ② when `zlib-ng` backend selected)
  - GPU-amenable: no — DEFLATE is inherently bit-serial
  - Upstream license: `Zlib`
  - Priority: `P0`
  - Layer: `adopt`
  - Consumes primitives: —
  - Notes: Default `miniz_oxide` is portable and safe; switch to `zlib-ng` when throughput matters and ship a feature flag.

- [x] **`libdeflate`** — block-oriented DEFLATE optimised for known-size inputs.
  - Reference impl: `C` · [ebiggers/libdeflate](https://github.com/ebiggers/libdeflate) · `MIT`
  - Existing Rust: [`libdeflater`](https://github.com/adamkewley/libdeflater) `1.25.2` (safe wrapper); [`libdeflate-sys`](https://crates.io/crates/libdeflate-sys) `1.25.2` (raw bindings)
  - Existing Rust kind: `FFI-wrapper`
  - Existing non-C alternatives: —
  - Parallelism: single-threaded codec (caller schedules parallel blocks)
  - SIMD: inherits libdeflate's hand-written CRC32 / vector intrinsics
  - Quadrant: ②
  - GPU-amenable: no — bit-serial DEFLATE
  - Upstream license: `MIT`
  - Priority: `P0`
  - Layer: `adopt`
  - Consumes primitives: —
  - Notes: FFI-only today. A pure-Rust libdeflate-equivalent (block-DEFLATE with `std::simd` / `target_feature` vector intrinsics) is a real opportunity but a multi-month project. SIMD-critical inner loops.

- [x] **`BGZF`** — Blocked GNU Zip Format used by samtools/htslib.
  - Reference impl: `C` · [samtools/htslib/bgzf.c](https://github.com/samtools/htslib/blob/develop/bgzf.c) · `MIT`
  - Existing Rust: [`noodles-bgzf`](https://crates.io/crates/noodles-bgzf) `0.47.0` (pure-Rust); [`bgzip`](https://github.com/informationsea/bgzip-rs) `0.3.1`; [`gzp`](https://github.com/sstadick/gzp) `2.0.2` (multithreaded write path)
  - Existing Rust kind: `pure-port/partial-port`
  - Existing non-C alternatives: —
  - Parallelism: noodles-bgzf reader is single-threaded; gzp uses `flume` channels for parallel write
  - SIMD: inherits codec dep (`miniz_oxide` SIMD or `libdeflate` SIMD depending on backend)
  - Quadrant: ① (noodles-bgzf) / ①+② (gzp with default libdeflater backend)
  - GPU-amenable: no — block layout is fixed and DEFLATE is bit-serial
  - Upstream license: `MIT`
  - Priority: `P0`
  - Layer: `adopt`; any shared raw-frame mechanics belong in `rsomics-seqio`
  - Consumes primitives: —
  - Notes: Adopt `noodles-bgzf` for IO correctness; pair with `gzp` / `libdeflater` for multithreaded *write* paths. `rsomics-bam` and `rsomics-vcf` currently keep raw-frame rewriting private. Only their demonstrated common, format-neutral contract may move into `rsomics-seqio`, with structural framing kept distinct from optional decompression and CRC validation. Parallel decoding remains an upstream or measured consumer-driven project, not a separate crate (tracking [zaeleus/noodles#17](https://github.com/zaeleus/noodles/issues/17)).

- [x] **`bgzip` (CLI)** — samtools companion for creating BGZF files.
  - Reference impl: `C` · [samtools/htslib/bgzip.c](https://github.com/samtools/htslib) · `MIT`
  - Existing Rust: [`crabz`](https://github.com/sstadick/crabz) `0.10.0` (pigz-style multithreaded gzip/BGZF CLI built on `gzp`)
  - Existing Rust kind: `rust-native` (crabz is a Rust-native pigz-style CLI, not a code-port of samtools/bgzip)
  - Existing non-C alternatives: `bgzip` ships as part of htslib
  - Parallelism: rayon-equivalent worker pool via `gzp` (`flume` + N workers)
  - SIMD: inherits libdeflate SIMD (default backend) or miniz_oxide SIMD
  - Quadrant: ①+② (Rust-native scheduling + FFI codec backend by default)
  - GPU-amenable: no — bit-serial DEFLATE
  - Upstream license: `MIT`
  - Priority: `P1`
  - Layer: `rsomics-index bgzip`
  - Consumes primitives: `gzp`, `libdeflater`
  - Notes: The BGZF CLI and its `.gzi` lifecycle belong to the accepted indexing workflow. `crabz` is an implementation candidate because it handles BGZF block layout through `gzp::deflate::Bgzf`, but compatibility and representative performance must be demonstrated inside `rsomics-index`; no generic compression product is implied. See also [`indexing.md`](indexing.md) `.gzi` entry.

- [x] **`pigz`** — parallel gzip CLI.
  - Reference impl: `C` · [madler/pigz](https://github.com/madler/pigz) · `Zlib`
  - Existing Rust: [`crabz`](https://github.com/sstadick/crabz) `0.10.0`; library is [`gzp`](https://crates.io/crates/gzp) `2.0.2`
  - Existing Rust kind: `rust-native` (crabz is independent Rust-native; takes the pigz approach but does not port pigz's code)
  - Existing non-C alternatives: —
  - Parallelism: parallel block compression via `gzp`
  - SIMD: inherits codec backend
  - Quadrant: ①+② (Rust-native scheduler + FFI codec backend by default)
  - GPU-amenable: no — bit-serial DEFLATE
  - Upstream license: `Zlib`
  - Priority: `P1`
  - Layer: `adopt`; excluded from the current product allowlist
  - Consumes primitives: `gzp`, `libdeflater` / `flate2`
  - Notes: `crabz` is the closest Rust equivalent. General-purpose parallel gzip is already served by `pigz` and `crabz` and does not justify an rsomics product. Product-specific parallel compression remains behind the owning product or `rsomics-seqio` contract when two consumers demonstrate it.

- [x] **`zstd`** — Facebook's zstandard codec.
  - Reference impl: `C` · [facebook/zstd](https://github.com/facebook/zstd) · `BSD-3-Clause OR GPL-2.0`
  - Existing Rust: [`zstd`](https://github.com/gyscos/zstd-rs) `0.13.3` (FFI, multi-threaded encoder); [`ruzstd`](https://github.com/KillingSpark/zstd-rs) `0.8.3` (pure-Rust decoder only)
  - Existing Rust kind: `FFI-wrapper/partial-port`
  - Existing non-C alternatives: —
  - Parallelism: zstd-rs exposes the upstream's multi-threaded encoder
  - SIMD: inherits zstd's hand-written SIMD
  - Quadrant: ② (production path); ① for `ruzstd` decoder
  - GPU-amenable: no — codec is bit-serial
  - Upstream license: `BSD-3-Clause OR GPL-2.0` (user picks); typically `BSD-3-Clause` in our adoption
  - Priority: `P1`
  - Layer: `adopt`
  - Consumes primitives: —
  - Notes: Use FFI `zstd` for production (much faster, multi-threaded encoder). `ruzstd` is decoder-only and lags upstream; not yet a drop-in replacement.

- [x] **`lz4`** — fast streaming compressor.
  - Reference impl: `C` · [lz4/lz4](https://github.com/lz4/lz4) · `BSD-2-Clause`
  - Existing Rust: [`lz4_flex`](https://github.com/PSeitz/lz4_flex) `0.13.1` (pure-Rust); [`lz4-sys`](https://crates.io/crates/lz4-sys) `1.11.1+lz4-1.10.0` (FFI)
  - Existing Rust kind: `pure-port/FFI-wrapper`
  - Existing non-C alternatives: —
  - Parallelism: single-threaded codec; caller schedules
  - SIMD: lz4_flex relies on auto-vectorize; lz4-sys inherits upstream's hand SIMD
  - Quadrant: ① (lz4_flex) / ② (lz4-sys)
  - GPU-amenable: no — codec is byte-serial
  - Upstream license: `BSD-2-Clause`
  - Priority: `P2`
  - Layer: `adopt`
  - Consumes primitives: —
  - Notes: `lz4_flex` is within ~10% of the C library, `no unsafe by default`. Used mostly for intermediate scratch files; rarely a user-facing format in genomics.

- [~] **`xz` / `liblzma`** — high-ratio LZMA codec.
  - Reference impl: `C` · [tukaani-project/xz](https://github.com/tukaani-project/xz) · `0BSD OR LGPL-2.1`
  - Existing Rust: [`xz2`](https://github.com/alexcrichton/xz2-rs) `0.1.7` (FFI); [`lzma-rs`](https://github.com/gendx/lzma-rs) `0.3.0` (pure-Rust, partial)
  - Existing Rust kind: `FFI-wrapper/partial-port`
  - Existing non-C alternatives: —
  - Parallelism: single-threaded
  - SIMD: none
  - Quadrant: ② (xz2) / ③ (lzma-rs)
  - GPU-amenable: no — LZMA is bit-serial
  - Upstream license: `0BSD` (xz core); some legacy parts `LGPL-2.1`
  - Priority: `P2`
  - Layer: `adopt`
  - Consumes primitives: —
  - Notes: FFI is fine for archival ingest. Only relevant for SRA archive ingest and some legacy CRAM. Not a focus.

- [x] **`niffler`** — format-sniffing reader (auto-detect gzip/bgzf/zstd/xz).
  - Reference impl: — (Rust-native concept; analogous to Python `xopen`)
  - Existing Rust: [`niffler`](https://github.com/luizirber/niffler) `3.0.1`; supplementary [`zopen`](https://crates.io/crates/zopen) `1.0.1` (hosted on chiselapp, GitHub aliveness unverifiable)
  - Existing Rust kind: `rust-native`
  - Existing non-C alternatives: `xopen` (Python)
  - Parallelism: single-threaded sniffer; the underlying codec defines parallelism
  - SIMD: inherits codec dep
  - Quadrant: ④ at the sniffing layer; the actual codec quadrant flows through (e.g. ② when xz is detected)
  - GPU-amenable: no — file-header lookup, microsecond-class work
  - Upstream license: `MIT`
  - Priority: `P1`
  - Layer: `adopt`
  - Consumes primitives: `flate2`, `zstd`, `xz2` (all transitively)
  - Notes: Adopt `niffler` as the default open-by-extension helper in CLI tools. Eliminates a class of "forgot to gunzip" user errors.
