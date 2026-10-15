#!/usr/bin/env Rscript

args <- commandArgs(trailingOnly=TRUE)
if (length(args) != 5L) {
    stop("usage: test_infercnv_ingestion_matched_package.R ARTIFACT_ROOT SOURCE_ROOT INPUT_RECEIPT EXISTING_SCRIPTS_DIR NEW_OUTPUT_DIR")
}
flag <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
if (length(flag) != 1L) stop("cannot identify package contract script")
proposal_dir <- dirname(normalizePath(sub("^--file=", "", flag)))
for (package in c("infercnv", "digest", "jsonlite", "Matrix", "BiocManager", "BiocVersion")) {
    if (!requireNamespace(package, quietly=TRUE)) stop("installed package missing: ", package)
}
accepted_support <- file.path(proposal_dir, "infercnv_ingestion_matched_support.R")
if (!identical(digest::digest(accepted_support, algo="sha256", file=TRUE),
               "d583b156e9a08f3422969697b4524469b8eb05bb92d6c65d41e2aa1df7990faa")) stop("accepted helper bytes differ")
source(accepted_support, local=TRUE)
source(file.path(proposal_dir, "infercnv_ingestion_runtime_support.R"), local=TRUE)
scripts_dir <- absolute_existing(args[[4L]], TRUE)
check_reused_scripts(scripts_dir)
source(file.path(scripts_dir, "infercnv_matched_support.R"), local=TRUE)
source(file.path(scripts_dir, "infercnv_oracle_io.R"), local=TRUE)
actual_runtime <- list(R_version=as.character(getRversion()), Bioconductor_version=as.character(BiocManager::version()),
    infercnv_version=as.character(getNamespaceVersion("infercnv")), system=Sys.info()[["sysname"]],
    architecture=Sys.info()[["machine"]], locale_all=Sys.getlocale(),
    locale=stats::setNames(vapply(locale_categories, Sys.getlocale, character(1)), locale_categories),
    thread_environment=Sys.getenv(thread_keys))
require_runtime_contract(actual_runtime)
output <- args[[5L]]
parent <- absolute_existing(dirname(output), TRUE)
if (!startsWith(output, "/") || file.exists(output) ||
    !identical(file.path(parent, basename(output)), output) || !dir.create(output)) {
    stop("new absolute package-test output directory required")
}
setwd(output)
fails <- function(action, message) {
    error <- tryCatch({force(action); NULL}, error=identity)
    if (!inherits(error, "error") || !grepl(message, conditionMessage(error), fixed=TRUE)) {
        stop("expected package rejection absent: ", message)
    }
}
trial_path <- file.path(proposal_dir, "infercnv_ingestion_matched.R")
if (!file.exists(trial_path)) stop("actual creation trial absent: ", trial_path)
for (target in c("creation_support", "counter_support", "oracle_io", "creation_checker", "imported_validator")) {
    fixture <- file.path(output, paste0("bad-code-", target))
    adjacent <- file.path(fixture, "trial-scripts")
    reused <- file.path(fixture, "reused-scripts")
    if (!dir.create(fixture) || !dir.create(adjacent) || !dir.create(reused)) stop("cannot create bootstrap fixture")
    new_names <- c("infercnv_ingestion_matched.R", "infercnv_ingestion_runtime_support.R", "infercnv_ingestion_matched_support.R")
    old_names <- c("infercnv_matched_support.R", "infercnv_oracle_io.R",
                   "validate_infercnv_ingestion_measurement.py", "validate_infercnv_samples_clustering_witness.py")
    if (!all(file.copy(file.path(proposal_dir, new_names), adjacent)) ||
        !all(file.copy(file.path(scripts_dir, old_names), reused))) stop("cannot copy bootstrap fixture")
    targets <- c(creation_support=file.path(adjacent, new_names[[3L]]),
                 stats::setNames(file.path(reused, old_names), c("counter_support", "oracle_io", "creation_checker", "imported_validator")))
    writeLines("mutated bootstrap bytes", targets[[target]])
    destination <- file.path(fixture, "must-not-exist")
    stdout <- file.path(fixture, "stdout")
    stderr <- file.path(fixture, "stderr")
    status <- system2(file.path(R.home("bin"), "Rscript"), c("--vanilla",
        shQuote(file.path(adjacent, new_names[[1L]])), shQuote(args[[1L]]), "plain", shQuote(args[[2L]]),
        shQuote(args[[3L]]), shQuote(reused), shQuote(destination)), stdout=stdout, stderr=stderr)
    if (status == 0L || file.exists(destination) ||
        !any(grepl("fixed code bytes differ before factory", readLines(stderr, warn=FALSE), fixed=TRUE))) {
        stop("actual bootstrap failed to reject changed code before factory: ", target)
    }
}
record <- load_creation_record(args[[1L]], "plain", args[[3L]], args[[2L]])
check_namespace_source(asNamespace("infercnv"), record, output)
expected <- read_expected_parts(record)
expected_obj <- readRDS(record$checkpoint)
validate_creation_object(expected_obj, expected)
factory <- getExportedValue("infercnv", "CreateInfercnvObject")
plain <- factory(raw_counts_matrix=record$counts, gene_order_file=record$positions,
                 annotations_file=record$annotations, ref_group_names=record$references,
                 delim="\t", max_cells_per_group=NULL,
                 min_max_counts_per_cell=record$depth, chr_exclude=record$exclusions)
gzip_record <- load_creation_record(args[[1L]], "gzip", args[[3L]], args[[2L]])
compressed <- factory(raw_counts_matrix=gzip_record$counts, gene_order_file=gzip_record$positions,
                      annotations_file=gzip_record$annotations, ref_group_names=gzip_record$references,
                      delim="\t", max_cells_per_group=NULL,
                      min_max_counts_per_cell=gzip_record$depth, chr_exclude=gzip_record$exclusions)
for (obj in list(plain, compressed)) {
    if (!methods::is(obj, "infercnv")) stop("factory class differs")
    validate_creation_object(obj, expected)
}
export_creation(plain, file.path(output, "plain"))
export_creation(compressed, file.path(output, "gzip"))
check_export_python(file.path(output, "plain"), record, scripts_dir, file.path(output, "plain-checker"))
check_export_python(file.path(output, "gzip"), record, scripts_dir, file.path(output, "gzip-checker"))
for (slot in c("expr.data", "count.data")) {
    changed <- plain
    values <- methods::slot(changed, slot)
    values[1L, 1L] <- values[1L, 1L] + 1
    methods::slot(changed, slot) <- values
    fails(validate_creation_object(changed, expected), "creation values, identities or ordered maps differ")
}
changed <- plain
changed@reference_grouped_cell_indices <- rev(changed@reference_grouped_cell_indices)
fails(validate_creation_object(changed, expected), "creation values, identities or ordered maps differ")
changed <- plain
changed@reference_grouped_cell_indices[[1L]] <- rev(changed@reference_grouped_cell_indices[[1L]])
fails(validate_creation_object(changed, expected), "creation values, identities or ordered maps differ")
changed <- plain
ri <- changed@reference_grouped_cell_indices[[1L]][1L]
oi <- changed@observation_grouped_cell_indices[[1L]][1L]
changed@reference_grouped_cell_indices[[1L]][1L] <- oi
changed@observation_grouped_cell_indices[[1L]][1L] <- ri
fails(validate_creation_object(changed, expected), "creation values, identities or ordered maps differ")
changed <- plain
colnames(changed@expr.data)[1L] <- "wrong-cell"
fails(validate_creation_object(changed, expected), "count and expression identities differ")
changed <- plain
changed@gene_order$stop[1L] <- changed@gene_order$stop[1L] + 1
fails(validate_creation_object(changed, expected), "creation values, identities or ordered maps differ")
changed <- plain
changed@count.data[1L, 1L] <- Inf
fails(validate_creation_object(changed, expected), "invalid count matrix")
bad_factory <- factory
body(bad_factory) <- quote(stop("different implementation"))
fails(check_function_source(bad_factory, record$paths[["R/inferCNV.R"]], "CreateInfercnvObject"),
      "installed function differs")
bad_hashes <- record$hashes
bad_hashes[["plain"]] <- paste(rep("0", 64L), collapse="")
fails(check_file_hashes(record$paths, bad_hashes), "input bytes changed")
bad_receipt <- file.path(output, "bad-input-receipt.json")
writeLines("{}", bad_receipt)
fails(load_creation_record(args[[1L]], "plain", bad_receipt, args[[2L]]), "input receipt bytes differ")
fails(load_creation_record(args[[1L]], "invalid", args[[3L]], args[[2L]]), "unknown raw input case")
rscript <- file.path(R.home("bin"), "Rscript")
if (any(Sys.getenv(thread_keys) != "1")) stop("set actual thread environment before starting R")
for (input_case in c("plain", "gzip")) {
    trial <- file.path(output, paste0("fresh-", input_case))
    status <- system2(rscript, c("--vanilla", shQuote(file.path(proposal_dir, "infercnv_ingestion_matched.R")),
        shQuote(args[[1L]]), input_case, shQuote(args[[2L]]), shQuote(args[[3L]]), shQuote(scripts_dir), shQuote(trial)),
        stdout=file.path(output, paste0(input_case, "-trial.stdout")),
        stderr=file.path(output, paste0(input_case, "-trial.stderr")))
    if (!identical(status, 0L)) stop("actual fresh factory trial failed: ", input_case)
    for (phase in c("warm", "measured")) {
        check_export_python(file.path(trial, phase), record, scripts_dir,
                            file.path(output, paste0(input_case, "-", phase, "-independent")))
    }
    metrics <- read.table(file.path(trial, "metrics.tsv"), header=TRUE, sep="\t", quote="", comment.char="",
                          colClasses="character", check.names=FALSE)
    if (!identical(names(metrics), c("key", "value")) || anyDuplicated(metrics$key)) stop("trial metrics malformed")
    values <- stats::setNames(metrics$value, metrics$key)
    needed <- c("schema_version", "measurement", "implementation", "input_case", "preparation_wall_seconds", "region_wall_seconds",
                "region_user_seconds", "region_system_seconds", "region_child_user_seconds",
                "region_child_system_seconds", "baseline_rss_bytes", "returned_rss_bytes")
    if (!identical(names(values), needed) || values[["implementation"]] != "infercnv" ||
        values[["schema_version"]] != "1" || values[["measurement"]] != "raw_creation" ||
        values[["input_case"]] != input_case) stop("trial metric inventory differs")
    numbers <- as.numeric(values[-seq_len(4L)])
    if (any(!is.finite(numbers)) || any(numbers < 0) || as.numeric(values[["region_wall_seconds"]]) <= 0 ||
        as.numeric(values[["baseline_rss_bytes"]]) <= 0 || as.numeric(values[["returned_rss_bytes"]]) <= 0 ||
        as.numeric(values[["region_child_user_seconds"]]) != 0 || as.numeric(values[["region_child_system_seconds"]]) != 0) {
        stop("actual trial counter contract differs")
    }
    runtime <- jsonlite::fromJSON(file.path(trial, "runtime.json"), simplifyVector=FALSE)
    packages <- c("infercnv", "digest", "jsonlite", "Matrix", "BiocManager", "BiocVersion")
    if (!identical(names(runtime$namespaces), packages) ||
        !identical(runtime$Bioconductor_version, "3.23") ||
        !identical(runtime$locale_all, "C") ||
        !identical(unlist(runtime$locale, use.names=TRUE), actual_runtime$locale) ||
        !identical(names(runtime$thread_environment), thread_keys) ||
        !identical(runtime$warm$expression$type, typeof(plain@expr.data)) ||
        !identical(runtime$measured$counts$type, typeof(plain@count.data)) ||
        any(unlist(runtime$thread_environment) != "1") || runtime$observed_threads_before < 1 ||
        runtime$observed_threads_after < 1) stop("actual runtime metadata differs")
    for (package in packages) {
        actual <- runtime$namespaces[[package]]
        if (!identical(actual$version, as.character(getNamespaceVersion(package))) ||
            !identical(actual$library_path, dirname(getNamespaceInfo(asNamespace(package), "path")))) {
            stop("actual loaded namespace version or library differs: ", package)
        }
    }
    if (!any(grepl("Inf", unlist(runtime$warm$options_dput), fixed=TRUE)) ||
        !any(grepl("Inf", unlist(runtime$measured$options_dput), fixed=TRUE))) stop("actual Inf options lost")
    if (!file.exists(file.path(trial, "io.lifetime.txt")) ||
        !identical(sha256_file(file.path(trial, "clock-proc-time.txt")), runtime$clock_source$sha256)) {
        stop("actual lifetime I/O or clock provenance absent")
    }
    provenance <- jsonlite::fromJSON(file.path(trial, "provenance.json"), simplifyVector=FALSE)
    if (!identical(provenance$input_case, input_case) ||
        !identical(provenance$receipt_sha256, input_receipt_sha256) ||
        !identical(provenance$creation_arguments$min_max_counts_per_cell, list(1L, "Inf")) ||
        !identical(unlist(provenance$sha256, use.names=TRUE), record$hashes)) stop("actual trial provenance differs")
    code_paths <- unlist(provenance$code_paths, use.names=TRUE)
    before <- unlist(provenance$code_sha256_before, use.names=TRUE)
    after <- unlist(provenance$code_sha256_after, use.names=TRUE)
    check_file_hashes(code_paths, before)
    if (!identical(before, after)) stop("trial code drifted")
    fixed <- unlist(provenance$fixed_code_sha256, use.names=TRUE)
    if (!identical(before[names(fixed)], fixed)) stop("trial fixed code binding differs")
}
check_file_hashes(record$paths, record$hashes)
cat("Actual package creation contracts passed; two fresh trials/four verified results are not a paired performance acceptance.\n")
