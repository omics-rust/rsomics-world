# K-mer consumer and boundary recheck

Status: candidate 0.2.3 passes exact-head Linux x86_64 package verification,
native tests and both consumers on all four platforms. All eight measurement bundles are
verified and retained. The narrow foundation correctness patch is approved
for publication using the resource-use decision below, with slower workloads
explicitly retained. This is not a speed-optimization claim. Microbenchmark
attribution remains inconclusive; the rejected inlining hint stays withdrawn.
Publication attempt `34274791181` failed registry authentication after package
verification; no corrected registry release or delivered sketch repair exists.
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
passed all eight jobs, including artifact upload; this closed that candidate's
consumer gate. The first run was not relabelled as green.

The current refresh at kmer
`1a0d9aac5307644fd6d9cdaab20c648bd9193984`, run
[34269918433](https://github.com/omics-rust/rsomics-kmer/actions/runs/34269918433),
also passed all eight jobs. It pins repaired sketch
`f430522bb3d3c6fd38e08af758dca11e5262f47b` and unchanged seq `d9734e5`.
Sketch's complete four-test sourmash suite executes in debug and release on
each native platform, including short-input and mixed-scale collection cases.
All eight ZIP digests, archive integrity, exact Git dependency identities in
metadata/lock/config and successful raw test logs were independently checked.
Ordinary exact-head CI
[34269915993](https://github.com/omics-rust/rsomics-kmer/actions/runs/34269915993)
passed as well. This workflow-only commit leaves production sources, tests,
benchmarks and Cargo files identical to `61de048`; it does not change the
dependency selected by the separate construction measurement.

The complete refresh evidence is retained at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/kmer-short-window-2026-09-09/consumers-34269918433/`,
recursively compared with its external scratch copy. This is source-head
consumer evidence, not a corrected registry publication or future release-head
gate.

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

### Real sketch construction refresh

Sketch commit `3802b1aaca54a95c14dbdeb6571aff4eb5ebafc3` adds a private
measurement recorder and a manual eight-job native workflow, not a production
optimization or a new public crate. Ordinary CI
[34261359503](https://github.com/omics-rust/rsomics-sketch/actions/runs/34261359503)
passed all four native test jobs plus lint, documentation, package and benchmark
smoke checks. Measurements
[34261423289](https://github.com/omics-rust/rsomics-sketch/actions/runs/34261423289)
completed all eight native jobs. The full evidence and scoped decision follow.

All three product controls use that exact sketch head, Rust 1.91.0 and matched
ordinary release-bin builds. Only kmer selection differs: registry 0.2.2,
corrected reference `23e42f3`, and candidate `61de048`. The latter two have the
same production source. The workflow asserts the registry checksum and pinned
Git revisions, compares resolved external packages and actual compiled feature
sets, and archives binary digests, manifests, lockfiles and Cargo messages.
The live sourmash 4.9.4 oracle explicitly tests the archived candidate before
timing, including the short-input regression; later test builds cannot replace
the measured file. This corrects a build-mode confound found during independent
review. No production source, public dependency requirement or API was changed.

The pinned complete inputs were rechecked locally and against live source
metadata:

| Input | Bytes | SHA-256 |
|---|---|---|
| NCBI ASM584v2 genomic FASTA | 4,699,745 | `53bb6a51b6e92139ced1e38f74b7938781027c52200922ff03718c2237d23bb4` |
| Source FASTA gzip | 1,379,902 | `a96d3cfa58c88d477013c768f90a11d2386d42a70d566abfb6d9013dcbd24255` |
| ENA SRR341550 read 1 gzip FASTQ | 87,439,836 | `d7a15c1762d64a5434ced0cc665d7f5d167ca81a71e239f8237b9cd490dd7683` |

The source URLs and full protocol live in sketch's
[measurement README](https://github.com/omics-rust/rsomics-sketch/blob/3802b1aaca54a95c14dbdeb6571aff4eb5ebafc3/.github/benchmarks/README.md).
A strict read-only scan verifies 6,282,141 complete 101-base FASTQ records
(634,496,241 bases), with no short reads in that benchmark input. This avoids
the known old baseline panic during timing without removing short-input tests.

Genome construction uses 4 warmup and 16 measured rounds; FASTQ abundance uses
1 warmup and 8 measured rounds. Every round executes all three product controls
and sourmash, with balanced positions and within-round predecessors. All commands
use DNA k31, scaled 1000, seed 42, one-worker settings, and identical absolute
input paths within the job. Every signature must match complete nonempty bytes.
Native time output, wall/user/system time, byte-normalized peak RSS, commands,
stderr/stdout, signatures and incremental observations are retained. A completion
marker requires all executions and byte comparisons to succeed; it does not
approve performance or publication.

Eight recorder tests passed locally, including subprocess failure, output
mismatch, no-overwrite, digest validation, explicit seed/profile and native RSS
normalization. Ruff, Rustfmt and YAML/Bash syntax checks passed. No local Rust
compilation or product timing was performed; physical boot APFS usage remains
94.6%, so native product validation runs in GitHub CI.

#### Genome results

All four genome jobs completed successfully, including the full three-test
live oracle on the archived corrected candidate and its post-test checksum.
Each retained artifact has 80 raw trials: 16 warmup observations and 64 measured
observations, forming 16 measured four-tool rounds. Independent verification
checked full output bytes, binary/lockfile digests, foundation revisions,
external dependency feature equality and native resource-unit normalization.

The following are medians of per-round candidate/other ratios, not ratios of
unpaired medians. Lower means faster or less peak memory. These results apply
only to the complete E. coli genome construction workload described above.

| Native runner | Time / published baseline | Time / same-source reference | Time / sourmash | Peak RSS / sourmash |
|---|---|---|---|---|
| Linux x86_64 | 0.769231 | 0.977528 | 1.110640 | 0.063158 |
| Linux aarch64 | 1.000000 | 1.000000 | 0.515002 | 0.058569 |
| macOS x86_64 | 0.884854 | 1.008772 | 1.095327 | 0.058394 |
| macOS aarch64 | 1.017518 | 1.000705 | 0.704724 | 0.062135 |

The current genome evidence supports a substantial peak-memory advantage on
all four targets, and a throughput advantage on the two ARM targets. It does
not support a universal throughput claim: the x86_64 candidate is approximately
11.1% slower than sourmash on Linux and 9.5% slower on macOS by paired medians.
Same-source control differences are recorded rather than attributed to the
guard. Genome evidence alone was not used to approve publication; the complete
FASTQ results and final decision are recorded below.

A separate read-only reviewer independently recomputed all four results,
including compiled feature sets from Cargo messages and all raw resource
values. Signatures contain the same 4,476 hashes and content MD5 across native
platforms; only their absolute input filename metadata differs. These are
single-job, single-host-per-platform observations, not machine replications
or a formal equivalence test of the corrected-source controls.

The four original ZIPs pass integrity checks and match the GitHub digests:

| Artifact ID | Native genome artifact | SHA-256 |
|---|---|---|
| `10070175121` | Linux x86_64 | `5d5cc037fdd168cba4c4c0a7b433962aa8680973e9417324ce31c1b6a882ef20` |
| `10070132041` | Linux aarch64 | `404fdc9b782928d60aad55ec3312b5507d53dab30e18378c1c955e22ce8dbb85` |
| `10070492835` | macOS x86_64 | `bf95054e2fb0436c87b754b39dcaca6c15b63ef1aa23b700c73fbdf63bdb5eb5` |
| `10070216489` | macOS aarch64 | `647b98bfd3f140843cb3a36c5885bce35a1f1c09a70c06ea50a5e8b3e6437c7a` |

They are preserved with extracted evidence, completed job logs and ordinary-CI
metadata under
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/kmer-short-window-2026-09-09/sketch-construction-34261423289/`.
The retained copy was recursively compared with external scratch at
`/Volumes/KIOXIA/Developments/tmp/sketch-construction-34261423289-mjkyFe/`.
All later FASTQ artifacts were added without overwriting these files.

#### ARM full FASTQ results

The Linux ARM and macOS ARM full gzip FASTQ abundance jobs completed
successfully. Each artifact contains 36 raw trials: four warmup observations
and 32 measured observations, forming eight four-tool rounds. Input SHA-256,
87,439,836 compressed bytes, k31, scaled 1000, seed 42 and abundance profile
match the pinned complete workload. All signature bytes match across variants
and repeats within each job. The archived candidate's full oracle and checksum
passed before timing.

| Native runner | Time / published baseline | Time / same-source reference | Time / sourmash | Peak RSS / sourmash |
|---|---|---|---|---|
| Linux aarch64 | 1.033869 | 1.000000 | 0.885291 | 0.082669 |
| macOS aarch64 | 1.014050 | 1.003284 | 1.360956 | 0.187212 |

These again are medians of eight within-round ratios. Candidate/reference
median peak-RSS ratios are 0.999576 on Linux ARM and 1.001642 on macOS ARM;
candidate/baseline ratios are 0.986625 and 1.035370 respectively. Candidate
elapsed-time differences versus the published baseline are reported, not
silently equated to zero or assigned a guard-specific cause.

The ARM FASTQ results support a substantial memory advantage on both runners
and a throughput advantage only on Linux ARM. macOS ARM takes approximately
36.1% longer than sourmash by paired median; its peak RSS is approximately
81.3% lower. Historical August FASTQ ratios are not substituted for this result.
At initial ARM collection both x86_64 jobs were live. Both subsequently
completed as recorded below; the partial ARM result was not treated as the
full four-native workload gate.

| Artifact ID | Native FASTQ artifact | SHA-256 |
|---|---|---|
| `10071134303` | Linux aarch64 | `1a34e4e0a37702da1d025bf724e0260cf2e540d1f4c5b93810d6cacd69024027` |
| `10071183821` | macOS aarch64 | `5f3c219c9084287712b1506a65630a644dad1033cdd12ff9b1a1add2c4600eff` |

Both ZIPs match GitHub digests and pass integrity checks. Binary and lockfile
hashes, exact dependencies, actual compiled external features, balanced unique
round/tool observations and all raw wall/user/system/RSS values were checked.
These artifacts and completed job logs are now preserved beside the genome
evidence at the permanent path above, with recursive scratch/copy comparison.

Independent review reproduced all ratios and every raw resource value. The
two platforms retain the same 40,425 hashes and abundances with signature MD5
`e7bb44c73d20b4063b654c04210a4f11`; cross-platform JSON differs only in the
absolute input filename. Linux ARM is slower than the registry-kmer baseline
in all eight measured pairs (3.16–3.77% longer elapsed time, median 3.39%).
That observed regression remains part of the eventual release decision;
closeness to the same-source corrected reference does not erase it.

#### Linux x86_64 full FASTQ result

Job `102180058482` completed with all three archived-candidate oracle tests
and binary/lockfile checksum checks passing. Artifact `10072798104` has SHA-256
`29e68c60307ab1af1e8ae680ec3eb724407f19f8b326071ff25cf1598a306020`.
The ZIP digest and integrity, all 36 raw trials, eight complete measured rounds,
balanced positions, input/parameter provenance, full output equality, dependency
identities, actual compiled features and ordinary release profiles were verified.

| Control | Median paired candidate time / control | Median paired candidate RSS / control |
|---|---|---|
| Registry kmer baseline | 0.699100 | 0.997876 |
| Same-source corrected reference | 0.966135 | 0.999808 |
| sourmash 4.9.4 | 1.864916 | 0.090191 |

On this AMD EPYC 7763 runner, candidate elapsed time is higher than sourmash
in all eight pairs: 86.49% higher by paired median, despite 90.98% lower peak
RSS. Candidate/reference ratios range from 0.935437 to 0.994874; equal sources
still have persistent timing differences, so the baseline improvement is not
assigned specifically to the guard. The same 40,425 hashes and abundances match
both ARM artifacts after removing only the absolute filename.

An independent reviewer reproduced every value and validated the provenance.
The ZIP, extracted evidence, job log and updated artifact metadata were copied
to the permanent evidence directory above and recursively compared with scratch.
These measurements describe product head `3802b1a`, not the subsequent
product-local repairs.

#### Intel macOS full FASTQ result and complete-run decision

Final job `102180058887` passed at the same pinned product head. Artifact
`10074526630` has SHA-256
`8f1ff839c0062660ce6190445e60502afd6bd266641f76147e187eca30b18cf0`.
Its ZIP/API digest, CRC, extracted bytes, all 36 raw observations, commands,
resource values, full signatures and three archived-candidate oracle passes
with post-test checksum were independently verified. The final artifact,
completed run metadata and raw log are permanently retained alongside the
other seven bundles; recursive comparison with scratch passed.

| Control | Median paired candidate time / control | Median paired candidate RSS / control |
|---|---|---|
| Registry kmer baseline | 0.843520 | 1.000000 |
| Same-source corrected reference | 1.004950 | 1.000227 |
| sourmash 4.9.4 | 1.815349 | 0.096582 |

On the Intel Core i7-8700B runner, candidate elapsed time exceeds sourmash in
all eight pairs: 81.53% longer by paired median, with 90.34% lower peak RSS.
Candidate/reference time ranges from 0.935102 to 1.070288; this is not evidence
of a guard-specific optimization. Signature content matches the other native
FASTQ artifacts after removing only the absolute input filename.

The completed run contains 464 trials, including 384 measured observations.
All eight archive digests, 24 product binary/lockfile checksum sets, dependency
identities, reconstructed compiled features and complete output bytes passed
both primary and independent review. Every measured candidate/sourmash pair
uses less peak memory. Paired median RSS reductions are 93.68–94.16% for genome
construction and 81.28–91.73% for FASTQ abundance. Throughput is mixed, with
the slower paths explicitly listed above. Each cell is one warm-cache runner
job, not an independent machine replication or general workload guarantee.

Decision: accept this evidence for the narrow `rsomics-kmer` correctness patch
under the architecture's strict throughput **or resource-use** gate. The patch
repairs a deterministic valid-short-input panic without changing hashes,
public API, iterator state or allocation policy. Accept the observed Linux ARM
FASTQ median 3.39% elapsed-time regression against the registry-kmer baseline
as an explicit correctness tradeoff; do not call it zero or assign its cause.
The failed microbenchmark null controls and slower sourmash comparisons remain
part of the record. No throughput/no-regression claim follows this decision.

Measured foundation `61de048` and release candidate `c79111f` have identical
production sources, tests and benchmarks. Their metadata/build identities
remain distinct: these are supporting consumer hot-path measurements, not a
claim that the 0.2.3 registry artifact was timed. Candidate-specific package,
API and consumer checks below complete the foundation gate. The newer sketch
`f430522` changed loading and comparison after the measured `3802b1a`; it still
requires its own final-head validation and measurements before product release.

### Fresh API and publication preflight

A fresh independent source review of kmer `61de048`, sketch `3802b1a` and seq
`d9734e5` found no correctness or public-API blocker for the foundation patch.
The production diff from registry baseline `d89e2df` is exactly the short-input
guard; constructors, exports, iterator layout, hash body, allocations and error
types are unchanged. The existing default boundary tests cover exhaustion and
reuse, and the real sketch caller handles allocation errors. This is a
patch-compatibility review, not a performance or publication approval.

The preflight listed `CARGO_REGISTRY_TOKEN` for 17 selected repositories,
excluding kmer and sketch. Both repositories are active `omics-rust` main-branch
repositories with administration available to the current GitHub account.
At that preflight no value was read, access list changed or publish dispatched.
The last successful seq/VCF publication runs predate its August 20 metadata
timestamp, which does not establish token rotation or current validity. Credential availability
was still unverified; the actual publication attempt below now establishes
that the stored credential fails registry authentication.

The separate [product contract recheck](../10-products/sketch-contract-review-2026-09-09.md)
found and repaired scale recovery and collection comparison defects after the
foundation review. Product head `f430522` passes exact-head four-native CI, but
still depends on registry kmer 0.2.2 and still reproduces its short-input panic.
Neither those green checks nor the earlier construction measurements establish
delivery of the corrected dependency.

## Release-candidate gate and remaining delivery

The unpublished 0.2.3 candidate is now
`c79111f31651bad581920b7b5c2ddde9a57534e1`. Only the root version in the
manifest and lockfile changes from `1a0d9aa`; parsed dependency metadata,
production source tree, tests and benchmarks remain unchanged. Fresh independent
API/source review confirmed that the sole production delta from the published
baseline is still the short-input guard, with no inlining experiment restored.
Normal exact-head CI
[34271886882](https://github.com/omics-rust/rsomics-kmer/actions/runs/34271886882)
passed all four native targets and lint/package/benchmark smoke. Candidate
consumer validation
[34272105400](https://github.com/omics-rust/rsomics-kmer/actions/runs/34272105400)
passed all eight jobs. Every retained metadata graph, lockfile and Cargo
configuration selects exactly one kmer 0.2.3 at `c79111f`. Raw logs confirm
the pinned product checkouts and native hosts; all four sketch oracle tests,
six live SeqKit contracts and the independent sequence kmer oracle execute in
both profiles. Ordinary CI logs also verify strict lint, package reconstruction
and benchmark smoke. Independent review found no remaining source/API/metadata
or correctness-evidence blocker.

The eight original archives, extracted metadata/configuration/lockfiles, exact
CI records and all 13 ordinary/consumer job logs are retained at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/kmer-short-window-2026-09-09/consumers-34272105400/`.
Every archive matches its GitHub SHA-256 digest and size, passes CRC, and
matches the extracted files. The permanent copy matches external scratch
`/Volumes/KIOXIA/Developments/tmp/kmer-consumers-34272105400-pix9ml/`.
This preparation is not publication and does not change measured identities.

### Publication attempt and credential barrier

World release decision `3be29dd4886040edc1661f25031fe0d0241af1b3` passed
exact-head control CI `34274695274`. The kmer remote main still matched
`c79111f`. Repository ID `1245819199` was added individually to the existing
organization secret's selected list. Before/after API records verify exactly
18 repositories: all 17 existing entries plus kmer, with visibility still
`selected`. No secret value was read or changed; sketch remains excluded.

The reviewed main-only workflow was dispatched once as
[34274791181](https://github.com/omics-rust/rsomics-kmer/actions/runs/34274791181).
Its exact-head package construction and verification passed, then uploading
0.2.3 failed with HTTP 403, `authentication failed`, exit 101. The secret was
present in the job environment (masked); this is no longer an unknown
repository-selection problem. It does not establish whether the credential
was revoked, expired or otherwise invalid for the request.

The sparse registry index was freshly read after the failed attempt: it still
ends at non-yanked 0.2.2 with the recorded checksum; no 0.2.3 exists. The raw
job log, exact run identity, selected-access before/after metadata and both
registry snapshots are preserved at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/kmer-short-window-2026-09-09/publication-34274791181/`,
recursively compared with
`/Volumes/KIOXIA/Developments/tmp/kmer-publication-20260909-2xSzvA/`.
No retry, credential extraction, broad access grant or fallback publication
was attempted. Registry adoption is stopped while independent work continues.

Remaining delivery:

1. Wait for a valid publication credential to replace the organization secret;
   do not request its value in chat or rerun the failed workflow unchanged.
   After a confirmed credential change, recheck main, registry and quality
   gates before one publication attempt. Verify exact head, registry checksum,
   archive contents and source provenance before claiming delivery.
2. Align sketch's minimum registry dependency and lockfile with the verified
   fixed release, remove its expected-failure diagnostic and run complete
   final-head native tests, oracles and representative measurements.
3. Publish the coherent sketch repair only after its own gates pass. A fixed
   foundation alone does not mean the product dependency has been delivered.

The physical-storage restriction is unchanged. No registry version was added;
only kmer's selected secret access was added. The published sketch dependency
remains affected.
