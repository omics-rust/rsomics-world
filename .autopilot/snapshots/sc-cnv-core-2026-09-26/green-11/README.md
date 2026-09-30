# Exact private creation support candidate: green-11

Baseline is accepted product `e62fc960aeeb9dc543298885213cda6a57196fe9`.
The new opt-in test and Cargo registration match red-7 exactly; add only
`benches/ingestion_support/mod.rs`. Original run `36715831817` proved the
path-attribute missing-helper failure after successful source/dependency,
ordinary and old measurement-support checks. It did not accept this helper.

The private helper reuses checked PreparedCounts/ExpectedStage, binds exact
ordered roles/maps and compares every count bit. The four-file exporter uses
actual group indices, literal fields and exclusive creation. Complete source
and tests passed root and fresh independent code review before this freeze.
The callback-error test is not a real operating-system flush-failure proof.

All old production/tests/fixtures/bench/support and Cargo.lock are unchanged.
Four native debug/release test gates, format/strict Clippy and original-evidence
inspection remain required. This snapshot does not create a timed factory,
new ingestion-speed/resource result, whole inferCNV claim or public release.
