# Native HMM region emission report

Implemented only product file
`/Volumes/KIOXIA/Documents/omics-rust/rsomics-sc/src/cnv/hmm/regions.rs`.
No module declarations, existing HMM files, metadata, or notices were changed.

The emitter uses the binding `CnvRegion` interface, checks paired lengths
before mutation, traverses validated chromosome blocks without sorting,
skips singleton chromosomes, counts all consecutive state runs (including
neutral state 3), emits only non-neutral runs, and preserves the caller's
shared counter across groups. Bounds are min(start)/max(end); gene indexes
remain global zero-based half-open. Counter overflow returns an error.

Read the pinned archive's `R/inferCNV_HMM.R` lines 706-758 and 1005-1081
directly. The source confirms shared group numbering, singleton exclusion,
state-transition splitting, and min/max coordinate bounds.

Tests were written before implementation with independent literal expected
regions: neutral gaps + singleton chromosome + numbering across two groups;
nested gene ends; and mismatched lengths with unchanged counter.

Verification: existing external Rust 1.91 rustfmt formatted the new file;
`rustfmt --edition 2024 --check src/cnv/hmm/regions.rs` returned exit 0.
This is parsing/formatting evidence only. Tests are uncompiled and unrun;
no Cargo build, test, Clippy, dependency download, install, or CI was attempted
because the binding task reports boot APFS occupancy of 92%. No executable
red/green test cycle or compatibility/performance claim is made. Integrated
feature verification and wiring remain with the main agent.

No deletes, staging, commits, publishing, CI dispatch, or subagents.
