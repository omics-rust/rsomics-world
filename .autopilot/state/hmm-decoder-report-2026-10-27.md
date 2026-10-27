# Isolated native i6 decoder handoff — 2026-10-27

Status: source written and standalone-formatted; **uncompiled and all tests
unrun**. This is an integration handoff, not correctness acceptance.

## Owned files and integration

- Product: `/Volumes/KIOXIA/Documents/omics-rust/rsomics-sc`.
- Added `src/cnv/hmm/decode.rs` and `src/cnv/hmm/log_normal.rs` only.
- Main must declare sibling modules `decode` and `log_normal`; no `mod.rs`,
  metadata, CLI, notices, `normal.rs`, `rng.rs`, or `spline.rs` was edited.
- `decode::Model::{new,decode}` has exactly the brief's private interface.
- Transition validation remains the enclosing config's responsibility, as
  specified in the binding brief. Model construction validates finite means,
  finite nonnegative SDs, and a positive shared median SD.

## Behavior and implementation

Read the pinned archive's `R/inferCNV_HMM.R:230-264,1101-1175` before writing
the decoder. Implemented the normalized inverse-negative-log-normal-tail
emission, neutral-biased initialization, six-state transitions, strict
first-maximum recurrence and traceback, singleton neutral state, and empty
input error. The upstream final gate rejects **any** negative-infinite final
state; consequently `t=0` errors for non-singletons. That behavior is retained,
not silently converted to a neutral path.

Scores are two stack six-element rows; predecessor storage is six bytes per
noninitial observation, plus the one-byte-per-observation output. Both heap
buffers reserve fallibly. Numerical failures propagate as InvalidInput errors.
No emission/score matrices are stored. Model state is reusable per group.

The normal-tail helper adapts historical team-owned
`rsomics-normaltest/src/ndtr.rs` Cephes coefficients. It computes the erfc tail
directly in log space, with reciprocal-polynomial evaluation for large
arguments, rather than taking the logarithm of an already underflowed
probability. Differences too large for a representable finite log tail are
reported, not clamped.

Five tests were written (four decoder tests, one tail test): neutral and
singleton/empty behavior; no-stay tie/traceback plus zero-transition failure;
strong loss/gain transitions overcoming neutral initialization; shared median
SD, emission normalization and numerical/calibration failures; reference
normal log-tail values at zero, one and forty standard deviations. The
median-SD expectation `-2.658871440537415` is independently reduced from
five weights `1/ln(2)` and one weight `1/1.8410216450092634`, not calculated
by the implementation under test. Numeric reference literals are not claimed
to be new installed-R measurements. No helper oracle or exhaustive grids ran.

## Verification and limits

Only existing external Rust 1.91 rustfmt ran:

```text
/Volumes/KIOXIA/Developments/rustup-home/toolchains/1.91.0-aarch64-apple-darwin/bin/rustfmt
rustfmt 1.8.0-stable (f8297e351a 2025-10-28)
--edition 2024 --check <the two owned source files>: exit 0
```

`TMPDIR`, `CARGO_HOME` and `CARGO_TARGET_DIR` were explicitly external for
formatting. The binding brief reports boot APFS at 92%, so no compiler,
compiled tests, installs, Cargo operations, dependency/data downloads,
benchmarks, Git commands, CI, deletion, staging, commit, publication, or
subagents were used. Public raw-source pages were read in the web tool only.
Formatting establishes parser/format acceptance, not Rust type correctness.

Static self-review checked recurrence/traceback alignment and first maxima,
the exact any-unreachable final gate, allocation bounds, stable large-tail
evaluation, and that the median test can distinguish a wrong shared SD.
Whole-feature review, installed-R workflow goldens, native four-platform
compilation/tests, strict Clippy, and performance acceptance remain main-owned.
The Cephes implementation is not a bit-for-bit R pnorm port; near-tie path
sensitivity must be assessed by integrated real-R goldens.

## Attribution / notice action for main

Do not credit the new tail as entirely original team math. Its coefficients
and erf/erfc approximation derive from Cephes through the historical
team-owned ndtr asset. The original header states: Cephes Math Library
Release 2.2 (June 1992), copyright 1984, 1987, 1988, 1992 Stephen L. Moshier.
The coefficient/header provenance was checked against the versioned source:
[SciPy v1.15.1 ndtr.h](https://raw.githubusercontent.com/scipy/scipy/v1.15.1/scipy/special/xsf/cephes/ndtr.h).

For the integrated notice review, retain that Moshier attribution, identify
the historical `rsomics-normaltest/src/ndtr.rs` asset and direct-log adaptation,
and reproduce the full BSD-3-Clause conditions/disclaimer for source and binary
distributions. The matching versioned
[SciPy v1.15.1 license](https://raw.githubusercontent.com/scipy/scipy/v1.15.1/LICENSE.txt)
was read: copyright (c) 2001-2002 Enthought, Inc. 2003-2024 SciPy Developers,
all rights reserved. Retain that copyright plus the Moshier header attribution;
keep the source/binary retention conditions, non-endorsement condition, and
full warranty/liability disclaimer. The current extracted implementation's
[xsf license](https://raw.githubusercontent.com/scipy/xsf/main/LICENSE)
was also read, but the version-matched SciPy text above is the pinned source
reference. Do not infer license sufficiency solely from the old crate's
MIT/Apache Cargo metadata. No GPL R normal-tail implementation was copied.
The existing infercnv notice should also name the HMM behavioral reference;
the decoder follows the described algorithm rather than transcribing the
upstream R implementation. Notices were deliberately left to main.
