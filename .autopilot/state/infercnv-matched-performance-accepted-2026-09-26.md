# inferCNV matched prepared-input performance — accepted bounded result

## Decision and scope

The current matched-measurement plan is complete and its bounded result is
accepted. On the fixed shipped full example and one native Linux x86_64 GitHub
runner, Rust prepared-input processing through inferCNV step 14 had a strict
measured region-wall advantage: every one of seven Rust samples was faster
than every corresponding infercnv sample. This is **not** a claim about full
inferCNV, input ingestion, downstream clustering/HMM/Bayes, large-cohort
scaling, biological validity, or a release. The whole inferCNV workflow remains
unfinished. The user requested a pause “after finish the infercnv” and manual
resumption next Saturday. The controller conservatively chose and communicated
a pause at this completed bounded measurement checkpoint; the broader
completion-scope clarification was unanswered, so this narrower stopping
boundary is **not** recorded as user-confirmed. No new implementation batch or
automatic resumption is authorized.

World measurement head `d1ba88173b5bc9fd4a591b8ee4dd7a370afdd67b`
passed exact-head Control plane run `36229421479` (108 tests and architecture
validation). The measured product is
`3715e55b8c06c4bb6b1605f42942805da29a4de7`; production bytes are
unchanged from `bdcc8a3a55596be8d46a774d39b0335845c75175`. No new
public crate, API, CLI, repository, or publication was created, and the private
benchmark did not warrant a Layer A promotion.

## Original evidence and independent acceptance

The actual oracle/measurement workflow was `omics-rust/rsomics-world` run
`36229683578`, attempt 1, oracle job `108370358299`; all substantive steps,
including installed-package R preparation contracts, smoke, seven measured
pairs, source-after verification, and artifact upload, succeeded. Artifact
`infercnv-oracle-36229683578-1` has ID `10902423116`. The preserved original
artifact ZIP is 474,945,070 bytes with SHA-256
`85d3998c9a5fc0a15b7190e2a86cc71477553a46ef43b416b1715f0c94ec7ff8`;
original Actions logs SHA-256 is
`e10614314f1be9e4fab83aa47933e2c76b765129e55c530c7809b7b0925f068f`.
All 709 archive entries were independently checked and extracted bytewise.
The original-evidence acquisition report is a **local-only, non-Git** file at
`.superpowers/sdd/2026-09-26-infercnv-matched-performance-plan/matched-acquisition-report.md`
(SHA-256 `d476ab749136dec31d5c7f6060ea214e3e67ca2a8cd4ba05a7801195918fa87f`).
It records the API response digests, 142-file source and lock checks, accepted
input receipts, installed-R evidence, and warning inventory. It is not assumed
to exist in a fresh clone. Earlier Cargo
resolver stderr remains in the preserved original Actions logs for run
`36223760183`, not in a standalone artifact file; historical snapshots were
not changed.

The controller then recomputed all 16 complete trials (one smoke pair excluded
from statistics, followed by seven alternating measured pairs), all
25,047,552 final numeric entries, exact identities and hashes, raw metrics,
GNU-time records, order, and summary values. Each trial verifies 1,565,472
entries. Across final-stage Rust results, the largest absolute error was
`1.9984014443252818e-15` and largest error divided by the unchanged
`1e-12 + 1e-12*abs(expected)` bound was `0.0008477393156599713`.
The R oracle outputs matched their accepted pin. The individual
preparation/region/lifetime CPU, wall and RSS counters, numerical-error
summaries, and stdout/stderr hashes are retained in
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-matched-performance-2026-09-26/run-36229683578/controller-raw-audit.json`
(SHA-256 `90a532d4de5dbb7e93d773a514d69508ee4fc5b53112caabcac5937ffab42431`).
The independent audit script in the same run root has SHA-256
`00dca5fc50f9e80ce1790dc0be225e6dfc625c1e5f107b3d123e5403d404653e`.
Original per-trial expression/identity outputs and GNU-time files are under
that run root's `artifact/matched-results/`; each trial's output-file hash map
is in its `artifact/matched-results/*/validated.json`. The accepted result
does not rest on `summary.json` alone.

## Measured result and provenance

The fixed prepared input has 9,939 genes by 184 cells before processing and
8,508 genes by 184 cells at step 14. The host was an Ubuntu 24.04.5 Linux
x86_64 GitHub runner with an AMD EPYC 9V74 processor and four visible logical
CPUs. The comparison used R 4.6.1, infercnv 1.28.0, Bioconductor 3.23,
OpenBLAS 0.3.26, and a locked Rust 1.91.0 release build. The executable
SHA-256 was `2c08c3e85c31146b79086fe45150ed96c9684f1f3ddd58cc485eb72eee63d137`.
The fixed infercnv call used cutoff 1, minimum three cells per gene, ordered
reference bounds, smoothing window 101, clamp 3, samples mode, and stopped at
step 14; HMM, scaling, denoising, plots, and downstream outputs were disabled.
Full arguments and package/session details are in each original R provenance
file. Trials ran sequentially as fresh processes on that one host, each with
an untimed warm-up and a measured call from the original prepared input.

| Metric, seven measured samples only | Rust | infercnv |
| --- | ---: | ---: |
| Region wall median (range), s | 0.349404364 (0.339628661–0.362700170) | 14.456 (14.041–15.248) |
| Whole-process wall median (range), s | 1.23 (1.21–1.25) | 46.76 (45.20–48.07) |
| Whole-process peak RSS median (range), bytes | 94,973,952 (94,842,880–95,109,120) | 1,240,866,816 (1,240,637,440–1,242,521,600) |

The infercnv/Rust ratio of region-wall medians is
`41.37326687768559`. Whole-process peak RSS medians are approximately
90.57421875 MiB and 1183.3828125 MiB, respectively. Baseline current RSS
was recorded separately; subtracting it from a lifetime high-water mark
would **not** measure kernel allocation. Whole-process GNU time includes
startup, preparation, warm-up, validation/export, and exit, while region time
excludes that work. The region comparison is not an ingestion or end-to-end
workflow comparison.

## Reporting qualifications and remaining work

All five native-library thread-limit environment variables and infercnv
`num_threads` were configured to `1`, but every one of eight R provenance
samples reported `Threads: 2` **before warm-up**. No post-warm or post-call
thread observation was recorded. An endpoint OS-thread count cannot identify
active workers or prove continuous effective single-thread execution. The
result is a configured-single-thread comparison, not verified one-OS-thread
operation. R's `proc.time()` precision/backend is not asserted equivalent to
Rust `Instant`.

Preparation durations are retained in the raw audit but are **not** a
like-for-like speed comparison. The R preparation timer starts after package
loading, helper sourcing and result-directory creation; Rust starts at trial
entry, before argument handling and result-directory creation. No
preparation-speed ratio is claimed. Whole-process GNU time includes these
lifecycle costs. These two reporting qualifications resolve the nonblocking
Minor findings of the whole-change review, a **local-only, non-Git** file at
`.superpowers/sdd/2026-09-26-infercnv-matched-performance-plan/final-review.md`
(SHA-256 `d5f546ccd1dd48dfb5617706150f02412e217f7e31ecf72c032e79cb89662f47`).
The local review is not assumed to exist in a fresh clone; neither the
benchmark nor immutable evidence changed.

Future work, only after the user's manual resume, includes the separate
[ingestion source audit](infercnv-ingestion-source-audit-2026-09-26.md) and
[downstream source audit](infercnv-downstream-audit-2026-09-26.md), with their
own implementation, oracle, scaling, and release gates. Neither audit is
accepted as an executed full-workflow result here.
