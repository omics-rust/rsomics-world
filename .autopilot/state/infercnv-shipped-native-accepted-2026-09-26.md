# Accepted native shipped-example conformance — 2026-09-26

Product commit: `bdcc8a3a55596be8d46a774d39b0335845c75175`, local main in
`/Volumes/KIOXIA/Documents/omics-rust/rsomics-sc`. All 139 product files match
the independently reviewed and remotely tested green-5 snapshot. The worktree
is clean. No production algorithm, dependency lock or old fixture changed.

The product remains unpublished and library-only. This acceptance covers
prepared-input preprocessing through stage 14, not raw ingestion, clustering,
HMM, denoising, biological validity or performance superiority.

## Exact execution and provenance

Four-native run `36222541338`, attempt 1, completed successfully at world head
`070dd67118745a50f31d08717b20ec1ea7e63609`, after that head passed Control
plane run `36222470504`. The selected oracle was the explicit committed
`infercnv-shipped-2026-09-26` receipt, accepted R run `36221554790`.

Each platform verified the same original oracle artifact and manifest,
re-ran the semantic checker, and tested the same green-5 source with Rust
1.91.0 and the unchanged lock SHA-256
`636ead1961f573b303be8c02f6068201acd11dde1d0cb9bab3d81ccf86bf5554`.
All 139 source hashes passed before and after execution. Archive, source
manifest, lock and oracle receipt/JSON/manifest bytes match controller pins.

| Native target | Artifact ID | Debug tests | Release tests |
|---|---:|---:|---:|
| Linux x86_64 | 10900115098 | 39 | 39 |
| Linux aarch64 | 10898998995 | 39 | 39 |
| macOS x86_64 | 10899616681 | 39 | 39 |
| macOS aarch64 | 10899651586 | 39 | 39 |

Each profile passes 13 internal, 16 contract, eight reader, one four-case
synthetic oracle and one two-case shipped oracle test. Each shipped case runs
the pipeline once and checks all ten stages, identities, dimensions, final
returned state and unchanged prepared input. The two cases are 1,775→1,487
and 9,939→8,508 genes, each with 184 cells.

Linux x86_64 also passed the real missing-configuration failure gate,
formatting and strict all-targets Clippy. The negative execution failed exactly
because `RSOMICS_INFER_CNV_ORACLE_ROOT` was absent; a compilation failure or
silently skipped test could not satisfy the gate.

## Numerical results

All eight platform/profile combinations have the same reported maxima:

- All 40 original synthetic stage comparisons pass; maximum absolute error
  remains `7.105427357601002e-15`.
- All 20 shipped stage comparisons pass. Stages 1 and 2 are numerically exact.
- Maximum shipped absolute error is `1.2732925824820995e-10`, at full-case
  depth normalization, where values have a larger scale.
- Maximum error divided by the unchanged allowed bound is
  `0.010642098320408794` (about 1.0642% of the allowance).

The criterion is `abs(actual - expected) <= 1e-12 + 1e-12 * abs(expected)`;
it is not a uniform `1e-12` absolute-error claim. Per-stage absolute and scaled
values for every profile are preserved in `verification.json` and raw logs.
No tolerance or production algorithm was changed to achieve these results.

## Preserved evidence and review

External root:
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-native-core-2026-09-26/run-36222541338/`.
It contains original run/jobs/artifact API records, all four original ZIPs,
the original Actions logs ZIP, extracted evidence, a single-use controller
verification script and its structured result. All ZIP digests match the API,
CRC/path/type checks passed, and every extracted file matches its original
archive. The eight original Actions debug/release step logs match all twenty
uploaded shipped-stage error reports in their corresponding profiles.

| Artifact | SHA-256 |
|---|---|
| Linux x86_64 ZIP | `f88b4b259abba0f566e59a4f9d066f5203ab7a5c908831d5bac680ff15fbb004` |
| Linux aarch64 ZIP | `9d03d796cd37247269d736ca49acffbd1626e26c2bad15ac19309a943769ab35` |
| macOS x86_64 ZIP | `5dab015516bd32544d6fe75cb519cf969f74dae8ebcc0b20e4d56f49c5d6da23` |
| macOS aarch64 ZIP | `c0f6e30208180f57af1f4fa2e3cc3dc87111ec9799a3310c1d15f06d7cfce0ce` |
| Original logs ZIP | `0598e54a6ee4d743d0953822fcc465343e5e0c004f687feab94ac463c891cfff` |
| Verification script | `95eb3eb67a92e163ffe0a2dcbf0aaf7f8f839798479d0d6d95c7df0bbec3a015` |
| Structured verification | `3a82ea0e6b6b887f36613eb45322bca3dbbbf2638372a9c3c7ac16a88c481090` |

Task-scoped and final whole-change source reviews are clean. The final review
covered all ten changed control-plane source/receipt files and five product
files; it requested no fixes and did not substitute its verdict for execution.
Earlier reader-only, failed oracle/checker and failed test-setup evidence remains
preserved separately. No Mac mini local build, installation or R execution ran.

Next: matched same-host prepared-input performance measurement, then raw-state
and downstream workflow implementation under separate source/oracle gates.
