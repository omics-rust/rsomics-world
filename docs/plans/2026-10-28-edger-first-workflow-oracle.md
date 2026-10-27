# First current edgeR fitted-workflow source witness

Implement only a source oracle before creating a native product. The coherent
boundary is docs/10-products/bulk-expression.md's current QL lifecycle, not
the retired legacy QLF micro-crate. Keep the full portfolio goal active.

## Qualified anchors

Current release edgeR4.10.7/Bioc3.23/R4.6.1, source commit
27ba7b39d819e68df2ecdea9f32a4b6039b2a0b5, archive SHA
a3cc0f66be29f1a61a74021f9848ec940794e653905c4332327a719dd9ad8393.
Pin its actual limma dependency3.68.5, commit
825d1c8330d024adb6c54af66d79b0f4ec258530, archive SHA
40b1d2059a76ad98db4bf80139760acf8e8bae92a8265aaaf16ac199831533f4.
Both declare GPL>=2. These are behavior sources, not permission to vendor their
implementation into an incorrectly licensed Rust product. Historical team-owned
code remains reusable with its own provenance.

Preflight source trees/archives are external at
/Volumes/KIOXIA/Developments/tmp/edger-source-preflight-20261028.sDtAJY/.
Main has read DGEList, filtering, subsetting, normalization, modern QL and test
source. Actual release source, rather than inconsistent Rd wording, governs.

## Input and complete calls

Reuse original shipped pasilla raw data, not a DESeq fit or normalized output.
Acquire the exact DESeq2_1.52.0 source archive only as a data carrier (SHA
8c91699286336350e66eec132ce6fdf5bb4af78e2a4d015a5a61224f62a95984),
without installing DESeq2. Its two original input hashes are
4b5570008936786da365d110df1c83c8cffc646d6c667d6b15ecbfb3d9ee735d
for pasilla_gene_counts.tsv.gz and
1844f871ca5d750f41685c73b87bd7dda291ec0d52304c1afb46a17ae88db745
for pasilla_sample_annotation.csv. Keep all14,599 original rows/seven samples
before edgeR's actual filter. Preserve explicit fb-suffix annotation identity
mapping and declared type/condition reference levels, with7×3 design~type+condition.

Run actual public calls: DGEList(counts,samples=joined_metadata),
filterByExpr(y,design=X), y[keep,,keep.lib.sizes=FALSE],
normLibSizes(y,method=TMM), glmQLFit(y,design=X), glmQLFTest using a fully named
condition contrast, topTags(n=Inf,sort.by=none,adjust.method=BH). Freeze exact
options and inputs before fitting. Do not substitute group-only filtering,
sequencing totals, DESeq size factors, fixed NB dispersion or tagwise estimates.

Use current defaults legacy=FALSE, abundance.trend=TRUE, robust=FALSE,
dispersion=NULL, top.proportion=NULL and keep.unit.mat=FALSE. Modern QL estimates
common NB dispersion from high-abundance genes, estimates average QL dispersion,
refits working dispersion, bias-adjusts deviance/df and uses limma's current
unequal-df empirical Bayes. estimateDisp is not a required modern-QL stage.
Do not silently switch to the old donor's legacy=TRUE/abundance.trend=FALSE.

## State and reload evidence

Retain raw counts/metadata, original keep mask/order, kept counts, original and
post-filter library sizes, TMM factors/effective sizes/offsets, held AveLogCPM,
design, shrunken AND unshrunk coefficients, fitted means, NB/working/average QL
dispersion, raw/adjusted deviance and df, s2 prior/post and prior df, convergence
diagnostics, contrasts and complete result/FDR tables. Observed CPM from y is
not fitted CPM from fit. Export actual cpm(y), logCPM and held AveLogCPM with
their original source defaults. Never fabricate kept/significant counts.

Save a full DGEList+DGEGLM analysis bundle with X, masks, original input and
normalization/QL options. Reload and derive forward/reverse contrasts/exports
without repeating filter/TMM/full QL fitting, and verify full saved-state
identity. glmQLFTest's legitimate contrast-specific null GLM must still run;
it is not a forbidden full-analysis refit. Preserve filtered rows via an
explicit mask/status, not fake0/NA statistical estimates.

Record NA/NaN/Inf distinctly in plain tables and metadata descriptors; a valid
infinite prior df must not become an unexplained JSON null. Retain complete
RDS state, warnings/errors, actual iter/failed fields and finite-output checks.
Do not infer convergence merely because a C-backed call returned. Iteration
counts are diagnostic, not cross-BLAS exact gates. Keep actual public-call
defaults, full package/runtime/DLL/source identity and all file checksums.

## Execution and exclusions

Use the existing hosted R4.6.1/source-build pattern with runner-temp storage,
exact edgeR/limma archives and only necessary release dependencies. Qualify
installed relevant R function bodies against release definitions (handle the
real assign-based DGEList subset method). Reuse existing private export/hash
helpers if appropriate; do not introduce a public component or broad framework.
No local R/software install. No native repo, placeholder binary, publication,
biological conclusion or performance claim is produced by this source task.

Later native work starts from reviewed filter/TMM/CPM assets plus a new typed
state, not the legacy QLF skeleton. Existing stats BH may be consumed where
finite-value policy fits; QL/NB/unequal-df policy stays product-owned until a
second concrete consumer proves an appropriate shared contract.

Exclude robust/legacy/classic/LRT/Treat, supplied offsets/weights/library sizes,
other normalizations, RPKM/splicing/single-cell import and cross-product fitted
state interchange from this first witness. Keep them explicit, not placeholders.
