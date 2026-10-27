# RNA-seq QC first shared report implementation

Continue the full portfolio while the prioritized inferCNV discovery candidate
runs on hosted native Linux. The dossier is `docs/10-products/rnaseq-qc-signal.md`;
the accepted source witness is run40520155303, retained under external
`rsomics-fixtures/evidence/rnaseq-qc-2026-10-27/run-40520155303/`.
No new public foundation or public repository is needed to build the candidate.

## Coherent operation

Implement one private `rsomics-rnaseq-qc` candidate in its owning external-disk
directory. `report --bam --genes --out` computes mapping, strandedness,
annotation distribution,100-point gene-body coverage and per-transcript/sample
TIN from one coordinate-sorted BAM traversal. Focused commands select these
same accumulators. Do not advertise any later dossier metric.

Use a validated internal BED12 transcript model, explicit metric eligibility
policies and named result types. Canonical full-CIGAR projection is the normal
distribution behavior; the explicitly selected `rseqc-5.0.5` compatibility
profile retains its source legacy projection. Record the selected policy,
filters, sampling population, annotation identity and omitted metrics. This
candidate is unpublished until the complete declared profile is accepted.

Use released `bamio0.8.4`, `common0.12.3`, `help0.4.0`, `intervals0.4.0` and
`pileup0.9.0` where needed, never path or product dependencies. Mandatory
`rsomics-help` owns CLI presentation. Existing common output transactions own
the JSON/TSV/manifest output set. Keep product policies private.

## Numerical and traversal contract

Read the actual retained five scripts/qcmodule source before porting the
historical donors. The existing source ledger is not a substitute for code.
Mapping, strandedness, distribution and TIN start eligibility observe each
original record before pileup filtering. The only BAM reader is the bamio
stream. Feed accepted records into the existing pileup engine and collect
only the precomputed transcript sampling points, including zero counts.

Explicit pinned pileup policy: skip flags0x704, MAPQ0, proper pairs,
overlap adjustment, maximum depth8000, adjusted BQ>=13, no BAQ without FASTA.
Do not write a competing overlap/depth engine. Global high-depth truncation
versus transcript-restarted source pileup remains a known unproved boundary;
record it rather than claiming general equivalence from the small fixture.

Preserve the source's first-N eligible strandedness population, midpoint
assignment priority, genomic-list percentiles, negative orientation, true
TIN sampling-list length, eligibility and eligible-zero summary membership.
The actual positive zero-entropy TIN is100/102, not zero. Mapping's fixed-width
source label projection was corrected separately without modifying raw output.

## Implementation and acceptance

One isolated implementation worker owns only the new private product directory
and its small integration tests/notices. It may refactor team-owned donor code;
record exact donor revisions and disposition in the merge record. It cannot
delete, publish, commit, push, alter foundations or stage any other worktree.
Root owns control-plane routing, source receipts and final acceptance.

No Mac compilation, installation or dependency download: boot APFS exceeds80%.
Use external source/scratch paths; dependency resolution, compile and native
tests run on hosted runner-temp. Preserve existing unrelated dirty files.

Gate the complete report and focused projections against both actual
single/paired source cases. Keep critical annotation/input/output-failure
tests, not a per-helper exhaustive RED/review campaign. Then perform one fresh
whole-feature review and a representative same-host coordinated report timing,
CPU/RSS/I/O comparison. Four native target classes, exact source/lock, strict
format/Clippy and a useful replacement advantage remain release requirements.
