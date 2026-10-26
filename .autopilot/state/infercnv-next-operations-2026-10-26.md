# Next integrated inferCNV work

Matrix workers and real-cohort scaling are complete within their documented
scope. Do not rebuild their per-helper oracle/verification campaigns. The
goal remains all products, with complete inferCNV the current priority.

Next: equivalent whole-workflow upstream comparison and integrated denoise,
then HMM/subcluster/Bayesian/reporting work. Reuse the existing pinned-package
CI environment rather than fabricate R results or treat missing4090 packages
as a global blocker. R4.6.1 is present there, but infercnv, fastcluster and
parallelDist are absent from default libraries; no installation was made.

Read actual pinned source before designing the next workflow contract:
external evidence/infercnv-ingestion-2026-09-30/run-36707344893/artifact/
infercnv-source.tar.gz, commit b421d9405c97a309b081ef86d455e976df93eae4,
infercnv1.28.0. DESCRIPTION declares BSD_3_clause + file LICENSE; LICENSE
identifies the2018 Broad authors. Preserve appropriate attribution on reuse.

Root inspected R/inferCNV_ops.R:1538-1622 and2232-2408. Observed facts:

- Denoising occurs at step22, after clustering/HMM/masking stages; do not
  silently relabel denoised values as the existing step14 expression export.
- Fixed-threshold centering uses one global mean over all reference values,
  or all values if no reference cells exist. It is not a per-gene mean.
- Strictly interior values are replaced by that center; exact bounds remain.
  The pipeline bypasses nonpositive explicit noise_filter values.
- The default SD width is sd_amplifier times the mean of per-reference-cell
  sample SDs across genes. It is not the per-gene SD across reference cells.
- No-reference SD mode uses observations as proxy. The logistic mode delegates
  to depress_log_signal_midpt_val; read that implementation before adding it.

These are source observations, not installed-package or native acceptance.
Continue through a functioning workflow and meaningful real-package checks,
with a narrow typed result/stage contract, existing rsomics-help/common UX,
critical tests and one integrated review. No speculative public foundation,
placeholder CLI flag or unnecessary comment is justified by these notes.
