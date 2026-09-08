# K-mer consumer and boundary recheck

Status: the short-sequence repair passes both consumers on all four native
platforms, including successful artifact retention at the second candidate.
Two equivalent guards measured slower in the original x86_64 comparisons. The
subsequent inlining hint made performance worse and has been withdrawn.
Equal-source controls now demonstrate substantial measurement confounding,
including a stable x86_64 difference in a same-process diagnostic. The earlier
ratios cannot be attributed entirely to the guard. No performance pass,
corrected registry release or delivered product repair is claimed.
Local product builds remain stopped by the physical boot-storage gate.

## Identities and evidence scope

The three local worktrees were clean during the initial source inspection. Their exact-head
GitHub Actions results were re-read; all four native target jobs succeeded.
Those old runs do not cover the newly identified missing regression.

| Repository | Inspected head | Successful exact-head CI |
|---|---|---|
| `rsomics-kmer` | `d89e2df0d8eae38b64eb7b43a41f57436fc25bb4` | `30734719265` |
| `rsomics-seq` | `d9734e51c4ed557f6d8790d97a686717ebc4769e` | `31377799041` |
| `rsomics-sketch` | `823fabe3ca092d6e0b30c235cfa961d80c45a543` | `30735400718` |

The retained August release dossier records the published versions and archive
checksums. This recheck does not refresh crates.io state or reproduce the old
performance distributions.

## Actual consumers

Every locally present accepted product's manifest and production source was
searched for the k-mer dependency and imports. Only sequence and sketch have
direct call sites:

| Product | Production call site | Consumed contract | Consumer-side evidence |
|---|---|---|---|
| `rsomics-seq` | `src/operations/kmers.rs::count_kmers` | `KmerCounts::try_new`, checked counting, `decode`; exact two-bit windows with `k` in 1–32 | `tests/kmers_oracle.rs` compares full CLI rows with an independent byte-window implementation; CLI tests cover invalid `k` and frozen tables |
| `rsomics-sketch` | `src/sketch.rs::build` | `CanonicalMurmur64::try_new` and `hashes`; arbitrary nonzero `k`, full-width seed, strand/case canonicalization and explicit invalid windows | `tests/oracle.rs` compares complete signature bytes with sourmash 4.9.4 for `k` 1/17/31/51 and a seed above u32; product tests cover ambiguity and output preservation |

The foundation now has two real product consumers. The old TODO asking to add
a second consumer is stale and can be closed. This does not establish two
consumers for each public API: sequence does not use `CanonicalMurmur64`, and
sketch does not use the exact-count accumulator. Metagenomics has no current
k-mer import; its future minimizer contract does not yet pin a matching hash,
seed, byte order, ambiguity policy and consumer test.

Keep the accepted foundation and existing versioned interfaces while their
maintenance work proceeds. Do not invent a second call site, add a public
crate, or force an unsuitable hash into metagenomics to justify the abstraction.
Before expanding the hashing API, supply two concrete product contracts for
the new item or keep it product-private. Any later internalization of existing
public items needs a compatible migration plan; nothing was removed here.

## P0: short input reaches an out-of-bounds slice

At the inspected k-mer head, `CanonicalMurmur64Hashes::next` returns exhaustion
only when `start > sequence.len().saturating_sub(k)`. For `start = 0`,
`sequence = b"ACG"`, and `k = 31`, that condition is `0 > 0`, so execution
continues to `sequence[..31]` on a three-byte slice. Empty input has the same
path. The iterator's `size_hint` reports zero, but calling `next` reaches an
out-of-bounds slice instead of returning `None`.

The sketch builder calls this iterator on each parsed sequence without a
length filter, so ordinary short FASTA/FASTQ records reach the affected path.
This is not an error in the exact-count sequence operation, which uses a
different iterator. The initial inspection established this from source;
the later runtime reproductions are recorded below.

The pre-repair canonical-hash tests cover exact-length windows, longer inputs,
case/strand normalization, ambiguity, long `k`, scratch reuse and allocation
failure. They omit `len < k`. The sourmash differential uses a 20,000-base
generated sequence for its `k` 1/17/31/51 cases, so it also misses this boundary.
The green CI results therefore do not contradict the source finding.

## Executed repair evidence

| Scope | Commit | CI run | Verified result |
|---|---|---|---|
| Foundation regression | `45782dac92236d9befa1b58adee362ab590a37e1` | [34249477451](https://github.com/omics-rust/rsomics-kmer/actions/runs/34249477451) | Both new empty/short tests panic on the old implementation on all four native platforms; the diagnostic step explicitly requires these failures |
| Foundation repair | `86fbbcec1ce1819024540a7d6817e55d5dcce6b3` | [34249836770](https://github.com/omics-rust/rsomics-kmer/actions/runs/34249836770) | 45 unit tests and all three new integration tests pass normally on all four native platforms; lint/package/benchmark smoke pass |
| Published sketch consumer | `8bacc91b4ad8892f93e21adbf02211eeedbf7703` | [34250061363](https://github.com/omics-rust/rsomics-sketch/actions/runs/34250061363) | The new sourmash differential reaches a bounds panic on registry kmer 0.2.2; CI explicitly expects this failure, so this is reproduction, not repair |

The first production change checks `sequence.len().saturating_sub(start) < k`
before accessing the window. It adds no public item, scratch buffer or
per-record allocation. The integration tests cover empty input, `k-1`, `k`,
`k+1`, repeated exhaustion, exact `len`/`size_hint`, and hasher reuse.

The sketch oracle regression compares complete signatures for mixed-length
and all-short FASTA/FASTQ, with `k` 31 and 51 (eight fixture combinations).
It remains an external-oracle test, matching the existing oracle suite's
execution convention. Its normal CI diagnostic must be removed when the
product moves to the fixed registry dependency.

The manual foundation `Consumer contracts` workflow pins both product heads
and both real upstream oracles. A runner-local Cargo configuration selects the
exact foundation Git SHA; resolved metadata must prove that SHA, and all later
commands remain `--locked`. Product manifests are not rewritten to Git/path
dependencies. Run [34250441947](https://github.com/omics-rust/rsomics-kmer/actions/runs/34250441947)
failed before compilation because a command-line override was not inherited
by a Cargo subprocess. Run
[34251056970](https://github.com/omics-rust/rsomics-kmer/actions/runs/34251056970)
then exposed a missing Cargo-home directory before configuration creation.
Both setup defects were corrected at
`1fa83cf05f1ec3f057c28d3d05a116b7e60c6ae9`. In
[34251307513](https://github.com/omics-rust/rsomics-kmer/actions/runs/34251307513),
all eight consumer validation steps passed. One macOS ARM sketch job failed
only while finalizing its uploaded artifact (intermediary HTTP 403), so its
exact job was retried once, with the same artifact-only failure. This first
candidate's overall run remains failed. The second candidate's complete
consumer run [34252192237](https://github.com/omics-rust/rsomics-kmer/actions/runs/34252192237)
passed all eight jobs, including artifact upload; this is the closed consumer
gate. The first run was not relabelled as green.

An independent read-only review approved the first candidate's correctness
and test coverage, but withheld release approval because of the performance
result below. It found no new public API, allocation, or valid-hash change.

## Measured performance hold

Run [34251528752](https://github.com/omics-rust/rsomics-kmer/actions/runs/34251528752)
compares baseline `d89e2df0d8eae38b64eb7b43a41f57436fc25bb4` with candidate
`a17e32f65b7e1b1f8beee1b63acd0ed1e2571647` (the first guard). It asserts
identical benchmark source, Cargo manifest and lockfile, builds both before
measurement, then alternates baseline/candidate order across five pairs.
Each pair uses Criterion 0.7, 50 samples, one-second warmup and a requested
three-second measurement on the existing 1,048,576-byte `k=31`, seed-42
fixture. All raw samples, estimates, logs and machine/source provenance are
retained; these are real measurements, not `--test` smoke.

| Native runner | CPU | Median of five candidate/baseline median-time ratios |
|---|---|---|
| Ubuntu x86_64 | AMD EPYC 9V74 | 1.199778; performance hold |
| Ubuntu aarch64 | Neoverse-N2 | 1.002289 |
| macOS aarch64 | Apple M1 (Virtual) | 0.999841 |
| macOS x86_64 | See retained provenance | 1.148879; performance hold |

The x86_64 ratios were 1.199778, 1.070189, 1.443603, 1.160394 and 1.200397.
This is about 20% longer median elapsed time, not a 20% throughput loss.
It does not support a no-regression decision. Shared-runner variation is
visible in other rounds; the effect is not attributed to compiler behavior
without assembly evidence.

The next candidate, production commit `0ba84a6`, uses the equivalent guard
`len < k || start > len - k`. Short-circuit evaluation protects subtraction
on short inputs while retaining a loop-invariant end boundary. At head
`ff0357d`, ordinary CI and both consumers
([34252192237](https://github.com/omics-rust/rsomics-kmer/actions/runs/34252192237))
passed. Paired benchmark collection
([34252186622](https://github.com/omics-rust/rsomics-kmer/actions/runs/34252186622))
also completed, but the median paired-time ratios still fail the performance
gate: Linux x86_64 1.161514, Linux ARM 0.966408, macOS x86_64 1.098681 and
macOS ARM 1.071380. Successful collection is not a performance pass.

The benchmark caller assembly for this comparison is byte-identical across
baseline and candidate; both call the iterator's `next` for each window.
Separate library assembly from
[34252843366](https://github.com/omics-rust/rsomics-kmer/actions/runs/34252843366)
shows changed guard instructions, block layout and register use, but does not
establish the cause of the slowdown. That diagnostic run's Linux artifacts
are available; both macOS artifact uploads failed. It is not a timing run.

An independent review recommended one bounded cross-crate optimization
experiment: ordinary `#[inline]` on `next`, leaving the second guard and
iterator state unchanged. Commit `0924135` adds only that attribute. Candidate
head `1dcb6a71b7dcab8c8095b63d474e4fbd9e9686b5` runs six balanced triplets
against both released `d89e2df` and corrected, non-inlined `23e42f3` controls
in [34253569461](https://github.com/omics-rust/rsomics-kmer/actions/runs/34253569461).
Consumer verification is
[34253574988](https://github.com/omics-rust/rsomics-kmer/actions/runs/34253574988).
Both library and caller assembly are retained to check what the compiler
actually did. The experiment failed its measured gate and was withdrawn by
`2f2eda21729dc049524cfedb77b77ffdead41aaa`:

| Completed native runner | Median candidate/released time ratio | Median candidate/corrected time ratio |
|---|---|---|
| Ubuntu x86_64 | 1.808787 | 1.620417 |
| Ubuntu aarch64 | 1.140139 | 1.122348 |
| macOS aarch64 | 1.240617 | 1.209461 |
| macOS x86_64 | 1.089177 | 0.939991 |

All four measurement jobs completed successfully. Intel macOS alone favors
inlining over the corrected control, but does not override the other three
platforms' regressions. The inlining consumer run completed with an
artifact-only HTTP 403 on Linux x86_64 sketch; it is not needed to approve an
experiment that has been rejected. No additional blind local guard change
follows it.

At the withdrawal head, `src`, tests, benchmarks and both Cargo files are
byte-identical to corrected control `23e42f3`. Run
[34254249238](https://github.com/omics-rust/rsomics-kmer/actions/runs/34254249238)
therefore supplies an equal-source candidate/reference control while retaining
the released baseline. Linux x86_64's six candidate/reference ratios are
0.825729, 1.001047, 0.833103, 1.055229, 0.994564 and 0.945402: identical source,
library assembly and caller assembly can still differ by approximately 17%
within a pair. Intel macOS's median is 1.037087. The run completed with three
successful jobs and a macOS ARM artifact-finalization HTTP 403 after
measurement. Its missing ARM artifact is not represented as retained evidence.

### Same-process null control

Commit `22faa44` replaces the separate Criterion processes with a CI-only,
unpublished comparison harness. It compiles three pinned dependency identities
in one binary and balances all six orders across 60 measured triplets after
six warmup triplets. Linux pins one allowed CPU. Input generation, `k=31`,
seed 42 and XOR reduction match the old workload; each observation verifies
the checksum, but this is not a substitute for full consumer oracles.

Run [34256249038](https://github.com/omics-rust/rsomics-kmer/actions/runs/34256249038)
passed all four native jobs at `3d4b7516c9914724ebc1c96031b046c9c71cbf1c`.
Normal exact-head CI
[34256246005](https://github.com/omics-rust/rsomics-kmer/actions/runs/34256246005)
also passed. Every artifact has 198 observations, including 180 measurements,
the three expected resolved Git identities, equal checksums, and fixture SHA-256
`ecfc819faea7528e21043367dc95845d311fb1ffaf19b3989be82067a0869f3d`.
Candidate and corrected reference have identical production sources.

| Native runner | Median candidate/reference time ratio | Median candidate/released time ratio |
|---|---|---|
| Ubuntu x86_64 | 0.906807 | 0.902147 |
| Ubuntu aarch64 | 1.002371 | 1.009129 |
| macOS x86_64 | 0.924302 | 0.921327 |
| macOS aarch64 | 1.014703 | 1.004022 |

These are diagnostic observations, not optimization results. The Linux x86_64
equal-source ratio stays between 0.893190 and 0.927590 across all 60 pairs,
so balancing time and CPU affinity alone does not remove the systematic bias.
The compiled harness has three separate caller loops and three distinct
iterator symbols; labels remain tied to call sites, object locations and
linked code locations. Identify or counterbalance that confound before using
small timing differences to approve or reject a production change. No
additional production optimization was made for this diagnostic.

The benchmark and consumer workflows now use `actions/upload-artifact@v7.0.1`.
This aligns their artifact client with its supported runtime; successful new
uploads do not establish the cause of earlier intermittent HTTP 403 failures.

### Shared-scan diagnostic and decision

Commit `61de048cb1fce0c860beb22e78d2cdf9233aac39` keeps all three dependency
revisions fixed and uses one non-inlined checksum over a borrowed iterator.
Only the selected iterator is constructed in a common enum payload before
timing. Production code is unchanged. Both exact-head normal CI
[34257483815](https://github.com/omics-rust/rsomics-kmer/actions/runs/34257483815)
and the four-native diagnostic
[34257486430](https://github.com/omics-rust/rsomics-kmer/actions/runs/34257486430)
completed successfully, including retained artifacts.

The Linux x86_64 executable has one checksum function at `0x1a740`, with the
repeated indirect iterator call at `0x1a77c`. Its three unrolled outer call
sites correspond to the balanced temporal positions and all pass iterator
state at `rsp + 0x38`. An independent read-only review checked these facts and
the retained binary/source hashes. This establishes that the intended shared
scan and common iterator-state address were actually compiled.

| Native runner | Median candidate/reference time ratio | Median candidate/released time ratio |
|---|---|---|
| Ubuntu x86_64 | 0.789447 | 0.773894 |
| Ubuntu aarch64 | 1.009003 | 0.973069 |
| macOS x86_64 | 0.859599 | 0.861923 |
| macOS aarch64 | 1.007179 | 1.003740 |

All four artifacts contain the expected 198 observations, including 60 full
measured triplets, identical fixture bytes and the three pinned dependency
identities. The equal-source Linux x86_64 ratios range from 0.763333 to
0.866569. Distinct library code/jump-table addresses, dispatch targets and
hasher scratch-buffer addresses remain; their actual contribution is not
established. The intervention therefore did not validate this microbenchmark
as a way to attribute the guard's performance effect.

Decision: measurement execution and provenance are valid; guard-specific
attribution and a no-regression conclusion are inconclusive. These runs do
not pass the performance release gate. Stop adding guard permutations or
further minor microbenchmark variants. Advance the actual consumer's release
gate on representative genome FASTA and gzip FASTQ abundance workloads.
Retain the older negative observations and the failed null controls; neither
selective omission nor a claimed synthetic speedup is justified.

The consumer comparison must pin the current sketch source, released and
corrected foundation identities and sourmash 4.9.4, prove resolved dependencies
and binary hashes, compare complete output bytes, and retain repeated timings
and peak RSS with input/machine/command provenance. The historical August
workloads motivate those choices, but their old performance numbers are not
evidence for a new release. The gate is the current product's strict
throughput or resource-use advantage, as required by `AGENTS.md`, with any
remaining workload-specific slowdown explicitly reported.

Both completed measurement sets, including all four platforms, are retained
outside scratch under
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/kmer-short-window-2026-09-09/`
in `first-guard` and `invariant-boundary`. Copies were compared recursively
against the downloaded source. First-run raw ZIP archives in
`archives-34251528752` also pass ZIP integrity checks and independently match
the GitHub artifact digests:

| Artifact ID | SHA-256 |
|---|---|
| `10066232031` | `4e31382b0a251cc660e29d3ac13bfd5e88ef2d79dfc7afdc69d14de0b3999c64` |
| `10066239983` | `d404cededd5ae0b924932b676af5e6648daac4ab616fdbec3b551278fa47c5aa` |
| `10066311402` | `a86086049318459a3554bf62085741cf68870d30dfc88f6201343e56d5f0901d` |
| `10066607598` | `31f243affa8f62adfae9260e6e31e3e423ac6bc322dc6a18c064de1e028c4ff8` |

Scratch copies remain under
`/Volumes/KIOXIA/Developments/tmp/kmer-repair-evidence-20260909-619Btb`.
The complete four-platform inlining set, three retained equal-source
platforms, and all four same-process controls have also been copied and
recursively compared in `inlining`, `equal-source` and `interleaved-control`.
The final four-platform shared-scan diagnostic is also retained and recursively
compared in `shared-scan-control`, including all binaries and raw observations.
No measurements or failed-candidate evidence was deleted.

## Remaining repair gate

Before the next k-mer or sketch release:

1. Complete both consumer suites and pinned upstream differentials with the
   exact candidate on all four native platforms.
2. Measure the affected hot path through the real sketch consumer against
   the pre-fix artifact and sourmash, retaining timing distributions and RSS
   with provenance. Benchmark smoke and the failed same-source microbenchmark
   controls are not performance passes. The source has no new allocation
   sites; this is not a measured process-memory result.
3. Finish the normal publication gates, then align the sketch minimum
   dependency and lockfile with the fixed registry release. Remove the
   expected-failure diagnostic, rerun exact-head CI and publish the consumer
   repair before calling it delivered.

The physical-storage restriction is unchanged. No registry version or secret
setting was changed. The published sketch dependency remains affected.
