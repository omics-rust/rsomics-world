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

if ("--installed" %in% commandArgs(trailingOnly=TRUE)) {
    stopifnot(requireNamespace("infercnv", quietly=TRUE))
    object <- probe_object(matrix(c(0.8, 0.9, 1.1, 1.2), 2L), rep("sample", 2L))
    expression <- object@expr.data
    capture <- new.env(parent=emptyenv())
    capture$hmm_calls <- 0L
    assign(".hmm_workflow_capture", capture, envir=.GlobalEnv)
    changing_predictor <- function(infercnv_obj) {
        cnv_mean_sd <- list()
        cnv_level_to_mean_sd_fit <- list()
        tumor_samples <- infercnv_obj@observation_grouped_cell_indices
        infercnv_obj@expr.data[,] <- 3
        infercnv_obj
    }
    trace("changing_predictor", where=.GlobalEnv, print=FALSE,
          tracer=quote(get(".hmm_workflow_enter", .GlobalEnv)(environment())),
          exit=quote(get(".hmm_workflow_exit", .GlobalEnv)(environment(), returnValue())))
    states <- changing_predictor(object)
    untrace("changing_predictor", where=.GlobalEnv)
    stopifnot(capture$hmm_calls == 1L,
              identical(capture$hmm$input@expr.data, expression),
              identical(object@expr.data, expression),
              identical(capture$hmm$states@expr.data, states@expr.data),
              all(states@expr.data == 3),
              !identical(expression, states@expr.data))
    cat("HMM trace preserves the pre-prediction expression snapshot\n")
}
