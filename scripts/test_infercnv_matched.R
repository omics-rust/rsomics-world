#!/usr/bin/env Rscript

args <- commandArgs(trailingOnly=TRUE)
if (length(args) != 1L) stop("usage: test_infercnv_matched.R BUNDLE_DIRECTORY")
script_flag <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
if (length(script_flag) != 1L) stop("cannot identify script path")
script_dir <- dirname(normalizePath(sub("^--file=", "", script_flag)))
source(file.path(script_dir, "infercnv_matched_support.R"), local=TRUE)
if (!requireNamespace("infercnv", quietly=TRUE) ||
    !requireNamespace("jsonlite", quietly=TRUE) ||
    !requireNamespace("digest", quietly=TRUE)) stop("required installed packages missing")

fails <- function(action) {
    if (!inherits(tryCatch({force(action); NULL}, error=function(err) err), "error")) {
        stop("negative contract did not fail")
    }
}
bundle <- normalizePath(args[[1L]], mustWork=TRUE)
record <- load_prepared_record(bundle, script_dir)
obj <- readRDS(owned_bundle_file(bundle, record$checkpoint))
if (!identical(validate_prepared(obj, bundle, record), invisible(TRUE))) {
    stop("baseline prepared object failed")
}
validate_run_formals()

changed <- obj
ri <- changed@reference_grouped_cell_indices[[1L]][1L]
oi <- changed@observation_grouped_cell_indices[[1L]][1L]
changed@reference_grouped_cell_indices[[1L]][1L] <- oi
changed@observation_grouped_cell_indices[[1L]][1L] <- ri
fails(validate_prepared(changed, bundle, record))

changed <- obj
changed@reference_grouped_cell_indices <- rev(changed@reference_grouped_cell_indices)
fails(validate_prepared(changed, bundle, record))
changed <- obj
changed@reference_grouped_cell_indices[[1L]] <- rev(changed@reference_grouped_cell_indices[[1L]])
fails(validate_prepared(changed, bundle, record))

changed <- obj
changed@.hspike <- list(marker=TRUE)
fails(validate_prepared(changed, bundle, record))
changed <- obj
changed@count.data[1L, 1L] <- changed@count.data[1L, 1L] + 1
fails(validate_prepared(changed, bundle, record))
changed <- obj
changed@expr.data[1L, 1L] <- changed@expr.data[1L, 1L] + 1
fails(validate_prepared(changed, bundle, record))
changed <- obj
colnames(changed@expr.data)[1L] <- "wrong-cell"
fails(validate_prepared(changed, bundle, record))
changed <- obj
changed@expr.data[1L, 1L] <- Inf
fails(validate_prepared(changed, bundle, record))
changed <- obj
changed@gene_order$start[1L] <- changed@gene_order$start[1L] + 1
fails(validate_prepared(changed, bundle, record))

bad <- record
bad$checkpoint_sha256 <- paste(rep("0", 64L), collapse="")
fails(validate_prepared(obj, bundle, bad))
bad <- record
bad$checkpoint <- "../unrelated.rds"
fails(validate_prepared(obj, bundle, bad))
bad <- record
bad$receipt$files[["full/01.tsv"]] <- paste(rep("0", 64L), collapse="")
fails(validate_prepared(obj, bundle, bad))

if (!identical(parse_rss_bytes("Rss: 17 kB\n"), 17408)) stop("RSS unit conversion failed")
for (value in c("", "Rss: 1 MB", "Rss: 0 kB", "Rss: 1 kB\nRss: 2 kB",
                "Rss: 9007199254740992 kB")) fails(parse_rss_bytes(value))
if (!identical(checked_delta(5, 2), 3)) stop("clock delta failed")
fails(checked_delta(1, 2))
fails(checked_delta(Inf, 2))
fails(checked_delta(2, -1))
cat("infercnv matched preparation contracts passed\n")
