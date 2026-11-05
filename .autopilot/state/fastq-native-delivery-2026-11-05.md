# FASTQ 0.2 native delivery

FASTQ delivery-only owner commit `ca3a6fb73372de2631f0f1902f79f0ef1253e9d4`
extends the four native jobs with exact clean source verification, installation
from the extracted crate, a live fastp comparison on the installed binary,
and retained native carriers. There is no production, Cargo.lock, version,
benchmark or publication-workflow delta from `a0796ca`.

The CI pins the control-plane helper at
`f580be8705678234fd4b1abdcf9e56860f43b33f`, file SHA-256
`6ddc67040bc97f1f2e427e630dd90fdfd40e22532353223986fa7903f483a1b2`.
The helper verifies package bodies/modes against Git and checks normalized
Cargo metadata. The normal/build dependency notices come from registry archives
whose checksums match Cargo.lock, not from license labels alone. It is private
release tooling, not a dependency between products.
Control plane run41976325830 passed at that exact helper commit.

## Local exact-head replay

Packet:
`/Volumes/KIOXIA/Developments/tmp/fastq-native-delivery-20261105.U0p3ZH/exact-ca3a6fb.mNy9eM`.
Physical external storage and resolved Cargo target were checked before build.
Clean Mac ARM package: 46 original Git paths and 48 archive files, including
Cargo-generated members. The package verifier test went RED for the absent
verifier, then GREEN on exact and altered Git source. Control-plane suite:
543 tests, 2 skipped. FASTQ fmt, strict Clippy, rustdoc, debug and release
54-test suites pass locally with the pinned live fastp oracle.

The verified package installs with `cargo install --locked --offline --path`.
Fourteen installed CLI calls include one expected malformed-input failure.
Complete FASTQ output agrees with fastp for one/four-worker SE, Phred64 and
paired gzip. Composition, post-poly JSON and rollback also pass. The first
clean `a0796ca` package replay is retained separately; it is not the exact
delivery head. The bundle gate rejects an unrelated binary.

| Evidence | SHA-256 |
|---|---|
| Source crate, 57,620 bytes | `431a75e572e178c48eab44d4b99c5e819f5fb76a39ec48f59798152fd5115b1e` |
| Local Mac ARM tarball, 791,677 bytes | `904a6d34cdddbee64a2b0c2c4e2bfeeb326b68eca5726e71a9ab73af7bc745d4` |
| Installed binary, 1,865,920 bytes | `b5bfc71c9f2e37b15ea3883554bd22e4412653552823e482bc2649e9b8f503f9` |
| `SOURCE.json` | `1b8e5aa4feac4510ce8d9e791a681d7b8f1788b4e879525916a18640e77c3476` |
| `lifecycle/VERIFIED.json` | `5b48a63afcfa1518e7cc414191cc7a80d160fb59f2a311d8505473c70d7e5e59` |

Independent replay checks the tarball's binary and all 110 indexed original
notice bodies for 54 enabled packages, including the nested libdeflate C
`COPYING`. Recorded dynamic linkage uses only system libiconv and libSystem
on this Mac. The locked Linux graph has 54 packages and 112 original notices;
this cross-target inventory is not proof of a hosted Linux artifact.

One independent integrated review found no Critical/Important defect. It
noted that the bundler rechecks the notice index hash, but not every listed
file between source verification and bundling. No workflow step mutates those
files; defer the extra guard and verify the actual archived bodies at adoption.
The local archive independently passes that check. The review did not assert
hosted CI success, public installation or new performance.

## Hosted qualification

Owner main was pushed over SSH443. Exact-head run
[41976716505](https://github.com/omics-rust/rsomics-fastq-preprocess/actions/runs/41976716505)
passed on all four native targets, including exact source packaging, installed
CLI/fastp verification, carrier assembly and upload. Linux x86_64, Linux aarch64,
macOS x86_64 and macOS aarch64 job IDs are 127077158727, 127077158580,
127077158543 and 127077158499. Exact-head job/step conclusions and the four
artifact IDs, byte counts, API digests and expirations are recorded in
`.autopilot/oracles/fastq-native-delivery-metadata-2026-11-05.json`.
These are hosted installation results and upload metadata, not adoption of
the original artifact bodies.

## Publication gates

The public GitHub tag and release lookup both return 404 for v0.2.0. The
registry index still exposes only 0.1.0 and 0.1.1. No publication was attempted.
A fresh read-only GitHub API identity request returns HTTP 401. The anonymous
ZIP request for actual artifact13391887980 also returns 401. SSH pushes do not
confer artifact download or release API authority. Original carrier adoption
and fresh public installation are still held. The registry token hold is
independent, and the unchanged credential was not retried.
