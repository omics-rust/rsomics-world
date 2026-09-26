# Checked runtime-provenance measurement candidate: green-7

This candidate fixes green-6's runtime environment record: a pure helper now
distinguishes absence from non-UTF-8 values and rejects tab/newline/CR before
writing TSV. Four behavioral regression tests cover these cases. Child CPU
is sampled before the self-CPU baseline to keep the latter closer to the call.

Actual test-first run 36226422538 failed only the new named measurement test
with E0425 missing format_env_value; ordinary 38 tests and all 142 source
checks passed. Green-6 passed four-native checks but remains intermediate
evidence because its provenance defect had not yet been repaired.

Only benches/cnv_matched.rs, benches/support/mod.rs and
tests/cnv_measurement_contract.rs differ from green-6. Production, fixtures and
the reviewed Cargo.lock remain unchanged. Locked four-native debug/release,
eleven measurement contract tests, bench checks and strict static checks are
pending for this snapshot, followed by scoped re-review. No actual performance
measurement, complete workflow, CLI or publication is claimed.
