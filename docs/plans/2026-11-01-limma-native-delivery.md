# Limma native/source GitHub delivery implementation plan

> **For agentic workers:** Use superpowers:executing-plans for one integrated
> delivery change. Root reviews the whole result once, not per helper.

**Goal:** Deliver the accepted count and ordinary-expression CLI as four
native GitHub archives plus a clean source package, without changing science.

**Architecture:** Preserve the independent owning repository and existing
four-native CI. Extend its actual package/install route using the successful
DESeq/Popgen delivery pattern and one private helper only where it simplifies
the workflow. There is no new public library or packaging framework.

**Tech stack:** Rust1.91, locked Cargo, GitHub Actions, Python standard library
for packaging and installed CLI checks only.

**Spec:** AGENTS.md, docs/10-products/bulk-expression.md and accepted
limma-native-logexpression/limma-logexpression-performance-2026-10-31 states.
The user has delegated execution and release decisions; no repeated approval
checkpoint is added for this already accepted product slice.

## Fixed scope and constraints

Owner is clean f8c8f422ef8532701e3c70bbb019c72471b1a98d at
/Volumes/KIOXIA/Documents/omics-rust/rsomics-limma. Existing owning native-four
CI41207239669 and both scientific workflows are accepted. Preserve every
existing src/tests/Cargo.toml/Cargo.lock/core-license/merge-provenance/checker/
guard body and mode, including publish=false and stats0.2.4. Registry
authorization remains a separate hold. Do not read secrets or retry publish.

No R/upstream/foreign execution on Mac, new fit/source/real benchmark campaign,
dependency upgrade, extra public operation or API. Preserve README exclusions
and qualified performance: ordinary serial throughput/scaling do not strictly
separate; process RSS passes and four-worker throughput passes; outputs are
larger. Count results are separate, not a combined or universal speed claim.

Mac Cargo/rustup/target/TMP/cache/scratch must resolve physically to allowed
external disks. Use the owning guarded-cargo.sh; use its dedicated pinned
Clippy branch, not --exec cargo clippy. No cleanup or cache deletion. New task
scratch is under /Volumes/KIOXIA/Developments/tmp; no scientific bulk copies.

## Review focus

- Both existing user input routes actually work after package installation.
- The shared source package contains exact committed bodies and clean VCS data.
- Actual platform archives contain the native executable and full notices.
- Failed package/install commands leave raw diagnostics in uploaded artifacts.
- Metadata distinguishes historical performance from newly built binaries.

## One integrated owning change

Files: modify .github/workflows/ci.yml, README.md, THIRD_PARTY_NOTICES.md;
add LICENSES actual text/provenance and at most one scripts/delivery.py.
If convenient, add only tests/data/expression.tsv with the EXACT six-row
ordinary expression already embedded in tests/workflow.rs; existing tests
remain unchanged. Do not regenerate fixtures or accepted source evidence.

- [ ] Extend existing four actual native hosts, Rust1.91/host checks, debug/
  release tests and Linux fmt/strict Clippy/rustdoc; initialize all paths under
  RUNNER_TEMP before toolchain setup through GITHUB_ENV.
- [ ] Resolve the enabled locked normal/build target graphs for all four
  targets; retain real dependency license texts and source/copied hash
  provenance, explicit line-ending normalization and any team-owned SPDX-only
  gap supplementation. Reuse existing exact texts where identities match.
  Do not describe all listed packages as linked or claim legal certification.
- [ ] On every actual native host, create a clean locked Cargo source package,
  verify full source/body/mode/VCS/normalized metadata, run packaged release
  tests, install from that package and exercise its installed executable.
- [ ] Installed version/help then both run/results/export lifecycles use the
  existing count13x7 and exact ordinary6x7 fixtures at four workers. Check
  count13->11/schema1, ordinary6/schema2, named sample/features, full saved
  result reuse/no-refit, unchanged bundle bytes and complete TSV identities/
  shapes. Retain each command/status/stdout/stderr/output without another
  numerical oracle or worker-performance campaign.
- [ ] Produce one native TAR per actual target: exactly one0755 executable,
  README/project license/complete LICENSES/notices/BUILD at0644, dirs0755.
  BUILD binds full head/compiler/host/lock/source/native/run/attempt and does
  not relabel old measured ELFs. Upload actual native/source/checksums and
  package/install diagnostics with contents:read; release writes stay root-only.
- [ ] README gives neutral CI/Releases links and actual download/source install
  instructions, both fixture invocations, actual stable slice and registry
  distinction, without pending-science or universal performance claims.

Author performs one lightweight connected preparation of helper/fixture and
metadata/archive contracts, freezes complete before/after protected bodies,
diff/context/REPORT/seal externally, and does not commit/push/dispatch/release.
No helper-level review ladder or duplicate Rust campaign; root owns current
strict checks and one whole fresh review.

## Root adoption and publication

- [ ] Read whole change/author evidence and one whole delivery review; close
  any material findings together and verify all protected bodies unchanged.
- [ ] Run proportional local format/strict Clippy/tests and actual clean
  package check. Commit only this concern to owning main, push and wait exact-
  full-SHA native-four CI. Do not create a PR or Co-Authored-By trailer.
- [ ] Recover complete original small delivery artifacts/logs/APIs; verify
  four package bodies, native modes/BUILD/notices and installed witnesses.
- [ ] Root creates v0.1.0 GitHub release only if no prior conflicting tag or
  release exists and all gates pass; fresh public MacARM download then checks
  installed both-route lifecycle. Record actual outcome in additive receipts
  and current states. No registry publish or signing/notarization claim.

This closes delivery for the accepted slice, not all limma operations or the
full portfolio goal. All historical evidence remains unchanged.
