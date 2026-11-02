# PLINK completed arena compaction

Base is clean private7cee8e4366ef957981c1741055c3ff716dd5b53b. Green14 native/source
and the separate qualified-CI-binary metadata comparison are accepted; the
original upstream-negative whole-performance gate remains. Plan:
`docs/plans/2026-11-03-plink-arena-compaction.md`.

## Next-path decision

Existing complete single-command attribution contains0.237102s of output
hashing and0.275726s of manifest/durable commit; these are coarse observations,
not isolated hash or fsync costs. Preserve final-file SHA-256 and every file/
directory sync. PGEN header backseeks make chronological stream hashing invalid.

The actual locked sha2-0.10.9 source dispatch falls back to software without
SHA-NI. Do not infer a speedup from changing a feature or version. The upstream
[assembly repository](https://github.com/RustCrypto/asm-hashes) is archived;
the [sha2-0.11 documentation](https://docs.rs/sha2/0.11.0/sha2/) documents x86
SHA-256 SHA-NI and software fallback, with AVX2 listed for SHA-512, not SHA-256.
No suitable measured hash-backend gain has been established. No dependency,
backend or cryptographic algorithm is changed by this decision.

Current RowsBuilder retains geometric String and field-range Vec capacity in
the shared immutable arena. Compact both once before Arc publication. This is
a narrow standard-library resource change, not a new parser, format policy,
foundation or speculative unsafe optimization. Allocation/copy cost is a risk
for later measured timing; neither RSS nor speed is presumed from source.

## Bounded implementation admission

Delegate only src/dataset/metadata/text.rs production compaction and the
existing10000-variant resource consumer in src/dataset/metadata.rs. Fix the
new2611523-byte ceiling before production edits, after observing the existing
2911523-byte baseline; require real RED/GREEN. Run existing guarded fmt,
strict Clippy and complete debug/release suites, no new scientific campaign
or scalar-helper test matrix. Preserve outputs/logs and inherited files.

All work/cache/tmp uses physically checked KIOXIA paths; preparation packet
`/Volumes/KIOXIA/Developments/tmp/plink-arena-compaction-20261103.lVXyaBFp`,
local128MiB envelope plus independent8GiB free headroom. No dependency download,
SSH, publication, source staging or commit by implementer. Root performs one
fresh whole-feature review after the frozen delta, then separately adjudicates
integration and any future native/source/performance execution.
