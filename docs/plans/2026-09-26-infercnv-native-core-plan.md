# Native inferCNV Core Implementation Plan

> **For agentic workers:** Use superpowers:subagent-driven-development. A fresh
> implementer owns product source, while the controller owns frozen snapshots,
> remote test execution, commits, and durable evidence. No local compilation.

**Goal:** Implement and verify a native prepared-counts-to-fold-change core
against all 40 accepted inferCNV checkpoints on four native platforms.

**Architecture:** An unpublished `rsomics-sc` library contains a checked
column-major CNV model, focused private kernels, and a streaming stage observer.
One coherent implementation task includes its tests and remote validation;
there is no independent empty-scaffold task or public CLI in this slice.

**Tech Stack:** Rust 1.91, edition 2024, `rsomics-common = "=0.12.3"`, standard
library numerical kernels, hosted GitHub Actions native runners.

**Spec:** `docs/plans/2026-09-26-infercnv-native-core-design.md`.

## Global Constraints

- Product source: `/Volumes/KIOXIA/Documents/omics-rust/rsomics-sc`.
- Control plane: `/Volumes/Zane's HDD/Documents/rsomics-world`.
- Local scratch: `/Volumes/KIOXIA/Developments/tmp`.
- No local builds, product tests, or dependency installs while boot APFS exceeds 80%.
- Cargo home and target are `/Volumes/KIOXIA/Developments/cargo-home` and `/Volumes/KIOXIA/Developments/cargo-target`; use hosted runner-temp counterparts for remote compilation.
- `publish = false`; no public GitHub repository, registry publication, binary, empty future modules, Layer B dependency, or new foundation API.
- Reuse published `rsomics-common` 0.12.3 for errors. Any future real CLI must use `rsomics-help`.
- Do not edit inherited VCF work or unrelated untracked files.
- Source comments are rare; no unsafe code, silent repair, nonfinite outputs, or production unwrap except statically obvious invariants.
- The controller alone stages/commits/pushes; agents never delete, create subagents, publish, or dispatch CI.
- Direct `main`, no PR, no Co-Authored-By. Preserve all raw red/green evidence externally.
- Golden comparisons use `abs(actual - expected) <= 1e-12 + 1e-12 * abs(expected)` for this bounded fixture, never silently widened.

## Review Focus

- A missing reference group must fail instead of selecting the all-cell proxy.
- A retained zero-depth column must fail before normalization, without mutating raw counts.
- Column-major storage must not silently transpose genes/cells or smooth across chromosomes.
- Every numeric allocation must respect the declared working-buffer bound; observer snapshots must not accumulate internally.
- A final-value match must not conceal a wrong intermediate transform, missing stage, or unpropagated observer failure.

## Task 1: Checked native pipeline and four-platform oracle regression

**Files in the product repository:**
- `Cargo.toml`: unpublished package `rsomics-sc` 0.1.0, Rust/edition above, exact common dependency; no binary.
- `src/lib.rs`: forbid unsafe and expose only the real `cnv` module.
- `src/cnv/mod.rs`: public type/function exports, not an implementation dump.
- `src/cnv/model.rs`: `Gene`, `Cell`, `PreparedCounts`, `CnvState`, checked construction and read-only accessors.
- `src/cnv/config.rs`: `ReferenceMode`, `PreprocessConfig`, validated runtime reference indices and numeric budget.
- `src/cnv/normalize.rs`: filtering, post-filter depth, log2, clipping and inverse transforms.
- `src/cnv/reference.rs`: group means, bounds and proxy subtraction.
- `src/cnv/smooth.rs`: per-chromosome direct triangular smoothing and global column median.
- `src/cnv/pipeline.rs`: `Stage`, `run`, `run_with_observer`, stage ordering and error propagation.
- `tests/cnv_oracle.rs` and `tests/support/mod.rs`: test-only TSV loading and complete stage/profile differentials.
- `tests/cnv_contract.rs`: public input/configuration/immutability/observer contracts and hand-derived end-to-end witnesses.
- `tests/fixtures/infercnv-1.28.0/`: copied expression/identity TSVs only, plus sorted checksums and provenance README.
- `README.md`: explicitly unreleased numerical core, prepared-input boundary and missing full workflow/release gates.
- `Cargo.lock`: generated remotely, then frozen locally for later locked validation.

**Controller-owned files in the control plane:**
- `.github/workflows/sc-cnv-core-candidate.yml`: immutable-source native matrix, debug/release tests, static checks and raw artifacts.
- `.autopilot/snapshots/sc-cnv-core-2026-09-26/`: separate immutable red/green snapshot subdirectories, archive and file hashes, source receipt.
- `.autopilot/state/infercnv-native-core-2026-09-26.md`: durable task/CI/review record.

**Interfaces:**
Use the exact types and accessors in the spec. `PreparedCounts::new` consumes
gene records, cell records, and a column-major `Vec<f64>` and returns
`rsomics_common::Result<PreparedCounts>`. `run` and `run_with_observer` consume
borrowed prepared counts and explicit configuration. The observer receives
`(Stage, &CnvState)` and returns the common `Result<()>`. `Stage::step()` maps
to `[1,2,3,4,8,9,10,11,12,14]`. All public state access is immutable.

The controller provides remote TDD execution as a service: the implementer
first writes tests, reports their expected missing-implementation failure,
and freezes those files. The controller captures and runs that snapshot,
returns the actual failure and generated lockfile, then authorizes code work.
This is coordination with the controller, not a new user approval gate.

The test-only `Fixture::load(profile: &str)` owns `ExpectedStage` records
indexed by step. `ExpectedStage` holds `Vec<Gene>`, `Vec<Cell>`, and column-major
`Vec<f64>` and provides borrowed `genes()`, `cells()`, and `values()` accessors.
`Fixture::stage(step: u8) -> &ExpectedStage` selects one expected checkpoint;
the stage-1 record constructs the `PreparedCounts`. These helpers do no numeric
transformation and do not belong in production source.

- [ ] Write the contract/golden tests and package manifest before the core.
  Copy fixtures from
  `/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-2026-09-26/run-36216764707/artifact/bundle`.
  Record original source commit, run/head, source hash, original bundle-manifest
  hash, and hashes of the selected files. Read the accepted oracle receipt.
  Do not copy RDS/package source or claim the selected fixture directory is a
  complete original bundle.

  One required independent public-API witness is:

  ```rust
  let input = PreparedCounts::new(
      vec![
          Gene { id: "a".into(), chromosome: "chr1".into(), start: 1, end: 1 },
          Gene { id: "b".into(), chromosome: "chr1".into(), start: 2, end: 2 },
      ],
      vec![Cell { id: "n".into(), group: "normal".into() },
           Cell { id: "t".into(), group: "tumor".into() }],
      vec![1.0, 3.0, 3.0, 1.0],
  ).unwrap();
  let config = PreprocessConfig {
      cutoff: 0.0, min_cells_per_gene: 1, window_length: 1,
      max_centered_threshold: 3.0,
      references: ReferenceMode::GroupBounds(vec!["normal".into()]),
      max_numeric_bytes: 4096,
  };
  let output = run(&input, &config).unwrap();
  for (actual, expected) in output.values().iter().zip([1.0, 1.0, 2.0, 0.5]) {
      assert!((actual - expected).abs() < 1e-12);
  }
  assert_eq!(input.state().values(), &[1.0, 3.0, 3.0, 1.0]);
  ```

  Extend the same fixture with `AllCells` and check reciprocal `sqrt(2)`
  signals. Pin the missing-reference error, empty/duplicate names, nonfinite
  counts, wrong shape, unordered/repeated chromosome blocks, all-filtered and
  zero-post-filter-depth failures, budget insufficiency/overflow and observer
  interruption. The observer test records exact stage numbers and returns an
  injected common error at step 10; later stages must never execute.

  The golden loop must compare each intermediate stage from a single pipeline
  execution, not repeatedly restart the pipeline with oracle intermediates:

  ```rust
  let mut steps = Vec::new();
  let mut observed_final = Vec::new();
  let output = run_with_observer(&input, &config, |stage, actual| {
      steps.push(stage.step());
      let expected = fixture.stage(stage.step());
      assert_eq!(actual.genes(), expected.genes());
      assert_eq!(actual.cells(), expected.cells());
      assert_eq!(actual.values().len(), expected.values().len());
      for (a, e) in actual.values().iter().zip(expected.values()) {
          assert!((a-e).abs() <= 1e-12 + 1e-12*e.abs());
      }
      if stage.step() == 14 { observed_final.extend_from_slice(actual.values()); }
      Ok(())
  }).unwrap();
  assert_eq!(steps, [1,2,3,4,8,9,10,11,12,14]);
  assert_eq!(output.values(), observed_final);
  ```

  Here `fixture` is the test-only loaded profile; final exact equality compares
  the native observer copy, not the R golden values.
- [ ] Controller freezes the test-first source and runs a Linux x86_64 red
  snapshot remotely. Save terminal run identity, failure log, resolved
  lockfile, and source hashes. Verify failure is the intended missing core,
  not dependency/environment failure. No local Cargo command is permitted.
- [ ] Implement the checked model and kernels from the spec. Keep numerical
  scratch reused and bounded; borrow observation snapshots. Add private
  direct-kernel tests for unequal group weights, saturation below/at/above
  thresholds, singleton/short/100/101/102 windows, cross-chromosome isolation,
  and global median. The two-gene window-101 witness is `[500/101,510/101]`.
  Expected-value code must not call or copy the kernel being tested.
- [ ] Self-review, freeze source, and ask the controller to execute the green
  candidate with the recovered lockfile. Exact commands on the hosted runner:

  ```sh
  cargo test --locked --all-targets
  cargo test --locked --release --all-targets
  cargo fmt --all -- --check
  cargo clippy --locked --all-targets -- -D warnings
  ```

  Four native jobs use `ubuntu-24.04`, `ubuntu-24.04-arm`, `macos-15-intel`,
  and `macos-15`; verify `uname` OS/architecture. Set runner-temp Cargo,
  rustup, target, scratch, and artifact paths inside a shell step, not
  job-level `runner` context. Upload source verification, lockfile, compiler
  version, tests and static-check logs even on failure. Stop on manifest
  mismatch. Debug/release tests run on every target; format/Clippy may be one
  Linux x86_64 gate since they do not replace native execution.
- [ ] Fresh task review receives spec, test evidence and full uncommitted
  product diff; fix correctness/quality findings through the implementer and
  revalidate changed code. Final whole-change review also covers the controller's
  CI/snapshot integration. Never silently increase numeric tolerances.
- [ ] Controller commits only verified product files on local `main`, records
  the exact source manifest and all four native results, and commits/pushes the
  control-plane receipt. Verify exact-head Control plane CI after that push.
  Record this as an unpublished core, then advance to raw-state integration
  and the complete downstream workflow; no release or speedup claim here.
