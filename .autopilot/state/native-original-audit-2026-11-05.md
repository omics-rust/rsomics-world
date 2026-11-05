# Native original-artifact audit

FASTQ and metagenomics have successful exact-head four-native install runs.
Their anonymous original ZIP endpoints previously returned 401; public
metadata did not establish original bodies. The product-local audit path
uses the [workflow's same-repository token](https://docs.github.com/en/actions/tutorials/authenticate-with-github_token)
with Actions read permission. No new account grant, token value, release,
tag, or registry operation is needed to verify.

Plan: [native original audit](../../docs/plans/2026-11-05-native-original-audit.md).
The producer `scripts/native_delivery.py` remains byte-identical to
`f580be87`; the new verifier is a private consumer of its existing
FASTQ and metagenomics delivery contract.

## Evidence before hosted wiring

- All four retained real BED 0.6 ZIP hashes match their official artifact
  metadata. That establishes the transport digest contract, not FASTQ or
  metagenomics artifact contents.
- New behavior tests go RED on absent verification, then GREEN. They
  reject the wrong run/attempt, wrong ZIP or installed binary, changed
  dependency notice and unexpected carrier member. The independent
  review also finds nested crate decompression without a bound and
  reliance on producer-only normalized metadata. Both reproduce on
  deliberately changed archives and pass after separate validation.
- Real local Mac arm64 FASTQ and metagenomics source, carrier and
  installed receipt bodies replay with the new verifier. They retain 14
  and 8 native calls and 110 and 124 original dependency notices.
  The test repack changes only the carrier's build run/attempt context
  to synthetic local identifiers; it is explicitly not a hosted original.
  Final replay under physically verified external storage:
  `/Volumes/KIOXIA/Developments/tmp/native-original-audit-final-20261105.WKNB4R`,
  `LOCAL-REPLAY.json` SHA-256
  `a9e8755412372a77a6c0933afe686225c27339a97459e6d46d24229c080b3a5e`.
- Final local control-plane suite: 546 tests run, 2 skipped, no failures;
  architecture validation passes. Audit helper SHA-256
  `2f22b025bc20ce5e1f8485aa4f1b478f348daccada019854aa121353917a8c72`.
- The auditor checks SHA-256 and size before ZIP parsing, bounds nested
  archives, rechecks every Git source body/mode plus normalized Cargo
  metadata, and binds the binary and notice bodies to the original
  install/build chain. It does not execute recovered binaries or
  regenerate the native oracle or registry inputs.

The hosted body-verification receipt is distinct from a local download,
public installation, performance approval, or publication. Do not retry
the unchanged registry credential. The metagenomics zero-read decision
and the separate sequence/BED semantic holds are unchanged.

## Hosted execution

The shared helper is live at control-plane
`1c5c7b08002be0c0042f50a295d888cc3e281650`; exact-head
[CI 41991715211](https://github.com/omics-rust/rsomics-world/actions/runs/41991715211)
passed. FASTQ main `b6872a50c00309b86b78611cd88c707681d9ac7d`
changes only CI: attempt-qualified names, a post-matrix audit job, and
an audit receipt uploaded on success. Producer, product, benchmark,
version, and publication workflow are unchanged.

[FASTQ run 41992261637](https://github.com/omics-rust/rsomics-fastq-preprocess/actions/runs/41992261637)
completed successfully at that exact head, attempt 1. All four native
install jobs passed. The dependent [original-body audit job](https://github.com/omics-rust/rsomics-fastq-preprocess/actions/runs/41992261637/job/127129195868)
downloaded and verified the four uploaded ZIPs and uploaded its audit receipt.

| Artifact | ID | ZIP SHA-256 from GitHub |
| --- | --- | --- |
| Linux x86_64 | 13397844581 | `a0d05548a5863ed549e30bbc0495475b81478fdab4fb205d2c19cca0ea188e29` |
| Linux aarch64 | 13398468012 | `127ebe999c9a449b75eada48021caf70f9e2702d0b21f4345ac7436471d66d33` |
| macOS x86_64 | 13398758261 | `d47ea41211a51cd1b1a893bd83cf8322655c15f87c462de45e39a81e28527359` |
| macOS aarch64 | 13398189465 | `e00f2c49eaf1c49402787eda67c386fa639c94260d62d149fd585d14019fbb2a` |
| Audit receipt carrier | 13398768414 | `f3749353ec8347cec2d57c00979593885baad4173d1c94e67db58329debb3efd` |

The external packet above also retains the public run, jobs and artifact
API responses. `FASTQ-HOSTED-AUDIT.json` binds their observed result to
the run, attempt and helper revision; its SHA-256 is
`5b96c239eaa0a0b505cd9d25bde72f6a94f426b5b498f9eda4d33c13580741c9`.
It is a local observation of hosted verification, not the hosted
`AUDITED.json` body. No original ZIP was downloaded to this Mac, and no
release or registry publication was attempted. Metagenomics has only
the local helper replay, not this hosted original-body result.
