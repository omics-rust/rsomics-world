# VCF shared region-file consumer and normal concat oracles

> **For agentic workers:** Execute one coherent consumer/compatibility wave with `superpowers:executing-plans`; one fresh whole-feature review, not per-helper approvals or reviews.

**Goal:** Make VCF's shared file-selection entry point use the existing public interval parser and turn observed normal concat workflows into maintained differential tests.

**Architecture:** VCF consumes `rsomics-intervals 0.4.0` without changing that foundation. The foundation already has concrete Call and CNV consumers. VCF alone retains its legacy colon-selector policy and converts normalized half-open bounds to its existing Noodles regions. Inline selection, overlap policy, traversal and output remain unchanged.

**Tech stack:** Rust1.91; existing common/help and typed VCF/BCF/index machinery; published intervals0.4.0; pinned installed bcftools/HTSlib1.24.

**Spec/evidence:** `docs/plans/2026-08-19-vcf-concat-design.md`; new captures at `/Volumes/KIOXIA/Developments/tmp/vcf-concat-pinned-probe-20261102.uIrZ88/REPORT.md` and `/Volumes/KIOXIA/Developments/tmp/vcf-region-file-formats-20261102.FI6p2q/REPORT.md`. These are observed data, not already accepted tests.

## Global constraints

- Owner is `/Volumes/KIOXIA/Documents/omics-rust/rsomics-vcf`, HEAD682942cfa69768dc3a127a8544f2f07213b704ea, inherited dirty work/index empty. Baseline is the174-body reviewed validated-stream candidate, not HEAD alone. Preserve all other work and old snapshots; no owning commit/stage/push/publication/deletion in this task.
- Every build/download/scratch/cache remains on verified external storage. Use the inspected VCF guard and exact external Rust/Cargo/Rustup/target/temp paths, locked/offline after the deliberate lock update; retain8GiB headroom.
- No public API, command/flag/version, new crate, Layer A item or foundation edit. Add only the existing intervals0.4 dependency and minimal resulting lockfile entries. Reject unrelated dependency churn.
- Preserve ordinary `Reader::open`, validated BGZF repair, all inline/caret behavior, sample-list parsing, typed record handling, selected-index/traversal rules, raw splice policy, transactions and numeric comparisons.
- Same-coordinate row ordering and missing-LF behavior each have a pending user choice. Do not select either policy, normalize away body order, or turn source corruption into expected native behavior. Continue unambiguous normal work.
- No performance/new native-four/complete-concat/publication claim from this wave. Existing frozen0.6 must not be published unchanged.

## Concrete foundation and policy boundary

`rsomics-intervals` clean cda29c88b67882b63e1710a22aa480e905d4423f is byte-identical to cached registry0.4.0 for the parser. Existing consumers are Call `src/region_file.rs` and CNV `src/selection.rs`. Its `read_region_file(path, mode)` already detects gzip content, BED/VCF filename formats, tabular coordinates, line-context errors and whole-reference selections.

Use `RegionFileMode::Targets` for VCF's existing file grammar: it already accepts whole-reference and colon selectors, unlike strict Call region files. Bounded foundation records become Noodles `start+1..=end`, with checked coordinate conversion; whole-reference tokens pass through the existing VCF region-string parser so colon selectors remain product-local. Keep foundation modes and Call/CNV behavior unchanged. The mode name describes a coordinate requirement, not the command using it.

Do not add another gzip/BED/VCF/tabular parser or copy Call's whole module. A small typed conversion is VCF policy; parsing remains Layer A. Preserve a literal reference name in bounded coordinate records rather than converting the row back into a colon-form string.

Known foundation source finding: suffix inference currently falls back to tabular format for non-UTF-8 filenames. Do not claim that edge qualified or silently fix it inside a second VCF parser. Record it for the consumer-driven foundation follow-up before public delivery; no runtime observation is inferred from source alone.

## Review focus

- BED is zero-based half-open; TSV is one-based closed; a VCF region row contributes POS, not REF span. Gzip decoding must not pass compressed bytes to UTF-8 text reading.
- Regions, targets and masks share one file path; `^` exclusion and existing comma/colon inline grammar must survive. Generic sample-list parsing must not become a region parser.
- Legacy colon files stay accepted without weakening coordinate conversion or moving product-specific grammar into Layer A.
- A malformed/truncated selection file must fail before output replacement with useful file context. Existing foundation validation is reused, not duplicated as a new defensive test matrix.
- Differential tests assert complete scientific rows and explicit per-tool safety contrasts; body-order normalization or counting ignored oracles as passes is not allowed.

## Task 1: Shared file-selection consumer and normal workflow evidence

**Files:**

- Modify `Cargo.toml`/`Cargo.lock` only for intervals0.4.0.
- Modify `src/commands/variant.rs` to separate regional files from generic sample lists while preserving existing public(crate) function signatures.
- Add `src/regions/file.rs` as the small product-private adapter and wire it through `src/regions.rs`; it may access the existing private `parse_region` for whole-reference/legacy tokens. Do not change overlap or indexed-query algorithms.
- Add one connected region-file CLI test group, preferably `tests/regions_file_cli.rs`, using the existing binary/fixture conventions.
- Extend `tests/concat_compat.rs` in coherent workflow groups, retaining existing tests and expected-divergence assertions.
- Do not change the three validated-stream bodies except for a demonstrated integration necessity recorded before editing.

**Interfaces:**

- Consume `rsomics_intervals::{read_region_file, RegionFileMode, GenomicRegion, RegionFileError}`.
- Produce `pub(crate) fn RegionSet::read_file(path: &Path, overlap: OverlapMode) -> Result<Self>` using private `file::read(path: &Path) -> Result<Vec<Region>>`. Keep `read_regions`, `read_targets`, `read_mask`, and `read_list` caller signatures stable.
- `read_list` remains the old generic list reader used by sample selection. Regional file branches use the adapter; inline branches retain `RegionSet::parse` / `RegionSelection` policy. Do not swallow I/O/decompression/parse errors.

- [ ] Freeze the174-body/mode/index baseline, dependency graph and exact permitted delta. Read the actual shared VCF entry points, complete foundation parser/contracts, and existing Call/CNV adapters before editing.
- [ ] Add the connected region-file consumer regression before production edits. For both `view` and indexed `concat -a`, assert complete literal rows for six observed normal formats plus old colon syntax: TSV point10→del10; TSV10..11→del10,snp11; BED9..10→del10; uppercase ordinary-gzip BED→del10; ordinary-gzip TSV→both; VCF POS10/REF=AC→del10. Expected red: the six normal formats fail on unchanged code; legacy passes. Include one meaningful target/caret or mask-file consumer assertion and one malformed-file rollback case, not a helper-by-helper test grid.
- [ ] Add intervals0.4.0 and its minimal lock update under verified external storage. Wire the existing reader through the typed product adapter. Keep existing inline/caret/sample behavior and validated BGZF source unchanged. Expected green: complete new CLI workflow plus existing selection/concat tests pass.
- [ ] Expand the existing encoding loop from diagonal `zip` to all16 input/output combinations and add literal full-body assertions. Retain naive BCF. Add one ordered-list/BCF-G workflow: relative-list cwd resolution and complete three-row GT/DP body; BCF-G to all four outputs yields eight columns and no FORMAT/sample schema.
- [ ] Add one complete phase-handoff workflow using the observed six confident/independent-sample default/compact cases, including thresholds30/0. Freeze the full four-row outputs from the retained capture rather than derive source expectations from Rust.
- [ ] Strengthen existing safety contrast bodies only where the single capture supplies exact observations. Preserve native rejection/transaction behavior for decreasing order, schema conflict and malformed QUAL, working native stdin, and recorded ligation divergences. Do not add placeholder tests for undecided tie/newline cases.
- [ ] Run one final coherent format/strict-Clippy/full debug+release/explicit concat-oracle bundle; run relevant existing selection oracles against pinned1.24 where the shared entry point changes. Report other ignored oracle groups separately. No benchmark or unchanged full native-four campaign is introduced here.
- [ ] Obtain one fresh whole-feature review of the exact new delta plus relevant inherited integration, fix material findings in one pass and record every declined scope/minor. Root alone retains recovery patches, state and later integration decisions.

## Decisions and remaining gates

The user authorized unattended meaningful product work; no additional approval is needed for this already demonstrated shared parsing gap. This plan does not choose the two pending public compatibility policies. Foundation filename handling, complete concat ordering/newline acceptance, representative performance/resource gates, owning exact-head native-four integration and independent delivery still precede publication.
