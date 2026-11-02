# PLINK complete output blocks implementation plan

> Use executing-plans for one coherent private change and one fresh whole-feature review. Existing unattended/direct-main authority applies; no helper-review ladder or new approval checkpoint. Root owns integration, native qualification and measured performance.

**Goal:** Remove synchronous handoffs inside LD calculation jobs while keeping
output order, bounded live memory and existing numeric/durability contracts.

**Architecture:** Workers compute one byte-admitted complete job, deliver its
buffer and summary together, and reuse that same buffer only after the writer
returns it. Exactly one payload per lane exists, transferred rather than
duplicated. One ordered writer appends and returns buffers. Make uses this same
private protocol; prune remains independent. No disk spool or public API.

**Tech stack:** Rust1.91, std channels and the existing Rayon pool.

**Spec:** AGENTS.md; `docs/10-products/genotype-popgen.md`;
`.autopilot/state/plink-compact-metadata-2026-11-02.md` next investigation;
actual owner `src/ordered.rs`, `src/ld/{mod,pairs}.rs`, `src/make/pipeline.rs`.

## Constraints and decisions

- Owner `/Volumes/KIOXIA/Documents/omics-rust/rsomics-plink`, clean499693cf.
- Verify actual external Cargo/Rustup/target/tmp/cache devices before every
  build/test; use existing guarded-cargo.sh with inherited CPPFLAGS removed.
- Scratch `/Volumes/KIOXIA/Developments/tmp`; preserve all old evidence.
- Metadata candidate native/source41478980090 continues unchanged, not replaced
  or restarted by this work. Future source gets its own capture/gate.
- Keep genotype/numeric kernels, emitted fields, precision, pair ordinals,
  filtering, reader close, single writer and atomic commit behavior unchanged.
- No dependency/version/CLI/schema/foundation changes. Rare invariant comments.
- No performance claim from structural reasoning or test timings. Existing
  negative matched performance evidence remains authoritative.

## Design

Replace the public-to-module Sink/send loop with complete results. The private
producer consumes an optional recycled payload and returns `(payload, summary)`.
After delivery, a lane waits for that payload to be returned before computing
its next cyclic job. The consumer writes and returns it even for an empty
output job, then accounts its summary. At most one buffer per lane exists;
the writer and queue merely transfer ownership. This intentionally retains
finite ordered backpressure, but no job blocks halfway through calculation.

Retain initialization and finish hooks so every successfully opened reader is
closed on success or error. Consumer failure drops receive/return endpoints
before joining workers, waking blocked senders/receivers. Producer errors
propagate and prevent output commit. Make's complete genotype block migrates
without a second PGEN writer; prune does not use this helper and is unchanged.

LD reserves a maximum4MiB result buffer per active lane, separately from reader,
cache, metadata and channel overhead. Derive a conservative maximum emitted
line length from selected variant ID/allele lengths, autosome/position fields,
flags and the actual six-significant formatter. Choose the job's candidate
pair count as the existing compute target capped by this byte bound. A single
line exceeding the cap selects the existing serial path, not an input error
or hidden allocation. Buffer allocation uses exact reservation and never grows
past its admitted capacity; unexpected bound violation propagates as an error.
The4MiB bound is a bounded result unit, not a claimed optimal tuning constant.

Resource admission must count the actual one-buffer-per-lane ownership model,
including capacities, planner vectors and channel overhead; reduce workers or
fall back to serial rather than overrun the budget. Preserve reader/cache
minimums. Keep pair ranges increasing and existing private persistent caches;
do not add work stealing, shared caches or affinity scheduling in this wave.

## Review focus

1. Output lengths include UTF-8 IDs/alleles and numeric exponents; a bound must
   never silently truncate or permit Vec growth beyond reservation.
2. Empty/filtered jobs still deliver summaries and return buffers in order.
3. Producer/consumer failure cannot deadlock, lose reader close or commit output.
4. One live payload per lane includes writer ownership and last-job shutdown.
5. Make's raw GT/DS/HDS and serial budget fallback remain byte-equivalent.

## One task: complete-result protocol and consumers

**Files:** `src/ordered.rs`, `src/ld/pairs.rs`, `src/ld/mod.rs`,
`src/make/pipeline.rs`; existing native workflow tests only where necessary.

- [ ] Add one deterministic real producer/consumer test showing complete work
  can finish in every lane before the writer drains the first result. Do not
  rely on tight timing thresholds; retain order and failure/cancel coverage.
- [ ] Add one meaningful pair-planning/output bound regression using long
  identifiers and the existing serial/parallel full-output comparison. Include
  an over-cap line selecting serial fallback without changing output.
- [ ] Observe the old protocol's actual failure of the new concurrency contract
  before replacing it; retain the external log. No exhaustive getter tests.
- [ ] Implement the complete-result protocol, one payload per lane, shutdown
  and finish/error propagation; migrate make and remove the old streaming Sink.
- [ ] Replace LD streaming Blocks with an admitted complete buffer, derive line
  bounds and byte-capped jobs; update LD/make accounting and serial fallback.
  Preserve all original output bytes and scientific job membership.
- [ ] Run guarded full debug/release all-feature tests, fmt and strict
  all-target/all-feature Clippy. Expected: complete source workflow passes,
  raw state and one/four-worker output agree, no ignored failures/diagnostics.
- [ ] Freeze complete delta, source hashes and logs; leave index empty and
  report root-ready. Do not commit/push/publish/dispatch or run cohort timing.

## Root integration

Read the whole change and obtain one fresh whole-feature review covering
ownership, bounds, cancellation and every consumer. Resolve findings, perform
an independent check and commit one concern. Capture immutable Git source and
run native-four/source qualification before one matched representative campaign.
No claim that this alone resolves cache reloads or all multiworker regressions.
