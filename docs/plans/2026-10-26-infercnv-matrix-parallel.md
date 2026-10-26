# Parallel matrix smoothing plan

Use executing-plans inline, direct main commits, one whole-feature review.
The user authorizes routine decisions and prefers integrated work/performance
over separate small-part RED campaigns. Keep the full portfolio goal active.

Goal: extend actual worker execution into the serial matrix path, starting
with triangular smoothing, then measure whole-CLI benefit on the same host.
The prior 12-process benchmark showed only 0.64/0.62s at 1/4 distance workers.
Source inspection identifies smoothing's per-cell/per-gene/per-window loop;
this is a concrete high-work candidate, not a measured-exclusive hotspot.

## Design

- Add a positive threads field to unpublished product PreprocessConfig; CLI
  passes the same common ThreadArgs selection to preprocessing and samples.
  Existing library tests/measurement callers explicitly select one worker.
- Extract product-local cnv/parallel.rs::for_chunks for two concrete callers:
  condensed distance output and disjoint smoothing cell-column chunks.
  It owns fallible scoped worker launch/join; no public foundation is created.
- Smoothing retains the original output replacement buffer and immutable input.
  Workers receive complete cell columns. Per-gene neighbor/weight accumulation
  order, chromosome boundaries and all formulas remain unchanged. No reduction
  is split across workers, no epsilon or floating reassociation is introduced.
- Check finite normalization/result once per output gene rather than inside
  every neighbor iteration. A nonfinite sequential sum cannot later recover
  to a finite sum; retain explicit denominator/result rejection.
- No new numerical heap buffers: the same input/output matrices remain live.
  Scoped-worker handles/stacks are execution metadata, already excluded from
  the numerical-buffer limit. Cell centering and other stages remain serial.

## Tasks and verification

- [x] Add meaningful smoothing serial/parallel bit-equality and arithmetic-error
  checks; retain all existing golden/oracle expectations. Wire the existing
  canonical raw1/4-worker CLI test through the new config, including expression
  output so the full matrix—not only trees—is compared.
- [x] Implement shared internal dispatch and smoothing/CLI config wiring;
  update existing config literals and private test-module imports only where
  necessary. Do not add a new dependency, feature selector or CLI command.
- [ ] Freeze an actual candidate and run existing Linux native benchmark mode
  for correctness and same-host1/4-worker whole-process measurement. No local
  Rust/R build while boot exceeds80%. If benefit is not useful, investigate
  before adding more parallel complexity; do not merge merely to show activity.
- [ ] One fresh whole-feature review; final four-native debug/release and
  strict Clippy before acceptance/commit. Record actual metrics and unchanged
  output evidence. No publication or full-inferCNV claim follows.

Review focus: disjoint slices, partial columns, launch/join errors, unchanged
chromosome/neighbor order, memory overlap, zero-worker config and requested
versus available CPUs. Existing process and kernel tests cover those shared
contracts; do not duplicate whole admission frameworks.

Continue with the 4,103-cell real input, equivalent upstream performance,
HMM/denoise and remaining products. This optimization is not a smaller
definition of the portfolio objective or a replacement for release gates.
