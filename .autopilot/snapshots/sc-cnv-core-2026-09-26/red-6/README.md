# Raw-input reader test-first candidate: red-6

Baseline is accepted product commit `3715e55b`. Only `tests/cnv_input.rs` is
added: 21 raw-reader contract tests, before any production or dependency edit.
The intended red is Rust E0432 for absent input API exports; dependency or
environment failures would not establish this test-first gate.

Includes stable coordinate ties, position-only chromosome rank, post-reduction
inclusive thresholds, annotation validation before filtering, reference order,
strict discarded rows, native f64 conversion, compensated totals, decoded
record limits and numeric workspace limits. Gzip tests use fixed byte fixtures.
No native raw-reader acceptance, performance or full inferCNV claim is made.
