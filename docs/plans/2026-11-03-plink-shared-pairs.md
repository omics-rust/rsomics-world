# PLINK shared pair-data implementation plan

> Execute one coherent private feature wave under executing-plans or bounded
> delegated implementation. Unattended/direct-main authority is established;
> no approval checkpoint, per-helper review ladder or evidence deletion.

**Goal:** Prepare selected LD pair data once and share immutable compact rows
across compute lanes instead of independent reader/window preparation per lane.

**Architecture:** Byte-admit a compact shared representation before allocation.
Keep existing pair ordinals, numeric kernels and bounded ordered output. If the
optional representation cannot fit, use the existing window/serial path.

**Tech stack:** Existing locked Rust1.91 dependencies; no new crate or C++ ABI.

**Spec:** AGENTS.md, genotype-popgen dossier, existing pairs CLI/numeric tests;
source analysis at
`/Volumes/KIOXIA/Developments/tmp/plink-green12-ld-source-analysis-20261103.s07vq9yg/REPORT.md`
(SHA256089aecd5881ae59da7a83b30287e63f7082ca3abcfabbe2644f68cba41f44a21).

## Global constraints

- Clean owner starts at1906a48b8e6e70bf2478c8e3daf1a44c38c56572 under
  `/Volumes/KIOXIA/Documents/omics-rust/rsomics-plink`.
- Build/test/scratch/cache only through the existing physically checked
  external guard. Scratch `/Volumes/KIOXIA/Developments/tmp`.
- Do not change make, prune policy, persisted formats, CLI, ordered channels,
  output block size, codec ABI, dependencies, public foundations or versions.
- No scientific rerun, remote upload, publication, commit or push by builder.
  Root integrates after one fresh whole-feature review and independent tests.
- Retain exact hardcall integer moments and dosage fallback arithmetic/order,
  major-frequency/prefix construction, six-significant formatting and errors.
- No SIMD redesign, complete-case arithmetic shortcut, batch-only tuning or
  formatter optimization in this wave. POPCNT is already present in real ELF.
- No performance claim from source inspection, tests or worker-count reports.

## Files and private seams

- Create `src/ld/prepared.rs`: private admission, compact owned slabs/row
  descriptors and borrowed row views. Own no FFI reader after preparation.
- Modify `src/ld/mod.rs`: shared admission alongside the existing fallback,
  one reusable `GenotypeReader::read_into` buffer during preparation.
- Modify `src/ld/pairs.rs`: share prepared data in serial and ordered jobs;
  preserve existing writer and Counts accumulation/report behavior.
- Modify `src/ld/moments.rs` only as needed for borrowed bitplane views and
  compact effective-dosage moments; old pruning consumers retain behavior.
- Extend existing LD tests/fixtures, not another external testing framework.

Root ruling during implementation also permits one existing assertion change
in `src/native_workflow.rs`: preserve its fixed-budget call/output comparison,
but relocate the old layout-specific mandatory-eviction assertion to the real
no-fit integrated LD consumer. See plink-shared-pairs-2026-11-03.md for the
actual14/15 failure and retained fallback requirement; no fixture/import change.

`PreparedPairs` is private owned immutable data. Its row lookup takes the
existing SelectedOrdinal and returns borrowed bitplanes/prefix and optional
effective dosage. No queued borrowed codec view or mutex-protected shared
reader. Keep the admission/model implementation narrow; exact internal helper
names may follow the existing modules without creating public API.

Use flat contiguous bitplanes and descriptor/prefix storage, not one full raw
Genotypes object per retained row. Required rows are the union of planned pair
windows, not unrelated selected variants. Prepare each required row once;
zero-pair inputs do not decode data gratuitously.

For dosage rows, retain exact founder-aligned effective values: explicit dosage
overrides hardcall; otherwise allele sum times16384; missing stays missing.
If a pair has dosage in either original row, both rows' effective values must
match the current dosage_moments, including the hardcall-only partner. Do not
silently use hardcalls when DS/HDS is present or reinterpret HDS as phased r2.
Preserve original-row dosage-presence eligibility rather than inferring it
only from whether selected founders received an explicit dosage.

## Admission and fallback

Account checked capacities for required-row mapping, descriptors, all slabs,
prefix bytes, metadata/index/planner, one reader workspace, peak reusable decode
scratch, output4MiB per active lane and ordered coordination/writer buffers.
Count actual allocation capacities and alignment, not only logical lengths.
Do not multiply the generic raw variant reserve by all rows or every lane.
Choose admitted active workers with the existing requested-worker/CPU bounds.
Long lines beyond4MiB retain the serial output fallback.

Failure to fit optional shared preparation returns to the existing admitted
window/serial execution; it must not reject formerly accepted low-memory work.
Actual I/O/parse/invariant errors propagate, never masquerade as no-fit.
Check sums before allocation; allocation may not transiently exceed the plan
while old caches/readers and the new slabs coexist. Keep prune resources intact.

## One coherent task

- [x] Add one meaningful RED consumer for repeated-window preparation: execute
  existing pairs on overlapping windows with four workers, assert full output
  and semantic report counts against the serial/oracle baseline, and assert one decode per
  required variant using test-only instrumentation. Before implementation the
  decode-count assertion must fail for the real duplicated-preparation reason.
- [x] Implement shared compact preparation and route serial/parallel pair
  calculation through it while keeping exact pair/job order and writer bytes.
- [x] Add one integrated mixed hardcall/DS/HDS, missingness and selected-founder
  consumer comparing complete output and semantic counts with the fallback;
  worker-count fields must report actual execution, not equal serial by fiat.
  Include a hardcall-only partner of a dosage row and non-word-aligned founders.
- [x] Exercise optional no-fit fallback and zero-pair behavior in existing
  budget tests; confirm previously admitted input still succeeds byte-exactly.
  Long-ID serial fallback remains covered by the existing test.
- [x] Run guarded full debug/release, fmt and strict all-target/all-feature
  Clippy. Expected full suite green, zero ignored diagnostics, no weakened old
  assertions or changed output pins. Record RED and final commands/statuses.
- [x] Freeze whole patch/source/log identities and allocation equations in an
  external packet. Owner index stays empty; root-ready, no speed claim.

## Review focus and final gates

Root integrated e210101 after one fresh whole-feature review with no findings,
307-body freeze verification and independent full debug15+3 pass. Native-four,
source and matched performance gates remain open; see the linked state ledger.

Review dosage-presence semantics outside selected founders, duplicate/overlap
window row mapping, capacity overflow and fallback at tight budgets, error
propagation during preparation, and exact output/job ordering. The integrated
tests above own these boundaries; do not add exhaustive scalar-helper tests.

Root reads complete source and one fresh whole-feature review, independently
verifies the whole suite and integrates one concern. Then perform native-four
and source compatibility qualification for that exact candidate. Only then
reuse the existing matched diagnostic protocol and unchanged product pins.
Do not rerun an unchanged full macro or relabel green12's negative result.

Ruling: shared immutable preparation is selected over a new SIMD kernel because
the source demonstrates independent per-lane preparation/layout work, while
actual ELF already has POPCNT and complete-case frequency is unmeasured. Cost
if ineffective: extra preparation/RSS may outweigh saved work; retain fallback
and require real matched measurements rather than promising a speedup.
