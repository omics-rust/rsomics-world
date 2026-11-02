# PLINK bounded ordered production and output pipeline

> Execute one integrated private product wave with `superpowers:executing-plans`; one fresh whole-feature review and one material correction pass. No helper-by-helper review ladder.

**Goal:** Remove the proven LD temporary-part write/recopy path and overlap prepared-PGEN decoding with the existing single ordered encoder, without changing scientific output or durability.

**Architecture:** Product-private bounded producer lanes feed one calling-thread writer. Existing Rayon `ThreadPool::in_place_scope` keeps the coordinator outside its compute pool and permits the non-Send PGEN writer to stay on its owning thread. Reuse product-local coordination where the two paths genuinely share it; no new public crate or foundation API.

**Evidence:** Root-adopted negative green8 whole campaign and `plink-phase-diagnostic-2026-11-02.md`. Eight actual once-only host commands show two sequential four-part LD batches and native pairs4 output blocks1264552 versus source632280. Final file sync takes about1.7s on this traced host. I/F sync is only17.04%/11.51% of their traced windows, so substantial non-sync work remains. These observations do not predict a speedup or identify old-CI causality.

## Scope and invariants

- Owner `/Volumes/KIOXIA/Documents/omics-rust/rsomics-plink`, clean private mainf49f3675184801bb1e2cbf935f0ecadbc6a22e84. Freeze its current source/body/mode baseline before editing. Preserve all historical snapshots, failed campaigns and source provenance.
- Cargo/Rustup/target/tmp/cache/build output stay under the verified KIOXIA paths. Use the fully inspected owning `scripts/guarded-cargo.sh` with ambient CPPFLAGS removed, locked/offline dependencies and at least8GiB headroom. No new dependency, public API, flag, schema, numerical rule or default policy.
- Preserve exact record order, numeric formatting, genotype projection and track/tail-bit cleanup, reference flags, PGEN append order, generation hashes, input-change checks and explicit reader/writer close failures.
- Keep `AtomicFile::commit_all`, generation transaction verification, file fsync and parent-directory fsync unchanged. The separate unified durability-policy question is unanswered; no answer or default change is inferred.
- No metadata arena, C++ ABI/zero-copy shortcut, QC/prune algorithm rewrite, cached-complete-call LD branch, hash omission, benchmark gate waiver or unchanged diagnostic/campaign replay.
- Do not stage/commit/push/publish the owner or edit World. Root integrates after one complete review. No deletion or modification of remote originals/capacity reservations.

## One integrated implementation

Allowed production paths: `src/ld/pairs.rs`, narrowly necessary `src/ld/mod.rs` resource accounting, `src/make/mod.rs` and a product-private `src/make/pipeline.rs`; optionally one small shared private coordination module wired in `src/lib.rs`. Existing codec handles/ABI, transaction module and dependency graph stay unchanged. Any materially different necessary path needs a reason before editing, not a silent expansion.

### LD output

- Retain cost-aware contiguous scheduling and the serial path, not indivisible huge partitions. Use bounded consecutive task subpartitions so later producers do not fill tiny queues and stall for an entire preceding large range. Retain reader/cache state per lane where safe; avoid multiplying decoder initialization at each output block. Replace temporary files plus serial recopy with fixed-size text blocks and bounded backpressure, consumed in original order by the sole existing output writer.
- Spawn at most the admitted number of producer lanes, not every blocking partition. Each lane owns its ordered partition sequence; the coordinator drains known partition/lane order. A later partition may not occupy every compute thread while the required earlier producer remains unscheduled.
- Producer buffers, queued/in-flight blocks, coordinator-held blocks and channel/coordination storage must be covered by the memory admission. Keep real worker accounting and cache reservations truthful; no unbounded whole-partition Vec or implicit memory-budget exemption.
- On first producer/read/close or writer error, drop all receivers before scope join so blocked senders can exit. Return the actual meaningful error and never publish partial output. Do not mask reader errors or deadlock on sender disconnection. No unsafe Send/Sync on codec handles.

### Prepared-PGEN materialization

- Optimize only the existing prepared PGEN route. BED/VCF discovery/import and requested-one-worker execution retain the existing serial implementation.
- Decode owned, bounded genotype blocks on reader workers created inside those workers. Recycle returned block ownership and use existing read_into buffer reuse rather than allocating fresh vectors for every row. Account forward, return and in-flight blocks, and release both queue directions on cancellation. Never move the non-Send reader/native view across threads; only owned Rust Genotypes may cross the queue. Shared prepared index locking and tail-bit cleanup remain unchanged.
- Keep the only STPgenWriter on the calling thread and append blocks in original selected-variant order. At requested4, admit at most3 decoder workers plus the single encoder; requested2 can use1+1. Low memory reduces lanes/block width or retains serial execution without exceeding the admitted budget.
- Account persistent selectors/metadata, every reader workspace, encoder workspace, live input and projected genotype rows, queued/in-flight blocks, index metadata and coordination allocations before spawning. Bound blocks by bytes as well as row count. Do not queue enormous rows under an assumed typical input.
- Preserve identity/nonidentity sample selection, provisional flags, allele-count checks, reader-generation verification and explicit close/finish errors. Existing manifest and report fields retain their contract; do not fabricate compute-thread claims through `writer_workers=1`.

## Verification and review

- [x] Freeze source baseline and read complete relevant existing data/codec/memory/transaction and workflow-test contracts before editing.
- [x] Add a small connected regression first that exercises enough selected rows/output bytes for multiple blocks/partitions at1/4 workers, exact full output equality, a meaningful low-memory fallback, and one controlled consumer write-failure cancellation. Reuse existing rich GT/DS/HDS/multiallelic/projection and transaction fixtures instead of a combinatorial helper matrix.
- [x] Implement both bounded pipelines, preserving serial/source/transaction behavior and memory accounting. Keep comments rare and focused on non-obvious invariants.
- [x] Run one coherent formatting/strict-Clippy/full debug+release/existing complete workflow-oracle bundle plus the connected regression. Keep default ignored cases and any actual failures explicit. Do not run a full representative performance campaign at every internal edit.
- [x] Freeze candidate delta and obtain one fresh whole-feature review of both paths, cancellation/order/lifetimes, memory and retained durability. Correct material findings once and record declined/minor scope. Root independently inspects the final delta/evidence, not a second review ladder.

After local acceptance, root prepares the changed owning four-native gate and a single complete representative workflow comparison on the frozen candidate. Old failed whole-wall/RSS gates remain failed until actual new measurements pass. Pairs sync and remaining QC/prune losses may still prevent overall acceptance; never promise or silently redefine the gate.
