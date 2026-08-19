# `rsomics-index` 0.1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build and release one coherent `rsomics-index` 0.1 product containing production-complete BGZF compression/indexing and local tabix build/query/list workflows.

**Architecture:** A product-private BGZF engine provides checked framing, bounded parallel deflate, text-aware blocks, GZI construction, and indexed reads. A single typed tabix record parser feeds both a sorted forward index builder and the final query overlap filter; command adapters own path, transaction, JSON, and compatibility policy.

**Tech Stack:** Rust 2024, Rust 1.91 minimum, clap 4.5, rsomics-common 0.12, rsomics-help 0.4, noodles 0.110, noodles-bgzf 0.47, libdeflater, crc32fast, crossbeam-channel, serde, tempfile, HTSlib 1.24 oracle.

**Spec:** `docs/superpowers/specs/2026-08-19-rsomics-index-design.md`

## Global Constraints

- Work only in `/Volumes/KIOXIA/Documents/omics-rust/rsomics-index`; the control-plane changes remain in `/Volumes/Zane's HDD/Documents/rsomics-world`.
- Before every Cargo command, verify `/` is below 80% and export `CARGO_HOME=/Volumes/KIOXIA/Developments/cargo-home`, `CARGO_TARGET_DIR=/Volumes/KIOXIA/Developments/cargo-target/rsomics-index`, and `TMPDIR=/Volumes/KIOXIA/Developments/tmp`.
- Keep BGZF mechanics product-private in 0.1; do not change a public foundation until a consumer-side extraction review after the product passes.
- Use `rsomics-help` for the command tree and `rsomics-common` for errors, JSON, output aliases, and atomic files.
- Named data and sidecar outputs are transactional; standard output never shares a stream with JSON.
- Comments are limited to public contracts or non-obvious stable invariants.
- Release compatibility is pinned to HTSlib 1.24 and release CI fails if the oracle cannot be built.
- Direct commits to `main`, one concern per commit, no coauthor, and exact-head CI after every push.

## Target file map

```text
rsomics-index/
├── .github/workflows/ci.yml       four-native-target CI and pinned oracle
├── benchmarks/index-vs-htslib.sh  reproducible release gate
├── src/
│   ├── bgzip/
│   │   ├── index.rs               GZI parse, scan, write, and lookup
│   │   ├── reader.rs              checked full and indexed decompression
│   │   └── writer.rs              bounded ordered parallel BGZF encoder
│   ├── commands/
│   │   ├── bgzip.rs               BGZF command validation and transactions
│   │   └── tabix.rs               tabix command validation and transactions
│   ├── tabix/
│   │   ├── build.rs               sorted scan and TBI/CSI accumulation
│   │   ├── config.rs              presets, detection, and stored header model
│   │   ├── index.rs               TBI/CSI load, output, and structural model
│   │   ├── query.rs               query planning, targets, and emission
│   │   └── record.rs              checked tabular interval parser
│   ├── bgzip.rs                   public BGZF options and summary
│   ├── cli.rs                     shared help tree and dispatch
│   ├── lib.rs                     narrow product library surface
│   ├── main.rs                    process entry
│   └── tabix.rs                   public tabix options and summaries
├── tests/
│   ├── bgzip_cli.rs
│   ├── bgzip_compat.rs
│   ├── tabix_cli.rs
│   ├── tabix_compat.rs
│   └── golden/
├── Cargo.toml
├── PERFORMANCE.md
├── README.md
└── THIRD_PARTY_LICENSES.md
```

---

### Task 1: Repository shell and fail-safe BGZF stream

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/main.rs`
- Create: `src/bgzip.rs`
- Create: `src/bgzip/writer.rs`
- Create: `src/bgzip/reader.rs`
- Create: `tests/bgzip_cli.rs`
- Create: `LICENSE-MIT`
- Create: `LICENSE-APACHE-2.0`
- Create: `.gitignore`

**Interfaces:**
- Produces: `bgzip::compress<R: Read, W: Write + Send + 'static>(source, sink, &CompressOptions) -> io::Result<(W, StreamStats)>`.
- Produces: `bgzip::decompress<R: Read, W: Write>(source, sink, Option<Range<u64>>) -> io::Result<StreamStats>`.
- Produces: `CompressOptions { level: u8, workers: NonZeroUsize, text: bool }` and `StreamStats { bytes_in: u64, bytes_out: u64, blocks: u64 }`.

- [ ] **Step 1: Create the minimum manifest and failing integration test**

```toml
[package]
name = "rsomics-index"
version = "0.1.0"
edition = "2024"
rust-version = "1.91"
license = "MIT OR Apache-2.0"
repository = "https://github.com/omics-rust/rsomics-index"
description = "BGZF, tabix, and sequence-resource indexing workflows"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
crc32fast = "1.5"
crossbeam-channel = "0.5"
libdeflater = "1"
noodles = { version = "0.110", features = ["bgzf", "core", "csi", "tabix"] }
noodles-bgzf = { version = "0.47", features = ["libdeflate"] }
rsomics-common = "0.12"
rsomics-help = "0.4"
serde = { version = "1", features = ["derive"] }

[dev-dependencies]
tempfile = "3"
```

```rust
#[test]
fn bgzf_round_trip_and_truncation() {
    let input = vec![b'A'; 200_000];
    let options = rsomics_index::bgzip::CompressOptions::default();
    let (encoded, stats) = rsomics_index::bgzip::compress(
        input.as_slice(), Vec::new(), &options,
    ).unwrap();
    assert!(stats.blocks >= 4);
    let mut decoded = Vec::new();
    rsomics_index::bgzip::decompress(encoded.as_slice(), &mut decoded, None).unwrap();
    assert_eq!(decoded, input);
    assert!(rsomics_index::bgzip::decompress(
        &encoded[..encoded.len() - 1], Vec::new(), None,
    ).is_err());
}
```

- [ ] **Step 2: Initialize the repository and verify the test fails**

Run:

```bash
git init -b main
git config user.name >/dev/null
git config user.email >/dev/null
cargo test --test bgzip_cli bgzf_round_trip_and_truncation -- --exact
```

Expected: compilation fails because `rsomics_index::bgzip` does not exist.

- [ ] **Step 3: Implement ordered bounded BGZF encoding and checked decoding**

```rust
pub fn compress<R, W>(
    mut source: R,
    sink: W,
    options: &CompressOptions,
) -> io::Result<(W, StreamStats)>
where
    R: Read,
    W: Write + Send + 'static,
{
    validate_level(options.level)?;
    let mut writer = writer::Writer::new(sink, options.level, options.workers)?;
    let bytes_in = writer.copy_from(&mut source, options.text)?;
    let blocks = writer.blocks();
    let (sink, bytes_out) = writer.finish()?;
    Ok((sink, StreamStats { bytes_in, bytes_out, blocks }))
}
```

The writer uses a fixed `2 * workers + 1` slot ring, sends monotonically
numbered blocks to compression workers, emits completed blocks in sequence,
and turns channel closure or a joined panic into `io::Error`. It writes the
canonical 28-byte EOF member once and flushes the returned sink. Text mode
chooses the final newline at or before the safe block limit; binary mode uses
the limit directly. The reader uses `noodles_bgzf::io::Reader` but checks that
the complete input terminates at exactly one canonical EOF and rejects bytes
after it.

- [ ] **Step 4: Run the focused test and library tests**

Run:

```bash
cargo test --test bgzip_cli bgzf_round_trip_and_truncation -- --exact
cargo test --lib
```

Expected: both commands pass and no build artifact appears outside the
configured external target.

- [ ] **Step 5: Commit the complete BGZF stream concern**

```bash
git add Cargo.toml Cargo.lock src tests/bgzip_cli.rs LICENSE-MIT LICENSE-APACHE-2.0 .gitignore
git commit -m "feat(index): add fail-safe BGZF streams"
```

---

### Task 2: GZI sidecars, partial reads, and transactional `bgzip`

**Files:**
- Create: `src/bgzip/index.rs`
- Create: `src/commands/bgzip.rs`
- Create: `src/commands/mod.rs`
- Modify: `src/bgzip.rs`
- Create: `tests/golden/text.txt`
- Modify: `tests/bgzip_cli.rs`

**Interfaces:**
- Consumes: Task 1 `compress`, `decompress`, `CompressOptions`, and `StreamStats`.
- Produces: `GziIndex::scan<R: Read + Seek>(&mut R)`, `GziIndex::read`, `GziIndex::write`, and `GziIndex::query(u64) -> io::Result<VirtualPosition>`.
- Produces: `bgzip::run(&RunOptions) -> rsomics_common::Result<Summary>` with `Mode::{Compress, Decompress, Test, Reindex}`.

- [ ] **Step 1: Add failing GZI and transaction tests**

```rust
#[test]
fn gzi_partial_read_matches_original() {
    let raw = (0..300_000).map(|i| (i % 251) as u8).collect::<Vec<_>>();
    let (encoded, _) = compress_bytes(&raw);
    let index = GziIndex::scan(&mut Cursor::new(&encoded)).unwrap();
    let mut out = Vec::new();
    decompress_indexed(Cursor::new(encoded), &index, 65_000, Some(131_000), &mut out).unwrap();
    assert_eq!(out, raw[65_000..196_000]);
}

#[test]
fn failed_named_output_preserves_destination() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("out.gz");
    std::fs::write(&output, b"old").unwrap();
    let error = run_bgzip_with_truncated_source(&output).unwrap_err();
    assert!(error.to_string().contains("truncated"));
    assert_eq!(std::fs::read(output).unwrap(), b"old");
}
```

- [ ] **Step 2: Run the new tests and verify both fail**

Run:

```bash
cargo test --test bgzip_cli gzi_partial_read_matches_original -- --exact
cargo test --test bgzip_cli failed_named_output_preserves_destination -- --exact
```

Expected: `GziIndex` and the command transaction are missing.

- [ ] **Step 3: Implement checked GZI scanning and indexed decompression**

```rust
pub fn query(&self, offset: u64) -> io::Result<VirtualPosition> {
    let i = self.entries.partition_point(|entry| entry.uncompressed <= offset);
    let entry = i.checked_sub(1).map_or(Entry::origin(), |j| self.entries[j]);
    let within = offset - entry.uncompressed;
    let within = u16::try_from(within)
        .map_err(|_| invalid("GZI entry does not cover requested offset"))?;
    VirtualPosition::try_from((entry.compressed, within)).map_err(invalid)
}
```

`scan` parses gzip fixed fields and every extra subfield, requires one `BC`
subfield of length two, validates `BSIZE`, reads `ISIZE`, and advances with
checked arithmetic. It records each non-origin block boundary and rejects a
missing/duplicate EOF, trailing bytes, decreasing offsets, or an index whose
entry does not name a real BGZF block.

- [ ] **Step 4: Implement command transactions and path rules**

```rust
pub fn run(options: &RunOptions) -> Result<Summary> {
    reject_aliases(options)?;
    match (&options.output, &options.index_output) {
        (Some(data), Some(index)) => run_pair(data, index, options),
        (Some(data), None) => write_atomic(data, |file| run_named(file, options)),
        (None, None) => run_stdout(options),
        (None, Some(_)) if options.mode == Mode::Compress => {
            Err(RsomicsError::ConfigError("--index-output requires named data output".into()))
        }
        _ => run_stdout(options),
    }
}
```

Test mode consumes and validates the full stream without output. Reindex uses
the scanner and only commits the `.gzi`. Partial reads require a GZI and seek
to the queried virtual position. `--size` limits emitted uncompressed bytes.

- [ ] **Step 5: Run BGZF tests and commit**

Run:

```bash
cargo test --test bgzip_cli
cargo test --lib bgzip
```

Expected: all BGZF and GZI tests pass.

```bash
git add src/bgzip.rs src/bgzip/index.rs src/commands tests/bgzip_cli.rs tests/golden/text.txt
git commit -m "feat(index): add indexed BGZF workflows"
```

---

### Task 3: Typed tabix configuration and sorted record stream

**Files:**
- Create: `src/tabix.rs`
- Create: `src/tabix/config.rs`
- Create: `src/tabix/record.rs`
- Create: `tests/tabix_cli.rs`
- Create: `tests/golden/records.bed`
- Create: `tests/golden/records.gff`
- Create: `tests/golden/records.sam`
- Create: `tests/golden/records.vcf`
- Modify: `src/lib.rs`

**Interfaces:**
- Produces: `Preset::{Bed, Gff, Sam, Vcf}`.
- Produces: `Config::from_preset`, `Config::detect`, `Config::parse(&[u8], line_no) -> Result<Record<'_>>`.
- Produces: `Record { reference: &[u8], start: u64, end: u64 }` using one-based inclusive internal coordinates.
- Produces: `SortedState::push(&Record, line_no) -> Result<()>`.

- [ ] **Step 1: Write failing preset, detection, and ordering tests**

```rust
#[test]
fn presets_share_checked_coordinate_model() {
    let bed = Config::from_preset(Preset::Bed).parse(b"chr1\t0\t10", 1).unwrap();
    let gff = Config::from_preset(Preset::Gff)
        .parse(b"chr1\tsrc\tgene\t1\t10\t.\t+\t.\tID=g", 1).unwrap();
    assert_eq!((bed.start, bed.end), (1, 10));
    assert_eq!((gff.start, gff.end), (1, 10));
}

#[test]
fn sorted_state_rejects_reference_reentry() {
    let mut state = SortedState::default();
    state.push(record(b"chr1", 1, 2), 1).unwrap();
    state.push(record(b"chr2", 1, 2), 2).unwrap();
    assert!(state.push(record(b"chr1", 3, 4), 3).is_err());
}
```

- [ ] **Step 2: Run the tests and verify missing types fail compilation**

Run:

```bash
cargo test --test tabix_cli presets_share_checked_coordinate_model -- --exact
cargo test --test tabix_cli sorted_state_rejects_reference_reentry -- --exact
```

- [ ] **Step 3: Implement presets, strict parser, detection, and ordering**

```rust
pub fn parse<'a>(&self, line: &'a [u8], line_no: u64) -> Result<Record<'a>> {
    let fields = split_required(line, self.max_column(), line_no)?;
    let reference = nonempty(fields[self.sequence - 1], line_no)?;
    let raw_start = parse_u64(fields[self.begin - 1], line_no)?;
    let raw_end = self.end.map_or(Ok(raw_start), |i| parse_u64(fields[i - 1], line_no))?;
    let (start, end) = self.coordinate_system.normalize(raw_start, raw_end)?;
    derive_formatted_end(self.preset, fields, Record { reference, start, end }, line_no)
}
```

VCF end uses valid `INFO/END` or `REF` length; SAM end uses checked CIGAR
reference consumption. Detection recognizes format headers and extensions,
then validates a representative data row. Ambiguous input fails with a request
for `--preset`. Custom columns cannot be combined with a preset.

- [ ] **Step 4: Run parser tests, malformed fixtures, and commit**

Run:

```bash
cargo test --test tabix_cli
cargo test --lib tabix::record
```

Expected: presets, custom columns, CRLF, empty/header-only input, bad columns,
zero/inverted coordinates, malformed CIGAR, and reference reentry all have
explicit passing assertions.

```bash
git add src/lib.rs src/tabix.rs src/tabix tests/tabix_cli.rs tests/golden
git commit -m "feat(index): add checked tabix records"
```

---

### Task 4: TBI and CSI index construction

**Files:**
- Create: `src/tabix/build.rs`
- Create: `src/tabix/index.rs`
- Modify: `src/tabix.rs`
- Modify: `tests/tabix_cli.rs`

**Interfaces:**
- Consumes: Task 3 `Config`, `Record`, and `SortedState`.
- Produces: `build(input: &Path, output: &mut File, &BuildOptions) -> Result<BuildSummary>`.
- Produces: `IndexKind::{Tbi, Csi { min_shift: u8 }}`.
- Produces: `load_index(data: &Path, explicit: Option<&Path>) -> Result<LoadedIndex>`.

- [ ] **Step 1: Add failing structural build tests**

```rust
#[test]
fn builds_queryable_tbi_and_csi() {
    for kind in [IndexKind::Tbi, IndexKind::Csi { min_shift: 14 }] {
        let fixture = bgzip_fixture("records.vcf");
        let bytes = build_index_bytes(&fixture, Preset::Vcf, kind).unwrap();
        let loaded = LoadedIndex::read(Cursor::new(bytes)).unwrap();
        assert_eq!(loaded.reference_names(), [b"chr1".as_slice(), b"chr2".as_slice()]);
        assert!(!loaded.query(0, 1..=20).unwrap().is_empty());
    }
}
```

- [ ] **Step 2: Run the structural test and verify it fails**

Run:

```bash
cargo test --test tabix_cli builds_queryable_tbi_and_csi -- --exact
```

- [ ] **Step 3: Refactor the retained accumulator into the new typed stream**

```rust
for record in RecordStream::new(input, config)? {
    let record = record?;
    sorted.push(&record.interval, record.line_no)?;
    let reference_id = references.intern(record.interval.reference)?;
    builder.add(reference_id, record.interval.start, record.interval.end, record.chunk)?;
}
builder.write(output)
```

Retain the old product's single-pass TBI bin/chunk accumulation and structural
bin compression. Replace the old CSI linear reference-name scan with the same
intern table. Make minimum shift a checked input, compute a sufficient CSI
depth from declared reference lengths and observed coordinates, and reject TBI
coordinates at or above `2^29`.

- [ ] **Step 4: Add atomic sidecar tests and run the build suite**

```rust
#[test]
fn malformed_late_record_does_not_replace_index() {
    let old = b"existing-index";
    let (data, index) = fixture_with_late_coordinate_regression(old);
    assert!(build_named(&data, &index).is_err());
    assert_eq!(std::fs::read(index).unwrap(), old);
}
```

Run:

```bash
cargo test --test tabix_cli builds_queryable_tbi_and_csi -- --exact
cargo test --test tabix_cli malformed_late_record_does_not_replace_index -- --exact
cargo test --lib tabix::build
```

- [ ] **Step 5: Commit TBI/CSI construction**

```bash
git add src/tabix.rs src/tabix/build.rs src/tabix/index.rs tests/tabix_cli.rs
git commit -m "feat(index): build checked tabix indexes"
```

---

### Task 5: Tabix query and list workflows

**Files:**
- Create: `src/tabix/query.rs`
- Modify: `src/tabix.rs`
- Modify: `tests/tabix_cli.rs`
- Add: `tests/golden/query-regions.tsv`
- Add: `tests/golden/query-targets.bed`

**Interfaces:**
- Consumes: `LoadedIndex`, stored header `Config`, and Task 3 record parser.
- Produces: `query(input, output, &QueryOptions) -> Result<QuerySummary>`.
- Produces: `list(input, output, explicit_index) -> Result<ListSummary>`.
- Produces: `QueryOptions { regions, regions_file, targets_file, print_header, header_only, unique, separate_regions, workers, cache_bytes }`.

- [ ] **Step 1: Add failing order, deduplication, header, and target tests**

```rust
#[test]
fn query_modes_preserve_their_contracts() {
    let data = indexed_vcf();
    let ordinary = query_text(&data, &["chr2:1-20", "chr1:1-20"], false, false);
    assert!(ordinary.find("chr2").unwrap() < ordinary.find("chr1").unwrap());
    let unique = query_text(&data, &["chr1:1-20", "chr1:10-30"], true, false);
    assert_eq!(unique.matches("chr1\t15").count(), 1);
    let separated = query_text(&data, &["chr1:1-20", "chr1:10-30"], false, true);
    assert!(separated.contains("#chr1:1-20"));
}
```

- [ ] **Step 2: Run the query test and verify the missing implementation**

Run:

```bash
cargo test --test tabix_cli query_modes_preserve_their_contracts -- --exact
```

- [ ] **Step 3: Implement physical-offset query planning and final filtering**

```rust
for selection in selections {
    if options.separate_regions {
        writeln!(output, "{}{}", config.comment as char, selection.label)?;
    }
    for chunk in index.query(selection.reference_id, selection.interval)? {
        for line in ChunkReader::new(&mut reader, chunk)? {
            let line = line?;
            let offset = line.virtual_position;
            let record = config.parse(line.bytes, line.line_no)?;
            if selection.overlaps(&record) && seen.insert_or_accept(offset, options.unique) {
                output.write_all(line.bytes_with_newline)?;
            }
        }
    }
}
```

Regions-file input may be BED by extension or one-based tabular. Targets are
normalized once and applied during one sequential pass, preserving source
order. Header modes read only leading stored meta lines. `--unique` and
`--separate-regions` are rejected together. `cache_bytes` bounds an LRU of
compressed BGZF blocks keyed by compressed offset; zero disables it.

- [ ] **Step 4: Run query/list tests including corrupt and stale indexes**

Run:

```bash
cargo test --test tabix_cli query
cargo test --test tabix_cli list
cargo test --lib tabix::query
```

Expected: inline/file regions, targets, header modes, explicit indexes,
missing contigs, repeated chunks, stale/truncated indexes, output failure, and
list order all pass.

- [ ] **Step 5: Commit query and list operations**

```bash
git add src/tabix.rs src/tabix/query.rs tests/tabix_cli.rs tests/golden
git commit -m "feat(index): query and inspect tabix indexes"
```

---

### Task 6: Unified `rsomics-help` command layer and product documentation

**Files:**
- Create: `src/cli.rs`
- Modify: `src/main.rs`
- Modify: `src/commands/bgzip.rs`
- Create: `src/commands/tabix.rs`
- Create: `README.md`
- Create: `THIRD_PARTY_LICENSES.md`
- Create: `LICENSES/HTSLIB-MIT.txt`
- Modify: `tests/bgzip_cli.rs`
- Modify: `tests/tabix_cli.rs`

**Interfaces:**
- Consumes: `bgzip::run`, `tabix::build`, `tabix::query`, and `tabix::list`.
- Produces: `rsomics-index bgzip`, `tabix build`, `tabix query`, and `tabix list`.
- Produces: `CommandReport` variants serialized only through `rsomics-common::run`.

- [ ] **Step 1: Add failing command-tree and JSON separation tests**

```rust
#[test]
fn help_exposes_only_stable_operations() {
    let help = command().render_long_help().to_string();
    for name in ["bgzip", "tabix"] { assert!(help.contains(name)); }
    for absent in ["fasta-index", "dict", "fm-search"] { assert!(!help.contains(absent)); }
}

#[test]
fn json_requires_named_data_output() {
    let output = binary().args(["--json", "bgzip"]).write_stdin(b"ACGT").output().unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.starts_with(b"{"));
    assert!(output.stderr.is_empty());
}
```

- [ ] **Step 2: Run command tests and verify they fail**

Run:

```bash
cargo test --test bgzip_cli help_exposes_only_stable_operations -- --exact
cargo test --test bgzip_cli json_requires_named_data_output -- --exact
```

- [ ] **Step 3: Implement the shared command tree**

```rust
#[derive(Debug, Subcommand)]
enum Command {
    Bgzip(commands::bgzip::Arguments),
    Tabix(commands::tabix::Arguments),
}

#[must_use]
pub(crate) fn run() -> process::ExitCode {
    let cli = rsomics_help::parse::<Cli>();
    let output = cli.output.clone();
    run_tool(&output, META, || execute(cli))
}
```

Use clap argument groups for incompatible modes, validate output aliases
before opening a transaction, and require named data/table output under
`--json`. README examples use only the public binary and explain BGZF/tabix
coordinates, TBI limits, CSI selection, local-only scope, FFI libdeflate, and
explicit exclusions.

- [ ] **Step 4: Run all ordinary tests, strict lint, docs, and package checks**

Run:

```bash
cargo-fmt fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --all-features
cargo package --allow-dirty --locked
```

Expected: every command passes and the package contains no scratch, oracle
build, or benchmark result directories.

- [ ] **Step 5: Commit the complete user-facing product slice**

```bash
git add src README.md THIRD_PARTY_LICENSES.md LICENSES tests Cargo.toml Cargo.lock
git commit -m "feat(index): unify compression and tabix workflows"
```

---

### Task 7: Pinned HTSlib 1.24 differential suite and four-native CI

**Files:**
- Create: `tests/bgzip_compat.rs`
- Create: `tests/tabix_compat.rs`
- Create: `.github/workflows/ci.yml`
- Modify: `tests/golden/*`

**Interfaces:**
- Consumes: the stable CLI from Task 6.
- Produces: ignored local oracle tests that require explicit `RSOMICS_HTSLIB_ORACLE_DIR` and mandatory release-CI invocation.

- [ ] **Step 1: Write failing cross-tool differential tests**

```rust
#[test]
#[ignore = "requires HTSlib 1.24 oracles"]
fn cross_tool_bgzf_and_tabix() {
    let oracle = Oracle::from_env().unwrap().require_version("1.24").unwrap();
    let fixture = representative_vcf();
    let ours = ours_bgzip(&fixture);
    assert_eq!(oracle.bgzip_decode(&ours).unwrap(), fixture);
    let hts = oracle.bgzip_encode(&fixture).unwrap();
    assert_eq!(ours_bgzip_decode(&hts), fixture);
    assert_structurally_equal(ours_tbi(&ours), oracle.tabix_tbi(&ours));
    assert_eq!(ours_query(&ours, "chr2:10-200"), oracle.query(&ours, "chr2:10-200"));
}
```

- [ ] **Step 2: Run against installed 1.24 and observe unsupported differences**

Run:

```bash
RSOMICS_HTSLIB_ORACLE_DIR=/opt/homebrew/bin \
  cargo test --test bgzip_compat -- --ignored --nocapture
RSOMICS_HTSLIB_ORACLE_DIR=/opt/homebrew/bin \
  cargo test --test tabix_compat -- --ignored --nocapture
```

Expected: initial failures identify exact output/exit differences; record and
fix the product behavior, not the golden, unless the 1.24 manual, source, and
binary contradict one another.

- [ ] **Step 3: Complete the pinned oracle matrix**

The BGZF matrix includes text/binary blocks, levels 0/1/6/9, one/four workers,
valid/corrupt/truncated streams, GZI build, offset/size reads, and stdout/named
output. The tabix matrix includes all presets, custom columns, TBI, CSI at two
minimum shifts, headers, inline/file regions, targets, unique, separators,
list order, coordinate limits, unsorted records, and corrupt indexes.

```rust
for case in Case::all() {
    let ours = case.run_ours()?;
    let upstream = case.run_upstream(&oracle)?;
    case.assert_compatible(ours, upstream)?;
}
```

- [ ] **Step 4: Add native CI and run the full local release gate**

```yaml
strategy:
  fail-fast: false
  matrix:
    include:
      - os: ubuntu-24.04
        runner: ubuntu-24.04
      - os: ubuntu-24.04-arm
        runner: ubuntu-24.04-arm
      - os: macos-15-intel
        runner: macos-15-intel
      - os: macos-15
        runner: macos-15
```

Linux x86_64 builds HTSlib tag 1.24 under runner scratch and runs both ignored
oracle suites. Every target runs fmt check, strict Clippy where supported,
tests, rustdoc, and locked package verification.

- [ ] **Step 5: Commit, create the GitHub repository, push, and wait for exact head**

```bash
git add .github tests
git commit -m "test(index): pin HTSlib compatibility"
gh repo create omics-rust/rsomics-index --public --source=. --remote=origin --push
gh run list --repo omics-rust/rsomics-index --commit "$(git rev-parse HEAD)"
```

Expected: the exact head passes all four native target classes before Task 8.

---

### Task 8: Representative performance gate and release verification

**Files:**
- Create: `benchmarks/index-vs-htslib.sh`
- Create: `PERFORMANCE.md`
- Modify: `README.md`
- Modify: `/Volumes/Zane's HDD/Documents/rsomics-world/docs/10-products/interval-annotation-index.md`
- Modify: `/Volumes/Zane's HDD/Documents/rsomics-world/docs/10-products/README.md`
- Modify: `/Volumes/Zane's HDD/Documents/rsomics-world/REGISTRY.md`

**Interfaces:**
- Consumes: clean exact-head release binary and pinned HTSlib 1.24 binaries.
- Produces: full semantic hashes, paired timing/RSS ledger, performance decision, release revision, CI run, and registry verification.

- [ ] **Step 1: Add the benchmark harness and verify smoke equality**

```bash
benchmarks/index-vs-htslib.sh generate --records 6000000
benchmarks/index-vs-htslib.sh smoke
```

The harness stores large fixtures under `/Volumes/Zane's HDD/rsomics-fixtures`
and transient results under `/Volumes/KIOXIA/Developments/tmp`. It refuses the
boot disk, refuses a dirty worktree for full runs, alternates tool order, and
compares full decompressed/query outputs and structural indexes before timing.

- [ ] **Step 2: Run full release measurements and write the evidence ledger**

```bash
benchmarks/index-vs-htslib.sh run --warmup 3 --runs 10
benchmarks/index-vs-htslib.sh summarize > PERFORMANCE.md
```

Expected: at least one representative compression, index-build, partial-read,
or query hot path has a strict throughput or peak-RSS advantage. Record every
other path as win, parity, or regression with exact provenance; do not publish
an unsupported global speed claim.

- [ ] **Step 3: Run the final clean review and exact-head CI**

```bash
cargo-fmt fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
RSOMICS_HTSLIB_ORACLE_DIR=/opt/homebrew/bin cargo test --test bgzip_compat -- --ignored
RSOMICS_HTSLIB_ORACLE_DIR=/opt/homebrew/bin cargo test --test tabix_compat -- --ignored
cargo doc --no-deps --all-features
cargo package --locked
git diff --check
```

Review every public item and the BGZF/tabix hot paths, then commit only the
performance and release documentation concern, push, and wait for the exact
head four-native run.

- [ ] **Step 4: Publish only if the registry credential is usable**

```bash
cargo publish --locked
```

If authentication remains revoked, record the exact tested head and package
checksum in `.autopilot/gates/gate-index-release-2026-08-19.md`, leave the
repository clean and publish-ready, and continue another unblocked product.
Do not create or expose a credential without the separate explicit security
confirmation.

- [ ] **Step 5: Verify the live package and update the control plane**

```bash
cargo install --locked rsomics-index --version 0.1.0 --root /Volumes/KIOXIA/Developments/tmp/rsomics-index-install
/Volumes/KIOXIA/Developments/tmp/rsomics-index-install/bin/rsomics-index --version
python3 scripts/validate_control_plane.py
```

Download the crates.io archive independently, verify its checksum, VCS
revision, and unpacked file tree, and run BGZF round-trip plus tabix build/query
smokes. Update the dossier, portfolio map, and registry from verified live
evidence only. Commit and push the control-plane concern and wait for its exact
head CI.

