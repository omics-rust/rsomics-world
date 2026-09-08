# Indexed concat stdout preflight regression

This snapshot changes only `tests/concat_cli.rs` relative to
`vcf-index-paths-fix-2026-09-09`, which passed full native audit run
`34279567563`. All other 170 file hashes are unchanged. No production code,
dependency, version, public API, or release metadata changes.

The new test corrupts an existing CSI sidecar of either input and requests
indexed overlap concat to stdout in each of the four output encodings. It
requires a nonzero exit, the selected index path in stderr, and no stdout
bytes. Existing code writes the output header before opening indexed inputs;
the expected failure is a nonempty stdout assertion, not a compiler error or
an invalid fixture. This expectation has not yet been tested remotely.

Run the manual candidate audit with `regressions_only=true`. It executes the
unchanged index-selection suite and the concat CLI suite on all four native
platforms. Full debug/release and compatibility verification will be required
again after production ingestion changes; this red run is not a release gate.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (321629 bytes) | `57bb9a593c649a062f452c26829549fa52ed843268fce52b2f2ee988f346da5b` |
| `files.sha256` | `c24b9e6bb5bbf292e3de2308b1da2316dfb3df29f78fe2d946d02aa398627fbc` |
| `tracked.patch` | `b1086166df1242d58776bb02c70c0b58b782d4a20659077e755f45f85eb02950` |
| `tests/concat_cli.rs` | `7473e9bc880f1a81374e8ea1225089ae674f7472b07de36d325e7381fc95099b` |

The file inventory is identical to the preceding snapshot. Every source hash
was verified against the owning worktree after capture. Source lives in
`/Volumes/KIOXIA/Documents/omics-rust/rsomics-vcf` on unchanged unpublished HEAD
`682942cfa69768dc3a127a8544f2f07213b704ea`; its inherited and new changes remain
unstaged. Snapshot artifacts are immutable audit inputs, not product commits.
The byte-path release hold and remaining concat gates recorded in
`.autopilot/state/vcf-index-selection-2026-09-09.md` still apply.
