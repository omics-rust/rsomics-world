# BED coordinate-generation implementation plan

> For execution: use executing-plans inline, with one whole-feature review.
> User unattended/direct-main/minimal-test/external-storage rules override
> approval handoffs, worktrees and per-helper review/test ladders.

Goal: implement the already scoped slop/shift/flank workflow inside rsomics-bed.
Spec: `.autopilot/state/bed-coordinate-contract-2026-11-03.md` and BED dossier.
The flank point-output decision remains pending; no silent policy selection.

Architecture: one private streaming executor around existing BedReader,
Genome and coordinate-preserving record output; narrow typed options/functions
for operations. Existing common/help CLI handles transactional output, aliases,
stdin and JSON. No index, new dependency, public foundation or borrowed storage.

Files: add `src/coordinate.rs`, `src/cli/coordinate.rs`,
`tests/coordinate.rs` and its pinned literal golden files; modify only `lib.rs`
and `cli.rs` registration. Add a private optional-column accessor in `bed.rs`
so the two source-specific unknown/missing-strand rules do not swallow errors
from the existing required-column API. Keep the published release/version unchanged.

## Task 1: complete slop/shift consumer workflow

Interfaces in `coordinate.rs`:
- `SlopOptions { left: f32, right: f32, fraction: bool, strand_relative: bool,
  header: bool }`.
- `ShiftOptions { plus: f32, minus: f32, fraction: bool, header: bool }`.
- `slop(input: impl Read, genome: &Genome, output: impl Write,
  options: &SlopOptions) -> Result<()>`; `shift` with ShiftOptions likewise.

Validate finite parameters once and genome membership/nonempty chromosomes at
the stream boundary. Preserve order/duplicates/suffix bytes and initial headers.
Use signed widened point coordinates internally rather than the existing
unsigned virtual_bounds helper. Slop uses binary32 fractional widths, truncates
widths before signed arithmetic, clips then handles collapsed/crossed bounds.
Shift promotes the binary32 parameter to binary64, scales if fractional, adds
to coordinates then truncates after its independent endpoint clipping. Restore
point report adjustments; invalid/overflowing output propagates an error, never
an illegal BED row. Do not claim full point-boundary compatibility from the
five observed source calls, or release unsupported behavior silently.

CLI: shared input/genome/output/header args; slop `--bases` or paired
`--left/--right`, `--fraction`, `--strand-relative`; shift `--shift` or paired
`--plus/--minus`, `--fraction`. Accept negative numeric values. Follow existing
help headings and use the existing unified parser, not another CLI model.

- [x] Add ONE integrated CLI golden test over the shared witness fixture and
  the five complete slop/shift captured outputs. Observe unavailable commands
  failing before implementation; no scalar/helper assertion matrix.
- [x] Implement the two operations and shared streaming/CLI execution. Source
  numeric/unknown-strand differences remain operation-specific, not inferred
  from other BED commands. No provenance/source narration comments.
- [x] Run that consumer, then full existing tests, fmt and strict Clippy under
  the actual external guard; retain outputs and any genuine failure.
- [x] One fresh whole-feature review; at most one necessary correction pass.
  Keep native/public install/performance/release gates explicit and pending.

## Task 2: flank after affected policy decision

- [ ] Resolve the actual illegal point-output question before defining its
  public behavior; preserve original source outputs and distinct native golden
  expectations for any approved repair.
- [ ] Add flank on the same private stream, with genomic left/right output
  order and its own source strand/fraction contract. Extend the existing
  integrated consumer, not a separate helper matrix or crate.
- [ ] At the coherent delivery boundary qualify all stable operations with
  full native-four/install and representative complete-output wall/RSS evidence.
  No placeholders, bug-emulation switch, unchanged performance rerun or
  publication before its required gates. The registry authorization hold stays.
