# Exact private creation support candidate: green-12

The 146 source files are byte-identical to green-11, including all accepted
production, dependency-lock, fixture and old measurement files. Only the
archive container changes: exclude macOS AppleDouble/xattr metadata and use
regular USTAR members that match the complete source manifest exactly.
Checksums also authenticate this README, files.sha256 and source.tar.gz.

Run 36718150977 passed all four native debug/release test gates for green-11:
55 exact-creation tests, 11 old measurement tests and 72 ordinary/external
tests per profile/target. Linux format/strict Clippy passed. Original source
and hashes agree, but its 292-member archive includes 146 undeclared
AppleDouble members and cannot pass the existing strict snapshot verifier.
The original snapshot/run is preserved, not rewritten or accepted as a
downstream measurement archive. Its red-7 predecessor has the same packaging
defect; the original missing-helper failure remains a bounded test-first
observation, not an accepted downstream source package.

This new clean archive requires fresh native CI and original artifact audit.
It introduces no algorithm, factory timer, performance claim or publication.
