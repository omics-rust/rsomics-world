# RNA-seq query-context implementation task

Read this first; it is your complete bounded task. Own only
`/Volumes/KIOXIA/Documents/omics-rust/rsomics-rnaseq-qc/` implementation files
needed for source query contexts: src/metrics/sampling.rs, src/report.rs,
src/cli.rs, tests/workflow.rs and necessary narrow private modules. You may add
Rayon as a direct external dependency and update Cargo.lock, preserving every
other current pin, including producer Git1bc989e25a42ce6c0c76e3572b41d4da283e74f2.
The parent owns registry adoption, README/release metadata, snapshots, Git and
CI. Do not edit other products, foundations or control-plane files.

Specification: docs/plans/2026-10-28-rnaseq-query-contexts.md. Read it completely,
then actual source and current consumer code. Do not re-audit the entire
portfolio. Implement query-local RSeQC contexts with bounded shared buffering,
one physical BAM traversal/no index, original quality bytes for each replay,
deterministic useful --threads execution, existing output/metric semantics and
the unchanged canonical global path. No new public API/LayerB dependency.
No approximation, tolerance change, workload filtering, placeholder or claimed
performance without the real comparison. Sorted min/max TIN query bounds,
short duplicate denominator, coverage last-winner, existing start eligibility,
error propagation and transactional output are important existing contracts.

Current native binary9ab7d65e... at the original complete input leaves6
coverage bins +7total and2 TIN rows at chr10:5029966; exact failed outputs are
`/Volumes/KIOXIA/Developments/tmp/rnaseq-whole-name-expiry-20261028.9yY1vQ/`.
The source-vs-query proof is
`/Volumes/KIOXIA/Developments/tmp/rnaseq-qc-remaining-chr10-2_vxlhi5/result.json`.
Actual full BAM/BED/source metrics are under
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/rnaseq-qc-real-performance-2026-10-28/run-40558683154/artifact/benchmark/`.
Use existing scripts/compare_rnaseq_qc_workflow.py compare_report/verify_outputs
from the world control plane for the integrated comparison. Do not rerun RSeQC,
download full data again or mutate the source metrics.

All project files stay external. CARGO_HOME=/Volumes/KIOXIA/Developments/cargo-home,
CARGO_TARGET_DIR=/Volumes/KIOXIA/Developments/cargo-target,
TMPDIR=/Volumes/KIOXIA/Developments/tmp. Before build/test/dependency downloads,
verify all resolved paths and Cargo metadata target_directory, free space on /
and KIOXIA, boot APFS occupancy using diskutil APFSContainerSize/Free; stop
compiling at80% or any boot build path. Use existing absolute Rust1.91 tools
under /Volumes/KIOXIA/Developments/rustup-home/toolchains/1.91.0-aarch64-apple-darwin/bin/;
set CARGO/RUSTC/RUSTDOC and jobs2. Avoid ambient cargo-fmt/clippy shims. Shared
Cargo target may be locked briefly by the independent DESeq task; don't create
another target or boot cache. No installs, deletion, Git mutations, publication,
other agents, exhaustive helper testing or unrelated edits. Use apply_patch.

Keep a compact code/command/result/remaining-issue report in a fresh external
scratch directory and return its path with DONE/DONE_WITH_CONCERNS/NEEDS_CONTEXT.
Ask parent only for a concrete missing decision/contradiction, not routine
approval. Parent performs one fresh integrated review and all public gates.
