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
  `5e300ee1e0309beaf1cf783fc5415386f6183cb5`; new-turn artifact inspection has
  not been performed.
- Expected-red VCF run `34300381682` is terminal failure on all four native jobs
  at `57a0ca26dd8fa3bb31f5875ed72d21f1a7fca46a`; no new naive production fix
  is claimed. InferCNV now takes priority without deleting that evidence.

## Progress

- [x] Read-only source, upstream, product, and storage audit.
- [x] Write design and oracle implementation plan.
- [x] Independent spec review: make profile/reference and creation parameters
      explicit, separate source pinning from environment reproducibility, and
      require meaningful signal contrasts without rejecting legitimate unchanged
      singleton-chromosome smoothing.
- [ ] Implement and independently review the real-package harness.
- [ ] Pass exact-head control-plane checks and run the remote oracle.
- [ ] Inspect artifacts and record numerical/provenance evidence.
- [ ] Begin native implementation against accepted stage-level goldens.
