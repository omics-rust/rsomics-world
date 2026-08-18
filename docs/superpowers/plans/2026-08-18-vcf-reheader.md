# rsomics-vcf reheader Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the complete, fail-loud `rsomics-vcf reheader` operation and release it as `rsomics-vcf 0.5.0` after compatibility, performance, native-CI, and registry verification.

**Architecture:** Keep all policy private to `rsomics-vcf`: a raw header model composes replacement, FAI, and sample edits; plain VCF copies body bytes; BGZF VCF inflates only header frames and structurally validates raw tail frames while copying; BCF streams typed records through stable edited dictionaries. The CLI remains a thin `rsomics-help` adapter and named output remains owned by `rsomics-common::AtomicFile`.

**Tech Stack:** Rust 2024, Rust 1.91 minimum, clap 4.5, noodles-vcf 0.90, noodles-bcf 0.88, noodles-bgzf 0.49, flate2 with zlib-rs, rsomics-common 0.12, rsomics-help 0.4, bcftools/HTSlib 1.24.

**Spec:** `/Volumes/Zane's HDD/Documents/rsomics-world/docs/plans/2026-08-13-vcf-reheader-design.md`

## Global Constraints

- Work in `/Volumes/KIOXIA/Documents/omics-rust/rsomics-vcf` directly on `main`; do not create a worktree or pull request.
- Before every Cargo invocation export `CARGO_HOME=/Volumes/KIOXIA/Developments/cargo-home`, `CARGO_TARGET_DIR=/Volumes/KIOXIA/Developments/cargo-target/rsomics-vcf`, and `TMPDIR=/Volumes/KIOXIA/Developments/tmp`.
- Before the first build in each session verify `/` is below 80% and Cargo's resolved target directory is on KIOXIA; stop if either condition is false.
- Do not add a dependency, a public crate, or a public library API.
- Keep comments rare; only CLI contract docs and non-obvious stable format invariants earn comments.
- Write and observe a failing test before every production behavior change.
- Preserve plain and BGZF VCF record bodies; do not claim full body validation on the raw-copy path.
- Require one of header replacement, FAI synchronization, or sample renaming; do not expose placeholder flags.
- Preserve the input encoding and allow nonzero `--threads` only for BGZF BCF.
- Use bcftools 1.24 and the VCF 4.1-4.5/BCF2 specifications as the oracle.
- Do not publish until the complete command, representative performance gate, exact-head four-native-target CI, package review, and registry smoke test pass.

Use this preflight before Task 1 and before any later session that resumes builds:

```bash
export CARGO_HOME=/Volumes/KIOXIA/Developments/cargo-home
export CARGO_TARGET_DIR=/Volumes/KIOXIA/Developments/cargo-target/rsomics-vcf
export TMPDIR=/Volumes/KIOXIA/Developments/tmp
export PATH="/opt/homebrew/Cellar/rust/1.97.1/bin:$PATH"
test "$(df -Pk / | awk 'NR==2 {gsub(/%/, "", $5); print ($5 < 80)}')" = 1
test "$(cargo metadata --locked --no-deps --format-version 1 | jq -r .target_directory)" = "$CARGO_TARGET_DIR"
df -h / /Volumes/KIOXIA
```

---

### Task 1: Raw header model and replacement contract

**Files:**
- Create: `src/reheader.rs`
- Create: `src/reheader/header.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Produces: `HeaderText::parse(&[u8]) -> Result<HeaderText>`
- Produces: `HeaderText::replace_from(&Path, usize) -> Result<HeaderText>`
- Produces: `HeaderText::{render, sample_names, contig_count}`
- Produces: `HeaderText::parse_noodles() -> Result<noodles_vcf::Header>`

- [ ] **Step 1: Write failing header tests**

Add unit tests in `src/reheader/header.rs` whose literal fixtures prove LF and CRLF normalization, exactly one first-position `##fileformat`, exactly one terminal `#CHROM`, fixed-column spelling, FORMAT/sample shape, unique samples, equal replacement sample count, and preservation of unknown metadata order.

```rust
fn fixture(source: &[u8]) -> tempfile::NamedTempFile {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(source).unwrap();
    file
}

#[test]
fn replacement_preserves_unknown_lines_and_normalizes_newlines() {
    let old = HeaderText::parse(HEADER_TWO_SAMPLES.as_bytes()).unwrap();
    let input = fixture(REPLACEMENT_CRLF.as_bytes());
    let replacement = HeaderText::replace_from(input.path(), old.sample_names().len()).unwrap();
    assert_eq!(
        replacement.render(),
        b"##fileformat=VCFv4.3\n##source=changed\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tR1\tR2\n"
    );
}

#[test]
fn replacement_rejects_a_different_sample_count() {
    let input = fixture(HEADER_ONE_SAMPLE.as_bytes());
    let error = HeaderText::replace_from(input.path(), 2).unwrap_err();
    assert!(error.to_string().contains("2 samples"), "{error}");
}
```

- [ ] **Step 2: Run the tests and verify RED**

Run:

```bash
cargo test --locked reheader::header::tests -- --nocapture
```

Expected: compilation fails because `reheader` and `HeaderText` do not exist.

- [ ] **Step 3: Add the minimal raw header model**

Use owned UTF-8 lines so header editing never touches record bytes. Parse fixed columns literally and render a single LF after every header line.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct HeaderText {
    metadata: Vec<String>,
    columns: Vec<String>,
}

impl HeaderText {
    pub(super) fn parse(raw: &[u8]) -> Result<Self>;
    pub(super) fn replace_from(path: &Path, expected_samples: usize) -> Result<Self>;
    pub(super) fn render(&self) -> Vec<u8>;
    pub(super) fn sample_names(&self) -> &[String] { &self.columns[9..] }
    pub(super) fn contig_count(&self) -> usize;
    pub(super) fn parse_noodles(&self) -> Result<noodles_vcf::Header>;
}
```

`parse` must reject bytes that are not UTF-8, embedded control characters other than tab, metadata after `#CHROM`, duplicate `##fileformat`, duplicate `#CHROM`, duplicate sample IDs, trailing tab fields, and genotype columns without FORMAT.

- [ ] **Step 4: Run focused and library tests and verify GREEN**

```bash
cargo test --locked reheader::header::tests -- --nocapture
cargo test --locked --lib
```

Expected: all tests pass with no warnings.

- [ ] **Step 5: Commit the header model**

```bash
git add src/lib.rs src/reheader.rs src/reheader/header.rs
git commit -m "feat(vcf): model reheader headers"
```

### Task 2: Sample edit parser and application

**Files:**
- Create: `src/reheader/samples.rs`
- Modify: `src/reheader/header.rs`
- Modify: `src/reheader.rs`

**Interfaces:**
- Consumes: `HeaderText::sample_names()` from Task 1
- Produces: `SampleSource::{List(String), File(PathBuf)}`
- Produces: `SampleEdit::read(&SampleSource) -> Result<SampleEdit>`
- Produces: `SampleEdit::apply(&[String]) -> Result<Vec<String>>`
- Produces: `HeaderText::set_samples(Vec<String>) -> Result<()>`

- [ ] **Step 1: Write failing sample tests**

Cover comma-list replacement, one-column files, complete two-column maps, backslash-escaped spaces, blank-line skipping, sites-only rejection, and every fail-loud divergence.

```rust
#[test]
fn escaped_pairs_rename_all_samples() {
    let edit = SampleEdit::parse_file(b"S1\tTumor\\ One\nS2\tNormal\\ Two\n").unwrap();
    assert_eq!(
        edit.apply(&["S1".into(), "S2".into()]).unwrap(),
        ["Tumor One", "Normal Two"]
    );
}

#[test]
fn mappings_reject_unknown_duplicate_and_conflicting_names() {
    for source in [
        b"missing\tN1\n".as_slice(),
        b"S1\tN1\nS1\tN2\n",
        b"S1\tN\nS2\tN\n",
        b"S1\tN1\textra\nS2\tN2\n",
        b"S1\tN1\nN2\n",
    ] {
        assert!(SampleEdit::parse_file(source)
            .and_then(|edit| edit.apply(&["S1".into(), "S2".into()]))
            .is_err());
    }
}
```

- [ ] **Step 2: Run the tests and verify RED**

```bash
cargo test --locked reheader::samples::tests -- --nocapture
```

Expected: compilation fails because `SampleEdit` is absent.

- [ ] **Step 3: Implement the exact sample grammar**

Tokenize backslash escapes before classifying every nonblank line as either one field or exactly two fields. Mixed modes and extra fields fail rather than falling back to positional replacement.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SampleSource {
    List(String),
    File(PathBuf),
}

enum SampleEdit {
    Positional(Vec<String>),
    Pairs(Vec<(String, String)>),
}

impl SampleEdit {
    pub(super) fn read(source: &SampleSource) -> Result<Self>;
    fn parse_list(source: &str) -> Result<Self>;
    fn parse_file(source: &[u8]) -> Result<Self>;
    pub(super) fn apply(&self, current: &[String]) -> Result<Vec<String>>;
}
```

Validate names as nonempty UTF-8 without tab, CR, LF, or disallowed control characters. Positional count must equal the existing count; pair sources and final names must each be unique; every pair source must exist.

- [ ] **Step 4: Integrate with `HeaderText` and verify GREEN**

```rust
impl HeaderText {
    pub(super) fn apply_samples(&mut self, source: &SampleSource) -> Result<()> {
        let edit = SampleEdit::read(source)?;
        let names = edit.apply(self.sample_names())?;
        self.set_samples(names)
    }
}
```

Run:

```bash
cargo test --locked reheader:: -- --nocapture
cargo test --locked --lib
```

Expected: all tests pass.

- [ ] **Step 5: Commit sample edits**

```bash
git add src/reheader.rs src/reheader/header.rs src/reheader/samples.rs
git commit -m "feat(vcf): apply reheader sample edits"
```

### Task 3: FAI synchronization

**Files:**
- Create: `src/reheader/fai.rs`
- Modify: `src/reheader/header.rs`
- Modify: `src/reheader.rs`

**Interfaces:**
- Produces: `Fai::read(&Path) -> Result<Fai>`
- Produces: ordered `FaiEntry { name: String, length: u64 }`
- Produces: `HeaderText::apply_fai(&Path) -> Result<()>`

- [ ] **Step 1: Write failing FAI tests**

Use literal expected headers to prove update, removal, append order, retention of non-length attributes, insertion of a missing length, zero and `u64::MAX` handling, and malformed input errors.

```rust
#[test]
fn fai_replaces_the_contig_set_in_fai_order() {
    let mut header = HeaderText::parse(
        b"##fileformat=VCFv4.3\n##contig=<ID=chr1,length=100>\n\
          ##contig=<ID=chr2,length=200,assembly=test>\n\
          #CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\n",
    ).unwrap();
    let fai = fixture(b"chr2\t250\t0\t0\t0\nchr3\t300\t0\t0\t0\n");
    header.apply_fai(fai.path()).unwrap();
    assert_eq!(
        header.render(),
        b"##fileformat=VCFv4.3\n##contig=<ID=chr2,length=250,assembly=test>\n\
          ##contig=<ID=chr3,length=300>\n\
          #CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\n"
    );
}

#[test]
fn fai_rejects_duplicates_and_invalid_lengths() {
    for source in [b"chr1\t1\nchr1\t2\n".as_slice(), b"chr1\tx\n", b"\t1\n", b"\n"] {
        assert!(Fai::parse(source).is_err());
    }
}
```

- [ ] **Step 2: Run the tests and verify RED**

```bash
cargo test --locked reheader::fai::tests -- --nocapture
```

Expected: compilation fails because `Fai` is absent.

- [ ] **Step 3: Implement the ordered FAI model and structured-line edit**

```rust
pub(super) struct Fai {
    entries: Vec<FaiEntry>,
}

struct FaiEntry {
    name: String,
    length: u64,
}

impl Fai {
    pub(super) fn read(path: &Path) -> Result<Self>;
    fn parse(source: &[u8]) -> Result<Self>;
}

impl HeaderText {
    pub(super) fn apply_fai(&mut self, path: &Path) -> Result<()>;
}
```

Split structured contig fields with a quote-aware scanner, replace only the `length` value, retain every other field in its original order, remove contigs absent from the FAI, and insert FAI-only lines immediately before `#CHROM`. Parse the rendered result through noodles before accepting it.

- [ ] **Step 4: Run focused and library tests and verify GREEN**

```bash
cargo test --locked reheader:: -- --nocapture
cargo test --locked --lib
```

Expected: all tests pass.

- [ ] **Step 5: Commit FAI edits**

```bash
git add src/reheader.rs src/reheader/header.rs src/reheader/fai.rs
git commit -m "feat(vcf): synchronize reheader contigs"
```

### Task 4: Private BGZF frame reader and copier

**Files:**
- Create: `src/format/bgzf.rs`
- Modify: `src/format/mod.rs`
- Modify: `src/index/build.rs`

**Interfaces:**
- Produces: `format::bgzf::EOF_BLOCK`
- Produces: `FrameReader<R>::new(R)`
- Produces: `FrameReader::next() -> io::Result<Option<Frame>>`
- Produces: `FrameReader::copy_through_eof(W) -> io::Result<u64>`
- Produces: `Frame::{Data(Vec<u8>), Eof}`

- [ ] **Step 1: Write failing frame tests**

Generate real frames with `noodles_bgzf::io::Writer` and prove exact raw copying, canonical EOF detection, partial-header rejection, partial-payload rejection, invalid `BSIZE`, missing EOF, duplicate/trailing frames, and empty data frames that are not the canonical EOF.

```rust
fn bgzf_fixture(source: &[u8]) -> Vec<u8> {
    let mut writer = noodles_bgzf::io::Writer::new(Vec::new());
    writer.write_all(source).unwrap();
    writer.finish().unwrap()
}

#[test]
fn copies_complete_frames_through_one_canonical_eof() {
    let input = bgzf_fixture(b"body");
    let mut output = Vec::new();
    let copied = FrameReader::new(input.as_slice())
        .copy_through_eof(&mut output)
        .unwrap();
    assert_eq!(copied, input.len() as u64);
    assert_eq!(output, input);
}

#[test]
fn rejects_bytes_after_the_eof_block() {
    let mut input = bgzf_fixture(b"body");
    input.extend_from_slice(b"trailing");
    assert_eq!(
        FrameReader::new(input.as_slice())
            .copy_through_eof(io::sink())
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidData
    );
}
```

- [ ] **Step 2: Run the tests and verify RED**

```bash
cargo test --locked format::bgzf::tests -- --nocapture
```

Expected: compilation fails because `format::bgzf` is absent.

- [ ] **Step 3: Implement bounded frame parsing**

Read the 18-byte canonical header, derive `frame_len = u16::from_le_bytes([header[16], header[17]]) + 1`, require `28..=65536`, and then read exactly the remaining bytes. Do not interpret compressed payloads on the copy path.

```rust
pub(crate) const EOF_BLOCK: [u8; 28] = [
    0x1f, 0x8b, 0x08, 0x04, 0, 0, 0, 0, 0, 0xff, 6, 0, b'B', b'C', 2, 0,
    27, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub(crate) enum Frame {
    Data(Vec<u8>),
    Eof,
}

pub(crate) struct FrameReader<R> { inner: R, finished: bool }

impl<R: Read> FrameReader<R> {
    pub(crate) fn next(&mut self) -> io::Result<Option<Frame>>;
    pub(crate) fn copy_through_eof<W: Write>(&mut self, output: W) -> io::Result<u64>;
}
```

After `Eof`, read one byte and require physical EOF. This distinguishes a complete stream from an embedded or duplicated terminator.

- [ ] **Step 4: Reuse the EOF constant in indexing and verify GREEN**

Replace the private constant in `src/index/build.rs` with `crate::format::bgzf::EOF_BLOCK`; retain its existing seek-based named-file preflight.

```bash
cargo test --locked format::bgzf::tests -- --nocapture
cargo test --locked index:: -- --nocapture
cargo test --locked --lib
```

Expected: frame and index tests pass without behavior changes to `index`.

- [ ] **Step 5: Commit the private BGZF primitive**

```bash
git add src/format/bgzf.rs src/format/mod.rs src/index/build.rs
git commit -m "refactor(vcf): share BGZF frame validation"
```

### Task 5: Plain VCF engine and public CLI

**Files:**
- Create: `src/reheader/vcf.rs`
- Create: `src/commands/reheader.rs`
- Create: `tests/reheader_cli.rs`
- Modify: `src/reheader.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/cli.rs`

**Interfaces:**
- Consumes: `HeaderText`, `SampleSource`, FAI edits, and `AtomicFile`
- Produces: `reheader::Options`
- Produces: `reheader::Summary`
- Produces: `reheader::write(input, &Options, output) -> Result<Summary>`
- Produces: `CommandOutput::Reheader { summary }`

- [ ] **Step 1: Write failing CLI and plain-VCF tests**

Add real binary tests for help spelling, required edit group, edit composition order, byte-identical body copying, stdin, JSON separation, output aliases against every input path, pre-existing destination rollback, ordinary gzip rejection, and nonzero plain-VCF threads.

```rust
struct Fixture {
    _directory: tempfile::TempDir,
    input: PathBuf,
    header: PathBuf,
    fai: PathBuf,
    body: Vec<u8>,
}

impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.vcf");
        let header = directory.path().join("replacement.vcfh");
        let fai = directory.path().join("reference.fai");
        let body = b"chr1\t1\t.\tA\tC\t.\tPASS\t.\tGT\t0/1\t0/0\n".to_vec();
        fs::write(
            &input,
            [HEADER_TWO_SAMPLES.as_bytes(), body.as_slice()].concat(),
        ).unwrap();
        fs::write(&header, REPLACEMENT_TWO_SAMPLES).unwrap();
        fs::write(&fai, b"chr1\t1000\t0\t0\t0\n").unwrap();
        Self { _directory: directory, input, header, fai, body }
    }

    fn input(&self) -> &str { self.input.to_str().unwrap() }
    fn header(&self) -> &str { self.header.to_str().unwrap() }
    fn fai(&self) -> &str { self.fai.to_str().unwrap() }
    fn body_bytes(&self) -> &[u8] { &self.body }
}

#[test]
fn plain_vcf_composes_header_fai_and_samples_without_touching_body() {
    let fixture = Fixture::new();
    let output = run(&[
        "reheader", "-H", fixture.header(), "-f", fixture.fai(),
        "-n", "C1,C2", fixture.input(),
    ]);
    assert_success(&output);
    let (header, body) = split_header(&output.stdout);
    assert_eq!(header, EXPECTED_COMPOSED_HEADER);
    assert_eq!(body, fixture.body_bytes());
}

#[test]
fn json_requires_named_variant_output() {
    let fixture = Fixture::new();
    let output = run(&["--json", "reheader", "-n", "N1,N2", fixture.input()]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}
```

- [ ] **Step 2: Run the tests and verify RED**

```bash
cargo test --locked --test reheader_cli -- --nocapture
```

Expected: tests fail because `reheader` is not a recognized subcommand.

- [ ] **Step 3: Add the complete CLI shape**

```rust
#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("edit")
        .required(true)
        .multiple(true)
        .args(["header", "fai", "samples_list", "samples_file"])
))]
pub(crate) struct Arguments {
    #[arg(value_name = "VARIANT", default_value = "-")]
    input: PathBuf,
    #[arg(short = 'H', long, value_name = "FILE")]
    header: Option<PathBuf>,
    #[arg(short = 'f', long, value_name = "FILE")]
    fai: Option<PathBuf>,
    #[arg(short = 'n', long, value_name = "LIST", conflicts_with = "samples_file")]
    samples_list: Option<String>,
    #[arg(short = 'N', long, value_name = "FILE")]
    samples_file: Option<PathBuf>,
    #[arg(short = 'o', long, value_name = "FILE", default_value = "-", hide_default_value = true)]
    output: PathBuf,
    #[arg(long, value_name = "INT", default_value_t = 0)]
    threads: usize,
}
```

Add `Reheader` to the root command enum and use `rsomics_help::parse` without a private help renderer.

- [ ] **Step 4: Implement plain input detection, header editing, and transactional output**

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Encoding { PlainVcf, BgzfVcf, RawBcf, BgzfBcf }

pub(crate) struct Options {
    pub(crate) header: Option<PathBuf>,
    pub(crate) fai: Option<PathBuf>,
    pub(crate) samples: Option<SampleSource>,
}

#[derive(Debug, Serialize)]
pub(crate) struct Summary {
    encoding: Encoding,
    header_replaced: bool,
    fai_applied: bool,
    samples_renamed: bool,
    contigs_before: usize,
    contigs_after: usize,
    samples_before: usize,
    samples_after: usize,
}

pub(crate) fn write<W: Write>(input: &Path, options: &Options, output: W) -> Result<Summary>;
```

For plain VCF, use `BufRead::fill_buf` to stop before the first non-header byte, compose all edits before writing, emit the edited header, and `io::copy` the remaining buffered stream. Detect gzip magic separately and return a conversion diagnostic for ordinary gzip.

In `commands/reheader.rs`, reject `--json` with stdout variant output, call `reject_output_alias` with input/header/FAI/sample-file paths, use stdout directly or `AtomicFile::new`, and commit only after `write` returns.

- [ ] **Step 5: Run targeted tests and verify GREEN**

```bash
cargo test --locked --test reheader_cli -- --nocapture
cargo test --locked cli::tests -- --nocapture
cargo test --locked --lib
```

Expected: every plain-VCF and CLI test passes; compressed and BCF cases are not yet added to the test file.

- [ ] **Step 6: Commit the plain command**

```bash
git add src/cli.rs src/commands/mod.rs src/commands/reheader.rs src/reheader.rs src/reheader/vcf.rs tests/reheader_cli.rs
git commit -m "feat(vcf): stream plain VCF reheader"
```

### Task 6: BGZF VCF header-frame rewrite and raw tail preservation

**Files:**
- Modify: `src/reheader/vcf.rs`
- Modify: `src/reheader.rs`
- Modify: `tests/reheader_cli.rs`

**Interfaces:**
- Consumes: `FrameReader` from Task 4
- Produces: `vcf::rewrite_bgzf<R: Read, W: Write>(R, W, &Options) -> Result<Summary>`
- Produces: `vcf::inflate_header_frame(&[u8]) -> io::Result<Vec<u8>>`

- [ ] **Step 1: Write failing BGZF behavior tests**

Create fixtures with the real noodles BGZF writer. Cover header and body in one frame, a header spanning frames, a body spanning several frames, a header-only stream, stdin, ordinary gzip, missing EOF, partial tail frame, invalid `BSIZE`, and bytes after EOF.

```rust
struct BgzfFixture {
    _directory: tempfile::TempDir,
    path: PathBuf,
    body: Vec<u8>,
    tail_frames: Vec<u8>,
}

impl BgzfFixture {
    fn with_large_body() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("input.vcf.gz");
        let body = (0..20_000)
            .map(|position| format!("chr1\t{position}\t.\tA\tC\t.\tPASS\t.\tGT\t0/1\t0/0\n"))
            .collect::<String>()
            .into_bytes();
        let bytes = bgzf_bytes(&[HEADER_TWO_SAMPLES.as_bytes(), body.as_slice()].concat());
        fs::write(&path, &bytes).unwrap();
        let tail_frames = raw_frames_after_first_body(&bytes);
        Self { _directory: directory, path, body, tail_frames }
    }

    fn path(&self) -> &str { self.path.to_str().unwrap() }
    fn body_bytes(&self) -> &[u8] { &self.body }
    fn tail_frames(&self) -> &[u8] { &self.tail_frames }
}

#[test]
fn bgzf_rewrites_only_header_frames_and_preserves_raw_tail_frames() {
    let fixture = BgzfFixture::with_large_body();
    let output = run(&["reheader", "-n", "N1,N2", fixture.path()]);
    assert_success(&output);
    assert_eq!(inflate_body(&output.stdout), fixture.body_bytes());
    assert_eq!(raw_frames_after_first_body(&output.stdout), fixture.tail_frames());
}

#[test]
fn bgzf_header_only_stdin_emits_one_complete_eof() {
    let input = bgzf_bytes(HEADER_TWO_SAMPLES.as_bytes());
    let output = run_with_stdin(&["reheader", "-n", "N1,N2"], &input);
    assert_success(&output);
    assert_eq!(count_eof_blocks(&output.stdout), 1);
    assert!(bgzf_read_all(&output.stdout).ends_with(b"N1\tN2\n"));
}
```

- [ ] **Step 2: Run BGZF tests and verify RED**

```bash
cargo test --locked --test reheader_cli bgzf_ -- --nocapture
```

Expected: tests fail because compressed VCF is still rejected or routed through the ordinary-gzip error.

- [ ] **Step 3: Implement frame-retaining header discovery**

Read one `Frame::Data(raw)` at a time, inflate that single frame with `noodles_bgzf::io::Reader`, append uncompressed bytes until the header boundary is found, and retain the raw reader at the next frame.

```rust
fn rewrite_bgzf<R: Read, W: Write>(
    input: R,
    output: W,
    options: &Options,
) -> Result<Summary> {
    let mut frames = FrameReader::new(input);
    let mut prefix = Vec::new();
    while header_end(&prefix).is_none() {
        match frames.next()? {
            Some(Frame::Data(raw)) => prefix.extend_from_slice(&inflate_frame(&raw)?),
            Some(Frame::Eof) => return rewrite_header_only(prefix, output, options),
            None => return Err(invalid("BGZF EOF block is missing")),
        }
    }
    rewrite_prefix_and_copy_tail(prefix, frames, output, options)
}
```

Write the edited header and uncompressed body prefix through a new `bgzf::io::Writer`, call `flush` without `try_finish`, take the inner writer, and call `FrameReader::copy_through_eof`. For header-only input, call `try_finish` so the output receives exactly one canonical EOF.

- [ ] **Step 4: Run focused, integration, and regression tests and verify GREEN**

```bash
cargo test --locked --test reheader_cli bgzf_ -- --nocapture
cargo test --locked --test reheader_cli -- --nocapture
cargo test --locked index:: -- --nocapture
```

Expected: all tests pass and raw tail-frame equality holds.

- [ ] **Step 5: Commit BGZF VCF support**

```bash
git add src/reheader.rs src/reheader/vcf.rs tests/reheader_cli.rs
git commit -m "feat(vcf): preserve BGZF VCF bodies"
```

### Task 7: Dictionary-safe raw and BGZF BCF rewrite

**Files:**
- Create: `src/reheader/bcf.rs`
- Modify: `src/reheader.rs`
- Modify: `src/commands/reheader.rs`
- Modify: `tests/reheader_cli.rs`

**Interfaces:**
- Consumes: `HeaderText::parse_noodles`, `format::{Writer, ParallelWriter, RecordScratch}`
- Produces: `bcf::rewrite<R: Read, W: Write>(R, W, &Options) -> Result<Summary>`
- Produces: `bcf::rewrite_parallel<R: Read, W: Write + Send + 'static>(R, W, &Options, NonZeroUsize) -> Result<Summary>`
- Produces: `preserve_indices(&vcf::Header, &mut vcf::Header) -> Result<()>`
- Produces: private retained-index collection and next-free-index assignment helpers
- Produces: `reheader::write_parallel(input, &Options, output, workers) -> Result<Summary>`

- [ ] **Step 1: Write failing BCF integration and dictionary tests**

Use the existing `view` command to create raw and BGZF BCF fixtures, then decode outputs back to canonical VCF. Cover replacement, FAI, samples, combined edits, retained INFO/FORMAT/FILTER/contig indices, newly appended definitions, removal of a used definition, removal of a used contig, sample-count mismatch, truncated records, raw-encoding preservation, and threads on both allowed and rejected encodings.

```rust
struct BcfFixture {
    _guard: tempfile::TempDir,
    directory: PathBuf,
    input: PathBuf,
    sample_pairs: PathBuf,
    fai_without_used_contig: PathBuf,
    expected_records: Vec<u8>,
}

impl BcfFixture {
    fn from_vcf(encoding: &str) -> Self {
        let guard = tempfile::tempdir().unwrap();
        let directory = guard.path().to_path_buf();
        let source = directory.join("source.vcf");
        let input = directory.join(format!("input-{encoding}.bcf"));
        let sample_pairs = directory.join("samples.tsv");
        let fai_without_used_contig = directory.join("without-chr1.fai");
        fs::write(&source, BCF_DICTIONARY_FIXTURE).unwrap();
        fs::write(&sample_pairs, b"S1\tN1\nS2\tN2\n").unwrap();
        fs::write(&fai_without_used_contig, b"chr2\t100\t0\t0\t0\n").unwrap();
        assert_success(&run(&[
            "view", "-O", encoding, "-o", path(&input), path(&source),
        ]));
        let expected_records = decode_records(&input);
        Self {
            _guard: guard,
            directory,
            input,
            sample_pairs,
            fai_without_used_contig,
            expected_records,
        }
    }

    fn compressed() -> Self { Self::from_vcf("b") }
    fn sample_pairs(&self) -> &str { path(&self.sample_pairs) }
    fn fai_without_used_contig(&self) -> &str { path(&self.fai_without_used_contig) }
    fn expected_records(&self) -> &[u8] { &self.expected_records }
}

fn path(path: &Path) -> &str { path.to_str().unwrap() }

#[test]
fn raw_and_bgzf_bcf_preserve_typed_records_and_input_encoding() {
    for encoding in ["u", "b"] {
        let fixture = BcfFixture::from_vcf(encoding);
        let output = fixture.directory.join(format!("out-{encoding}.bcf"));
        let result = run(&[
            "reheader", "-N", fixture.sample_pairs(), "-o", path(&output), path(&fixture.input),
        ]);
        assert_success(&result);
        assert_eq!(detect_encoding(&output), encoding);
        assert_eq!(decode_records(&output), fixture.expected_records());
    }
}

#[test]
fn removing_a_used_bcf_contig_fails_without_replacing_output() {
    let fixture = BcfFixture::compressed();
    let output = fixture.directory.join("existing.bcf");
    fs::write(&output, b"keep").unwrap();
    let result = run(&[
        "reheader", "-f", fixture.fai_without_used_contig(), "-o", path(&output), path(&fixture.input),
    ]);
    assert!(!result.status.success());
    assert_eq!(fs::read(output).unwrap(), b"keep");
}
```

- [ ] **Step 2: Run BCF tests and verify RED**

```bash
cargo test --locked --test reheader_cli bcf_ -- --nocapture
```

Expected: tests fail because BCF input is unsupported.

- [ ] **Step 3: Build stable edited BCF dictionaries**

Preserve original `IDX` values for retained contigs and retained IDs shared by INFO, FILTER, and FORMAT. Assign new contig and string IDs after the highest occupied original index, sharing one new string index when the same ID appears in multiple namespaces.

```rust
fn preserve_indices(original: &vcf::Header, edited: &mut vcf::Header) -> Result<()> {
    assign_contig_indices(original.string_maps().contigs(), edited.contigs_mut())?;
    let mut assigned = retained_string_indices(original.string_maps().strings());
    assign_header_indices(edited.infos_mut(), &mut assigned)?;
    assign_header_indices(edited.filters_mut(), &mut assigned)?;
    assign_header_indices(edited.formats_mut(), &mut assigned)?;
    *edited.string_maps_mut() = vcf::header::StringMaps::try_from(&*edited)
        .map_err(|error| invalid(format!("building edited BCF dictionaries: {error}")))?;
    Ok(())
}
```

Check every retained name resolves at its original index before any output is written.

- [ ] **Step 4: Stream typed records through serial and parallel writers**

```rust
fn rewrite_records<R: Read>(
    mut reader: noodles_bcf::io::Reader<R>,
    original: &vcf::Header,
    edited: &vcf::Header,
    writer: &mut impl VariantWriter,
) -> Result<u64> {
    let mut raw = noodles_bcf::Record::default();
    let mut count = 0;
    loop {
        let read = reader.read_record(&mut raw).map_err(read_error)?;
        if read == 0 { break; }
        count += 1;
        let record = vcf::variant::RecordBuf::try_from_variant_record(original, &raw)
            .map_err(|error| record_error(count, error))?;
        writer.write_record(edited, &record, count)?;
    }
    writer.finish()?;
    Ok(count)
}
```

Use `Writer(OutputFormat::BcfRaw)` for raw input, `Writer(OutputFormat::Bcf)` for serial BGZF BCF, and `ParallelWriter(OutputFormat::Bcf, workers)` only for nonzero-thread BGZF BCF. Reject nonzero threads before opening output for every other encoding.

- [ ] **Step 5: Run BCF, CLI, and full local tests and verify GREEN**

```bash
cargo test --locked --test reheader_cli bcf_ -- --nocapture
cargo test --locked --test reheader_cli -- --nocapture
cargo test --locked --all-features
cargo test --release --locked --all-features
```

Expected: all tests pass with controlled errors rather than panics or partial destinations.

- [ ] **Step 6: Commit BCF support**

```bash
git add src/reheader.rs src/reheader/bcf.rs src/commands/reheader.rs tests/reheader_cli.rs
git commit -m "feat(vcf): rewrite BCF headers safely"
```

### Task 8: Complete bcftools 1.24 compatibility gate and CI integration

**Files:**
- Create: `tests/reheader_compat.rs`
- Create: `tests/upstream/bcftools-reheader/README.md`
- Create: `tests/upstream/bcftools-reheader/input.vcf`
- Create: `tests/upstream/bcftools-reheader/replacement.vcfh`
- Create: `tests/upstream/bcftools-reheader/reference.fai`
- Create: `tests/upstream/bcftools-reheader/samples.txt`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: `RSOMICS_BCFTOOLS` already provided by Linux x86_64 CI
- Produces: ignored release oracle `cargo test --test reheader_compat -- --ignored --test-threads=1`

- [ ] **Step 1: Write the failing live-oracle matrix**

Pin the first version line to `bcftools 1.24`. Compare complete plain and BGZF VCF output for header replacement, FAI, positional samples, pair samples, composition, and stdin. Compare raw and BGZF BCF as normalized headers and typed record bodies after `bcftools view --no-version -Ov`.

```rust
#[derive(Clone, Copy, Debug)]
enum TestEncoding { PlainVcf, BgzfVcf, RawBcf, BgzfBcf }

#[derive(Clone, Copy, Debug)]
enum EditCase { Header, Fai, PositionalSamples, PairSamples, Composed }

#[test]
#[ignore = "release oracle: requires bcftools 1.24"]
fn declared_success_matrix_matches_bcftools_1_24() {
    assert_oracle_version();
    for encoding in [
        TestEncoding::PlainVcf,
        TestEncoding::BgzfVcf,
        TestEncoding::RawBcf,
        TestEncoding::BgzfBcf,
    ] {
        for edit in [
            EditCase::Header,
            EditCase::Fai,
            EditCase::PositionalSamples,
            EditCase::PairSamples,
            EditCase::Composed,
        ] {
            assert_equivalent(encoding, edit);
        }
    }
    assert_stdin_equivalent(TestEncoding::PlainVcf, EditCase::Composed);
    assert_stdin_equivalent(TestEncoding::BgzfVcf, EditCase::Composed);
}

#[test]
#[ignore = "release oracle: requires bcftools 1.24"]
fn fail_loud_divergences_are_controlled() {
    for case in [
        Divergence::SampleCount,
        Divergence::UnknownSource,
        Divergence::DuplicateFinalSample,
        Divergence::UsedBcfContigRemoved,
        Divergence::TruncatedBgzf,
    ] {
        let ours = run_ours(case);
        assert!(!ours.status.success(), "{case:?}");
        assert!(!ours.status.code().is_some_and(|code| code >= 128), "{case:?}");
    }
}
```

The compatibility helper must derive expectations from bcftools output, while the divergence test asserts rsomics safety independently and never blesses bcftools warnings or crashes.

- [ ] **Step 2: Run the oracle and verify RED**

```bash
RSOMICS_BCFTOOLS="$(command -v bcftools)" \
  cargo test --locked --test reheader_compat -- --ignored --test-threads=1 --nocapture
```

Expected: the matrix executes against the live oracle. A clean first pass is valid; every mismatch must first become a focused non-ignored regression test before production code changes.

- [ ] **Step 3: Correct only behaviors exposed by the oracle**

For each mismatch, add the smallest non-ignored regression case to `tests/reheader_cli.rs`, observe it fail, then change the corresponding narrow production function. Keep the normalized BCF comparator explicit:

```rust
fn canonical_bcf(path: &Path) -> Vec<u8> {
    let output = require_success(Command::new(oracle()).args([
        OsStr::new("view"), OsStr::new("--no-version"), OsStr::new("-Ov"), path.as_os_str(),
    ]));
    strip_only_bcftools_provenance(output.stdout)
}
```

- [ ] **Step 4: Add the exact oracle invocation to CI**

Append to the existing Linux x86_64 compatibility step:

```yaml
RSOMICS_BCFTOOLS="$(command -v bcftools)" \
  cargo test --locked --test reheader_compat -- --ignored --test-threads=1
```

- [ ] **Step 5: Run the complete local gate and verify GREEN**

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo test --release --locked --all-features
RSOMICS_BCFTOOLS="$(command -v bcftools)" \
  cargo test --locked --test reheader_compat -- --ignored --test-threads=1
cargo package --locked
bash -n benchmarks/*.sh
```

Expected: every command exits zero with no warnings.

- [ ] **Step 6: Commit, push, and wait for exact-head native CI**

```bash
git add .github/workflows/ci.yml tests/reheader_compat.rs tests/upstream/bcftools-reheader
git commit -m "test(vcf): complete reheader compatibility gate"
git push origin main
head=$(git rev-parse HEAD)
run=$(gh run list --commit "$head" --workflow ci.yml --json databaseId --jq '.[0].databaseId')
gh run watch "$run" --exit-status
test "$(gh run view "$run" --json headSha --jq .headSha)" = "$head"
```

Expected: Linux and macOS jobs pass on native `x86_64` and `aarch64` at the exact pushed head.

### Task 9: Representative performance gate

**Files:**
- Create: `benchmarks/reheader-vs-bcftools.sh`
- Modify after measurement: `PERFORMANCE.md`

**Interfaces:**
- Produces: `generate RSOMICS_VCF BCFTOOLS WORKSPACE`
- Produces: `run RSOMICS_VCF BCFTOOLS WORKSPACE RESULT_DIRECTORY`
- Produces: raw paired timings, summary statistics, peak RSS, hashes, commands, versions, machine data, and git cleanliness

- [ ] **Step 1: Write the benchmark harness contract before optimization**

The `generate` mode creates on external storage a 2,000,000-record plain VCF, its BGZF equivalent, a replacement header, FAI, and sample mapping. The `run` mode verifies body identity before timing, performs three alternating warmups and ten alternating measured pairs, and rejects dirty product trees or non-1.24 bcftools.

```bash
case "${1:-}" in
  generate) generate_workload "$2" "$3" "$4" ;;
  run) run_gate "$2" "$3" "$4" "$5" ;;
  *) usage ;;
esac
```

Use `cmp` for plain body bytes, decompressed-body SHA-256 for BGZF, and raw-tail frame SHA-256 to ensure the fast path is the measured path. Record wall/user/system time and peak RSS with `/usr/bin/time -lp`.

- [ ] **Step 2: Run syntax and a reduced smoke workload and verify RED or GREEN meaningfully**

```bash
bash -n benchmarks/reheader-vs-bcftools.sh
SMOKE_RECORDS=10000 benchmarks/reheader-vs-bcftools.sh generate \
  "$CARGO_TARGET_DIR/release/rsomics-vcf" "$(command -v bcftools)" \
  /Volumes/KIOXIA/Developments/tmp/rsomics-vcf-reheader-smoke
BENCH_RUNS=2 BENCH_WARMUPS=1 RSOMICS_BENCH_SMOKE=1 \
  benchmarks/reheader-vs-bcftools.sh run \
  "$CARGO_TARGET_DIR/release/rsomics-vcf" "$(command -v bcftools)" \
  /Volumes/KIOXIA/Developments/tmp/rsomics-vcf-reheader-smoke \
  /Volumes/KIOXIA/Developments/tmp/rsomics-vcf-reheader-smoke/results
```

Expected: syntax passes; smoke mode either exposes a harness defect that is fixed before proceeding or writes complete provenance and equal-output evidence. The smoke ratio is never a release result.

- [ ] **Step 3: Commit the reproducible harness**

```bash
git add benchmarks/reheader-vs-bcftools.sh
git commit -m "test(vcf): add reheader performance gate"
```

- [ ] **Step 4: Build a clean release binary and run the representative gate**

```bash
cargo build --release --locked
workspace=/Volumes/KIOXIA/Developments/tmp/rsomics-vcf-reheader-gate-20260818
benchmarks/reheader-vs-bcftools.sh generate \
  "$CARGO_TARGET_DIR/release/rsomics-vcf" "$(command -v bcftools)" "$workspace"
benchmarks/reheader-vs-bcftools.sh run \
  "$CARGO_TARGET_DIR/release/rsomics-vcf" "$(command -v bcftools)" \
  "$workspace" "$workspace/results"
```

Expected: outputs match before timing; both plain VCF and BGZF VCF show a strict throughput or resource-use advantage. If either principal path is at or below parity, do not document a pass or bump the version; profile that path, add a regression test for every correctness bug found, and repeat the same clean-tree gate.

- [ ] **Step 5: Record measured evidence without broadening the claim**

Add a `reheader` section to `PERFORMANCE.md` containing the exact product revision, bcftools/HTSlib 1.24 identity, fixture hashes and sizes, output hashes, machine/OS/Rust data, commands, warmups/runs, timing distribution, peak RSS, and the per-path decision copied from the retained result files.

```bash
shasum -a 256 "$workspace"/inputs/* "$workspace"/results/*
git diff --check PERFORMANCE.md
git add PERFORMANCE.md
git commit -m "docs(vcf): record reheader performance gate"
```

### Task 10: Product documentation, release, and independent registry proof

**Files:**
- Modify: `README.md`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `/Volumes/Zane's HDD/Documents/rsomics-world/docs/10-products/variant.md`
- Modify: `/Volumes/Zane's HDD/Documents/rsomics-world/docs/plans/2026-08-13-vcf-reheader-design.md`

**Interfaces:**
- Produces: documented stable `reheader` command in `rsomics-vcf 0.5.0`
- Produces: exact-head four-native-target CI and publish run
- Produces: registry archive checksum, VCS identity, clean install, help, and oracle smoke evidence

- [ ] **Step 1: Document only the completed command**

Add the stable contract and examples to `README.md` after the command is fully green:

```console
rsomics-vcf reheader -H header.vcfh -o renamed.vcf calls.vcf
rsomics-vcf reheader -f reference.fa.fai -N samples.tsv -o renamed.bcf calls.bcf
rsomics-vcf reheader -n tumor,normal calls.vcf.gz > renamed.vcf.gz
```

Describe same-encoding output, atomic named output, fail-loud sample and dictionary rules, raw BGZF tail preservation, and BCF-only threads. Do not mention `setgt` as available.

- [ ] **Step 2: Run a fresh API and hot-path review**

Inspect every production `unwrap`, comment, allocation, clone, unbounded buffer, and public item introduced by the feature:

```bash
rg -n "unwrap\(|expect\(|TO[D]O|FIXM[E]|//|pub " src/reheader.rs src/reheader src/commands/reheader.rs src/format/bgzf.rs
git diff 203b11974adf719f24ac485fbcc8d02fa77e5423 --stat
git diff 203b11974adf719f24ac485fbcc8d02fa77e5423 -- src README.md PERFORMANCE.md Cargo.toml Cargo.lock
```

Expected: production unwraps are restricted to statically obvious invariants, comments explain only stable format reasons, buffers are bounded by a header or one 64 KiB frame, and no item is public outside the crate.

- [ ] **Step 3: Prepare version 0.5.0 and run the full release gate**

Change `Cargo.toml` package version to `0.5.0`, refresh only the root package entry in `Cargo.lock`, and run:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo test --release --locked --all-features
RSOMICS_BCFTOOLS="$(command -v bcftools)" \
  cargo test --locked --test reheader_compat -- --ignored --test-threads=1
cargo package --locked
bash -n benchmarks/*.sh
git diff --check
```

Expected: every gate exits zero and `cargo package --list --locked` contains no scratch or evidence artifacts.

- [ ] **Step 4: Commit, push, and require exact-head CI**

```bash
git add README.md PERFORMANCE.md Cargo.toml Cargo.lock
git commit -m "chore: prepare rsomics-vcf 0.5.0"
git push origin main
head=$(git rev-parse HEAD)
run=$(gh run list --commit "$head" --workflow ci.yml --json databaseId --jq '.[0].databaseId')
gh run watch "$run" --exit-status
test "$(gh run view "$run" --json headSha --jq .headSha)" = "$head"
```

Expected: the exact release head passes all four native jobs and the pinned Linux oracle.

- [ ] **Step 5: Publish from the exact release head**

```bash
gh workflow run publish.yml --ref main
publish_run=$(gh run list --workflow publish.yml --branch main --json databaseId,headSha \
  --jq ".[] | select(.headSha == \"$head\") | .databaseId" | head -1)
gh run watch "$publish_run" --exit-status
test "$(gh run view "$publish_run" --json headSha --jq .headSha)" = "$head"
```

Expected: crates.io accepts `rsomics-vcf 0.5.0` from the exact CI-tested head.

- [ ] **Step 6: Independently verify the registry artifact on external storage**

Create a fresh external Cargo home and target, download the crates.io archive, compare its SHA-256 with the crates.io API checksum, inspect `.cargo_vcs_info.json`, compare the unpacked package tree with the local `cargo package`, install with `--locked`, and rerun one complete plain, BGZF, raw-BCF, and BGZF-BCF oracle case.

```bash
verify=$(mktemp -d /Volumes/KIOXIA/Developments/tmp/rsomics-vcf-0.5.0-verify.XXXXXX)
CARGO_HOME="$verify/cargo-home" CARGO_TARGET_DIR="$verify/target" \
  cargo install rsomics-vcf --version 0.5.0 --locked --root "$verify/install"
"$verify/install/bin/rsomics-vcf" --version
"$verify/install/bin/rsomics-vcf" reheader --help
```

Expected: version/help are correct, registry VCS equals the release head, archive checksums agree, package trees agree, and installed outputs match bcftools 1.24 under the declared comparator.

- [ ] **Step 7: Record the release in the control plane and wait for its CI**

Update the variant dossier with the final product revision, exact CI and publish run IDs, checksum, install binary hash, oracle hashes, performance decision, and explicit `setgt` exclusion. Mark the design status released.

```bash
cd "/Volumes/Zane's HDD/Documents/rsomics-world"
python3 scripts/validate_control_plane.py
git diff --check
git add docs/10-products/variant.md docs/plans/2026-08-13-vcf-reheader-design.md
git commit -m "docs(vcf): record reheader release"
git push origin main
head=$(git rev-parse HEAD)
run=$(gh run list --commit "$head" --workflow control-plane.yml --json databaseId --jq '.[0].databaseId')
gh run watch "$run" --exit-status
test "$(gh run view "$run" --json headSha --jq .headSha)" = "$head"
```

Expected: the control-plane exact head is green and the retained evidence distinguishes implemented, validated, published, and excluded work.

## Plan self-review

- Spec coverage: every CLI, edit-order, encoding, transaction, error, compatibility, performance, CI, and publication requirement maps to Tasks 1-10.
- Placeholder scan: the plan contains no accepted-but-unimplemented public behavior; intermediate local commits are not pushed until the full compatibility gate is green.
- Type consistency: `HeaderText`, `SampleSource`, `Options`, `Summary`, `Encoding`, `FrameReader`, and the serial/parallel writer signatures are introduced before their consumers and retain the same names throughout.
- Execution choice: inline execution with `superpowers:executing-plans`, because the user authorized unattended execution and the current developer policy does not authorize subagent delegation.
