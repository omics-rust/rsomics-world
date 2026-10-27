# Private native DESeq2 first whole-workflow task

## Ownership and authority

Implement one coherent private checkpoint from
docs/plans/2026-10-28-deseq-native-fitted-workflow.md and the bulk-expression
product dossier. Own only
`/Volumes/KIOXIA/Documents/omics-rust/rsomics-deseq/`, a new empty main Git
repository. No other agent edits this product. The parent continues RNA/BAM.
Read this brief first. The full release promise is not narrowed to this first
real checkpoint. Do not create another public crate, LayerB dependency,
placeholder command, guessed result, or published package.

User process ruling: favor working integrated workflows and useful Rust
parallelism over per-helper exhaustive RED/review loops. Run focused checks
while developing and one complete workflow check before reporting. The parent
does the fresh whole-workflow review and Git/publication gates. Do not commit,
stage, push, delete, publish or spawn agents. Keep comments rare and consistent.
Use apply_patch for source edits. Send a concrete escalation if numerical/source
evidence is insufficient; never replace a difficult method with fake success.

## Storage and runtime

CARGO_HOME=/Volumes/KIOXIA/Developments/cargo-home;
CARGO_TARGET_DIR=/Volumes/KIOXIA/Developments/cargo-target;
TMPDIR=/Volumes/KIOXIA/Developments/tmp.
Task evidence and full report go in
`/Volumes/KIOXIA/Developments/tmp/deseq-native-first-workflow-20261028.yBqTXJ/`.
Before each build/test/dependency download verify all three resolved paths,
Cargo's metadata target_directory, free space on / and KIOXIA, and boot APFS
occupancy from APFSContainerSize/APFSContainerFree in diskutil info -plist /.
Stop compilation at80% or any boot-disk build path, but keep source work safe.
Last root check was77.15%, not permission to skip a fresh check.

Use existing Rust1.91 absolute tools under
`/Volumes/KIOXIA/Developments/rustup-home/toolchains/1.91.0-aarch64-apple-darwin/bin/`.
Set CARGO/RUSTC/RUSTDOC explicitly and CARGO_BUILD_JOBS=2. Avoid ambient
cargo-clippy/cargo-fmt rustup shims; existing absolute cargo-clippy+driver and
rustfmt work. No software installation or R execution. Build/source/data/scratch
stay external. Preserve every historical clone and fixture.

## Actual authority and first real input

Released DESeq2 1.52.0, commit16aeab6d6158bd8cbb8e98764c2f33399b1a3fd3,
source root
`/Volumes/KIOXIA/Developments/tmp/deseq-source-preflight-20261028.lyOhA0/DESeq2/`.
Read actual core.R, fitNbinomGLMs.R, results.R, vst.R and relevant C++ numerical
behavior. LGPL source is behavior reference, not permission to copy it into
an incorrectly licensed product; team-owned Rust seeds may be reused.

Accepted actual whole R source witness40561801963 is at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/deseq-first-workflow-2026-10-28/run-40561801963/artifact/workflow/`.
Use its complete inputs/counts.tsv, identity-joined metadata, exported named
model matrix and fit/results/transform tables. Discover exact file names there,
not guessed paths. Do not rerun R or reduce/filter the dataset. The source
retains14599 genes,7 samples,2240 all-zero rows, a7x3 rank3 design~type+condition,
and12359 nonzero converged Wald fits. The original fb-suffix sample mapping is
explicit in the witness; sample joins must use identity, never position.

## Required functioning checkpoint

Create a private publish=false product, not the old micro-crate skeleton.
Implement a typed dataset, one full-rank named numeric-design route, ratio
normalization, gene/trend/MAP dispersion, general-design Wald fit, named
contrast results, independent filtering/BH, normalized counts, fit-reusing
design-aware VST and versioned reusable fit bundle. CLI run/results/transform
must use current rsomics-help and rsomics-common consistently. Results and
transform reload a fit without repeating numerical fitting; exercise original
and reverse contrast and export equivalence. One explicit named matrix route
is the first private checkpoint; general metadata/formula support, positive
Cook's replacement and other first-release cases remain required before public
release, not advertised placeholder commands now.

Counts are checked nonnegative integers with unique gene/sample IDs. Design
is finite, named, full-rank, identity-joined and n>p for dispersion. Contrasts
must explicitly resolve against named coefficients; never silently zero an
unknown or missing coefficient. Preserve full covariance or sufficient fit
state for no-refit general contrasts, raw versus floored means, leverage,
convergence, all-zero/NA reasons, original/replacement counts, filter decisions,
raw MAP versus final dispersions and provenance. A failed solve is an error or
implemented upstream fallback, not identity covariance or zero coefficients.
Use deterministic per-gene Rayon work with an explicit worker count.

## Selected reuse and known pitfalls

All seeds below are team-owned clones under /Volumes/KIOXIA/Documents/omics-rust:

- rsomics-deseq-sizefactors: reuse the ratio/median math, not parser/CLI.
- rsomics-deseq-estimate-dispersions: refactor base statistics, rough start,
  Cox-Reid log posterior/gradient, optimizer/grid and parametric-trend kernels.
- rsomics-deseq-lrt: general-design GLM skeleton and the correct gene_disp
  grid-fallback criterion; no categorical-only design/parser reuse.
- rsomics-deseq-results: numerical/fixture asset only, not its two-coefficient
  fit, condition-only Cook's grouping, hardcoded p=2 or approximate LOWESS.
- rsomics-deseq-vst/src/vst.rs::transform: reconnect math to the fitted trend;
  do not run its blind refit path.
- rsomics-voom/src/lowess.rs lowess/lowest: existing team-owned correct seed;
  filter uses f=.2,nsteps=3,delta=.01*range. Do not use lowess_sorted's voom
  policy or Trend interpolation. An independent source-level check selects
  actual source grid16; old results LOWESS incorrectly selects17.
- Existing rsomics-stats bh_adjust is finite-only; gather/scatter around the
  product's NA mask instead of copying another public BH routine. Verify the
  registry/source API before selecting its version; do not modify foundations.

Actual1.52 uses natural-log ridge1e-6/log(2)^2 and augmented weighted QR.
The legacy1e-6 natural-log constant is wrong. Preserve raw versus floored mu,
full covariance/hat and genuine optimizer/convergence behavior. Gene-disp
iter==1 and iter>=100 require the actual grid-fallback rules; on this input
48 iter1 and198 iter100 genes exceed the grid gate, so it is not theoretical.
There are72 dispersion outliers; raw MAP differs from final dispersion on all
72 and must not be overwritten. Full-design cells1/2/2/2 select source robust
trimmedVariance, not condition-group trimmedCellVariance; p=3. MaxCooks is all
NA on this input but the finite Cook's matrix still needs correct computation.

## Integrated comparison and reporting

Use abs(native-source)<=absolute+relative*abs(source), with bounds fixed before
the first native fit: size factors/normalized counts1e-10+1e-10*abs(source),
dispersions1e-6+1e-4*, coefficients/SE/Wald1e-4+1e-5*, p/padj1e-7+1e-4*,
fitted means1e-6+1e-5*, leverage/Cook's1e-6+1e-4*, VST1e-5+1e-6*.
Do not widen them after a mismatch. Exact IDs/order/missing states/outlier/filter/discovery decisions;
source filterThreshold3.897806930592444,padj NA5800,1330 padj<.1. Keep failures
and all outputs. Avoid a new collection of per-helper R oracle jobs.

Retain executable/source/dependency/command identities and comparison results
in the task evidence directory. Write REPORT.md there with what works, files,
commands/results and concrete remaining issues, plus any deviations. Report
back briefly with DONE/DONE_WITH_CONCERNS/NEEDS_CONTEXT and the report path.
Never call the product release-ready from this checkpoint alone. Do not modify
the control plane or unrelated products; the parent owns durable goal state.
