# BED genomecov implementation plan

Use executing-plans inline, followed by one whole-feature review.

Goal: complete the BED-only genome coverage operation inside rsomics-bed.
Spec: `.autopilot/state/bed-track-contract-2026-11-03.md` and its thirteen
actual BEDTools2.31.1 witnesses. Unionbedg is a separate subsequent operation;
its two pending valid-input repair choices do not affect this implementation.

Architecture: one chromosome-group event buffer using existing DepthBuffer,
typed genomecov options, a narrow CLI adapter and existing atomic/shared-help
plumbing. No new dependency, public foundation, parallel runtime or version.
Direct main is explicitly authorized. Keep all output/build/cache external.

## Interface and rulings

- `genomecov(input: impl Read, genome: &Genome, output: impl Write,
  options: &GenomecovOptions) -> Result<()>` in `src/genomecov.rs`.
- Modes histogram/default, bedgraph, bedgraph-all, depth, depth-nonzero.
  Optional strand plus/minus, five-prime/three-prime end, split, max u32,
  finite scale f32, optional track options. Scale requires non-histogram;
  track requires bedgraph; ends exclude split and require valid BED6 strand.
- Preserve input chromosome-group order; absent chromosomes follow genome
  order. No sorted-start requirement. Repeated chromosome groups fail loudly.
- Reuse strict BED/genome parsers. Selected unknown contigs, out-of-genome spans,
  invalid/missing required strands, zero-origin virtual points and arithmetic
  overflow fail, rather than inheriting unsafe source indexing/permissiveness.
  Interior points widen before endpoint/split selection; upper virtual bounds
  clip to the declared size. Empty/zero-size chromosomes emit no depth rows.
- BED12 split applies only to exactly12 fields and uses checked virtual span
  as the block anchor. Other operations keep their existing raw block policy.
- Histogram fraction uses binary32 then six significant digits. Scale is
  binary32. Coalesce equal raw depths before max/scale; do not coalesce capped
  output values. Max affects histogram/nonfinal bedgraph runs, not per-base
  output; retain the source's uncapped final bedgraph run.
- Extract the existing general-significant formatter from map into private
  `src/numeric.rs`; map and genomecov are concrete consumers. Keep its existing
  numeric contract/tests unchanged.

## One complete operation task

1. Add `tests/genomecov.rs` with literal fixture/golden checks for histogram,
   all-zero-inclusive bedgraph, capped boundaries and split blocks; all13
   retained workflow cases compare complete output to actual bedtools2.31.1.
   Add one CLI lifecycle proving stdin/JSON named output and atomic failure
   preservation on a repeated chromosome. Run it RED for unrecognized command.
2. Implement typed options/reduction/reporting, private formatter extraction,
   `src/cli/genomecov.rs`, one parent dispatch arm and library export. No README
   stable-operation claim or release before complete gates.
3. Run focused GREEN, full debug/release, fmt and strict Clippy through the
   existing physically verified external guard. No micro-helper test matrix.
4. One whole-feature review of API/CLI/error paths and source behavior. Address
   important findings once with focused regression; defer nonblocking polish.
5. Representative complete-track performance/RSS with retained real RefSeq
   data and exact binaries; strict resource or throughput advantage required.
   Commit only this concern, push and verify all four exact-head native hosts.
   Do not publish or claim complete acceptance before these gates.

Review focus: sparse long chromosomes must not allocate per-base storage;
filtered chromosomes still appear empty; adjacent raw equal depths coalesce
but capped distinct depths do not; zero-origin/missing-strand errors propagate;
an invalid later chromosome must preserve a pre-existing named output.

Progress: steps1–4 complete with126 debug/release tests, strict Clippy/fmt,
18 source workflow comparisons and15-call local binary lifecycle. One review's
split adapter issue is RED/GREEN fixed; no re-review requested. Candidate
c6e35f8 is pushed; native run41519713617/full originals are adopted and
representative performance remains open. Step5 integrates the qualified
private candidate before measurement to
bind committed/four-native source; it does not promote a release. Detailed
rulings and retained identities are in
`.autopilot/state/bed-genomecov-execution-2026-11-03.md`.
