# Sparse BCF bidirectional oracle candidate

Full native source-snapshot diagnostic, not a release. All production files
are byte-identical to `vcf-empty-tail-fix-2026-09-09`, whose full diagnostic
run `34291571671` is independent of this test expansion, at world
`ac6ccc9e8cecec466844b627f3b2f5cd41d46037`. Its exact-head control CI
`34291509323` passed. Do not infer that run's final status from this snapshot.

The archive contains 174 files: only `tests/index.rs`, `tests/index_compat.rs`,
and two added shared fixture files differ from the 172-file repair snapshot.
All old index test functions and old compatibility groups are unchanged.
Only the index suite's fixture helper moves literal data and raw-ID validation
to a test-local module. Both empty/nonempty VCF byte sequences were verified
against the original literals. This is product-internal test reuse, not a new
foundation or public API. The diagnostic Python probe remains independent.

The added ignored oracle group requires bcftools 1.24, which creates the
BCF and its CSI. It checks that the sparse dictionary remains 5/2/9 and raw
records remain 2/2/5, with observed-version CSI span expected to be six or
zero for nonempty/empty files. These slot counts are not a general format
requirement. The product builds copies with zero/two decompression workers.
Across the three indexes, both tools must return exact expected default
statistics, total counts and populated/empty/mixed-region records. Product
`view` and overlap `concat` must both match the explicit bodies. There are
72 query checks (two file shapes × three index sources × four selections ×
three readers). This test is not an output-encoding matrix; those native
regressions are retained unchanged.

Product `--stats --all` is checked against the declared-contig contract for
all three sources. The upstream `--all` invocation is deliberately not used
as an equivalence oracle: the separate pinned probe has observed signal 11
on this sparse dictionary. That diagnostic remains in the full workflow,
with timeouts and core dumps disabled. All source review findings are resolved,
including upstream default-statistics checks on product-generated CSI.

Primary behavior references are [HTSlib indexed iteration](https://github.com/samtools/htslib/blob/1.24/hts.c#L3217-L3254)
and [bcftools statistics](https://github.com/samtools/bcftools/blob/1.24/vcfindex.c#L62-L206).
Source inspection informs the tests; it does not replace their execution.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (332155 bytes) | `aae04505c4e50fb0c46af7f4cca871c294666bd4ddd4d701f746e72fb9c7ea99` |
| `files.sha256` | `392b1c4976f3ba58e6b637dad064a66b2dc546f785f601f7d7e4d2753ea8c2ef` |
| `tracked.patch` | `32c92f50cb4d3b455b70829c35c0060875f39f01f894c4b2aea3dd5c9d2f8706` |
| `tests/index.rs` | `def4246ee27036f8a5acfe3c9cfff3f2f52300a711ea4d7911e854119742ce22` |
| `tests/index_compat.rs` | `2807feeaf446afa1dd6337426cbcc9e8728507c08a93529f24d3f286ef6c9d72` |
| `tests/common/sparse_bcf.rs` | `4cffe92279f08d0c3e405270e1384a0645badd9d1442fe410e2b987f24b57bf7` |
| `tests/fixtures/sparse-ids.vcf` | `64412868280570686854a1712376f42595ee1c7acffc718208780203e2acfbac` |

Archive inventory/hashes, fixture equivalence, rustfmt and diff checks pass;
the new oracle group is not executed yet. Run `regressions_only=false`, with
62 rather than 61 expected ignored/release-oracle groups per profile. The
initial `vcf-sparse-oracle-2026-09-09` draft was never staged or dispatched.
Owning VCF HEAD/index remain unchanged at
`682942cfa69768dc3a127a8544f2f07213b704ea`. No local Rust or product binary
ran, and all generated artifacts remain on external disks. Concat's remaining
correctness/performance and exact-product release gates remain open.
