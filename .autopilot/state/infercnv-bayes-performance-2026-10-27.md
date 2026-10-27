# Whole raw grouped-i6 Bayesian performance

Plan: docs/plans/2026-10-27-infercnv-whole-bayes-performance.md.
Native functional acceptance and binary identities are already retained in
infercnv-bayes-accepted-2026-10-27.md; do not rerun that completed gate.

Worldd7d233d449a0057a6560760d37a2a819b59fd609 passes exact-head controller
40514664136. Actual same-host performance40514973287 was dispatched once
at2026-10-27 13:46UTC on that source, using the retained Linux x86_64 binary
from40510071393/artifact12737096223. Follow it; do not duplicate it when the
turn/source head changes. Complete raw R Create+run19 is timed, not the
earlier stage17 replay. R1-thread public-call time excludes package startup;
native1/4 includes its entire CLI and JSON. Both use the same available
first-four CPU affinity and BLAS/OMP1, with actual machine/load provenance.
One warm R plus three measured R calls interleave with one warm and five
measured native worker pairs. Original and adjusted reports must match the
accepted stable/structural contract; only the existing three source-only
state3/4 ambiguities are allowed. Failed/partial trials cannot become samples.

The isolated R driver has one critical parameter/output-path test passing
on existing4090 R4.6.1, not installed-infercnv evidence. Root reads its
complete code and test. Python comparison/summary and actual Bash selector
critical tests pass; complete controller461 tests57.028s, two existing native
skips, architecture and whitespace checks pass. The speed decision requires
the slowest native1 process to beat the fastest measured R public call;
startup overhead cannot create the advantage. Memory is process peak RSS,
not aggregate fork RSS or a hard bound. No performance result is yet claimed.

Boot APFS91.74% still forbids local compile/install/dependency download.
Hosted runner-temp and external fixture/scratch paths are used throughout.
Ward choice holds only Ward policy. Default Leiden's honest single-partition
limit and next k5 witness remain independent, as does the full active30-family
goal. Product42988589 is committed/clean, publish=false and not released.
