# VCF indexed-input repair — 2026-09-08

Status: regression reproduced; repair retained locally, not compiled or
verified after the repair. Do not commit or publish the concat candidate yet.

The product remains based on `682942cfa69768dc3a127a8544f2f07213b704ea`.
All inherited concat changes remain in its external-disk worktree. No unrelated
file was staged, deleted, or replaced.

## Reproduction

The initial `cargo test --locked --offline --test concat_cli` passed all 30
existing tests. Seven new process-level groups in
`tests/index_selection_cli.rs` exercise actual-format detection with non-format
filename extensions, VCF TBI preference, CSI fallback, stale/corrupt preferred and
alternate indexes, different dual-index contents, BCF CSI-only behavior,
missing indexes, and preservation of named output on failure. They cover both
`view` and concat indexed/ligation entry points.

Before the production repair, four groups passed and three failed:

- A valid TBI was rejected because the unused CSI was stale.
- A stale preferred TBI was accepted because preflight checked the valid CSI.
- BCF with only TBI did not report the format-specific missing-CSI condition.

The first test draft also had an invalid empty-TBI fixture; adding the required
tabix header fixed that fixture before recording the three failures above.

## Local repair

`index::read_fresh` detects VCF versus BCF from input bytes, chooses TBI then
CSI for VCF and only CSI for BCF, validates the chosen file's timestamp, and
parses that index once. An existing but unreadable, stale or corrupt preferred
file does not trigger fallback. `regions::IndexedRecords` supplies the parsed
index directly through Noodles' `set_index`, so query no longer independently
resolves a different file. Ligation shares the format-aware validation.

This stays private inside the VCF product. No Layer A API or public crate was
added. The other indexed consumers (`annotate`, `filter`, `norm`, `view`) use
the same existing reader, so their full regressions are required before merge.
Rustfmt and whitespace checks pass; that is not a build/test result.

Local repair fingerprints (SHA-256):

| File | SHA-256 |
|---|---|
| `src/index.rs` | `5c6fed7b698aa3ca89ed62331623f411624999f846ec1164c41b50b695870d24` |
| `src/regions.rs` | `2fd2a88b9033ea4a5f271affba58dd1897ff376aac73ff793fe07f0ee2bb023a` |
| `src/concat/ligate.rs` | `0043c5e6c11427910835b6c5ae73daa46519581a0e2e4c318fb2428387973d3b` |
| `tests/index_selection_cli.rs` | `9124a3279a2c2cd86dd22bb4a6ec5234bd4ff28baa31d08f694edf9197d2f60e` |

## Verification gate

The initial preflight checked `df /` (47%) and resolved Cargo home, target and
TMPDIR to KIOXIA. That percentage was misleading: the macOS physical boot APFS
container is about 94% occupied, with approximately 14.3 GB free out of
245.1 GB. The initial baseline and red-test runs therefore occurred under an
incorrect storage preflight, although every configured project build and
scratch path was external. No further build or test was run after discovering
the physical occupancy. The repaired code has no green test claim.

The authorized Linux `4090` alternative has a 98%-full root, a 99%-full
`/data3`, and bcftools 1.13. It was not used for builds, tests, or new downloads.
No user data was cleaned up. The control-plane manual now explicitly requires
physical APFS size/free-byte accounting instead of `df /` alone.

Once a host meets the storage rule, run the seven index-selection groups first,
then all VCF tests and strict Clippy. Continue the accepted concat repair plan:
replace per-input threads with bounded indexed ingestion, remove redundant
naive-mode passes without weakening integrity checks, close the pinned oracle
matrix, and measure many-input/many-sample workloads. Preserve the unfinished
candidate until the complete release gate, including four-native CI, is met.
