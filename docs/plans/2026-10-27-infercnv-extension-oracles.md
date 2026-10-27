# inferCNV subcluster and Bayesian workflow implementation plan

> Execute inline with superpowers:executing-plans and one isolated Bayesian
> witness worker. User authorization supersedes per-helper reviews and plan
> approval checkpoints. One integrated review follows functioning evidence.

**Goal:** Capture actual Ward-subcluster i6 workflows and actual Bayesian
post-HMM outputs, then use their contracts to extend the native workflow.

**Architecture:** Reuse installed infercnv1.28.0/R4.6.1, the existing
shipped-input bundle, and verified real grouped-i6 replay objects. Keep the
accepted grouped implementation unchanged while the new membership and
posterior contracts are measured. No new public foundation or placeholder CLI.

**Tech stack:** Existing R/Bioconductor/JAGS oracle, Python controller tests,
GitHub Actions; subsequent native implementation remains Rust1.91.

**Spec:** AGENTS.md and
`.autopilot/state/infercnv-next-workflow-contract-2026-10-27.md`.

## Global constraints

- Source commit b421d9405c97a309b081ef86d455e976df93eae4, archive
  b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207.
- Native grouped HMM code40c3e446/docs7853e922 and its receipts remain intact.
- Main branch, one concern per commit, no PR, no Co-Authored-By.
- Cargo/TMP/fixtures stay on the named external disks. Boot APFS91.7%:
  no local builds, package installations or dependency downloads.
- No workspace deletion, repository revival, publication or automation.
- New oracle selection is opt-in/default-false; preserve old selectors.
- Do not equate Ward profiles with default Leiden/PCA or this work with full
  inferCNV/portfolio completion.

## Review focus

- Entry/exit mutation: retain the original expression object and actual states.
- Tree labels versus returned members: measure both, not a copied partition.
- Calibrated raw SD versus group-fitted SD: Bayesian uses the former.
- JAGS random streams: retain versions/protocol and independent repeats;
  R set.seed does not establish JAGS bitwise reproducibility.
- Filter/reassignment: retain original/filtered states and actual reports,
  including removed region IDs; do not replace probability estimation.

## Task 1: Actual Ward-subcluster raw workflows

Files: create scripts/infercnv_subclusters_workflow.R and its small R test;
root owns oracle workflow selector/tests and durable execution state.

Consumes: shipped bundle with load_bundle/export_state/capture_call from
existing oracle helpers. Driver CLI: SHIPPED OUTPUT.

Produces: subcluster-workflows/witness.json plus expression14, states17,
transformed spike, calibration, actual parent Ward trees/subcluster members,
four original reports and actual replay RDS. Profiles are grouped pheight0.5
and qnorm0.1 on the real full input, seed261027, HMMi6/no Bayesian/no plots.
Capture the actual subcluster predictor, not the whole-sample predictor.

- [ ] Write a critical record/entry-snapshot test and observe missing behavior.
- [ ] Implement the driver, reuse parsers/serialization helpers, and record
  actual cutree labels alongside the package-returned groups. Include one
  installed three-cell Ward characterization with nontrivial leaf order.
- [ ] Parse/check the script in existing R4.6.1, run the combined installed
  campaign, retain original API/logs/artifact and inspect its real outputs.
- [ ] Resolve any demonstrated membership/intent contradiction before wiring
  a native public subcluster policy. An affected policy can wait while the
  independent Bayesian workflow advances.

## Task 2: Actual Bayesian post-HMM workflow (isolated worker)

Worker owns only scripts/infercnv_bayes_workflow.R and its critical R test.
Driver CLI: HMM_WORKFLOWS OUTPUT. Root owns acquisition/workflow/state.

Consumes: verified run40477088139 grouped full-case actual input/result RDS
and four stage17 reports; installed infercnv/JAGS and original BUGS model.

Produces: bayes-workflows/witness.json, immutable original and actual final
states, original/filtered MCMC objects, region/cell posterior tables, calibrated
means/precisions, removed/reassigned region IDs, actual adjusted reports,
versions/model identities and timing. Use two full-case JAGS repeats, actual
six-chain500-adapt/200-update/1000-sample protocol, no algorithm/source patch.
R seed remains261027; explicitly distinguish it from JAGS initialization.

- [ ] Write a critical posterior/filter record test before implementation.
- [ ] Implement direct public inferCNVBayesNet/filterHighPNormals and actual
  report adjustment over the retained real objects, preserving input evidence.
- [ ] Root integrates default-false campaign routing and verifies both output
  families in one installed environment. No native Bayesian acceptance follows
  merely from valid R output or stable serialization.
- [ ] Inspect actual posterior variability and filtering before finalizing
  native statistical comparison or a public Bayesian configuration.

## Execution rulings

The user chose unattended execution and consolidated tests/review; no new
approval or per-helper RED/native campaign is required. Inline root plus one
isolated oracle worker implements independent file sets; no shared writes.
Use the existing docs/plans and .autopilot/state layout, direct main and
existing external scratch, not generic worktree/cleanup scaffolding. Cost of
these choices is one combined integration check and whole-feature review.

Do not implement ambiguous Ward membership or invent JAGS seed equivalence.
The native implementation, four-platform real-workflow comparison, relevant
performance and publication gates remain subsequent required work, not waived
by completion of this evidence task.
