# Private prepared-input measurement candidate: green-6

This candidate adds a private feature-gated Linux x86_64 measurement executable,
portable support tests and a single-stage fixture reader to accepted product
bdcc8a3a. The executable prepares only stage 1, performs one discarded warm-up
and one timed cnv::run call, checks immutable input/final results after timing,
and writes region metrics plus final TSVs outside the timed call. Other target
classes compile an explicit unsupported-execution path.

Production source and original fixture bytes remain unchanged. Cargo.lock is
the reviewed remote resolution from expected-red run 36223760183, adding only
nix 0.29.0 and cfg_aliases 0.2.2 with the root dev-dependency edge. Its SHA256 is
e469fc5d773632a5893adfd3156ea816204b2208a3090fbd432fc37112d11f46.

Four-native debug/release conformance, named measurement tests, compile-only
bench checks, formatting and strict all-feature Clippy remain pending for this
snapshot. Source review runs independently. This is not accepted performance,
an end-to-end workflow, a product CLI or a publication decision.

Preserve prior snapshots and all evidence. The archive and source manifest
identify the exact candidate; a later trusted receipt may be created only
after genuine four-native verification.
