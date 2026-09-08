# VCF index path regressions: test-only snapshot

This diagnostic snapshot follows `vcf-concat-index-2026-09-09` with exactly one
changed file: `tests/index_selection_cli.rs`. All 170 other file hashes,
including production source and Cargo.lock, are unchanged. The owning VCF
checkout remains on unpublished 0.6.0 commit
`682942cfa69768dc3a127a8544f2f07213b704ea`; none of its changes is staged.
See the previous snapshot README for the capture procedure and release limits.

Four new groups cover:

- Unix: byte-preserving `default_output_path` using exact `OsString` bytes,
  without filesystem access;
- Linux: index lookup after renaming an already indexed ASCII fixture and its
  sidecar to matching non-UTF-8 paths, with and without an unrelated lossy-name
  sibling; VCF TBI, VCF CSI, BCF CSI and all three indexed command paths;
- Linux: default index creation retains the actual input filename bytes;
- Unix: a dangling preferred TBI with a valid CSI fails atomically.

The filesystem tests are Linux-only because APFS requires valid UTF-8 names
for creation. The filesystem-free path test covers the same byte-preservation
contract on both macOS architectures. This is an explicit platform distinction,
not runtime skipping after a fixture failure. [Apple APFS filename contract](https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/APFS_Guide/FAQ/FAQ.html)

The unchanged production helper uses `input.display()` to create sidecar paths.
Expected results before its repair are eight passing and three failing groups
on Linux, and eight passing and one failing group on macOS. These are predictions,
not observed results. No production fix is present. Run the manual workflow
with `regressions_only=true` to collect four-native red-test evidence.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (321308 bytes) | `6afa597b5a96144fa628397dcb9b1d23fe3e1216d6c9adc4622bcf0f5886560d` |
| `files.sha256` | `1afb5e607a86a6261c797b04f8610acde320070da6262f0eb8f1cbe5d6e67a81` |
| `tracked.patch` | `e5d3b7bdd099cf919ea6bf29c94824ea5492af505fa19780f4915b0203d39f42` |
| `tests/index_selection_cli.rs` | `920a1444679845fe7dce98e6de339712a70c9e5aa3a234ad694d28af4882f41a` |

All 171 source files were hash-checked after capture. The original snapshot
and its full audit run remain immutable; this test-only snapshot does not
replace their evidence or change the frozen product release head. An earlier
local test draft was not dispatched because its raw-byte filesystem tests
incorrectly included APFS; that draft is not part of this workflow or commit.
