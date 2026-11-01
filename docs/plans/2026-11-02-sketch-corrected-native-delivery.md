# Corrected Sketch native delivery implementation plan

> For agentic workers: execute one coupled task with one fresh whole review;
> root alone commits, pushes, dispatches measurements and publishes.

**Goal:** Make the already-repaired persistent-sketch workflow usable without
waiting for registry authentication, adopting the real short-input fix.

**Architecture:** Keep the four-operation product and all Rust algorithms/API
unchanged. Pin the already-qualified Layer A kmer source directly. Deliver
exact Git-preserving SOURCE.tar plus native binaries, not a normalized crate.

**Tech stack:** Rust1.91, locked Cargo/Git, stdlib Python, sourmash4.9.4 and
four native GitHub Actions classes.

**Spec:** World AGENTS.md; metagenomics-sketch.md's initial Sketch slice;
sketch-contract-review-2026-09-09.md; kmer-short-window-2026-09-09.md. Clean
owners: Sketchf430522bb3d3c6fd38e08af758dca11e5262f47b,
Kmerc79111f31651bad581920b7b5c2ddde9a57534e1,
Seqd9734e51c4ed557f6d8790d97a686717ebc4769e.
The fresh public registry index still has only kmer through0.2.2; no0.2.3.

## Global constraints

- Edit only /Volumes/KIOXIA/Documents/omics-rust/rsomics-sketch. Freeze its
  actual source/body/mode baseline first. All src/, Rust tests, fixtures,
  benches and upstream attribution bodies stay unchanged. Do not edit kmer
  or seq. Seq's KmerCounts path is unaffected; no synthetic Seq release.
- Cargo.toml changes only version0.1.1, publish=false and rsomics-kmer to
  version0.2.3/git https://github.com/omics-rust/rsomics-kmer/rev
  c79111f31651bad581920b7b5c2ddde9a57534e1. Lock changes only self/kmer source
  and version; retain every unrelated package/checksum. No path dependency,
  copied foundation kernel, new public item/format/command or API policy.
- Local Cargo/Rustup use /Volumes/KIOXIA/Developments/cargo-home and
  rustup-home; target is cargo-target/rsomics-sketch; TMP/cache/scratch stay
  under /Volumes/KIOXIA/Developments/tmp. Verify physical external devices,
  tool/cache paths and resolved target; cap4GiB incremental plus8GiB independent
  headroom. No boot-total stop, shared-data cleanup or local source-oracle run.
- Bootstrap the future Git dependency through an external copy of this real
  product's manifest/lock under the verified baseline guard before adopting
  its exact delta in the owner; do not weaken path checks to fetch/build.
- Old red/green runs, performance/control variants and raw records remain
  immutable. No registry token read/retry, publication attempt, PR/worktree,
  deletion, reminder or new speculative shared release framework.

## Review focus

1. Short/empty sequence construction now succeeds through the actual pinned
   foundation; a passing expected-panic diagnostic is not qualification.
2. Scale93/high-scale round trips, global collection downsampling and pair-local
   search keep their already-approved contracts; no new tolerance or format.
3. Exact SOURCE.tar preserves Git bodies/modes and Git dependency identity;
   installation is online/reproducible, not a registry/offline-vendor claim.
4. Installed four-command workflows reload their signatures and preserve
   profile/label/value/error contracts, including pre-commit output protection.
5. Current-head resource measurements bind actual normal-build binaries,
   complete inputs, source, options and native hosts; slower throughput stays
   visible and no guard-specific or universal speed claim is invented.

## One coupled implementation

**Files:** Cargo.toml/Cargo.lock, README.md/ATTRIBUTION.md as necessary;
.github/workflows/ci.yml and benchmark.yml; .github/benchmarks/measure.py and
test_measure.py; new scripts/guarded-cargo.sh and product-local delivery.py;
original dependency/Git-foundation terms/notices/provenance. Root owns World.

**Consumes:** exact existing four-operation code/oracles, the accepted c791
foundation and RNA-seq QC's Git-preserving source/native delivery pattern.
**Produces:** real dependency adoption, normal native CI/source/install
artifacts, complete current-product measurement workflow and honest docs.

- [x] Freeze the clean baseline; verify source-pin public accessibility and
  exact resolved c791/version0.2.3. Stage resolution externally under the
  baseline guard, inspect the full lock delta, then adopt it. Keep all other
  dependencies and every protected Rust body exact. Set publish=false.
- [x] Replace the old expected-panic CI branch with the complete existing
  four-test live sourmash oracle, debug and release, on native Linux/macOS
  x86_64/aarch64. Retain fmt/strictClippy/rustdoc/Rust tests and recorder tests.
  Prepare Cargo/Rustup/target/TMP/pip caches under RUNNER_TEMP before setup.
  Actual hosts, source Git identities, oracle version and install provenance
  must be recorded. No skipped short-sequence or expected failure remains.
- [x] Reuse the exact SOURCE.tar approach, retaining the complete Git files,
  Cargo manifest/lock and verified source manifest. Do not cargo-package away
  the Git dependency or label a tarball a registry crate. Test/install from
  the source archive; run the complete oracle against that installed binary.
  Add one connected installed four-command lifecycle including short-input
  construction, reload/inspect, comparison and search, plus a pre-commit
  failure preserving existing output. Use existing independent expectations;
  keep all actual argv/status/stdout/stderr/output/binary identities.
- [x] Bundle one0755 native executable, original terms/docs/BUILD/checksums,
  directories0755 and metadata0644. Derive enabled target graphs from actual
  locked normal/build metadata; Git kmer terms bind exactc791, not a registry
  checksum. Retain all applicable nested terms without normalization. Upload
  source/native candidates/raw CI outcomes always for90days. No legal,
  signing/older-OS/system-runtime clearance or public-release claim.
- [x] Add a product profile to the existing measurement recorder: explicit
  --profile product selects only candidate+sourmash with alternating order;
  existing default four-tool protocol/tests stay intact for historical use.
  Require both real executables; no duplicated fake baseline/reference roles.
  One connected recorder test verifies two roles, balanced positions, exact
  measured row counts and complete output-byte comparison. Run the existing
  recorder suite once; no per-function test/review campaign.
- [x] Replace active benchmark.yml's obsolete three-kmer build selectors with
  one current-head normal release build, archived before test-feature builds.
  Record exact Cargo/Git/source/lock/compiler/native host/compiled external
  features and binary hash. Full oracle checks the archived candidate. Measure
  candidate+sourmash4.9.4 on both COMPLETE existing inputs/four native classes:
  E.coli FASTA4699745B/SHA53bb6a51b6e92139ced1e38f74b7938781027c52200922ff03718c2237d23bb4,
  SRR341550_1.fastq.gz87439836B/SHAd7a15c1762d64a5434ced0cc665d7f5d167ca81a71e239f8237b9cd490dd7683.
  Keep k31/scaled1000/seed42, abundance only for FASTQ,4warmups/16 genome
  measured rounds and1warmup/8 FASTQ rounds, native time/RSS and one CLI process.
  Every output must byte-match; preserve all raw trials/inputs hashes/logs.
  Linux actual one-CPU affinity is recorded; no exclusive/cold-host claim.
  Gate remains strict lower candidate process RSS in every measured pair in
  all8 jobs. Report all time/CPU/RSS distributions, including throughput losses;
  do not infer compare/search performance or a guard-causal improvement.
- [x] README distinguishes corrected candidate from available registry0.1.0,
  GitHub source/native delivery from a crate, and old performance from the
  pending final-head gate. Inspect actual locked output code and narrow any
  inherited post-commit rollback promise rather than changing common policy.
  Locally run guarded Rust checks and own installed lifecycle only; no Mac
  sourmash/scientific runtime. Freeze complete candidate diff/protected bodies,
  notices and raw local outcomes for root's single whole review.

## Root gates and rulings

Root resolves material review findings in one pass, commits only the concern,
then waits for exact-head native-four/source/install/full-oracle acceptance.
Root alone dispatches the current-head eight-job construction gate, retains
originals and decides from actual results. Only after both gates and original
adoption may GitHubv0.1.1 and fresh public installation be earned. Registry and
foundation publication stay held; no old measured binary is relabelled.

Ruling: the old registry-first wait is replaced only for GitHub delivery by a
full immutable Git pin plus SOURCE.tar, an already-used repository pattern.
No public model changes. Cost if wrong: source delivery is held by actual
native install rather than silently substituting registry0.2.2.
Ruling: retire the guard-attribution/control experiment from this final product
gate; use two real roles and retain every old result. Cost if wrong: no causal
guard/regression-isolation claim, only current-product/source resource evidence.
Ruling: unattended execution and one integrated review are already delegated;
there is no new human checkpoint or helper test/review ladder.
Ruling: the whole review established seqio's separate gzip decoder thread.
Environment caps do not bound total threads; Linux alone confines them to one
CPU. Correct the protocol and deliver it inside native archives without changing
the experiment or numeric gate. Cost: no equivalent one-thread/one-CPU Mac claim.
