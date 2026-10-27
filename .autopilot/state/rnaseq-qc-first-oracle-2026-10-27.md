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
