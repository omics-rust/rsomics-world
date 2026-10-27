# First actual complete DESeq2 fitted-workflow oracle

## Current state

Run40561801963/job122392501549 at exact source
89ece7f45e59517cf7631e9c5931a48c4a7760c1 is terminal SUCCESS and the bounded
complete actual-source witness is accepted. Controller40561524398 at that
head is also SUCCESS. Original API/26172835byte ZIP/log/176 files are retained
and API/head/size/digest/CRC/safe-path verified at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/deseq-first-workflow-2026-10-28/run-40561801963/`.
All55 manifest file identities pass. The production receipt is
`.autopilot/oracles/deseq-first-workflow-2026-10-28.json`.
Witness SHA8f2dc0a7b0377a22c2e1afda549003842577aa4dea24c9a675af5293bcb94bd7;
manifest423e585319836ceb0b7c54a47bb2a42431e97dff9929301e9273195ddf3270f8.
This accepts the actual source witness, not four-platform/performance or
biological acceptance. A new private native fitted workflow is now committed
locally at718f12a2815157755118dd435e65c76dd18565c8 and passes the complete
fixed witness at one/four workers on Mac. Its separate implementation/gate
state is deseq-native-fitted-workflow-2026-10-28.md. The full30-product goal
remains active and unchanged.

Independent raw-table checks confirm unfiltered14599x7 counts/identity join,
2240 all-zero rows,7x3 rank3 model and actual parametric fit. All12359 nonzero
genes have betaConv=TRUE; all-zero rows retain NA. pvalue NA2240/padj NA5800
include3560 independently filtered nonzero rows. At alpha0.1 the actual
filter threshold is3.897806930592444,1330 padj values are below0.1.
Forward, normalized-count and VST reload tables are byte-identical; independent
counts/size-factor division agrees exactly in double precision. Actual reverse
contrast LFC/stat signs reverse, other columns/NA state agree. Full
counts/mu/H/cooks, coefficients and row diagnostics retain all feature IDs.

Captured public-call conditions contain9 messages and zero warnings, but the
raw log has4 exporter-only DataFrame warnings about ignored check.names/
stringsAsFactors arguments. Do not claim globally zero warnings. Numerical
columns and exact reload evidence are unaffected. The installed DLL hash is
recorded and agrees with the inline installed-file manifest; DLL bytes are
not retained, so no independent offline DLL rehash is claimed. setup-r's
effective user library is under runner-temp/Library, still hosted temporary
storage; runtime.json records its actual path.

Plan: docs/plans/2026-10-28-deseq-first-workflow-oracle.md.
Source qualification: deseq-source-preflight-2026-10-28.md. Source, original
pasilla files and official current release remain DESeq2 1.52.0/Bioc3.23,
R4.6.1, archive SHA
8c91699286336350e66eec132ce6fdf5bb4af78e2a4d015a5a61224f62a95984.
Root independently rehashes the retained archive and both input files.

One bounded worker returns ownership of scripts/deseq_workflow_oracle.R;
root reads it completely and checks key definitions/exports/defaults against
the actual pinned source. The hosted wrapper uses the existing R4.6.1 setup
pattern, provisions only release Depends/Imports/LinkingTo plus digest/jsonlite,
installs the exact verified source, and keeps all project paths under
runner-temp. No local R build/install/invocation occurs. One compact routing/
shell-syntax control test and architecture validation pass locally; exact-head
controller acceptance is separate from the unexecuted real source workflow.

The driver uses one unfiltered real14599x7 pasilla dataset, explicit sample
identity mapping, design~type+condition, actual DESeq Wald and named results,
ordinary default Cook's cutoff omitted, normalized counts and fit-reusing
design-aware VST. It saves/reloads the full fit, checks original results and
exports, derives the reverse contrast without a new DESeq fit, and retains
full-precision tables/raw RDS/provenance/NA/convergence/filter state. Complete
witness/manifest are emitted only after the calls and reload checks finish.

The next private native plan is now
docs/plans/2026-10-28-deseq-native-fitted-workflow.md. It fixes the complete
dataset/fitted-analysis/results contract and numerical bounds before any
native fit. The target repo now exists as a real private checkpoint; no public
foundation API or placeholder product is introduced.

A bounded five-donor/source recheck selects team-owned size-factor/VST/general
GLM/dispersion kernels for refactor, not their CLI/parser/state models. Real
pasilla source state exposes active grid-fallback and72 dispersion-outlier
paths: raw MAP and final dispersion must remain separate. Legacy GLM misses
the natural-log ridge conversion; legacy Cook's uses two-group policy/p=2,
not the actual full-design grouping/p=3. Legacy results LOWESS selects the
wrong rejection-grid index17, while R selects16. The existing team-owned
voom/lowess.rs kernel, used with f=.2/nsteps=3/delta=.01*range, reproduces the
actual R rejection curve to2.73e-12 in an independent Python translation and
selects16; prefer this existing seed over a new implementation. This is a
source/algorithm selection result, not a Rust execution or native acceptance.
Use the existing rsomics-stats BH API with caller-owned NA handling; no new
speculative numerical foundation is planned.

This real design does not positively test Cook's masking/replacement or all
general design/contrast classes. Keep the dossier's full release promise.
