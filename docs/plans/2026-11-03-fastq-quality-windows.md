# FASTQ quality-window implementation plan

Execute inline under the existing unattended authority. Spec and source pins:
`.autopilot/state/fastq-quality-frontier-2026-11-03.md`, the FASTQ section of
`docs/10-products/sequence-fastq.md`, and owner `PROVENANCE.md`.

**Goal:** complete front/tail/right quality-window trimming inside the existing
`run`/`trim` SE/PE workflow, not a new crate or stand-alone flag product.
**Architecture:** one private range-selection engine; public typed settings in
the consuming product's TrimConfig; shared record/quality ranges, reports,
ordered Rayon execution, and existing output transaction.

## Decisions

- Public `QualityWindow { window_size, mean_quality }` and `QualityCut` with
  optional front/tail/right windows. Validate sizes1..1000 and qualities1..30
  once at the execution boundary. No speculative Layer A change.
- CLI uses `--cut-front`, `--cut-tail`, `--cut-right`, shared window/mean options
  and per-mode overrides, matching fastp's defaults4/20 and inheritance.
- Compute ranges in original fixed-trim coordinates, before poly-G/poly-X.
  Otherwise nonzero fixed-front and tail settings change upstream cut points.
- Quality-selected discard is an explicit per-record result attributed to
  TooShort even when ordinary quality/length filters are disabled. Do not emit
  empty FASTQ to imitate a missing upstream read.
- Use explicit existing Phred encoding for raw-byte thresholds; normalize64
  only through the existing output path. No heuristic encoding detection.
- Preserve source boundary and mode-priority rules demonstrated by the pinned
  oracle. Document that this is fastp1.3.6 window semantics, not Trimmomatic.
- Retain existing fixed/poly-only hot paths and public trim_record contract.
  Add quality-trimmed reads/bases to the existing report, not fastp JSON claims.

## Integrated delivery task

- [x] Extend one real-oracle workflow test with all three modes, combinations,
  fixed/poly ordering, Phred64, actual retained SE/PE bytes and1/4-worker order.
  Add one malformed-output-preservation check; observe missing-option failure.
- [x] Implement private `transform/quality.rs`; extend `TrimConfig`, metrics,
  CLI and pipeline. Reuse narrow typed interfaces, keep comments sparse.
- [x] Guard actual external paths and resolved target before local commands;
  run fmt, strict Clippy, debug/release and required pinned oracle tests.
- [x] Measure full quality workflows on representative data with complete
  sequence/quality/order comparison, alternating pairs and CPU/wall/RSS. Old
  fixed/poly measurements do not qualify the new path.
- [x] One fresh whole-feature/public-API/hot-path review and necessary fix pass.
  Review discard semantics, original-coordinate cuts, pair synchronization,
  encoding and report totals together, not through a helper-test matrix.
- [x] Document stable scope only after gates; commit/push owner main, wait for
  exact-head native four-platform CI, then advance independent release gates.
  New TrimConfig fields require the next0.2 release, not a0.1 patch.

## Candidate outcome

Implementation `e3a0d81`, scope documentation `ad92c90`, and CI diagnostics
`a0796ca` are pushed. Exact-head run41963314684 passes all four native classes.
The preceding run41962564002 failed macOS ARM release tests; production logic
was not changed and this failure is not represented as a repaired defect.
Original failed metadata and the subsequent successful run remain separate.

The clean `ad92c90` source package matches all45 Git bodies/modes plus Cargo's
two generated files. A package-installed Mac ARM binary passes14 actual calls,
including live fastp outputs, worker order, Phred64, paired gzip, composition,
JSON and failure rollback. These are local candidate gates, not public delivery.
See `.autopilot/state/fastq-quality-execution-2026-11-05.md`.

- [ ] Complete native distribution/source/license delivery and actual public
  installation gates before announcing0.2. Registry authentication remains
  separate from SSH push and native CI; no publication was attempted here.
