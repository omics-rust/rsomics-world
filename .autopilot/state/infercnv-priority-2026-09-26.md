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

Path correction `debafe7b493f324859c5c01df6aa0856f638d79d` passed exact-head
Control plane run `36216214651`. Dispatched oracle run `36216239022` accepted
the workflow, installed R 4.6.1/JAGS and downloaded the exact source, then failed
before installing BiocManager: `--vanilla` bypassed setup-r's startup-profile
repository option, leaving `CRAN="@CRAN@"`. A remote R-only diagnostic reproduced
that unset option and verified that explicitly using the action's exported
`RSPM` resolves a concrete repository URL. The correction sets only that option;
oracle execution keeps `--vanilla`.

Failure evidence is retained under external fixtures:
`evidence/infercnv-2026-09-26/run-36216239022/`. The original artifact ZIP passes
CRC checks and matches GitHub's SHA-256
`e3e049ef861d6cae1dcd67a4ba51c6346857bcebfc80f2611a803e2c13cf7612`.
No real infercnv output was produced; the original source archive and install
failure log are preserved, not substituted with synthetic output.

Repository-option correction `2bf49e93404ee5fbbb4499e288ae9a6b7975c5c7`
passed Control plane run `36216379718`. Oracle run `36216425084` was dispatched
at that exact head. Dependency installation completed, but the exact source
could not load because the downloaded `igraph` binary requires absent
`libglpk.so.40`. The next bounded correction installs Ubuntu Noble's
`libglpk-dev` (which depends on `libglpk40`) and tests namespace loading for
every declared import before installing infercnv; installed package names
alone are insufficient readiness evidence. No oracle output was produced.
Its artifact is retained in `evidence/infercnv-2026-09-26/run-36216425084/`,
passes ZIP CRC checks, and matches GitHub SHA-256
`618282a8de7a7933ac6135d5f0475bce41b8d3acc34f966add9f40ff7a15a194`.
Both terminal execution runs also retain their full `run-logs.zip`, including
the system package and runtime setup logs. SHA-256 values in run order are
`e089ebb09204abd72beda1febd00a1dfc73d8d5160fafc6311a4087b13d1b45f`
and `d36d7f7821585c76f49f8b14dce3cbface413dc421bba5b03c8bdadc36aa2b1e`;
both pass ZIP CRC checks.

Namespace-readiness correction `38625af42b55b089678a387b79adf4cc8d4e8a28`
passed Control plane run `36216747712`; oracle run `36216764707` was dispatched
at that head and completed successfully. The controller verified original
ZIPs, extraction, manifest, source identity, and runtime metadata. Independent
numerical review recomputed 185,536 entries with maximum absolute difference
`8.881784197001252e-16`. The bounded pre-clustering oracle is now accepted;
see `infercnv-oracle-accepted-2026-09-26.md` and its explicit coverage limits.

Parallel source audit `infercnv-smoothing-audit-2026-09-26.md` was independently
reviewed. It records chromosome/window boundary semantics and an unmeasured
linear-time optimization hypothesis. The pinned source's existing example
has 10,338 expression rows and 184 cells (42 reference, 142 malignant), as
read directly from the retained count and annotation files. It is a possible
real-data regression fixture, not representative large-data performance
evidence; its explicitly example-only gene order must not be reused for new
biological datasets.

- [x] Read-only source, upstream, product, and storage audit.
- [x] Write design and oracle implementation plan.
- [x] Independent spec review: make profile/reference and creation parameters
      explicit, separate source pinning from environment reproducibility, and
      require meaningful signal contrasts without rejecting legitimate unchanged
      singleton-chromosome smoothing.
- [x] Implement and independently review the real-package harness.
- [x] Pass exact-head control-plane checks and run the remote oracle.
- [x] Inspect artifacts and record numerical/provenance evidence.
- [x] Begin native implementation against accepted stage-level goldens.

Native continuation: `docs/plans/2026-09-26-infercnv-native-core-plan.md` and
`infercnv-native-core-2026-09-26.md`. Test-first source preparation has started;
no native compatibility or implementation completion is claimed yet.
