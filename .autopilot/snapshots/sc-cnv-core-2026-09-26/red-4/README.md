# Private prepared-input measurement test-first candidate: red-4

Baseline is accepted product bdcc8a3a. This snapshot changes only Cargo.toml
and adds tests/cnv_measurement_contract.rs: the non-default measurement feature,
Linux-only nix resource dev-dependency and six behavioral contract tests.
The support module and single-stage loader entry point are intentionally absent.
No benchmark or production implementation is introduced here.

Cargo.lock retains the accepted baseline bytes. The explicitly selected
Linux-only test-first run resolves the measurement dev-dependency remotely
against that lock, verifies existing package identities remain unchanged and
preserves both lock versions. Every source other than that intentional lock
update must remain byte-identical. Controller acceptance of the returned lock
precedes implementation and locked four-native testing.

This is expected-red evidence, not a working benchmark, performance result,
product CLI or publication candidate. All original fixtures and production
core bytes remain unchanged. Preserve earlier snapshots and raw failures.
