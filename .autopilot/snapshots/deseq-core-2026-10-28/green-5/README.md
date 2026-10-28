# Buffered fitted-bundle checkpoint

Owning clean localmainc9f99b70c191c6ed29d42c8d4cd49ed151b573b7 remains
private and publish=false. This changes only two lines in src/artifact.rs
relative to schema4/green4: named JSON save uses the existing published
common0.12.3 buffered transactional write_output path. Schema, mathematical
state, CLI, JSON bytes, Cargo/lock and public APIs are unchanged. The named
bundle requirement, JSON I/O error classification and atomic commit remain.

All12 tests, strict release all-target Clippy and formatting pass freshly in
the parent review. The published helper's explicit flush and transaction
implementation were read; no new foundation API or buffering wrapper is added.

Original full pasilla native old/new A/B uses one warmup plus three timed
pairs per worker setting. Median run wall8.26->0.41s at1 worker and7.57->0.22s
at4 workers; system time collapses to0.01s. All16 fitted bundles match exact
bytes within worker count, and original source/reload/export checks pass.
These Mac warm-cache observations diagnose the save bottleneck, not a formal
upstream compute/scaling claim. Full report:
/Volumes/KIOXIA/Developments/tmp/deseq-buffered-save-20261028.rjDl8a/REPORT.md.
Local executable SHA
f19563fc1cac02080cf7cbe91002d9d280d0af8318eedeeaa838376b64887d19.

Run the unchanged schema4 four-native complete-workflow gate on this snapshot.
After acceptance, formally remeasure this exact accepted binary, with the
original source/inputs/flags/bounds. Green3 performance and green4 native
results must not be relabelled as evidence for this later checkpoint. Local
trends/fallback, declared product scope and publication remain separate gates.
