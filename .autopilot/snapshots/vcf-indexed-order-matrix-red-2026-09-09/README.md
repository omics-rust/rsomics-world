# Rejected region-at-a-time merge: full ordering and dedup matrix

Expected-failure diagnostic only. Independent review rejected the proposed
region-at-a-time algorithm before any fixed-snapshot dispatch or product
commit. This archive must not be used as a release candidate.

The preceding test-only snapshot ran as `34284272693` at
`496a7db14ff2c6b9593757772ffdddbf54db299a`. All four native platforms observed
2/3/65/257 child threads at 1/2/64/256 inputs and the one-thread assertion
failed. Separate broken-pipe diagnosis and byte-zero BrokenPipe classification
also failed. Resource suites passed one test and failed two; writer suites
passed ten and failed one. The unchanged stdout regression emitted 218 header
bytes and failed, while the other 30 concat cases passed. All four artifact
digests/sizes, ZIP contents, manifests and result counts were verified; retained
evidence is under
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/vcf-index-selection-2026-09-09/io-red-34284272693/`.

Production changes followed those failures: deferred writer construction after
indexed/header preflight, retained indexed readers, caller-thread merging,
shared typed region filtering, and innermost I/O kind plus leaf diagnostic
preservation. The structured error source chain is not retained. Resource and
writer fault tests are unchanged. Four concat test groups were added.

The rejected algorithm completes a merge per region. Under variant-overlap
selection, one input's POS 10 record with its first differing base at 100 is
selected only in region 100, after another input's POS 20 record from region 20.
It emits 20 before 10. A same-POS fixture also splits a tie group and defeats
cross-input deduplication. The two new regression groups now collect every
VCF/BCF and dedup-off/on result before asserting, so the first expected failure
cannot hide later cases. These output mismatches are predictions until the
remote run executes. The earlier `vcf-indexed-order-red-2026-09-09` draft was
never dispatched and remains ignored because its assertions stopped too early.

The additional serial/parallel preflight and default record-overlap three-input
tests are expected to pass. That three-input fixture retains 18 old query
occurrences and 15/5 output records for this diagnostic. The intended replacement
will union selected CSI/TBI virtual chunks per contig and merge once per
output-header contig; it will count each overlapping candidate record once per
input rather than counting repeated query occurrences. It will not alter the
existing `view` visit count contract. No envelope scan, input worker threads,
self-referential structure, extra dependency or new public API is required.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (326222 bytes) | `12ca4e1212dc59c1112de52c2749cc980d8338cb940d0fcbeeaa7c81769dc4a4` |
| `files.sha256` | `96656c91b7ec3cf54027371c4b69fe9480f98fae66afe984e6bfeddea4a5864a` |
| `tracked.patch` | `ea4e868171d45341770e35bd8def43e895c4d4acdafbceb80442ee5638a6b800` |
| `tests/concat_cli.rs` | `54e20f6e52df3a662370ae404abbeaf663702e03eee60cea0c0876b0bd087ce3` |

The 172-file inventory is unchanged. Five hashes differ from `vcf-indexed-io-red`:
`src/concat.rs`, `src/concat/stream.rs`, `src/regions.rs`, `src/format/writer.rs`,
and `tests/concat_cli.rs`. Run with `regressions_only=true`. All files remain
on external disks; no local Rust execution occurred. Owning product HEAD is
still unpublished `682942cfa69768dc3a127a8544f2f07213b704ea`, and all candidate
changes remain unstaged. Publication and remaining concat gates stay closed.
