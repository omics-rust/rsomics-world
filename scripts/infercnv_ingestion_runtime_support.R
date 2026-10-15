locale_categories <- c("LC_COLLATE", "LC_CTYPE", "LC_MONETARY", "LC_NUMERIC", "LC_TIME", "LC_MESSAGES")
thread_keys <- c("OPENBLAS_NUM_THREADS", "OMP_NUM_THREADS", "MKL_NUM_THREADS", "BLIS_NUM_THREADS",
                 "VECLIB_MAXIMUM_THREADS", "NUMEXPR_NUM_THREADS", "RAYON_NUM_THREADS")
metric_keys <- c("schema_version", "measurement", "implementation", "input_case", "preparation_wall_seconds",
                 "region_wall_seconds", "region_user_seconds", "region_system_seconds",
                 "region_child_user_seconds", "region_child_system_seconds", "baseline_rss_bytes", "returned_rss_bytes")

require_runtime_contract <- function(value) {
    wanted <- list(R_version="4.6.1", Bioconductor_version="3.23", infercnv_version="1.28.0",
                   system="Linux", architecture="x86_64")
    if (any(!vapply(names(wanted), function(name) identical(value[[name]], wanted[[name]]), logical(1)))) {
        stop("runtime requirement differs")
    }
    if (!identical(value$locale_all, "C") || !identical(names(value$locale), locale_categories) ||
        anyNA(value$locale) || any(value$locale != "C")) {
        stop("locale categories differ")
    }
    if (!identical(names(value$thread_environment), thread_keys) || anyNA(value$thread_environment) ||
        any(value$thread_environment != "1")) stop("thread environment differs")
    invisible(TRUE)
}

creation_metrics <- function(input_case, preparation, counters, baseline, returned) {
    rss <- c(baseline, returned)
    if (!is.character(input_case) || length(input_case) != 1L || is.na(input_case) ||
        !(input_case %in% c("plain", "gzip")) || !is.numeric(preparation) || length(preparation) != 1L ||
        !is.finite(preparation) || preparation < 0 || !is.numeric(counters) ||
        !identical(names(counters), metric_keys[6L:10L]) || any(!is.finite(counters)) || any(counters < 0) ||
        counters[[1L]] <= 0 || any(counters[4L:5L] != 0) || !is.numeric(rss) || length(rss) != 2L ||
        any(!is.finite(rss)) || any(rss <= 0 | rss != trunc(rss) | rss > 2^53 - 1)) stop("invalid creation metrics")
    stats::setNames(c("1", "raw_creation", "infercnv", input_case, sprintf("%.17e", preparation),
                      sprintf("%.17e", counters), sprintf("%.0f", rss)), metric_keys)
}

write_metrics <- function(path, metrics) {
    if (!is.character(metrics) || !identical(names(metrics), metric_keys) || anyNA(metrics) ||
        any(grepl("[\t\r\n]", metrics))) stop("metric table contract differs")
    link <- Sys.readlink(path)
    if (file.exists(path) || (!is.na(link) && nzchar(link))) stop("new metadata file required")
    write.table(data.frame(key=names(metrics), value=unname(metrics)), path, sep="\t", row.names=FALSE, quote=FALSE)
    invisible(TRUE)
}

capture_lifetime_io <- function(source_path, output_path) {
    link <- Sys.readlink(output_path)
    if (file.exists(output_path) || (!is.na(link) && nzchar(link))) stop("new metadata file required")
    input <- file(source_path, "rb")
    on.exit(close(input))
    chunks <- list()
    repeat {
        chunk <- readBin(input, "raw", n=65536L)
        if (!length(chunk)) break
        chunks[[length(chunks) + 1L]] <- chunk
    }
    bytes <- do.call(c, chunks)
    if (!is.raw(bytes) || !length(bytes)) stop("empty lifetime I/O snapshot")
    writeBin(bytes, output_path)
    invisible(TRUE)
}
