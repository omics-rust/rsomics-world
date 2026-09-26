# inferCNV matched performance — execution ledger

## Current status

Design and implementation plan complete; implementation pending. No speed or
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
