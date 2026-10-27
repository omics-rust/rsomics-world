# Isolated native HMM region task

Implement only src/cnv/hmm/regions.rs in rsomics-sc. Do not edit any other
product file, module declarations, CLI, metadata or notices. Main integrates.

Output data/interface:

```rust
#[derive(Debug)]
pub struct CnvRegion {
    pub name: String,
    pub state: u8,
    pub chromosome: String,
    pub start: u64,
    pub end: u64,
    pub gene_start: usize,
    pub gene_end: usize,
}
pub(super) fn regions(genes: &[crate::cnv::Gene], states: &[u8],
                      counter: &mut usize)
    -> rsomics_common::Result<Vec<CnvRegion>>;
```

gene_start/end are global zero-based half-open bounds. State values are
one-based1..6 from the validated decoder. Genes are validated ordered
chromosome blocks; do not sort or revalidate every coordinate. Check paired
lengths once. Main calls groups in report order, counter initially0 and
shared across groups. Do not reset it per group or per chromosome.

Read actual pinned R/inferCNV_HMM.R:706-758,1005-1081. Archive:
/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-ingestion-2026-09-30/run-36707344893/artifact/infercnv-source.tar.gz,
root infercnv-b421d9405c97a309b081ef86d455e976df93eae4/.

For each chromosome with at least two genes, split consecutive equal-state
runs, increment the global counter for EVERY run including neutral3, and
name it `{chromosome}-region_{counter}`. Neutral runs are counted but omitted
from returned predictions. Skip singleton chromosomes completely, with no
counter increment. Region positions use min(start), max(end) across its
genes, not merely the first start and last end (genes can overlap/nest).
Preserve order; don't merge disjoint same-state runs or cross chromosomes.
The global genes/state paths and region ranges let main reproduce all
per-gene/region tables without copying full gene records into each region.

Write two critical tests: neutral gaps plus singleton chromosome plus
global numbering across groups; nested gene ends prove max end semantics.
Use independent literal expectations. No per-helper review or CI campaign;
main verifies the integrated raw-to-calibration/state/report feature once.
Rare concise comments only for stable non-obvious contracts; no phase/audit
narration or generic public foundation. No placeholder behavior.

Product=/Volumes/KIOXIA/Documents/omics-rust/rsomics-sc.
World=/Volumes/Zane's HDD/Documents/rsomics-world.
Cargo home=/Volumes/KIOXIA/Developments/cargo-home;
target=/Volumes/KIOXIA/Developments/cargo-target;
scratch=/Volumes/KIOXIA/Developments/tmp.
Boot APFS92%: no local builds, compiling tests, installs or downloads. Only
existing external Rust1.91 rustfmt may check syntax/format. Do not delete,
stage, commit, publish, dispatch CI or spawn subagents.

Write a short report to .autopilot/state/hmm-regions-report-2026-10-27.md in
world, explicitly noting uncompiled tests. Return status/files/concerns/path.
