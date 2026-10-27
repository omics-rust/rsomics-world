# First actual five-metric RNA-seq QC oracle

Plan: docs/plans/2026-10-27-rnaseq-qc-first-oracle.md; dossier and prior21-donor
inventory remain the authority. This advances the next family while inferCNV
hosted measurements run; it is not a product implementation or release.

The implemented actual driver runs single/paired BAM profiles separately,
each covering mapping, strandedness, distribution,100-point coverage and TIN.
Mixed usable layouts would produce source "Mixture" with no numeric fractions
despite exit0, so this is necessary source-population separation rather than
extra advertised metrics.88/175 valid BAM records include flags, boundaries,
complete CIGAR variants and informative coverage. Four deliberately eligible
TIN transcripts include entropy-zero-positive and eligible-zero-exon coverage;
the latter has11 distinct intronic starts to satisfy the source threshold.
Empty transcripts are distinct and excluded from the source summary population.

The three critical driver tests pass; actual BAM generation/sorting/indexing
and quickcheck pass with existing pysam0.24. No installed5.0.4 RSeQC result is
misused as5.0.5 evidence. Root reads the complete driver/test, verifies the
final interruption propagation test, and checks all four new workflow Bash
blocks plus architecture/whitespace. Existing full controller463 tests pass
before that final targeted fix; exact-head CI will exercise all464 tests.

New manual hosted workflow acquires the exact PyPI5.0.5 wheel with its pinned
SHA and GitHub0d2e017 source archive, isolates Python3.12/pysam0.24/numpy2.2.6,
records all resolved dependencies/wheel members and installed script bodies,
and records wheel/GitHub equivalence without assuming version text proves it.
Coverage uses supported --skip-plot; no R installation is required. All source
calls/output tables/status/stdout/stderr/identities are retained. Zero-exit
empty output is not accepted; interrupted campaigns do not launch more metrics.

Actual hosted RSeQC5.0.5 execution is still pending. Do not claim numerical
compatibility, representative performance, native engine, public foundation
or release from these tests. Root will inspect complete actual numeric output
and any wheel/source disagreement before selecting the historical merge.
No Mac install/build/dependency download, deletion or automation occurred.

## Actual completed source witness

Run40520155303/job122244741432 on
world64a323edcde7a289192abd288987cf3655a14bae is terminal SUCCESS. Original
897158-byte artifact12740753197 passes API/head/size/SHA and ZIP CRC checks;
ZIP SHA b54f52612cd1bc8f0ea94f622ba24cba7379c9432c9914e78e185f2bc6ca1a97.
Root retains original API/ZIP/extraction at
rsomics-fixtures/evidence/rnaseq-qc-2026-10-27/run-40520155303/. All recorded
workflow files verify; witness SHA
05affe2124a41813500e16b5b19afc319a1fdf7c52d422f3027a29ad190b307a,
manifestbd13fd167c7fd3cc8813e00dfd7ba45dd96dd60b93f55215064a91af82f965ea.
All ten actual five-metric calls succeed. Five installed script bodies and
seven wheel qcmodule files match the fixed GitHub source, not just version
strings. Runtime is Python3.12, numpy2.2.6, pysam0.24.0/embedded samtools1.23.1,
bx-python0.15.1, pyBigWig0.3.26, logomaker0.8.7.

Actual single/paired: mapping unique82/164; distribution eligible83/166,
tags86/172 and assigned83/166; strandedness usable81/162 with80/160 correct
and1/2 ambiguous. Raw100-point coverage is identical, sum547/range1..8.
TIN plus84.43616115817822, minus82.38743604347249; positive entropy-zero
case100/102=0.9803921568627451, eligible-zero0 included, empty0 excluded.
Mean41.950997339628366, median41.68391410016762, populationSD41.46857661703114.
CLI100 sampling is not actual TIN length102; retain the true list length.

A collector projection omitted the raw zero-valued long mapping label with
no whitespace before its number. Original stdout is complete and unchanged.
Root's narrow regression observes RED then GREEN; the corrected parser
retains all15 counters in both cases. Derived corrected mappings and their
original/raw/parser/output hashes are separate at derived-mapping-20261027/;
the original successful artifact was not rewritten or upstream rerun.

Concrete architecture correction: reuse rsomics-pileup0.9.0 at
8d85ae1b0cc4cea0ff5276094eb556321298aa34, already consumed by call/run and
BAM mpileup/coverage/bedcov. bamio raw visitor records all metric eligibility
before pileup filtering; push_with_state/drain callbacks provide point counts.
Explicit options are skip0x704, MAPQ0, proper pairs only, overlap adjustment,
depth8000; column BQ>=13, no BAQ without FASTA. Defaults are not the profile.
No new public API is needed. Sparse-point stepping is a measured future
optimization; high-depth global-cap versus restarted-transcript equivalence
is unproven, not accepted from the small fixture. Do not duplicate an engine.

This is bounded actual source evidence, not a native implementation,
representative performance, public foundation or release. The first coherent
five-metric shared-scan engine remains the next family implementation.
