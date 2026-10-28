# Concrete two-product TMM extraction candidate

Read-only assessment:
/Volumes/KIOXIA/Developments/tmp/tmm-layer-a-assessment-20261028.FsrSCJ/REPORT.md,
SHA0e9fbe39f1001bb3fb4a77ef81e3ca3adba33ad32fffe2d63892ecb69929876e.
Parent read report, both current call-site contracts, full shared kernel and
existing stats API/dependency/dirty-lock diff. Two actual consumers are edgeR
3a97e8f and limmaabb4a01; their149-line TMM/reference/rank/type7 block is exact.

Candidate is one existing-stats default-TMM function on borrowed retained
rows/provided libraries, returning reference sample and factors. No generic
matrix/formula/expression facade, new package or public quantile/rank API.
Product filtering, integer/fractional input policy, library/effective-offset
state, QL and voom/EB remain local. Keep the actual reduction/tie/NaN arithmetic.
The adapter's borrowed-row vector/all boundary validation must be counted,
not called zero-cost. Consumer-side lifecycle/source normalization assertions
and coordinated native/performance evidence precede public adoption.

Both final consumer performance baselines are now frozen; limma's complete
current voom path passes throughput/RSS and its worker-scaling gate fails.
Adopted implementation plan: docs/plans/2026-10-28-stats-tmm-candidate.md.
The bounded stats0.2.5 candidate uses clean external verification copies and
real isolated consumer call sites before any production adoption/public item.
The 8066 borrowed rows require129056 bytes on64-bit, because each slice reference
is a16-byte fat pointer; the earlier assessment's64528-byte estimate was wrong.
The fixed old/new hot-path check includes this adapter and public validation.

Stats has inherited Cargo.lock SHA
4b664c523c043985df8f5a3cd1eea135fe0390039b87e8c2dfa4739869038dc7:
it changes self-version AND removes common/CLI dependencies, while manifest
and public source remain0.2.4/common0.6. Preserve it; do not stage/reset it or
run mutating Cargo against it by accident. The tracked base lock also has a
stale self0.2.3 entry versus manifest0.2.4; only the task-owned clean copy may
normalize its package version while preserving the dependency base. Parent
may later index-only stage the independently validated new candidate lock,
leaving the inherited working file byte-identical. This does not authorize
unrelated dependency cleanup or waive edgeR's held full probability contract.
