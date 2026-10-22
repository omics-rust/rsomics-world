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

Separate oracle design/implementation plan: docs/plans/2026-10-22-infercnv-reference-only-oracle.md. Fresh builder check approved. Bounded test-first agent owns three new edge-only script/test files and one adjacent existing-opt-in workflow step; root audits distance concurrently. Canonical exporter/36cases/152trees/accepted receipt/pin heredocs are protected. Edge implementation/capture/admission are not complete.

Implementation is now committed at d7d808039e213840a63df0673074403e0940ca93, fresh independent static R/source/spec/validator/workflow review approved. Seventeen focused tests pass. Initial full421-test run failed only the stale exact workflow mirror(65.735s), which root synchronized without touching test or pins; coherent full421 run64.227s passes, two existing skips. Native R capture/outcome/authoritative artifact admission remain pending. Script retains raw input/call RDS with immediate identical read-back and typed summaries; no guessed pooled outcome.

Exact-head controller39780963588 passes b3691d50b50c884f4753390d6db07ce7bfd79ea7. Real R oracle39781233583/attempt1 dispatched with datasetshipped,samples_clusteringtrue,ingestion_contractsfalse and blank snapshots. Independent bounded collector retains originals externally under infercnv-reference-only-2026-10-22/run-39781233583; actual capture/status/shape admission remains pending. No canonical old receipt or152-tree fixture changes.

That real run failed in the new script's locale initialization before creating
the object or calling either route. No grouped/pooled outcome exists yet.
Failure/debug/live-R regression record is
infercnv-reference-edge-locale-2026-10-22.md. Canonical capture and17 checker
tests passed before this failure; their success is not edge admission.

## Next private boundary and memory checks

Read-only type/budget audit: CellGroup has private fields/no public constructor
(input.rs33-67). Use a private borrowed named-index view to adapt real getters
and original fixture permutations; no public constructor/foundation is needed.
Bind views to borrowed CnvState; validate names, complete range/disjoint
membership/coverage and source cell-group agreement once before <=2 bypass.
Never sort/reconstruct supplied maps; validate originals before constructing
the generated pooled name. Raw-count positivity rules do not apply to
clustering values. Empty-observation outcome remains pending actual R.

Group numeric workspace must aggregate live allocations. Distance's budget
is requested condensed output only; Ward's includes its own scratch/tree
outputs but excludes caller's distance Vec. With B=prior retained outputs,
owned pooled/index/validation scratch and any filtered-matrix buffer, D=current
distance bytes, W=required_workspace(n), T=current tree and M=new member indices:
check peaks B+D, B+D+W and, after releasing distances, B+T+M. T is already
inside W; do not count it twice there. Retain prior tree/member vectors for
the next group's B. Release validation scratch before kernels where possible.
Borrowed state/maps, metadata and RSS are not bounded by this additional
requested-workspace limit. Creation's stronger actual-capacity contract
(input.rs523-546) is different; do not silently claim it for current kernels.

Required tests include bypass validation (which never reaches distance checks),
actual CreatedInput order, probe permutations, immutable borrowed bits, pooled
name collision and exact multi-group retained-output/pooled-index budget
boundaries. Do not pass the full budget separately to both kernels and call
that an overlapping-peak guarantee. No group implementation has begun.
