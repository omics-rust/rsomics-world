#!/usr/bin/env Rscript

args <- commandArgs(trailingOnly=TRUE)
if (length(args) != 2L) stop("usage: infercnv_matched.R BUNDLE_DIRECTORY NEW_OUTPUT_DIRECTORY")
script_flag <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
if (length(script_flag) != 1L) stop("cannot identify script path")
script_dir <- dirname(normalizePath(sub("^--file=", "", script_flag)))
source(file.path(script_dir, "infercnv_matched_support.R"), local=TRUE)
source(file.path(script_dir, "infercnv_oracle_io.R"), local=TRUE)

if (!requireNamespace("infercnv", quietly=TRUE) ||
    !identical(as.character(utils::packageVersion("infercnv")), "1.28.0") ||
    !requireNamespace("jsonlite", quietly=TRUE) ||
    !requireNamespace("digest", quietly=TRUE)) stop("pinned infercnv/jsonlite/digest not installed")
if (.Platform$OS.type != "unix" || !file.exists("/proc/self/smaps_rollup")) {
    stop("matched measurement requires native Linux /proc/self/smaps_rollup")
}
for (key in c("OPENBLAS_NUM_THREADS", "OMP_NUM_THREADS", "MKL_NUM_THREADS",
              "BLIS_NUM_THREADS", "VECLIB_MAXIMUM_THREADS")) {
    if (!identical(Sys.getenv(key), "1")) stop("thread limit mismatch: ", key)
}

bundle <- normalizePath(args[[1L]], mustWork=TRUE)
output <- args[[2L]]
if (file.exists(output)) stop("output directory already exists")
dir.create(output, recursive=TRUE, showWarnings=FALSE)
if (!dir.exists(output)) stop("cannot create output directory")

prepare_start <- as.numeric(proc.time()[["elapsed"]])
record <- load_prepared_record(bundle, script_dir)
obj <- readRDS(owned_bundle_file(bundle, record$checkpoint))
validate_prepared(obj, bundle, record)
validate_run_formals()
identity <- prepared_identity(obj)
warm_dir <- file.path(output, "warm")
measured_dir <- file.path(output, "measured")
dir.create(warm_dir)
dir.create(measured_dir)
if (!dir.exists(warm_dir) || !dir.exists(measured_dir)) stop("cannot create run directories")
preparation_wall_seconds <- checked_delta(as.numeric(proc.time()[["elapsed"]]), prepare_start)

provenance <- list(package_version=as.character(utils::packageVersion("infercnv")),
                   r_version=R.version.string, args=matched_args(),
                   session=capture.output(sessionInfo()),
                   blas=extSoftVersion()[["BLAS"]],
                   linux_threads_before_warm=grep("^Threads:",
                       readLines("/proc/self/status", warn=FALSE), value=TRUE),
                   thread_limits=as.list(Sys.getenv(c("OPENBLAS_NUM_THREADS", "OMP_NUM_THREADS",
                           "MKL_NUM_THREADS", "BLIS_NUM_THREADS", "VECLIB_MAXIMUM_THREADS"))),
                   checkpoint=record$checkpoint,
                   checkpoint_sha256=record$checkpoint_sha256,
                   gc_policy="drop warm result then gc(full=TRUE) before baseline")
jsonlite::write_json(provenance, file.path(output, "provenance.json"), pretty=TRUE, auto_unbox=TRUE)

warm <- run_prepared(obj, warm_dir)
rm(warm)
if (!identical(prepared_identity(obj), identity)) stop("warm-up mutated prepared input")
gc(full=TRUE)
baseline_rss_bytes <- parse_rss_bytes(paste(readLines("/proc/self/smaps_rollup", warn=FALSE), collapse="\n"))
before <- unclass(proc.time())
measured <- run_prepared(obj, measured_dir)
after <- unclass(proc.time())
region_wall_seconds <- checked_delta(after[["elapsed"]], before[["elapsed"]])
region_user_seconds <- checked_delta(after[["user.self"]], before[["user.self"]])
region_system_seconds <- checked_delta(after[["sys.self"]], before[["sys.self"]])
region_child_user_seconds <- checked_delta(after[["user.child"]], before[["user.child"]])
region_child_system_seconds <- checked_delta(after[["sys.child"]], before[["sys.child"]])
if (region_wall_seconds <= 0 || region_child_user_seconds != 0 ||
    region_child_system_seconds != 0) stop("invalid measured region counters")
if (!identical(prepared_identity(obj), identity)) stop("measured run mutated prepared input")
validate_prepared(obj, bundle, record)

write_expression(measured, file.path(output, "14.tsv"))
write_identity(measured, file.path(output, "14.genes.tsv"), file.path(output, "14.cells.tsv"))
metrics <- c(schema_version="1", implementation="infercnv",
             preparation_wall_seconds=sprintf("%.9f", preparation_wall_seconds),
             region_wall_seconds=sprintf("%.9f", region_wall_seconds),
             region_user_seconds=sprintf("%.9f", region_user_seconds),
             region_system_seconds=sprintf("%.9f", region_system_seconds),
             region_child_user_seconds=sprintf("%.9f", region_child_user_seconds),
             region_child_system_seconds=sprintf("%.9f", region_child_system_seconds),
             baseline_rss_bytes=sprintf("%.0f", baseline_rss_bytes))
write.table(data.frame(metric=names(metrics), value=unname(metrics)),
            file.path(output, "metrics.tsv"), sep="\t", row.names=FALSE, quote=FALSE)
