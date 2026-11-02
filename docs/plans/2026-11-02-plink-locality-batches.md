# PLINK locality-preserving bounded batches

One private-internal optimization wave on owning clean
`80fafbf8eaed950f03d7fed88dd641d58ca99882`. No public API, numerical policy,
durability, validation predicate or supported operation changes.

## Evidence and target

Authenticated run41421169780 completes26 graphs/210 stages but fails the
original performance gate: native whole medians8.57/9.79s at1/4 workers,
source4.63/2.62s. All seven stage medians regress. The bound original records
are retained in `.autopilot/state/plink-ordered-output-2026-11-02.md`.

Source review finds fine-grained ordered producer traffic in make, small
round-robin pair jobs with independent overlapping caches and a per-job output
flush. Root reads `make/pipeline.rs`, `ordered.rs` and the pairs execution
path. These are plausible causes, not measured causal attribution. Preserve
generation hashing, close-error propagation and every existing fsync.

## Implementation wave

1. Separate conservative memory admission from useful execution batch sizing.
   Make must amortize channel handoff over reusable, byte-bounded genotype
   batches rather than targeting only256KiB using worst-case128B/sample rows.
   Keep explicit optional GT/phase/dosage track accounting and existing budget
   fallbacks; no unsafe packed-only assumption for DS/HDS or multiallelic data.
2. Coarsen ordered pair work while preserving contiguous row/window locality
   inside each job. Avoid repeatedly preparing overlapping windows for tiny
   jobs and flushing almost-empty output blocks. Use existing bounded output
   streaming/recycling and deterministic row order, not unbounded result
   collection or one chromosome-sized queue. Consider load balance as well as
   locality; simply assigning one giant range per lane can serialize output.
3. Keep the solution inside the existing owner modules and ordered executor.
   No new scheduling framework, foundation, dependency or cosmetic refactor.
   Existing serial path, genotype semantics and public reports remain intact.
4. Reuse existing connected GT/DS/HDS make→LD fixtures and worker/budget tests.
   Add only focused cases that expose meaningful batching/locality invariants,
   short final blocks and deterministic full output. One debug/release/fmt/
   strict-Clippy wave, followed by one fresh integrated review. No helper
   review ladder or parameter sweep.
5. Root reviews source and actual memory bounds before integration/native4.
   Prepare a small diagnostic runner for I/pairs on representative prepared
   inputs, but do not execute a new scientific/timing campaign without root
   checking identities, outputs, capacity and command/resource recording.
   Full whole-workflow acceptance remains separate; never promote a local
   kernel improvement into a release or cross-platform performance claim.

All work uses verified external Cargo/Rustup/target/tmp/cache paths. Preserve
all old originals and negative results. No delete/stage/commit/push/publish
authority for implementation workers. Root owns those integration actions.
