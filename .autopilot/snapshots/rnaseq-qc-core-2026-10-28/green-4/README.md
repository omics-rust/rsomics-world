# Complete shared RNA-seq QC report, test lint correction

This source changes only two test exon-vector initializations from green3.
Clippy1.91 also rejects the singleton range array inside Vec::from, not just
vec syntax; std::iter::once(range).collect() states the intended single exon
without a singleton range array or lint suppression. Production source and
Cargo.lock are unchanged from green3.

Run40544150761 compiles and runs the full five-metric native report. Its
completed Linux x86_64/aarch64 jobs pass all eight debug/release tests and
all four actual single/paired1/4-worker report comparisons; the full gate
still fails the test lint. This is not four-native release acceptance.
Use normal four-native mode here with the imported fixed lock; do not rerun
dependency resolution or unrelated helper campaigns. publish=false and
representative performance/high-depth limitations remain as documented.
