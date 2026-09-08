# Empty-tail BCF/CSI repair candidate

Full four-native source-snapshot diagnostic, not a product release. Only
`src/regions.rs` and `src/index/stats.rs` differ from the preceding
`vcf-empty-tail-red-2026-09-09`. All 172 archive entries/hashes were verified;
the expected-red test file is byte-identical. Owning product HEAD and index
remain unchanged at `682942cfa69768dc3a127a8544f2f07213b704ea`.

## Observed failure and bounded repair

Expected-red run `34290914478`, world
`d7fd393970ea1605ae9cb512e010c6528e21f36a`, fails only the focused index step
on all four native platforms. Each has 11 passing and two failing groups.
Every one of the 32 view/concat × v/z/b/u × empty/populated-file × empty/mixed
query cases rejects a legitimate RID outside the CSI span. Both statistics
shapes omit declared tail contigs. Unknown-region/corrupt-index preservation
guards pass. Concat/resource/writer focused suites remain 37/3/11 green.
No ordinary debug/release or oracle suites were requested for this focused red.

All four API ZIP digests/sizes, CRCs, 56 extracted-file byte identities,
172-item source checks before/after, heads, native jobs and dependency-lock
identities were checked. Evidence is retained under
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/vcf-index-selection-2026-09-09/empty-tail-red-34290914478/`.
The first actual Linux ARM assertion log was read before production edits at
`/Volumes/KIOXIA/Developments/tmp/vcf-empty-tail-first-red-UTWI2S/linux-aarch64.log`.
Exact-head control CI `34290856966` passed.

The repair caches BCF's index slot span once per input. Header/index validation
and raw-ID resolution still precede handling a missing tail. The existing
`view` visitor skips valid BCF IDs beyond that span; concat's chunk-union
reader produces an empty chunk iterator. Unknown IDs and malformed indexes
still fail. VCF handling and both operations' existing record counters remain
unchanged. `--stats --all` appends only declared contigs beyond the CSI span,
sorted by raw ID, with absent metadata represented as `.`. Default statistics
and total counts are unchanged. No speculative foundation or public API was
introduced, and the private resolver is not on the per-record hot path.

## Identities and remaining gates

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (331334 bytes) | `3d59b657a6973b27d0e6d8a61855f0625b3fa27f0398c937f669e45981dace7b` |
| `files.sha256` | `723e7792d7d1bfe25d3d7c30a696f70a0f44563a9690c60ff6f1a533a0c3419b` |
| `tracked.patch` | `03c7d5c64dd206d2c112989edd42f6a7edb68f3e9930345f76001b1106019766` |
| `src/regions.rs` | `e5ed234af0098f9cd6e59b837f0140b366f967e652f189e5aa837338e9f198be` |
| `src/index/stats.rs` | `d3c96793750ca72e408cbe3cb1259e1b6fe0018fbe180f856832707826a65580` |
| `tests/index.rs` | `f4bed365b932d3a3ad2ac2370a522bc05ce616cbdcfa2bd500a8b6c3f2680dc9` |

Independent source review, rustfmt and diff checks pass. Runtime verification
is pending: dispatch `regressions_only=false` and audit every native artifact,
ordinary debug/release suite, and pinned oracle result. The Linux probe must
show the actual HTSlib-created index's empty-tail query and our `--all` output
corrected; bcftools 1.24's observed `--all` signal-11 defect remains an explicit
non-equivalence, not behavior to copy. Full sparse-ID interoperability tests,
remaining concat correctness/representative performance, exact owning-product
head CI and registry authorization remain separate gates. No release claim.

No local Rust build, tests or product binaries ran; boot APFS remains over
95% occupied. All local snapshots, downloads and evidence use external disks.
