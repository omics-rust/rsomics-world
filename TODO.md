# rsomics current work

This is the executable control-plane checklist. Product-specific operation
maps belong in their dossiers under `docs/`.

Current execution ruling (2026-10-23): the active goal covers the whole
portfolio, not only inferCNV. Prioritize functioning integrated workflows,
then measured throughput, memory and useful parallelism. Retain existing
tests and critical compatibility/error checks, but consolidate validation and
fresh review at workflow/release boundaries rather than starting a separate
RED campaign, provenance audit and review for every small component. The
four-native release gate, storage rules and no-placeholder rule still apply.
See [current execution state](.autopilot/state/portfolio-execution-2026-10-23.md).

Current frontier (2026-11-02): Popgen, DESeq, RNA-seq QC, limma, table, ecology and BED0.2 have
earned GitHub native releases and fresh actual public-install lifecycles;
registry authorization remains separately held. Each performance result stays
bound to its measured implementation/binary, with losses and scope retained.
Ecology's alpha/rarefaction and all eight complete-study beta modes now have
accepted correctness/resource/original evidence. Its reviewed public delivery
passes exact four-nativeCI41341218191 with complete originals adopted.
GitHubv0.1.0 and the fresh public Mac14-call lifecycle pass; post-release
README7f27eb01 passes exactCI41344601496. Registry remains separately held.
PLINK's historical green7 prepared-reader/LD qualification and failed whole/
stage material-performance profiles remain retained, not a release gate pass.
Green15 native/source originals are adopted; its subsequent arena comparison
shows no material cohort RSS benefit. The maintained SHA-256 candidate223e342f
now passes green16 run41558979368/1 on all four native jobs and its same-attempt
upstream consumer. Complete originals and the12-call matched measurement are
root-adopted, but wall median is6.21percent higher and RSS effectively unchanged.
The upgrade is withdrawn: private da64da53 restores the exact green15 source
tree. Preserve the negative candidate evidence; no unchanged rerun or release.
See `.autopilot/state/plink-sha2-backend-2026-11-03.md`.
Sketch's corrected adoption at eb1b1510 has completed native4 originals,
all eight current-binary resource jobs, GitHubv0.1.1 and the fresh public
11-call lifecycle. All96 measured pairs use less RSS;42 wall losses remain,
so this is a resource advantage rather than universal faster throughput.
Documentation head ddc4e76c has passed exact CI41368042858. Registry delivery
remains separately held; do not repeat the accepted native/resource/install
campaigns. See `.autopilot/state/sketch-corrected-native-delivery-2026-11-02.md`.
Index's corrected whole41327532095 has complete correct originals adopted:
three serial primaries pass throughput, binary compression alone fails the
material-advantage gate. A new implementation/profile is not yet justified;
retain the negative result and advance other products without an unchanged rerun.
Signal's approved arithmetic/data-loss/filter repairs are already implemented
and four-native/source qualified. Its mandatory summary region-order decision
remains pending; the optional matrix sorted-BED anomaly is separate, not the
cause of the failed NPZ comparison. Structure torsion, edgeR QL numerical policy, inferCNV Leiden quality
and Peak sparse-default policy keep their explicit holds; other work continues.
No pending human decision is silently resolved or used to block the portfolio.
Exact gates and original receipts live in each product dossier/state file.
No unchanged accepted source/native/performance campaign needs replay. All
project writes/build/cache/scratch remain external; global boot occupancy is
no longer a stop condition.

## P0.1 — finish the architecture reset

- [x] Freeze the 30-product and nine-foundation allowlist after rejecting the
      planning-only workflow and expression-utility boundaries and separating
      variant calling and copy-number analysis from VCF format operations.
- [x] Back up all crates.io and GitHub retirement candidates.
- [x] Delete 594 dependency-ordered crates.io reset candidates.
- [x] Delete 596 GitHub reset candidates.
- [x] Back up and delete the all-yanked orphan `rsomics-bam`.
- [x] Replace the operation-per-crate instructions and conventions.
- [x] Regenerate `REGISTRY.md` from live GitHub, crates.io, and the allowlist.
- [ ] Convert stale domain documents from crate queues into product-operation
      surveys.
      The [data-structure](docs/01-foundations/data-structures.md) and
      [compression](docs/01-foundations/compression.md) surveys and
      [I/O format map](docs/01-foundations/io-formats.md) now record consumer
      ownership and adoption gates. The [preprocessing](docs/02-genomics/preprocessing.md),
      [methylation](docs/05-epigenomics/methylation.md),
      [peak-calling](docs/05-epigenomics/peak-calling.md) and
      [quantification](docs/03-transcriptomics/quantification.md) surveys now distinguish
      current product operations from retired micro-crate assets and historical
      upstream observations; other domain pages remain.
- [x] Record the final reset gate and exact live counts.

## P0.2 — product dossiers

Each dossier must include upstream scope, operations, deduplication, historical
assets, target modules, shared consumers, compatibility, performance, and
exclusions.

- [x] `rsomics-annotation`
- [x] `rsomics-bam`
- [x] `rsomics-bed`
- [x] `rsomics-call`
- [x] `rsomics-cnv`
- [x] `rsomics-composition`
- [x] `rsomics-count`
- [x] `rsomics-deseq`
- [x] `rsomics-ecology`
- [x] `rsomics-edger`
- [x] `rsomics-fastq-preprocess`
- [x] `rsomics-fastq-qc`
- [x] `rsomics-index`
- [x] `rsomics-liftover`
- [x] `rsomics-limma`
- [x] `rsomics-metagenomics`
- [x] `rsomics-methyl`
- [x] `rsomics-minimap2`
- [x] `rsomics-peak`
- [x] `rsomics-phylo`
- [x] `rsomics-plink`
- [x] `rsomics-popgen`
- [x] `rsomics-rnaseq-qc`
- [x] `rsomics-sc`
- [x] `rsomics-seq`
- [x] `rsomics-signal`
- [x] `rsomics-sketch`
- [x] `rsomics-structure`
- [x] `rsomics-table`
- [x] `rsomics-vcf`

Rejected public boundary:

- [x] `rsomics-workflow`: the only candidate is a product-specific sample-sheet
      checker, not a workflow engine or coherent utility product.
- [x] `rsomics-expression`: its two candidates belong to `rsomics-count` and
      product-local differential-expression reporting.

## P0.3 — public-foundation audit

- [x] Complete the initial source and API audit for all nine retained
      foundations.
- [x] `rsomics-common`: narrow the runtime through real product consumers.
- [x] `rsomics-help`: replace duplicate CLI models with the shared Clap layer.
- [x] `rsomics-seqio`: derive strict stream contracts from sequence and
      preprocessing.
- [x] `rsomics-kmer`: correct the published codec boundary and add a
      four-native-target maintenance-release gate.
- [x] `rsomics-kmer`: verify `seq` and `sketch` as two current target-product
      consumers, with concrete call sites and exact-head CI.
- [ ] `rsomics-kmer`: deliver the canonical Murmur short-input repair.
      Candidate 0.2.3 passed native consumer and scoped performance gates;
      publication failed registry authentication, so the fixed dependency is
      not delivered. See the [consumer recheck](docs/01-foundations/kmer-consumer-review-2026-09-09.md).
- [ ] `rsomics-kmer`: review public items individually before API expansion;
      two products using different APIs do not prove each item is shared.
- [x] `rsomics-intervals`: retain only the two-consumer coordinate model and
      move checked indexing and BED policy into `rsomics-bed`.
- [x] Reconstruct `rsomics-bamio` and `rsomics-pileup` through BAM and calling.
- [ ] Reconstruct `rsomics-stats` through at least two stateful workflows.
- [ ] Reconstruct `rsomics-phylo-tree` through phylogenetics and ecology.
- [ ] Internalize or replace `rsomics-igzip`, then remove its public registry
      boundary when immutable dependencies allow.
- [ ] Correct repository metadata in future foundation releases.

## P0.4 — current product correctness repairs

- [ ] `rsomics-sketch`: deliver signature scale recovery and collection-wide
      comparison sampling, separately from the kmer dependency repair; see the
      [product contract recheck](docs/10-products/sketch-contract-review-2026-09-09.md).

## P1 — sequence pilot

- [x] Complete the `seq/fastq` historical asset classification.
- [x] Define the `rsomics-seq` subcommand surface.
- [x] Define preprocessing versus QC boundaries for FASTQ.
- [x] Create or recover target product repositories on KIOXIA.
- [x] Merge the first consumer slice without reviving micro-crate dependencies.
- [x] Consolidate golden and compatibility tests for the implemented operations.
- [x] Add end-to-end tests composing multiple subcommands.
- [x] Benchmark implemented slices against the real upstream tools.
- [x] Review `common` and `seqio` public API changes through the `seq` and
      `fastq-preprocess` consumers.
- [x] Review `help` through sequence, preprocessing, BED, and annotation.
- [x] Review `kmer` through the actual `sketch` second consumer; the short-input
      repair and per-item public-API gate remain open above.
- [x] Publish the coherent five-command `rsomics-seq` first-release slice.
- [x] Publish the coherent `run`/`trim`/`filter`
      `rsomics-fastq-preprocess` first-release slice.

## P1 — interval pilot

- [x] Complete the `bed/annotation/index` historical asset classification.
- [x] Define the `rsomics-bed` subcommand surface.
- [x] Separate annotation semantics from generic interval algebra.
- [x] Separate indexing/compression concerns from BED operations.
- [ ] Recover `rsomics-bed-utils`, `rsomics-gff-head`, and `rsomics-gff-merge`
      from verified Git bundles where local clones are absent.
- [x] Merge the first release slice.
- [x] Consolidate compatibility fixtures and benchmarks.
- [x] Review `common`, `help`, and the interval coordinate model through BED
      and annotation.
- [x] Publish the coherent four-command `rsomics-annotation` first-release
      slice.
- [ ] Review sequence-index I/O through the `rsomics-index` consumer.
      The [source recheck](docs/01-foundations/indexing.md) confirms current
      BGZF/tabix does not consume `seqio`; future `faidx` and annotation
      adapter tests are required before sharing a reference-access API.

## P1 — coordinate conversion

- [x] Replace the inherited narrow liftover path with strict chain parsing,
      typed mapping outcomes, and transactional BED3-6/BED12 conversion.
- [x] Verify the implemented slice against pinned official UCSC binaries,
      representative synthetic topologies, and a real `hg38ToHg19` chain.
- [x] Publish `rsomics-liftover 0.1.0` with `common`, `help`, and `intervals`
      as its only public rsomics dependencies.
- [ ] Add the reference-aware VCF slice only after its allele, normalization,
      rejected-record, and modern-oracle contract is complete.

## P2 — alignment and variation

- [x] Recover and classify the archived `rsomics-bam 0.1.0/0.2.0` packages
      as test and benchmark assets.
- [x] Classify all BAM and VCF historical implementations.
- [x] Define BAM/VCF format-operation boundaries versus RNA-seq QC and signal.
- [x] Review `bamio` through the first BAM consumer and keep the provisional
      reader backend product-private until a second consumer exists.
- [x] Harden the provisional `pileup` ingestion and projection contract with
      checked sorted input, long-CIGAR coverage, overlap differentials, and
      four-native-target CI before product integration.
- [x] Review `pileup` through BAM, calling, and methylation consumers.
- [ ] Review `intervals` through BAM region selection without moving
      alignment-index policy into the foundation.
- [x] Reconstruct and publish the coherent `rsomics-vcf` first-release slice.
- [ ] Transfer the verified VCF index-selection and byte-path fixes into the
      owning product's release candidate; resolve the defective unpublished
      0.6.0 head before any publication retry.
- [ ] Complete the VCF concat slice with bounded indexed ingestion, validated
      two-pass naive mode, the missing oracle matrix, and many-input and
      many-sample resource/performance evidence.
- [x] Reconstruct and publish the coherent three-command `rsomics-call`
      first-release slice.
- [x] Publish the gate-complete coherent `rsomics-bam` first-release slice
      after the crates.io name cooldown.

## P3 — stateful workflows; inferCNV prioritized by the user

- [x] Reject the historical five-step approximation as an inferCNV replacement;
      retain only useful parser/fixture assets in the single-cell dossier.
- [x] Execute and independently audit the pinned real-package pre-clustering
      oracle, synthetic and shipped data, rather than infer behavior from names.
- [x] Accept the unpublished prepared-counts numerical core through step 14
      on all four native targets with unchanged checkpoint tolerances.
- [x] Measure the prepared-only region on a matched host. The bounded 41.37x
      result excludes ingestion, clustering, CNV calling, reports and release.
- [x] Accept canonical creation witnesses and seven actual original-gzip/
      integer-filter/error probes with immutable original-artifact receipts.
- [x] Accept the strict explicit-header TSV/gzip reader at `rsomics-sc e62fc96`,
      with four-native 72-test profiles and independent source/artifact review.
- [x] Execute and independently accept installed samples/Ward.D2 witnesses:
      36 cases/52 routes, original run `36707344893`; no native/performance claim.
- [x] Accept native reference-gene selection, distances and Ward from original
      frozen R states:52 routes/152 trees on four native targets, including
      positive-filter edge semantics. No current-Rust-to-R tree claim follows.
- [x] Validate the integrated raw-to-samples CLI and grouped/pooled ownership;
      green20/run39820518598 passes all four native targets, including real
      1/4-worker execution and a canonical8508-gene/184-cell full-path check. See the
      [current workflow state](.autopilot/state/infercnv-samples-workflow-2026-10-23.md).
- [x] Measure canonical warm raw creation separately from prepared numerical
      preprocessing: exact outputs and strict wall-time advantage accepted on
      one Linuxx86 shipped case. Cold/cohort/full-path performance stays open.
- [ ] Measure the integrated slice's throughput/memory and useful parallelism;
      do not infer replacement speed from kernel benchmarks or thread count.
      Canonical warm-input CLI is now measured:0.64/0.62s at1/4 workers and
      about57.6MiB peak RSS; the small difference has overlapping ranges.
      Profile the serial matrix path, then verify larger-cohort and equivalent
      upstream comparisons. See the [actual measurements](.autopilot/state/infercnv-samples-performance-2026-10-26.md).
- [ ] Resolve default Leiden, HMM i3/i6, Bayesian refinement, denoising/masking,
      output/report and unified `rsomics-help` CLI contracts before full-CNV claims.
- [ ] Apply full product/API/performance/release gates before publishing `sc`.
      Current status and claim limits live in [the dossier](docs/10-products/sc.md).

## Durable evidence

- Namespace allowlist:
  `docs/00-overview/registry-reset-keep.txt`
- Source routing ledger:
  `docs/00-overview/portfolio-inventory.tsv`
- Reconstruction rationale:
  `docs/00-overview/portfolio-reconstruction.md`
- Registry reset archives:
  `/Volumes/KIOXIA/Documents/omics-rust/_retired/`
- Machine progress journals:
  `.autopilot/state/`
