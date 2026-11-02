# Count native/source delivery

## Existing product and evidence

Owner `/Volumes/KIOXIA/Documents/omics-rust/rsomics-count` is clean at
5bc764d8af75171c4ac2e4a9e4a153d0e2d05b49; root freshly verifies public main
agrees and inspects README, Cargo, CLI, live-oracle test and native CI. There
is no owning AGENTS.md. The current version is0.1.0; no algorithm change or
version bump is needed merely to provide its first GitHub binary release.

Counting is the default command, with `--summary` paired output, not a separate
summarize command. `normalize` consumes the product's own featureCounts-style
gene matrix and exposes TPM, FPKM and FPKM-UQ. It already uses common/help/bamio/
intervals. Annotation/grouping/count/normalization policy remains private.
The dossier is docs/03-transcriptomics/quantification.md and the released
contract is docs/10-products/count-gate-2026-08-02.md.

Historical native4 CI30739798218 is green. Three hosts have actual pinned
featureCounts2.1.1 differential runs; Linux ARM has full frozen compatibility,
not a live upstream binary. Keep that distinction. Historical registry install
is accepted; current registry availability is not inferred from old records.
Fresh public GitHub release API is empty at the frontier inspection.

Root confirms `git diff3a0154b1..HEAD` is empty for src, Cargo.toml and Cargo.lock.
The two later commits only add evidence/CI. The historical2M-read/1M-fragment
resource gate applies to implementation3a0154b1: wall1.67–2.56x source in the
declared cases, roughly87% lower RSS in the two reported four-worker cases.
This justifies unchanged-code delivery, not timing inheritance for new binaries
or a universal speed claim. Do not replay unchanged benchmarks for packaging.

## One coherent implementation

One worker owns only this flat owner. Root owns World state, commits/main,
pushes, the one integrated whole-feature review, original adoption and any
public release. No PR, worktree, new repository/crate or unrelated edits.

- Add a physical external-path guard before local Cargo, with dedicated Count
  target/cache paths and8GiB independent free-space reserve. No project writes
  on the boot disk, no aggregate boot-usage stop, no false OS-footprint claim.
- Preserve every src/test/Cargo.lock body and their Git executable modes.
  Cargo metadata changes, if necessary for packaging, must be narrow and
  explicitly reported. Keep dependencies and existing science/API unchanged.
- Reuse the proven Table/Ecology/BED private delivery pattern, not a speculative
  public packaging framework or changes to other products. Produce clean
  Cargo source and four native Linux/macOS x86_64/aarch64 binary carriers.
- Bind exact Git source, Cargo.lock, native Rust1.91/compiler host, actual
  normal/build dependency graph, source package, installed binary and checksums.
  Preflight full metadata online with locked dependencies where necessary;
  offline no-deps/tree queries must not assume foreign-host caches exist.
- Retain the complete original shipping dependency/license texts, including
  registry foundations and vendored native dependency terms. Exclude dev-only
  packages from shipping claims. Do not claim legal, minimum-OS, DSO, signing
  or notarization clearance from source metadata.
- Keep existing four-native debug/release/frozen tests and three pinned live
  oracle routes. Linux ARM stays explicitly frozen-only; do not silently claim
  a fourth live oracle or widen into a new upstream source-build project.
- Package/rebuild/install from the clean source and run one connected installed
  lifecycle: multi-library count with full matrix/summary, own-output normalize
  in all three modes, shared help/version/JSON, representative stdin or gzip
  input, and failure preserving existing paired outputs. Use known fixtures/
  golden contracts and numerical printing, not invented behavior or a separate
  test for every helper. Default counting plus normalize is the public scope.
- Local checks use only own compiled product and existing frozen fixtures.
  Keep FEATURECOUNTS unset locally: its current test returns without executing
  when unset, which is not a live pass. Actual three live jobs remain hosted.
- Correct README's actual distribution/installation state. Keep historical
  wall/CPU/RSS losses and scientific exclusions visible. Source comments stay
  rare; user/API help is contract documentation, not narration.
- Retain full native/source/install/notices logs and artifacts on success or
  failure. No cargo publication, secret read, live registry mutation or other
  product release is part of the worker task.

## Verification and integration

Use guarded formatting, strict all-target Clippy, existing relevant debug/release
and package checks, and one real installed lifecycle. Report true ignored or
environment-inert oracle tests separately. Protect the starting source/lock/
tests; no per-helper proof ladder, nested reviewer or new benchmark campaign.

Root reads the whole frozen implementation and gets one integrated review of
source/package/public interfaces, license closure, guard, lifecycle and native
workflow. Address material findings in one correction pass, with explicit
rulings on any declined scope. Root then commits and pushes owning main and
waits exact-head native4 CI, recovers/adopts complete originals, creates the
first real GitHub0.1.0 native release and checks a fresh public Mac installation
of the same count-to-all-normalizations workflow. Do not relabel historical
installed or measured executables as this new binary. Registry publication is
not retried; existing registry and GitHub distributions stay distinct.
