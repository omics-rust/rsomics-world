#!/usr/bin/env Rscript

flag <- grep("^--file=", commandArgs(FALSE), value=TRUE)
if (length(flag) != 1L) stop("cannot identify runtime contract test")
script_dir <- dirname(normalizePath(sub("^--file=", "", flag)))
source(file.path(script_dir, "infercnv_ingestion_runtime_support.R"), local=TRUE)
if (!identical(as.character(getRversion()), "4.6.1")) stop("pure gate requires existing R 4.6.1")
root <- tempfile("ingestion-runtime-", tmpdir=Sys.getenv("TMPDIR"))
if (!nzchar(Sys.getenv("TMPDIR")) || !dir.create(root)) stop("explicit scratch directory required")
count <- 0L
check <- function(label, action) {
    force(action)
    count <<- count + 1L
    cat(label, "passed\n")
}
fails <- function(action, message) {
    error <- tryCatch({force(action); NULL}, error=identity)
    if (!inherits(error, "error") || !grepl(message, conditionMessage(error), fixed=TRUE)) stop("bounded rejection absent: ", message)
}
runtime <- list(R_version="4.6.1", Bioconductor_version="3.23", infercnv_version="1.28.0",
                system="Linux", architecture="x86_64", locale_all="C",
                locale=stats::setNames(rep("C", length(locale_categories)), locale_categories),
                thread_environment=stats::setNames(rep("1", length(thread_keys)), thread_keys))
check("accepted runtime record", stopifnot(require_runtime_contract(runtime)))
check("wrong actual runtime rejected", {
    for (name in c("R_version", "Bioconductor_version", "infercnv_version", "system", "architecture")) {
        changed <- runtime
        changed[[name]] <- "wrong"
        fails(require_runtime_contract(changed), "runtime requirement differs")
    }
})
check("non-C locale rejected", {
    changed <- runtime
    changed$locale[["LC_CTYPE"]] <- "C.UTF-8"
    fails(require_runtime_contract(changed), "locale categories differ")
    changed$locale <- runtime$locale[-1L]
    fails(require_runtime_contract(changed), "locale categories differ")
    changed <- runtime
    changed$locale_all <- "C.UTF-8"
    fails(require_runtime_contract(changed), "locale categories differ")
})
check("unset or non-one thread limit rejected", {
    changed <- runtime
    changed$thread_environment[[1L]] <- "2"
    fails(require_runtime_contract(changed), "thread environment differs")
    changed$thread_environment <- runtime$thread_environment[-1L]
    fails(require_runtime_contract(changed), "thread environment differs")
})
counters <- c(region_wall_seconds=0.25, region_user_seconds=0.125, region_system_seconds=0,
              region_child_user_seconds=0, region_child_system_seconds=0)
metrics <- creation_metrics("plain", 1, counters, 1024, 2048)
check("literal native-compatible metric bytes", {
    path <- file.path(root, "metrics.tsv")
    write_metrics(path, metrics)
    wanted <- c("key\tvalue", "schema_version\t1", "measurement\traw_creation", "implementation\tinfercnv", "input_case\tplain",
        "preparation_wall_seconds\t1.00000000000000000e+00", "region_wall_seconds\t2.50000000000000000e-01",
        "region_user_seconds\t1.25000000000000000e-01", "region_system_seconds\t0.00000000000000000e+00",
        "region_child_user_seconds\t0.00000000000000000e+00", "region_child_system_seconds\t0.00000000000000000e+00",
        "baseline_rss_bytes\t1024", "returned_rss_bytes\t2048")
    stopifnot(identical(readLines(path), wanted))
    fails(write_metrics(path, metrics), "new metadata file required")
    stopifnot(identical(readLines(path), wanted))
})
check("invalid timing or RSS rejected", {
    for (value in c(0, -1, Inf, NaN)) fails(creation_metrics("plain", 1, replace(counters, 1L, value), 1, 1), "invalid creation metrics")
    for (value in c(0, -1, 1.5, Inf)) fails(creation_metrics("plain", 1, counters, value, 1), "invalid creation metrics")
    fails(creation_metrics("unknown", 1, counters, 1, 1), "invalid creation metrics")
    fails(creation_metrics("plain", -1, counters, 1, 1), "invalid creation metrics")
    fails(creation_metrics("plain", 1, replace(counters, 4L, 1), 1, 1), "invalid creation metrics")
})
check("metric keys and separators rejected before creation", {
    path <- file.path(root, "invalid.tsv")
    changed <- metrics
    names(changed)[[2L]] <- names(changed)[[1L]]
    fails(write_metrics(path, changed), "metric table contract differs")
    changed <- metrics
    changed[[3L]] <- "bad\tvalue"
    fails(write_metrics(path, changed), "metric table contract differs")
    stopifnot(!file.exists(path))
})
check("lifetime I/O bytes copied without interpretation", {
    source_path <- file.path(root, "io-source")
    wanted <- charToRaw("rchar: 123\nsyscr: 2\nread_bytes: 0\n")
    writeBin(wanted, source_path)
    output <- file.path(root, "io.lifetime.txt")
    capture_lifetime_io(source_path, output)
    stopifnot(identical(readBin(output, "raw", n=1000L), wanted))
    fails(capture_lifetime_io(source_path, output), "new metadata file required")
})
cat("Runtime helper contracts passed:", count, "; package-free, no factory or performance claim.\n")
