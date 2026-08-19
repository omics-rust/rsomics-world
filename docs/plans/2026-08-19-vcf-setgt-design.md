# `rsomics-vcf setgt` design

Status: authorized for implementation on 2026-08-19. This design is the
complete scope of `rsomics-vcf` 0.6.0.

## Purpose and boundary

`setgt` edits sample genotypes selected by genotype state, typed expressions,
allelic balance, or a reproducible random draw. It is a subcommand of the
existing `rsomics-vcf` product because it uses the same VCF/BCF data model,
sample expressions, genotype representation, encoding paths, and installation
identity as `filter`, `view`, and the other variant-format operations.

The compatibility oracle is bcftools 1.24 `+setGT`, its official help, and
the tag-1.24 `plugins/setGT.c` source whose SHA-256 is
`aeb47bf7c1a384f0a15248c7377127dfca1a6facd270fc46512329471d9b51de`.
VCF 4.1 through 4.5 and BCF2 remain the format authorities. The historical
`rsomics-vcf-setgt` revision
`a01b957b2259f4a75834c8354b2467cc3ea78cf6` supplies fixture, CLI, and behavior
seeds only. Its text-only reader, separate `rsomics-vcf-expr` dependency,
fixed eight-allele buffer, whole-file rewrite, direct destination truncation,
conditional oracle tests, and incomplete target and replacement surface are
discarded.

The implementation extends only private `rsomics-vcf` modules. It does not
revive an operation-sized repository, add a public foundation, or publish a
partial command. The command remains absent from public help and README until
the complete contract in this document passes its release gate.

## Stable command contract

The command is:

```text
rsomics-vcf setgt [OPTIONS] [VARIANT]
```

`VARIANT` defaults to standard input. `-o, --output FILE` defaults to standard
output. `-O, --output-type TYPE` accepts `v`, `z`, `b`, or `u` and defaults to
plain VCF. `--threads INT` controls BGZF compression workers; zero selects the
serial writer. Input encoding is detected by content and may be plain VCF,
BGZF VCF, raw BCF, or BGZF BCF.

Exactly one `-n, --new-gt TYPE` is required. One principal
`-t, --target-gt TARGET` is required. A second target may be supplied only as
`r:FLOAT` to apply a random fraction to the principal selector. `r:FLOAT`
alone means a random fraction of all genotypes. Repeated principal selectors,
repeated random selectors, or a fraction outside the open interval `(0, 1)`
fail during argument conversion.

Principal targets are:

- `.` for partially or completely missing genotypes;
- `./x` for partially but not completely missing genotypes;
- `./.` for completely missing genotypes at any ploidy or phase;
- `a` for all genotypes;
- `q` for samples selected by exactly one `-i, --include EXPR` or
  `-e, --exclude EXPR`;
- `b:TAG<CMP>VALUE` for complete diploid heterozygotes whose two-tailed
  binomial probability from integer `FORMAT/TAG` satisfies `<`, `<=`, `=`,
  `==`, `>=`, or `>` against a finite numeric value.

`-i` and `-e` are mutually exclusive and valid only with `q`. `q` requires one
of them. A binomial target does not accept an additional expression. `-s,
--seed INT` accepts a signed 64-bit integer only when a random selector is
present and defaults to zero. Random draws follow input-record and header
sample order, after the principal predicate passes.

Replacement forms are:

- `.` sets every allele to missing while preserving ploidy;
- `0` sets every allele to unphased reference while preserving ploidy;
- `0p` sets every allele to phased reference while preserving ploidy;
- `m` and `mp` set every allele to the unphased or phased second-most-common
  allele while preserving ploidy;
- `M` and `Mp` set every allele to the unphased or phased most-common allele
  while preserving ploidy;
- `X` sets every allele to the allele with greatest `FORMAT/AD` depth while
  preserving ploidy;
- `p` phases the existing genotype without reordering alleles;
- `u` unphases the existing genotype and sorts alleles by missingness then
  allele index;
- `i` reverses the two alleles of a diploid genotype while preserving whether
  the separator is phased;
- `c:GT` applies a validated genotype template and replaces ploidy.

Custom templates contain one or more `/`- or `|`-separated allele terms. A
term is a nonnegative integer, `.`, `m`, `M`, or `X`. Empty terms, signs,
overflow, trailing separators, and mixed or repeated separators without an
allele fail before input is opened. A numeric term beyond the current
record's alternate-allele count resolves to missing, matching the useful
bcftools behavior for a template applied across records with different allele
counts. Symbolic `m`, `M`, and `X` terms resolve per record and sample.

Undocumented character combinations accepted accidentally by the upstream
bit-mask parser are not part of the contract. In particular, conflicting
base replacements, `Xp`, replacements suffixed with `u` or `i`, and multiple
custom prefixes fail rather than selecting a branch by implementation order.

Global `--json` follows the existing product rule: variant output must use a
named file so JSON can use standard output. The summary reports records read,
records changed, genotypes changed, alleles changed, and output encoding.

## Selection semantics

Missing-state selectors inspect the actual typed alleles of each sample.
Vector-end padding in BCF is not ploidy. A zero-ploidy or absent sample value
is not silently classified as a completely missing genotype.

The query selector reuses the product-private compiled expression engine. A
site-valued true expression selects every sample and a site-valued false
expression selects none. A sample-valued expression uses its sample truth
vector. Exclusion inverts the corresponding site or selected-sample truth;
sample-selection masks remain outside the inversion.

The expression engine gains typed genotype comparisons required by the
bcftools 1.24 expression contract:

- exact missing spellings preserve phase and ploidy: `.`, `./.`, and `.|.`;
- `mis`, `ref`, `alt`, `hom`, `het`, `hap`, `RR`, `AA`, `RA` or `AR`, `Aa` or
  `aA`, `R`, and `A` are matched case-insensitively;
- equality and inequality operate on genotype classes or exact spellings;
- regex matching renders the typed genotype spelling and never coerces a
  genotype into an arbitrary ordinary string.

These changes remain inside `rsomics-vcf::expression` and are shared by
`filter` and future product operations. They do not create a public expression
crate.

The binomial selector requires an integer FORMAT definition and one value for
each REF plus ALT allele needed by the selected genotype. It considers only
complete diploid heterozygotes. Homozygous, haploid, polyploid, partially
missing, and completely missing genotypes do not pass. Missing values or an
undersized vector for a genotype that otherwise qualifies fail with record
and sample context rather than turning the whole operation into a no-op.

A random selector uses the same 48-bit linear congruential sequence as
HTSlib's seeded `drand48` path. The implementation owns this small state
directly so results are identical across supported operating systems and do
not depend on libc. The signed seed contributes its low 32 two's-complement
bits and initializes `X0 = (seed_low32 << 16) | 0x330e`. Each draw advances
`X = (0x5deece66d * X + 0xb) mod 2^48` and returns `X / 2^48` as `f64`, exactly
matching HTSlib 1.24's three-`u16` state. A draw is consumed only for a
genotype that passed the principal selector. Output compression workers never
change draw order.

## Replacement semantics

Each replacement operates on a typed genotype without a fixed ploidy limit.
Missing, reference, minor, major, and depth replacements preserve actual
sample ploidy. `p` changes each allele boundary to phased, `u` removes phase
and applies a stable missing-first numeric order, and `i` changes only
diploid genotypes. A non-diploid inversion is an unchanged genotype, not an
error.

Minor and major alleles are determined from the edited record's pre-edit
genotypes. Counts include reference and every declared alternate allele and
ignore missing alleles. The most common allele is the first allele with the
maximum count. The second-most-common allele is the first different allele
with the next maximum count, including a zero count. A record with no called
alleles or no second allele cannot resolve the requested symbol and fails.
This avoids allowing stale `INFO/AC` to choose a replacement different from
the genotype data being edited.

`X` requires `FORMAT/AD` with integer `Number=R` semantics. For each selected
sample, the first allele with maximum nonmissing depth wins. A syntactically
missing AD value produces a missing replacement allele. A negative depth,
wrong value type, extra value, undersized vector, or record-wide cardinality
failure is invalid input. An unselected sample's missing AD does not fail the
record.

Custom templates preserve every written separator. `m` and `M` resolve once
per record; `X` resolves independently for each selected sample. A symbolic
term that cannot resolve becomes missing only when the corresponding source
value is explicitly missing. Structural and cardinality failures remain
errors.

The edit result distinguishes an unchanged genotype from a changed genotype
and counts the number of allele slots whose encoded allele or phase changed.
Ploidy reduction counts each removed allele slot as an allele change; ploidy
expansion counts each added slot. Record and genotype counts increment only
once for each changed unit.

## Genotype-derived INFO reconciliation

After at least one genotype in a record changes, existing `INFO/AC` and
`INFO/AN` values are recomputed from the final typed genotypes. They are not
added when absent. Their header definitions must have the standard integer
`Number=A` and integer `Number=1` contracts respectively; conflicting
definitions or record value types fail. AC contains one count per ALT allele,
including zero, and AN counts every nonmissing called allele.

Other annotations are left unchanged. `AF`, `NS`, likelihood summaries, and
cohort-specific tags may encode estimation or subset policy that cannot be
inferred safely from this command. The documentation states that users who
need those tags regenerated should run their chosen annotation workflow.

## Format and transaction paths

The command always uses the existing typed `format::Reader` because every
successful operation inspects or mutates sample values. `format::Writer` and
`ParallelWriter` provide all four output encodings. The processor holds only
the current record, compiled selector state, reusable allele-count and AD
scratch buffers, and the random state. It does not reopen input or collect the
file.

A sites-only input is copied with zero changes. A header with samples but no
FORMAT/GT definition, a record without the declared GT series, a sample count
mismatch, or a non-genotype GT value fails. The command does not invent a
ploidy for records that lack genotypes.

Named output uses `rsomics-common::AtomicFile` and rejects aliases with the
input. Commit occurs only after input EOF, writer finish, flush, file sync,
and transaction finalization. Parse, expression, selection, edit,
compression, write, broken-pipe, finish, and sync errors propagate to the
top-level nonzero exit. Standard output cannot be transactional, but header,
selector, replacement, and expression binding finish before its first byte.

## Product structure

The implementation adds private modules:

```text
src/
├── genotype.rs
├── genotype/
│   ├── counts.rs
│   └── edit.rs
├── setgt.rs
└── setgt/
    ├── target.rs
    ├── replacement.rs
    ├── random.rs
    └── stream.rs
```

`src/commands/setgt.rs` owns Clap conversion, common-layer output,
transactions, and summary delivery. `setgt::target` parses and evaluates
principal and random selectors. `setgt::replacement` parses replacement
syntax and resolves record or sample symbols. `setgt::random` implements the
portable HTSlib-compatible sequence. `setgt::stream` owns the record loop.

`genotype::edit` owns product-private genotype mutation and change accounting.
`genotype::counts` owns checked typed allele counts and AC/AN reconciliation.
The existing `filter` replacement path moves to these shared internals without
changing its public `--set-GTs .|0` contract. Expression genotype classes are
implemented beside existing typed expression values and comparisons.

No Layer A API is added. `rsomics-common` continues to supply atomic output
and error contracts, and `rsomics-help` continues to supply the unified CLI
presentation. No setgt-only requirement has a second product consumer that
would justify a new public item.

## Deliberate compatibility differences

The ordinary successful contract is compared against bcftools 1.24. The
following observed upstream behaviors are defects or parser accidents and are
tested as explicit differences:

- query-selected phase inversion edits the selected sample, not the first
  sample repeatedly;
- malformed or missing required AD and binomial vectors fail instead of
  silently returning the original record;
- existing AC and AN cannot remain stale after genotype edits;
- ambiguous target and replacement combinations fail during configuration;
- destination files are transactional rather than partially truncated on a
  later record error;
- sample cardinality, GT type, allele index, and output-finalization failures
  are never downgraded to unchanged output or a warning.

The oracle matrix records each divergence separately. It does not normalize a
wrong upstream output into the expected rsomics output.

## Verification and release gate

Tests precede each implementation group. The local gate covers:

- CLI help, required options, conflicts, repeatability, stdin, stdout, JSON,
  aliases, output types, threads, seed validation, and unsupported compound
  syntax;
- target parsing and classification for missing, partial, complete, all,
  query, binomial, random-only, and random-composed forms;
- replacement parsing and editing for missing, reference, minor, major,
  depth, phase, unphase, inversion, custom templates, arbitrary ploidy,
  mixed phase, numeric overflow, and record-varying ALT counts;
- exact genotype class and spelling expression comparisons, site and sample
  truth, include and exclude logic, selected-sample masks, and regex behavior;
- two-tailed binomial boundaries, multiallelic allele lookup, non-qualifying
  ploidies, malformed cardinality, and missing values;
- exact seeded random sequences, draw consumption order, repeatability across
  output types and compression thread counts, fractions near zero and one,
  and signed seed conversion;
- AD winner, tie, missing, negative, wrong type, short, long, unselected
  missing, and custom-template resolution;
- AC/AN recomputation, zero counts, missing alleles, variable ploidy, absent
  tags, conflicting definitions, and invalid prior record values;
- plain and BGZF VCF plus raw and BGZF BCF inputs, all four outputs, standard
  input, sites-only files, absent GT, malformed and truncated records, and
  writer-finalization failure;
- named-output rollback for configuration, parse, expression, record,
  compression, finish, and sync failures;
- unchanged behavior of `filter --set-GTs .|0` after the private genotype
  helper extraction.

The pinned oracle suite uses bcftools and HTSlib 1.24. It compares every
declared normal target and replacement form across biallelic, multiallelic,
haploid, diploid, polyploid, phased, unphased, partially missing, and
completely missing fixtures. Encoding coverage includes all four input
classes and all four output classes. VCF comparisons normalize only tool
provenance header lines; BCF comparisons use typed headers and records.
Random cases compare exact selected sample positions for fixed seeds.

Expected divergence tests cover the upstream query-inversion sample-offset
bug, silent AD record skip, stale AC/AN output, permissive character-mask
combinations, and partial destination output. These tests require the rsomics
result described in this document and independently demonstrate the upstream
1.24 behavior.

The performance gate uses a representative many-sample file with mixed
missingness and a nontrivial changed fraction. It measures at least all-to-
missing, missing-to-reference, query-selected replacement, and BGZF/BCF
typed output against bcftools 1.24 with alternating order, warmups, repeated
trials, wall time, CPU time, peak RSS, command lines, versions, revision,
machine, input hash, and output semantic hash. Profiling may justify a private
raw VCF genotype fast path only after the typed implementation is correct.
Any fast path must retain identical validation and fall back before output
when its preconditions do not hold. Publication requires a strict throughput
or resource-use advantage on at least one representative setgt hot path.

Publication requires formatting, strict Clippy, debug and release tests, the
complete pinned oracle, representative performance evidence, package
verification, a fresh public-API and production-hot-path review, and exact-
head native CI on Linux and macOS for both `x86_64` and `aarch64`. After CI,
the published archive is downloaded independently, matched to the release
head and package tree, installed with a fresh external Cargo home and target,
and smoke-tested on all four encodings. Only then may 0.6.0 publish.
