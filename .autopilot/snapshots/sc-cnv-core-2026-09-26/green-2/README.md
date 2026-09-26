# Native CNV candidate: green-2

Follow-up to frozen green-1 run 36218401374. That candidate passed all
23 tests in debug and release on four native targets, but strict Clippy
failed two manual-is-multiple-of diagnostics. Independent task review also
required three additional edge witnesses; green-1 was not accepted.

This snapshot changes only `src/cnv/config.rs`, `src/cnv/normalize.rs`,
`src/cnv/smooth.rs`, and `tests/cnv_contract.rs`: checked budget arithmetic
is isolated for overflow tests; successful post-filter depth normalization
and the 100-gene linear smoothing case have independent expected values;
both Clippy diagnostics use the standard integer method. There are 27 tests.

Dependency lockfile, fixture TSVs, and golden tolerances remain unchanged.
`files.sha256` identifies the entire frozen source; `checksums.sha256`
identifies this receipt, that manifest, and the archive. Native execution
and scoped re-review are pending. No public repository, CLI, registry
publication, complete workflow parity, or speedup is claimed.
