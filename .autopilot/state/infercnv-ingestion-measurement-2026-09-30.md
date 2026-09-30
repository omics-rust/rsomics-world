# Canonical raw-ingestion measurement ledger

Status: exact controller helper implemented and independently reviewed;
exact-head CI pending. No actual timed factory or speed/resource claim.

Design:
`../../docs/plans/2026-09-30-infercnv-raw-ingestion-measurement-design.md`.
First independently testable controller step:
`../../docs/plans/2026-09-30-infercnv-creation-export-checker-plan.md`.

The helper verifies both warm/measured exports independently against stage 1:
binary64 count bits including signed zero, literal TSV identities/coordinates,
roles and ordered maps. Each phase must contain exactly four regular files;
symlinks, directories and missing/extra files fail. Trial-root metadata remains
outside this helper and belongs to the subsequent complete-trial driver.
Source/input/receipt pins and timings are not established by this helper.

## Test and review evidence

- Eleven tests first failed explicitly because the checker file was absent,
  then passed after implementation.
- Three genuine phase-entry mutation tests failed before the inventory guard:
  empty directory, directory symlink and broken symlink; all passed afterward.
- Final focused suite: 21 tests. Full controller suite: 223 tests, passed by
  both worker and root with external TMPDIR and bytecode disabled.
- Literal quoted IDs/groups remain literal; added CSV quotes or quoted numbers
  cannot alias the expected values. The existing generic CSV table reader is
  intentionally not reused for this different literal TSV contract.
- Root smoke-checked both original plain/gzip creation exports from run
  `36703475039` against its shipped full stage-1 tables and witnessed maps:
  9939 genes × 184 cells; four hashes exactly match accepted creation exports.
  This only exercises checker loading/comparison; it is not a timing run or
  acceptance of that run's incomplete samples/Ward evidence package.

Frozen controller SHA256:

| File | SHA256 |
| --- | --- |
| `scripts/validate_infercnv_ingestion_measurement.py` | `0fb1c968e46ff4dd041a42907ab70efaee9fae8beadf1254c81b1edf3c19cb36` |
| `scripts/test_validate_infercnv_ingestion_measurement.py` | `da5f3d79593cabbe8b0c9337f4fdb6fbe7a05efb28fef40768a4f382cebae879` |

Product `rsomics-sc` is still clean at
`e62fc960aeeb9dc543298885213cda6a57196fe9`; production Rust, Cargo.lock and
  old prepared measurement support have not changed. No local build is permitted
while boot APFS remains above the 80% gate. Controller scratch stays external.

Fresh independent code review approved this exact-state helper after reading
both complete files and independently passing 21 focused and 223 controller
tests. Review accepted only the helper contract, not a source-pinned or timed
measurement. Its frozen hashes remain unchanged.

## Remaining gates

1. Independent full helper review, main commit/push and exact-head controller CI.
2. Private Rust creation comparison/export support with its own implementation
   plan, test-first failures and four-native debug/release gate.
3. Actual installed-package factory trial, source/runtime pins, paired process
   driver, immutable input/candidate receipts and complete provenance tests.
4. Thirty-two fresh processes yielding 64 checked factory outputs; 28 measured
   samples with frozen plain/gzip strict wall advantage criteria.
5. Original artifact/log/metrics/source audit before any bounded performance
   acceptance. Full inferCNV and publication remain separate.
