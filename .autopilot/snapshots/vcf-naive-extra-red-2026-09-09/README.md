# Naive concat extended-BGZF regression snapshot

Expected-red, focused four-native test snapshot; not a release. All 174 source
entries/hashes are verified. Relative to `vcf-indexed-bcf-fixture-fix-2026-09-09`,
only `tests/concat_cli.rs` changes: one appended group with the complete prior
70,130-byte prefix preserved. All production bytes are unchanged. The separate
indexed fixture correction is undergoing full verification in `34299936099`.

The new group runs 32 legal extended-frame cases and four canonical controls.
VCF and BCF each use two inputs with matching headers/dictionaries and four
complete expected records. Each input has four independently generated BGZF
data frames: one magic byte, two further bytes, the rest of the header plus
five body bytes, and a complete raw tail frame. Each member and the complete
decoded stream are checked independently with flate2; BCF records are decoded
before product execution. These headers use dense IDs, not sparse IDX fields.

Small extra subfields appear before/after BC in the first input's first frame,
second input's header-boundary frame, or second input's post-boundary tail.
A 9,000-byte extra payload is tested in either input's first frame. XLEN and
the relocated BSIZE are updated and independently checked. Every case runs
to stdout and a named destination. Valid output must reproduce the complete
decoded byte stream and typed body, first-input compressed prefix, and second
input's raw tail plus terminal EOF. Failed commands must preserve the synthetic
named marker and emit no preflight stdout. Failures accumulate across cases.

Source review predicts rejection because naive's local inflater still assumes
a fixed BGZF header. That rejection may mask the separate ordinary reader's
8 KiB compressed-prefix sniff limitation; this red cannot attribute the large
case failure to that second source finding alone. EOF uniqueness and corrupt
frames are covered separately, not established by the new positive matrix.
Missing named output would trigger a fixture assertion before all cases finish.

Both independent source reviews approve expected-red execution. Rustfmt/diff
checks pass; no local Rust or product execution occurred. The standard
`--no-ext-diff --binary` patch was applied to a fresh HEAD archive and matches
every tracked snapshot file. It is unchanged from the preceding capture
because this appended test is part of an inherited untracked source file;
the source archive and manifest include that file in full.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (343516 bytes) | `d183f02759bb34d75dfa63e4260828d8f21b772306c5cf6e67e5b6849be8d328` |
| `files.sha256` | `7d5649d0b15ff6f4e120d811eb90c9e8820f2926073e43bb633bffe54e38a9b8` |
| `tracked.patch` | `936957909d3a430a28a9821eccbfce30beeba91b1b10aebf75b7c8fad6f318af` |
| full concat CLI test | `9bb47947c5293e7591108382f4b93edf5d1fec5521c4c8e225dcdcce98f2ef69` |
| original concat CLI prefix | `06d7a94b237ab77eec2cceb6e18d7f8fd6a6e14510fb6cfd01d7f70a0187ef8c` |

Product HEAD remains `682942cfa69768dc3a127a8544f2f07213b704ea`, index empty.
All scratch is external and boot occupancy exceeds 95%. Dispatch with
`regressions_only=true`; full suites/oracles/static/package are deliberately
skipped for expected red. No two-pass implementation or speedup is claimed.
