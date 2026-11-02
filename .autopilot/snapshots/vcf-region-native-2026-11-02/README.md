# VCF region-file four-native integration candidate

This is a frozen integration candidate, not a public release or release
qualification. The archive contains all 176 current owner file bodies at
`rsomics-vcf` base HEAD `682942cfa69768dc3a127a8544f2f07213b704ea`, including
the inherited uncommitted/untracked implementation and the adopted region-file
consumer/oracle increment. It is not a Git-HEAD-only archive. Member paths
remain relative to the owner root, and each member's original mode is retained.

The existing private validated-stream naive repair is included. Region-file
selection reuses the published `rsomics-intervals 0.4.0` parser through the
product-private adapter; Cargo.lock retains that exact version. No foundation
API, owner source, test or policy is changed by preparing this snapshot.

| File | Bytes | SHA-256 |
|---|---:|---|
| `source.tar.gz` | 360484 | `c196a5cf254ad62b2821f6447e1a20316c15065faa80e29f755c025e9ea51d6d` |
| `files.sha256` | 15963 | `2ceca46765c96f7a456010b398fc905424ab6b02b2680e47eae141c93e36fe53` |

`files.sha256` is byte-identical to the adopted 176-body manifest. Before
dispatch, every archive member body/mode and complete member set were checked
against the frozen owner manifest/modes; the owner index remained empty.
Protected inherited bodies remain:

- `src/format/reader.rs`: `d8bd41031553dcd9740cb2752988080b7d7eecc7fa7a55ee0758ca7abba4c7d4`
- `src/concat/naive.rs`: `3472c022c6dcd00fdc21cc0b54bcbc87055923dc2b0b06eb57d19dcd03fcc255`
- `tests/concat_cli.rs`: `67b504cb8ac1beb3c2d1f3c4f71802d76a4c0364bc6b49779ead1446707ba966`

Run the existing `vcf-candidate.yml` with `regressions_only=false` for the full
four-native integration attempt. Its native matrix, Rust 1.91.0, full debug/
release suites, strict Clippy/package checks and pinned Linux bcftools 1.24/
validation-corpus oracle gates are unchanged. The new region-file consumer
regression is already in the default full suite. The focused
`regressions_only=true` option remains available and skips full-release gates.

Each job retains this source TAR, manifest and README alongside existing
logs/dependency metadata. It also retains actual built binaries under their
original `debug/rsomics-vcf` and `release/rsomics-vcf` paths/modes in
`native-binaries.tar.gz`, with member and archive SHA-256 files. Focused-only
jobs retain debug only. These are integration evidence, not installable public
release assets or performance-qualified binaries.

The adopted local evidence is 452 ordinary passes/64 ignored in each profile,
plus explicit concat7/filter3/view4 passes per profile. This preparation does
not rerun or extend that evidence, and no native result is claimed in advance.
Missing-final-LF naive splicing and same-coordinate indexed ordering remain
explicit human/public-policy holds. Foundation non-UTF8 suffix inference,
representative performance/resource qualification and independent delivery/
registry gates remain unresolved. No old frozen 0.6 publication is authorized.
