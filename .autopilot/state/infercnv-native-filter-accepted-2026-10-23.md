# Native reference-gene filtering accepted

Actual terminal run39815910744/attempt1 succeeds at world
4316e07f70507fad2341bc1cc573dcf8928840e6. All four Linux/macOS x86_64/aarch64
jobs pass debug/release on Rust1.91. Each profile passes99 tests across ten
targets, including11 filter contracts. Actual totals are52 routes/20 active/
4 empty-outlier/6 zero-selected/152 composed trees/107331 distance pairs.
Existing automated source/oracle admission and postchecks pass; Linux
formatting and strict Clippy pass.

Frozen green17 archive6a7afbc1cbe373e38ce2994e044b4e4c1618c9f72970953fec562b4d1aeb9fff,
manifestdaa6f6878178ac86dc8e06a29c6c03156874f361565d50d08e43cb0ceeab697a,
162 source files. Root stages the exact verified archived filter.rs and
distance.rs bytes, excluding concurrent integration work, and commits them
as productbe35e65. Tests/fixtures/lock are unchanged from the actual RED.
The private product still has no remote and remains publish=false.

Native jobs: Linuxx86119954344673; Linuxarm119954345158;
macOSx86119954345139; macOSarm119954345043.
Original terminal APIs, four job logs, whole-run logs ZIP and collection
summary remain external at evidence/infercnv-native-filter-2026-10-23/
run-39815910744/. Whole ZIP SHA
18f386b507239e4dbe96f879c3076ba18483be69e77e3eb456c5756c0e239e55.
The existing automated checks and literal native totals suffice; no second
per-file/artifact admission campaign is introduced.

Scope: filtering from the original frozen R-state matrices, followed by the
accepted distance/Ward kernels. This does not establish the integrated
samples owner/CLI, current-Rust-to-R end-to-end trees, arbitrary-input R
precision, speed, cohort/biological validity or publication readiness.
Continue the actual raw-to-samples CLI candidate in green18 and measure its
whole-slice performance after functional validation.
