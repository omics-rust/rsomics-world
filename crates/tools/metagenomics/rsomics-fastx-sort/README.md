# rsomics-fastx-sort

Deterministic FASTA sorting by abundance or by length — a pure-Rust port of
`vsearch --sortbysize` and `vsearch --sortbylength`.

## Usage

```
rsomics-fastx-sort <INPUT> -o <OUTPUT> --mode <size|length> [OPTIONS]
```

### Sort by abundance (`--sortbysize`)

```bash
rsomics-fastx-sort input.fasta -o sorted.fasta --mode size --sizeout
```

- Sorts by `;size=N` abundance **descending**.
- Tie-break: byte-wise comparison of the raw input header (with `;size=N`
  annotation intact) ascending — identical to vsearch's comparator.
- Sequences without `;size=N` default to abundance 1.
- `--sizeout`: strip existing `;size=N` from the header and reappend at end.
- `--minsize` / `--maxsize`: filter by abundance before sorting.

### Sort by length (`--sortbylength`)

```bash
rsomics-fastx-sort input.fasta -o sorted.fasta --mode length
```

- Sorts by sequence length **descending**.
- Tie-break (three-tier, matching vsearch):
  1. Length descending.
  2. Abundance descending.
  3. Raw input header ascending.

### Common options

| Flag | Default | Description |
|---|---|---|
| `--minseqlength` | 1 | Discard sequences shorter than N |
| `--maxseqlength` | 50000 | Discard sequences longer than N |
| `--sizeout` | off | Append `;size=N` to output headers |
| `--sizein` | off | Accepted for compatibility (vsearch always reads `;size=`) |
| `-t` / `--threads` | all cores | Thread count |

## Performance

Measured on macOS aarch64 (mini_m2), vsearch 2.31.0, 500 k sequences / 46 MB
input FASTA, 5 runs with hyperfine:

| Mode | ours `-t1` | vsearch `-t1` | Ratio |
|---|---|---|---|
| `--sortbysize` | 214 ms ± 12 ms | 377 ms ± 14 ms | **1.76×** |
| `--sortbylength` | 226 ms ± 13 ms | 402 ms ± 8 ms | **1.78×** |

## Compatibility

Byte-exact vs vsearch 2.31.0 on:
- Basic golden fixtures (4 records with ties)
- Adversarial 100 k-record synthetic (heavy ties, mixed case, U-containing,
  varying label formats including `;extra=` attributes that stress the raw-header
  strcmp tie-break)
- 46 MB / 500 k-record performance fixture

All `diff` comparisons produce empty output.

## Origin

This crate is an independent Rust reimplementation based on:
- The vsearch source (BSD-2 licence) for `sortbysize.cc` and `sortbylength.cc`
  — comparator logic read and cited directly.
- Black-box behaviour testing against the upstream binary (vsearch 2.31.0).

The upstream source is BSD-2-licensed; no GPL code was referenced.

License: MIT OR Apache-2.0.  
Upstream: [vsearch](https://github.com/torognes/vsearch) (BSD-2-Clause).
