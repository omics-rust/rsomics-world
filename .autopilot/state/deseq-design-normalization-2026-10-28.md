# General design and normalization execution

Plan: docs/plans/2026-10-28-deseq-design-normalization.md. The schema1/green1
checkpoint is independently accepted on all four native classes at run
40589627680; its source/binaries/evidence remain immutable and private.

Product implementation owns only rsomics-deseq, with no Git mutations or
public foundation changes. New schema2 separates raw model-matrix CLI names
from saved DESeq resultsNames mapping; formula provenance includes explicit
types/levels/reference, terms and coding. Normalization has one typed owner:
ratio SF, supplied SF, NF-only or SF+NF, preserving both supplied states but
using NF for counts/offsets. The NF moments initializer follows the actual
objectNZ column-mean policy. Actual Inf replacement opt-out and formula-only
two-level Cook exemption are implemented; positive replacement remains held.

Current new code compiles,9 compact tests/strict Clippy pass, and the unchanged
full pasilla witness passes once at each worker count. New general compatibility
is not yet accepted; an independently implemented actual source driver is
still being prepared. Product report:
/Volumes/KIOXIA/Developments/tmp/deseq-design-normalization-20261028.lzbwBH/.

The source driver owns only new world R script/hosted wrapper/routing test.
It freezes seven general workflows on1,200genes×18samples, plus eight Cook
workflows on14sample7+7 and its10sample7+3 subset. All general cases use
minReplicatesForReplace=Inf, not an unrecorded mismatch with native settings.
Inputs/generator/RNG/case inventory are sealed before the first actual source
fit. Positive branches count only when outputs demonstrate them; an uncovered
branch is not accepted by changing a receipt. No local R install/execution.

Next: read the complete actual source driver, run its bounded hosted source
workflow, compare complete native general workflows at1/4 workers, perform one
integrated review and freeze a fresh four-native candidate when stable. Keep
publish=false and preserve every earlier failed/successful witness. Continue
the same coherent lifecycle toward Cook refit/trends/low-df, not new helpers.
