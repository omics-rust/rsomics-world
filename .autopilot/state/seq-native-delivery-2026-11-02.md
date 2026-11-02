# Sequence 0.3 native delivery

Owning main: `8eaf9c8a685c08bdf1698745b65a3a01c609ec70`.
Actual exact-head CI `41435718744` is SUCCESS on Linux and macOS,
each x86_64 and aarch64. Original artifact acquisition and independent
acceptance are in progress; no GitHub release or registry publication yet.

Plan: `docs/plans/2026-11-02-seq-native-delivery.md`.
The reviewed delivery delta changes owning version/lock metadata, CI,
installation tooling, documentation and notices. All thirteen production Rust
source bodies and the registry publication workflow remain byte-identical to
accepted `3ba1818e85477573995f6a391522f2ef35054c79`; dependency resolution
is unchanged apart from the owning version.

One fresh integrated review finds no P0/P1/P2 issue. Root corrects the copied
helper docstring before integration. Optional P3 remains: fourteen late-error
and alias checks enforce exit/output preservation and retain stderr but do not
join the structured command ledger. These checks execute; this is an audit
detail, not missing functional coverage. Copied license text whitespace stays
unchanged; only generated extra EOF blank lines are normalized.

Local candidate evidence lives in
`/Volumes/KIOXIA/Developments/tmp/seq-delivery-final-20261102.nzYcRv`.
Its dirty-tree packaging explicitly reports `clean_vcs=false`; it is not final
source acceptance. After root commits, clean locked/offline packaging and the
complete normalized-source check pass separately in
`/Volumes/KIOXIA/Developments/tmp/seq-delivery-root-clean-20261102.Nd6GI5`.
The clean source crate is 94641 bytes, SHA256
`417d9d67f8e45fbc852b018d7886adb7b73248733b1f8e98d7fe4ff0b6497907`,
with 93 members, 91 original files and `clean_vcs=true`.

All four actual CI jobs pass debug/release tests, native-host validation,
pinned SeqKit source oracle, clean source packaging, fresh package install,
seven-command lifecycle and delivery artifact upload. Linux x86_64 additionally
passes format, strict Clippy, rustdoc and benchmark smoke. Retained artifacts
must still establish final executable/architecture/source identities before
release. No new representative timing campaign is required or claimed.

Measured executables remain separately preserved under
`/Volumes/KIOXIA/Developments/tmp/seq-strand-campaign-20261102.JNGmJL`.
The earlier standalone resource-use acceptance does not become fresh timings
for metadata-only 0.3 binaries. Standalone wall-time regressions and overlapping
ranges remain disclosed; no universal throughput claim.

Next: retain and independently inspect four original CI artifacts, then check
the live tag/release namespace before an immutable GitHub v0.3.0 release and
fresh public binary/source installation. Registry credentials remain on hold;
do not retry the unchanged organization secret. No unrelated files or old
releases are removed.
