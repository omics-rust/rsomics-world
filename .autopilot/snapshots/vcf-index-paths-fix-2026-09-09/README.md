# VCF byte-preserving index path repair

This snapshot changes only `src/index.rs` relative to
`vcf-index-paths-red-portable-2026-09-09`. The other 170 file hashes, including
the regression suite, are unchanged. `default_output_path` now appends the dot
and index suffix to an `OsString`, retaining the input's native path bytes.
No public item, dependency, version, or index-selection policy changes.

The production change followed observed red tests in run `34278711477` at
control-plane commit `5f886722d2e10263c5ff6393058c1fb6253c5144`. Both Linux
architectures passed eight groups and failed the three byte-path groups.
Both macOS architectures passed eight groups and failed the filesystem-free
byte-path group. The logs show original byte 255 replaced by bytes 239, 191,
189. These are product assertion failures, not compilation or fixture failures.

The fixed snapshot has not yet been compiled or tested. Run the manual audit
with its default full scope, not `regressions_only=true`, to verify all ordinary
tests in both profiles, four native platforms and Linux's pinned oracle,
formatting, strict Clippy and package checks. Tests intentionally remain
unchanged from the red snapshot.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (321328 bytes) | `4ce4d9fa1f058d8a2b0f6ec5f7008f5eaf07548951b45d81985e3d908f718704` |
| `files.sha256` | `81fa8caa108a7d9c09ab4ef3af42462c41c6790f1c74727d94cec1c0f6248b3a` |
| `tracked.patch` | `b1086166df1242d58776bb02c70c0b58b782d4a20659077e755f45f85eb02950` |
| `src/index.rs` | `d2cc0e7124f18985d11e75d6aaea8cea3bfa03bce8b6191ee23ff8d10ed5c6ab` |

The authoritative VCF worktree is still dirty on frozen unpublished 0.6.0
commit `682942cfa69768dc3a127a8544f2f07213b704ea`; no product changes are staged
or committed. That frozen commit also contains the preexisting lossy default
index-creation helper and must not be published unchanged now that its defect
is known. A separately verified backport or a verified superseding release
must resolve it; candidate-snapshot tests do not fix that committed source.

All concat ingestion, naive preflight, expanded compatibility, representative
performance and final product-HEAD release gates remain open. This archive
is an audit input, not a release. Earlier snapshots and their evidence remain
immutable. See `vcf-concat-index-2026-09-09/README.md` for capture provenance.
