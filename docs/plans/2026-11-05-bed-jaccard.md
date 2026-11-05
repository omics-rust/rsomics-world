# BED Jaccard implementation plan

> Execute inline in the authorized owning repository on main. Keep the
> independent whole-feature review and publish only a qualified contract.

**Goal:** Add sorted BED Jaccard similarity to `rsomics-bed`, preserving
BEDTools' meaningful merge, strand, fraction, header and ordering behavior.

**Architecture:** Reuse the existing two-input order checks within the
product. Merge at most the active chromosome's spans per input and strand;
compare each matched lane with a two-pointer sweep. Write the final report
only after both inputs and their chromosome orders have been validated.

**Tech stack:** Rust 1.91, existing BED parsers, rsomics-common/help, no new
public foundation or dependency.

**Spec:** `AGENTS.md`, `docs/10-products/bed.md`, the pinned BEDTools 2.31.1
source and 28 executable witnesses in
`/Volumes/KIOXIA/Developments/tmp/bed-jaccard-source-20261105.neNjvD`.

## Contracts and holds

- Owner `rsomics-bed` starts at `b4a3282`. Retain the historical
  `rsomics-bed-jaccard` two-pointer idea and five useful goldens from
  `ca81d15e39d042ed3568c03e702894cc94814f1a`, not its whole-input sorting
  or f64 ratios. Add the witnessed strand and overlap controls.
- BED only, exactly two inputs. One may be stdin. Shared genome order,
  optional A header, named atomic TSV and common JSON/output behavior.
- Merge overlapping or book-ended spans before selection. `-s` matches
  both strands independently; `-S +|- ` selects only that strand.
  Unknown or absent strand has no directional lane. Ratios use f32 and the
  same source fraction validation: explicit values in (0,1], reciprocal
  with no explicit B, and either with explicit B requires explicit A.
- Output columns are intersection, union, jaccard, n_intersections. Each
  admitted merged pair adds one hit; union is both merged totals minus
  admitted overlap. Empty union is undefined and prints `nan`.
- Three witnessed defects await a user policy: loss at a tied ordinary/point
  start, loss of BED12 blocks under split merge, and a recycled zero-width
  hit under either-fraction selection. Preserve the original bytes and keep
  those behaviors out of qualification. No source commit or release before
  a coherent contract.
- Use only physically verified external project output, build, cache and
  scratch paths. No destructive cleanup, dependency change or version bump.

## One connected implementation unit

- [x] In `tests/jaccard.rs`, exercise the real missing subcommand against
  pinned BEDTools. The overlapping/book-ended example must produce
  `17 53 0.320755 2`; `-f 0.6 -r` yields `15 55 0.272727 1`.
  Cover the witnessed directional values, nonlexical/genome order, point
  widening and empty union with integrated cases; no per-helper tests.
  The invalid B tail must fail without replacing a named result.
- [x] Add `jaccard(a, b, output, &JaccardOptions, Option<&Genome>) -> Result<()>`
  in `src/jaccard.rs`. Lift the current map pair-order machinery into
  `src/order.rs` without changing map. Consume raw starts in order and
  finish each merged lane before fraction selection. Surface the tied
  point case explicitly in the local candidate until the ruling.
- [x] Wire `src/cli/jaccard.rs` and the existing command tree. Reuse
  shared output and alias handling. Run focused GREEN and the full owner
  suite; keep split and the tie ruling visible in the execution record.
- [ ] Resolve the held semantics and compare the resulting full
  workflow with real source and known-divergence fixtures. Run fmt, strict
  Clippy, docs, release tests, and one independent whole-feature review.
- [x] Compare complete outputs and paired wall/RSS on the retained sorted
  RefSeq transcripts and exons for the current core. Results bind only
  the measured executable.
- [ ] Requalify the final intended source; commit one coherent concern,
  verify exact-head native-four CI and installed lifecycle, and record
  evidence in the control plane. Publication is separate.
