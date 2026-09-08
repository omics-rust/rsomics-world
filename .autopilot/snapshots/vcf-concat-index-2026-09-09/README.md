# Unreleased VCF source snapshot

This is an audit input, not a product release or a second owning repository.
The authoritative editable source remains in the external-disk
`rsomics-vcf` checkout. Its frozen `main` remains
`682942cfa69768dc3a127a8544f2f07213b704ea` (unpublished 0.6.0).
The snapshot retains the existing uncommitted concat candidate and the
unverified private index-selection repair, without changing that Git HEAD,
index, version, or dependency graph.

Captured 2026-09-09 from
`/Volumes/KIOXIA/Documents/omics-rust/rsomics-vcf`.
All 171 tracked or non-ignored untracked files were reviewed by path and
verified as regular, non-symlink source, test, benchmark, license, or repository
metadata files. The archive excludes `.git`, build outputs, caches and secrets.
No files were deleted. The four unrelated untracked control-plane files are
not included.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (320806 bytes) | `7de7ecdb1d5375c56f97037b00dee26682fdf4c90e39f28abb7b13e94015da30` |
| `files.sha256` | `d71d58ea62b5d1cb97dd2cf82559bd8419c47259434ce61e0a19dfc24a382475` |
| `tracked.patch` | `e5d3b7bdd099cf919ea6bf29c94824ea5492af505fa19780f4915b0203d39f42` |

`files.nul` is the sorted NUL-separated Git file inventory used by `tar`.
`files.sha256` records every source file's content digest. `tracked.patch`
records only the tracked-file diff; new candidate files are in the full archive.
`worktree-status.txt` records the source worktree's status.

The manual, read-only-permission `VCF candidate audit` workflow validates the
archive digest before extraction and every file digest before and after tests.
It runs the seven index-selection regression groups plus all ordinary tests
in debug/release on four native platform classes. Linux x86_64 also runs strict
Clippy, formatting, package verification and the existing pinned bcftools 1.24
compatibility suites in both profiles. Each platform retains source, toolchain,
resolved dependency and raw test evidence. It has no publication step or
registry secret access. Package verification does not authorize publishing
the candidate under its inherited 0.6.0 version.

This is source-snapshot validation, not exact-product-HEAD release CI.
Successful tests alone do not close the remaining concat ingestion, naive
preflight, compatibility-matrix or representative performance gates in
`docs/plans/2026-08-19-vcf-concat-design.md`. A later source change requires a
new immutable snapshot and validation. Formal release still requires the
owning product's final commit, complete release gate, and exact-head CI.
