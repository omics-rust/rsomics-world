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

The exact-head hosted audit must still run after all four native jobs.
Its result is a body-verification receipt, distinct from a local download,
public installation, performance approval, or publication. Do not retry
the unchanged registry credential. The metagenomics zero-read decision
and the separate sequence/BED semantic holds are unchanged.
