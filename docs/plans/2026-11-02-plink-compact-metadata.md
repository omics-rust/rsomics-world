# PLINK compact metadata implementation plan

> Agentic workers use executing-plans for one coherent private-model wave and one fresh whole-feature review. Unattended/direct-main authority is established; no separate design approval, helper-review ladder or scratch deletion. Root owns commits/native4/performance/publication.

**Goal:** Remove per-field allocation and duplicate typed strings from PLINK
metadata without changing any input, numeric, output or durability contract.

**Architecture:** One immutable shared text arena per parsed table owns text
and flat field boundaries. Rows reference consecutive field indices; typed
IDs/alleles reference owned arena ranges. Small headers/preambles remain
ordinary strings. This is product-private storage, not a generic public table
crate or borrowed self-referential model.

**Tech stack:** Rust1.91, standard-library Arc/ranges and existing dependencies.

**Spec:** rsomics-world/AGENTS.md; `docs/10-products/plink.md`;
`.autopilot/state/plink-locality-batches-2026-11-02.md` and root-adopted
`plink-targeted-diagnostic-2026-11-02-green-10.json`.

## Global constraints

- Owner `/Volumes/KIOXIA/Documents/omics-rust/rsomics-plink`, clean9d43df4.
- Existing guarded Cargo paths/toolchain/scratch must physically resolve to
  approved external disks before every build/test/download. Scratch is a new
  namespace under `/Volumes/KIOXIA/Developments/tmp`.
- No CLI/JSON schema, input acceptance, genotype/numeric kernel, C++ ABI,
  generation/hash/fsync, dependency, version or public foundation change.
- PGEN/PVAR output is transformed, not an identity copy; no bypass.
- Existing sample ordinals, founder/sex/SID defaults, multiallelic and PR
  handling, BED allele orientation, split-PAR and VCF revalidation remain.
- No raw source-body or genotype download, new benchmark or remote execution
  in the implementation task. Source review motivates a storage change;
  causal timing attribution and performance acceptance are not pre-earned.
- Comments stay rare. No unsafe lifetime extension or leaked string buffers.
- Do not modify unrelated user work, stage/commit/push/delete/publish.

## Review focus

1. Cross-arena equality/hash must compare field contents, never arena identity
   or ordinal. Existing VCF second-pass checks remain meaningful.
2. Unknown columns, headers, whitespace tokenization and INFO/PR rewriting
   retain exact existing serialization; no implicit source-field loss.
3. Owned clones survive source reader/table drop and allele substrings remain
   valid UTF-8. No self-reference, unchecked narrowing or unsafe code.
4. Shared heap is counted once while owned vectors/alleles/other chromosome
   strings and reader clone buffers are still charged. Do not game admission.
5. PGEN/BED/VCF construction and all statistics/LD consumers use the same
   model; no optimized PGEN path plus an accidental legacy duplicate model.

## One task: coherent metadata representation and consumer migration

**Files:** new `src/dataset/metadata/text.rs`; modify
`src/dataset/metadata.rs`, `src/dataset/mod.rs`, `src/io/{bed,vcf}.rs` and
only the make/stats/LD consumer sites that depend on owned string APIs.
No changes to genotype encoding, ordering channels or pair job sizes.

**Private interfaces:**

```rust
// Names may be refined locally; these ownership/semantic contracts are fixed.
struct Text { /* Arc to sized Arena + byte range, no per-value string */ }
struct Fields { /* Arc to Arena + consecutive field-index range */ }
struct RowsBuilder { /* growing text/boundaries/row ranges */ }
impl RowsBuilder {
    fn push_row<'a>(&mut self, fields: impl IntoIterator<Item=&'a str>);
    fn finish(self) -> Vec<Fields>;
}
impl Fields {
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<&str>;
    fn iter(&self) -> impl Iterator<Item=&str>;
    fn text(&self, index: usize) -> Option<Text>;
}
impl Text { fn as_str(&self) -> &str; }
```

Use usize ranges, not a new4GiB input cap. Arena construction preserves valid
UTF-8 token bytes and field boundaries, including legitimate empty values
when constructors supply them. Text equality/order/hash/display are string-
content semantics across independent arenas. Fields equality is row-content
semantics. Defaults such as sample0 can share an arena constant rather than
allocate per default. Allele substrings share the same owned text; no unsafe
view or per-field Arc<str> allocation. Arc to a sized Arena keeps reference
storage compact. Row field boundaries are flat, not Vec<String> per row.

- [ ] Add the real resource regression before implementation: parse PSAM
  plus10,000 valid five-column PVAR rows, compare owned metadata accounting
  with a fixed byte budget. First measure/retain the old count, then set one
  literal useful limit below that count that the compact representation can
  meet. Expected RED is existing duplicate storage exceeding that budget,
  not a benchmark or arbitrary struct-size/text assertion. Preserve fixture
  and exact old result; do not delete failure data.
- [ ] Add one small consumer-visible roundtrip with unknown PSAM/PVAR
  columns, UTF-8 values, multiple ALT alleles and INFO/PR updates. Literal
  full output bytes derive from the current writer's retained pre-change
  output. Exercise a clone after original metadata/source buffers drop.
  Existing actual VCF workflow tests own cross-pass/arena semantic equality;
  do not add tests for every getter or layout constant.
- [ ] Implement the private arena/row/text ownership model and migrate
  Sample/Variant textual fields plus every constructor (PGEN, BED and VCF).
  VCF metadata should finalize a shared table, while its transient second-pass
  record may own a separate arena; contents must still compare equal.
- [ ] Adapt consumer lookups/formatting with borrowed string access, preserving
  selection and order. Keep sample selector semantics; do not redesign the
  selector file grammar or other chromosomes in this wave.
- [ ] Update metadata_bytes and VCF reader/make/LD workspace estimates.
  Count unique arena backing once per live ownership scope, vector capacity
  and owned allele containers accurately; document only the non-obvious
  shared-ownership invariant if needed. Do not drop budget checks or rely on
  allocator-private layouts. A conservative small Arc/control allowance is
  acceptable; representation tests cannot replace code review of accounting.
- [ ] Run guarded fmt, strict all-target/all-feature Clippy, full debug/release
  tests including the existing complete native workflow and raw GT/DS/HDS
  cases. Expected: all existing and new tests pass; no suppressed diagnostic.
  Do not claim remote representative speed/RSS improvement from unit budgets.
- [ ] Freeze the exact complete source delta and logs, including actual RED,
  literal pre-change outputs, final test populations and known limitations.
  Leave owning index empty and report root-ready; no partial CLI advertised.

## Root integration and next evidence

Root reads the full delta and obtains one fresh-context whole-feature review,
rules on findings, performs a focused independent check and commits the one
concern. A new immutable source capture and actual four-native/source-consumer
qualification precede representative timing. Reuse the same retained I/pairs
cohort/oracle bindings for one later matched campaign; no changing inputs,
durability or population to manufacture a win. The current slow measurements
remain authoritative until new same-campaign evidence exists. Pairs scheduling
is a separate substantive work item, not silently bundled into this refactor.
