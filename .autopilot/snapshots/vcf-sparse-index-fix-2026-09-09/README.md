# Sparse BCF index and statistics repair candidate

Full native source-snapshot diagnostic candidate, not a product release.
The 172-file inventory is unchanged; only `src/index/build.rs`,
`src/index/stats.rs`, and `tests/index.rs` differ from the preceding red.
All original sparse regression source is byte-identical; one additional
corrupt-index regression is appended. Owning VCF HEAD and Git index remain
unchanged at unpublished `682942cfa69768dc3a127a8544f2f07213b704ea`.

Observed red: run `34288833017`, world
`75c2cf858cde5dba0c17692b67db8b61ef8fb95e`. On all four native targets the
focused, ordinary-debug and ordinary-release index suites each pass seven
groups and fail exactly two. Both nonempty worker cases reject valid RID 5;
both empty cases create three slots instead of ten. Default and `--all`
statistics mislabel RID 2 as the empty contig and RID 5 as n/a. The unknown-RID
guard passes. Concat 37, resources three and writer 11 remain green in the
focused suites. Ordinary suites stop at the index failure, so this is not a
complete ordinary-suite pass. Artifact digests/sizes, ZIP bytes, source checks
and exact failure groups were verified and copied to
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/vcf-index-selection-2026-09-09/sparse-red-34288833017/`.

The builder now derives CSI/linear-index slot span from actual header dictionary
IDs, including a trailing declared empty contig. The public build-summary
count stays the declared contig count. Existing raw-ID dictionary validation
is retained. Statistics use the detected BCF format and original string map
before any index auxiliary names; VCF/index-only naming behavior is unchanged.
Unnamed empty slots are omitted; a nonzero count in an unnamed BCF slot fails
before output. The extra guard tests CSI slot 1 with records, with/without
fabricated auxiliary names, under `--stats` and `--nrecords`. It has not itself
been observed red and is not claimed as an additional reproduced defect.

The retained Linux x86_64 probe confirms bcftools/HTSlib 1.24 can read and index
the fixture. Its default statistics and totals are correct, while `--stats
--all` terminates by signal 11 with empty stdout/stderr. All 16 post-version
commands completed within their deadlines, with cores disabled. This is an
observed upstream edge-case defect, not behavior to emulate or a claim about
other versions/platforms.

The same probe exposes a separate uncorrected compatibility gap: HTSlib omits
trailing empty RID slots from its CSI. Querying declared RID 9 succeeds with
no records in bcftools but exits 4 in current rsomics `view`. Queries for RIDs
2 and 5 agree. This candidate intentionally leaves indexed query code
unchanged; successful index/statistics validation cannot close that gate.
Also check `--all` reporting for declared contigs beyond the last CSI slot
during that follow-up. Do not advertise complete sparse-index interoperability.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (330063 bytes) | `c2a668944b88b1cb4e93d1ec4063f0872680b29ed8611646eac688e2153b9278` |
| `files.sha256` | `ace96131d2b5cd3c7bb0c5a9d7c1e3bab1737a9cd0cbe09d4f2aabe8f7f197b8` |
| `tracked.patch` | `82828835fb5b8af7568b6ccf19531af63a00d91658550e993155646d24647f10` |
| `src/index/build.rs` | `c8ef94a53140ad9533db17806fb6a94df445d7a27533a425e3aed3ce8cd7fdff` |
| `src/index/stats.rs` | `8a552d3c21acd1f27e4ca708739b4a80513f47052be995d0328cd42017fc990c` |
| `tests/index.rs` | `e39d4236b600617edc9397ad5bacb8fbde6eda392c18d4b91cdffb8c981ed5f2` |

Source review, rustfmt and diff checks pass; remote execution is pending.
Run with `regressions_only=false`. No local Rust ran, no public API/dependency
was added, and no huge-header allocation safety, benchmark advantage or
release readiness is claimed. Header allocation, empty-tail interoperability,
remaining concat correctness/performance and exact-product-head gates remain.
