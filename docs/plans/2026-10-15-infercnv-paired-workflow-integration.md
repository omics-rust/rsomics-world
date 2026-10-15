# Real paired ingestion workflow integration

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans.

**Goal:** Add a distinct, explicit raw-ingestion measurement selector without
changing existing prepared, witness or installed-contract route meanings.

**Architecture:** The installed-contract route remains the prerequisite. A
new optional `ingestion_snapshot` string selects only the already accepted
green14 native source; the offline paired owner consumes original API/ZIP,
source/build records and same-run untimed R profile. Existing receipt meanings
stay immutable, and every failure retains evidence through always-upload.

**Tech Stack:** Existing GitHub Actions, Bash, stdlib controller tests and
Rust1.91.0/native Linuxx86_64. No local builds/R/install/download.

**Spec:** `2026-09-30-infercnv-raw-ingestion-measurement-design.md` and
`2026-10-15-infercnv-paired-ingestion-owner.md`.

## Global Constraints

- Root owns `.github/workflows/infercnv-oracle.yml`, new
  `scripts/test_infercnv_ingestion_workflow.py`, this plan, the existing
  package-contract plan's complete workflow copy and tracked state ledger.
- Do not edit the concurrently delegated paired owner/test. Wait for its
  completed spec/code review before freezing their literal workflow pins.
- Direct main/scoped commits; no PR/Co-Authored-By/publication/deletion.
- All local tests stdlib-only `-B`, external TMPDIR/Cargo paths after APFS/free
  space guard. All real builds/package execution occur in native CI only.
- After push verify exact-head controller CI, then dispatch once with the
  fixed selection. No retries/dropped failed campaign or threshold changes.

## Review Focus

1. Selector combinations: raw requires shipped, contracts=true, matched empty
   and samples=false; unknown snapshots reject before downloads/builds.
2. Default/prepared/old-contract routes: new steps skip when string is empty;
   old step bodies and old receipts stay byte-identical except selector gate.
3. Cargo identity: the actual optimized bench JSONL and physical toolchain
   executable, not a copied binary or rustup proxy, reach the offline owner.
4. Evidence failure: always-source/code/executable checks and upload remain
   effective even if owner preparation or sample32 fails.
5. Drift and documentation: literal reviewed transitive pins verified before
   execution and after; embedded workflow copy exactly matches real YAML.

## Task 1: Selector and route regression tests

- [ ] Add stdlib tests reading the actual workflow. Extract indented Bash
  run blocks and require `bash -n` for each. Execute only the selector block
  with fake scalar environment/current small real receipts and external
  temporary GITHUB_ENV/EVIDENCE_DIR; never execute install/build blocks.
- [ ] Assert the new input is optional string/default empty, environment
  wiring exists, all added execution steps have nonempty-selector guards,
  the fixed snapshot receipt/verifier/build target/CLI are the raw ones, and
  always-postchecks/upload remain present. Full embedded YAML equals actual.
- [ ] Retain RED on the absent selector before changing the workflow. Run the
  full dataset×prepared×clustering×contracts×raw-empty/green14/unknown matrix:

```python
old_valid = ((not contracts or dataset == 'shipped' and not matched and not clustering)
             and (not clustering or dataset == 'shipped')
             and (not matched or dataset == 'shipped' and matched == prepared_name))
valid = old_valid and (not raw or raw == 'green-14' and dataset == 'shipped'
                      and contracts and not matched and not clustering)
```

## Task 2: Exact source/build/context/owner integration

- [ ] Add optional input and environment:

```yaml
ingestion_snapshot:
  description: Optional accepted raw-ingestion snapshot; requires installed contracts
  type: string
  required: false
  default: ''
```

- [ ] Extend the selector early gate, requiring raw snapshot_name from
  `.autopilot/oracles/infercnv-ingestion-trial-candidate-2026-10-15.json`
  after checking that receipt's frozen hash. Preserve existing input behavior.
- [ ] Add guarded raw source acquisition: original native run/attempt jobs
  API replies into EVIDENCE_DIR, copy receipt, invoke the accepted raw snapshot
  verifier with green14 snapshot and new RUST_SOURCE, save source-before.
  No network calls inside the paired owner.
- [ ] Freeze literal SHA256 values for reviewed owner/import closure and its
  four accepted receipts in a dedicated workflow heredoc. Read that literal
  mapping into the context only after `sha256sum --check`. Do not generate
  authenticity pins from arbitrary checked-out bytes. Existing12 contract
  pins remain unchanged; retain controller HEAD and relevant source/scripts.
- [ ] After successful untimed installed-package contracts, install Rust1.91.0
  in runner-temp and build only the authenticated exact raw bench:

```sh
cargo +1.91.0 build --locked --release --features infercnv-measurement \
  --bench cnv_ingestion_matched --message-format=json
```

  Retain raw JSONL/stderr and compiler/cargo identity; get the physical rustc
  using `rustup which --toolchain 1.91.0 rustc`. Verify Cargo's resolved target
  dir and all runner-temp storage variables before building. No path deps or
  production edits. The owner selects the sole actual executable from JSONL.
- [ ] Generate the exact closed context defined in the owner plan with
  absolute originalAPI/ZIP/extract/native/R/Cargo/toolchain paths,
  `contract_root=$EVIDENCE_DIR/ingestion-package/fresh-plain`, reviewed code
  pins, actual selected Python/Rscript/R executable and GNUtime. Record
  actual machine/CPU/load/cargo version/build env outside factory timers.
- [ ] Invoke the real owner exactly once:

```sh
python3 -B scripts/measure_infercnv_ingestion.py \
  --context "$EVIDENCE_DIR/ingestion-context.json" \
  --output-dir "$EVIDENCE_DIR/ingestion-campaign"
```

  Preserve exit status through pipefail/tee. A complete slower campaign is
  evidence with a failed speed gate, not a reason to change the metric.
- [ ] Add always guarded raw source/code/executable postchecks and record
  failures even if build/context was never created. Existing always-upload
  retains complete evidence. Sync the complete workflow copy in the package
  plan using an explicit mechanical replacement, not independent editing.
- [ ] Run selector/Bash/copy tests, full stdlib controller suite once, scoped
  independent review including owner integration and production hot path.
  Commit scoped files, push, verify exact-head controller CI before dispatch.

## Task 3: Real campaign acceptance, not implementation self-report

- [ ] Dispatch dataset=shipped, ingestion_contracts=true,
  ingestion_snapshot=green-14, matched_snapshot empty, samples_clustering=false.
- [ ] Fetch original exact-head run/jobs/artifact/listing/full logs externally;
  verify attempt/repository/workflow/required unique successful steps,
  artifact size/digest/CRC/safe unique regular paths before extraction.
- [ ] Independently audit actual32serial processes,64exact f64/identity/map
  exports, closed metadata, source/input/code/build/executable/Python/R proofs,
  pilots and28 measured samples. Recompute both strict range decisions and
  distributions from raw metrics. No self-issued summary substitutes for this.
- [ ] Record bounded result/failed gate and remaining full-infercnv gaps in
  durable tracked ledgers. Never claim representative cohorts, cold-cache,
  original implicit-header dialect, factory peak memory, whole-workflow speed
  or release readiness from this single canonical warm-ingestion experiment.
