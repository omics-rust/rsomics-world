# Cook replacement/refit execution

Plan: docs/plans/2026-10-28-deseq-cook-refit.md. Owning localmain8bb14ad
implements schema3 with original assays/diagnostics preserved and typed
replacement eligibility, flags, counts, zero and nonzero refit states. Reuse
the existing dispersion/GLM/trend/prior/normalization kernels and one pool;
no new Layer A API, dependency or product boundary is introduced.

All15 workflows of unchanged actual-source40592944209 pass at1/4 workers,
2,220,279 numeric comparisons per worker count plus exact decisions/state and
byte-identical exports. Original pasilla and reload contracts pass. Seven+seven
default cases have3 flags/2 refits/1 zero/3 changed entries; seven+three cases
have4 flags/3 refits/1 zero/2 changed entries. Flags/refits need not imply count
changes. One actual refit fallback retains its real optimizer status.

Source iteration counts remain recorded diagnostics, not a new exact gate.
Exploratory exact-count failures are retained; statistical/deviance/convergence
and decision gates were not weakened. Primary-source correction before code
established that both SF/NF truncate(mean*effective factor); positive SF plus
ordinary NF evidence is not an independent NF-outlier oracle claim.

Fresh whole-feature review found one P2: originally all-zero max-Cook in a
partial7+1/7+2 design must become0 when selected columns are all eligible.
The fix maps eligible columns to0 before propagating selected ineligible NAs.
Real7+2 RED-to-GREEN and7+3 NA control pass; no-refit/global branches remain.
Scoped rereview passes. Parent freshly reran all11 release tests, strict
all-target Clippy and formatting successfully, with APFS77.17%/KIOXIA239GiB.

Product report and retained failed/successful runs:
/Volumes/KIOXIA/Developments/tmp/deseq-cook-refit-20261028.BdlNQJ/REPORT.md.
Final local executable7ff51d2ff98cfc8ff98f72e9e0e6fe713228877373a29470326d1b19384dc6b7.
Schema3 green3 freezes27 committed files, archive
66cbf540ec369169da7df9dccdf2dda516ee6c6276a670d3e864e58e028d7749,
manifestdd67c410ff5d902272f0481b4db58c2a3b7a883ae54ab82634d9872f1e6c8aab.
The existing snapshot verifier confirms the exact27 regular-file inventory.
The updated wrapper requires schema3 and all15 cases, including positive Cook
replacement, with both unchanged source receipts and1/4-worker determinism.
Its two compact integration/Bash checks pass. Exact controller40602332374
atc598ffaf0c44c6b79b1c362ad6d75b4effd91907 passes; four-native40602456359
is terminal SUCCESS on all four classes. Original APIs/ZIPs/logs and exact27
owning source files/lock/binaries, both complete source manifests and all15
plus pasilla reports1/4 are retained and independently verified. Positive
default-vs-disabled original dataset/genes/dispersions remain identical, with
refits kept separately. Authoritative verification.json SHA is
79c6462118f6cb6331fd77b6a01a5ec934805b966f68b6b0e2e1fac0474493f6 under
/Volumes/Zane's HDD/rsomics-fixtures/evidence/deseq-native-2026-10-28/run-40602456359/.
This accepts the bounded schema3 workflow, not representative performance or
complete DESeq2 scope. The next mean/low-df implementation has a separate plan.
Mean/local trends, low-df prior and formal whole-workflow performance keep
publish=false; continue the full portfolio in parallel with source-qualified
edgeR work, not a speculative shared statistical API.
