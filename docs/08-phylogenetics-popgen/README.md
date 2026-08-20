# 08 — Phylogenetics & population genetics

> Multiple-sequence alignment, phylogenetic tree inference (ML, Bayesian,
> coalescent, ultra-large), and population-genetic analysis (PCA, admixture,
> ancestry, IBD).

Status: upstream landscape survey. Public product boundaries and implementation
decisions are authoritative in [`docs/10-products`](../10-products/README.md)
and the namespace allowlist. `Layer` and `rsomics-*` labels retained in the
sub-area surveys are research routing labels, not approved crates or
repositories.

## Sub-areas

| File | Scope |
|------|-------|
| [`alignment-msa.md`](alignment-msa.md) | Multiple-sequence alignment: MAFFT, MUSCLE5, Clustal Omega, T-Coffee, KAlign3, FAMSA, UPP, PASTA. |
| [`trees.md`](trees.md) | Tree inference and manipulation: RAxML-NG, IQ-TREE2/3, MrBayes, BEAST2, FastTree2, MEGA, PhyML, UShER, TreeShrink, ASTRAL. |
| [`population-genetics.md`](population-genetics.md) | Retained PLINK-style genotype analysis and population-genetics products, plus excluded future upstream families. |

## Design notes

- These domains are dominated by 15-25-year-old C/C++ codebases (RAxML,
  PLINK, MAFFT) with hand-tuned inner loops. Inner-loop performance is
  excellent; the gains from Rust are around **memory safety, type-safe
  model expressions, modern parallelism (`rayon`), and packaging** — not
  brute speed.
- The IO situation is comparatively pleasant: VCF/BCF/FASTA/Phylip/Newick/
  Nexus all have working Rust crates (`noodles`, `vcf`, `petgraph`-based
  tree libs). `phylotree`-style crates exist but no canonical Rust tree
  library has emerged yet — this is a real ecosystem gap.
- For phylogenetics: IQ-TREE2 is the broadly-preferred ML inference tool;
  RAxML-NG is its closest peer. Both have rich model selection + bootstrap
  + concordance-factor machinery. A Rust ML inference tool would need
  feature parity in model coverage to be adopted.
- For population genetics, PLINK2 is the genotype-analysis behavior anchor
  and scikit-allel is the array-level statistics anchor. The retained products
  need a measured throughput or resource advantage on declared hot paths;
  packaging or a language rewrite alone is not enough.
- License watch: MAFFT **BSD-3** + Clustal Omega **GPL-2**, MUSCLE5
  **GPL-3**, RAxML-NG **AGPL-3**, IQ-TREE **GPL-2**, MrBayes **GPL-3**,
  BEAST2 **LGPL-2.1**, FastTree2 **GPL-2**, PhyML **CeCILL/GPL**, UShER
  **MIT**, ASTRAL **Apache-2.0**, PLINK2 **GPL-3**, ADMIXTURE **proprietary-free**,
  EIGENSOFT mixed (BSD/Boost), vcftools **GPL**, Hail **MIT**, sgkit **Apache-2.0**.
