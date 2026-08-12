# VCF Typed Annotation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a complete, bounded-memory `rsomics-vcf annotate` command for typed header edits and annotation transfer, validated against bcftools 1.24 and released only after correctness, four-platform CI, and performance gates pass.

**Architecture:** Keep annotation policy private to `rsomics-vcf`. Parse edits and column mappings into checked plans before writing the output header, then process the target and optional annotation inputs as forward coordinate-sorted streams. Reuse the product's typed `RecordBuf`, expression, region, variant writer, `rsomics-common` transaction, and `rsomics-help` command tree; do not add a public foundation or revive a historical micro-crate.

**Tech Stack:** Rust 1.91 minimum and Rust 1.97.1 local toolchain, noodles VCF/BCF/BGZF types, existing rsomics expression and region modules, `rsomics-common 0.12`, `rsomics-help 0.4`, bcftools/HTSlib 1.24 oracles, Cargo tests and shell benchmark ledger.

## Global Constraints

- Work only in `/Volumes/KIOXIA/Documents/omics-rust/rsomics-vcf`; control-plane edits stay in `/Volumes/Zane's HDD/Documents/rsomics-world`.
- Before every build or test, require `CARGO_HOME=/Volumes/KIOXIA/Developments/cargo-home`, `CARGO_TARGET_DIR=/Volumes/KIOXIA/Developments/cargo-target/rsomics-vcf`, and `TMPDIR=/Volumes/KIOXIA/Developments/tmp`; stop if `/` reaches 80% use.
- Direct commits go to `main`, one independently reviewable concern per commit, with no `Co-Authored-By` line.
- Do not expose `annotate` in public help until every declared option in this plan is implemented and its ordinary tests pass.
- Use `rsomics-help` for the command tree and `rsomics-common::AtomicFile` for named output; `--json` requires named output.
- Source comments remain rare and explain only non-obvious stable invariants. Public help and README text describe user contracts.
- Production parse, schema, coordinate, compression, write, flush, and transaction errors propagate and exit nonzero.
- No new Layer A item is permitted: every new module in this plan is product-private.
- The 0.4 command excludes experimental merge logic, dynamic source-column expressions, `--force`, `--single-overlaps`, automatic output indexing, compression levels, and provenance header stamping.

## File Structure

```text
src/
├── annotate.rs                  stream orchestration, public-private options, summary
├── annotate/
│   ├── columns.rs              -c/-C grammar, field modes, header binding
│   ├── edit.rs                 fixed, INFO, FILTER, FORMAT, and genotype transfer
│   ├── header.rs               removals, retained complements, appended lines, renames
│   ├── matching.rs             spans, pair logic, overlap fractions, allele maps
│   ├── set_id.rs               checked site-format ID rendering
│   └── source.rs               sorted tabular and VCF/BCF annotation streams
├── commands/
│   └── annotate.rs             Clap adapter, output transaction, JSON separation
├── cli.rs                      public command and structured summary
├── commands/mod.rs             command adapter registration
├── lib.rs                      private annotate module registration
├── query.rs                    reusable site-format renderer
└── query_format.rs             site-only format validation
tests/
├── annotate_cli.rs             ordinary lifecycle, formats, transactions, failures
├── annotate_compat.rs          pinned bcftools 1.24 differential matrix
└── upstream/bcftools-annotate/ selected fixtures, README, upstream license
benchmarks/
└── annotate-vs-bcftools.sh     reproducible correctness-first performance gate
```

---

### Task 1: Checked column grammar and action model

**Files:**
- Create: `src/annotate.rs`
- Create: `src/annotate/columns.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Produces: `ColumnSpec::parse(&str) -> Result<ColumnSpec>` and `ColumnSpec::from_file(&Path) -> Result<ColumnSpec>`.
- Produces: `BoundColumns::bind(ColumnSpec, SourceKind, &Header, Option<&Header>) -> Result<BoundColumns>` for tabular or variant sources.
- Produces the exact shared types below for Tasks 4–7.

```rust
pub(crate) enum MatchField {
    Chrom,
    Pos,
    From,
    To,
    Ref,
    Alt,
    Id,
    End,
    Ignore,
}

pub(crate) enum MatchLayout {
    Position,
    Interval,
}

pub(crate) enum SourceKind {
    Tabular,
    Variant,
}

pub(crate) enum SourceField {
    Id,
    Qual,
    Filter,
    Info(String),
    Format(String),
    AllInfo,
    AllFormat,
    Tabular(usize),
}

pub(crate) enum Destination {
    Id,
    Qual,
    Filter,
    Info(String),
    Format(String),
    AllInfo,
    AllFormat,
}

pub(crate) enum WriteMode {
    Replace,
    ReplaceMissing,
    Add,
    AddMissing,
    Append,
    AppendMissing,
    ReplaceExisting,
}

pub(crate) struct Transfer {
    pub(crate) source: SourceField,
    pub(crate) destination: Destination,
    pub(crate) mode: WriteMode,
}

pub(crate) enum Column {
    Match(MatchField),
    Transfer(Transfer),
}
```

- [x] **Step 1: Add failing parser tests**

Test literal match columns, `INFO/TAG`, `FMT/TAG`, `DST:=SRC`, ignored `-`, `~ID`, `~INFO/END`, all seven write-mode prefixes, whole INFO/FORMAT, and malformed combinations. Assert that tabular plans require exactly one CHROM plus either POS or FROM/TO, while VCF/BCF plans infer coordinate columns.

```rust
#[test]
fn parses_match_and_transfer_columns() {
    let plan = ColumnSpec::parse(
        "CHROM,FROM,TO,REF,ALT,+INFO/DB,FMT/NEW:=FMT/OLD,~ID",
    )
    .unwrap();
    assert_eq!(plan.match_layout(), MatchLayout::Interval);
    assert_eq!(plan.transfers().len(), 2);
}

#[test]
fn rejects_ambiguous_coordinate_and_mode_grammar() {
    for value in ["POS,FROM,TO,DB", "CHROM,FROM,DB", "CHROM,POS,++DB"] {
        assert!(ColumnSpec::parse(value).is_err(), "{value}");
    }
}
```

- [x] **Step 2: Run the focused test and confirm RED**

```bash
env CARGO_HOME=/Volumes/KIOXIA/Developments/cargo-home \
    CARGO_TARGET_DIR=/Volumes/KIOXIA/Developments/cargo-target/rsomics-vcf \
    TMPDIR=/Volumes/KIOXIA/Developments/tmp \
    /opt/homebrew/Cellar/rust/1.97.1/bin/cargo test annotate::columns --lib
```

Expected: compilation fails because `ColumnSpec` and the grammar types do not exist.

- [x] **Step 3: Implement the grammar without record mutation**

Parse prefixes in the order `.+`, `.=` , `.`, `+`, `=`, `-`; parse rename syntax only once; normalize bare tags to INFO destinations; reject duplicate match roles and duplicate destination writes. `from_file` reads one nonempty, non-comment column expression per line and joins them in file order.

```rust
impl ColumnSpec {
    pub(crate) fn parse(source: &str) -> Result<Self> {
        let raw = split_checked_csv(source)?;
        let fields = raw.into_iter().map(parse_column).collect::<Result<Vec<_>>>()?;
        validate_layout(&fields)?;
        Ok(Self { fields })
    }
}

fn split_checked_csv(source: &str) -> Result<Vec<&str>>;
fn parse_column(source: &str) -> Result<Column>;
fn validate_layout(fields: &[Column]) -> Result<()>;
```

- [x] **Step 4: Run focused tests and strict Clippy**

```bash
env CARGO_HOME=/Volumes/KIOXIA/Developments/cargo-home \
    CARGO_TARGET_DIR=/Volumes/KIOXIA/Developments/cargo-target/rsomics-vcf \
    TMPDIR=/Volumes/KIOXIA/Developments/tmp \
    /opt/homebrew/Cellar/rust/1.97.1/bin/cargo test annotate::columns --lib
env CARGO_HOME=/Volumes/KIOXIA/Developments/cargo-home \
    CARGO_TARGET_DIR=/Volumes/KIOXIA/Developments/cargo-target/rsomics-vcf \
    TMPDIR=/Volumes/KIOXIA/Developments/tmp \
    /opt/homebrew/Cellar/rust/1.97.1/bin/cargo-clippy --all-targets --all-features -- -D warnings
```

Expected: parser tests pass and Clippy reports no warnings.

- [x] **Step 5: Commit the grammar**

```bash
git add src/lib.rs src/annotate.rs src/annotate/columns.rs
git commit -m "feat(vcf): parse typed annotation columns"
git push origin main
```

Wait for exact-head four-native-target CI before continuing.

Completed in `3bcbb84`: eight focused grammar tests and all 131 library tests
pass, strict all-target/all-feature Clippy is clean on Rust 1.97.1, and exact-head
CI run `31638366855` passed on all four native target classes.

---

### Task 2: Header edits, removals, and renames

**Files:**
- Create: `src/annotate/header.rs`
- Modify: `src/annotate.rs`

**Interfaces:**
- Consumes: `Destination` from Task 1.
- Produces: `HeaderPlan::bind(&Header, HeaderOptions) -> Result<HeaderPlan>`.
- Produces: `HeaderPlan::prepare(&self, &mut Header) -> Result<()>` and `HeaderPlan::apply(&self, &mut RecordBuf) -> Result<bool>`.

```rust
pub(crate) struct HeaderOptions {
    pub(crate) appended: Vec<String>,
    pub(crate) remove: Option<String>,
    pub(crate) rename_chromosomes: Option<PathBuf>,
    pub(crate) rename_annotations: Option<PathBuf>,
}

pub(crate) enum Removal {
    Id,
    Qual,
    Filters(Selection<String>),
    Info(Selection<String>),
    Format(Selection<String>),
}

pub(crate) struct Selection<T> {
    pub(crate) keep: bool,
    pub(crate) values: Vec<T>,
}

pub(crate) struct Renames {
    pub(crate) chromosomes: HashMap<String, String>,
    pub(crate) infos: HashMap<String, String>,
    pub(crate) formats: HashMap<String, String>,
    pub(crate) filters: HashMap<String, String>,
}

pub(crate) struct HeaderPlan {
    appended: Vec<String>,
    removals: Vec<Removal>,
    renames: Renames,
}
```

- [x] **Step 1: Add failing header and record-edit tests**

Cover removal of ID/QUAL/all FILTER/all INFO/all FORMAT, selected-tag removal, `^` complements, GT removal, appended INFO/FORMAT/FILTER/contig definitions, chromosome rename, annotation rename, missing old tags, duplicate new tags, and two mappings targeting the same name.

```rust
#[test]
fn renames_header_and_record_keys_together() {
    let (mut header, mut record) = fixture();
    let plan = HeaderPlan::bind(
        &header,
        options_with_renames("chr1\t1\nINFO/OLD\tNEW\nFORMAT/X\tY\nFILTER/q10\tLowQ\n"),
    )
    .unwrap();
    plan.prepare(&mut header).unwrap();
    plan.apply(&mut record).unwrap();
    assert!(header.infos().contains_key("NEW"));
    assert_eq!(record.reference_sequence_name(), "1");
    assert!(record.info().get("NEW").is_some());
}
```

- [x] **Step 2: Run the focused test and confirm RED**

Run `cargo test annotate::header --lib` with the global external-disk environment. Expected: missing `HeaderPlan` symbols.

- [x] **Step 3: Implement checked header preparation and record edits**

Parse every appended line as a one-line VCF header and merge only INFO, FORMAT, FILTER, and contig records. Build all rename maps before mutating, reject collisions, then reconstruct the relevant ordered maps and record fields. Apply removals before renames, matching bcftools command order established by the pinned differential.

```rust
impl HeaderPlan {
    pub(crate) fn prepare(&self, header: &mut Header) -> Result<()> {
        apply_appended_lines(header, &self.appended)?;
        validate_removals(header, &self.removals)?;
        remove_header_definitions(header, &self.removals);
        validate_rename_targets(header, &self.renames)?;
        rename_header_maps(header, &self.renames)?;
        Ok(())
    }
}
```

- [x] **Step 4: Run focused and existing format tests**

Run `cargo test annotate::header --lib` and `cargo test --test view --test norm_cli`. Expected: all pass.

- [x] **Step 5: Commit the header engine**

```bash
git add src/annotate.rs src/annotate/header.rs
git commit -m "feat(vcf): edit annotation headers safely"
git push origin main
```

Wait for exact-head CI.

Completed in `1415e6c`: eight focused header/record tests, all 139 library
tests, and 31 existing `view`/`norm` integration tests pass; strict
all-target/all-feature Clippy is clean on Rust 1.97.1, and exact-head CI run
`31639880846` passed on all four native target classes.

---

### Task 3: Site-format ID rendering

**Files:**
- Create: `src/annotate/set_id.rs`
- Modify: `src/query.rs`
- Modify: `src/query_format.rs`
- Modify: `src/annotate.rs`

**Interfaces:**
- Produces: `query_format::SiteFormat::bind(&str, &HeaderTypes) -> Result<SiteFormat>`.
- Produces: `SiteFormat::render(&self, &Header, &RecordBuf, &mut Vec<u8>) -> Result<()>`.
- Produces: `IdPlan::bind(Option<&str>, &HeaderTypes) -> Result<Option<IdPlan>>` and `IdPlan::apply(&mut self, &Header, &mut RecordBuf) -> Result<bool>`.

```rust
pub(crate) struct IdPlan {
    only_missing: bool,
    format: SiteFormat,
    scratch: Vec<u8>,
}
```

- [x] **Step 1: Add failing renderer and ID tests**

Reuse the query parser for `%CHROM`, `%POS`, `%ID`, `%REF`, `%ALT`, `%FIRST_ALT`, `%QUAL`, `%FILTER`, `%INFO/TAG`, `%TYPE`, subscripts, escaped literals, and `+FORMAT`. Reject sample loops, FORMAT/sample fields, newlines, tabs, empty output, and VCF-invalid whitespace.

```rust
#[test]
fn sets_only_missing_ids_from_site_fields() {
    let format = SiteFormat::bind("%CHROM_%POS_%REF_%FIRST_ALT", &schema()).unwrap();
    let mut record = record("chr1\t10\t.\tA\tC,G\t.\tPASS\t.");
    let mut output = Vec::new();
    format.render(&header(), &record, &mut output).unwrap();
    assert_eq!(output, b"chr1_10_A_C");
}
```

- [x] **Step 2: Run renderer tests and confirm RED**

Run `cargo test query_format::tests annotate::set_id --lib` with external paths. Expected: `SiteFormat` is absent.

- [x] **Step 3: Refactor one authoritative query renderer**

Move site-token validation and typed rendering behind `SiteFormat`; keep sample-loop rendering in `query`. Serialize `RecordBuf` into a reusable VCF line buffer, trim its line ending, and call the same fixed/info formatter used by query. Parse the leading `+` only as the set-if-missing policy.

- [x] **Step 4: Run query, annotate, and strict Clippy tests**

Run `cargo test --lib`, `cargo test --test query --test query_cli`, and strict all-target Clippy. Expected: existing query bytes remain unchanged and the ID tests pass.

- [x] **Step 5: Commit the shared renderer**

```bash
git add src/query.rs src/query_format.rs src/annotate.rs src/annotate/set_id.rs
git commit -m "feat(vcf): render annotation IDs from site fields"
git push origin main
```

Wait for exact-head CI.

Completed in `40c7d09`: five focused site-format and ID tests, all 144
library tests, and 11 existing query integration tests pass; strict
all-target/all-feature Clippy is clean on Rust 1.97.1, and exact-head CI run
`31640941696` passed on all four native target classes. The renderer reuses
its serialized-line, column-span, and output allocations across records.

---

### Task 4: Sorted annotation source readers

**Files:**
- Create: `src/annotate/source.rs`
- Modify: `src/annotate.rs`
- Modify: `src/annotate/columns.rs`
- Test assets: copy selected historical BED fixture into `tests/upstream/bcftools-annotate/`

**Interfaces:**
- Consumes: `BoundColumns` from Task 1.
- Produces the owned source record and reader below.

```rust
pub(crate) struct AnnotationRecord {
    pub(crate) contig: usize,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) serial: u64,
    pub(crate) reference: Option<String>,
    pub(crate) alternates: Vec<String>,
    pub(crate) id: Option<String>,
    pub(crate) info_end: Option<usize>,
    pub(crate) payload: Payload,
    pub(crate) zero_based: bool,
}

pub(crate) enum Payload {
    Variant(Box<RecordBuf>),
    Tabular(Vec<Option<Vec<u8>>>),
}

enum SourceReader {
    Variant {
        reader: Reader,
        header: Header,
        scratch: RecordScratch,
    },
    Tabular {
        reader: Box<dyn BufRead>,
        line: Vec<u8>,
        columns: BoundColumns,
        bed: bool,
    },
}

pub(crate) struct AnnotationSource {
    reader: SourceReader,
    next: Option<AnnotationRecord>,
    active: VecDeque<AnnotationRecord>,
    contigs: HashMap<String, usize>,
    last_coordinate: Option<(usize, usize, u64)>,
}
```

- [x] **Step 1: Add failing source-reader tests**

Cover sorted plain/BGZF BED, sorted plain/gzip/BGZF tabular input, all four VCF/BCF encodings, blank/comment tab lines, BED zero-based half-open conversion, tabular one-based inclusive conversion, CRLF, long fields, missing columns, invalid integers, end-before-start, unknown contigs, coordinate regressions, truncated gzip/BGZF/BCF, and source standard-input rejection.

```rust
#[test]
fn bed_and_tab_coordinates_remain_distinct() {
    let bed = read_one("regions.bed", "CHROM,FROM,TO,DB");
    let tab = read_one("regions.tsv", "CHROM,FROM,TO,DB");
    assert_eq!((bed.start, bed.end), (9, 20));
    assert_eq!((tab.start, tab.end), (10, 20));
}
```

- [x] **Step 2: Run source tests and confirm RED**

Run `cargo test annotate::source --lib`. Expected: missing source-reader implementation.

- [x] **Step 3: Implement streaming readers with bounded buffers**

Use `Reader` and `RecordScratch` for variant sources. Use `BufRead` plus `MultiGzDecoder` for tabular sources and detect BED by case-insensitive `.bed`, `.bed.gz`, `.bed.bgz`, or `.bed.bgzf` suffix. Map names to the target header's contig order and retain one lookahead record.

- [x] **Step 4: Run source tests and malformed-input regression**

Run `cargo test annotate::source --lib` plus `cargo test --test validate --test index`. Expected: all pass without writing to the boot disk.

- [x] **Step 5: Commit the source readers**

```bash
git add src/annotate.rs src/annotate/columns.rs src/annotate/source.rs tests/upstream/bcftools-annotate
git commit -m "feat(vcf): stream sorted annotation sources"
git push origin main
```

Wait for exact-head CI.

Completed in `e120382`: nine source-reader tests cover plain, gzip, BGZF, raw
BCF, compressed BCF, BED coordinate semantics, CRLF and long records, sorted
target-contig order, malformed values, truncation, and standard-input rejection.
All 154 library tests and the full non-oracle integration suite pass locally;
the boxed variant payload keeps enum size bounded without changing ownership.
Exact-head CI run `31642574358` passed on all four native target classes after
rerunning a Linux oracle job whose first download attempt received HTTP 503.

---

### Task 5: Forward overlap and allele matching

**Files:**
- Create: `src/annotate/matching.rs`
- Modify: `src/annotate/source.rs`
- Modify: `src/annotate.rs`

**Interfaces:**
- Consumes: `AnnotationRecord` and target `RecordBuf`.
- Produces: `AnnotationSource::first_match(&mut self, &RecordBuf, PairLogic, OverlapFractions) -> Result<Option<Matched<'_>>>`.

```rust
pub(crate) enum PairLogic {
    Snps,
    Indels,
    Both,
    All,
    Some,
    Exact,
    Id,
}

pub(crate) struct OverlapFractions {
    pub(crate) annotation: f64,
    pub(crate) target: f64,
}

pub(crate) struct Matched<'a> {
    pub(crate) source: &'a AnnotationRecord,
    pub(crate) allele_map: Vec<Option<usize>>,
}
```

- [x] **Step 1: Add failing coordinate, pair, and resource-bound tests**

Cover point and span matches; END and REF-derived target spans; REF/ALT, ID, and END constraints; all pair-logic values; symbolic, breakend, spanning-deletion, reference-only, and mixed records; reciprocal overlap boundaries; repeated coordinates; contig transitions; and a million sorted annotations with active memory bounded by maximum overlap.

```rust
#[test]
fn forward_join_discards_expired_annotations() {
    let mut source = generated_source(1_000_000, 5);
    for target in generated_targets(1_000_000) {
        source.first_match(&target, PairLogic::Some, OverlapFractions::default()).unwrap();
        assert!(source.active_len() <= 5);
    }
}
```

- [x] **Step 2: Run matching tests and confirm RED**

Run `cargo test annotate::matching --lib`. Expected: missing matching types and functions.

- [x] **Step 3: Implement the active-window join and allele correspondence**

Advance source records while their start can overlap the current target, remove expired records, preserve source order among active records, and return the first compatible match. Construct an allele map from source REF/ALT indices to target indices and fail only when a requested allele-indexed transfer cannot be represented.

- [x] **Step 4: Run matching tests under debug and release profiles**

Run `cargo test annotate::matching --lib` and `cargo test --release annotate::matching --lib`. Expected: identical results and bounded active-state assertions.

- [x] **Step 5: Commit the matcher**

```bash
git add src/annotate.rs src/annotate/source.rs src/annotate/matching.rs
git commit -m "feat(vcf): match sorted annotation streams"
git push origin main
```

Wait for exact-head CI.

Completed in `bbcb609`: 12 focused tests cover sorted active-window matching,
point and interval coordinates, REF- and END-derived spans, reciprocal overlap,
tabular constraints, every pair-logic class, symbolic and mixed alleles,
allele remapping, repeated coordinates, contig transitions, and one million
source rows while retaining at most five active records. The focused matcher
finishes in about 0.45 seconds in the local release profile; all 167 library
tests, the non-oracle integration suite, and strict all-target Clippy pass.
Exact-head CI run `31644600764` passed on all four native target classes,
including the pinned bcftools 1.24 compatibility gate on Linux x86_64.

---

### Task 6: Fixed-field and INFO transfer

**Files:**
- Create: `src/annotate/edit.rs`
- Modify: `src/annotate/columns.rs`
- Modify: `src/annotate.rs`
- Modify: `src/norm.rs`
- Modify: `src/norm/cardinality.rs`

**Interfaces:**
- Consumes: `Matched`, `BoundColumns`, target/output `Header`.
- Produces: `Editor::bind(&Header, Option<&Header>, BoundColumns) -> Result<Editor>`.
- Produces: `Editor::apply_info(&self, &Header, &Matched, &mut RecordBuf) -> Result<bool>`.
- Promotes the module and `norm::cardinality::{combinations, infer_ploidy}` to `pub(crate)` for product-internal reuse.

- [ ] **Step 1: Add failing fixed and INFO transfer tests**

Cover ID, QUAL, FILTER, one INFO tag, renamed source/destination tags, all INFO, flags, integer/float/character/string values, missing values, every write mode, array append, duplicate suppression where the mode requires it, and schema mismatches. Cover `Number=A`, `R`, and `G` for reordered, subset, and extended target alleles with haploid, diploid, triploid, and mixed source records.

```rust
#[test]
fn remaps_number_r_to_target_alleles() {
    let matched = matched("A", &["G", "C"], "A", &["C", "G"]);
    let value = info_integer_array(&[10, 2, 3]);
    assert_eq!(remap_info(Number::ReferenceAlternateBases, value, &matched).unwrap(),
               info_integer_array(&[10, 3, 2]));
}
```

- [ ] **Step 2: Run edit tests and confirm RED**

Run `cargo test annotate::edit --lib`. Expected: missing `Editor` and remapping functions.

- [ ] **Step 3: Implement typed fixed and INFO mutation**

Copy source header definitions before output header emission. Apply write modes without string round trips. For `Number=G`, enumerate canonical genotype combinations with checked arithmetic and map every source genotype cell to the target allele order. Reject type, count, or ploidy ambiguity instead of dropping values.

- [ ] **Step 4: Run edit, norm, and full library tests**

Run `cargo test annotate::edit --lib`, `cargo test norm --lib`, and `cargo test --lib`. Expected: annotation and normalization cardinality tests all pass.

- [ ] **Step 5: Commit typed field transfer**

```bash
git add src/annotate.rs src/annotate/columns.rs src/annotate/edit.rs src/norm.rs src/norm/cardinality.rs
git commit -m "feat(vcf): transfer typed annotation fields"
git push origin main
```

Wait for exact-head CI.

---

### Task 7: FORMAT, genotype, and sample transfer

**Files:**
- Modify: `src/annotate/edit.rs`
- Modify: `src/annotate/columns.rs`
- Modify: `src/annotate.rs`

**Interfaces:**
- Adds: `SampleSelection::bind(&Header, &Header, Option<&str>, Option<&Path>) -> Result<SampleSelection>`.
- Adds: `Editor::apply_samples(&self, &Header, &Matched, &mut RecordBuf) -> Result<bool>`.

```rust
pub(crate) struct SampleSelection {
    source_to_target: Vec<(usize, usize)>,
}
```

- [ ] **Step 1: Add failing FORMAT and sample tests**

Cover automatic same-name mapping, ordered inclusion, exclusion with `^`, samples file, missing source or target samples, GT phasing and ploidy, scalar and array FORMAT values, all write modes, absent target FORMAT keys, whole FORMAT transfer except GT, explicit GT transfer, and `Number=A/R/G` remapping per sample.

```rust
#[test]
fn transfers_selected_samples_by_name_not_position() {
    let selection = SampleSelection::bind(
        &header_with_samples(&["A", "B"]),
        &header_with_samples(&["B", "A"]),
        Some("A"),
        None,
    )
    .unwrap();
    assert_eq!(selection.pairs(), &[(0, 1)]);
}
```

- [ ] **Step 2: Run FORMAT tests and confirm RED**

Run `cargo test annotate::edit::tests::format --lib`. Expected: sample-transfer interface is absent.

- [ ] **Step 3: Implement keyed sample mutation**

Resolve samples by header name once, retain target sample order, extend the target FORMAT key list deterministically, initialize unselected samples as missing for newly introduced keys, and remap genotype allele indices through `Matched::allele_map`. Fail on unavailable selected names and impossible genotype alleles.

- [ ] **Step 4: Run edit, view sample, and norm sample tests**

Run `cargo test annotate::edit --lib`, `cargo test --test view`, and `cargo test norm:: --lib`. Expected: all pass.

- [ ] **Step 5: Commit sample transfer**

```bash
git add src/annotate.rs src/annotate/columns.rs src/annotate/edit.rs
git commit -m "feat(vcf): transfer annotation sample fields"
git push origin main
```

Wait for exact-head CI.

---

### Task 8: Complete streaming command and unified CLI

**Files:**
- Modify: `src/annotate.rs`
- Create: `src/commands/annotate.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/cli.rs`
- Modify: `README.md`
- Create: `tests/annotate_cli.rs`

**Interfaces:**
- Produces the complete stream contract below and exposes it only after all ordinary tests pass.

```rust
pub(crate) struct Options {
    pub(crate) source: Option<SourceOptions>,
    pub(crate) header: HeaderOptions,
    pub(crate) set_id: Option<String>,
    pub(crate) set_id_missing_only: bool,
    pub(crate) expression: Option<String>,
    pub(crate) expression_logic: Logic,
    pub(crate) keep_sites: bool,
    pub(crate) mark_sites: Option<MarkSites>,
    pub(crate) regions: Option<RegionSet>,
    pub(crate) output_format: OutputFormat,
}

pub(crate) struct SourceOptions {
    pub(crate) path: PathBuf,
    pub(crate) columns: ColumnSpec,
    pub(crate) samples: Option<SampleRequest>,
    pub(crate) pair_logic: PairLogic,
    pub(crate) min_overlap: OverlapFractions,
}

pub(crate) enum SampleRequest {
    Names { values: Vec<String>, exclude: bool },
}

pub(crate) enum MarkSites {
    Present(String),
    Absent(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct Summary {
    pub(crate) read: u64,
    pub(crate) written: u64,
    pub(crate) annotated: u64,
    pub(crate) unchanged: u64,
    pub(crate) filtered: u64,
    pub(crate) output_format: OutputFormat,
}

pub(crate) fn write(input: &Path, options: &Options, output: impl Write) -> Result<Summary>;
pub(crate) fn write_parallel<W: Write + Send + 'static>(
    input: &Path,
    options: &Options,
    output: W,
    workers: usize,
) -> Result<Summary>;
```

- [ ] **Step 1: Add failing lifecycle and help tests**

Test no-action rejection; source-without-columns; columns-without-source; mark-without-source; mutually exclusive expression, region, sample, and header-file options; all output types; stdin target; named output; JSON separation; BGZF workers; indexed regions; expression drop and keep-unchanged behavior; mark-present and mark-absent; write/flush failures; same input/output aliases; and old-file preservation after a late malformed record.

Assert the unified help contains every declared option and none of the excluded flags.

- [ ] **Step 2: Run CLI tests and confirm RED**

Run `cargo test --test annotate_cli`. Expected: the public command is absent.

- [ ] **Step 3: Implement orchestration before exposing the command**

Read and validate the target header, bind header edits and the optional source, prepare the complete output header, bind expressions and ID formatting, then emit records through `Writer` or `ParallelWriter`. For indexed regions use `IndexedRecords`; otherwise use `Reader`. Apply target selection, removals, renames, ID edits, source matching, field transfer, and mark-sites in the pinned bcftools order. Call `finish()` before committing named output.

- [ ] **Step 4: Wire the authoritative command tree and README**

Expose `Command::Annotate`, `CommandOutput::Annotate`, and `commands::annotate::Arguments`. Use readable long names with bcftools-compatible short aliases where they do not conflict with unified `-h`: `--header-lines` has no short alias and `-H` remains repeatable `--header-line`.

```rust
/// Edit and transfer VCF or BCF annotations
Annotate(commands::annotate::Arguments),
```

- [ ] **Step 5: Run the ordinary product gate**

Run formatting with the explicit Rust 1.97.1 `rustfmt`, strict all-target/all-feature Clippy, `cargo test`, `cargo test --release`, rustdoc with warnings denied, and `cargo package --locked`. Expected: no warning, failure, unfinished public option, or boot-disk output.

- [ ] **Step 6: Commit the complete public command**

```bash
git add src/annotate.rs src/commands/annotate.rs src/commands/mod.rs src/cli.rs README.md tests/annotate_cli.rs
git commit -m "feat(vcf): add typed annotation workflow"
git push origin main
```

Wait for exact-head four-native-target CI, including the ordinary annotate suite on every target.

---

### Task 9: Pinned bcftools 1.24 differential and attribution

**Files:**
- Create: `tests/annotate_compat.rs`
- Add: `tests/upstream/bcftools-annotate/README.md`
- Add: selected files under `tests/upstream/bcftools-annotate/`
- Modify: `THIRD_PARTY_LICENSES.md`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Adds an ignored live suite selected with `RSOMICS_BCFTOOLS` and a CI-pinned bcftools 1.24 build.
- Normalizes only bcftools provenance header lines before comparing complete headers and record bodies.

- [ ] **Step 1: Copy and fingerprint the selected official fixtures**

Select official cases for fixed and typed fields, all write modes, remove complements, rename collisions, interval and allele matching, pair logic, sample mapping, `Number=A/R/G`, mark-sites, overlap fractions, regions, and malformed inputs. Record bcftools tag/version, source paths, license, and SHA-256 values in the fixture README.

- [ ] **Step 2: Add failing differential cases**

For each case, generate required BGZF/CSI/TBI artifacts in an external temporary directory, run both binaries, decode BCF to canonical VCF through pinned bcftools, remove only `##bcftools_` provenance lines, and compare complete remaining bytes and exit success/failure.

```rust
#[test]
#[ignore = "requires bcftools 1.24"]
fn typed_annotation_matrix_matches_bcftools_1_24() {
    for case in cases() {
        case.assert_complete_equivalence();
    }
}
```

- [ ] **Step 3: Run the live matrix and fix every semantic mismatch**

```bash
env RSOMICS_BCFTOOLS=/opt/homebrew/bin/bcftools \
    CARGO_HOME=/Volumes/KIOXIA/Developments/cargo-home \
    CARGO_TARGET_DIR=/Volumes/KIOXIA/Developments/cargo-target/rsomics-vcf \
    TMPDIR=/Volumes/KIOXIA/Developments/tmp \
    /opt/homebrew/Cellar/rust/1.97.1/bin/cargo test --test annotate_compat -- --ignored --nocapture
```

Expected: every declared case matches complete normalized output or the same fail-loud decision.

- [ ] **Step 4: Add the pinned CI invocation and rerun full gates**

Linux x86_64 builds bcftools 1.24 from the existing SHA-256-pinned archive and runs `annotate_compat`; the other three native targets run all ordinary tests. Run the complete local debug/release, Clippy, rustdoc, and package gates again.

- [ ] **Step 5: Commit compatibility evidence**

```bash
git add tests/annotate_compat.rs tests/upstream/bcftools-annotate THIRD_PARTY_LICENSES.md .github/workflows/ci.yml
git commit -m "test(vcf): pin annotation compatibility"
git push origin main
```

Wait for exact-head four-native-target CI and record its run ID.

---

### Task 10: Performance gate, release, and independent verification

**Files:**
- Create: `benchmarks/annotate-vs-bcftools.sh`
- Modify: `PERFORMANCE.md` if present, otherwise create it
- Modify: `Cargo.toml`
- Modify: `README.md`
- Modify: `/Volumes/Zane's HDD/Documents/rsomics-world/docs/10-products/variant.md`

**Interfaces:**
- Benchmark accepts explicit rsomics binary, bcftools binary, target, annotation source, column plan, and result directory.
- Every timed run first proves complete normalized output equality and records fixture, output, command, binary, environment, timing, CPU, and peak-RSS fingerprints.

- [ ] **Step 1: Add the correctness-first benchmark harness**

Generate two non-trivial workloads on external storage: a multi-million-record BED-to-INFO interval join with bounded overlap and a multi-sample typed VCF-to-VCF transfer with reordered alleles and `Number=A/R/G` fields. Use three warmups and at least ten alternating measured pairs per workload.

- [ ] **Step 2: Run release benchmarks and make a measured decision**

Run on the Apple M2 reference host against bcftools/HTSlib 1.24. Require one complete representative path to show strict throughput or resource-use advantage. If neither workload passes, profile the measured hot path, make one evidence-led optimization, and rerun the same fixtures; do not change semantics or invent a favorable microbenchmark.

- [ ] **Step 3: Record the complete evidence ledger**

Write exact revision, OS, CPU, memory, Rust, bcftools/HTSlib, commands, fixture construction, SHA-256 values, raw timing distribution, paired summary, CPU, RSS, output equality, and the publication decision. State losses and parity honestly.

- [ ] **Step 4: Run the final pre-release review**

Review every production annotate module for error propagation, unchecked allocation, accidental whole-file retention, repeated parsing, comments, public help accuracy, and product-specific code that escaped into Layer A. Run format, strict Clippy, debug/release tests, live oracle, rustdoc, package verification, and exact-head four-target CI.

- [ ] **Step 5: Prepare and publish 0.4.0**

Set `version = "0.4.0"`, update the lockfile and README release surface, commit `chore: prepare rsomics-vcf 0.4.0`, push, wait exact release-head CI, and trigger the existing GitHub publication workflow from that exact head.

- [ ] **Step 6: Independently verify crates.io**

Download the static registry archive to a new external directory, compare its checksum with the crates.io API, verify `.cargo_vcs_info.json`, diff the unpacked tree against `cargo package`, perform a fresh locked registry install to an external root, check `--version` and unified `annotate --help`, and rerun one complete BED and one typed VCF oracle through the installed binary.

- [ ] **Step 7: Close the control-plane gate**

Update `docs/10-products/variant.md` with the exact release head, CI and publish run IDs, registry archive checksum and size, installed binary checksum, oracle output fingerprints, and measured performance decision. Validate `scripts/validate_control_plane.py`, commit `docs(vcf): record annotation release`, push, and wait exact-head control-plane CI.
