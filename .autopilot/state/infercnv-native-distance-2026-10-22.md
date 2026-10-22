# Native inferCNV distance continuation

Design: `docs/plans/2026-10-22-infercnv-native-distance-design.md`.
Plan: `docs/plans/2026-10-22-infercnv-native-distance-plan.md`.

Ward accepted at world `f8383d573083427b385306c33926a3f17780e9c1`, product `4c9e78929fa97ef164fac205a28c153f21c221e1`; exact-head controller CI `39771308104` passed.

Distance work has begun, not accepted. Independent bounded extraction work owns only its two world scripts and product profile metadata/fixture README; root owns controller routing/tests. No local Cargo/R/build/install: boot APFS occupancy remains above the 80% gate. Native RED and four-class GREEN remain required. No public interface or performance claim.

The separately pending `all_observations` reference-name collision question remains unanswered. Native distance operates on actual captured indices, preserving that witness without deciding future public group policy.

20:08 CST user decision supersedes the preceding pending status: in pooled mode, explicitly reject reference-name collision with `all_observations`. Record this as a known upstream incompatibility rather than reproducing duplicated observation membership. Add the real negative test when implementing group policy; the current original-vector mathematical gate retains the unmodified historical witness.

Task 1 committed `730ace2d182a075bc6645932077a47de7843c93f`, fresh independent spec/quality review approved. Profiles: 152 rows, 30127 bytes, SHA256 `0686eeeda334a55b45117abf83bdc125badd33560a01a6a53d0746afa65a27dd`; original trees unchanged. Eleven focused tests passed. Its first full run raced with root's intentional selector RED (402 tests, five failures, two skips); the subsequent coherent full controller run passed 404 tests in 72.701s, two existing skips. Installed-R opt-in was unset; no local R/Cargo ran.

Task 2 selector tests first failed five of seven against the old workflow, then all eight passed as part of the coherent full suite. Real Bash checks cover closed selector/feature mapping and successful legacy/witness acquisition routing plus failure before samples-root export. The source/ZIP verifiers and receipt schema remain unchanged.

Frozen red-10 has 157 guarded source files, archive SHA256 `7793e036f3ea72a03734931d3a4cf84fcf4dabd99e5f971ba25e1b653f966304`, manifest SHA256 `e8c284c110fb068480948cb81ee1be87b5e9a3acd49efc3cdd57855fc1f3b4d7`. Both new modules are absent; Cargo.lock is unchanged. Actual native RED is pending dispatch after exact-head controller CI. Standalone rustfmt touched only the three owned private tests.
