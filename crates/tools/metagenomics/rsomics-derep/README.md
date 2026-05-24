# rsomics-derep

Full-length FASTA dereplication — port of `vsearch --derep_fulllength`.

Collapses identical sequences into one representative record, sums abundances, and outputs unique sequences sorted by decreasing abundance.

## Usage

```
rsomics-derep <in.fasta> -o <out.fasta> [OPTIONS]
```

| Flag | Default | Description |
|---|---|---|
| `--sizein` | off | Parse `;size=N` abundance from input headers |
| `--fasta-width N` | 80 | FASTA output line width (0 = no wrap) |
| `-t / --threads N` | all cores | Worker threads |
| `-q / --quiet` | off | Suppress progress stderr |

The output always appends `;size=N` to each header (equivalent to vsearch `--sizeout`). The input `;size=N` field is stripped and replaced.

## Origin

This crate is an independent Rust reimplementation of `vsearch --derep_fulllength` based on:

- The published method: Rognes T, Flouri T, Nichols B, Quince C, Mahé F. **VSEARCH: a versatile open source tool for metagenomics.** PeerJ. 2016;4:e2584. DOI: [10.7717/peerj.2584](https://doi.org/10.7717/peerj.2584)
- Reading the vsearch v2.31.0 source (`src/derep.cc`, `src/fasta.cc`) — vsearch is BSD-2-Clause, so source inspection is permitted.
- Black-box behaviour verification against vsearch 2.31.0 on synthetic and real FASTA inputs.

Key behaviours sourced from `derep.cc`:
- Sequences normalised by uppercasing and U→T before hashing (case-insensitive, RNA/DNA unified).
- `derep_compare_full()` tie-breaking: descending abundance → lexicographic label → input order (first-occurrence seqno).
- `;size=N` stripped from the input header label; new abundance appended at the end.
- Default `fasta_width` = 80 (from vsearch `--help` and empirical verification).

License: MIT OR Apache-2.0  
Upstream credit: [vsearch](https://github.com/torognes/vsearch) — BSD-2-Clause — Rognes et al. 2016.
