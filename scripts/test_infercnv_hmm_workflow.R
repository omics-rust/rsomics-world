#!/usr/bin/env Rscript

script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
script_dir <- dirname(normalizePath(sub("^--file=", "", script)))
source(file.path(script_dir, "infercnv_hmm_workflow.R"))

known <- list(message="smoothing parameter value too small", call="smooth.spline(logv ~ logm)")
observed <- list(status="error", error=known)
stopifnot(hmm_call_outcome(observed, known) == "expected_error")
stopifnot(hmm_call_outcome(list(status="success")) == "success")

must_fail <- function(result, expected=NULL) {
    failure <- tryCatch(hmm_call_outcome(result, expected), error=identity)
    if (!inherits(failure, "error")) stop("unexpected workflow result was accepted")
}

must_fail(observed)
must_fail(list(status="success"), known)
must_fail(list(status="error", error=list(message="unrelated I/O failure", call=known$call)), known)
must_fail(list(status="error", error=list(message=known$message, call="unrelated_function()")), known)
cat("HMM workflow outcome checks pass\n")
