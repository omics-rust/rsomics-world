# Phylo native and source delivery plan

> Use superpowers:executing-plans with bounded parallel implementation and
> one fresh whole-delivery review. Preserve original evidence and user edits.

**Goal:** Deliver actual native binaries and complete verified source for the
seven-command Phylo product including the accepted WPGMA extension.

**Architecture:** Adapt the existing independent-product delivery pattern.
Product-local lifecycle fixtures exercise installed binaries; Cargo produces
normalized source, native CI validates all four real hosts, and bundles carry
their dependency notices and provenance. No new public Rust API or dependency.

**Spec:** `docs/10-products/phylo.md`,
`.autopilot/state/phylo-wpgma-2026-11-03.md`, owning CLI/contracts and the
existing seq source/native delivery pattern.

## Constraints and review focus

- Owner `/Volumes/KIOXIA/Documents/omics-rust/rsomics-phylo`, directmain.
- All local project build/cache/tmp data remain on verified external disks.
- No registry retry with known rejected credentials; no release-name reserve.
- Rooted WPGMA and UPGMA, unrooted NJ, common JSON and transactional output
  must survive installation from the normalized crate, not only source tests.
- Verify complete source membership/body/mode and normalized metadata;
  no path dependencies or missing license text in native bundles.
- Bind actual host/executable, sourcehead, package, lifecycle and checksums.
- Existing0.1crate and measured784cef7 binary remain immutable/separate;
  never claim a new carrier was remeasured without actually measuring it.

## Task1: Complete product-local carrier

Files: owner `scripts/delivery.py`, `.github/workflows/ci.yml`, package version
and lock, README/PERFORMANCE and generated `licenses`/THIRD_PARTY_NOTICES.

- [x] Prepare0.2metadata, unchanged dependency lock, complete four-target
  normal/build license inventory from verified registry archives.
- [x] Implement source/smoke/bundle actions by adapting the existing seq helper;
  installed lifecycle covers seven commands and all three inference methods.
- [x] Run one local candidate source/install/lifecycle; keep candidate status
  explicit and retain all outputs, not a release-success surrogate.
- [x] Run fmt, strictClippy, full debug/release suite and whole-delivery review.
- [ ] Push final candidate once complete; wait for exact-head native4 including
  normalized crate install and lifecycle, and retain original artifacts/logs.
- [ ] Independently verify original native/source carriers, then publish a
  GitHub release and perform a fresh public-install lifecycle if gates pass.
- [ ] Record publication/authentication boundaries in the control plane.
