# Metagenomics product gate — 2026-08-02

Status: `rsomics-metagenomics 0.1.0` published and independently verified.

## Released contract

The first release is one complete abundance-aware amplicon lifecycle:

- `dereplicate` performs full-length or prefix FASTA dereplication with checked
  length, abundance, and top-N filters;
- `sort-abundance` orders FASTA records by decreasing `;size=N` abundance with
  stable ties and checked filters;
- `rereplicate` expands abundance-labelled records after computing an exact
  record and byte plan under explicit default budgets.

Strict mode rejects missing required abundance, malformed, duplicate, zero, or
overflowing values. The named `vsearch` profile reproduces VSEARCH 2.31.0
header truncation, abundance parsing, tie order, and 80-column formatting where
the upstream contract intentionally differs. Full-length dereplication streams;
prefix dereplication and sorting retain the records required by their global
ordering contracts.

The command tree uses `rsomics-help 0.4.0`. Runtime errors, JSON results, output
aliases, and transactions use `rsomics-common 0.12.0`; strict plain, gzip, and
BGZF FASTA input uses `rsomics-seqio 0.5.0`. No product-specific policy moved
into those foundations, and this release requires no new `rsomics-kmer` API.

## Exact identities

- published source and VCS identity:
  `34e60b53de0ba1f5937e28fd4f68abb4a1442435`;
- exact-head four-native-target CI: `30748781007`;
- publish workflow: `30748853312`;
- crates.io archive SHA-256:
  `1119a829c3cd2e250a55276d53a8cf15d866296ef8cb36dcaf6da3b650b2785a`;
- crates.io archive size: 27,630 bytes.

## Historical asset disposition

The historical `rsomics-derep` revision
`f3663ce011e8f65ec5bcc227a495165b12ab7dd0`, `rsomics-fastx-sort` revision
`ab7377cd840d2ca97d659a15cc404a90b91f9e69`, and `rsomics-rereplicate`
revision `40e2fb78e9ef2c1f6e632583e925923baf1d9730` were refactored into internal
operation modules. Their useful algorithms, cases, and benchmark shapes were
retained; the three private FASTA parsers, direct-file writes, duplicate header
logic, ineffective thread flags, unchecked arithmetic, and micro-crate command
trees were replaced. Generic length sorting remains routed to `rsomics-seq`.

The historical classifier, taxonomy, and report assets were audited but are
not present as commands or placeholder modules. They remain source and test
assets for a later complete database/build/classify/report slice.

## Correctness and compatibility

The release passed 20 ordinary and oracle tests: nine library tests, one
command-tree test, five CLI integration tests, and five mandatory live VSEARCH
2.31.0 groups. CI downloads the official VSEARCH binary for each native target
and verifies its published SHA-256 before running the oracle; an unavailable
oracle fails rather than skips the job.

The differential matrix covers randomized full-length and prefix
dereplication, abundance sorting with filters and top-N, rereplication header
edges, empty input, zero abundance, and overflow failure. Separate
representative fixtures produced byte-identical outputs with these SHA-256
values:

| Fixture | Output SHA-256 |
|---|---|
| full-length, high duplication | `cd6e5af40ee05b9b262ef580228de8d1249ef6c823a80bb208674a68f71b139d` |
| full-length, all unique | `a7c484d25703d671e73255dbdffa6663f13d531af821de3043efb55da5439b69` |
| prefix dereplication | `0030267e9ea7130470dda65347a106752038cb08968dbf9c685a6aa08f3a9b06` |
| abundance sort | `eb1a6aa92a68d74e173e19c59521ed8281836ae970dbb5767620afb3998919be` |
| rereplication | `5716b7e09f5ebb7fc3d8e693473d9f3b11fab1841c8507593ba13cc1d2bd67ff` |

Checked sums and expansion plans fail before a named output is committed.
Strict parsing and I/O errors reach the top-level command and return nonzero.
The implementation contains cohesive abundance, parser/writer, and operation
modules without future-facing empty files or duplicated legacy parsers.

## Foundation decision

Metagenomics is another real consumer of `rsomics-common`, `rsomics-help`, and
`rsomics-seqio`, but the already published surfaces covered its complete first
slice. The shared abundance/header model stays inside the product because no
second product needs its VSEARCH-specific semantics.

`rsomics-kmer` is absent from the release. A future taxonomy-labelled
minimizer database may justify a general primitive only after classifier-side
tests demonstrate the exact contract and another product uses the same API.
No public taxonomy, classifier, report, or sketch-interchange crate was added.

## Representative performance

The tracked benchmark evidence in the product repository records exact
commands, input and output hashes, all seven measured runs, RSS observations,
and environment details. The gate ran on an Apple M2 Mac mini with eight cores
and 8 GB RAM, macOS 26.6 (25G72), Rust 1.91.0, and VSEARCH 2.31.0. One warm-up
preceded seven measured warm runs; separate full-output runs proved byte
equality.

| Operation | rsomics median | VSEARCH median | Decision |
|---|---:|---:|---|
| full-length, high duplication | 0.26 s, 7.65 MB | 0.27 s, 7.44 MB | 1.04x throughput |
| full-length, all unique | 0.38 s, 145.11 MB | 0.54 s, 171.88 MB | 1.42x throughput and 15.6% less RSS |
| prefix dereplication | 0.18 s, 68.83 MB | 0.23 s, 35.57 MB | 1.28x throughput; RSS regression recorded |
| abundance sort | 0.43 s, 172.57 MB | 0.58 s, 103.09 MB | 1.35x throughput; RSS regression recorded |
| rereplication | 0.03 s, 19.51 MB | 0.30 s, 1.93 MB | 10.0x throughput; RSS funds exact expansion preflight |

The release has a throughput advantage on every declared operation and a
throughput plus memory advantage on the all-unique full-length fixture. Prefix
and sort buffering remain explicit memory optimization targets. The
rereplication result is specific to the declared null-output formatting gate,
not a general 10x claim for every storage path.

## Publication verification

The crates.io API reports version 0.1.0 as non-yanked. An independently
downloaded archive matches the recorded checksum and its
`.cargo_vcs_info.json` identifies the exact source head. An isolated
external-disk `cargo install --locked rsomics-metagenomics@0.1.0` succeeded.

The installed binary completed strict dereplication, abundance sorting, and
rereplication on an independent smoke fixture. The resulting output hashes
were `82799ead097da8736314e52863eddee1916cf9b69483dbe92c9c713916b735ab`,
`3a2199cf9d43f7653a6f205fdb81dbc9ae4547a9449c3337696bd01cff26b884`,
and `3bfdb32c91a6e2e5db6b2f7110bedf1be70cdd160f07af12896cb9823421df61`.

The registry token was exposed to the repository only for publish workflow
`30748853312`; repository access was removed immediately after success and
verified absent.
