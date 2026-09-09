# Reheader BGZF extra-subfield expected-red candidate

Focused four-native source snapshot, not a product release. Only appended
`tests/reheader_cli.rs` content differs from
`vcf-sparse-oracle-empty-layout-2026-09-09`. All 174 archive entries and
hashes were verified; every old reheader test byte is retained. Production
code is unchanged. The preceding corrected-oracle full run `34293201835`
at world `e2f13662f2a19c0c308274f39b717bbf46e16319` is separate and still
being checked; its control CI `34293102709` passed.

The [pinned BGZF specification](https://github.com/samtools/hts-specs/blob/da617203a9527537746e200abda2885bec3a822c/SAMv1.tex#L846-L905)
permits additional RFC1952 subfields before and after `BC`. The existing
product frame parser accepts this, but reheader's initial classifier demands
XLEN=6 and `BC` first. Its Noodles BGZF 0.49 decompressor also assumes an
18-byte header. Fixing only the first classifier is therefore insufficient;
the product-level test covers the complete route without modifying it yet.

Two added groups bring reheader CLI tests from 17 to 19:

- 36 valid cases: VCF (12) and BCF (24), extra subfields before/after/both,
  optional leading noncanonical empty data frame, file path or stdin marker
  backed by FD0, and zero/two BCF compression workers (VCF zero only). BCF
  header bytes span frames. Independent gzip decoding verifies unchanged
  fixture payload and CRC; successful outputs must retain their binary/text
  magic, canonical terminal EOF, renamed samples, full record body and JSON
  encoding. All command failures are collected rather than stopping at the
  first case; failure must preserve the existing output destination.
- Four malformed cases: missing/duplicate BC, partial extra payload and
  header-frame CRC damage must fail without replacing output. Before repair
  these may all fail at the early classifier; passing guards do not prove
  four distinct downstream validation mechanisms ran.

This is not a pipe-cancellation or full output-encoding matrix. Independent
review found no fixture/API blocker; explicit decoded-format identity checks
were added after review. Rustfmt and diff checks pass; no tests have run yet.
Dispatch with `regressions_only=true`; reheader is a separate focused step.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (333815 bytes) | `33e29dde9e2cf18ecd2a344f166d2aad1d6f4db8fa969aa1bf03c50d0682fb2a` |
| `files.sha256` | `efdf5ae15f32d4111481e806de96b178f400bf1a81396df4224a350768c203ec` |
| `tracked.patch` | `f7b5202958fcd8ab0143bdad7bd0731249531d6bb806381e267f3be028886424` |
| `tests/reheader_cli.rs` | `d58d168be68f3725af43cf0b97a2534d1545ab07dcf11643de154f787eb69db4` |

Owning product HEAD/index remain unchanged at
`682942cfa69768dc3a127a8544f2f07213b704ea`. No public API, dependency or
foundation was added. No local Rust or product execution occurred; all
scratch and artifacts are external. No broad BGZF interoperability,
performance or release claim follows from this candidate.
