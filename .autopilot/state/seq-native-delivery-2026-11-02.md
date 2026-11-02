# Sequence 0.3 native delivery

Owning main: `8eaf9c8a685c08bdf1698745b65a3a01c609ec70`.
Actual exact-head CI `41435718744` is SUCCESS on Linux and macOS,
each x86_64 and aarch64. Original artifact acquisition and independent
acceptance pass. GitHub v0.3.0 is published and fresh public binary/source
installation passes. No registry publication.

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

## Original delivery assets and GitHub release adopted

Packet: `/Volumes/KIOXIA/Developments/tmp/seq-native4-evidence-20261102.UMyrRF`.
Root reads the verifier and independently checks all four original artifact
ZIP digests against raw GitHub metadata, all five release asset hashes,
BUILD exact-head/run/binary/source identities and all eight profiles at
56 passes each. All four source archives equal the clean local source crate.
The packet retains raw APIs/log ZIP and safe complete archive member tables.
Four 48-file bundles contain only explicit executable/docs/license paths;
actual ELF/Mach-O CPU identities and installed lifecycle proofs match.

Both live release and tag queries return 404 before creation. One release
creation succeeds at 2026-11-02T11:33:45Z:
https://github.com/omics-rust/rsomics-seq/releases/tag/v0.3.0
Fresh live metadata confirms tag commit `8eaf9c8a685c08bdf1698745b65a3a01c609ec70`,
published/non-draft/non-prerelease status and exactly six uploaded assets:
four native bundles, the source crate and SHA256SUMS. Public asset digests
match the adopted local files. SHA256SUMS is 508 bytes, SHA256
`9e12791d56faeb419c29642aa6576096c6ff3899fd90181db3dccf1096ed4315`.
Assets are unsigned; none is described as remeasured or registry-published.

## Fresh public installation accepted

Packet: `/Volumes/KIOXIA/Developments/tmp/seq-public-install-20261102.r7RHbf`.
Unauthenticated public Mac ARM bundle, source crate and sums downloads match
adopted bytes/hashes and live GitHub asset digests. Fresh binary installation
and isolated-target offline source installation each pass the existing
22-call seven-command lifecycle. Root independently verifies all downloaded
and installed body hashes and both lifecycle records, including connected
quality preservation, JSON, BGZF and late-error/alias preservation.

Downloaded binary: 1673872 bytes, SHA256
`373380e66d20e5da51bee600e612f0bab2d964e2f62b060dc7173dd2672d1faf`.
Fresh source-built binary: 1673968 bytes, SHA256
`2ee1f04f7cfde121af24298d2e6e67e3cb3672acf07ea09de98be218602bc89a`.
Both are functional acceptance, not byte-reproducible builds or new timings.
All caches, install roots, source target and scratch remain external.

Post-release README commit `b3d045affba732f2075cac4fc164bf08e204d220`
points users to actual GitHub assets. Its CI `41438098373` is pending;
immutable release/tag/assets remain at accepted `8eaf9c8`, never retagged or
rebuilt from this documentation-only head. Tag-triggered CI `41437808068`
is additional CI, not registry publication; Publish is dispatch-only.

Registry credentials remain on hold;
do not retry the unchanged organization secret. No unrelated files or old
releases are removed.
