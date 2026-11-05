# FASTQ 0.2 native delivery implementation plan

> Execute inline with one fresh review of the entire delivery change.
> Independent dependency-source inventory may run in parallel. The user has
> authorized direct work on owning main and unattended execution.

**Goal:** Qualify an installable FASTQ 0.2.0 carrier from exact clean source on
Linux and macOS, each on x86_64 and aarch64, without changing FASTQ behavior.

**Architecture:** Reuse the package/notice boundary proven by Table and BED in
one private control-plane script. FASTQ owns only its installed behavior check
and orchestration. Metagenomics is the next intended consumer once its report
policy is settled; no Rust foundation or Layer B dependency is introduced.

**Tech stack:** Rust 1.91, locked Cargo, Python 3.12 standard library, native GitHub
Actions, pinned fastp 1.3.6.

**Spec:** `AGENTS.md`, `docs/10-products/sequence-fastq.md`, and
`.autopilot/state/fastq-quality-execution-2026-11-05.md`.

## Constraints

- Owner starts at clean `a0796cae7a908cb3f6f18c323c9dc0c6d9421c6d`.
  No production, version, lock, benchmark, or publication-workflow changes.
- Bind package and bundle to the new exact commit and native Rust host. Use
  the established external-storage guard locally and explicit runner scratch
  for hosted outputs. Retain allowlisted evidence, not all runner scratch.
- Use the existing real fastp profile and installed lifecycle. Do not rerun
  representative timing or re-review unchanged algorithms.
- All files in the source package must be exact Git bodies/modes, except
  Cargo's normalized manifest and two generated members. Compare the
  normalized Cargo metadata and VCS SHA independently.
- Retain enabled normal/build dependency license originals from exact locked
  registry sources, including vendored C notices. Distinguish this graph from
  an inferred inventory of machine-code symbols.

## Task 1: shared package and carrier proof

- [x] Add `scripts/test_native_delivery.py`: real Git and source-archive
  fixture accepts an exact package and rejects one altered source body before
  extraction. Observe the absent verifier failure first.
- [x] Add `scripts/native_delivery.py`. Its `source` mode takes owner, commit,
  Rust host and output root; uses the current Cargo package from its resolved
  target; extracts to a fresh directory under the output root; verifies metadata
  and retains the enabled dependency graph/notice originals. Its `bundle` mode
  consumes the source receipt, installed binary and successful lifecycle
  receipt; checks their binary identity, native format/arch and records
  checksums. Print the extracted package directory for the workflow to use.
- [x] Run the new test and full control-plane checks. Verify against a real
  clean FASTQ source package and native installed binary before relying on CI.
  Commit the control-plane script; push and qualify exact Control plane CI
  (run41976325830 at `f580be87`).

## Task 2: FASTQ installed CLI gate

- [x] In `scripts/verify_install.py`, retain the real SE/PE quality-window
  lifecycle from the prior packet, parameterizing binary, pinned fastp,
  version and output directory. Emit a success receipt only after full
  outputs, ordered workers, Phred64, gzip, JSON and rollback checks pass.
- [x] Extend `.github/workflows/ci.yml` on the existing four native classes:
  check real Rust host, retain debug/release/fastp gates, package clean source,
  install from verified extraction, execute the installed lifecycle, bundle
  that same binary, upload only delivery artifacts. Checkout the helper at an
  exact verified World commit, outside the owning tracked tree.
- [x] Replay source/installation/carrier locally on physical external storage.
  Verify no production diff and one integrated review of the whole delivery
  surface. Commit/push owner main, wait exact-head CI, and verify all four
  artifact identities/metadata. Run41976716505 passes at `ca3a6fb`.
  Original artifact adoption is separate from run or upload success.

## Delivery and review boundaries

Review wrong head or altered crate bodies, host mismatch, wrong installed
binary, missing registry/vendor license texts, and artifact allowlisting.
Source/tool identities and failed commands stay recoverable. Do not use an
earlier local package as exact-head evidence for a new commit.

After exact-head installed gates, adopt original archives and verify license,
build and package closure. Public release needs a usable authorized GitHub
API path and a fresh anonymous installation. crates.io is separately gated;
do not retry the unchanged invalid token or announce an unpublished version.

Local exact-head evidence and the independent review are recorded in
`.autopilot/state/fastq-native-delivery-2026-11-05.md`.
