# Shared region-file suffix handling

Current: producer fix committed and four-native CI passed; no publication,
public API addition or consumer adoption is claimed. Owning clean baseline is
cda29c88b67882b63e1710a22aa480e905d4423f at
`/Volumes/KIOXIA/Documents/omics-rust/rsomics-intervals`.

The fully read parser converts the entire filename to UTF8 before matching
ASCII suffixes. A non-UTF8 prefix therefore silently selects generic tabular
coordinates instead of BED/VCF. Actual Call src/region_file.rs and CNV
src/selection.rs use this existing API with native Path values; VCF's newly
accepted consumer uses the same parser. Fix the producer once, not three
product-local suffix parsers.

Keep existing case-insensitive BED/VCF/.gz/.bgz matching and tabular fallback,
coordinate modes, compression/content detection and error models unchanged.
Compare only the native filename's ASCII suffix bytes; no lossy conversion,
new public item, dependency or operation. This runs once per file, not in the
record/geometry hot path; do not invent a throughput claim or a benchmark
campaign for filename classification.

Use one regression for the suffix contract, then existing full format/strict
Clippy/debug/release/package checks. Preserve inherited source except the
small classifier/test and intentional patch-version metadata. Extend existing
four-native CI with release tests and recorded actual host/compiler evidence,
not a new framework. Before any later consumer delivery, adopt the corrected
published version or explicitly pinned source and verify the consumer; current
published0.4.0 and the frozen VCF candidate remain unaffected by local edits.

All source, caches, tests and scratch stay physically external. Scratch:
`/Volumes/KIOXIA/Developments/tmp/intervals-filename-20261102.Uu40q5`.
No publication/token retry, consumer path dependency, destructive operation
or unrelated cleanup is authorized by this repair.

## Producer verification

Owning main c11a1e3d8e56ee8b4cc9792f49a079c946f163ac contains only the
classifier/regression, package0.4.1 metadata and existing native-CI extension.
The regression fails before the fix. After it, formatting, strict Clippy,
debug and release tests pass locally (9 unit plus2 integration each).
The first package attempt correctly refuses a dirty tree; after the reviewed
commit, clean cargo package --locked --offline succeeds without --allow-dirty.

One bounded integrated review reports no P1/P2 findings:
`/Volumes/KIOXIA/Developments/tmp/intervals-filename-review-20261102.iHSr1A/review.md`,
SHA23530184b5ea07e56e06556bf3a5b1db067e6dfc1c9b68bd2fc0c78b7eb8d67c.
No additional helper-test matrix or hot-path benchmark is warranted for this
once-per-filename classifier.

Actual exact-head CI41419073769 succeeds on Linux x86_64/aarch64 and macOS
x86_64/aarch64. All four host assertions, debug tests and release tests pass;
Linux x86_64 also passes fmt, strict Clippy and package. Current owner is clean.
This is producer qualification, not consumer adoption. Published0.4.0 and the
frozen VCF candidate still retain the prior filename behavior. Publication
authorization remains held; do not retry unchanged credentials.
