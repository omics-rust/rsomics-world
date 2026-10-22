#!/usr/bin/env Rscript

if (length(commandArgs(trailingOnly=TRUE)) != 0L) stop("locale regression takes no arguments")
if (!identical(as.character(getRversion()), "4.6.1")) stop("locale regression requires R 4.6.1")
script_arg <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
if (length(script_arg) != 1L) stop("cannot identify locale regression script")
script <- normalizePath(sub("^--file=", "", script_arg), mustWork=TRUE)
capture <- new.env(parent=baseenv())
source(file.path(dirname(script), "infercnv_samples_reference_edges.R"), local=capture)

categories <- c("LC_COLLATE", "LC_CTYPE", "LC_MONETARY", "LC_NUMERIC", "LC_TIME",
                "LC_MESSAGES", "LC_PAPER", "LC_MEASUREMENT")
seeded <- c("LC_MESSAGES", "LC_PAPER", "LC_MEASUREMENT")
for (category in seeded) {
    accepted <- Sys.setlocale(category, "C.UTF-8")
    actual <- Sys.getlocale(category)
    if (!nzchar(accepted) || !nzchar(actual) || identical(actual, "C")) {
        stop("locale regression setup failed: ", category, " did not accept non-C C.UTF-8")
    }
}
locale_values <- function() vapply(categories, Sys.getlocale, character(1))
log_locales <- function(label) {
    cat(label, " LC_ALL=", Sys.getlocale(), "\n", sep="")
    values <- locale_values()
    for (category in categories) cat(label, " ", category, "=", values[[category]], "\n", sep="")
    flush.console()
}
require_c_locale <- function() {
    if (!identical(Sys.getlocale(), "C") || any(locale_values() != "C")) {
        stop("locale initializer did not reset every category to C")
    }
}

log_locales("before")
failure <- tryCatch({capture$initialize_samples_locale(); NULL}, error=identity)
log_locales("after")
if (!is.null(failure)) stop(failure)
require_c_locale()
first <- locale_values()
capture$initialize_samples_locale()
log_locales("second")
require_c_locale()
if (!identical(locale_values(), first)) stop("locale initializer is not idempotent")
cat("samples locale contract passed\n")
