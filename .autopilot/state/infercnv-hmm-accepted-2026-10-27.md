# Accepted native grouped-sample i6 workflow

Bounded functional acceptance, not full inferCNV or publication. Product code
40c3e446f3b72954ae1edaa60606ec4af742bcc3 and documentation
7853e92282c43b01d785df40fd19b6a933d035d9 are committed; the worktree is clean
and publish=false. No public SC repository or registry release was created.

Exact source: green25, worlda133ad5d2a9dfaffee6facfba313ac6db73445af;
manifestd2ad2aa0532ebdb9dd9446914828d941dca7a0910626d71f3ef40321b39313ac,
archive51ddcb7577ba551afa0d21845ea9f3cc48b2871586be98bedcfe1a21bcf70a9a.
All180 candidate files match the code commit; the later documentation commit
changes only README. Controller40484562370 and native40484702873 both pass.
Native Linux/macOS x86_64/aarch64 each pass131 debug and131 release tests;
formatting and strict Clippy pass. The default-feature test count excludes
the older opt-in external suites; their retained evidence is unchanged.

The real R oracle is40477088139/artifact12721768541, pinned by
.autopilot/oracles/infercnv-hmm-workflow-2026-10-27.json. Each native platform
runs full/proxy/aggregated raw workflows at1/4 workers (24 native processes).
Each retains8508 genes,184 cells and six prediction groups. Exact states,
memberships/order and all four original report projections match; region
counts are82/56/39 and predicted-gene rows9213/5984/5810. Worker fingerprints
match within each platform. Expression14 keeps the existing
abs1e-12+rel1e-12 criterion (maximum scaled error0.0010313925). Calibration
uses the predeclared abs1e-9+rel1e-8 criterion (maximum scaled error1.853901e-6).
Neither tolerance was relaxed. The original synthetic spline error is retained
as R-only negative evidence, not a fabricated successful calibration.

Root downloaded, hashed and safely extracted all four original native ZIPs,
verified API run/head/size/digest, source manifest/archive, compiled binary
hashes, complete six-trial summaries and worker fingerprints. External root:
rsomics-fixtures/evidence/infercnv-hmm-native-2026-10-27/run-40484702873/.
Its acceptance-summary.json records complete artifact and binary identities.

- Linux x86_64 artifact12724377147,77649841 bytes,
  SHA961dda2ddeb189476ccc2c70bcf6c4f847330a8b02fecdae4b713bd17047eb5c.
- Linux aarch64 artifact12723924190,77622656 bytes,
  SHAbe94fe8635ae519ab6f0afafa00c51208bf2e6d80d1ddce7dd1359157d3fc7e8.
- macOS x86_64 artifact12724802157,77603992 bytes,
  SHA0f3479d547652550111ba9e71012ae4e48abcc41162dd7308263fbaaf0d6cf78.
- macOS aarch64 artifact12723904295,77568023 bytes,
  SHAe17a61eb7589fbfa046a8981691669a6737579788b02051421029692b7e1c725.

One whole-feature review passes without Critical/Important findings; its
minor array/range contract documentation item is resolved in README. See
infercnv-hmm-review-2026-10-27.md. No per-helper or repeated denoise review.

Linux x86_64 full-case native owner wall observations are1.541/1.231 seconds
at1/4 workers, with about104.2/104.3MiB peak RSS. These are individual
process observations, not a paired R speed comparison or distribution-based
performance qualification. No replacement-speed or biological-validity claim.

Remaining: actual subcluster discovery, Bayesian postprocessing, i3/per-cell
and per-chromosome profiles, default Leiden/PCA behavior, reporting/persisted
state and representative full-path performance/release gates. Read the next
workflow contract before extending interfaces. First capture actual Ward
subcluster membership, including its source traversal-chunk rule; do not
assume plain cutree labels. Full inferCNV and the entire portfolio goal remain
active. No local compilation/install/download-to-boot, deletion or automation.
