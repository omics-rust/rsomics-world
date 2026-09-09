# BCF record-boundary and split-magic repair candidate

Full four-native verification snapshot, not a product release. All 174 source
entries/hashes are verified. Exactly four source paths differ from the red:
`src/format/mod.rs`, `src/format/reader.rs`, `src/reheader.rs` and
`src/reheader/bcf.rs`. Both complete red CLI files are unchanged.

Expected-red run `34295859204`, world
`86869341898cf2b4932c7ebd91571c44c6ff39bd`, fails the concat and reheader
steps on all four native targets. Control CI `34295758040` passed. Linux ARM
raw assertions were read before edits at external scratch
`vcf-bcf-framing-first-red-bcCYpy/linux-aarch64.log`: concat 37 pass/one fail,
all 24 malformed invocations accepted, 12 named destinations replaced and
four naive stdout outputs emitted; reheader 19 pass/two fail, all 24 malformed
inputs accepted/replaced and all 16 legal split-magic inputs rejected.
Each consumer's six header-only controls pass. This focused run deliberately
skips full debug/release and oracle execution. Artifact auditing is separate.

A reader-private checked BCF function distinguishes actual decoded EOF before
calling Noodles. Available bytes followed by a zero return are invalid record
length, not EOF. Interrupted is retried; other helper-level I/O kinds and
partial-length errors propagate. Nonindexed compressed BCF and reheader's
decoded BCF use buffered readers to provide this contract; raw reheader reuses
its already-buffered input. Indexed queries are explicitly outside this fix.

Reheader now collects exactly three decoded magic bytes across frames before
classifying. Complete compressed prefix frames are replayed unchanged; empty
frames do not terminate classification. Only the magic array is bounded:
raw-prefix memory across arbitrarily many empty frames remains a follow-up.
Naive concat's separate full integrity pass has not been removed.

Three additional unit groups cover empty/zero/partial record lengths, helper
I/O kinds, and valid one-byte/Interrupted input under buffer sizes 1/7/8192.
These are additional tests, not observed-red evidence. The existing command
error wrapper still converts some input I/O errors to InvalidInput; helper
kind preservation is not claimed end-to-end. Added decoded buffering and
per-record checks require later throughput/RSS/many-input measurement.

Independent differentiated source reviews found no blocker for remote
verification. Rustfmt/diff and control-plane static checks pass. No Rust
tests/product binaries ran locally. Dispatch with `regressions_only=false`.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (338480 bytes) | `b4ba55961c195772e8acc84aea0e35088e31e3ca24bfcbf17c87e2aa8c754d23` |
| `files.sha256` | `0f5dc02442961241aa013b06ca0a36b10c1c3655b7e0383574458c17ec870ac7` |
| `tracked.patch` | `696b653b76868289738dfdf43fb22792a0046a51890f4f3450746e9a57deb61d` |
| `tests/concat_cli.rs` | `06d7a94b237ab77eec2cceb6e18d7f8fd6a6e14510fb6cfd01d7f70a0187ef8c` |
| `tests/reheader_cli.rs` | `a6cd1b893e7626385c03e290f0f99aa57f1aaee2bc8dd2747504799537768d18` |

Owning product HEAD and empty index remain
`682942cfa69768dc3a127a8544f2f07213b704ea`. No public API, dependency or
foundation change. Scratch/evidence are external; boot occupancy exceeds
95% and local Rust/product execution remains prohibited. Publication and
performance gates are open.
