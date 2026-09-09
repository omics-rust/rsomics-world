# Corrected pinned-oracle empty-index layout

Full native diagnostic snapshot, not a release. This is the preceding
`vcf-sparse-oracle-bidirectional-2026-09-09` archive with exactly one test
expectation changed: bcftools 1.24 creates three, not zero, CSI slots for
this completely empty BCF. All 174 entries and hashes were verified; all
other bytes, including all production code and independent zero-slot
regressions, are identical to that snapshot.

Run `34292257895`, world `d4383d4e8e672cbb881d1eee383039cac93bb1eb`, fails
the new oracle group in both profiles on Linux x86_64 at
`tests/index_compat.rs:185`: actual span three, expected zero. The nonempty
case completes all bidirectional checks before the empty case reaches this
assertion. The empty case's later interoperability checks have not executed;
do not count them as passed. The other three native jobs succeed and the
Linux failure is isolated to the oracle step. Its exact-head control CI
`34292169465` passed. Full artifacts are being retained separately.

The actual failure was read in
`/Volumes/KIOXIA/Developments/tmp/vcf-sparse-oracle-first-failure-cZSubA/linux-x86_64.log`.
[HTSlib 1.24 BCF indexing](https://github.com/samtools/htslib/blob/1.24/vcf.c#L4391-L4431)
counts declared contigs when initializing the index, then expands by raw
record IDs. Thus this empty file starts with three slots while its nonempty
counterpart expands to six. This corrects an oracle-fixture assumption, not
a production bug. A zero-slot empty index remains independently covered as
a separate fixture; neither shape is advertised as the sole format layout.

The external staging directory
`/Volumes/KIOXIA/Developments/tmp/vcf-sparse-oracle-layout-source-giEGZU`
was extracted from the previous frozen archive and overlaid only with the
corrected `tests/index_compat.rs`. A newer uncommitted `tests/reheader_cli.rs`
expected-red draft in the live product worktree is deliberately excluded
from this snapshot and `tracked.patch`, and remains untouched locally.
`worktree-status.txt` records that live draft honestly. This archive is an
explicit isolated source candidate, not a capture of every current file.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (332158 bytes) | `c92ab989d15a2bf8bc194c638b3f698db306b1a12fdc379e18a0ce4beaf5056c` |
| `files.sha256` | `73546225149f54bae22c8fd9d4c41935dc50465291e3300b83dccc3114224fe4` |
| `tracked.patch` | `8d3df48cba188ec53d8e899e20b8426cc24a2f1eebae5eb66ff99dadc068d915` |
| `tests/index_compat.rs` | `4cd780ab23494cda5450c30fd9c401cc484197bae23de7a4b3d57ccc3c8b46de` |

Dispatch `regressions_only=false` to validate both complete file shapes.
No production code is modified to accommodate the bad test expectation.
Owning product HEAD/index remain unchanged at
`682942cfa69768dc3a127a8544f2f07213b704ea`. No local Rust or product binary
ran; all generated files and downloaded evidence are on external storage.
