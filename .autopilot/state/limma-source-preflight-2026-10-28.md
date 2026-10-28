# Current limma count-workflow source preflight

Bounded read-only preflight is complete; source-only witness is next, not a
native product or publication. Plan:
docs/plans/2026-10-28-limma-first-workflow-oracle.md.

Actual current authority is edgeR::voomLmFit -> limma::contrasts.fit -> eBayes
-> topTable, with exact edgeR4.10.7/limma3.68.5/Bioc3.23/R4.6.1 pins already
qualified. Source location does not authorize Layer B product dependencies.
The first raw-pasilla count workflow has block=NULL and sample.weights=FALSE;
no donor or pairing is inferred from sequencing layout metadata.

Source inspection confirms structural-zero correction changes sigma/df only,
df-weighted LOWESS/actual adaptive span, auto moderation based on equality of
positive df, and approximate weighted nonorthogonal contrast SE. Full base fit
must survive reload; contrast objects cannot supply fitted observations.
Targets raw libraries, effective TMM libraries and actual voom logCPM are
different states. Legitimate NA/Inf shapes must not become fabricated success
or a generic finite-everything failure. No actual source outcomes are known.

Old voom/quality-weight/ebayes implementations are numerical and fixture seeds
with incomplete state, not target skeletons or grounds to revive micro-crates.
Filter/TMM, solves and moderation arithmetic have named candidate consumers,
but no public API decision is made before actual consumer-side contracts.

Detailed source/asset line index and preflight record:
/Volumes/KIOXIA/Developments/tmp/limma-first-workflow-preflight-20261028.TXxtak/report.md,
SHA9f9754684202736c1b0fe3f9daa368a7bcd3e87cac56aca4a2e004ea5db331eb.
Parent has read the full report and actual voomLmFit/contrasts/eBayes source.
No local R, tests, native repository, dependency adoption or publication occurs
in this preflight. Future source qualification remains separate from native
four-platform and complete-workflow performance gates.

The bounded source driver/hosted workflow is now implemented and parent-read.
It reuses private DESeq/edgeR source/hash/table helpers, preserves actual
archive member case, and freezes raw input/mapping/design/coding/defaults/
contrasts before fitting. Full base/contrast/moderated states and observations,
trend, library and model exports survive saved-state checks. Only legitimate
contrasts/moderation/exports repeat on reload, never a voom/linear-model refit.
Data-independent formula environments avoid transient execution identity.

One compact routing/Bash test passes; parent's full493-test control suite
passes with two existing skips, and architecture validation passes. No local
R parse/run/install or actual source outcome is claimed. Hosted source/body/
fit/reload/coverage qualification is next after exact-head controller CI.
Handoff:
/Volumes/KIOXIA/Developments/tmp/limma-source-oracle-handoff-20261028.hnsmZV/handoff.md.
