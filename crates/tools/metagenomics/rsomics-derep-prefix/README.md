# rsomics-derep-prefix

Prefix FASTA dereplication — port of `vsearch --derep_prefix`.

A shorter sequence that is an exact prefix of a longer one is collapsed into
the longer sequence (the representative), summing abundances. Output is sorted
by decreasing abundance; ties broken by raw-header `strcmp` then input order.

## Usage

```
rsomics-derep-prefix <in.fasta> -o <out.fasta> [--sizein] [--minseqlength 32]
```

Options match vsearch defaults: `--minseqlength 32`, `--maxseqlength 50000`,
`--fasta-width 80`.

## Origin

This crate is an independent Rust reimplementation of `vsearch --derep_prefix`
based on:

- The vsearch source file `src/derep_prefix.cc` (BSD-2-Clause licensed; read
  for algorithm correctness, not copied).
- Black-box behavior testing against vsearch 2.31.0.

The algorithm: read all sequences, sort shortest-first, then for each
sequence (a) find an exact match → accumulate abundance, (b) find a shorter
existing prefix entry → promote the longer seq as representative, or (c) open
a new cluster. Uses FNV-1A incremental prefix hashing matching vsearch's
`compute_hashes_of_all_prefixes`. Open-addressing hash table at 2/3 fill rate.

License: MIT OR Apache-2.0.
Upstream credit: vsearch <https://github.com/torognes/vsearch> (BSD-2-Clause / GPL-3.0).
