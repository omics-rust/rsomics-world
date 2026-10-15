# R runtime helper resumed acceptance

Plan: `../../docs/plans/2026-09-30-infercnv-r-runtime-helper-plan.md`.
Status: source review and fresh package-free Linux execution pass; exact-head
controller delivery pending. No installed infercnv or canonical factory run.

Root's pre-pause missing-helper RED was preserved on4090 under
`/dev/shm/rsomics-ingestion-runtime-root-20260930.Rycrpc`. It explicitly names
the absent runtime helper; helper/test later match their reviewed hashes.
On2026-10-15 root created a distinct fresh remote gate
`/dev/shm/rsomics-runtime-resume-20261015.nqfw8m`, copied owning files and ran
R4.6.1 --vanilla with LC_ALL=C/explicit TMPDIR: exit0, all8 behavior groups,
empty stderr and exact before/after hashes. No package/build/install occurred.

Evidence copied without overwriting originals to external
`evidence/infercnv-ingestion-measurement-2026-09-30/r-runtime-resume-2026-10-15/`.
Helper SHA256:
`bcbbd0e3b6a2d053562345cd9ee59a4ff3e9dc4c8d4e6e2a7db66097d5787418`.
Test SHA256:
`35442d32734d592dd21a305e6e4527bf33994da8917e93e4d3d9b7807eb657b4`.
Fresh GREEN stdout SHA256:
`c86f214fac566e8dc5b4844e2217fe8b357b2290ab05f130e6062c15df0e35f7`.
Preserved historical RED stderr SHA256:
`3f5dca27961d7ff1f1ad6be6d3633f8a23ead5d7b58d1c13fbcf6c0f029e8cc7`.

Fresh root controller run passes274 tests; architecture/whitespace checks pass.
Fresh independent whole-change review finds no Critical/Important issue and
verifies exact R plan/source bytes. Reviewer did not execute R; root's actual
remote run, not source review, establishes this bounded execution result.

Final minor (deferred): RSS parameters are concatenated before scalar shape
checking. Malformed caller arguments can produce two plausible fields. Actual
trial callers use the already scalar-validated RSS reader; no malformed actual
sampling is demonstrated. Strengthen the private argument contract separately.

Final minor (deferred): existing-path rejection is not atomic-exclusive create,
and broken-symlink preservation lacks a dedicated regression. This gate requires
the isolated single-writer output directory; documentation now states that
invariant instead of promising race-safe creation. Shared/concurrent output
directories are not supported by this helper acceptance.

Ruling: keep the reviewed helper unchanged for this bounded single-writer gate;
the actual installed-package/paired driver must enforce unique nonoverlapping
outputs and apply the independent literal metric parser to real bytes. Cost if
wrong: a future caller could misuse argument shapes or overwrite concurrent
output; such a consumer must not claim this helper proves those guarantees.

Installed package/source/runtime authenticity, actual clock-documentation bytes,
full raw factory/timing,32 processes/64 states, native clustering and publication
remain separate gates. No performance or whole inferCNV result is accepted.
