# inferCNV priority — 2026-09-26

Plan: `docs/plans/2026-09-26-infercnv-oracle-plan.md`.
Spec: `docs/plans/2026-09-26-infercnv-oracle-design.md`.

The user resumed work, prioritized inferCNV, and authorized parallel agents.
Three independent read-only audits covered upstream behavior, historical code,
and product/foundation ownership. All agree that the old approximation cannot
be merged as inferCNV and the accepted destination is `rsomics-sc cnv`.

## Execution decisions

- Keep the existing product boundary and avoid a speculative public matrix/HMM
  crate. Moving it would alter the published BAF/LRR contract and the planned
  single-cell boundary; the single-cell product has no existing public API.
- Start with a step-14 numerical oracle, then implement against those values.
  This is not a reduced definition of the complete inferCNV goal.
- Honor the user's delegated decisions instead of adding routine approval gates.
- Keep progress in tracked `.autopilot/state`, not disposable skill scratch.
- Local boot APFS is 97.81% used; do not compile/install locally. Remote hosted
  CI is the established validation path. No cleanup is authorized here.

## Preflight

| Task/interface | Check |
|---|---|
| Task 1 internal | R generator and Python checker use one explicit JSON/TSV contract; no Rust or public API claim |
| Task 1 → Task 2 | The workflow uploads its raw bundle even on failure; Task 2 validates the same checker entry and exact head |
| Task 2 internal | A green workflow is insufficient without artifact inspection and provenance verification |

## Preserved earlier work

- World starts at `57a0ca26dd8fa3bb31f5875ed72d21f1a7fca46a`, with inherited
  `.autopilot/state/vcf-naive-preflight-2026-09-09.md` edits and untracked `.csv`,
  `crlf.txt`, `floatreads.txt`, `ws.txt`. Do not stage them.
- VCF run `34299936099` is now terminal success on all four native jobs at
  `5e300ee1e0309beaf1cf783fc5415386f6183cb5`; its raw artifacts and the
  expected-red run below were preserved and independently inspected. Receipt:
  `vcf-final-run-receipts-2026-09-26.md`, committed as `d618c95`.
- Expected-red VCF run `34300381682` is terminal failure on all four native jobs
  at `57a0ca26dd8fa3bb31f5875ed72d21f1a7fca46a`; no new naive production fix
  is claimed. InferCNV now takes priority without deleting that evidence.

## Progress

Design/routing commit `9dbcc0e75161705c386cf852a77b769141911a82` was pushed.
Exact-head control-plane run `36215221498` passed. The baseline control-plane
suite had seven passing tests before adding the new oracle checker. The
new checker first failed because its module did not exist; its implementation
and independent review are in progress, not yet accepted upstream evidence.

The first harness candidate passed 25 control-plane tests, independently rerun
by the controller. A fresh review still found three important gaps: serialized
signal/profile contrasts were not checked; cell-group and chromosome/coordinate
identities could change while IDs stayed constant; and the fixture put gain and
loss in different observation groups instead of the declared combined case.
Fix round 1 adds negative tests and addresses all three before any accepted
oracle dispatch. JSON parameter completeness is included in the same bounded
fix. The candidate has not been committed, published, or called compatible.

The final scoped review accepted the harness for commit and remote execution.
Round 2 added explicit stage-12 gain/loss median checks, a counterexample test,
and the two missing explicit argument records. The controller independently
reran all 37 Python tests and the architecture validator successfully. R syntax
was checked with remote R 4.6.1, but real infercnv has not run. This acceptance
applies to the harness only, not upstream outputs or native compatibility.

Harness commit `367d54aa18dc6b0a26f8e2c6e114a76cebb07286` passed exact-head
Control plane run `36216128459`. GitHub rejected the oracle workflow itself
in run `36216127904` before creating any jobs: `runner.temp` is unavailable
in job-level `env` (lines 15–20). Ordinary YAML parsing did not validate Actions
context availability. The bounded correction initializes paths from
`RUNNER_TEMP` inside the first shell step and persists them through `GITHUB_ENV`,
matching the existing VCF workflow. No package installation or oracle output
exists from this failed validation run.

- [x] Read-only source, upstream, product, and storage audit.
- [x] Write design and oracle implementation plan.
- [x] Independent spec review: make profile/reference and creation parameters
      explicit, separate source pinning from environment reproducibility, and
      require meaningful signal contrasts without rejecting legitimate unchanged
      singleton-chromosome smoothing.
- [x] Implement and independently review the real-package harness.
- [ ] Pass exact-head control-plane checks and run the remote oracle.
- [ ] Inspect artifacts and record numerical/provenance evidence.
- [ ] Begin native implementation against accepted stage-level goldens.
