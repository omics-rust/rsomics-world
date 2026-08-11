# BAM Split Consolidation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> `superpowers:executing-plans` to implement this plan task by task. Every
> production behavior follows a red-green-refactor cycle.

**Goal:** Add one production-grade `rsomics-bam split` operation that replaces
the four historical read-group, random-part, BED12-gene, and paired-end split
micro-crates without adding a foundation or placeholder surface.

**Architecture:** A small command adapter selects one mutually exclusive mode
and passes typed options into a public `split` module. Private mode, label,
BED12, and grouped-output modules share the validated alignment reader and one
all-or-nothing output set. BAM-to-BAM routing uses validated raw records;
SAM/CRAM paths use decoded records.

**Tech Stack:** Rust 1.91.0, clap 4.5, noodles 0.110, noodles-util 0.79,
rsomics-bamio 0.8.4, rsomics-common 0.12, rsomics-help 0.4, tempfile 3,
samtools 1.24, RSeQC 5.0.4.

## Global Constraints

- Work only in `/Volumes/KIOXIA/Documents/omics-rust/rsomics-bam`; build and
  scratch output stay on `/Volumes/Zane's HDD`.
- Before every Cargo invocation, verify `/` remains below 80% and set
  `CARGO_HOME`, `RUSTUP_HOME`, `TMPDIR`, and `CARGO_TARGET_DIR` to the external
  paths in `AGENTS.md`.
- Do not add a public foundation API. Split filenames, BED12 policy, routing,
  and mate projection remain product-owned.
- `rsomics-help` remains the only CLI presentation layer.
- Keep source comments rare; public API docs state contracts, not history or
  implementation phases.
- Never expose an unfinished mode in help or README.
- All named outputs are one grouped transaction and must preserve prior files
  on every failure.
- Commit one reviewable concern at a time directly to `main`; never add a
  co-author trailer.
- Do not publish until strict format, Clippy, ordinary tests, complete live
  oracles, representative performance, package verification, and exact-head
  native Linux/macOS x86_64/aarch64 CI all pass.

---

### Task 1: Import and Verify Oracle Assets

**Files:**

- Copy into: `tests/golden/split/read-group/`
- Copy into: `tests/golden/split/genes/`
- Copy into: `tests/golden/split/mates/`
- Create: `tests/golden/split/MANIFEST.tsv`

**Interfaces:**

- Produces only immutable fixture inputs and literal expected record bodies.
- No command, module, help entry, or production code is created in this task.

- [ ] **Step 1: Copy exact historical fixtures and record hashes**

Copy the two-RG BAM and captured record bodies, gene BAM/BED/goldens, mate BAMs,
indexes, and record bodies from the four audited repositories. Write their
SHA-256 values into `tests/golden/split/MANIFEST.tsv`; do not copy historical
source or CLI files.

- [ ] **Step 2: Verify every imported oracle independently**

Decode each BAM with samtools 1.24 and compare its record bodies to the copied
literal files. Run RSeQC 5.0.4 once into external scratch and require the gene
and mate outputs to match the same files. Recompute the manifest and require
no hash drift.

- [ ] **Step 3: Commit**

Commit only fixtures and their manifest as
`test(bam): import split oracle corpus`.

### Task 2: Make Output Transactions Own Their Targets

**Files:**

- Modify: `src/output.rs`

**Interfaces:**

- Changes `TransactionalFile<'a>` to `TransactionalFile { target: PathBuf,
  temporary: NamedTempFile, permissions: Option<Permissions> }`.
- Preserves `new(&Path)`, `reopen`, `file_mut`, `temporary_path`, `commit`, and
  `commit_all` call behavior for every existing command.

- [ ] **Step 1: Write the failing ownership regression test**

Add a test that creates a `PathBuf`, constructs a transaction from it, moves
the transaction after the original path leaves scope, writes bytes, commits,
and asserts the target bytes. The test must require the transaction to outlive
the borrowed local path at compile time.

- [ ] **Step 2: Verify RED**

Run `cargo test output::tests::transaction_owns_its_target`. Expected: borrow
checker failure against `TransactionalFile<'a>`.

- [ ] **Step 3: Store the canonical target as an owned `PathBuf`**

Clone the target in `new`; update error formatting and rename/persist calls to
borrow `self.target`. Do not change backup or rollback semantics.

- [ ] **Step 4: Verify GREEN and transaction regressions**

Run all `output::tests`, then the complete ordinary suite. Expected: every
existing single and grouped transaction test passes.

- [ ] **Step 5: Commit**

Commit as `refactor(bam): own transactional output targets`.

### Task 3: Implement Labels, Auxiliary Values, and Read-Group Routing

**Files:**

- Create: `src/split/mod.rs`
- Create: `src/split/label.rs`
- Create: `src/split/tag.rs`
- Create: `src/split/output.rs`
- Modify: `src/lib.rs`
- Modify: `src/commands/mod.rs`
- Create: `src/commands/split.rs`
- Modify: `src/cli.rs`
- Test: `tests/split.rs`

**Interfaces:**

```rust
pub enum Format { Sam, Bam, Cram }

pub enum Mode<'a> {
    ReadGroup,
    Tag([u8; 2]),
    Parts { count: usize, seed: u64, skip_unmapped: bool },
    Genes(&'a Path),
    Mates,
}

pub struct Options<'a> {
    pub mode: Mode<'a>,
    pub output_prefix: &'a Path,
    pub unaccounted: Option<&'a Path>,
    pub unaccounted_header: Option<&'a Path>,
    pub format: Format,
    pub maximum_outputs: usize,
    pub zero_pad: usize,
    pub reference: Option<&'a Path>,
    pub additional_threads: usize,
    pub program: Option<Program<'a>>,
}

#[derive(Serialize)]
pub struct OutputSummary { pub label: String, pub path: PathBuf, pub records: u64 }

#[derive(Serialize)]
pub struct Summary { pub outputs: Vec<OutputSummary>, pub skipped: u64 }

pub fn run(input: &Path, options: Options<'_>) -> Result<Summary>;
```

- [ ] **Step 1: Write and verify the failing command-contract test**

Add `split_help_exposes_one_mode_selector_set`, parse
`rsomics-bam split --help`, and require the literal options below. Run it and
confirm the failure is that `split` is unknown.

```rust
for option in [
    "--output-prefix", "--tag", "--parts", "--genes", "--mates",
    "--unaccounted", "--unaccounted-header", "--max-outputs",
    "--zero-pad", "--seed", "--skip-unmapped", "--output-fmt",
    "--reference", "--no-pg", "--threads",
] {
    assert!(help.contains(option), "missing {option} in {help}");
}
assert!(!help.contains("read-group\n"));
```

- [ ] **Step 2: Write failing literal tests for filename components**

Require `rg1 -> rg1`, `a/b -> a%2Fb`, `a%b -> a%25b`, backslash to `%5C`,
byte `0xff` to `%FF`, and empty input to fail. Require different byte strings
to produce different components.

- [ ] **Step 3: Verify RED, then implement the percent encoder**

Run the label unit test; expected missing module failure. Implement a single
byte loop preserving ASCII alphanumeric, dot, dash, and underscore and writing
uppercase `%HH` for all other bytes. Re-run until green.

- [ ] **Step 4: Write failing tag-value tests**

Use real validated raw BAM records and require `Z`, `H`, `c/C/s/S/i/I`,
negative integer padding, missing, and invalid `A/f/B` outcomes. Literal
expected labels include `-005` for value -5 at width 3 and `007` for 7.

- [ ] **Step 5: Verify RED, implement tag decoding, verify GREEN**

Return a three-way result: present bytes, unaccounted missing, or unaccounted
invalid type. Strip exactly one terminating NUL from `Z` and `H`; decode
integers little-endian without lossy casts.

- [ ] **Step 6: Write the failing two-RG integration test**

Run `split --no-pg -o PREFIX tworg.bam`. Require exactly `PREFIX.rg1.bam` and
`PREFIX.rg2.bam`, record counts 8 and 1, committed record bodies equal to the
literal goldens, and each decoded header to retain only its matching `@RG`.

- [ ] **Step 7: Verify RED, implement the complete typed command and grouped BAM routing**

Define mutually exclusive clap selectors in one `ArgGroup`, require
`--output-prefix`, add the `Command::Split` dispatch and typed public API, then
precreate one sink per header RG, derive each header independently, write
validated raw records without decoding on BAM-to-BAM input, finish every
writer, and call one `TransactionalFile::commit_all`. Add one program record
to every derived header when requested.

- [ ] **Step 8: Add missing, unknown, empty-group, integer-tag, and limit tests**

Each behavior gets one test that first fails against the current branch:

- missing/unknown RG fails and leaves prior output bytes intact;
- `--unaccounted` receives all missing records and empty RG outputs commit;
- a declared empty RG creates a zero-record output;
- `--tag NM` produces labels `0`, `3`, `4`, and `6` with literal counts;
- `--max-outputs 2` fails or routes the remaining three records;
- `--tag RG` synthesizes exactly one RG line for an undeclared value;
- unsafe label bytes cannot escape the prefix directory;
- any computed output aliasing input or unaccounted output fails before commit.

- [ ] **Step 9: Implement each failing branch one at a time**

Do not batch behavior before observing its test fail. Validate canonical target
identities when each dynamic sink is created. The unaccounted replacement
header must match every reference name and length in order.

- [ ] **Step 10: Verify the command contract and complete read-group/tag matrix**

Run the help test, `cli::tests::command_tree_is_valid`, split unit/integration
tests, and all ordinary tests. The command becomes visible only in this green,
functional commit.

- [ ] **Step 11: Commit**

Commit as `feat(bam): add transactional tag splitting`.

### Task 4: Add Deterministic Random Parts

**Files:**

- Create: `src/split/parts.rs`
- Modify: `src/split/mod.rs`
- Modify: `tests/split.rs`

**Interfaces:**

```rust
pub(super) struct SplitMix64(u64);
impl SplitMix64 {
    pub(super) fn new(seed: u64) -> Self;
    pub(super) fn index(&mut self, upper: usize) -> usize;
}
```

- [ ] **Step 1: Write failing generator-vector and exact-cover tests**

Pin the first eight 64-bit outputs for seed 0 as literals. Run the nine-record
fixture with three parts and require total count nine, no repeated query name,
the exact input-name set, and identical membership across two seed-7 runs.

- [ ] **Step 2: Verify RED and implement SplitMix64**

Use wrapping arithmetic and checked conversion of a nonzero `upper`; do not add
an RNG dependency. Verify the vector and exact-cover tests pass.

- [ ] **Step 3: Write and satisfy skip/limit failure tests**

Require `--parts 0` and part count above `--max-outputs` to fail before any
final file. Require `--skip-unmapped` to remove exactly flag-0x4 records and no
others.

- [ ] **Step 4: Run all split tests and commit**

Commit as `feat(bam): consolidate random BAM partitioning`.

### Task 5: Add Strict BED12 Gene Routing

**Files:**

- Create: `src/split/bed.rs`
- Modify: `src/split/mod.rs`
- Modify: `tests/split.rs`

**Interfaces:**

```rust
pub(super) struct ExonIndex { by_reference: Vec<Vec<Range<i32>>> }
impl ExonIndex {
    pub(super) fn read(path: &Path, header: &sam::Header) -> Result<Self>;
    pub(super) fn contains(&self, reference_id: i32, position: i32) -> Result<bool>;
}
```

- [ ] **Step 1: Write failing BED12 table tests**

Use literal rows to require comments/track/browser skipping, adjacent/overlap
merge, case-sensitive reference names, and point containment at start,
end-minus-one, and excluded end.

- [ ] **Step 2: Add one failing test per malformed contract**

Cover fewer than 12 fields, negative/start-after-end coordinates, zero block
count, count/list mismatch, invalid integers, zero block size, block overflow,
block outside transcript, and unknown reference. Every case must name the
one production validation whose removal would make it pass.

- [ ] **Step 3: Implement strict streaming BED12 parsing and merged indexes**

Read lines with `BufRead::read_until`, preserve reference bytes, parse only
required fields, use checked addition, sort ranges, merge overlaps and
adjacency, and use `partition_point` for lookup. Re-run after each validation.

- [ ] **Step 4: Write the failing RSeQC routing-golden test**

Require five `in`, two `ex`, two `junk`, and exact record-body hashes for all
three files. Add an explicit record whose CIGAR overlaps an exon but leftmost
start is outside and require `ex`.

- [ ] **Step 5: Implement gene mode and transaction preservation**

Route `UNMAP|QCFAIL` to junk before reference lookup. For every other record,
validate a nonnegative, in-range reference and start, then perform the point
test. Finish and commit all three sinks together.

- [ ] **Step 6: Verify tests and commit**

Run all split tests and all ordinary tests. Commit as
`feat(bam): consolidate BED12 gene splitting`.

### Task 6: Add Paired-End Projection

**Files:**

- Create: `src/split/mates.rs`
- Modify: `src/split/mod.rs`
- Modify: `tests/split.rs`

**Interfaces:**

```rust
pub(super) const MATE_KEEP_FLAGS: u16 = 0x10 | 0x100 | 0x200 | 0x400;
pub(super) fn project_raw(record: &mut RawRecord);
pub(super) fn project_decoded(record: &mut sam::alignment::RecordBuf);
```

- [ ] **Step 1: Write failing ordinary and flag-corpus golden tests**

Require byte-identical decoded record bodies for `R1`, `R2`, and `unmap` on
both retained fixtures. Require mapped-without-READ1 to route to `R2` and
unmapped records to remain unchanged.

- [ ] **Step 2: Verify RED and implement raw projection**

Retain only `MATE_KEEP_FLAGS`, clear all other flags, set mate reference and
position to -1, and template length to zero. Route before mutation. Verify the
BAM goldens pass.

- [ ] **Step 3: Write failing SAM/CRAM decoded-path equivalence tests**

Convert the same corpus to SAM and reference-backed CRAM with test utilities,
run mate mode to each supported output format, and compare normalized headers,
record order, flags, and mate fields to the BAM result.

- [ ] **Step 4: Implement decoded projection and multi-format writers**

Use `RecordBuf` mutation for non-raw paths. SAM and BAM use the existing output
writer; CRAM uses the noodles alignment writer with the indexed reference
repository. Require a reference for CRAM output and finish before commit.

- [ ] **Step 5: Verify all formats/modes and commit**

Run split tests, complete ordinary debug tests, and release tests. Commit as
`feat(bam): consolidate paired alignment splitting`.

### Task 7: Close Compatibility, UX, and Failure Gates

**Files:**

- Modify: `tests/split.rs`
- Create: `tests/split_compat.rs`
- Modify: `.github/workflows/ci.yml`
- Modify: `README.md`
- Modify: `src/cli.rs`

**Interfaces:**

- Adds ignored `samtools_1_24_split_oracle` and
  `rseqc_5_0_4_split_oracles` tests.
- Adds `CommandOutput::Split { summary: split::Summary }` to the shared JSON
  envelope.

- [ ] **Step 1: Write failing JSON and help-experience tests**

Require the envelope command name `split`, stable per-output labels, paths and
counts, and `skipped`; require mode conflicts to fail through shared help and
errors. Require `--json` not to mix machine output with progress text.

- [ ] **Step 2: Complete the final adapter and machine summary**

Map clap values into `split::Options`, construct the program record, call
`split::run`, and return the typed summary. Keep human success silent; JSON is
emitted only by `rsomics-common::run`.

- [ ] **Step 3: Add grouped rollback and corrupt-input tests**

Start with literal previous bytes at every target. Trigger malformed SAM/BAM,
invalid tag type, malformed BED, output directory failure, and late writer
failure. Require all prior bytes unchanged and no new final path.

- [ ] **Step 4: Add and run live oracle matrices**

For samtools 1.24 compare decoded bodies and relevant `@RG` headers for default,
empty, missing/unknown with unaccounted, explicit RG, integer NM, limits,
padding, SAM input, BAM input, CRAM input, and no-PG. For RSeQC 5.0.4 compare
gene and mate bodies exactly and parts by disjoint-cover invariants.

- [ ] **Step 5: Run strict local verification**

Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D
warnings`, debug tests, release tests, ignored live oracles, and `cargo package
--locked`. Build the unpacked package with Rust 1.91.0. Record exact commands,
versions, hashes, and outputs.

- [ ] **Step 6: Update README only after all modes pass**

Add the `split` command row, examples for the four selectors, input/output
formats, transactional guarantee, strict BED behavior, encoded labels, and
explicit exclusions. Do not mention a benchmark advantage yet.

- [ ] **Step 7: Commit and pass exact-head four-native CI**

Commit as `test(bam): close split compatibility gates`, push, and wait for the
exact SHA. Linux x86_64 must run the complete samtools/RSeQC oracle or consume
committed goldens when RSeQC is intentionally unavailable; no silent skip can
count as a pass.

### Task 8: Re-run Performance and Publish the Complete Slice

**Files:**

- Create: `tools/benchmark-split.sh`
- Create: `.autopilot/performance/split-0.27.0/manifest.tsv`
- Create: `.autopilot/performance/split-0.27.0/timings.tsv`
- Create: `.autopilot/performance/split-0.27.0/summary.json`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `docs/10-products/bam.md` in the control repository after release

**Interfaces:**

- Benchmark script accepts explicit rsomics, samtools, RSeQC, BAM, BED12, and
  output-root paths and never downloads data.
- Every round emits normalized output fingerprints and peak RSS beside timing.

- [ ] **Step 1: Write the failing benchmark smoke test**

Run the script against the small corpus with one round and require four mode
rows, tool versions, fixture hashes, output fingerprints, wall/user/system
times, peak RSS, and a nonzero exit if any output comparison differs.

- [ ] **Step 2: Implement and verify the benchmark harness**

Use alternating paired order, separate per-run output directories on Zane,
explicit `-@ 0` single-thread comparison, and cleanup by moving generated
outputs to the external Trash only after hashes are captured.

- [ ] **Step 3: Run representative measurements**

Use `bench_3M.bam` plus its index and `gene_model_chr12.bed12`. Run default RG
against samtools 1.24 and parts, genes, and mates against RSeQC 5.0.4. If the
fixture lacks multiple RGs, generate a deterministic multi-RG derivative and
record its construction and hash. Require complete correctness before timing.

- [ ] **Step 4: Review the hot path if any mode is not strictly advantageous**

Do not publish on an equal/slower result. Profile the specific mode, write a
failing performance-sensitive regression only when it guards a functional
resource contract, optimize without changing outputs, and rerun the full
matrix.

- [ ] **Step 5: Record the performance decision and commit**

Add machine, versions, fixture sizes/hashes, rounds, distributions, output
fingerprints, peak RSS, and per-mode verdicts. Commit product evidence as
`docs(bam): record split performance evidence`.

- [ ] **Step 6: Prepare and verify version 0.27.0**

Bump package metadata, run the complete release gate again, package locked,
compile the unpacked archive, and commit as
`chore: prepare rsomics-bam 0.27.0`.

- [ ] **Step 7: Push, wait exact-head CI, publish, and verify live registry**

After all four native targets and the Linux x86_64 package/oracle job pass,
run the manual publication workflow. Verify crates.io checksum, VCS revision,
unyanked status, byte-identical downloaded archive, and a fresh external-disk
registry install with split help plus one smoke per mode.

- [ ] **Step 8: Close the control-plane ledger and continue**

Replace planned language in `docs/10-products/bam.md` with exact revisions,
CI run IDs, archive hashes, registry evidence, and benchmark verdicts. Commit,
push, wait exact-head control-plane CI, then select the next real BAM operation
without pausing at the phase boundary.
