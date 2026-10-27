# First private shared RNA-seq QC report candidate

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

Mac APFS92.8%, KIOXIA245GiB free. No local Cargo/build/install/download;
project sources, snapshots, fixtures and scratch remain external. Preserve
unrelated inherited world VCF and root scratch changes.
