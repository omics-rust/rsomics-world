# Private DESeq fitted-workflow candidate

Source is owning local main718f12a2815157755118dd435e65c76dd18565c8
from rsomics-deseq. The product remains publish=false with no public remote.
This snapshot is a real fitted-analysis checkpoint, not a release decision.

The complete original DESeq2 1.52 witness (14,599 genes, seven samples,
rank-three design) passes fixed numerical bounds at one/four workers on the
development Mac. Reloaded contrasts, normalized counts and fitted VST agree;
exports are deterministic across workers. Seven tests, strict Clippy and
formatting pass. A fresh whole-workflow review's mixed-sign contrast and JSON
I/O findings are fixed and the scoped follow-up passes.

Use the four-native deseq-candidate gate against the retained source witness,
without refitting R or changing bounds. Formula coding, supplied normalization,
positive Cook replacement, low-residual-df and nonparametric trend support,
broader compatibility and formal performance evidence remain release work.
Dispersion iteration paths are retained diagnostics, not bitwise acceptance.
