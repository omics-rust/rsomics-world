# Count read projection implementation plan

Goal: implement extension5/extension3/read2pos from the existing count dossier.
Spec: `.autopilot/state/count-projection-contract-2026-11-03.md`, including
the actual leading-deletion endpoint witness. No shift option or foundation.

Public Rust `ProjectionOptions` contains extension5/extension3 u32 values and
optional typed `ReadPosition::{FivePrime,ThreePrime}`. CountOptions gains one
default-disabled projection field. CLI uses --read-extension5,
--read-extension3 and --read2pos {3,5}; extensions are limited to i32::MAX.
This is an unreleased API addition and requires a minor-version release later.

- [x] Capture one mixed single workflow and one paired strand/fraction
  workflow against official featureCounts2.1.1. One joint native golden test
  must fail on the current binary before implementation.
- [x] Add private `projection.rs`; preserve first/last assignment-section
  endpoints, including empty initial sections, in `geometry.rs`. Projection
  mutates positive assignment Blocks only, leaving Extent and split intact.
  Apply raw-mate orientation, 5prime then3prime extension then reduction;
  clip using actual header length, not annotation bounds.
- [x] Wire through alignment Reader using one private read configuration,
  preserving default-path behavior and bounded parallel counting. Add CLI
  options through existing help/common plumbing. No new dependencies.
- [x] Compare complete counts/summaries through the existing live oracle,
  run debug/release/strict Clippy and one whole-feature review. Measure a
  representative projected counting workload before publication.
- [x] Commit/push owning source only when ready, then require exact-head
  four-native CI. Never replace immutable0.1.0 artifacts or claim new binary
  performance from historical measurements.

Owning source `2b5e450372689063736721dc1b807dcba9d62717` is pushed. Exact-head
CI41607175618 passed. Local debug/release each pass40 test functions,
including actually enabled source compatibility; strict Clippy and one fresh
whole-feature review pass. No package or GitHub release is published.
