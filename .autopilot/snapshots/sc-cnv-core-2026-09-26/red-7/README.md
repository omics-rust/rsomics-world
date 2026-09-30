# Exact private creation support test-first candidate: red-7

Baseline is accepted product `e62fc960aeeb9dc543298885213cda6a57196fe9`.
Only Cargo.toml registers the new opt-in test and
`tests/cnv_ingestion_measurement_contract.rs` is added. The expected private
helper `benches/ingestion_support/mod.rs` is deliberately absent. Intended
Linux RED is E0583 on that missing module after valid dependency metadata;
environment, lock, source or unrelated failures are not a test-first proof.

Contract tests exercise actual creation, exact count bits/signed zero/one ULP,
literal identities, coordinates, ordered roles/maps, no/all references,
gzip, malformed expected state and non-destructive export failures.
Existing production, lock, old fixtures/tests/bench/support remain unchanged.
No timed factory, speed/resource advantage, native helper acceptance or full
inferCNV/release claim is established by this test-first snapshot.
