# First actual DESeq2 fitted-workflow witness

Advance the full portfolio while the real whole-BAM report measurement runs.
This is a source oracle, not a native implementation, new public foundation,
product-boundary change or release. Keep the complete bulk-expression dossier's
first-release scope, rather than reviving independent size-factor/GLM crates.

## Qualified source and real input

Use released DESeq2 1.52.0, Bioconductor3.23 and R4.6.1, not devel1.53.x.
Official package archive SHA-256 is
8c91699286336350e66eec132ce6fdf5bb4af78e2a4d015a5a61224f62a95984;
release commit16aeab6d6158bd8cbb8e98764c2f33399b1a3fd3. Source preflight
and external archive are recorded in deseq-source-preflight-2026-10-28.md.

Use the unfiltered shipped pasilla count matrix14599x7, including2240
all-zero rows. Original count gzip SHA
4b5570008936786da365d110df1c83c8cffc646d6c667d6b15ecbfb3d9ee735d;
sample-annotation SHA
1844f871ca5d750f41685c73b87bd7dda291ec0d52304c1afb46a17ae88db745.
Preserve original files, gene and count-column order. Record the documented
fb-suffix metadata identity mapping, then join/reorder metadata by identity.
Explicitly record factor levels and the actual rank-checked model matrix.

## Complete public operation

Call actual DESeqDataSetFromMatrix with design~type+condition, then actual
DESeq Wald with the pinned ordinary defaults: ratio size factors, parametric
dispersion fit with upstream fallback, no beta prior, replacement threshold7,
minmu0.5, no parallel R pool. Derive explicit condition treated-versus-untreated
results with alpha0.1, ordinary independent filtering/BH and the unmodified
upstream default Cook's cutoff. Do not replace a missing cutoff with TRUE.

Export normalized counts and varianceStabilizingTransformation(blind=FALSE)
from the existing fitted state, not vst()'s subsample/refit shortcut. Save and
reload the fit, reproduce the original results/exports exactly, and derive a
reverse contrast through the reloaded fit without another DESeq/dispersion fit.
Retain original RDS objects plus interoperable full-precision tables for
results, fitted coefficients, size factors, model matrix, row diagnostics,
assays and transformed values. Preserve NA/nonconverged/filter/outlier state,
actual dispersion fit type/coefficients, public function bodies/formals,
source/package/dependency/BLAS/runtime identities and raw warnings/logs.

The witness is complete only after the real calls finish, reload checks pass
and every retained file is hashed. A missing package, failed source call or
bad input fails the gate, never a successful skip or copied numerical formula.
One hosted Linux execution establishes this first oracle; it is not a native
four-platform or performance gate. No Mac R build/install/invocation is needed.
All installation, source, evidence and scratch paths live under runner-temp.

## Claim boundaries and next native work

This dataset's design-cell sizes1/2/2/2 do not positively exercise Cook's
masking or replacement, numeric/interaction designs, difficult convergence,
trend fallback, general contrast classes or large-workload performance.
Those remain later integrated workflow cases required by the release dossier.
First reconstruct one product-owned dataset/fitted-state contract from this
actual witness; port selected team-owned numerical assets behind it. Keep
model/dispersion/design policy internal until a second real product proves an
identical policy-free foundation need. Do not create a placeholder crate.
