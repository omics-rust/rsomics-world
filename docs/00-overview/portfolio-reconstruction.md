# rsomics portfolio reconstruction

Status: active architecture ledger and registry-reset record. Routing remains
provisional until each product family is reconstructed, but the 2026-07-30
registry reset was explicitly authorized and is tracked below.

The final live namespace and recovery check is recorded in
[`registry-reset-gate-2026-07-31.md`](registry-reset-gate-2026-07-31.md).

The generated crate-level ledger is
[`portfolio-inventory.tsv`](portfolio-inventory.tsv). Regenerate it with:

```bash
python3 scripts/portfolio_inventory.py
```

New consolidation outputs listed in
[`portfolio-consolidation-outputs.txt`](portfolio-consolidation-outputs.txt)
are excluded so creating `rsomics-bed` or another target repository does not
inflate the historical source-pool counts.

## Why this reconstruction exists

The previous partition rule treated one callable operation as one independently
published crate. That produced technically testable units, but confused a Rust
module boundary with a user-facing product, repository, CI, version, and
installation boundary.

The replacement rule is:

1. A Layer B crate is a coherent product or workflow family recognizable to a
   bioinformatics user.
2. Operations within one product are subcommands or modules.
3. Code used by only one target product remains internal to that product.
4. A Layer A crate exists only when at least two target products consume the
   primitive, or when an explicitly documented near-term product makes the
   second consumer concrete.
5. Correct implementations, compatibility goldens, and performance evidence are
   migration assets even when their current package boundary is rejected.

## Reconciled baseline

At the pre-reset snapshot, the inventories disagreed:

| Surface | Count |
|---|---:|
| Local directories with `Cargo.toml` | 622 |
| Local Git repositories | 608 |
| GitHub `rsomics-*` repositories excluding `rsomics-world` | 608 |
| crates.io packages with the prefix | 606 |
| Rows in `REGISTRY.md` | 603 |

Fourteen local candidates were not Git repositories. `rsomics-kstat` had a
manifest but no Rust source file. The old registry header still said 231
crates. These facts made that registry unsuitable as the source of truth.

These counts are the pre-reset evidence snapshot. Local clones remain in place,
so the generated implementation ledger continues to describe the recoverable
code pool even after remote repositories and crates.io packages are removed.

The 622 local candidates break down as follows after description-first routing:

| Intended container | Current candidates |
|---|---:|
| Coherent product families | 422 |
| Generic capability pools, not presumed products | 172 |
| Existing foundation libraries | 28 |

The routing confidence is high for 574 rows and medium for 48. High means the
package description names the upstream or the crate name has an unambiguous
format/workflow prefix. Medium includes explicit product-boundary corrections
whose provenance remains visible in the dossier.

`upstream_families` in the ledger is derived from the package description when
possible. `upstream_mentions` is broader and includes README/test mentions.
Keeping the columns separate prevents incidental text such as “networkx was run
inside a scanpy conda environment” from misclassifying a graph algorithm as a
Scanpy operation.

`git_head` and `worktree_state` record the exact local source snapshot used by
the routing pass. A dirty repository is never copied blindly: its ownership and
diff are resolved when that asset enters a product migration.

At the 2026-07-31 generated snapshot, 554 source repositories were clean, 54
were dirty, and 14 candidates were not Git repositories. Dirty does not mean
discardable: it means the migration must inspect and attribute the local diff
before selecting a source revision.

## Provisional product families

The current implementation pool maps to 30 accepted product families.
Rejected workflow-metadata and differential-expression-reporting candidates
remain in capability pools:

| Product family | Candidates | Intended boundary |
|---|---:|---|
| `rsomics-bed` | 42 | BED/interval suite |
| `rsomics-plink` | 42 | PLINK-style genotype analysis and genotype QC |
| `rsomics-bam` | 41 | SAM/BAM/CRAM format operations |
| `rsomics-seq` | 34 | FASTA/FASTQ sequence utilities |
| `rsomics-vcf` | 30 | VCF/BCF inspection, transformation, filtering, indexing, and format statistics |
| `rsomics-sc` | 29 | stateful single-cell analysis workflow |
| `rsomics-rnaseq-qc` | 21 | RSeQC/Picard RNA-seq QC |
| `rsomics-ecology` | 19 | community diversity, dissimilarity, ordination, association, and permutation analysis |
| `rsomics-edger` | 17 | edgeR workflow |
| `rsomics-popgen` | 14 | population variation, differentiation, admixture statistics, and selection scans |
| `rsomics-limma` | 16 | limma workflow |
| `rsomics-signal` | 15 | deepTools/bigWig signal workflows |
| `rsomics-table` | 16 | csvtk/datamash-style tabular suite |
| `rsomics-fastq-preprocess` | 12 | trimming, correction, UMI, deduplication |
| `rsomics-deseq` | 12 | DESeq2 workflow |
| `rsomics-phylo` | 11 | alignment trimming, evolutionary distance, tree inference, comparison, and measures |
| `rsomics-structure` | 9 | PDB and protein-structure analysis |
| `rsomics-composition` | 10 | compositional transforms, zero handling, and inference |
| `rsomics-index` | 5 | bgzip/tabix and sequence index utilities |
| `rsomics-metagenomics` | 5 | abundance-aware amplicon processing, taxonomic classification, and reports |
| `rsomics-peak` | 5 | chromatin peak calling, annotation, and quantification |
| `rsomics-count` | 4 | feature/read counting and length-aware count-matrix normalization |
| `rsomics-annotation` | 4 | GFF/GTF, transcript, and functional-consequence annotation |
| `rsomics-call` | 2 | alignment pileup, genotype likelihoods, and lightweight small-variant calling |
| `rsomics-cnv` | 2 | BAF/LRR copy-number and chromosome-level polysomy analysis |
| Five single-implementation products | 5 | FastQC, persistent sequence sketches, liftOver, methylation, and minimap2 |

This table is a routing result, not a declaration that every proposed family
must remain separate. The joint bulk-expression review retained DESeq2,
edgeR, and limma as distinct stateful products and rejected the generic
`rsomics-expression` boundary. The metagenomics/sketch review is complete:
persistent sketch artifacts, collections, and search remain a
separate product from exact amplicon and read-classification workflows.
The workflow review rejected `rsomics-workflow`: a private three-column sample
manifest is consumer policy, while a real workflow engine would be a new
product with no supporting implementation in the source pool.
The expression review moved count-matrix collation into `rsomics-count` and
kept simple significance-category annotation product-local.
The structure review consolidated nine coordinate-analysis binaries into
`rsomics-structure` and internalized `rsomics-pdb-core`: eight historical
dependents become one target-product consumer after consolidation.
The variant review split the upstream bcftools executable boundary by workflow:
format operations stay in `rsomics-vcf`, alignment-to-variant work forms
`rsomics-call`, BAF/LRR analysis forms `rsomics-cnv`, consequence operations
join `rsomics-annotation`, and genotype QC/LD/family operations join
`rsomics-plink`.

The official suite shapes provide operation inventories, but not every
executable is a product boundary:

- [samtools](https://www.htslib.org/doc/samtools.html) and
  [bedtools](https://bedtools.readthedocs.io/en/latest/content/bedtools-suite.html)
  expose mostly coherent command families over shared formats.
- [bcftools](https://samtools.github.io/bcftools/bcftools) mixes format
  operations, variant calling, copy-number analysis, functional annotation,
  and genotype analysis; those workflows are split across the reviewed
  products.
- [SeqKit](https://bioinf.shenwei.me/seqkit/usage/) groups FASTA/Q utilities
  behind one installation.
- [PLINK](https://www.cog-genomics.org/plink/2.0/general_usage) composes
  operations through flags in one workflow.
- [Scanpy](https://scanpy.readthedocs.io/en/stable/index.html) operates over a
  shared AnnData state rather than independent function-sized products.

## Capability pools

The following 172 Layer B candidates are not accepted as products merely
because they currently ship binaries:

| Capability pool | Candidates | Default disposition |
|---|---:|---|
| Statistical functions | 91 | Move reusable APIs into `rsomics-stats`; expose a CLI only for a coherent user workflow |
| Graph algorithms | 59 | Consolidate reusable graph representation/algorithms; do not publish one binary per NetworkX function |
| Generic ML transforms | 11 | Keep only consumers required by real omics products |
| Generic image functions | 8 | Quarantine until bioimage scope and real workflows are approved |
| Generic HMM decoder | 1 | Internalize with a concrete sequence-model product |
| Workflow metadata | 1 | Keep schemas with consuming workflows; do not publish a generic path checker |
| Differential-expression reporting | 1 | Internalize with a DE product only when it produces a real report |

This distinction prevents a correct SciPy or NetworkX reimplementation from
automatically becoming an rsomics product.

## Foundation decision after product-level collapse

Raw dependent-crate counts exaggerate public reuse. A foundation used by 22
PLINK micro-crates still has only one target-product consumer after those crates
merge.

Observed product-level reuse currently supports these public foundations.
Historical dependent counts are retained only as source-pool evidence; the
named products, not those raw counts, justify the boundary:

| Foundation | Historical crate consumers | Named initial product consumers |
|---|---:|---|
| `rsomics-common` | 560 | all 30 accepted products |
| `rsomics-help` | 317 | all 30 accepted CLI products |
| `rsomics-bamio` | 70 | `bam`, `call`, `count`, `methyl`, `minimap2`, `rnaseq-qc`, `signal` |
| `rsomics-intervals` | 11 | `bed`, `annotation`, `peak`, `signal` |
| `rsomics-kmer` | 6 | current: `seq`, `sketch`; concrete next review: `metagenomics` |
| `rsomics-seqio` | 8 | `seq`, `fastq-preprocess`, `fastq-qc`, `minimap2` |
| `rsomics-stats` | 3 | `composition`, `deseq`, `edger`, `limma`, `sc`, `ecology`, `popgen`, `plink` |
| `rsomics-phylo-tree` | 9 | `composition`, `phylo`, `ecology` |
| `rsomics-pileup` | 2 | `bam`, `call`, `methyl` |

The following current libraries have zero or one target-family consumer and
should default to internalization unless a second concrete product is found:

- `align-core`, `bbi`, `coverage-core`, `csvio`, `debruijn`, `distance`,
  `ebayes-core`, `fm-index`, `fqgz`, `hmm`, `igzip`, `models`, `pdb-core`,
  `pgen`, `popgen-core`, `seqstats`, `taxonomy`, `vcf-expr`, and `vcf-valfmt`.

This does not mean their APIs or tests are discarded. It means their code moves
under the sole consuming product instead of retaining a separately versioned
public package. `bbi`, `fm-index`, and similar primitives may remain provisional
public candidates when a named near-term second product is added to the map.

The observed dependency relationships are:

```mermaid
flowchart LR
    common["common"] --> products["nearly all target products"]
    help["help"] --> cli["30 CLI families"]
    bamio["bamio"] --> bam["bam"]
    bamio --> count["count"]
    bamio --> methyl["methyl"]
    bamio --> minimap2["minimap2"]
    bamio --> signal["signal"]
    bamio --> rnaqc["rnaseq-qc"]
    bamio --> call["call"]
    intervals["intervals"] --> bed["bed"]
    intervals --> signal
    intervals --> peak["peak"]
    intervals --> annotation["annotation"]
    kmer["kmer"] --> seq["seq"]
    kmer --> meta["metagenomics"]
    kmer --> sketch["sketch"]
    seqio["seqio"] --> seq
    seqio --> fastq["fastq-preprocess"]
    seqio --> fastqqc["fastq-qc"]
    seqio --> minimap2
    tree["phylo-tree"] --> composition["composition"]
    tree --> phylo["phylo"]
    tree --> ecology["ecology"]
    pileup["pileup"] --> bam
    pileup --> methyl
    pileup --> call
```

## Revised size estimate

The product-level dependency calculation makes the earlier Layer A estimate too
generous.

- Current implementation pool: 30 coherent products.
- Clearly justified shared foundations: 9.
- Strategic foundation candidates with concrete near-term second consumers:
  approximately 3–6.
- Rationalized current portfolio: approximately 40–46 crates, including the
  temporary compatibility dependency.
- After adding genuinely missing anchors such as short-read/spliced alignment,
  quantification, classification, and assembly: approximately 55–75 crates.

The number may change after the 48 medium-confidence rows and missing-anchor
priorities are reviewed, but the evidence does not support hundreds of public
crates.

The aggressive registry-reset allowlist is
[`registry-reset-keep.txt`](registry-reset-keep.txt). It contains the 30
provisional product-family names and nine foundations with demonstrated
cross-product reuse. Names not yet published reserve an intended boundary;
their absence does not justify retaining operation-sized predecessors.

## Registry reset

The reset is a namespace cleanup, not source-code destruction:

- All 595 crates.io candidates were archived with every published version,
  checksum, and archive validation before deletion began.
- All 597 GitHub retirement candidates were archived as complete Git bundles.
  Dirty local worktrees additionally have status records, binary patches, and
  untracked-file archives.
- The backups live under
  `/Volumes/KIOXIA/Documents/omics-rust/_retired/registry-reset-2026-07-30/`.
- Local clones under `/Volumes/KIOXIA/Documents/omics-rust/` are intentionally
  retained as the reconstruction source pool.
- 596 GitHub repositories were deleted and verified absent. The organization
  now retains the existing allowlisted repositories, `rsomics-world`,
  organization metadata, and the temporary compatibility dependency
  `rsomics-igzip`.
- 594 crates.io reset candidates were deleted in reverse topological order and
  verified through exact success responses. `rsomics-igzip` is temporarily
  protected because published `rsomics-seqio` versions depend on it.
- The all-yanked orphan `rsomics-bam` was archived and deleted separately after
  the reset. It had no live version, repository, local clone, or reverse
  dependency.

The completed reset leaves 11 published crates: ten allowlisted names with
non-yanked releases plus temporary `rsomics-igzip`. The other 27 accepted
product names are boundaries to reconstruct, not empty packages to publish
immediately.

Machine-readable progress and failure journals live under `.autopilot/state/`.
Only an exact success response is recorded as a deletion; rate-limited or
ambiguous requests stop the batch and remain pending.

## Reconstruction order

Remote cleanup does not determine implementation order. Reconstruction uses the
retained local clones and validated archives:

1. **Freeze partition growth.** Do not create another operation-sized crate or
   expand a foundation for the sake of the current micro-crate layout.
2. **Recover consolidation seeds from the source pool.** Treat `fasta-utils`,
   `fastq-utils`, `vcf-utils`, `gff-utils`, and `bed-utils` as implementation
   inputs, not as package boundaries that must be republished.
3. **Consolidate low-state format suites.** `table`, `seq`, `bed`, `vcf`, and
   `annotation`, preserving compat goldens and per-operation benchmarks.
4. **Separate format from workflow.** Keep samtools-like operations in `bam`;
   route RSeQC/Picard to `rnaseq-qc` and deepTools to `signal`.
5. **Consolidate stateful statistical workflows.** DESeq2, edgeR, limma, and
   Scanpy families move as complete data-model workflows.
6. **Consolidate domain analysis.** PLINK, population genetics, ecology,
   composition, phylogenetics, metagenomics, and structure.
7. **Resolve capability pools.** Promote only APIs with two target-product
   consumers; quarantine or remove product status from the rest.
8. **Publish only complete products.** A family is republished after its
   operation map, compatibility suite, public API boundary, and performance
   evidence pass review. Recreating an old micro-crate name is not a goal.

## Review gates

Before a provisional route becomes final:

- Confirm a real upstream tool/package or a documented rsomics-native workflow.
- Confirm the operation belongs to the target product's shared data model.
- Preserve every verified compatibility fixture and performance record.
- Recount dependencies at the target-product level.
- Require two target-product consumers before retaining a public foundation.
- Treat crates.io retirement as a separate, explicitly reviewed action.
