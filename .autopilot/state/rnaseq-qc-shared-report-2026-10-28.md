# First private shared RNA-seq QC report candidate

## Current state

Green4 run40546148508 at world1224fbf18d7af1b4b76869fc6e41b1611d7bb43b
is terminal SUCCESS on Linux/macOS x86_64/aarch64. Every platform passes
format, strict Clippy, all8 debug/release tests and four complete source
comparisons. All16 native calls match the actual five-metric RSeQC outputs;
single and paired reports are each byte-identical across workers/platforms.
Original API/ZIP/log/source/binary/report evidence is retained and verified at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/rnaseq-qc-native-2026-10-28/run-40546148508/`.
This accepts the complete bounded synthetic semantic workflow, not general
depth-cap equivalence, representative performance or publication readiness.
Sources remain private, publish=false, with no public repo. The next real
input/performance plan is docs/plans/2026-10-28-rnaseq-qc-real-performance.md.
Below is the historical ledger, not an instruction to repeat bootstraps.
The owning source is committed locally as6fdda78 (initial main commit);
all17 non-documentation frozen files remain exactly green4. Only README and
the merge record add the measured native acceptance and pending release scope.

| Platform | Artifact | Original ZIP SHA-256 |
| --- | --- | --- |
| Linux x86_64 | 12751813613 | 3e8130c10199aa0828a63ba964709cef9810713facc0d9e1bd3751e96d967d11 |
| Linux aarch64 | 12752269619 | a70edc73c1094fa606f055956dcca10793e29d15dac8aa0600b51e7ed4bf2dca |
| macOS x86_64 | 12752793988 | 9f099a716d06664f475a1bc4a308247e01486baff7dd37b6070e7afcebc06d36 |
| macOS aarch64 | 12752008584 | 710414a29604c3c65d5e3d72a91770d84f6358fbe1660eb11f32f0e8d5bbcf21 |

The Linux x86_64 binary SHA is
ae968a1a490433086b6e50867a15b9797bea47e903faf032967b5a3b377fa840.
Report SHA: single d5dbd31436e5fab4d971a40217439104a070477a95c8d3010ec905c6d762512b;
paired d42c3128578f6380c06b1e1030140c6d2a88d79e243eac2aef1643a9c6409f74.
Source/lock identities remain the green4 archive/manifest and imported lock
listed below. Frozen manifests retain the candidate's earlier maturity text;
the exact-run evidence here supersedes that text, not the numerical limits.

Fresh2026-10-28 storage check: boot APFS76.65% occupied,53.30GiB free;
KIOXIA245GiB and HDD256GiB free. No local build/dependency install is needed;
all project source/data/scratch remains on external storage or hosted runners.

## Source and execution ledger

Full30-family goal remains active. Existing dossier/source inventory is not
repeated. The new external owning rsomics-rnaseq-qc directory now contains
real mapping, strandedness, distribution, coverage and TIN logic, shared
original-record BAM reading, existing pileup, mandatory help, transactional
JSON/TSV/manifest and focused commands. No public repository or registry
name is created; publish=false. Git is initialized on main with no commit.

One implementation worker and a differentiated annotation/distribution
worker return source ownership. Eight compact unit/workflow tests are
authored, not executed. One completed fresh whole-feature source review
has no actionable finding and explicitly does not judge missing native/
oracle/performance/platform evidence. External rustfmt/check passes.

Released dependencies: bamio0.8.4/common0.12.3/help0.4.0/pileup0.9.0. No new
foundation API or product/path dependency. Unused intervals is not added.
The canonical profile uses full CIGAR and half-open midpoint semantics;
explicit rseqc-5.0.5 retains M-only/soft-clip/=/X and zero-width query quirks.
TIN keeps source sample-list length/eligibility/eligible-zero membership;
the source query-restarted versus global depth8000 boundary is unproved.

Root comparator reparses original raw source outputs, including the missing
fixed-width mapping projection, without rewriting original evidence. It
checks all five complete metrics, manifest output hashes, original-record
count/one traversal and single/paired1/4-worker equality. One critical
mutation test rejects mapping omissions, changed coverage, false entropy-zero
TIN and excluded eligible-zero membership. Comparison targets the explicit
compatibility profile, not false canonical/RSeQC identity.

New hosted gate uses the existing generic snapshot and API/ZIP verification
helpers, not a competing artifact verifier. Four focused routing tests and
the full476-controller suite pass with two existing native skips. The actual
original RSeQC ZIP passes its offline production acquisition verifier,71
workflow files, exact receipt/head/manifest/witness. No native Rust call has
run. Freeze green1 and use Linux bootstrap first, then a fixed-lock fresh
source and all four native targets. Representative coordinated-report
CPU/RSS/I/O evidence and final release metadata remain required.
Green1 freezes18 regular source files: archive SHA
74bd3c028d6a8343cf2b5ab4ea75641118eef00e8d3cf10f9137e4672cdddb31,
manifestc69ada7b9ecc6b88591d3f7d1ddb27713e2a503d85b4d6ed295e5b141703ccf2.
The generic snapshot verifier passes. No generated lock is included yet.

Before the first native dispatch, root aligns the successful operation's
manifest status to complete; publish=false still describes product maturity.
Unexecuted green1 is preserved, not modified. Executable candidate source is
green2,18 files: archive3393ce593a0f12c776629240511112ddf8b570abfb63cc651b5e249b49c1d0db,
manifestea5362ee64d0fd75657433702836261fba9004e79dd9721f89ce0d7be1117441.
No algorithm or dependency changes; generic snapshot verification passes.
Controller40541903549 at worldc8472c59315a11238adf2c9e82e3e6e2e984aaa5
passes. Do not claim native compilation or source agreement from it.

Worlda2bd1398f49c9de47340953317c4cbc479421d05 controller40542330116 passes.
The first complete Linux bootstrap is dispatched as40542614508 at that
exact head with green2/bootstrap_dependencies=true. Generated dependencies,
compilation, all authored tests, strict Clippy and actual five-metric report
comparisons run together; results are pending. No public release occurs.

Bootstrap40542614508 is terminal FAILURE only at strict Clippy: two test-only
single-range Vec literals. Actual Rust compilation and all8 tests pass in
both debug/release. The default comparison step was skipped after the lint
failure, not a passing or failing numerical comparison. Both literals are
corrected with Vec::from arrays, without disabling a lint or changing logic.
The complete comparison now runs whenever metadata succeeds, even after an
independent lint/test failure. Four focused controller tests still apply.

Original artifact12751616253 is1970677 bytes, ZIP SHA
8e0352ac3912c833a2d9cad86e214bd22abbfbc9c69d27b8572fe7e6d0930a26;
API exact-head/size/digest, ZIP CRC and safe paths pass. Original run/API/logs/
ZIP/extraction are under rsomics-fixtures/evidence/rnaseq-qc-native-2026-10-28/
run-40542614508/. Import Cargo.lock SHA
c09dea6b5397ca673d7052042d403fa55d2663e45c99672dc6f5ea6f51f308e2.
New green3 freezes19 regular files: archive
aa98668517064301e709ac127ac4a9a6c738deb5fe58aea8443164bf17c34152,
manifest7d58990a5471408748b6c4826292703ee289b44efb3d3f0ff9e862c9c107f27c.
Normal four-native mode, not another lock bootstrap, is next. No actual
five-metric native upstream acceptance or representative speed is claimed.

Green3 normal run40544150761 uses world79605bf198f5ca5622e67796c0167e3a05a159e4.
Its completed Linux x86_64/aarch64 jobs pass all8 tests per debug/release and
all four real single/paired1/4-worker five-metric report comparisons, including
transactional output hashes, original-record counts and within-profile exact
worker equality. Test-only lint still fails: Clippy1.91 checks array literals
as well as vec macros, so Vec::from([range]) was not a correction. Root reads
the actual pinned lint source and replaces both with once(range).collect(),
without suppression or any production change. Green4 freezes19 files:
archivea451612a20af21f1d73e15fadb60f039f18ef9c613a16a1652ba3f7c9177ddb0,
manifesta7449cb79fc889a3d2fcc4fd6fb8a56813d2c21a70d91481d2332af55f50a562.
Its generic snapshot verifier passes; final strict lint/native gate remains
required. No per-helper campaign, numerical relaxation or new review is added.

Mac APFS92.8%, KIOXIA245GiB free. No local Cargo/build/install/download;
project sources, snapshots, fixtures and scratch remain external. Preserve
unrelated inherited world VCF and root scratch changes.
