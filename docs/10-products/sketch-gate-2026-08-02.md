# Sequence sketch product gate — 2026-08-02

Status: `rsomics-sketch 0.1.0` published and independently verified.

## Released contract

The release provides four complete commands:

- `sketch` builds deterministic canonical-DNA FracMinHash signatures from
  plain, gzip, or BGZF FASTA/FASTQ inputs;
- `inspect` validates the pinned sourmash 0.4 profile and emits metadata;
- `compare` emits pairwise or labelled-matrix Jaccard and directional
  containment results with an explicitly named containment-root ANI estimate;
- `search` performs deterministic linear Jaccard or containment ranking over
  signature files and collections.

The product uses `rsomics-common 0.11.1`, `rsomics-help 0.4.0`,
`rsomics-seqio 0.4.0`, and `rsomics-kmer 0.2.2` from crates.io. Product policy,
signature serialization, comparison, and search remain internal. Indexing,
prefetch, gather, protein and reduced-alphabet profiles, fixed-size Mash
sketches, and `.msh` files are absent rather than placeholders.

## Exact identities

- published source and VCS identity:
  `823fabe3ca092d6e0b30c235cfa961d80c45a543`;
- exact-head four-native-target CI: `30735400718`;
- publish workflow: `30735473183`;
- crates.io archive SHA-256:
  `1c4eb4546f24d56d356e36a3b5df84d496cbacea69991bf8654a159c1eb0c71f`;
- crates.io archive size: 27,685 bytes;
- `rsomics-kmer 0.2.2` source:
  `d89e2df0d8eae38b64eb7b43a41f57436fc25bb4`;
- `rsomics-kmer 0.2.2` archive SHA-256:
  `e1254977d1eaf89b29e727b7ea552ec8bd4bd0740b45fa40ac943e93ffaf9ed4`.

## Historical asset disposition

The historical `rsomics-kmer-dist` head
`7eb179076a1bb6ecdbfc85e9624e96e5a1060e7b` was not revived. Its exact
full-k-mer profile is neither bounded MinHash nor FracMinHash. Small formula
ideas remain test assets; the profile loader, output, CLI, ambiguous-window
policy, unchecked arithmetic, and product boundary were discarded.

## Correctness and compatibility

The implementation pins sourmash 4.9.4 canonical DNA behavior: uppercase
normalization, forward/reverse-complement byte canonicalization, MurmurHash3
x64 first-lane output with a full-width seed, scaled threshold selection,
sorted unique hashes, optional retained-hash abundances, and the sourmash
content digest. Default construction skips exactly the windows affected by a
non-ACGT byte; `--check-sequence` rejects the first such byte.

The live oracle suite covers `k` values 1, 17, 31, and 51, scaled values 1, 7,
10, and 100, abundance, lowercase and ambiguity, a seed above the `u32` range,
multiple input signatures in one collection, containment-matrix orientation,
linear-search scores, and content digests. Generated uncompressed signature
files are byte-identical to sourmash. Independent unit tests cover scaled
thresholds, digest validation, gzip round trips, downsampling, empty-sketch
semantics, failed-input output preservation, and output alias rejection.

Named outputs use `rsomics-common` transactions. Parse, integrity, profile,
compatibility, and I/O failures propagate to the top-level command and return
nonzero. The initial implementation has four cohesive operation and CLI
modules plus compact crate roots, with no empty feature modules or placeholder
commands.

Exact-head CI passed formatting, strict Clippy, rustdoc with warnings denied,
package reconstruction, benchmark smoke, the ordinary test suite, and the live
sourmash oracle on native Ubuntu 24.04 and macOS 15 for both `x86_64` and
`aarch64`.

## Foundation decision

`rsomics-sketch` is the second real product consumer of `rsomics-kmer` after
`rsomics-seq`. It drove one general API: an allocation-reusing canonical
DNA-window Murmur64 iterator with arbitrary nonzero `k`, full-width seed,
explicit invalid windows, and no product-level threshold or sketch policy.
Foundation unit tests plus sketch consumer tests pin the shared contract.

FracMinHash selection, abundance maps, signature types, digest policy,
comparison metrics, ANI labels, and search ranking remain in the product. No
public sketch, distance, signature, collection, or taxonomy crate was added.

## Representative performance

The tracked product evidence is
[`benchmarks/2026-08-02`](https://github.com/omics-rust/rsomics-sketch/tree/main/benchmarks/2026-08-02).
It records exact commands, tool revisions, input and output hashes, all timing
observations, RSS observations, and environment details.

On an Apple M2 running macOS 26.6, the 4.70 Mbp *E. coli* K-12 MG1655 genome
at `k=31`, `scaled=1000` produced byte-identical output. Across three warmups
and 15 measured runs, sourmash averaged 1.035143 seconds and rsomics averaged
0.813499 seconds: 1.27 times the throughput. Five separate memory runs averaged
134,955,008 versus 9,682,944 bytes peak RSS, a 92.8% reduction.

The secondary 6,282,141-read gzip FASTQ abundance case also produced
byte-identical output. One paired release-gate observation measured 43.29
seconds and 108,118,016 bytes for sourmash versus 51.04 seconds and 9,093,120
bytes for rsomics. This path is 17.9% slower but uses 91.6% less peak RSS. The
single timing is sufficient to expose the regression, not to claim a stable
distribution. The release therefore passes through a strict throughput win on
the genome path and a strict resource-use win on both paths; FASTQ abundance
throughput remains an explicit optimization target.

## Publication verification

The crates.io API reports version 0.1.0 as non-yanked. An independently
downloaded registry archive matches the recorded checksum and its
`.cargo_vcs_info.json` identifies the exact source head. An isolated
external-disk `cargo install --locked rsomics-sketch@0.1.0` succeeded. The
installed binary reported version 0.1.0 and generated the expected
`79299a40184a8b5c1bb119c72eff6ca60aa3620d3dde796600c51b5d62ba569d`
signature from the recorded *E. coli* fixture.

The registry token was exposed to the repository only for publish workflow
`30735473183`; repository access was removed immediately after success and
verified absent.
