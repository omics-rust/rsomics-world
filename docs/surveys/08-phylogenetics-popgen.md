# Survey: phylogenetics and population genetics

Verified 2026-07-31 against current upstream documentation, the generated
portfolio inventory, and the historical source trees. Product contracts live
in [`phylo.md`](../10-products/phylo.md) and
[`genotype-popgen.md`](../10-products/genotype-popgen.md).

## Retained products

| Product | Upstream behavior families | Historical assets | State |
|---|---|---:|---|
| `rsomics-phylo` | trimAl, distance/tree inference, comparison, and measures | 11 | dossier complete |
| `rsomics-plink` | PLINK 2 default plus named PLINK 1 legacy profiles | 31 | dossier complete |
| `rsomics-popgen` | scikit-allel 1.3.13, vcftools profiles, and published estimators | 14 | dossier complete |

The old source pool had split input-format variants into separate binaries.
The corrected ownership is:

| Operation | Canonical product |
|---|---|
| genotype counts, missingness, HWE, heterozygosity, sex and pedigree QC | `rsomics-plink` |
| LD pairs/matrix/pruning, blocks, GRM/KING, PCA, GWAS, scores | `rsomics-plink` |
| π, theta, Tajima's D, Dxy, fixed differences, and SFS | `rsomics-popgen` |
| FST, PBS, Patterson statistics, EHH/iHS/XP-EHH, Garud statistics | `rsomics-popgen` |
| VCF view/filter/norm/query/index | `rsomics-vcf` |

`rsomics-vcf-popgen` is split during migration: its FST and Dxy code goes to
`rsomics-popgen`, while its heterozygosity and haplotype-LD assets inform
`rsomics-plink`. The repository is not revived.

## Foundation consequences

- `rsomics-pgen` only implements PLINK 1 BED and has one product consumer. It
  is internalized in `rsomics-plink`.
- `rsomics-popgen-core` has one product consumer. It is internalized in
  `rsomics-popgen`.
- A policy-free FST kernel may enter `rsomics-stats` only after both retained
  products have concrete consumer tests.
- HWE and LD remain product-local until PLINK and scikit-allel policy
  differences are separated from a truly shared numerical contract.
- Both products use `rsomics-common` and the required `rsomics-help` layer.

## Evidence quality

The strongest historical population-genetics assets have committed
scikit-allel 1.3.13 or vcftools 0.1.17 goldens. The strongest PLINK assets have
committed PLINK outputs and some live differentials. They are not release
proof:

- several live tests return success when the oracle binary is absent;
- some PLINK comparisons check only IDs or tiny reports;
- PGEN is not implemented;
- window, accessibility, ploidy, multiallelic, founder, and chromosome
  policies are fragmented;
- most benches use tiny fixtures or skip when a private path is unset;
- every historical product lacks the four-native-platform CI gate.

These assets are classified as direct merge, refactor then merge,
test/fixture/benchmark only, or discard in the product dossiers.

## Broader landscape

ADMIXTURE/fastSTRUCTURE, ANGSD, RFMix, IBD segment callers, EIGENSOFT
qp-statistics, sgkit, and Hail remain survey references. They are not public
rsomics products until a separate boundary and source/evidence dossier
supports them. No name is published to reserve it.
