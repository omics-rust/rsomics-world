# `rsomics-vcf setgt` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> `superpowers:executing-plans` to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the complete typed `rsomics-vcf setgt` operation and publish it
as version 0.6.0 only after correctness, compatibility, performance, package,
and four-native-platform gates pass.

**Architecture:** Extend the private typed expression engine with genotype
predicates, extract a private genotype-edit layer shared with `filter`, and
build a sequential setgt processor over the existing typed reader and writer.
Keep the command out of the public command tree until the engine is complete;
then add the unified CLI, oracle, benchmark, and release evidence.

**Tech Stack:** Rust 1.91 MSRV; Rust 1.97.1 development toolchain; Clap 4;
noodles VCF/BCF/BGZF; rsomics-common 0.12; rsomics-help 0.4; serde; existing
Rayon-backed BGZF writer; bcftools and HTSlib 1.24 oracle.

**Spec:** `docs/plans/2026-08-19-vcf-setgt-design.md`

## Global Constraints

- Product repository: `/Volumes/KIOXIA/Documents/omics-rust/rsomics-vcf`.
- Control repository: `/Volumes/Zane's HDD/Documents/rsomics-world`.
- Before every Cargo command, export
  `CARGO_HOME=/Volumes/KIOXIA/Developments/cargo-home`,
  `CARGO_TARGET_DIR=/Volumes/KIOXIA/Developments/cargo-target/rsomics-vcf`,
  `TMPDIR=/Volumes/KIOXIA/Developments/tmp`, and prepend
  `/opt/homebrew/Cellar/rust/1.97.1/bin` to `PATH`.
- Stop before compiling if `/` is at or above 80% usage or if Cargo's resolved
  target directory is not under `/Volumes/KIOXIA/Developments/cargo-target`.
- Use direct commits to `main`, one concern per commit, no pull request, no
  worktree, and no `Co-Authored-By` trailer.
- Preserve unrelated worktree changes and every user-owned untracked file.
- Add no dependency and no Layer A API unless implementation evidence proves
  the existing private layers cannot satisfy the approved spec.
- Keep comments rare; only stable invariants and non-obvious compatibility or
  safety reasons may be comments. Clap help remains user-facing API text.
- No partial `setgt` command may appear in public help or README. Register it
  only in Task 5 after Tasks 1 through 4 are complete and green.
- Every behavior group follows red-green-refactor TDD and ends with a focused
  commit.
- Publication requires exact-head CI on native Linux and macOS for both
  `x86_64` and `aarch64`, then independent registry download and install.

Run this preflight before the first Cargo command in every task:

```bash
export CARGO_HOME='/Volumes/KIOXIA/Developments/cargo-home'
export CARGO_TARGET_DIR='/Volumes/KIOXIA/Developments/cargo-target/rsomics-vcf'
export TMPDIR='/Volumes/KIOXIA/Developments/tmp'
export PATH="/opt/homebrew/Cellar/rust/1.97.1/bin:$PATH"
test "$(df -Pk / | awk 'NR == 2 { gsub(/%/, "", $5); print $5 }')" -lt 80
case "$CARGO_TARGET_DIR" in
  /Volumes/KIOXIA/Developments/cargo-target/*) ;;
  *) exit 1 ;;
esac
rustc --version
cargo metadata --locked --offline --no-deps --format-version 1 \
  | python3 -c 'import json,sys; p=json.load(sys.stdin); assert p["target_directory"].startswith("/Volumes/KIOXIA/Developments/cargo-target/"); print(p["target_directory"])'
```

---

### Task 1: Add typed genotype expression predicates

**Files:**

- Modify: `src/expression/value.rs`
- Modify: `src/expression/evaluate/comparison.rs`
- Modify: `src/expression/evaluate.rs`

**Interfaces:**

- Consumes: existing `value::Atom::Genotype(Genotype)` and comparison dispatch.
- Produces:
  `Genotype::spelling(&self) -> String` and
  `Genotype::matches_class(&self, pattern: &str) -> Option<bool>` for exact,
  named, and symbolic genotype predicates.
- Preserves: all numeric, missing, ordinary string, set, and regex comparison
  behavior already covered by the expression suite.

- [ ] **Step 1: Write failing expression tests**

Add focused tests beside the existing expression evaluator tests. The fixture
must include haploid, diploid, polyploid, phased, mixed-phase, partial-missing,
complete-missing, ref-alt, and alt-alt genotypes. Assert the exact bcftools
1.24 token distinctions:

```rust
assert_eq!(truth("GT = 'het'", &header, &record).sample_passes(),
           Some(&[true, false, true, true, false, false][..]));
assert_eq!(truth("GT = 'AA'", &header, &record).sample_passes(),
           Some(&[false, true, false, false, false, false][..]));
assert_eq!(truth("GT = 'Aa'", &header, &record).sample_passes(),
           Some(&[false, false, true, false, false, false][..]));
assert_eq!(truth("GT = './.'", &header, &record).sample_passes(),
           Some(&[false, false, false, false, true, false][..]));
assert_eq!(truth("GT ~ '^1[|/]0$'", &header, &record).sample_passes(),
           Some(&[true, false, false, false, false, false][..]));
```

Also assert `!=`, `mis`, `ref`, `alt`, `hom`, `hap`, `RR`, `RA`, `AR`, `R`,
`A`, exact `.|.`, concrete `0/1`, and an invalid ordering comparison error.

- [ ] **Step 2: Run the focused tests and verify the existing failure**

```bash
cargo test --locked expression::evaluate::tests::genotype -- --nocapture
```

Expected: failure containing `expected a string value` for genotype/string
comparison.

- [ ] **Step 3: Implement genotype spelling and class matching**

In `value.rs`, render the first allele without a separator and each later
allele with its stored `/` or `|`. Implement named classes case-insensitively.
Implement symbolic classes by case pattern so same-case `AA`/`aa` means
alternate homozygous while mixed-case `Aa`/`aA` means heterogeneous alternate
alleles. Keep `RR`, `RA`/`AR`, `R`, and `A` distinct.

In `comparison.rs`, handle genotype versus text before ordinary text
coercion. Equality uses a recognized class or exact spelling; inequality
negates it. Regex renders the exact genotype spelling. Reject ordered and set
comparisons for genotypes with a typed error.

- [ ] **Step 4: Run expression and filter regression suites**

```bash
cargo test --locked expression:: -- --nocapture
cargo test --locked --test filter_cli
cargo test --locked --test filter_compat -- --ignored --test-threads=1
```

Expected: all pass with `/opt/homebrew/bin/bcftools` reporting 1.24.

- [ ] **Step 5: Commit the expression increment**

```bash
git add src/expression/value.rs src/expression/evaluate/comparison.rs src/expression/evaluate.rs
git commit -m 'feat(vcf): support genotype expression classes'
```

---

### Task 2: Extract the private genotype edit and count layer

**Files:**

- Create: `src/genotype.rs`
- Create: `src/genotype/edit.rs`
- Create: `src/genotype/counts.rs`
- Modify: `src/lib.rs`
- Modify: `src/filter.rs`

**Interfaces:**

- Consumes: noodles `RecordBuf`, sample `Genotype`, `Allele`, and existing
  filter selection vectors.
- Produces:

```rust
pub(crate) enum MissingPolicy { Ignore, Error }
pub(crate) enum InfoPolicy { BestEffort, Strict }

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Change {
    pub(crate) genotypes: u64,
    pub(crate) alleles: u64,
}

pub(crate) fn edit_selected<F>(
    record: &mut RecordBuf,
    selected: &[bool],
    missing: MissingPolicy,
    edit: F,
) -> Result<Change>
where
    F: FnMut(usize, &Genotype) -> Result<Genotype>;

pub(crate) struct AlleleCounts {
    pub(crate) counts: Vec<u64>,
    pub(crate) total: u64,
}

pub(crate) fn allele_counts(record: &RecordBuf) -> Result<AlleleCounts>;
pub(crate) fn reconcile_ac_an(
    header: &vcf::Header,
    record: &mut RecordBuf,
    policy: InfoPolicy,
) -> Result<()>;
```

- [ ] **Step 1: Write failing helper tests**

Cover arbitrary ploidy, selected and unselected samples, missing GT key,
missing selected value, wrong GT type, allele/phase change counts, ploidy
growth and shrink, out-of-range alleles, AC/AN zero counts, absent tags,
invalid definitions, and strict versus best-effort reconciliation.

```rust
let change = edit_selected(
    &mut record,
    &[false, true, true],
    MissingPolicy::Error,
    |_, gt| Ok(gt.as_ref().iter().map(|allele|
        Allele::new(allele.position().map(|_| 0), Phasing::Unphased)
    ).collect()),
).unwrap();
assert_eq!(change, Change { genotypes: 2, alleles: 4 });
```

- [ ] **Step 2: Run the helper tests and verify the module is absent**

```bash
cargo test --locked genotype:: -- --nocapture
```

Expected: compilation failure until `genotype` and its declared interfaces
exist.

- [ ] **Step 3: Implement the focused modules**

`edit_selected` clones the FORMAT key set and sample rows once, validates the
selection width, calls the closure only for selected samples, compares old and
new typed alleles including phase and length, rebuilds `Samples`, and returns
checked `u64` counts. `MissingPolicy::Ignore` preserves filter's current no-GT
behavior; `Error` carries sample context for setgt.

`allele_counts` counts final typed alleles against `REF + ALT` and rejects
out-of-range positions. `reconcile_ac_an` updates only tags already present.
`InfoPolicy::Strict` validates standard header number/type and record value
type; `InfoPolicy::BestEffort` preserves an invalid existing value so filter
behavior does not change.

- [ ] **Step 4: Replace filter's local genotype mutation**

Map filter failures to a `selected` vector, call `edit_selected` with
`MissingPolicy::Ignore`, fill every allele with missing or reference while
preserving ploidy, and call best-effort AC/AN reconciliation. Remove the old
local `valid_ac`, `valid_an`, decrement, and adjustment helpers only after all
their tests have equivalents.

- [ ] **Step 5: Run filter and full library regression tests**

```bash
cargo test --locked genotype:: -- --nocapture
cargo test --locked filter:: -- --nocapture
cargo test --locked --test filter_cli
cargo test --locked --test filter_compat -- --ignored --test-threads=1
cargo test --locked --lib
```

- [ ] **Step 6: Commit the private extraction**

```bash
git add src/genotype.rs src/genotype src/lib.rs src/filter.rs
git commit -m 'refactor(vcf): share typed genotype edits'
```

---

### Task 3: Parse setgt targets, replacements, and random state

**Files:**

- Create: `src/setgt.rs`
- Create: `src/setgt/target.rs`
- Create: `src/setgt/replacement.rs`
- Create: `src/setgt/random.rs`
- Modify: `src/lib.rs` with `#[cfg(test)] mod setgt;` only

**Interfaces:**

```rust
pub(crate) struct Target {
    pub(crate) principal: Principal,
    pub(crate) random_fraction: Option<f64>,
}

pub(crate) enum Principal {
    AnyMissing,
    PartialMissing,
    CompleteMissing,
    All,
    Query,
    Binomial(Binomial),
}

pub(crate) struct Binomial {
    pub(crate) tag: String,
    pub(crate) comparison: Comparison,
    pub(crate) threshold: f64,
}

pub(crate) enum Replacement {
    Missing,
    Reference { phased: bool },
    Minor { phased: bool },
    Major { phased: bool },
    Depth,
    Phase,
    Unphase,
    Invert,
    Custom(Template),
}

pub(crate) struct Random48 { state: u64 }
```

`Target::parse(&[String])`, `Replacement::parse(&str)`, and
`Random48::new(i64)` return typed values or `RsomicsError::ConfigError`.
`Random48::next() -> f64` implements the exact 48-bit state transition.

- [ ] **Step 1: Write failing parser and RNG tests**

Assert every accepted spelling and reject empty, duplicate, conflicting,
nonfinite, boundary-fraction, malformed binomial, overflowing allele, empty
template term, trailing separator, and undocumented compound forms.

```rust
let mut random = Random48::new(7);
assert_eq!(random.next().to_bits(), 0x3fd10d6bf5d44040);
assert_eq!(Target::parse(&["./x".into(), "r:0.25".into()]).unwrap().random_fraction,
           Some(0.25));
assert!(Replacement::parse("0u").is_err());
assert!(Replacement::parse("c:0/2|.").is_ok());
```

Derive the expected RNG bit patterns once from a tiny bcftools/HTSlib 1.24
probe retained under external scratch; do not derive expectations from the
Rust implementation under test.

- [ ] **Step 2: Run tests and verify the parsers are absent**

```bash
cargo test --locked setgt::target::tests -- --nocapture
cargo test --locked setgt::replacement::tests -- --nocapture
cargo test --locked setgt::random::tests -- --nocapture
```

- [ ] **Step 3: Implement typed parsers without a character mask**

Parse a principal target by exact whole spelling, parse optional `r:` only as
a second orthogonal selector, and parse binomial expressions with one checked
operator. Parse replacements into variants and custom terms carrying their
preceding separator as noodles `Phasing`. No branch may depend on incidental
character presence.

- [ ] **Step 4: Implement the portable 48-bit generator**

```rust
const MASK: u64 = (1 << 48) - 1;
const MULTIPLIER: u64 = 0x5deece66d;
const ADDEND: u64 = 0xb;

let low = seed as u32 as u64;
let state = (low << 16) | 0x330e;
self.state = self.state.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND) & MASK;
self.state as f64 / (1u64 << 48) as f64
```

- [ ] **Step 5: Run parser, RNG, format, and Clippy checks**

```bash
cargo test --locked setgt:: -- --nocapture
cargo clippy --locked --lib --all-features -- -D warnings
```

- [ ] **Step 6: Commit the configuration model**

```bash
git add src/setgt.rs src/setgt src/lib.rs
git commit -m 'feat(vcf): model setgt rules'
```

---

### Task 4: Implement typed selection and replacement

**Files:**

- Modify: `src/setgt.rs`
- Modify: `src/setgt/target.rs`
- Modify: `src/setgt/replacement.rs`
- Modify: `src/genotype/counts.rs`
- Modify: `src/expression/mod.rs`
- Modify: `src/expression/evaluate.rs`
- Modify: `src/expression/evaluate/function.rs`
- Modify: `src/expression/evaluate/function/binomial.rs`

**Interfaces:**

```rust
pub(crate) struct Program {
    target: Target,
    replacement: Replacement,
    expression: Option<Compiled>,
    random: Option<Random48>,
}

pub(crate) enum Query {
    Include(String),
    Exclude(String),
}

impl Program {
    pub(crate) fn bind(
        header: &vcf::Header,
        target: Target,
        replacement: Replacement,
        query: Option<Query>,
        seed: i64,
    ) -> Result<Self>;

    pub(crate) fn apply(
        &mut self,
        header: &vcf::Header,
        record: &mut RecordBuf,
        number: u64,
    ) -> Result<genotype::Change>;
}
```

- [ ] **Step 1: Write failing selector tests**

For a single typed multi-sample record, assert exact vectors for every missing
state, all, site query, sample query, include, exclude, selected-sample mask,
binomial, and fixed-seed random composition. Assert record/sample context in
missing GT, wrong GT, short AD/binomial, negative depth, and out-of-range
allele errors.

- [ ] **Step 2: Run selector tests and verify `Program` is absent**

```bash
cargo test --locked setgt::tests::selects -- --nocapture
```

- [ ] **Step 3: Implement principal selection**

Read typed genotypes once, preserve the distinction between absent value and
zero ploidy, evaluate `Compiled` truth for query mode, and evaluate the
existing two-tailed binomial routine for complete diploid heterozygotes.
Expose `binomial_two_sided(i32, i32) -> Option<f64>` through a private
`expression` re-export rather than copying its numerical implementation. Add
`Truth::sample_selection(&self) -> Option<&[bool]>` so exclusion computes
`selected && !passes` and never turns an unselected sample into a target.

Apply the random draw after principal selection in sample order. Consume no
draw for a failed principal predicate.

- [ ] **Step 4: Write failing replacement tests**

Cover missing/reference ploidy preservation, phased variants, stable
unphasing order, diploid inversion, unchanged non-diploid inversion,
minor/major ties, no-called-allele failure, AD ties and missing values, custom
ploidy growth/shrink, mixed separators, symbolic terms, out-of-range custom
alleles becoming missing, and precise change counts.

- [ ] **Step 5: Implement record and sample replacement contexts**

Compute pre-edit allele counts once when `m` or `M` is used. Validate AD only
for selected samples when `X` occurs. Resolve the replacement to a new typed
`Genotype`, call `genotype::edit_selected` with `MissingPolicy::Error`, and
run strict AC/AN reconciliation only when a genotype changed.

- [ ] **Step 6: Run all engine tests and library regression**

```bash
cargo test --locked setgt:: -- --nocapture
cargo test --locked genotype:: -- --nocapture
cargo test --locked expression:: -- --nocapture
cargo test --locked --lib
cargo clippy --locked --lib --all-features -- -D warnings
```

- [ ] **Step 7: Commit the typed engine**

```bash
git add src/setgt.rs src/setgt src/genotype/counts.rs
git commit -m 'feat(vcf): apply typed setgt rules'
```

---

### Task 5: Add the complete stream and unified public command

**Files:**

- Create: `src/setgt/stream.rs`
- Create: `src/commands/setgt.rs`
- Create: `tests/fixtures/setgt.vcf`
- Create: `tests/setgt_cli.rs`
- Modify: `src/setgt.rs`
- Modify: `src/lib.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/cli.rs`

**Interfaces:**

```rust
#[derive(Clone, Debug)]
pub(crate) struct Options {
    pub(crate) output_format: OutputFormat,
    pub(crate) target: Target,
    pub(crate) replacement: Replacement,
    pub(crate) query: Option<Query>,
    pub(crate) seed: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct Summary {
    pub(crate) read: u64,
    pub(crate) changed_records: u64,
    pub(crate) changed_genotypes: u64,
    pub(crate) changed_alleles: u64,
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

- [ ] **Step 1: Write failing CLI and stream integration tests**

The fixture must include declared AC/AN, AD, DP, GQ, biallelic and
multiallelic records, mixed ploidy, partial and complete missingness, and a
record with valid missing AD. Test:

```rust
let output = run(&["setgt", "-t", ".", "-n", "0", fixture]);
assert!(output.status.success());
assert!(String::from_utf8_lossy(&output.stdout).contains("0/0"));

let envelope: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
assert_eq!(envelope["result"]["command"], "setgt");
assert_eq!(envelope["result"]["summary"]["changed_genotypes"], 3);
```

Also test rsomics-help layout, every argument conflict, named-output rollback,
stdin/stdout, JSON separation, serial and parallel restrictions, sites-only
no-op, absent GT failure, and all four output round trips.

- [ ] **Step 2: Run CLI tests and verify `setgt` is unknown**

```bash
cargo test --locked --test setgt_cli -- --nocapture
```

Expected: the binary rejects `setgt` because it is not yet registered.

- [ ] **Step 3: Implement the sequential stream**

Follow `filter::stream`: open `format::Reader`, read and bind the header before
writing it, process one `RecordBuf` at a time, apply `Program`, aggregate
checked counters, write every record in order, and call writer `finish` before
returning the summary. The parallel variant changes only the writer.

- [ ] **Step 4: Register the complete command**

Add Clap arguments matching the spec, parse them into typed `Target` and
`Replacement`, enforce JSON/output separation, use `AtomicFile` for named
output, add `Command::Setgt`, `CommandOutput::Setgt`, dispatch, and one focused
help-layout test. Remove `#[cfg(test)]` from the setgt module only now.

- [ ] **Step 5: Run the public command and full local regression suite**

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --test setgt_cli -- --nocapture
cargo test --locked --all-features
cargo test --release --locked --all-features
```

- [ ] **Step 6: Commit the complete command surface**

```bash
git add src/setgt.rs src/setgt/stream.rs src/commands/setgt.rs src/lib.rs \
  src/commands/mod.rs src/cli.rs tests/fixtures/setgt.vcf tests/setgt_cli.rs
git commit -m 'feat(vcf): add typed setgt command'
```

---

### Task 6: Pin the bcftools 1.24 compatibility and divergence matrix

**Files:**

- Create: `tests/setgt_compat.rs`
- Create: `tests/upstream/bcftools-setgt/README.md`
- Create: `tests/upstream/bcftools-setgt/core.vcf`
- Create: `tests/upstream/bcftools-setgt/malformed-ad.vcf`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**

- Consumes: public `rsomics-vcf setgt`, pinned `RSOMICS_BCFTOOLS` or
  `bcftools`, and existing `view` for typed normalization.
- Produces: ignored release-oracle tests runnable serially and invoked by the
  Linux x86_64 CI oracle job.

- [ ] **Step 1: Write the normal-behavior oracle cases**

Create a table-driven test for `.`, `./x`, `./.`, `a`, `q` include/exclude,
binomial operators, fixed-seed random alone and composed, `.`, `0`, `0p`,
`m`, `mp`, `M`, `Mp`, `X`, `p`, `u`, `i`, and custom numeric and symbolic
templates. Compare normalized header and typed records, not tool provenance
lines. The normal-comparison fixture omits AC and AN so their intentional
reconciliation difference cannot mask ordinary genotype compatibility; a
separate divergence fixture contains both tags.

Encode the same core fixture as plain VCF, BGZF VCF, raw BCF, and BGZF BCF.
Exercise each input class and each output class; use a pairwise covering matrix
plus an all-four-output round trip for every replacement family.

- [ ] **Step 2: Verify the oracle test initially exposes mapping gaps**

```bash
RSOMICS_BCFTOOLS=/opt/homebrew/bin/bcftools \
  cargo test --locked --test setgt_compat -- --ignored --test-threads=1 --nocapture
```

Expected: any mismatch names the target, replacement, encoding, record, and
sample rather than emitting a single whole-file assertion.

- [ ] **Step 3: Fix only demonstrated ordinary-contract mismatches**

Change the typed parser or engine when the oracle and 1.24 source agree. Do not
weaken errors or add parser accidents to make a raw diff pass. Add a focused
unit regression for every implementation correction before rerunning the
matrix.

- [ ] **Step 4: Add explicit known-divergence tests**

Independently demonstrate and assert the rsomics result for:

- query inversion changing the selected sample rather than sample zero;
- malformed AD failing instead of silently preserving the record;
- AC/AN reconciliation after a GT edit;
- ambiguous new-GT combinations failing configuration;
- a pre-existing named destination surviving a late malformed record.

Each test must first capture the exact bcftools 1.24 behavior so the divergence
cannot become an unsupported narrative.

- [ ] **Step 5: Add the exact oracle command to CI and rerun local gates**

Append this to the existing Linux x86_64 compatibility step:

```bash
RSOMICS_BCFTOOLS="$(command -v bcftools)" \
  cargo test --locked --test setgt_compat -- --ignored --test-threads=1
```

Then run syntax, debug, release, and the complete ignored setgt oracle.

- [ ] **Step 6: Commit compatibility evidence**

```bash
git add tests/setgt_compat.rs tests/upstream/bcftools-setgt .github/workflows/ci.yml
git commit -m 'test(vcf): pin setgt compatibility gate'
```

---

### Task 7: Build and run the representative performance gate

**Files:**

- Create: `benchmarks/setgt-vs-bcftools.sh`
- Modify: `PERFORMANCE.md` only after a formal clean-head run

**Interfaces:**

- Consumes: release binary, bcftools 1.24, external scratch and target paths.
- Produces: generated fixture manifest, alternating raw trials, timing and RSS
  summaries, semantic hashes, machine/tool/revision provenance, and an
  explicit pass decision.

- [ ] **Step 1: Write the benchmark harness with a small smoke mode**

Generate a deterministic many-sample VCF outside the repository. Measure
all-to-missing, missing-to-reference, query-selected reference, plain VCF,
BGZF VCF, and BCF. Use three warmups and ten alternating measured pairs for a
formal run. Validate normalized output before timing and after every measured
pair. Record `git status --porcelain`, exact binary hashes, commands, input and
output hashes, wall, user, system, peak RSS, OS, CPU, and tool versions.

- [ ] **Step 2: Run shell syntax and a small smoke fixture**

```bash
bash -n benchmarks/setgt-vs-bcftools.sh
benchmarks/setgt-vs-bcftools.sh \
  --records 2000 --samples 8 --warmups 0 --runs 1 \
  --results /Volumes/KIOXIA/Developments/tmp/rsomics-vcf-setgt-smoke
```

Expected: every semantic comparison passes and the harness emits a decision
without division-by-zero or platform-specific awk behavior.

- [ ] **Step 3: Commit the reproducible harness**

```bash
git add benchmarks/setgt-vs-bcftools.sh
git commit -m 'test(vcf): add setgt performance gate'
```

- [ ] **Step 4: Build from a clean head and run the formal gate**

```bash
cargo build --release --locked
benchmarks/setgt-vs-bcftools.sh \
  --records 2000000 --samples 8 --warmups 3 --runs 10 \
  --results /Volumes/KIOXIA/Developments/tmp/rsomics-vcf-setgt-gate-20260819
```

The gate passes only if one representative operation has strictly lower
median wall time or peak RSS and all semantic hashes agree. Equal performance
does not pass.

- [ ] **Step 5: Record a passing formal result or trigger Task 8**

If the typed engine passes, add its exact provenance and bounded claim to
`PERFORMANCE.md` and skip Task 8. If no path passes, retain the result directory
and execute Task 8; do not alter `PERFORMANCE.md` and do not prepare a release.

---

### Task 8: Add a measured raw-VCF fast path when required

Execute this task only when Task 7 proves no strict advantage. The retained
Task 7 result is the evidence that makes this optimization necessary.

**Files:**

- Create: `src/setgt/raw.rs`
- Modify: `src/setgt.rs`
- Modify: `src/setgt/stream.rs`
- Modify: `src/format/text.rs` only for a narrow reusable header lookup
- Modify: `tests/setgt_cli.rs`
- Modify: `tests/setgt_compat.rs`

**Interfaces:**

```rust
pub(crate) enum RawPlan {
    AnyMissingToReference,
    AnyMissingToMissing,
    AllToReference,
    AllToMissing,
}

pub(crate) fn rewrite(
    record: &[u8],
    schema: &HeaderTypes,
    plan: RawPlan,
    output: &mut Vec<u8>,
) -> Result<Change>;
```

- [ ] **Step 1: Add failing equivalence and malformed-input tests**

Require raw and typed paths to produce identical typed records and summaries
for plain and BGZF VCF, arbitrary FORMAT key order, GT-only and multi-key
samples, mixed ploidy and phase, AC/AN present or absent, empty/missing sample
values, CRLF, long records, invalid columns, out-of-range alleles, and
truncation.

- [ ] **Step 2: Run focused tests and verify no raw plan exists**

```bash
cargo test --locked setgt::raw -- --nocapture
cargo test --locked --test setgt_cli setgt_raw -- --nocapture
```

- [ ] **Step 3: Implement the narrow checked rewrite**

Enable it only for VCF input and VCF/BGZF-VCF output, no expression,
binomial, random selector, custom replacement, or symbolic allele. Parse the
FORMAT column, locate GT, validate exactly the declared sample width, parse
every genotype allele and separator, rewrite only selected GT spans, and
recompute existing standard AC/AN. Any unsupported precondition selects the
typed path before the header is written; malformed records fail rather than
fall back after output begins.

- [ ] **Step 4: Run the full oracle and regression suites**

```bash
cargo test --locked --all-features
cargo test --release --locked --all-features
RSOMICS_BCFTOOLS=/opt/homebrew/bin/bcftools \
  cargo test --locked --test setgt_compat -- --ignored --test-threads=1
```

- [ ] **Step 5: Commit and rerun the exact formal benchmark**

```bash
git add src/setgt/raw.rs src/setgt.rs src/setgt/stream.rs src/format/text.rs \
  tests/setgt_cli.rs tests/setgt_compat.rs
git commit -m 'perf(vcf): accelerate common setgt rewrites'
```

Rerun Task 7's formal command from the clean new head. Proceed only after a
strict measured advantage. Add the exact passing result to `PERFORMANCE.md`.

---

### Task 9: Prepare, publish, and independently verify 0.6.0

**Files:**

- Modify: `README.md`
- Modify: `PERFORMANCE.md`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify after publication in control repo:
  `docs/10-products/variant.md`
- Modify after publication in control repo:
  `docs/plans/2026-08-19-vcf-setgt-design.md`

**Interfaces:**

- Consumes: passing Tasks 1 through 7, plus Task 8 only if triggered.
- Produces: exact product release head, four-native CI run, publish run,
  non-yanked registry archive, independent install, and control-plane release
  ledger.

- [ ] **Step 1: Perform a fresh API, hot-path, comments, and package review**

Verify all new product internals are private, no new dependency or public
foundation API exists, production errors propagate, only statically obvious
production invariants use `unwrap`/`expect`, source comments satisfy the
sparse-comment rule, stream memory is bounded, and every advertised syntax is
covered by a normal or explicit-divergence oracle case.

- [ ] **Step 2: Add stable user documentation and bump the version**

Document only the complete command, accepted grammar, examples, AC/AN policy,
and deliberate fail-loud differences in `README.md`. Set both manifest and
lockfile package version to `0.6.0`. Keep benchmark claims bounded to the exact
passing paths.

- [ ] **Step 3: Run a fresh full local release gate**

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo test --release --locked --all-features
RSOMICS_BCFTOOLS=/opt/homebrew/bin/bcftools \
  cargo test --locked --test setgt_compat -- --ignored --test-threads=1
bash -n benchmarks/*.sh
cargo package --locked
cargo run --locked -- setgt --help
cargo run --locked -- --version
```

- [ ] **Step 4: Commit and push the release candidate**

```bash
git add README.md PERFORMANCE.md Cargo.toml Cargo.lock
git commit -m 'chore: prepare rsomics-vcf 0.6.0'
git push origin main
```

If SSH port 22 is unavailable, push the same head through the authenticated
HTTPS repository URL without changing remote configuration. Record the full
40-character head.

- [ ] **Step 5: Wait for exact-head CI and publish from that head**

Use `gh run list` and `gh run view` to require the product CI run's
`headSha` to equal the release head and every native job to succeed. Then
dispatch the existing publish workflow for version `0.6.0`, require its
`headSha` to match, and wait for success. Do not publish locally or from a
different revision.

- [ ] **Step 6: Verify the registry independently**

Under a new directory from `mktemp -d` on KIOXIA:

- query crates.io and require 0.6.0 is non-yanked;
- download the registry archive and match its SHA-256 to the API checksum;
- require `.cargo_vcs_info.json` to name the exact release head and clean tree;
- compare the unpacked registry tree with a fresh local `cargo package` tree;
- install with fresh external Cargo home and target;
- verify version and complete help;
- smoke all four input and output encodings against bcftools 1.24;
- assert seeded random positions, AC/AN, and normalized typed records.

- [ ] **Step 7: Record the release in the control plane**

Update the variant dossier with exact commit, CI and publish run IDs,
registry checksum and size, package-tree hash, installed binary hash, oracle
smoke hashes, bounded performance result, and retained verification path.
Mark the design status released. Run:

```bash
python3 scripts/validate_control_plane.py
git diff --check
git add docs/10-products/variant.md docs/plans/2026-08-19-vcf-setgt-design.md
git commit -m 'docs(vcf): record setgt release'
git push origin main
```

Wait for the control repo exact-head CI to pass. Preserve unrelated untracked
files throughout.
