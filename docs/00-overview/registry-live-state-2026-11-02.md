# Public namespace snapshot — 2026-11-02

Observed through14:54:57 UTC. Root independently verifies all233 retained
manifest identities and replays the39 crate/repository and30 product-release
metadata records. This is a public availability snapshot, not a quality gate,
asset download, private-repository inventory or registry publication attempt.

Evidence packet:
`/Volumes/KIOXIA/Developments/tmp/portfolio-live-20261102.nCWPGZ`.
TABLE.json SHA256
`d19bec0b59d00c5767e654f290a470ac7c5d5ceba591f6d69b2196525d82f1ba`.
All76 requests return200/404; no429 or transport failure. Public organization
inventory has36 repositories, including33 of39 allowlisted names. All33
allowlisted public repositories use main and are not archived.

## Products

`live` requires a public repository and non-yanked crates.io version.
`repo-only` can include a GitHub binary release. `planned` here means no
observed public endpoint, not no private implementation. A GitHub404 does not
prove deletion or absence of a private repository.

| Product suffix | Public GitHub repo | crates.io max version | Latest GitHub release | Public status |
|---|---|---|---|---|
| annotation | yes | 0.1.0 | 404 | live |
| bam | yes | 0.29.0 | 404 | live |
| bed | yes | 0.1.0 | v0.2.0 | live |
| call | yes | 0.1.3 | 404 | live |
| cnv | yes | 0.1.0 | 404 | live |
| composition | yes | 0.1.0 | 404 | live |
| count | yes | 0.1.0 | v0.1.0 | live |
| deseq | yes | 404 | v0.1.0 | repo-only |
| ecology | yes | 404 | v0.1.0 | repo-only |
| edger | 404 | 404 | 404 | planned |
| fastq-preprocess | yes | 0.1.1 | 404 | live |
| fastq-qc | yes | 0.1.0 | 404 | live |
| index | yes | 404 | 404 | repo-only |
| liftover | yes | 0.1.0 | 404 | live |
| limma | yes | 404 | v0.1.0 | repo-only |
| metagenomics | yes | 0.1.0 | 404 | live |
| methyl | yes | 0.1.0 | 404 | live |
| minimap2 | yes | 0.1.0 | 404 | live |
| peak | 404 | 404 | 404 | planned |
| phylo | yes | 0.1.0 | 404 | live |
| plink | 404 | 404 | 404 | planned |
| popgen | yes | 404 | v0.1.0 | repo-only |
| rnaseq-qc | yes | 404 | v0.1.0 | repo-only |
| sc | 404 | 404 | 404 | planned |
| seq | yes | 0.2.0 | v0.3.0 | live |
| signal | 404 | 404 | 404 | planned |
| sketch | yes | 0.1.0 | v0.1.1 | live |
| structure | 404 | 404 | 404 | planned |
| table | yes | 404 | v0.1.0 | repo-only |
| vcf | yes | 0.5.0 | 404 | live |

Product totals:17 live,7 repo-only,6 planned. Twenty-four public product
repositories,17 registry identities and10 latest GitHub releases are distinct
counts. Each observed GitHub release has six assets; names, target_commitish
and timestamps are retained, but no asset or tag target is revalidated here.

## Foundations

All nine have public repositories and non-yanked registry versions.
Foundation GitHub release endpoints were not part of this product-release
query; do not infer their presence or absence from the table.

| Foundation suffix | crates.io max version |
|---|---|
| bamio | 0.8.4 |
| common | 0.12.3 |
| help | 0.4.0 |
| intervals | 0.4.0 |
| kmer | 0.2.2 |
| phylo-tree | 0.2.0 |
| pileup | 0.9.0 |
| seqio | 0.6.0 |
| stats | 0.2.4 |

The temporary igzip compatibility crate is outside this39-name long-term
allowlist. No new name is reserved, published, deleted or yanked by this audit.

## Authentication and evidence boundaries

Fresh secret metadata still gives CARGO_REGISTRY_TOKEN updated
2026-09-08T20:26:08Z, selected visibility. No secret value was read and the
unchanged failed credential was not retried. Registry and GitHub delivery
remain independent. Published identities do not prove current main's scope,
native correctness, performance, licensing or release readiness.

The external TSV export and an attempted duplicate export have identical
bytes/hash and valid40 lines/12 columns: root initially misread Python's
escaped bytes representation as literal backslash delimiters. The preliminary
CORRECTION.json reason is therefore not adopted. Original metadata/manifest
never changed; TABLE.json and independently replayed raw APIs are authority.
