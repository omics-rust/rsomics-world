# rsomics-bed Relations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add byte-checked `cluster`, `window`, and single-database `closest` commands to `rsomics-bed` without reviving historical micro-crates or weakening the five published operations.

**Architecture:** Extend the existing byte-preserving `BedRecord`, extract the COITrees coordinate backend behind one private index core, and build a full-record relation wrapper for neighborhood queries. `cluster` remains independent: unstranded mode streams in constant space, while same-strand mode buffers one chromosome to match BEDTools' chromosome-local `+` then `-` output order.

**Tech Stack:** Rust 1.91, edition 2024, `clap` 4.5, `rsomics-help` 0.4, `rsomics-common` 0.11, `rsomics-intervals` 0.3, `coitrees` 0.4, BEDTools 2.31.1, Criterion 0.7.

**Spec:** `docs/superpowers/specs/2026-08-20-rsomics-bed-relations-design.md`

## Global Constraints

- Work only in `/Volumes/KIOXIA/Documents/omics-rust/rsomics-bed`; control-plane records stay in `/Volumes/Zane's HDD/Documents/rsomics-world`.
- Before every Cargo command, require boot-disk use below 80%, inspect KIOXIA free space, and export `CARGO_HOME=/Volumes/KIOXIA/Developments/cargo-home`, `CARGO_TARGET_DIR=/Volumes/KIOXIA/Developments/cargo-target/rsomics-bed-relations`, and `TMPDIR=/Volumes/KIOXIA/Developments/tmp`.
- BEDTools must report exactly `bedtools v2.31.1`; live compatibility tests never skip a missing or changed oracle.
- Preserve `BedRecord` bytes and validate BED4/BED6 fields only when a selected option consumes them.
- Production parse, I/O, ordering, coordinate, and compatibility failures propagate and leave named outputs unchanged.
- Keep the COITrees `i32::MAX - 1` coordinate limit explicit; never cast or clamp an unrepresentable coordinate.
- Do not add a Layer A API: the index, tie, strand, zero-length, and output policies remain product-private.
- Keep source comments rare and limited to stable compatibility reasons or non-obvious invariants.
- Commit one independently reviewable concern at a time, push directly to `main`, and wait for exact-head native Linux/macOS `x86_64`/`aarch64` CI after every push.
- Publish 0.2.0 only after all three commands, the full regression suite, package verification, representative performance, and exact-head CI pass.

Before each Cargo command, run this preflight and stop if it fails:

```bash
test "$(df -P / | awk 'NR==2 {gsub(/%/, "", $5); print $5}')" -lt 80
df -h / /Volumes/KIOXIA
export CARGO_HOME=/Volumes/KIOXIA/Developments/cargo-home
export CARGO_TARGET_DIR=/Volumes/KIOXIA/Developments/cargo-target/rsomics-bed-relations
export TMPDIR=/Volumes/KIOXIA/Developments/tmp
test "$(bedtools --version)" = "bedtools v2.31.1"
cargo metadata --no-deps --format-version 1 | jq -er '.target_directory | startswith("/Volumes/KIOXIA/Developments/cargo-target/")'
```

---

### Task 1: Mode-aware BED fields

**Files:**
- Modify: `src/bed.rs`

**Interfaces:**
- Produces initially: `BedRecord::line_number() -> usize`, `BedRecord::field_count() -> usize`, and the checked `BedRecord::strand(&str) -> Result<Strand>` contract.
- Adds joined and appended-column writers with the first relation consumer. Adds name access only when `closest --different-name` consumes it.
- Preserves the private `Strand::{Forward, Reverse}` model without speculative helpers.
- Preserves: BED3 parsing and all current raw/coordinate writers byte for byte.

- [ ] **Step 1: Add failing parser and writer tests**

Add unit tests proving that BED3 remains valid until strand is requested, BED6 accepts only `+` and `-`, CRLF input stays normalized to one LF, and appended columns preserve every original field. Add name and joined-output cases with their first consumers.

```rust
let record = read_records(&b"chr1\t10\t20\tname\t0\t-\r\n"[..]).unwrap().remove(0);
assert_eq!(record.line_number(), 1);
assert_eq!(record.field_count(), 6);
assert_eq!(record.strand("closest A").unwrap(), Strand::Reverse);
```

- [ ] **Step 2: Verify the new tests fail for missing accessors**

Run: `cargo test --locked bed::tests`

Expected: compilation fails because the new field accessors and writers do not exist.

- [ ] **Step 3: Extend the owned record without eagerly validating optional fields**

Store the physical line number when `ParsedRecord` becomes `BedRecord`. Locate requested optional fields by splitting `raw` on tabs; return `InvalidInput` with the operation label and physical line number for missing, empty, or invalid consumed fields. Implement `write_joined` and `write_column` with `rs_context` on every write.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Strand {
    Forward,
    Reverse,
}

pub(crate) fn strand(&self, label: &str) -> Result<Strand> {
    match self.field(5, label, "strand")? {
        b"+" => Ok(Strand::Forward),
        b"-" => Ok(Strand::Reverse),
        value => Err(invalid(format!(
            "{label} BED line {}: invalid strand {:?}",
            self.line_number,
            String::from_utf8_lossy(value)
        ))),
    }
}
```

- [ ] **Step 4: Run focused and existing record tests**

Run: `cargo test --locked bed::tests`

Expected: all parser, optional-field, and byte-preservation tests pass.

- [ ] **Step 5: Commit the record contract**

```bash
git add src/bed.rs
git commit -m "refactor(bed): expose checked relation fields"
```

### Task 2: Complete cluster command

**Files:**
- Create: `src/cluster.rs`
- Modify: `src/lib.rs`
- Modify: `src/cli.rs`
- Modify: `tests/cli.rs`
- Modify: `tests/compat.rs`
- Create: `tests/golden/cluster.input.bed`
- Create: `tests/golden/cluster.default.expected.bed`
- Create: `tests/golden/cluster.distance.expected.bed`
- Create: `tests/golden/cluster.strand.input.bed`
- Create: `tests/golden/cluster.strand.expected.bed`

**Interfaces:**
- Consumes: Task 1 `BedRecord` field and output methods.
- Produces: public `cluster::ClusterOptions { distance: u64, same_strand: bool }` and `cluster::cluster(input: impl Read, output: impl Write, options: ClusterOptions) -> Result<()>`.
- Produces: CLI `rsomics-bed cluster [BED] [-o BED] [--distance BP] [--strand any|same|-s]`.

- [ ] **Step 1: Freeze exact oracle cases before implementing**

Create committed fixtures from BEDTools 2.31.1 for default overlap/book-end clustering, `-d 5`, bridge intervals, chromosome changes, duplicate/zero-length rows, and same-strand interleaving. The same-strand expected file must show `+` rows before `-` rows within each chromosome.

```bash
bedtools cluster -i tests/golden/cluster.input.bed > /Volumes/KIOXIA/Developments/tmp/cluster.default.oracle.bed
bedtools cluster -d 5 -i tests/golden/cluster.input.bed > /Volumes/KIOXIA/Developments/tmp/cluster.distance.oracle.bed
bedtools cluster -s -i tests/golden/cluster.strand.input.bed > /Volumes/KIOXIA/Developments/tmp/cluster.strand.oracle.bed
cmp /Volumes/KIOXIA/Developments/tmp/cluster.default.oracle.bed tests/golden/cluster.default.expected.bed
cmp /Volumes/KIOXIA/Developments/tmp/cluster.distance.oracle.bed tests/golden/cluster.distance.expected.bed
cmp /Volumes/KIOXIA/Developments/tmp/cluster.strand.oracle.bed tests/golden/cluster.strand.expected.bed
```

- [ ] **Step 2: Add failing library and CLI tests**

Test default, distance, strand reordering, global one-based IDs, start-order violation, chromosome reappearance, invalid BED6 strand, `u64` distance overflow, stdin, JSON/named-output separation, output aliases, and the `-s` alias conflict with `--strand`.

```rust
let mut output = Vec::new();
cluster(
    &b"chr1\t1\t10\nchr1\t15\t20\n"[..],
    &mut output,
    ClusterOptions { distance: 5, same_strand: false },
).unwrap();
assert_eq!(output, b"chr1\t1\t10\t1\nchr1\t15\t20\t1\n");
```

- [ ] **Step 3: Verify cluster tests fail because the module and command are absent**

Run: `cargo test --locked cluster tests::top_level_and_nested_help_render committed_golden_outputs_match live_bedtools_231_compatibility`

Expected: compilation or assertion failure naming the missing cluster module/command.

- [ ] **Step 4: Implement unstranded streaming and chromosome-buffered same-strand execution**

Use checked `end.checked_add(distance)` reach, a closed-chromosome set, and nondecreasing start validation. In same-strand mode, collect only the current chromosome, validate every strand, then run the same clustering state first over forward records and then reverse records; increment one global cluster counter across both groups and chromosomes.

```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClusterOptions {
    pub distance: u64,
    pub same_strand: bool,
}

pub fn cluster(
    input: impl Read,
    output: impl Write,
    options: ClusterOptions,
) -> Result<()>;
```

- [ ] **Step 5: Wire the typed CLI through shared parsing and transactional output**

Resolve `--strand` as `any` by default and `same` when either `--strand same` or `-s` is present. Declare the two spellings mutually exclusive so no precedence is invented. Reuse `require_named_json_output`, `reject_output_alias`, `open_input`, and `write_output`.

- [ ] **Step 6: Run the cluster contract and the full existing suite**

Run: `cargo fmt --all -- --check && cargo clippy --locked --all-targets --all-features -- -D warnings && cargo test --locked --all-features`

Expected: all six commands pass, including live BEDTools 2.31.1 differentials and the original five-operation regressions.

- [ ] **Step 7: Commit, push, and wait for exact-head CI**

```bash
git add src/cluster.rs src/lib.rs src/cli.rs tests/cli.rs tests/compat.rs tests/golden/cluster.*
git commit -m "feat(bed): add compatible cluster operation"
git push https://github.com/omics-rust/rsomics-bed.git HEAD:main
```

Fetch `main` into `refs/remotes/origin/main`, then require every job in the exact-head Actions run to conclude successfully before Task 3.

### Task 3: One private coordinate-index core

**Files:**
- Create: `src/interval_index.rs`
- Modify: `src/overlap_index.rs`
- Modify: `src/intersect.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Produces: private `IndexedInterval { start: u64, end: u64, overlap_start: u64, overlap_end: u64 }`.
- Produces: private `IntervalIndex::build(&[(String, u64, u64)], &str) -> Result<Self>`, `IntervalIndex::record(usize) -> IndexedInterval`, and `IntervalIndex::intersection_candidates(&str, u64, u64, &str, &mut Vec<usize>) -> Result<()>`.
- Preserves: `IndexedBed::load`, `IndexedBed::record`, `IndexedBed::intersection_candidates`, `CoverageBed`, and every published intersect/subtract behavior.

- [ ] **Step 1: Add core tests around duplicate IDs and the backend boundary**

Move no production code first. Add tests that build one coordinate core with duplicate bounds and zero-length records, assert candidate IDs return in original record-ID order, accept `i32::MAX - 1` as the last inclusive coordinate, and reject `i32::MAX` with the input label.

- [ ] **Step 2: Verify the tests fail because `IntervalIndex` does not exist**

Run: `cargo test --locked interval_index`

Expected: compilation fails on the missing module and types.

- [ ] **Step 3: Extract the existing COITrees machinery without changing algorithms**

Move `ChromIndex`, bound grouping, virtual overlap bounds, coordinate checking, and candidate sorting into `interval_index.rs`. Keep the target record ID as metadata and retain the SIMD/scalar `meta_id!` compatibility macro. Make `IndexedBed` a coordinate-only reader and wrapper; leave merged subtraction coverage separate.

```rust
pub(crate) fn intersection_candidates(
    &self,
    chrom: &str,
    start: u64,
    end: u64,
    label: &str,
    ids: &mut Vec<usize>,
) -> Result<()>;
```

- [ ] **Step 4: Prove the refactor is behavior-neutral**

Run: `cargo test --locked --all-features intersect subtract committed_golden_outputs_match live_bedtools_231_compatibility`

Expected: every old byte-level output and every intentional fail-loud divergence remains unchanged.

- [ ] **Step 5: Run strict local verification and commit**

Run: `cargo fmt --all -- --check && cargo clippy --locked --all-targets --all-features -- -D warnings && cargo test --locked --all-features`

```bash
git add src/interval_index.rs src/overlap_index.rs src/intersect.rs src/lib.rs
git commit -m "refactor(bed): separate coordinate index core"
```

### Task 4: Full-record relation index

**Files:**
- Create: `src/relation_index.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: Task 3 `IntervalIndex` and Task 1 `BedRecord`.
- Produces for `window`: private `RelationBed::load(input: impl Read, label: &str) -> Result<Self>`, `record(usize) -> &BedRecord`, `len() -> usize`, and file-ordered range candidates.
- Guarantees: uniform B column count, duplicate multiplicity, original B-file IDs, shared virtual bounds, and no unused closest-only API. Directional orderings are added only when failing `closest` tests establish their contract.

- [ ] **Step 1: Add failing relation-index tests**

Test full raw record retention, stable duplicate IDs, variable B widths rejected with both physical line numbers, range candidate ordering, zero-length virtual footprints, and absent chromosomes.

```rust
let relation = RelationBed::load(
    &b"chr1\t30\t40\tfirst\nchr1\t10\t20\tsecond\nchr1\t30\t40\tlast\n"[..],
    "B",
).unwrap();
assert_eq!(relation.len(), 3);
```

- [ ] **Step 2: Verify the new module tests fail**

Run: `cargo test --locked relation_index`

Expected: compilation fails because `RelationBed` is absent.

- [ ] **Step 3: Implement one relation wrapper over the coordinate core**

Load B through `BedReader`, validate a uniform field count, retain owned records in file order, and build the shared coordinate index. Return range hits in B-file order. Keep closest-specific directional structures out until the closest eligibility and tie tests consume them.

- [ ] **Step 4: Run relation, index, and existing operation tests**

Run: `cargo test --locked interval_index relation_index intersect subtract`

Expected: new structure tests and all old index consumers pass.

- [ ] **Step 5: Commit the private shared relation mechanism**

```bash
git add src/relation_index.rs src/lib.rs
git commit -m "refactor(bed): add private relation index"
```

### Task 5: Complete window command

**Files:**
- Create: `src/window.rs`
- Modify: `src/lib.rs`
- Modify: `src/cli.rs`
- Modify: `tests/cli.rs`
- Modify: `tests/compat.rs`
- Create: `tests/golden/window.a.bed`
- Create: `tests/golden/window.b.bed`
- Create: `tests/golden/window.pairs.expected.bed`
- Create: `tests/golden/window.asymmetric.expected.bed`
- Create: `tests/golden/window.strand.expected.bed`
- Create: `tests/golden/window.count.expected.bed`
- Create: `tests/golden/window.any.expected.bed`
- Create: `tests/golden/window.none.expected.bed`

**Interfaces:**
- Consumes: `RelationBed`, `BedRecord::strand`, raw/joined/column writers.
- Produces: public `window::StrandFilter::{Any, Same, Opposite}`, `window::WindowReport::{Pairs, Any, Count, None}`, `window::WindowOptions`, and `window::window(a: impl Read, b: impl Read, output: impl Write, options: WindowOptions) -> Result<()>`.
- Produces: CLI `window -a BED -b BED [-o BED]` with `--window`, paired `--left/--right`, `--strand-relative`, `--strand`, `--report`, and compatible `-u/-c/-v` aliases.

- [ ] **Step 1: Freeze pair, asymmetric, strand-relative, and report-mode oracles**

Generate exact BEDTools 2.31.1 output for default `-w 1000`, `-l/-r`, `-sw`, `-sm`, `-Sm`, `-u`, `-c`, and `-v`. Include no-hit A rows, B-file order, duplicates, zero-length records, and negative-strand A rows.

- [ ] **Step 2: Add failing option and output tests**

Test that `--window` conflicts with side options, left/right are a required pair, `--strand-relative` rejects missing A strand, strand filters reject missing B strand, upper-bound expansion overflow fails, lower-bound expansion saturates at zero, and all four report modes write exact bytes.

```rust
let options = WindowOptions {
    left: 10,
    right: 20,
    strand_relative: false,
    strand: StrandFilter::Any,
    report: WindowReport::Count,
};
window(a.as_slice(), b.as_slice(), &mut output, options).unwrap();
assert_eq!(output, b"chr1\t100\t200\tA\t2\n");
```

- [ ] **Step 3: Verify window tests fail because the command is absent**

Run: `cargo test --locked window`

Expected: compilation or command-tree failure naming the missing window API.

- [ ] **Step 4: Implement indexed window queries and report reducers**

Convert A to BEDTools virtual bounds, apply left/right with checked upper addition and zero-bounded lower subtraction, swap sides only for reverse A under strand-relative mode, query `RelationBed`, filter strand eligibility, and preserve B-file order. Pair mode emits every A/B join; any and none emit A once; count appends one decimal count.

- [ ] **Step 5: Wire canonical typed CLI values and conflict-checked aliases**

Resolve report aliases before opening input. `-u` maps to any, `-c` to count, and `-v` to none; Clap rejects multiple report selectors. BEDTools uses the operation-specific `-sm` and `-Sm` spellings, so the product keeps one canonical `--strand any|same|opposite` interface instead of introducing misleading `-s/-S` aliases. B remains a named file, and shared output alias/JSON handling applies unchanged.

- [ ] **Step 6: Run full strict verification and live oracle comparisons**

Run: `cargo fmt --all -- --check && cargo clippy --locked --all-targets --all-features -- -D warnings && cargo test --locked --all-features`

Expected: every window mode is byte-identical to the pinned oracle and the previous six commands remain green.

- [ ] **Step 7: Commit, push, and wait for exact-head CI**

```bash
git add src/window.rs src/lib.rs src/cli.rs tests/cli.rs tests/compat.rs tests/golden/window.*
git commit -m "feat(bed): add indexed window operation"
git push https://github.com/omics-rust/rsomics-bed.git HEAD:main
```

### Task 6: Closest distance and eligibility engine

**Files:**
- Create: `src/closest.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `RelationBed`, `BedRecord::name`, and `BedRecord::strand`.
- Produces: public `closest::DistanceMode::{None, Unsigned, Reference, A, B}`, `closest::TieMode::{All, First, Last}`, `closest::ClosestOptions`, and `closest::closest(a: impl Read, b: impl Read, output: impl Write, options: ClosestOptions) -> Result<()>`.
- Internal: `Candidate { id: usize, unsigned: u64, signed: i128 }` so signed orientation never overflows the `u64` coordinate domain.

- [ ] **Step 1: Freeze a closest semantic matrix from BEDTools**

Commit A/B fixtures and exact output covering overlaps, book-ended records, left/right equidistance, duplicate B records, nested records, zero length, absent chromosomes, empty B, BED3/BED4/BED6 placeholder widths, same/opposite strand, different name, ignored overlaps, three signed orientations, and all/first/last ties.

- [ ] **Step 2: Add failing distance and eligibility unit tests**

Assert overlap distance zero, non-overlap gap plus one, virtual zero-length bounds, reference left negative/right positive, A orientation reversal on negative A, B orientation based on each B strand, name inequality, strand eligibility, and ignored overlaps. Require consumed BED4/BED6 fields to fail with the correct side and physical line.

```rust
assert_eq!(unsigned_distance((10, 20), (20, 30)), 1);
assert_eq!(reference_distance((10, 20), (0, 5)), -6);
assert_eq!(reference_distance((10, 20), (25, 30)), 6);
```

- [ ] **Step 3: Verify focused tests fail on the absent engine**

Run: `cargo test --locked closest::tests`

Expected: compilation fails because the closest types and distance functions do not exist.

- [ ] **Step 4: Implement checked distance and eligibility types**

Use `i128` only for signed display and comparison; preserve coordinates as `u64`. Validate option-required fields lazily. First collect eligible overlaps; when none remain, advance the start/end directional iterators only until the minimum eligible unsigned distance is known on both sides. Preserve B-file IDs for tie handling.

- [ ] **Step 5: Pass the semantic engine tests**

Run: `cargo test --locked closest::tests relation_index`

Expected: all distance, direction, field-validation, and nearest-search tests pass without CLI code.

- [ ] **Step 6: Commit the closest engine**

```bash
git add src/closest.rs src/lib.rs
git commit -m "feat(bed): implement closest relation engine"
```

### Task 7: Complete closest CLI and byte-level output

**Files:**
- Modify: `src/closest.rs`
- Modify: `src/cli.rs`
- Modify: `tests/cli.rs`
- Modify: `tests/compat.rs`
- Create: `tests/golden/closest.a.bed`
- Create: `tests/golden/closest.b.bed`
- Create: `tests/golden/closest.*.expected.bed`

**Interfaces:**
- Produces: CLI `closest -a BED -b BED [-o BED] [--strand any|same|opposite] [--different-name] [--ignore-overlaps] [--distance none|unsigned|reference|a|b] [--tie all|first|last]`.
- Preserves: A streamability, named B requirement, transactional output, JSON separation, and exact uniform B-field placeholder width.

- [ ] **Step 1: Add failing output-order, placeholder, and CLI tests**

Test default B-file tie order, signed-mode directional order, first/last selection by B-file identity, no-hit and empty-B rows, optional distance `-1`, `-d`/`-D` aliases, `-t`, `-s/-S`, `-N`, `-io`, stdin A, alias rejection, and incompatible selector combinations.

- [ ] **Step 2: Verify tests fail before dispatch exists**

Run: `cargo test --locked closest tests::top_level_and_nested_help_render live_bedtools_231_compatibility`

Expected: command-tree or output assertions fail for the absent dispatch.

- [ ] **Step 3: Implement tie ordering and no-hit serialization**

For distance zero, order eligible B records by file ID. For nonzero ties, reproduce the pinned oracle order for the selected signed orientation before applying `all`, `first`, or `last`. Emit `.` for string B fields and `-1` for coordinate/score-like fields exactly as the frozen BED3/BED4/BED6 oracle matrix records; append `-1` when a distance mode is enabled and no B is eligible.

- [ ] **Step 4: Wire CLI selectors through one typed `ClosestOptions` conversion**

Parse every canonical long value and compatibility alias into one option value before opening A or B. Reject selector conflicts through Clap; do not accept multi-database, `k`, upstream/downstream preference, or BAM flags.

- [ ] **Step 5: Run strict full-suite verification**

Run: `cargo fmt --all -- --check && cargo clippy --locked --all-targets --all-features -- -D warnings && cargo test --locked --all-features && cargo test --release --locked --all-features && cargo package --locked`

Expected: all eight commands, live oracle matrices, package contents, and prior fail-loud behavior pass.

- [ ] **Step 6: Commit, push, and wait for exact-head CI**

```bash
git add src/closest.rs src/cli.rs tests/cli.rs tests/compat.rs tests/golden/closest.*
git commit -m "feat(bed): add compatible closest operation"
git push https://github.com/omics-rust/rsomics-bed.git HEAD:main
```

### Task 8: Seeded differential coverage

**Files:**
- Create: `tests/differential.rs`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

**Interfaces:**
- Consumes: public operation APIs and the pinned `bedtools` executable.
- Produces: deterministic generated-case comparisons for every declared option family without network, clock, or platform-dependent ordering.

- [ ] **Step 1: Add a failing deterministic generator test**

Use a fixed `ChaCha8Rng` seed and generate sorted BED3/BED4/BED6 records with duplicate coordinates, nesting, zero length away from origin, no-hit chromosomes, both strands, and bounded coordinates. Serialize each generated case once and invoke both implementations with matching flags.

- [ ] **Step 2: Add `rand` and `rand_chacha` only as dev dependencies**

```toml
rand = "0.9"
rand_chacha = "0.9"
```

- [ ] **Step 3: Implement 100 fixed cases per supported option family**

Compare exact bytes for declared compatible behavior. For explicit fail-loud divergences, assert the exact rsomics error and separately record the upstream output so the divergence cannot turn into a compatibility claim.

- [ ] **Step 4: Run debug and release differential lanes**

Run: `cargo test --locked --test differential && cargo test --release --locked --test differential`

Expected: all seeded comparisons pass against BEDTools 2.31.1.

- [ ] **Step 5: Commit differential evidence**

```bash
git add Cargo.toml Cargo.lock tests/differential.rs
git commit -m "test(bed): add relation differentials"
```

### Task 9: Documentation, benchmarks, and release gate

**Files:**
- Modify: `README.md`
- Modify: `MIGRATION.md`
- Modify: `PERFORMANCE.md`
- Modify: `benches/operations.rs`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `.github/workflows/ci.yml` only if the existing eight-command test and benchmark lanes do not exercise the new cases.
- Create: `docs/10-products/bed-performance-2026-08-20.md` in the control repository.

**Interfaces:**
- Produces: complete user-facing command/help contract, historical merge provenance, benchmark smoke coverage, and a reproducible representative release decision.
- Changes version: `0.1.0` to `0.2.0` only after every earlier gate is green.

- [ ] **Step 1: Extend benchmark smoke fixtures and byte checks**

Add 50,000-record cluster, sparse relation, and dense relation fixtures. Before timing, run rsomics and BEDTools once and compare complete output hashes. Benchmark both tools for unstranded cluster, same-strand cluster, window pairs/count, and closest default/distance; keep existing five operations and dense subtraction.

- [ ] **Step 2: Run benchmark smoke and inspect scaling**

Run: `cargo bench --locked --bench operations -- --test`

Expected: every new benchmark command succeeds and every pre-timing output comparison matches.

- [ ] **Step 3: Refresh README and migration provenance**

Document the eight installed subcommands, exact declared option surfaces, input/output behavior, index limit, same-strand cluster ordering, fail-loud divergences, and classifications of historical cluster/closest/window repositories. Remove the five-operation-only package description.

- [ ] **Step 4: Run the formal external-disk benchmark**

Build release binaries on the external target, generate and hash a multi-million-record cluster stream plus sparse and dense A/B relation fixtures under `/Volumes/Zane's HDD/rsomics-fixtures/`, verify exact outputs, and collect ten paired measurements after warmup for wall time, CPU, and peak RSS. Record machine, OS, source heads, oracle version and archive hash, commands, fixture hashes, output hashes, distributions, ratios, and a pass/fail decision per hot path.

- [ ] **Step 5: Re-run all published-operation performance checks**

Use the exact 0.1 representative fixture recipe after the shared index changes. Require each of sort, merge, intersect, subtract, and complement to retain its documented throughput or resource advantage; record regressions honestly and optimize before versioning if a required advantage is lost.

- [ ] **Step 6: Perform the final source and public API review**

Inspect every production `unwrap`, comment, public item, allocation in query loops, candidate ordering, coordinate cast, error conversion, and output write. Require no swallowed failures, speculative flags, duplicated relation parser, narrated comments, or public foundation additions.

- [ ] **Step 7: Bump to 0.2.0 and run the release command set**

Run: `cargo fmt --all -- --check && cargo clippy --locked --all-targets --all-features -- -D warnings && cargo test --locked --all-features && cargo test --release --locked --all-features && cargo package --locked && cargo bench --locked --bench operations -- --test`

Expected: every command exits zero with BEDTools 2.31.1 present and the package contains complete documentation for exactly eight stable operations.

- [ ] **Step 8: Commit, push, and require exact-head four-platform CI**

```bash
git add Cargo.toml Cargo.lock README.md MIGRATION.md PERFORMANCE.md benches/operations.rs .github/workflows/ci.yml
git commit -m "feat(bed): prepare 0.2.0 relation release"
git push https://github.com/omics-rust/rsomics-bed.git HEAD:main
```

- [ ] **Step 9: Publish only with a valid registry credential**

After exact-head CI succeeds, dispatch the repository release workflow, verify crates.io checksum and package install from a fresh external Cargo home, create the exact `v0.2.0` tag/release if the workflow does not, then record registry and CI evidence in the control dossier. If crates.io authentication remains expired, record the credential gate and continue the next unblocked family without weakening or repeating the release.
