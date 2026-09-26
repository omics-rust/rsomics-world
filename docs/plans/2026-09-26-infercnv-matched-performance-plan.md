# inferCNV matched-performance implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Measure and validate the accepted prepared-input Rust and inferCNV pipelines on the same native Linux host without changing the production algorithm.

**Architecture:** A private Rust benchmark and an R runner perform one warm-up and one timed call per process. A Python driver verifies inputs and every output, launches alternating fresh processes, and reports region and lifetime metrics separately. The existing oracle workflow hosts measurement only after its real upstream runs and a four-native-tested snapshot are verified.

**Tech Stack:** Rust 1.91, Linux-only nix 0.29.0 dev dependency, R 4.6.1/infercnv 1.28.0, Python standard library, GitHub Actions, GNU time.

**Spec:** `docs/plans/2026-09-26-infercnv-matched-performance-design.md`.

## Global constraints

- Product root: `/Volumes/KIOXIA/Documents/omics-rust/rsomics-sc`; control root: `/Volumes/Zane's HDD/Documents/rsomics-world`.
- Production-core bytes remain identical to `bdcc8a3a55596be8d46a774d39b0335845c75175`; no public API, binary, production dependency, CLI or publication.
- No local Cargo/R/build/dependency installation while boot APFS exceeds 80%. Hosted runners execute all such work. External scratch: `/Volumes/KIOXIA/Developments/tmp`; evidence: `/Volumes/Zane's HDD/rsomics-fixtures/evidence/`.
- Rust version 1.91; common stays exactly 0.12.3. Add only `nix = { version = "=0.29.0", default-features = false, features = ["resource"] }` under Linux target dev-dependencies, and its required lock nodes. No authored unsafe.
- Preserve every existing lock package version/checksum. Resolve remotely once against the existing lock; subsequent tests and builds use `--locked`.
- Non-default feature `infercnv-measurement`; custom bench `cnv_matched`, `harness=false`, `test=false`, required feature. Actual timing is Linux x86_64 only; unsupported benchmark invocation fails.
- Use the full accepted example only: stage 1 9939 by 184, final 8508 by 184; exact identities and `1e-12 + 1e-12*abs(expected)` numerical criterion.
- Cutoff 1, detection minimum 3, bounds references `Microglia/Macrophage` then `Oligodendrocytes (non-malignant)`, window 101, clamp 3, one thread.
- One smoke pair excluded, then seven pairs of fresh processes with alternating first implementation. Preserve all failures/partials. No unit-test speed assertions.
- Rare explanatory source comments; types/names carry intent. Direct main commits, one concern each, no PR/Co-Authored-By. Controller stages and pushes only owned files and waits for exact-head CI.
- One implementation agent at a time; independent read-only reviews/preparation may run concurrently. Do not delete evidence/workspaces or create automations.

## Review focus

1. The warm-up changes the input or leaves a result live: compare original computational state and drop the result before baseline; Task 1/2 tests check the state/export path.
2. An RDS with the right primary matrix has wrong groups or a hidden spike: Task 2 semantic negative tests reject it before any timing.
3. Lifetime high-water RSS looks like region allocation: Task 2 schema/summary tests keep baseline and peak separate and never subtract them.
4. A custom benchmark runs accidentally in ordinary conformance: Task 1 uses explicit named measurement tests and compile-only bench checks; ordinary all-targets tests never enable its feature.
5. A successful CI receipt points at different timed bytes: Task 3 tests exact run/head/attempt, source/manifest/lock pins, safe extraction and unchanged production bytes.

## File and interface map

Product changes: `tests/support/mod.rs` gains a single-stage entry point; `benches/support/mod.rs` owns metric parsing, clocks and final export; `benches/cnv_matched.rs` owns the trial lifecycle; `tests/cnv_measurement_contract.rs` covers support; `Cargo.toml`, resolved `Cargo.lock`, and `tests/README.md` bind opt-in execution.

Control changes: `scripts/infercnv_matched.R` and `scripts/infercnv_matched_support.R` own the actual R call and preparation checks; `scripts/test_infercnv_matched.R` exercises those checks with the real installed package. `scripts/measure_infercnv.py` owns process orchestration, TSV validation and summaries, with `scripts/test_measure_infercnv.py`. `scripts/verify_infercnv_measurement_snapshot.py` and its test bind the native candidate to the measured source. Existing candidate/oracle workflows are extended, not duplicated.

Trial executables accept exactly two positional arguments:

```text
cnv_matched CASE_DIRECTORY NEW_OUTPUT_DIRECTORY
Rscript --vanilla scripts/infercnv_matched.R BUNDLE_DIRECTORY NEW_OUTPUT_DIRECTORY
```

Both create `metrics.tsv`, `14.tsv`, `14.genes.tsv`, `14.cells.tsv`; metrics is a two-column `metric\tvalue` table with unique keys:

```text
schema_version = 1
implementation = rust | infercnv
preparation_wall_seconds
region_wall_seconds
region_user_seconds
region_system_seconds
region_child_user_seconds
region_child_system_seconds
baseline_rss_bytes
```

All durations are finite nonnegative decimals, region wall is positive, child CPU is zero, RSS is a positive integer. Runtime/configuration provenance is a separate JSON or TSV, not overloaded into numeric fields. The driver stores raw stdout/stderr and GNU time's separate lifetime file beside the executable-owned result directory.

### Task 1: Private Rust trial and four-native verification

**Files:** Product files in the map above; controller-owned `.github/workflows/sc-cnv-core-candidate.yml` and new immutable snapshots under `.autopilot/snapshots/sc-cnv-core-2026-09-26/`.

**Interfaces:**
- Consume existing `PreparedCounts::new`, `PreprocessConfig`, `cnv::run`, `CnvState` accessors.
- Produce `ExpectedStage::load(root: &Path, step: u8) -> Result<Self, String>`; existing ten-stage loading delegates to it, preserving all prior validation.
- Produce `parse_rss_bytes(&str) -> Result<u64, String>`, `checked_cpu_micros(i64, i64) -> Result<u64, String>`, `checked_counter_delta(u64, u64) -> Result<u64, String>` in private bench support. Units are seconds/microseconds for the CPU pair, and before/after for deltas.
- Produce the executable/output contract above. Product baseline is `bdcc8a3a`; world baseline is `282d2171`.

- [x] Write test-first single-stage tests and measurement-support contracts. Signal the controller before implementing missing functions, so the immutable red snapshot captures actual failure.

```rust
#[test]
fn rss_requires_one_positive_kib_counter() {
    assert_eq!(measurement::parse_rss_bytes("Rss: 17 kB\n").unwrap(), 17408);
    for text in ["", "Rss: 1 MB\n", "Rss: 0 kB\n", "Rss: 1 kB\nRss: 2 kB\n"] {
        assert!(measurement::parse_rss_bytes(text).is_err(), "{text}");
    }
}

#[test]
fn cpu_units_and_deltas_are_checked() {
    assert_eq!(measurement::checked_cpu_micros(2, 3).unwrap(), 2_000_003);
    assert!(measurement::checked_cpu_micros(-1, 0).is_err());
    assert!(measurement::checked_cpu_micros(0, 1_000_000).is_err());
    assert!(measurement::checked_cpu_micros(i64::MAX, 0).is_err());
    assert_eq!(measurement::checked_counter_delta(7, 12).unwrap(), 5);
    assert!(measurement::checked_counter_delta(12, 7).is_err());
}
```

Also test RSS numeric overflow/trailing garbage/missing counter; a fixture directory containing only stage 1 succeeds for that stage, wrong step/missing file/malformed row fails; result exports preserve rectangular column-major values and exact gene/cell identities. Tests use real support functions and temporary directories, not timing thresholds.

- [x] Controller adds `measurement` and `resolve_measurement_lock` dispatch booleans. Resolution is allowed only for Linux expected-red with measurement enabled. Run `cargo metadata --all-features --format-version 1` against the existing lock once; use Python `tomllib` to compare the pre-resolution lock and reject changed/removed existing packages except root dependency edges. Retain both locks and logs. Source-after skips only this intentionally changed lock for the resolution run. Every subsequent run verifies it normally.
- [x] Freeze red source, commit/push and wait for exact-head Control plane CI. Dispatch the Linux red run. Run ordinary tests plus `cargo test --locked --features infercnv-measurement --test cnv_measurement_contract -- --nocapture`; require the intended missing-support compile/test failure, not dependency or runner failure. Preserve source, logs and the mechanically resolved lock; inspect allowed lock changes before copying that generated lock into the product.
- [ ] Implement the private support and trial. Use safe `nix::sys::resource::getrusage(UsageWho::RUSAGE_SELF)`, checked `Instant` wall time and unique `Rss` from `/proc/self/smaps_rollup`. Gate OS calls to Linux; keep pure parsers portable. No new production code.

```rust
let input = PreparedCounts::new(first.genes().to_vec(), first.cells().to_vec(), first.values().to_vec())?;
drop(first);
let warm = cnv::run(&input, &config)?;
drop(warm);
let baseline_rss_bytes = read_current_rss()?;
let before = process_cpu()?;
let start = Instant::now();
let output = cnv::run(&input, &config)?;
let end = Instant::now();
let after = process_cpu()?;
let wall = end.checked_duration_since(start).ok_or("wall clock reversed")?;
```

The snippet shows ordering; private OS helper return types may be narrow structs. Compute preparation wall before warm-up. Validate original input against stage 1 after timing by reloading it, avoiding an extra retained stage-1 copy during measurement. Load expected stage 14 only after timing, compare every identity/value, then export. Use buffered output and enough decimal precision to round-trip f64. Reject preexisting output dirs and propagate all errors to nonzero exit. Reference roles in export derive from the fixed accepted configuration; no guessed arbitrary profile support.

- [ ] Format with external standalone rustfmt; freeze green source with the reviewed lock. All four native targets run old debug/release tests unchanged, explicit measurement test debug/release, and `cargo check --locked --all-features --bench cnv_matched`. Linux runs format/strict all-feature all-target Clippy. Never invoke a harness-free bench from `cargo test --all-targets --all-features`.
- [ ] Independent task review covers both product and CI diffs. Controller verifies four jobs, original artifacts/logs, exact source hashes and unchanged production bytes; commit product only after acceptance as `test(sc): add private prepared-input measurement harness`. Record native run identity for Task 3; preserve all red/green artifacts.

### Task 2: R trial, pinned input and checked alternating driver

**Files:** Create `scripts/infercnv_matched.R`, `scripts/infercnv_matched_support.R`, `scripts/test_infercnv_matched.R`, `scripts/measure_infercnv.py`, `scripts/test_measure_infercnv.py`, and `.autopilot/oracles/infercnv-measurement-input-2026-09-26.json`.

**Interfaces:**
- R `validate_prepared(obj, bundle_root, record)` verifies the full stage-1 RDS and returns invisibly; `run_prepared(obj, out_dir)` makes the fixed actual infercnv call; `prepared_identity(obj)` captures all computational slots for unchanged-input checks.
- Python `validate_pins(bundle: Path, receipt: dict) -> dict`, `validate_trial(result: Path, expected: Path, implementation: str) -> dict`, `summarize_trials(trials: list[dict]) -> dict`, `run_measurement(bundle: Path, binary: Path, output: Path, input_receipt: Path, r_script: Path) -> dict`.
- CLI: `python3 -B scripts/measure_infercnv.py --bundle BUNDLE --binary BINARY --output NEW_DIR --input-receipt RECEIPT --r-script SCRIPT`.
- The input receipt binds accepted oracle receipt SHA, source commit, fixed full dimensions/config, and six relative-path SHA256 entries for full stage 1/14 expression/genes/cells. Controller derives them from already accepted bytes, not from a fresh unaccepted export.

- [ ] Write Python tests first, observe missing implementation failure using external TMPDIR and `python3 -B -m unittest discover -s scripts -p test_measure_infercnv.py`. Use small explicit unit-only matrices, not purported upstream fixtures.

```python
def test_missing_and_duplicate_samples_fail(self):
    with self.assertRaises(ValueError):
        summarize_trials([])
    trials = self.valid_trials()
    trials[-1] = trials[0].copy()
    with self.assertRaises(ValueError):
        summarize_trials(trials)

def test_smoke_is_not_a_measured_sample(self):
    trials = self.valid_trials()
    trials[0]["phase"] = "smoke"
    with self.assertRaises(ValueError):
        summarize_trials(trials)
```

Cover incorrect pins, finite/tolerance/identity/output failures, duplicate metric keys, missing/NaN/negative durations, nonzero child CPU, bad RSS, malformed GNU time, nonzero subprocess and timeout, exactly seven unique pairs, smoke exclusion, alternating order and overlapping/nonoverlapping distributions. Subprocess tests may invoke a tiny real Python child to exercise failure, clearly unit-only; no fake infercnv measurement.

- [ ] Implement pin validation and R preparation. Verify fresh oracle.json full/stage-1 checkpoint path and hash within the validated bundle. Compare dense expression and count.data dimensions/names/values to pinned stage 1, gene coordinates, ordered reference/observation maps against cell table roles/groups, complete disjoint coverage and NULL `.hspike`. Capture those slots before calls and compare afterwards without repairing them. Reuse `infercnv_oracle_io.R` export helpers; do not copy oracle-generation logic.
- [ ] R negative tests load the actual fresh stage-1 checkpoint after installation, prove baseline preparation passes, then mutate one group membership, hidden spike, count.data and gene coordinate at a time and require preparation failure before timing. Test malformed RSS and invalid clock deltas through pure support helpers. The hosted runner runs these before measurement; local R stays forbidden.
- [ ] Implement `run_prepared` with exact flags from the source-reviewed design, including `save_rds=FALSE`, `save_final_rds=FALSE`, `write_expr_matrix=FALSE`, `write_phylo=FALSE`, `diagnostics=FALSE`, `inspect_subclusters=FALSE`, samples mode and `up_to_step=14`. Verify formal names against the pinned source. Pre-create separate warm/measured output dirs; release warm return, `gc(full=TRUE)`, sample baseline and proc.time, time the actual call, retain return and validate/export after timing. Report all exact arguments, package/session/BLAS/effective-thread information outside timing.
- [ ] Driver sets OPENBLAS_NUM_THREADS, OMP_NUM_THREADS, MKL_NUM_THREADS, BLIS_NUM_THREADS and VECLIB_MAXIMUM_THREADS to `1` before every direct child. Use `subprocess.run` with argument lists, `check=True`, timeout and raw output files; preserve partials on error. GNU time wraps the exact executable, not Cargo or a shell pipeline.

```python
order = [("smoke", 0, ("rust", "infercnv"))]
order += [("measured", pair, ("rust", "infercnv") if pair % 2 else ("infercnv", "rust"))
          for pair in range(1, 8)]
for phase, pair, implementations in order:
    for implementation in implementations:
        run_and_validate(phase, pair, implementation)
```

`run_and_validate` is a private driver helper. Timeout is 1200 seconds per fresh process. GNU time format uses unique tab-separated wall/user/system seconds, maximum_rss_kib and exit_status. Store raw per-trial metrics, final hashes, finite maximum absolute/scaled errors and original execution order. Reject any wrong result before accepting its timings. Summaries record medians/ranges and R/Rust median wall ratio; clear advantage iff `max(rust) < min(infercnv)`. Baseline and peak have separate labels and no derived region allocation.
- [ ] Run focused Python tests, all control-plane tests and validation; review R source and driver independently. R execution gate remains explicitly pending until Task 3's actual installed oracle job. Commit only reviewed scripts/pins; exact-head Control plane CI must pass.

### Task 3: Bind verified source and execute the real same-host measurement

**Files:** Create `scripts/verify_infercnv_measurement_snapshot.py`, `scripts/test_verify_infercnv_measurement_snapshot.py`, `.autopilot/oracles/infercnv-measurement-candidate-2026-09-26.json`; modify `.github/workflows/infercnv-oracle.yml`; record `.autopilot/state/infercnv-matched-performance-2026-09-26.md` and final accepted evidence receipt only if gates pass.

**Interfaces:**
- Snapshot verifier CLI accepts `--receipt`, `--run-json`, `--jobs-json`, `--snapshot-dir`, `--output-dir`; output directory must be new.
- Receipt binds repository/workflow, successful four-native run/head/attempt, four target names, snapshot name/archive SHA/manifest SHA/lock SHA and production-baseline manifest hashes. Native identity comes from Task 1 verified evidence, not a placeholder.
- Existing oracle workflow gains optional string `matched_snapshot` (empty default), only accepted when dataset is shipped and equal to the trusted receipt's selector.

- [ ] Write verifier negative tests first: wrong run/head/attempt/conclusion/workflow/target set, snapshot digest/manifest/lock mismatch, removed/extra source, changed production bytes, traversal/absolute/link/duplicate archive paths and preexisting destination. Verify expected failure, then implement using hashlib/tarfile/JSON/pathlib and explicit regular-file-only extraction; never trust archive extraction defaults.
- [ ] Controller creates candidate receipt from actual verified Task 1 run. Workflow fetches live run/jobs metadata under actions:read, validates receipt and snapshot before installing/building Rust, preserves evidence, sets Cargo/Rustup/target/TMP paths under RUNNER_TEMP and uses Rust 1.91. Ordinary empty-selector oracle flow remains unchanged. Validate all dispatch inputs before expensive installation.
- [ ] Build once with `cargo build --locked --release --features infercnv-measurement --bench cnv_matched --message-format=json`. Resolve the unique executable from compiler-artifact JSON, verify regular executable and hash it. Run existing synthetic/shipped generation and validators unchanged, then Task 2 R negative tests and actual smoke/seven pairs. Copy exact machine/compiler/environment/load/source/input/package information into measurement evidence. No additional task-owned workload runs concurrently on that host.
- [ ] Always verify all extracted source/lock bytes after trials, preserve both source checks and all raw results in the existing oracle artifact. Failure leaves no accepted summary or performance claim. Workflow uses fail-loud pipefail and preserves partial evidence in its always-upload step.
- [ ] Run controller tests, exact-head Control plane CI and fresh whole-change review before dispatch. Dispatch only after verifying main equals reviewed SHA; inspect the created run's head. Wait for actual R/measurement execution, download original artifact/API metadata/logs to external evidence, verify ZIP/API digests and every measurement path/output/pin. Compare original stdout with reported values; recompute summary independently from raw samples.
- [ ] If all gates pass, commit scoped performance results with hardware/version/input/flags/timing distribution/RSS provenance and claim limits. If samples overlap or fail, record actual failure without weakening correctness/timing criteria, then fix the concrete cause or advance an unblocked task. No automatic publication; full workflow/representative scaling remain separate gates.

## Controller self-review and handoff

The design's input, computational-state, source, metric and acceptance contracts are mapped to Tasks 1–3. Shared interface names and metric keys above are binding. The user explicitly delegated routine choices and requested parallel agents, so execution uses subagent-driven development without another approval prompt. Direct main, external-only paths and evidence preservation override worktree/cleanup defaults. Keep durable task/run state and review rulings in the new plan workspace plus tracked control-plane ledger.
