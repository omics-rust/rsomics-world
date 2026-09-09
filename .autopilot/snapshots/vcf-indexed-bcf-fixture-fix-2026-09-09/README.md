# Corrected sparse-BCF unit fixture

Full four-native verification snapshot, not a release. This isolated capture
starts from `vcf-indexed-bcf-framing-fix-2026-09-09` and replaces only
`src/regions.rs`. Its entire production prefix is byte-identical; only the
supplemental unit fixture changes. All 174 source entries/hashes are verified.
The next naive CLI draft in the live worktree is deliberately excluded.

Full run `34298954629`, exact world
`9868be30ec97bde5343ea1ddf6be04fc92dd3a0f`, exposed a fixture error. The first
Linux ARM raw log was read at external scratch
`vcf-indexed-bcf-unit-failure-MNjEp4/linux-aarch64.log`: both debug/release
invocations stop in the library with 280 pass/two fail. Both supplemental
BCF coarse-filter groups report missing reference names at lines 672/685.
Focused selection 12, concat 38, index 13, resources three, writer 11 and
reheader 21 pass. These are not complete ordinary-suite results. Full
all-target artifact auditing is separate; the failed run remains retained.

Pinned Noodles BCF 0.88 assigns record IDs from explicit header IDX fields,
but delegates header serialization to Noodles VCF 0.90, whose contig writer
omits `idx()`. The fixture therefore encoded raw RID 2 while its serialized
header assigned only dense IDs 0/1. The production filter correctly rejected
that inconsistency. Initial static review checked record assignment but missed
the distinct header serialization path.

The fixture now writes the literal original sparse header into a BCF 2.2
prefix with its exact little-endian length and NUL terminator, then appends
only the record bytes encoded with those same maps. Independent readback
asserts both map directions for IDs 2/5, raw RID 2, complete typed record
decoding and physical record exhaustion before exercising the predicate.
Neither production behavior nor any frozen CLI assertion is changed.

Capture is from external directory `vcf-indexed-fixture-source-oVIC67`, not
the full live worktree. The previous `tracked.patch` is a rendered review diff
because local `diff.external=difft` was active. Its source archive and all
source identities remain valid; do not treat that rendered diff as replayable.
This capture explicitly uses `--no-ext-diff --binary` with an isolated external
Git index. The resulting patch was applied to a fresh owning-HEAD archive and
reproduced every tracked snapshot file. The actual product index stays empty.
Future captures must explicitly disable external diff and verify patch replay.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (341245 bytes) | `fc9dcba16792b8496fa3d36ecb1334aa09fb6bd1a6df28909e9c25e216cd92b2` |
| `files.sha256` | `1375e24d03f0aa6c274f54e608c6f3a525161bfcc7bcaf9a5301f9c944714b77` |
| `tracked.patch` | `936957909d3a430a28a9821eccbfce30beeba91b1b10aebf75b7c8fad6f318af` |
| corrected `src/regions.rs` | `d409d0c0608dc5619b1da2874ca684a27e177cf35029bc92e8a89bc3bec0811a` |

Owning HEAD remains `682942cfa69768dc3a127a8544f2f07213b704ea`. Rustfmt/diff
and static source checks pass; no local Rust/product execution occurred.
All scratch is external, boot occupancy exceeds 95%, and publication and
performance gates remain open. Dispatch with `regressions_only=false`.
