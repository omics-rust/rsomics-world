# Table 0.2 delivery

Seven operations: validate, select, filter, sort, join, groupby and crosstab.
No new foundation, dependency or product repository. The implementation and
resource decision are in `table-crosstab-execution-2026-11-03.md` and the
owner's `benchmarks/2026-11-03/crosstab.md`.

## Source and native acceptance

Release source `2376972224f08de9a5b7cb2157019a0a64272cf5` passed exact-head
CI `41619468413`: four native classes, Linux benchmark artifact and Linux
full pinned oracle. Default profiles have 113 passed / 7 named oracle ignores;
the full oracle has 120 passed / zero ignored. Each native class exercises
debug, release and packaged-source tests, then installs the packaged binary
and completes 14 calls (13 successes and one intended failure).

Root freshly read the production/API surface and confirmed that the only Rust
change since measured/reviewed `463ad7c` is command visibility/help. No kernel
or extra aggregate behavior was changed during delivery. Root version changed
to 0.2.0; the external lock graph and 122 original license texts did not change.
The notice inventory's lock binding was updated for the root-version-only diff.

All four source archives are 115450 bytes, SHA-256
`51e19ab70da2cf5c028ef688bf681466772623f74e07c371ad023326faca3fe6`.
Each contains 185 exact Git originals plus original manifest/clean VCS metadata,
187 files total. Root independently verified source bodies/modes, normalized
metadata, enabled dependency graphs, carrier license bodies, ELF/Mach-O CPU
classes, complete logs and matrix values as data; no recovered binary was run
during original adoption. Original ZIP API digests are checked by collection.

Original packet:
`/Volumes/KIOXIA/Developments/tmp/table-02-originals-20261103.XDCN1P7u`.
Adoption receipt: `.autopilot/oracles/table-native-delivery-2026-11-03-v02.json`.
Local source install packet:
`/Volumes/KIOXIA/Developments/tmp/table-02-install-20261103.ejpdu0Si`.

## Public GitHub release and actual download

[v0.2.0](https://github.com/omics-rust/rsomics-table/releases/tag/v0.2.0),
release `432557746`, was published at `2026-11-03T10:37:12Z` with the four
adopted native carriers, common source archive and SHA256SUMS. The tag resolves
to the exact release source. All six uploaded asset IDs/sizes/API digests were
verified before publication. Old v0.1.0 tag, release body and asset identities
are preserved. These artifacts are not signed or notarized.

Anonymous fresh GETs of release metadata and all six assets return HTTP200;
every asset matches the adopted original hash. The downloaded source helper
executes the downloaded MacARM binary, not a local build, through all 14 calls.
Root separately rehashed six downloads, checked helper bytes against exact
release Git, and compared ten complete installed output files with Mac native
originals (gzip files compared after decompression). The lifecycle includes
quoted/multiline fields, 1/4-worker sort equality, duplicate join keys, gzip
groupby, matrix count/sum, positional matrix projection and atomic failure.

Release/public packet:
`/Volumes/KIOXIA/Developments/tmp/table-02-release-20261103.288hqca3`.
Receipt: `.autopilot/oracles/table-public-delivery-2026-11-03-v02.json`.
The first draft lookup by public tag returned404 because the draft had no
public tag route yet. Root found the existing draft by release-list ID,
verified its complete six assets and published that same draft. It was not
recreated, and no upload/tag/old release was overwritten.

Fresh anonymous sparse registry GET returns404, with no versions. No registry
token was read, created or changed; the known org-token authorization hold was
not retried. GitHub delivery is complete, crates.io publication is not.

README-only owner follow-up `8b55b70bc9a95c1c1ff998c39625046b76b7556c`
advertises the actual release without replacing its immutable tag or assets.
Its exact-head CI `41620971065` completed successfully, all six jobs. This
docs-only follow-up does not replace or requalify the immutable release bodies.

## Remaining portfolio

This closes the crosstab implementation/GitHub-delivery slice, not the complete
Table dossier or the 30-family goal. Other planned Table operations remain
undocumented until implemented. The next differentiated workflow frontier is
FASTQ preprocessing quality-window trimming, starting from the pinned fastp
oracle rather than the ambient different version. Registry authorization and
scientific/compatibility holds on other families retain their existing scope.
