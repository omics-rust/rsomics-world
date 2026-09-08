# 01 — Foundations

Cross-product contracts and upstream surveys for file I/O, compression,
indexing, data structures and execution. A topic appearing here is not
automatically a public foundation; the accepted boundaries and concrete
consumer evidence determine what belongs in Layer A.

## Sub-docs

- [`consumer-driven-audit.md`](consumer-driven-audit.md) — retained public
  foundations, concrete product drivers, and completion gates.
- [`common-consumer-contract.md`](common-consumer-contract.md) — live
  `rsomics-common` call-site matrix and published narrow 0.7 boundary.
- [`help-consumer-contract.md`](help-consumer-contract.md) — unified CLI UX,
  Clap single-source contract, and three-product prototype evidence.
- [`seqio-release-gate-2026-07-31.md`](seqio-release-gate-2026-07-31.md) —
  narrowed sequence-I/O API, two-consumer evidence, representative reader
  performance, and verified registry release.
- [`seqio-bgzf-consumer-contract.md`](seqio-bgzf-consumer-contract.md) — BAM
  and VCF raw-frame overlap, explicit exclusions, and extraction gates without
  freezing a public API.
- [`kmer-consumer-review-2026-09-09.md`](kmer-consumer-review-2026-09-09.md) —
  per-product call sites, canonical short-input repair and current consumer
  correctness/performance evidence.
- [`io-formats.md`](io-formats.md) — FASTA/FASTQ, SAM/BAM/CRAM, VCF/BCF,
  GFF/GTF, BED, MAF, PAF, h5ad. Centred on `noodles` + `needletail`.
- [`compression.md`](compression.md) — current sequence/BGZF/CRAM ownership,
  codec and CLI references, shared-path evidence gates and explicit exclusions.
- [`indexing.md`](indexing.md) — fai/bai/csi/tbi/gzi random-access indexes
  and the `tabix` CLI.
- [`data-structures.md`](data-structures.md) — capability-to-product ownership
  for indexes, hashing, sketches, probabilistic filters and graphs; upstream
  references and adoption gates, not an algorithm-per-crate completion queue.
- [`parallelism.md`](parallelism.md) — `rayon`, async I/O, GPU offload via
  `candle`/`burn`/`wgpu`; how big tools thread today and where Rust improves
  on that.

## Design posture

- Evaluate existing implementations (`noodles`, `needletail`, `rust-htslib`
  and others) against the selected consumer contract before replacing them.
  Their availability does not establish current compatibility or performance.
- A missing Rust capability is not itself a public-crate boundary. Adopt an
  external dependency or keep the implementation inside its first product.
  Promote it only after two named products demonstrate the same policy-free
  contract with consumer tests.
- Higher modules use mature shared parsers when those contracts fit. A parser
  that carries one product's compatibility or output policy stays inside that
  product until a second consumer demonstrates the same policy-free contract.
