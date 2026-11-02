# Sequence strand-extraction implementation plan

> Execute inline or as one isolated implementation wave, with one fresh whole-feature review. The user's unattended authorization replaces another approval checkpoint. No helper-by-helper review ladder.

**Goal:** Extend the existing sequence product with complete record-preserving regional extraction and reverse-complement workflows, not additional crates.

**Architecture:** Existing strict input traversal, common atomic/alias handling, seqio validated writer/encoder and help remain authoritative. Product-local region resolution and reusable record-sized strand buffers supply the new operations. No foundation API or dependency is needed.

**Spec:** `docs/10-products/sequence-fastq.md`, the contract below, and pinned SeqKit2.13.0 command/source behavior.

## Source and contract

Clean owner is `/Volumes/KIOXIA/Documents/omics-rust/rsomics-seq` at
`d9734e51c4ed557f6d8790d97a686717ebc4769e`. Actual CLI/modules contain only
stats/kmers/grep/convert/validate. Later dossier groups remain incomplete.

Read pinned source before reconstructing behavior:

- https://raw.githubusercontent.com/shenwei356/seqkit/v2.13.0/seqkit/cmd/subseq.go
- https://raw.githubusercontent.com/shenwei356/seqkit/v2.13.0/seqkit/cmd/seq.go
- https://bioinf.shenwei.me/seqkit/usage/#subseq

Root inspected the region parser, streaming subseqByRegion and seq reverse/
complement/quality output routes. SeqKit's region coordinates are signed,
nonzero, 1-based inclusive; first-record alphabet selection matters. Its
plain FASTA side-index creation is not a required native side effect. Use
stream input for source goldens to avoid index side effects and preserve order.

Historical team-owned assets, not dependencies:

- fasta-subseq2f0bc3e920a429c36af7a1fb5cb45b7fdf47c802: refactor region parser/
  resolver and retain positive/negative/clamped/header-coordinate evidence.
  Its shell always emits FASTA and requires UTF8 headers: discard that shell.
- fasta-utils93e81c2ab88524f97bdaaab6f34105743d798b96: refactor revcomp scratch
  and DNA/RNA/IUPAC complement tables into this product.
- fastq-utils7f0551d463977e6343af3c9477879799d1bcbc81: retain reverse-quality
  semantics/RNA golden; replace per-record allocations and manual output.
  Its sequence-only live differential is insufficient for full FASTQ parity.

## Public operation slice

- `subseq [INPUT=-] --region/-r START:END [--region-coord/-R] --output/-o PATH`.
  Preserve input FASTA/FASTQ, full byte header, record order and quality slice.
  Coordinates follow pinned source, including clamping and empty windows;
  reject zero, malformed or negative-start/positive-end regions. If requested,
  append the original signed coordinates to the parsed ID with source-compatible
  description formatting. Observe full raw oracle output before pinning it.
- `revcomp [INPUT=-] --output/-o PATH`. Match `seqkit seq -r -p -w 0`:
  infer one alphabet from the first record using the existing classifier,
  DNA/RNA IUPAC case-preserving complement, protein/unlimited reverse-only,
  and reverse FASTQ quality. Recycle sequence/quality capacity across records.
- Existing JSON summary policy, stdin/stdout, content-detected input compression,
  named BGZF output, alias refusal and atomic rollback apply unchanged.
  Do not add a threads flag to a serial path or claim unmeasured scaling.
- BED/GTF/flanks/indexed retrieval, wrapping and broader seqkit-seq transforms
  remain explicit later scope; do not advertise them or discard their assets.
  Existing grep matching/alphabet policy must not silently change to accommodate
  revcomp. Share policy-free internals only when their contracts truly agree.

## One coherent implementation wave

- [ ] Read complete owner input/output/CLI/library/tests and selected historical
  algorithms/tests; freeze exact source baseline and inspect dirty state.
- [ ] Prepare an owner-specific external preflight from the already inspected
  intervals guard. Verify physical volume, paths, resolved target, Rustup,
  caches and8GiB external headroom before every build/oracle invocation. Use
  Cargo locked/offline; no new downloads/dependencies or boot-disk output.
- [ ] Capture actual installed SeqKit identity, then a compact connected set
  of complete FASTA/FASTQ source outputs for both operations: signed/clamped/
  empty ranges, coordinate headers, DNA/RNA/IUPAC and full reversed qualities.
  If installed version differs, do not call it2.13.0 or reuse old goldens as
  fresh evidence. Record source behavior contradictions before implementation.
- [ ] Add connected CLI regressions first and observe their failure because
  commands are absent. Include the extraction→revcomp pipeline, compressed
  named output/rollback and JSON summary using existing helpers, not a new
  framework or exhaustive helper matrix.
- [ ] Implement modules `operations/subseq.rs` and `operations/revcomp.rs`,
  narrowly necessary private strand/region internals, library exports and CLI
  routing. Keep comments rare; no manual FASTX parser/writer or new public
  Layer A item. Prefer a validated region type with private coordinate fields.
- [ ] Run formatting, strict all-target/all-feature Clippy, full debug/release
  tests and actual complete-output source differentials. Record all failures;
  missing upstream is not a passing compatibility test.
- [ ] Obtain one fresh integrated review of region/quality/header/order/error
  behavior and bounded allocation, then one material correction pass. Preserve
  source/fixture provenance in the owner merge record; root reviews final delta.

## Subsequent delivery gates

Do not bump/publish merely because local implementation passes. After source
acceptance, qualify actual native Linux/macOS x86_64/aarch64 at the frozen
candidate and measure both useful operations plus the connected workflow on
representative non-trivial reads, retaining whole outputs, exact binary/input/
flags, timing distributions and process RSS. Strict throughput or resource
advantage is required per operation; no inheritance of0.2.0 performance.
Registry authorization is separately held. No unchanged credential retry.

Review focus: FASTQ quality coordinate/reversal consistency; signed-range
arithmetic without overflow; first-record DNA/RNA selection versus later
records; byte headers/coordinate suffix formatting; named-output rollback
after malformed late input or write failure. Existing strict boundary/type
checks should not be duplicated in every operation.
