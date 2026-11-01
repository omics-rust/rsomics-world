# PLINK prepared PGEN and resident LD window

## Authority and scope

Spec: AGENTS.md and docs/10-products/genotype-popgen.md. Root adopts wave B
from plink-source-wave-design-20261101.fSV9DW/REPORT.md,
SHA672e57195626a2f6fa3627409067f2a27117277fa8227cbf80032228748151ac.
Base is clean private main5ec85fd1363ca21cac621a3a682b684f688448e1.
HWE wave A is complete: one whole review, full local/real F contracts, actual
green5 native-four/source41319636397/1 and complete originals are root-adopted
in plink-native-2026-11-01-green-5.json. Do not repeat A or its native campaign.

One coherent private data-plane wave eliminates repeated PGEN index/header/
REF preparation and per-pair generic-map lookups. It introduces no new
operation, public API/crate, data scope, numerical rule or performance claim.
The original whole929-person seven-stage negative result remains authority.

## Global constraints

- Owner: /Volumes/KIOXIA/Documents/omics-rust/rsomics-plink, direct main.
  This owner intentionally has no remote; do not create one or publish.
- Cargo home /Volumes/KIOXIA/Developments/cargo-home; rustup rustup-home;
  target /Volumes/KIOXIA/Developments/cargo-target/rsomics-plink; every TMP,
  cache, report, fixture copy and task ledger is on approved external disks.
  Existing owning guarded-cargo.sh admits all builds/tests; unset CPPFLAGS.
  Clippy uses its dedicated `clippy` branch, offline/locked dependencies.
- Mac may run this Rust product and stdlib data checks, not upstream/foreign
  scientific tools. No network/download, Linux run, profile, CI/API mutation,
  key access, deletion, Git stage/commit/push or publication by implementer.
- Preserve HWE A, numeric.rs, source comparison/goldens, vendor/dependencies,
  all durability/atomic output rules, stdout/report schemas and error exits.
  Do not weaken tie/epsilon/threshold/missingness/REF/phase/dosage contracts.
- Keep source comments rare: explain only necessary unsafe/ownership invariants.
  No phase/audit/history narration, generic frameworks or speculative Layer A.
- A single implementer owns the coupled source wave; no child subagents or
  worker-spawned reviews. Root owns one integrated spec+quality review and
  concentrated correction. Do not build helper-level review/test ladders.

## Task B: one prepared source, task-local readers, bounded LD workspace

Likely paths: src/io/pgen.rs, native/pgen_shim.h/.cc, src/dataset/mod.rs,
src/make/mod.rs/filter.rs, src/stats/mod.rs, src/ld/mod.rs/pairs.rs/prune.rs,
existing connected tests and README/MERGE_RECORD. A narrow private module is
acceptable if it separates reader ownership from LD window policy; do not
split one-method files or restructure unrelated code. Vendor stays unchanged.

### Prepared PGEN ownership

Read actual vendored Pgfi/Pgr initialization/cleanup before changing FFI.
Prepare/validate immutable offsets/header/index/REF/track flags once per
Dataset invocation. Header index outlives every borrowed task-local reader;
mutable FILE/cursor/LD-base/decode buffers are never shared between workers.
Retain process-global reader initialization locking for upstream global state
and serialize the first header-FILE transfer and later reader factory calls.
Do not mark mutable FILE/index state unconditionally Sync. Prefer private
borrowed Reader lifetimes and scoped ownership over Arc/global registries.

Dataset::reader returns a correctly bounded borrowed trait object. BED/VCF
adaptation and full owned Genotypes/read_into behavior stay unchanged.
Resolve REF once without a full decode-reader probe; LD workspace queries use
prepared requirements rather than allocating another PGEN reader. Bind an
opened reader handle to the prepared file generation (device/inode/size/
change time as appropriate), preserving existing manifest validation. Do not
claim complete concurrent-file mutation isolation from those checks.

Every successful path explicitly closes all readers and then the prepared
owner, propagating CleanupPgr/CleanupPgfi/fclose failures before output commit.
Drop is error-path cleanup, not the only successful close. C++ exceptions
remain inside the FFI boundary. Keep unsafe justification narrow and local.

Budget shared index/offsets/REF once in resident/fixed memory, and reader
workspaces per concurrent reader. Admit corresponding allocation before it
occurs; plumb the existing invocation budget where needed, not a second
configuration system. Never double-count the old monolithic workspace or
omit the new owner/factory allocation. Preserve the2GiB real-profile budget.

### LD resident selected-ordinal window

Keep existing packed bitplanes, POPCNT dispatch, major-frequency/prefix work
and T1 direct output; these are not new optimizations. Introduce selected
ordinal identity distinct from raw VariantId and compute maximum live span
from sorted metadata with an O(V) window scan, not pair enumeration.

Each existing weighted part gets a bounded ring/dense window. Refill/evict
before borrowing a row, then compare fixed a with resident b slices without
per-pair ensure/HashMap/Box churn. Wrap may produce two slices. Cross-part
edge rereads may remain; do not promise zero duplicate decode. Reuse a single
VariantData construction path rather than duplicating dosage/kernel policy.

Prune keeps its exact order2 reverse/newly-added traversal. Already removed b
may still participate in its current window; do not compact/reorder it early.
Eviction follows the actual active/window boundary. Preserve chromosome/
coordinate/selected-gap behavior, hardcall/dosage branches, strict r2+epsilon,
row/list order, thresholds and all full output populations.

Count index/window descriptors plus workers*(reader+live variants+I/O/scratch)
before allocation. If the full live span does not fit the existing budget,
retain the verified bounded evicting path as fallback: no unbounded matrix or
new rejection just to enable dense mode. Keep numeric formatting and its row
allocations unchanged in B; allocation-free rendering is explicitly excluded.

### Integrated verification and handoff

Extend the existing connected workflow for changed ownership/reader lifecycle
and window behavior, not a new testing framework. Keep all six input lanes,
36 generations/74878 raw cells, HWE/tiny-tail/rounding and1/4 equality; include
selected gaps/wrap/part/chromosome and prune order2 through actual outputs.
Observe one meaningful connected RED for repeated preparation/window work,
then GREEN. Do not assert a made-up CPU time or use an empty test/mocks.

Use retained full real F assets as read-only originals, with independent
typed copies in the new private packet. Existing CAS-INDEX at
/Volumes/Zane's HDD/rsomics-fixtures/evidence/plink-whole-real-green4-20261101.zCJ7RN
maps complete source/native reports. Do not import its top-level assess.py.
Where the current runner/comparator permits, check the complete real F pair/
prune/QC outputs at1/4 once against the fixed source/old-native contract;
33780398 evaluated pairs,2527798 emitted,696 kept/22333 removed stay fixed.
This is own-Rust correctness evidence, not a new performance campaign. Do not
create a new raw bridge or scientific oracle merely to expand test coverage.

Focused iteration is allowed; once source-final, run fmt/strict Clippy/full
debug and release once. Freeze complete before/after tracked source, diff,
protected identities and raw checks in one private external packet, bounded
to1GiB plus8GiB independent headroom; shared Cargo target remains guarded and
external. Preserve failures and all original bodies. Return one short report
with actual results and limitations, no stage/commit. Root owns the whole
review, main commit, exact new snapshot/native gate and final unchanged
seven-stage performance campaign. Do not rerun the A-only benchmark.
