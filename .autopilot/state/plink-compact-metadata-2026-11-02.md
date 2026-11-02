# PLINK compact metadata wave

Status: implementation in progress, no integration/native/performance or
publication acceptance. Full portfolio goal remains active; the previous
turn made concrete code, namespace and32-command diagnostic progress.

Owner: `/Volumes/KIOXIA/Documents/omics-rust/rsomics-plink`, base
`9d43df417bf2a729438fb4194e9f82cbba10fe6c`.
Plan: `docs/plans/2026-11-02-plink-compact-metadata.md`.
Scratch:
`/Volumes/KIOXIA/Developments/tmp/plink-compact-metadata-20261102.gGmEWE`.

Root reads full metadata parser/writer and PGEN/BED/VCF constructor and
revalidation paths, workspace accounting and relevant make/LD consumers.
The private model retains per-field owned strings plus typed duplicates;
~99 textual-field uses across16 files are a bounded coherent migration.
I's slower/higher-RSS same-campaign diagnostic remains authoritative.

Root chooses a standard-library owned shared text arena, flat field bounds
and row ranges with content-based Text/Fields semantics across arenas.
Headers/preambles and input/output/numeric/durability policies are unchanged.
No public foundation, dependency, self-reference or unsafe lifetime extension.
Do not optimize by identity-copying the transformed PGEN/PVAR products.

This advances the source-confirmed allocation/storage problem directly; no
causal phase-time fraction or performance improvement is claimed before
measurement. Privileged perf is not assumed available. A separate instrumented
build is not made a prerequisite for the structural cleanup; this avoids
another measurement-infrastructure wave before changing the product.

Actual pre-implementation resource RED:10,000 valid five-column PVAR rows
plus PSAM account for5022863 bytes under the old model. Fixed4500000-byte
budget test fails as intended. A literal full-output unknown-column/UTF-8/
multiallelic/INFO-PR roundtrip passes before the representation change.
Baseline/red logs and newly-created fixtures are retained; none deleted.
Worker verifies USBdisk12s1,37GiB free and external Cargo/Rustup/tmp/cache/
resolved target before the tests. No main implementation result is yet
adopted. Preserve unrelated BED map work and inherited world changes.

Next: complete all constructors/consumers/accounting in one model, full
debug/release/fmt/strictClippy, freeze and one fresh whole-feature review.
Actual four-native/source-consumer qualification and later same-input matched
performance remain separate gates. Pair scheduling is not changed in this
wave and no CLI promises a measured improvement yet.
