#!/usr/bin/env Rscript

args <- commandArgs(trailingOnly=TRUE)
if (length(args) != 6L) {
    stop("usage: infercnv_ingestion_matched.R ARTIFACT_ROOT plain|gzip SOURCE_ROOT INPUT_RECEIPT EXISTING_SCRIPTS_DIR NEW_OUTPUT_DIR")
}
flag <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
if (length(flag) != 1L) stop("cannot identify trial script")
proposal_dir <- dirname(normalizePath(sub("^--file=", "", flag)))
script_path <- normalizePath(sub("^--file=", "", flag), mustWork=TRUE)
scripts_dir <- normalizePath(args[[5L]], mustWork=TRUE)
packages <- c("infercnv", "digest", "jsonlite", "Matrix", "BiocManager", "BiocVersion")
for (package in packages) {
    if (!requireNamespace(package, quietly=TRUE)) stop("required namespace unavailable: ", package)
}
code_paths <- c(trial=script_path, runtime_helper=file.path(proposal_dir, "infercnv_ingestion_runtime_support.R"),
    creation_support=file.path(proposal_dir, "infercnv_ingestion_matched_support.R"),
    counter_support=file.path(scripts_dir, "infercnv_matched_support.R"), oracle_io=file.path(scripts_dir, "infercnv_oracle_io.R"),
    creation_checker=file.path(scripts_dir, "validate_infercnv_ingestion_measurement.py"),
    imported_validator=file.path(scripts_dir, "validate_infercnv_samples_clustering_witness.py"))
code_hashes <- vapply(code_paths, function(path) digest::digest(path, algo="sha256", file=TRUE), character(1))
fixed_code <- c(creation_support="d583b156e9a08f3422969697b4524469b8eb05bb92d6c65d41e2aa1df7990faa",
    counter_support="0c3870354e5ac5bafedbd018148492bb5f339714b5412c409866825b2e783956",
    oracle_io="5a01aeb620451d0254bbab775c53111a61eeefcaa32ac9b96ab142563ca5839d",
    creation_checker="0fb1c968e46ff4dd041a42907ab70efaee9fae8beadf1254c81b1edf3c19cb36",
    imported_validator="682d2a41bbe901a02b6687756e64cc1359fa1e3881d49af40eaa5fc725211222")
if (!identical(code_hashes[names(fixed_code)], fixed_code)) stop("fixed code bytes differ before factory")
source(code_paths[["creation_support"]], local=TRUE)
source(code_paths[["runtime_helper"]], local=TRUE)
proposal_dir <- absolute_existing(proposal_dir, TRUE)
scripts_dir <- absolute_existing(scripts_dir, TRUE)
for (path in code_paths) absolute_existing(path)
check_reused_scripts(scripts_dir)
source(file.path(scripts_dir, "infercnv_matched_support.R"), local=TRUE)
source(file.path(scripts_dir, "infercnv_oracle_io.R"), local=TRUE)
runtime_contract <- list(R_version=as.character(getRversion()), Bioconductor_version=as.character(BiocManager::version()),
    infercnv_version=as.character(getNamespaceVersion("infercnv")), system=Sys.info()[["sysname"]],
    architecture=Sys.info()[["machine"]], locale_all=Sys.getlocale(),
    locale=stats::setNames(vapply(locale_categories, Sys.getlocale, character(1)), locale_categories),
    thread_environment=Sys.getenv(thread_keys))
require_runtime_contract(runtime_contract)
if (!file.exists("/proc/self/smaps_rollup")) stop("required procfs unavailable")
python_command <- Sys.which("python3")
if (!nzchar(python_command)) stop("independent Python runtime absent")
python_path <- normalizePath(python_command, mustWork=TRUE)
python_version <- system2(python_path, c("-I", "-B", "--version"), stdout=TRUE, stderr=TRUE)
if (length(python_version) != 1L || !grepl("^Python 3\\.12\\.[0-9]+$", python_version) ||
    !is.null(attr(python_version, "status"))) stop("independent checker requires Python 3.12")
python_sha256 <- sha256_file(python_path)
clock_documentation <- local({
    document <- tools::Rd_db("base")[["proc.time.Rd"]]
    if (is.null(document)) stop("installed proc.time documentation absent")
    capture.output(tools::Rd2txt(document))
})
output <- args[[6L]]
parent <- absolute_existing(dirname(output), TRUE)
if (!startsWith(output, "/") || file.exists(output) ||
    !identical(file.path(parent, basename(output)), output) || !dir.create(output)) {
    stop("new absolute output directory required")
}
setwd(output)
prepare_start <- as.numeric(proc.time()[["elapsed"]])
record <- load_creation_record(args[[1L]], args[[2L]], args[[4L]], args[[3L]])
ns <- asNamespace("infercnv")
check_namespace_source(ns, record, output)
factory <- getExportedValue("infercnv", "CreateInfercnvObject")
counts <- record$counts
positions <- record$positions
annotations <- record$annotations
references <- record$references
exclusions <- record$exclusions
depth <- record$depth
warm_metadata <- local({
    expected_obj <- readRDS(record$checkpoint)
    if (!methods::is(expected_obj, "infercnv")) stop("accepted checkpoint class differs")
    expected <- read_expected_parts(record)
    validate_creation_object(expected_obj, expected)
    warm <- factory(raw_counts_matrix=counts, gene_order_file=positions,
                    annotations_file=annotations, ref_group_names=references,
                    delim="\t", max_cells_per_group=NULL,
                    min_max_counts_per_cell=depth, chr_exclude=exclusions)
    if (!methods::is(warm, "infercnv")) stop("factory returned unexpected class")
    validate_creation_object(warm, expected)
    export_creation(warm, file.path(output, "warm"))
    check_export_python(file.path(output, "warm"), record, scripts_dir, file.path(output, "warm-checker"))
    creation_metadata(warm)
})
invisible(gc(full=TRUE))
threads_before <- read_threads()
baseline_rss_bytes <- read_rss()
before <- unclass(proc.time())
measured <- factory(raw_counts_matrix=counts, gene_order_file=positions,
                    annotations_file=annotations, ref_group_names=references,
                    delim="\t", max_cells_per_group=NULL,
                    min_max_counts_per_cell=depth, chr_exclude=exclusions)
after <- unclass(proc.time())
retained_result_rss_bytes <- read_rss()
counters <- region_counters(before, after)
threads_after <- read_threads()
preparation_wall_seconds <- checked_delta(before[["elapsed"]], prepare_start)
local({
    expected_obj <- readRDS(record$checkpoint)
    if (!methods::is(expected_obj, "infercnv") || !methods::is(measured, "infercnv")) {
        stop("measured or accepted checkpoint class differs")
    }
    expected <- read_expected_parts(record)
    validate_creation_object(expected_obj, expected)
    validate_creation_object(measured, expected)
})
export_creation(measured, file.path(output, "measured"))
check_export_python(file.path(output, "measured"), record, scripts_dir, file.path(output, "measured-checker"))
check_file_hashes(record$paths, record$hashes)
check_reused_scripts(scripts_dir)
check_file_hashes(code_paths, code_hashes)
if (!identical(sha256_file(record$receipt_path), input_receipt_sha256)) stop("input receipt changed")
capture_lifetime_io("/proc/self/io", file.path(output, "io.lifetime.txt"))
metrics <- creation_metrics(record$input_case, preparation_wall_seconds, counters, baseline_rss_bytes, retained_result_rss_bytes)
writeLines(clock_documentation, file.path(output, "clock-proc-time.txt"))
runtime <- list(R_version=runtime_contract$R_version, Bioconductor_version=runtime_contract$Bioconductor_version,
    system=as.list(Sys.info()), locale_all=runtime_contract$locale_all, locale=as.list(runtime_contract$locale),
    Python=list(executable=python_path, version=python_version, sha256=python_sha256),
    namespaces=stats::setNames(lapply(packages, function(package) {
        list(version=as.character(getNamespaceVersion(package)),
             library_path=dirname(getNamespaceInfo(asNamespace(package), "path")))
    }), packages), thread_environment=as.list(runtime_contract$thread_environment),
    observed_threads_before=threads_before, observed_threads_after=threads_after,
    blas=extSoftVersion()[["BLAS"]], options_dput=as.list(capture.output(dput(options()[c("scipen", "digits", "warn", "OutDec")]))),
    warm=warm_metadata, measured=creation_metadata(measured),
    preparation_policy="verified raw/expected/source bytes; validated/exported warm factory result; released local warm/oracle states; gc(full=TRUE); baseline",
    region_clock="proc.time elapsed/self/child counters around actual CreateInfercnvObject; seconds rounded down to milliseconds per installed R documentation; actual accuracy is system-dependent, not assumed nanoseconds",
    clock_source=list(path="clock-proc-time.txt", sha256=sha256_file(file.path(output, "clock-proc-time.txt")),
                      function_text=as.list(deparse(base::proc.time))),
    rss_scope="baseline then returned object live before delta formatting, validation, export or post-call gc",
    io_scope="unmodified /proc/self/io lifetime snapshot after both verified exports and rechecks, before final metadata writes; includes startup, source-proof writes, warmup, validators, exports and instrumentation; not factory-region deltas",
    lifetime_scope="external monitor includes package startup, warm-up, independent Python validators and exports; not factory peak")
jsonlite::write_json(runtime, file.path(output, "runtime.json"), auto_unbox=TRUE, pretty=TRUE, null="null")
provenance <- list(input_case=record$input_case, receipt_sha256=input_receipt_sha256,
    original_run=record$receipt$accepted_original_run, original_artifact=record$receipt$accepted_original_artifact,
    source_commit=record$receipt$source_commit, paths=as.list(record$paths), sha256=as.list(record$hashes),
    creation_arguments=list(raw_counts_matrix=counts, gene_order_file=positions,
        annotations_file=annotations, ref_group_names=unname(as.list(references)), delim="\t",
        max_cells_per_group=NULL, min_max_counts_per_cell=list(1, "Inf"), chr_exclude=unname(as.list(exclusions))),
    code_paths=as.list(code_paths), code_sha256_before=as.list(code_hashes), code_sha256_after=as.list(vapply(code_paths, sha256_file, character(1))),
    fixed_code_sha256=as.list(fixed_code),
    code_authentication="external controller MUST authenticate exact trial/helper/reused-R/Python bytes against independently frozen source before process launch; self/runtime-helper hashes here are drift baselines, not circular authenticity pins")
jsonlite::write_json(provenance, file.path(output, "provenance.json"), auto_unbox=TRUE, pretty=TRUE, null="null")
writeLines(capture.output(sessionInfo()), file.path(output, "session.txt"))
write.table(utils::installed.packages()[, c("Package", "Version", "LibPath"), drop=FALSE],
            file.path(output, "packages.tsv"), sep="\t", row.names=FALSE, quote=FALSE)
check_file_hashes(code_paths, code_hashes)
if (!identical(sha256_file(python_path), python_sha256)) stop("independent Python executable changed")
write_metrics(file.path(output, "metrics.tsv"), metrics)
