# External CNV fixture reader candidate: green-4

This snapshot implements the test-only explicit-root reader after the actual
`red-3` missing-method failure in Linux run `36220790633`. Only
`tests/support/mod.rs` changes from red-3; all eight added reader tests,
production code, original synthetic fixtures and Cargo.lock are unchanged.

The reader returns file/row-contextual errors for malformed tables, validates
shape before indexing and rejects nonfinite values. The original synthetic
`Fixture::load(profile)` entry point delegates to the explicit-root loader.
Positive tests cover a non-square matrix and the existing synthetic profile.

Four-native debug/release verification and strict static checks are pending.
This snapshot has no external-oracle feature and makes no shipped-data
conformance claim. The shipped oracle's separately diagnosed R/Python decimal
parser bridge is under review; no production numerical algorithm or tolerance
has changed to accommodate it.

`files.sha256` identifies exact source bytes. `checksums.sha256` covers this
receipt, the source manifest and archive. Do not publish this candidate.
