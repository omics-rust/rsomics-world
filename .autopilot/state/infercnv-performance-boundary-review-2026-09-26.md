# Prepared-input performance boundary review

Independent read-only review by `/root/infercnv_oracle_review` of the matched
measurement proposal in `infercnv-real-data-preflight-2026-09-26.md`, pinned
infercnv source `b421d9405c97a309b081ef86d455e976df93eae4`, and accepted local
Rust core `13961bf6a42cb4cbbf36c528331aafb98a52937b`. No benchmark execution
or performance acceptance occurred.

## Executable measurement contract

Call this prepared-input preprocessing through step 14, not pure numerical
kernel timing. R retains argument bookkeeping, INFO logging and explicit GC
even with saving disabled (`R/inferCNV_ops.R:380-450,619,776,875,916,957,1036`).
Rust retains configuration/reference checks, allocations and metadata copying
(`src/cnv/pipeline.rs:46-59`, `src/cnv/normalize.rs:31-57`). Do not remove these
normal implementation costs from either side.

Prepare identical dense, ordered stage-1 input outside timing; load packages,
parse files, construct and validate input, and pre-create the output directory
before starting the timer. Record preparation costs separately. The supplied
gzip naturally creates a dense R matrix (`R/inferCNV.R:146-149`). Time exactly
one installed `infercnv::run(stage1_object, ...)` or release Rust
`cnv::run(&prepared, &config)` call until return, retaining its result. This
covers stages 2, 3, 4, 8, 9, 10, 11, 12 and 14 plus normal wrapper work. The R
return at `R/inferCNV_ops.R:1062-1064` precedes downstream clustering and HMM;
Rust's corresponding sequence is `src/cnv/pipeline.rs:37-82`.

Use the accepted cutoff 1, detection minimum 3, two reference groups in their
accepted order, bounds mode, pyramidinal window 101 and clamp 3. Explicitly
disable reference regrouping, scaling, HMM, denoising, outlier pruning,
non-DE masking and chromosome-end trimming. Set `analysis_mode="samples"`
and `inspect_subclusters=FALSE`: `up_to_step=14` alone does not prevent early
step-7 clustering in the subclusters/random-trees branch
(`R/inferCNV_ops.R:715-753`).

Disable resume, `save_rds`, `save_final_rds`, `plot_steps`, `write_expr_matrix`,
`write_phylo` and diagnostics; set `no_plot` and `no_prelim_plot` true.
Do not export stages or run a snapshotting/scanning Rust observer while
timing. Use `cnv::run`, retaining its internal validation. Do not remove R's
`count.data` to equalize footprint: upstream retains and subsets it
(`R/inferCNV.R:320-324,445-449`), an actual implementation resource cost.

Use one native host without competing heavy work. Set inferCNV's thread
argument and BLAS/OpenMP environment controls to one before process startup;
record effective settings and actual BLAS backend. `num_threads` only assigns
inferCNV's global setting here (`R/inferCNV_ops.R:388`), not BLAS limits.
Keep logging destination fixed and local: R resets the logging threshold to
INFO even with `debug=FALSE`, so presetting a quieter threshold is insufficient.

Run at least seven fresh-process trials per implementation in alternating or
recorded randomized order. Each contains an untimed warm-up and one measured
call from unchanged stage-1 input, not the warm-up output. Release warm-up
results first and document any pre-measurement GC; preserve R's internal GC.
Collect region wall time and process user/system CPU deltas, individual samples
and median/spread. Keep whole-process time separate because it includes
startup, preparation and warm-up.

Record whole-process peak RSS with OS/unit provenance and the post-warm-up,
pre-call resident-memory baseline. Never subtract that baseline from a lifetime
high-water mark and label the result core-only memory. Validate/hash returned
results after timing, using separately accepted full stage comparisons for
correctness. Preserve machine, CPU, OS, compiler, package, runtime, flags and
input provenance with every sample.

## Claim boundary

The predicted 9939-to-8508 genes by 184 cells support only a bounded,
warm prepared-input, single-thread comparison after correctness is accepted.
They cannot establish large-cohort scaling, sparse-input efficiency, ingestion
speed, HMM performance or biological validity. Representative larger input and
complete workflow validation remain independent gates.
