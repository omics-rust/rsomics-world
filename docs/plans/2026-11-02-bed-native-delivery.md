# BED 0.2 native/source GitHub delivery implementation plan

> For agentic workers: use superpowers:executing-plans for one coupled delivery
> task with one whole review. Root owns integration, GitHub mutation and adoption.

**Goal:** Deliver the completed eight-operation BED slice as four native
GitHub archives and one complete clean Cargo source package.

**Architecture:** Reuse Table/Ecology's product-local delivery pattern without
changing interval algorithms, public API, dependencies or the command tree.
Registry authorization is separate from usable GitHub installation.

**Tech stack:** Rust1.91, locked Cargo, stdlib Python delivery helper and four
native GitHub Actions classes.

**Spec:** World AGENTS.md, docs/10-products/bed.md and the retained
bed-performance-2026-08-20.md decision. Clean owner is
02b85a1a348c271e485cca629dc4e71fa075388a, tree49a916836bc119696a6845d97c03c992b1d44dec,
91 Git files. Fresh live main agrees; no GitHub release or v0.2.0 tag exists.
Old native32329089591 does not qualify the new delivery implementation.

## Global constraints

- Owner: /Volumes/KIOXIA/Documents/omics-rust/rsomics-bed. Cargo home and
  Rustup are /Volumes/KIOXIA/Developments/cargo-home and rustup-home; target is
  /Volumes/KIOXIA/Developments/cargo-target/rsomics-bed; TMP/cache/scratch are
  under /Volumes/KIOXIA/Developments/tmp. Verify actual external devices and
  resolved tool paths before any build/download. Do not stop on unrelated
  boot-disk occupancy, and do not delete or migrate unrelated/shared data.
- Bound incremental local target/scratch growth to4GiB plus independent8GiB
  headroom. No local upstream scientific executable/runtime is built or run.
  Own Rust unit/CLI/lifecycle execution and stdlib data checks are allowed.
- Keep all src/, tests/, benches/, Cargo.toml and Cargo.lock bodies/modes
  byte-exact. Stable0.2 commands remain sort, merge, intersect, subtract,
  complement, cluster, window and closest. No new options, operation or public
  foundation; later coverage/BEDPE/reference/multi-B work remains absent.
- Add only product-local delivery/guard/notice files, update README/MIGRATION
  delivery status and existing ci.yml. Keep original license bodies intact.
  Any unavoidable packaging-metadata change needs a root ruling first.
- Old performance remains bound to d7b1507b178053a087862255d84a244e4921f192
  and measured executable9a55972f96e11bf087515b61799575505291d12d67902b4c965b8b9e32b51ff8.
  Retain closest RSS loss, window/sort ordering differences, narrow merge win
  and rejected candidate. Do not replay unchanged benchmarks or claim a new
  delivery binary was timed. Original performance data must remain available
  and verified before release; missing evidence holds release, not implementation.
- Root alone stages/commits/pushes/releases; use existing owning main, no PR,
  new worktree, registry publication/token retry, deletion or reminders.

## Review focus

One whole review checks the installed artifact, library/API and unchanged hot
paths together, with particular attention to these user-visible boundaries:

1. Eight commands are installed; no later operations or registry availability
   are implied by README/help/package metadata.
2. Source package preserves all Git bodies and public library/docs/goldens;
   native bundles contain exactly the intended executable and complete enabled
   dependency/adapted-code terms, without case-colliding license trees.
3. Installed sort output is actually reloaded into merge, while trailing BED
   fields, duplicate hits, signed closest/no-hit and known ordering differences
   are tested against existing independently checked goldens.
4. Standard input, common JSON with named output and one malformed-input
   atomic-failure case retain their current contracts.
5. Native host/target identity, clean source VCS metadata and checksum identity
   are actual observations, not cross-compilation, signing or old-OS claims.

## One coupled implementation

**Files:** create scripts/guarded-cargo.sh, scripts/delivery.py,
THIRD_PARTY_NOTICES.md, THIRD_PARTY_LICENSES.md and a canonical licenses/
registry tree/provenance; modify .github/workflows/ci.yml, README.md and
MIGRATION.md. Root owns World dossier/state and release records.

**Consumes:** exact91-file Git baseline, unchanged locked enabled graphs and
registry archives, current eight CLI contracts, existing tests/golden bytes.
**Produces:** exact clean source.crate, four native TARs with BUILD/checksums,
original terms and an installed eleven-call lifecycle with raw outputs.

- [x] Freeze the91 original bodies/modes and intentional changed-path list
  before editing. Record that no delivery helper/native carrier currently
  exists; use one connected missing-delivery check, not invented helper tests.
- [x] Add an owner-specific external-path guard from the current Ecology
  pattern. Preserve tool/target/cache checks and reject storage overrides.
  Create new target/scratch directories only at the explicit external paths.
- [x] Derive the actual locked normal/build graphs for each native target;
  copy original checksum-locked license bodies, including applicable nested
  terms, and document shipping versus dev-only/source graph scopes. Do not
  guess licensing from SPDX alone or introduce dependencies.
- [x] Implement source/smoke/bundle actions using the accepted delivery shape.
  A clean source package has exact Git body/mode membership, Cargo.toml.orig
  and clean VCS identity. A native bundle has one0755 product, original terms,
  README/notices/BUILD at0644 and directories0755; no build scratch or oracle
  binary. Record Rust/native host/head/lock/run/attempt, not minimum-OS/signing.
- [x] Run one installed eleven-call lifecycle: version, help, all eight
  operations and one expected parse failure. Sort the existing sort.input.bed
  through stdin and compare existing sort.expected.bed; reload that result
  through merge with JSON and a named output. Its exact merged BED3 result is
  chr1:50-200, chr1:300-400, chr2:50-200. Use the existing intersect/subtract/
  complement/cluster/window-pairs goldens with their exact compat-test flags.
  Use closest -D b and closest.b-oriented.expected.bed. Keep duplicates,
  trailing columns and no-hit rows; do not sort away contract differences.
  The malformed intersect input must return the common input-error exit and
  leave a pre-existing named output unchanged. Record each argv/status/stdout/
  stderr/output and installed binary hash. No repeated standalone helper gate.
- [x] Extend current CI to actual ubuntu-24.04/ubuntu-24.04-arm/macos-15-intel/
  macos-15. Configure Cargo/target/TMP step-locally under RUNNER_TEMP before
  toolchain setup. Retain pinned Rust1.91 and SHA-checked bedtools2.31.1 build,
  actual oracle version, debug/release full tests, strict Clippy, fmt, package
  verification and existing benchmark smoke. Add rustdoc, clean source package
  tests/install, installed lifecycle and native bundle; always upload original
  artifacts/logs with90-day retention. No cross-build substitutes or skipped
  oracle qualification. Local verification uses own Rust lib/CLI tests only;
  all native CI hosts run complete actual upstream compatibility/differentials.
- [x] Update installation/scope/metadata docs honestly; GitHub release remains
  pending until root's later public action. Keep current public API and all
  scientific/format bodies unchanged. Guarded local fmt/strictClippy/rustdoc,
  Rust lib/CLI tests and actual installed lifecycle run once; freeze complete
  diff/source/notices and raw local results for one whole review.

## Root adoption

Implementation, one whole review/correction and actual native-four original
adoption are complete atbcbee5f8/run41348074250. See
bed-native-delivery-2026-11-02.json. GitHub publication and fresh public
installation remain the next mandatory gates.

Root independently reviews the complete candidate plus one fresh whole review,
then addresses material findings together. Before any owning push, verify old
performance originals and exact protected-body equality. The new exact-head
four-native gate and original source/archive/lifecycle evidence must pass
before root may publish GitHubv0.2.0 and run a fresh public Mac download of the
same eleven-call lifecycle. No registry publication, performance reassignment,
old evidence overwrite or claim of portfolio completion.

Ruling: unattended main commits/releases and parallel implementation are
already delegated by the human. This bounded plan follows the established
product boundary and delivery pattern, so it adds no approval checkpoint or
per-helper review campaign. The state ledger is durable; accepted originals
are preserved rather than deleting a skill workspace at completion.
