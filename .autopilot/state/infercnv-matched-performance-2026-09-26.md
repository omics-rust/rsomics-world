# inferCNV matched performance — execution ledger

## Current status

Design and implementation plan complete; Task 1 test-first snapshot ready. No speed or
memory advantage has been measured or accepted. This follows accepted shipped
correctness, not another correctness reconstruction.

- Product baseline: `bdcc8a3a55596be8d46a774d39b0335845c75175`.
- Accepted four-native shipped conformance: run `36222541338`, source head
  `070dd67118745a50f31d08717b20ec1ea7e63609`.
- Accepted R oracle: run `36221554790`, receipt
  `../oracles/infercnv-shipped-2026-09-26.json`.
- Correctness documentation head: `282d2171dc799e5ef7e94962e7b0b8c53a4a922e`;
  exact-head Control plane run `36222894885` passed.
- Design: `../../docs/plans/2026-09-26-infercnv-matched-performance-design.md`.
- Plan: `../../docs/plans/2026-09-26-infercnv-matched-performance-plan.md`.
- Private work ledger: `.superpowers/sdd/2026-09-26-infercnv-matched-performance-plan/progress.md`.

## Environment boundary

Fresh preflight: boot APFS total 245107195904, free 5162151936 bytes
(97.89% occupied), KIOXIA 59 GiB free, HDD 254 GiB free. Local Cargo/R/builds
remain stopped; hosted native runners carry all compilation and measurement.
Source, scratch and evidence remain on the prescribed external disks.

## Scope and decisions

Only full-example prepared-input preprocessing through step 14 is measured.
Production core stays unchanged. One fresh-process smoke pair is excluded;
seven alternating pairs are accepted only after every result passes unchanged
identity and numerical criteria. Region clocks and baseline RSS are separate
from whole-process timing/peak RSS. This is not full CNV calling, large-cohort
scaling, biological validation, a public CLI or a release decision.

Independent design review required and now confirms both: full RDS group/hidden
state checks, and binding timed source/lock to verified four-native bytes.
Routine design/execution decisions use the user's explicit delegation; no
additional approval pause, destructive action or automatic publication.

## Test-first candidate

Plan commit `1f28ece08eab71d9fde6322c4e0647286b89d8fd` passed exact-head
Control plane run `36223517522`. Local external-temp standard-library checks:
84 tests passed and control-plane validation passed. All ten candidate workflow
shell blocks passed Bash 3.2 syntax; independent CI review has no important
finding. Resolver stderr must additionally be preserved from original Actions
logs; artifact-only logging improvement is a deferred minor.

Frozen `red-4` contains 140 files; only Cargo.toml and the new six-test
measurement contract differ from baseline. Support implementation is absent.
Production bytes and original lock remain unchanged. Archive SHA256:
`33d91da411d1518688ffb0b318fdded730fe6b1340834fc921eac05c42bba02c`;
manifest SHA256:
`bb8f6e6ef31668fa62a5a62a2826a4f0ce3b330ff02cd7e13cc0312cdd12719d`.
Hosted red/remote lock resolution remain pending; no measurement was run.
