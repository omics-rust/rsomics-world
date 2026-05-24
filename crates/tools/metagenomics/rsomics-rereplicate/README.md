# rsomics-rereplicate

Expand `;size=N` abundance annotations back into N individual FASTA records.
The inverse of dereplication (`rsomics-derep` / `vsearch --derep_fulllength`).

## Usage

```
rsomics-rereplicate <INPUT> -o <OUTPUT> [OPTIONS]

Arguments:
  <INPUT>   Input FASTA file (use `-` for stdin)

Options:
  -o, --output <OUTPUT>       Output FASTA file (use `-` for stdout)
  --sizeout                   Append `;size=1` to each emitted copy
  --fasta-width <N>           Sequence line wrap width; 0 = no wrap [default: 80]
  -t, --threads <N>           Thread count (currently single-threaded; flag accepted)
  -q, --quiet                 Suppress progress messages
  --json                      Machine-readable JSON output on stderr
```

## Behaviour

Matches `vsearch --rereplicate` v2.31.0 byte-for-byte:

- Each record with `;size=N` emits N copies in input order.
- The `;size=N` annotation is **stripped** from output headers by default.
- With `--sizeout`, each copy receives `;size=1` appended.
- Records with no `;size=` annotation are treated as abundance 1 (warned on stderr).
- No `minseqlength` or `maxseqlength` filtering (vsearch does not apply these here).
- Sequence bytes preserved exactly — case and U characters unchanged.
- FASTA output wraps at 80 columns (configurable via `--fasta-width`).

## Performance

On 50 000 amplicons / 8.7 MB input (aarch64 macOS, Apple M2, hyperfine 15 runs):

| Tool | Mean | Ratio |
|------|------|-------|
| vsearch 2.31.0 | 141.3 ms ± 5.9 ms | 1.00× |
| rsomics-rereplicate 0.1.0 | 38.8 ms ± 0.7 ms | **3.65×** |

Output: 88 MB (50 500 reads, byte-exact vs vsearch).

## Origin

This crate is an independent Rust reimplementation of `vsearch --rereplicate`
based on:

- The vsearch source code (BSD-2-Clause) — `src/rereplicate.cc` and `src/fasta.cc`
  from <https://github.com/torognes/vsearch>
- Black-box behaviour testing against the upstream binary (vsearch 2.31.0)

The vsearch source is dual-licensed (GPL-3 or BSD-2-Clause).  We read the
BSD-2-Clause copy; our Rust implementation is MIT OR Apache-2.0.

License: MIT OR Apache-2.0  
Upstream credit: vsearch <https://github.com/torognes/vsearch> (BSD-2-Clause / GPL-3)
