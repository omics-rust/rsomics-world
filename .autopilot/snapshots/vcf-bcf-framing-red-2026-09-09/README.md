# BCF record-boundary and split-magic expected-red candidate

Focused four-native test-only snapshot, not a release. All 174 source entries
and hashes are verified. Only appended content in `tests/concat_cli.rs` and
`tests/reheader_cli.rs` differs from the preceding fully verified green
`vcf-reheader-extra-fix-2026-09-09` snapshot (run `34294546912`, world
`8c45d22737fb5fb7899840a4dac0983d99584ea1`, control `34294508570`). Both
complete prior test-file prefixes are byte-identical. Production is unchanged.

The [official BCF 2.2 site layout](https://samtools.github.io/hts-specs/VCFv4.5.pdf#page=41)
requires site fields within `l_shared`; zero length cannot represent a valid
record. Pinned Noodles 0.88, however, returns the same zero for physical EOF
and an encoded zero `l_shared`. The product's nonindexed reader and reheader
loops currently accept that as EOF. Separately, reheader's first nonempty
frame classifier cannot recognize BCF magic split after byte one or two.
These are source hypotheses until this candidate runs.

Three new groups cover:

- Concat: 24 malformed invocations (four zero placements, ordered raw/ordered
  BGZF/naive BGZF, stdout/named output) plus six valid header-only controls.
  Compatible fixtures are independently parsed to locate record boundaries;
  BGZF is rebuilt after mutation and round-trip checked. Naive must fail
  before any stdout; ordered streaming stdout is not required to roll back.
  Every named destination must remain unchanged on failure.
- Reheader: 24 malformed invocations (four layouts, raw or BGZF with zero/two
  workers, path/FD0 input) plus six header-only controls. Pure four-byte zero
  words at the physical decoded end must fail too, so draining later bytes
  cannot substitute for record validation. Successful header-only output is
  independently read with Noodles and must have no record bytes immediately
  after its header, before any record read.
- Reheader split magic: 16 valid cases (split one/two, optional empty frames,
  path/FD0, zero/two workers), exact decompressed fragments, complete typed
  body/sample/encoding/JSON checks and failure-output preservation.

Failures are collected across cases before terminal assertions. Concat has
38 groups and reheader 21. Rustfmt and diff checks pass; no Rust tests or
product execution ran locally. Dispatch with `regressions_only=true`. Read an
actual assertion failure before repairing either contract. Indexed query
paths remain outside this slice and need explicit chunk/index fixtures.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (337573 bytes) | `0275f39050b2bfbb23df3fc15b675ccb585be372e569db946667cc303bd33133` |
| `files.sha256` | `d0ad1671e7c6c82a012e0c4b1dd6d20022bd6670be8566e42cc37383a88586a8` |
| `tracked.patch` | `6b2d7f7bf33b5c02323c9647d49c40f4a461c76d70be6be1889071e357479e6b` |
| `tests/concat_cli.rs` | `06d7a94b237ab77eec2cceb6e18d7f8fd6a6e14510fb6cfd01d7f70a0187ef8c` |
| `tests/reheader_cli.rs` | `a6cd1b893e7626385c03e290f0f99aa57f1aaee2bc8dd2747504799537768d18` |

Owning product HEAD and empty index remain at
`682942cfa69768dc3a127a8544f2f07213b704ea`. No public API/dependency or
foundation changes. Local boot occupancy is above 95%; scratch/evidence stay
external. This candidate does not close performance or publication gates.
