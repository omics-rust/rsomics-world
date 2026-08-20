# Public namespace live state

Status: verified on 2026-08-20 CST after the index dossier correction.

This is a current-state snapshot, not a rewrite of the 2026-07-31 registry
reset gate. The reset archives and historical source inventory remain
unchanged.

## Summary

| Surface | Product families | Foundations | Temporary | Control plane | Total `rsomics-*` |
|---|---:|---:|---:|---:|---:|
| crates.io with a non-yanked stable release | 17 | 9 | 1 | 0 | 27 |
| GitHub repositories | 19 | 9 | 1 | 1 | 30 |

The two repository-only products are `rsomics-index` and `rsomics-table`.
Eleven accepted products remain planning-only and have neither a GitHub
repository nor a crates.io package. No unexpected operation-sized name is live
on either surface.

## Published products

| Product | crates.io version |
|---|---:|
| `rsomics-annotation` | 0.1.0 |
| `rsomics-bam` | 0.29.0 |
| `rsomics-bed` | 0.1.0 |
| `rsomics-call` | 0.1.3 |
| `rsomics-cnv` | 0.1.0 |
| `rsomics-composition` | 0.1.0 |
| `rsomics-count` | 0.1.0 |
| `rsomics-fastq-preprocess` | 0.1.1 |
| `rsomics-fastq-qc` | 0.1.0 |
| `rsomics-liftover` | 0.1.0 |
| `rsomics-metagenomics` | 0.1.0 |
| `rsomics-methyl` | 0.1.0 |
| `rsomics-minimap2` | 0.1.0 |
| `rsomics-phylo` | 0.1.0 |
| `rsomics-seq` | 0.2.0 |
| `rsomics-sketch` | 0.1.0 |
| `rsomics-vcf` | 0.5.0 |

`rsomics-vcf` 0.6.0 is prepared at
`682942cfa69768dc3a127a8544f2f07213b704ea`. Its release workflow
`32244558404` reached `cargo publish` and failed with crates.io 403
authentication; the 0.6.0 package is not live.

## Published foundations

| Foundation | crates.io version |
|---|---:|
| `rsomics-bamio` | 0.8.4 |
| `rsomics-common` | 0.12.3 |
| `rsomics-help` | 0.4.0 |
| `rsomics-intervals` | 0.4.0 |
| `rsomics-kmer` | 0.2.2 |
| `rsomics-phylo-tree` | 0.2.0 |
| `rsomics-pileup` | 0.9.0 |
| `rsomics-seqio` | 0.6.0 |
| `rsomics-stats` | 0.2.4 |

`rsomics-igzip` 0.1.0 remains the sole temporary public dependency.

## Repository-only products

- `rsomics-index`: exact-head four-native CI is green. Publication remains
  gated by a valid current-head performance run and registry credentials.
- `rsomics-table`: release candidate
  `2bd0fd3698c152bb27e7d0d7635d51fb41655112` passed exact-head CI
  `32314817480`; publish run `32321555568` failed at the registry credential
  gate.

## Planning-only products

- `rsomics-deseq`
- `rsomics-ecology`
- `rsomics-edger`
- `rsomics-limma`
- `rsomics-peak`
- `rsomics-plink`
- `rsomics-popgen`
- `rsomics-rnaseq-qc`
- `rsomics-sc`
- `rsomics-signal`
- `rsomics-structure`

These names remain dossier-backed boundaries. Their absence from GitHub and
crates.io is intentional and no placeholder repository or package is needed.

## Verification method

GitHub names, default branches, archive state, and push timestamps were read
from the `omics-rust` organization repository API. crates.io names and current
stable versions were read from the public crates search API and filtered to
the exact `rsomics-` prefix. Local heads for the active BAM/variant wave were
fetched over HTTPS and compared with their GitHub `main` heads.

No repository, package, version, or owner state was changed during this
verification.
