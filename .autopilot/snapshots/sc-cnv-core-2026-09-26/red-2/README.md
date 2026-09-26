# Native CNV final-review regression: red-2

Production behavior is unchanged from green-2. This test-only snapshot adds
two exact-bit private median regressions and a public stage-3 observer test
for smallest positive subnormal depths. Current median arithmetic silently
rounds valid subnormal midpoints incorrectly; these assertions must fail
before the production correction. Floating-point epsilon comparisons are
deliberately not used for these witnesses.

Only `src/cnv/normalize.rs` (test module) and `tests/cnv_contract.rs` differ
from green-2. The existing remote-generated Cargo.lock remains unchanged.
The workflow now preserves an existing lock during expected-red runs and
uses `--no-fail-fast` so both the private and public test targets execute.
Failure status remains real; no failure is converted to success.

`files.sha256` identifies source bytes; `checksums.sha256` identifies this
receipt, that manifest, and the archive. This is unpublished prepared-input
core evidence, not a full workflow or release.
