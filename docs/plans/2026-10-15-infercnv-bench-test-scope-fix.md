# Private creation bench test-scope repair

Spec: `2026-09-30-infercnv-raw-ingestion-measurement-design.md`.
Execution ledger: `.autopilot/state/infercnv-native-creation-trial-2026-09-30.md`.

## Verified failure and cause

Run36730544730 at e172578b fails Linux strict Clippy on the new harness-free
bench: two unused imports and the unused NEXT counter in the shared creation
helper's test module. The bench is linted with test configuration, but its
`#[test]` function is excluded. Module-level test-only imports/state remain.
The same original job passes19 dedicated trial tests before linting.

## Repair and gates

- [x] Preserve original failed run/jobs/artifacts/logs and exact diagnostics.
- [x] Move only `use super::*`, the atomic import and NEXT into their one
  test function. Retain the writer-error test and its assertions unchanged;
  no lint allowance, visibility change, numerical code or timer change.
- [x] Freeze new immutable green-14. Only the shared helper hash may differ
  from green-13; require all149 exact source members and three checksums.
- [ ] External standalone formatting and controller tests, fresh independent
  review, scoped world commit/push and exact-head Control plane success.
- [ ] Run all four native classes with trial/support flags and original oracle;
  require strict Clippy,19Linux/14mac trial tests,55 creation/11 old support/
  72 ordinary tests per profile, both benches, unchanged source and lock.
- [ ] Audit original APIs/ZIPs/logs before committing owning product changes.

The existing real Clippy invocation is the regression test: green-13 fails;
the identical invocation on green-14 must pass. Do not add a source-text test
or count a warning-suppressed build as verification. Original green-13 remains
unchanged and unaccepted; no canonical performance or public release follows.
