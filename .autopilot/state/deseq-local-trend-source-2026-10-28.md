# Local-trend complete source witness

The four-case source-only driver/workflow is implemented and parent-reviewed;
it has not executed R locally or hosted yet. Plan:
docs/plans/2026-10-28-deseq-local-trend-source.md. This is not a native
implementation, dependency adoption, measured advantage or publication.

Reuse existing private source/design/table/state helpers, pin DESeq2 1.52.0
and official locfit1.5-9.12/R4.6.1/Bioc3.23, and install exact source archives
only on the hosted runner. Main read all driver/workflow source plus actual
upstream formula/default/model/prediction/VST code. Captured local model is
full saved state, not training-answer lookups. Unnamed/empty locfit vectors
use positional labels safely in portable exports; full RDS stays unchanged.

Freeze all four inputs/RNG/design/contrasts/queries/options before any fit:
full pasilla explicit local, one fixed increasing-dispersion parametric request,
same synthetic counts with NF only, and NF plus deliberately different SF.
Actual fallback needs both source message and actual local fitType. If not
triggered, keep uncovered/non-green; do not retune input or seed. Preserve
warnings and nonfinite state. Public full fitted VST is authoritative; saved
local trend, integration/FMM state and SF/NF priority traces must reproduce it.

Two fresh R processes separate fit and reload. Reload must derive all saved
contrasts/trend/normalized/VST outputs without input regeneration or full/trend
refitting. Full input/package/model hashes and failed-case files are retained.
The workflow always uploads evidence and propagates pipeline failure including
tee/log-write errors, rather than accepting only the source process's status.

Parent's two compact routing/Bash tests pass, including fit/reload/log failures;
they use stand-ins, not R. Full495 controller tests pass with two existing
skips, under fresh external-path/target/APFS guard (about77.24%). No per-helper
oracle matrix or new outlier×local combination is added. A transient agent
capacity failure left completed files; parent took over review and reran the
checks. It is not a source or native outcome.

Next: commit, exact-head controller CI, then one hosted source run with
unconditional original evidence retention. Native locfit candidate persistence
remains unselected and must not be smuggled into this source-only gate.
