# Raw-reader implementation and lock resolution candidate: green-8

Accepted baseline product is `3715e55b`. Adds product-local raw TSV/gzip
creation, 25 focused input tests and one raw-to-checkpoint external test.
Production numerical core remains unchanged. Formatting used standalone
Rust 1.91 rustfmt; no controller compilation or dependency resolution occurred.

This snapshot intentionally retains the baseline Cargo.lock. A Linux-only run
with expected_red=true and resolve_input_lock=true resolves and verifies only
the flate2 1.1.9 Rust gzip dependency expansion. The expected_red selector is
used solely for that workflow's single-native dependency-resolution gate; no
test failure is required or claimed. The genuine absent-input-API red is
already preserved in run 36688501838 (red-6).

The returned generated lock must be verified and copied before a new immutable
four-native green snapshot. This intermediate candidate is not accepted,
released, benchmarked or evidence of complete inferCNV.
