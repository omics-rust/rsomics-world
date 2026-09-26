# External fixture reader: native candidate verification

Green-4 run `36221334797` at exact world head
`0c96a8a9b1be882959cd5a784a1dc0c96fdf94e4` succeeded on Linux and macOS,
both x86_64 and aarch64. Matching Control plane run `36221298670` passed.
This verifies the test-only loader candidate, not completion of shipped-data
conformance or a new accepted product commit. Full Task 2 review is pending.

Each target passed 38 tests in debug and release: 13 private numerical tests,
16 product contract tests, 8 reader tests and the four-profile synthetic oracle
test. Each mode compared all 40 synthetic checkpoints; maximum absolute delta
remained `7.105427357601002e-15`. No test was ignored. Linux x86_64 formatting
and strict Clippy passed. The separate red-3 run `36220790633` captured the
missing reader API before implementation.

The controller verified terminal run/head/target conclusions, API artifact
digests, all ZIP CRCs and extracted bytes, exact snapshot archive/manifest
identity, 137 before/after source hashes, and unchanged Cargo.lock SHA-256
`636ead1961f573b303be8c02f6068201acd11dde1d0cb9bab3d81ccf86bf5554`.
Only `tests/support/mod.rs` differs from red-3. Production is unchanged from
accepted local product commit `13961bf6a42cb4cbbf36c528331aafb98a52937b`.

Raw evidence: external fixtures
`evidence/infercnv-native-core-2026-09-26/run-36221334797/`.

| Artifact | Original ZIP SHA-256 |
|---|---|
| Linux x86_64, ID 10899287237 | `d7746f60d4db2cb044104b50d595fd06a695501193f1eff7e1ccc4d312f80ff0` |
| Linux aarch64, ID 10898499720 | `e16be9e909cd416254cc335889d0576c9c3ceffac79cb064bb828f662cac1fc8` |
| macOS x86_64, ID 10899092573 | `983eb296d0fd902f6ee216ddcc26347a536f729bb8b06eaa23cdea8c6f05d95a` |
| macOS aarch64, ID 10899307153 | `dde8223ae8787e80188b03d316c4de2f248258ffdf597c89131a3013013f06f5` |
| Complete job logs | `429d4cc4e3b2fc0e2f10ff984d32638c7bf9f7cd8fb98fcf07ecf1cee64685de` |

No real shipped-data checkpoint was supplied to this candidate, no production
parser or public API was added, and no performance or release claim is made.
