# Indexed BCF zero-length record regression snapshot

Expected-red, focused four-native test snapshot; not a release. All 174 source
entries are verified against their manifest. Relative to
`vcf-bcf-framing-fix-2026-09-09`, only `tests/index_selection_cli.rs` differs:
one appended group, preserving its complete original 8,944-byte prefix.
All production files are unchanged. The prior nonindexed/reheader repair is
still undergoing full verification in run `34296650518`.

The new group independently creates a one-frame BGZF BCF and CSI. Its source
header, RID 0/POS 10, complete valid record and gunzip payload are checked.
Actual virtual offsets are measured after header/body reads; the exhausted
data-frame endpoint is `(compressed_frame_length, 0)`. The serialized CSI is
read back and its raw query must return the entire body, including zero bytes.
The product's index builder is not used: it already rejects short shared data.

Sixteen malformed commands cover zero-before-record and record-before-zero,
indexed `view -r`/`concat -a -r`, and v/z/b/u named output. Eight positive
controls use the same index construction and must retain the complete literal
record body. Malformed input must fail without replacing the synthetic prior
destination. No rollback promise is made for streaming stdout.

This slice covers one contig, one chunk and one data frame, not disjoint chunks,
cross-frame zero words, excluded corruption or ligation. Positive decoding
checks record content, not explicit output encoding or physical exhaustion.
Negative checks do not require a particular error class. Focused execution
deliberately skips full debug/release suites, upstream oracles and packaging.

Both independent reviews found no blocker for expected-red execution. Rustfmt
and static diff checks pass. No Rust tests or product binaries ran locally:
boot APFS occupancy remains above 95%. All scratch and snapshots are external.
Owning product HEAD remains `682942cfa69768dc3a127a8544f2f07213b704ea` and its
Git index is empty. Dispatch with `regressions_only=true`.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (340136 bytes) | `7bf83074013805d05be9aface00b711fc8ae0ea3f002bf66703d2a0b77740d1e` |
| `files.sha256` | `e09a661d9f9b8c245800a27e7fefcf9bf2185c89828c25ecabe078b9a68ca071` |
| `tracked.patch` | `12a7c0375ea7a5e6cd44153f89529d27111e86a9f3bea3177e4bc7a2aa64db95` |
| complete indexed CLI test | `12a550f651ba228edca366cf73b34d656106178f07bd6139173eed2c8de625b3` |
| original indexed CLI test prefix | `920a1444679845fe7dce98e6de339712a70c9e5aa3a234ad694d28af4882f41a` |
