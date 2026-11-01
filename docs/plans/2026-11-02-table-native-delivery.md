# Table six-operation native delivery implementation plan

> For agentic workers: use superpowers:subagent-driven-development for the
> single coupled task below. Root owns integration, CI and publication.

**Goal:** Deliver the already qualified six-operation table product as four
native GitHub installation archives and its complete source package.

**Architecture:** Preserve the owning Rust implementation, locked dependencies,
tests, comparison rules and benchmark inputs. Extend its existing native CI
with packaged-source installation and one connected installed-binary workflow;
use the established limma/DESeq delivery pattern rather than a new framework.

**Tech stack:** Rust1.91, Cargo, existing rsomics-common/help, Python stdlib,
GitHub Actions on four native hosts.

**Spec:** AGENTS.md, docs/10-products/table.md and
docs/10-products/table-performance-2026-08-20.md. Owning baseline is clean
2bd0fd3698c152bb27e7d0d7635d51fb41655112,
treea7c80bf87d72bce97e3df9072856bdd3d5d8e91c.

## Global constraints

- Owner: /Volumes/KIOXIA/Documents/omics-rust/rsomics-table. Scratch, Cargo,
  rustup, targets and every tool cache remain physically external.
- Stable scope stays validate/select/filter/sort/join/groupby. No Rust,
  Cargo.toml/lock, upstream fixture/comparator or numerical changes.
- Registry authorization is still held. Do not invoke publish.yml, expose a
  secret, imply cargo install works from crates.io or add a public foundation.
- All original accepted performance stays explicitly bound to old2bd0fd3/
  ELF6e490331. Infrastructure-only source equivalence preserves that evidence,
  not an invented timing of the new delivery executable. Disclose the sort
  memory and high-cardinality grouping throughput trade-offs.
- Direct main/root-only commits; no PR, Co-Authored-By, deletion or unrelated
  changes. Keep original logs and failed checks. No upstream executable on Mac.

## Review focus

The installed workflow must catch wrong package/source revisions, incomplete
stable operation dispatch, corrupted CSV quoting/schema or joined multiplicity,
malformed-input output replacement, and missing enabled-dependency notices.
These are one lifecycle/review boundary, not five independent test campaigns.

## Task: complete packaged CLI delivery

**Files:** existing owning README.md and .github/workflows/ci.yml;
new scripts/delivery.py and scripts/guarded-cargo.sh; generated LICENSES/
notice bodies/provenance and THIRD_PARTY_NOTICES.md only if needed. Root owns
World state/dossier updates. Do not modify publish.yml or benchmark behavior.

**Consumes:** exact owning Git source, Cargo package/list/metadata, original
locked registry archives and target-specific normal/build dependency graph.
**Produces:** complete verified source .crate, one installed native archive
per target, manifest/checksums/notices and raw lifecycle/install receipts.

- [ ] Read actual CLI/tests/packaging and existing limma delivery implementation.
  Verify clean baseline/source identity and retain protected-body manifest.
- [ ] Add one installed CLI lifecycle check: authored quoted CSV/TSV data
  passes validate, selected/filtered/sorted output joins metadata and groups
  with independently computed identities/values/multiplicity. Cover a gzip
  named result and one malformed-input transactional failure in this workflow.
  First invoke it against the absent candidate package/delivery entrypoint to
  establish the missing delivery, then run it once against the actual Rust
  binary after implementation. No mocks, helper-level review ladder or new
  representative scientific benchmark.
- [ ] Derive only enabled normal/build dependency notices from the unchanged
  lock and actual native graphs. Preserve source/license identities and include
  required notices in the binary archive; do not claim complete system DSO
  inventory. Reuse established tooling where applicable.
- [ ] Extend native CI, keeping existing pinned csvtk/full-oracle jobs and
  package gates. Check actual native compiler host, explicit runner-local
  tool/cache/target routes, strict Clippy/format, full debug/release tests,
  complete package source/Git equality, packaged-source release tests and
  cargo install --path followed by the same lifecycle. Retain all four delivery
  candidates and failures for90 days; do not silently ignore required steps.
- [ ] Document GitHub release installation as not yet published until root
  actually publishes. State six-operation scope and unchanged source/performance
  limitations. No advertised unfinished operation.
- [ ] Hand off one frozen diff/report, protected-source/lock equality, actual
  local guarded formatting/Clippy/full tests and installed-workflow logs.
  Local scratch/targets are capped4GiB plus independent8GiB headroom.

Root performs one integrated delivery/public API/hot-path review, corrects any
material finding once, commits/pushes and waits for exact-head native/oracle CI.
Only complete originals and actual packaged lifecycle acceptance authorize a
GitHub v0.1.0 release. Root then downloads the public Mac native archive into
fresh external storage and runs the same complete lifecycle once. Old accepted
source/performance and registry hold remain separate, immutable authorities.

Ruling: user-authorized unattended execution supersedes the skill's interactive
plan approval and worktree/PR steps. The existing owning/main layout and one
integrated task/review are retained; no per-helper approval/test ladder.
