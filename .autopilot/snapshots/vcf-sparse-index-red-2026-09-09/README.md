# Sparse BCF index construction and statistics: expected-red diagnostic

Test-only source change relative to the full chunk-union candidate. Exactly
`tests/index.rs` differs; all production source and the 172-file inventory are
unchanged. No local Rust ran and no owning-product commit or publication is
authorized by these diagnostic results. VCF HEAD remains unpublished
`682942cfa69768dc3a127a8544f2f07213b704ea` with inherited edits unstaged.

Predecessor run `34287699977`, world
`63a00b6b2a20be7da7b6bb1d413a44110f310c10`, completed successfully on all four
native targets. Each Linux profile passed 424 ordinary tests and each macOS
profile passed 422, with 61 oracle tests ignored in ordinary debug/release.
Linux x86_64 explicitly passed all 61 pinned oracles in each profile over 11
suites, formatting, strict Clippy, harness syntax and package verification.
Concat passed 37 focused groups, resources passed three, and writer errors
passed 11. Source and artifact identities are retained separately in
`.autopilot/state/vcf-indexed-ingestion-2026-09-09.md`.

The new fixture contains three declarations in header order chrA/chrB/empty,
with raw IDs 5/2/9 and three records in RID order 2/2/5. The fixture verifies
both parsed dictionaries and raw IDs before invoking the tested operation.

- Build matrix: empty/nonempty input times serial/two-worker decompression;
  expected CSI span is ten slots, summary remains three declared contigs,
  totals and each indexed region are checked. The current nonempty builder
  should reject RID 5; empty input should produce only three index slots.
  Failures are collected across all four cases before asserting.
- Statistics: an independently constructed Noodles CSI exposes the naming
  bug without depending on the defective builder. Default and `--all` outputs
  are collected separately. Intended output names only declared contigs,
  omits dictionary holes, and preserves their real lengths/counts. This is not
  yet a claim of equivalence with bcftools `--all` behavior.
- Invalid IDs: raw RID 1 (hole), 10 (outside), and i32::MAX with an unchanged
  small header must still fail with a dictionary diagnostic and preserve an
  existing index, under both decompression modes. No huge-ID header is created.

Expected result: two new groups fail and the invalid-ID group passes. These
are predictions until a native run reaches the intended assertions. The
existing six index groups should remain green. Full runs also record a bounded
Linux x86_64 bcftools 1.24 sparse-ID behavior probe: version/input/raw-RID
prerequisites are asserted, while stats and query exit codes/output are
retained without assuming equivalence or an upstream crash. Core dumps are
disabled; all child commands use a runner-temporary fixture directory and
ten-second deadlines. This probe is separate diagnostic evidence, not a new
advertised product operation or an assertion-free compatibility test.

Relevant primary sources inspected before designing the regression:

- [Pinned VCF 4.3 / BCF 2.2 specification](https://github.com/samtools/hts-specs/blob/da617203a9527537746e200abda2885bec3a822c/VCFv4.3.tex), dictionary and IDX sections.
- [HTSlib 1.24 index construction](https://github.com/samtools/htslib/blob/1.24/vcf.c), `bcf_index` uses record RIDs.
- [bcftools 1.24 index statistics](https://github.com/samtools/bcftools/blob/1.24/vcfindex.c), raw index slot iteration and header-ID name lookup.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (329471 bytes) | `15c4370a2446324beacd28f784691af6b014a7a48f40f1855c8bbcbd35942bb2` |
| `files.sha256` | `1bd368f8883a42ed3fe9df72e1cc2d82f2100485100deb708cf47e489a66eaa3` |
| `tracked.patch` | `c7997f866c291add0c7b67d2d40ba7fbf87f7a49927e3aa15f6d171c66fc8eef` |
| `tests/index.rs` | `4d4f97b6bb5755955e116404a2ac16926ad0ba755a88409ec2c34d94dd699008` |

Run with `regressions_only=false` to include the pinned oracle probe. No
production repair occurs before observed red evidence. A guarded-header
allocation limit is a separate open question: upstream Noodles header and
index structures already grow densely by IDX/RID, so resizing our later
linear-index vector alone cannot establish huge-header allocation safety.
