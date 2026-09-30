# Canonical raw-ingestion measurement ledger

Status: exact controller helper implemented, independently reviewed and
accepted by exact-head CI. No actual timed factory or speed/resource claim.

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

Controller commit `afb398e992664adfbeaa5a7b89c4cfb7a55e7800` is pushed to main.
Exact-head Control plane run `36710205465` completed successfully. This gate
accepts the helper alone, not a timed trial or product performance.

## Remaining gates

1. Complete: independent helper review, main push and exact-head controller CI.
2. Private Rust creation comparison/export support with its own implementation
   plan, test-first failures and four-native debug/release gate.
3. Actual installed-package factory trial, source/runtime pins, paired process
   driver, immutable input/candidate receipts and complete provenance tests.
4. Thirty-two fresh processes yielding 64 checked factory outputs; 28 measured
   samples with frozen plain/gzip strict wall advantage criteria.
5. Original artifact/log/metrics/source audit before any bounded performance
   acceptance. Full inferCNV and publication remain separate.

## Fresh actual-factory source review

Root read the actual pinned archive factory, reduction and object validator;
an independent read-only reviewer verified their input/source hashes against
original run `36707344893`. Creation functions need a new installed-body/formals
check: the accepted samples witness pins four different run/cluster functions.
All three factory functions are in `R/inferCNV.R`, SHA256
`2b25f5fdf9665f382073255e44729ff38cca32f3862c158f91ebf58d43e475a2`:
`CreateInfercnvObject`, `.order_reduce` and `validate_infercnv_obj`.

The canonical plain count file is 36,378,958 bytes, SHA256
`539ea675832047cc77dc550c648dd421f2f5370aa2e7b52e0907b7dc690237c5`.
Its existing witnessed gzip is 4,407,153 bytes, SHA256
`489e4ea3ae98886eee1fbba5c01484b1ba0f10989d5e79b4c01873a0f1e15e12`.
Use those original bytes; do not regenerate gzip for the measurement.

The [input-only receipt](../oracles/infercnv-ingestion-measurement-input-2026-09-30.json)
freezes those cases, auxiliary files, exact expected state/maps/checkpoint,
factory source and accepted original artifact. Root verified every physical
hash/size against that receipt and streamed gzip to verify its exact decoded
size/hash equals plain. This is input authentication, not trial acceptance.

Keep intrinsic `digest(raw.data)` and object validation inside the timer.
Resolve absolute input/source/expected paths first, then run from the new
trial output directory: the upstream validator writes `broken.infercnv_obj`
to its working directory on a failure. Do not silence actual vanilla logger
advisories by changing scipen/logging to improve a measurement.

Warm validation/export runs in a local scope; retain only scalar/storage
metadata afterward. Drop both warm and expected objects and run full GC before
baseline RSS. Do not retain the old prepared benchmark's `prepared_identity`
list, which deliberately holds matrix references for a different lifecycle.
After the measured factory returns, collect retained-result RSS before any
oracle reload/comparison, hash/export, delta formatting or reclamation.
R options contain Inf: preserve typed options or dput text, not an implicit
JSON null. Record actual class/type and clock resolution without inferring
double storage, duplicate allocation, or nanosecond accuracy.

Private Rust support is specified in
`../../docs/plans/2026-09-30-infercnv-private-creation-support-design.md`.
Fresh independent spec review found no load-bearing gap after checking the
actual typed state/group constructor and old fixture/export contracts.
No factory trial has run and no new performance claim follows this review.

## Private Rust support test-first freeze

Root read the complete helper and test proposals, reused the product's checked
count constructor instead of maintaining duplicate invariants, and saved the
complete exact-code [plan](../../docs/plans/2026-09-30-infercnv-private-creation-support-plan.md).
The product now has only the new opt-in test and Cargo.toml registration;
the helper remains absent until the genuine remote RED is inspected.
Rust 1.91 standalone formatting passed without any local Cargo execution.

Candidate `red-7`: 145 source files, archive SHA256
`74025cd4c05f992d8521a1529841f70b359b300acf033b758b4268095d32cf2d`,
manifest SHA256
`0c83801bf5af66403e5da1a8f3ab9d2e45d8488d1a7d3d8021b864ddfff84626`.
All accepted green-10 source hashes excluding only changed Cargo.toml still
match, including production/old tests/support/bench and Cargo.lock.
The new test SHA256 is
`e9d9d450cb701c6ba03301dca0b4aeb31c6733d9634234ff9fd60f9216729df2`.

Candidate workflow adds only a default-false ingestion-support selector,
requires the existing measurement flag and runs the dedicated test in debug
and release. YAML, all 12 Bash blocks and the actual four-case selector guard
passed; all 223 controller tests and architecture checks passed. Native RED
dispatch, original failure inspection and four-native GREEN remain pending.
