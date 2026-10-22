# Read-only samples group contract inventory

Next workflow scope, not implementation or acceptance. Source root:
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-ingestion-2026-09-30/run-36707344893/artifact/samples-clustering`.
Pinned infercnv commit b421d9405c97a309b081ef86d455e976df93eae4, metadata/source.txt.

`metadata/source/infercnv-b421d9405c97a309b081ef86d455e976df93eae4/R/inferCNV_tumor_subclusters.R`:

- Lines34-40,75-81: observation-list order precedes references; pooled observations concatenate ordered maps into all_observations, references remain separate. Never reconstruct index maps by sorting Cell.group.
- Lines189-193,229-231,240-259: >2 cells produce Ward tree in group input order, one group_s1 subcluster with global indices reordered by tree order.
- Lines262-264: <=2 bypass distance/Ward; preserve supplied indices, no hc entry. Singleton names NULL is an R representation distinction.
- Lines45-71,118-123: filter is a temporary view; positive filter with references uses hard-coded >=0.8 over row means of absolute z-scores from global reference mean/sample SD. R -integer(0) selects zero genes. It does not mutate returned state/maps.
- metadata/function-run.txt600-607: samples forces partition_method=none and no per-chromosome subclusters.

Existing exporter probes cover singleton/pair/permuted triplet, multiple reference order, pooled index order, equal profiles/distances/permutations, constant/no/all qualifying genes and filter boundaries at 0/0.2/0.8. Boundary positive filters select genes4,5,6. Original collision proves duplicated observation lookup; user20:08 CST chose early explicit failure in pooled mode when a reference is named all_observations, documented known incompatibility.

Potential narrow integration: borrow stage14 CnvState plus existing ordered CellGroup slices from CreatedInput, a grouped/pooled choice, temporary selected-gene view and numeric budget. Ordered per-group output contains name, optional existing private Dendrogram, and one named subcluster of global indices. No public foundation or speculative clustering-method enum.

Evidence gap before claiming all CreatedInput support: references-only/zero observations is permitted by input.rs448-457 but absent from all36 original cases. Pooled source creates an empty all_observations; capture a pinned real-R edge oracle before choosing its empty/NULL result-map behavior. Do not infer it. Group implementation has not begun; source audit is not a workflow/performance result.
