# First actual complete DESeq2 fitted-workflow oracle

## Current state

Run40561801963/job122392501549 is authoritatively live at exact source
89ece7f45e59517cf7631e9c5931a48c4a7760c1. Last checked step: Install release
dependencies and the exact source. Controller40561524398 at that head is
SUCCESS. Do not restart this run because an observation times out. No actual
fit/output acceptance is claimed before the source call and retained original
artifact are verified. The full30-product goal stays active.

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

Next inspect this exact run's terminal state; retain original API/ZIP/log and
verify source/witness/input/file identities plus the actual fitted outputs.
Fix a concrete driver/provisioning error if observed, without relabeling it as
a passed oracle. Then use the actual fitted state to shape the private native
dataset/analysis contract before numerical migration. No new target repo or
public foundation API exists yet; no placeholder product is published.
This real design does not positively test Cook's masking/replacement or all
general design/contrast classes. Keep the dossier's full release promise.
