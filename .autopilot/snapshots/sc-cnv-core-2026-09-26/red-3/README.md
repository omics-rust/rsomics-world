# External CNV fixture reader: red-3

Test-first snapshot based on accepted unpublished product commit
`13961bf6a42cb4cbbf36c528331aafb98a52937b`. Only
`tests/cnv_fixture_reader.rs` is added. Production, the existing reader,
synthetic fixtures, Cargo manifest and dependency lock remain unchanged.

The new tests require `Fixture::load_from_root(&Path)` returning a result.
The current reader has no such API, so an actual missing-method compile
failure is expected from this snapshot. It has not been observed locally.
Negative cases cover missing stage files, wrong cell headers, extra rows,
empty expression files, malformed coordinates and nonfinite values. Positive
controls check a two-gene/three-cell matrix's exact identities and column-major
values across all ten stages, plus the existing synthetic entry point.

This is reader-contract preparation, not real-data numerical conformance.
The real shipped-data R oracle is running separately and remains unaccepted.
No production algorithm, tolerance, public API or release scope is changed.

`files.sha256` identifies the exact source bytes; `checksums.sha256` covers
this receipt, the manifest and the source archive. Preserve the expected-red
run's true failure result and original logs before implementing the reader.
