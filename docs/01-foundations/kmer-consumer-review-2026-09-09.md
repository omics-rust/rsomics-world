# K-mer consumer and boundary recheck

Status: the short-sequence repair passes both consumers on all four native
platforms, including successful artifact retention at the second candidate.
Two equivalent guards nevertheless regressed in x86_64 measurements. The
subsequent inlining hint made performance worse and has been withdrawn.
An equal-source measurement control is running before further structural
changes. No corrected registry release or delivered product repair is claimed.
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

The Intel macOS measurement is still running; the three completed platforms
already reject the attribute. The inlining consumer run has an artifact-only
HTTP 403 on Linux x86_64 sketch; it is not needed to approve an experiment
that has been rejected. No additional blind local guard change follows it.

At the withdrawal head, `src`, tests, benchmarks and both Cargo files are
byte-identical to corrected control `23e42f3`. Run
[34254249238](https://github.com/omics-rust/rsomics-kmer/actions/runs/34254249238)
therefore supplies an equal-source candidate/reference control while retaining
the released baseline. Resolve measurement variation and review the iterator's
internal representation before another production optimization. This does not
relax the performance threshold, change a public API or authorize publication.

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
No measurements or failed-candidate evidence was deleted.

## Remaining repair gate

Before the next k-mer or sketch release:

1. Complete both consumer suites and pinned upstream differentials with the
   exact candidate on all four native platforms.
2. Measure the affected hot path against the pre-fix baseline and retain raw
   timing distributions and provenance. Benchmark smoke is not a performance
   decision. The source has no new allocation sites; this is not a measured
   process-memory result.
3. Finish the normal publication gates, then align the sketch minimum
   dependency and lockfile with the fixed registry release. Remove the
   expected-failure diagnostic, rerun exact-head CI and publish the consumer
   repair before calling it delivered.

The physical-storage restriction is unchanged. No registry version or secret
setting was changed. The published sketch dependency remains affected.
