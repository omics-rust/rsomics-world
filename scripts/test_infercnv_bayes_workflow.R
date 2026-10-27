#!/usr/bin/env Rscript

script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
script_dir <- dirname(normalizePath(sub("^--file=", "", script)))
driver <- file.path(script_dir, "infercnv_bayes_workflow.R")
if (file.exists(driver)) source(driver)
if (!exists("bayes_posterior_record") || !exists("bayes_filter_record")) {
    stop("Bayesian posterior/filter evidence record helpers are missing")
}

expression <- matrix(seq_len(6L), 3L, 2L,
                     dimnames=list(c("gene-z", "gene-a", "gene-m"), c("cell-z", "cell-a")))
joins <- list(list(cnv_regions=factor("region-z"), Genes=c(3L, 1L), Cells=c(2L, 1L), State=2L),
              list(cnv_regions=factor("region-a"), Genes=2L, Cells=2L, State=4L))
theta <- list(matrix(c(0.05, 0.05, 0.60, 0.10, 0.10, 0.10,
                       0.05, 0.05, 0.70, 0.05, 0.05, 0.10), 2L, byrow=TRUE),
              matrix(c(0.10, 0.10, 0.50, 0.10, 0.10, 0.10,
                       0.10, 0.10, 0.50, 0.10, 0.10, 0.10), 2L, byrow=TRUE))
theta <- lapply(theta, function(value) {
    colnames(value) <- paste0("theta[", seq_len(6L), "]")
    value
})
epsilon <- list(matrix(c(0, 0, 0.75, 0.25, 0, 0, 0, 0, 1, 0, 0, 0), 6L,
                       dimnames=list(as.character(seq_len(6L)), c("epsilon[2]", "epsilon[1]"))),
                matrix(c(0, 0, 0.5, 0.5, 0, 0), 6L,
                       dimnames=list(as.character(seq_len(6L)), "epsilon[1]")))

posterior <- bayes_posterior_record(joins, theta, epsilon, expression)
stopifnot(identical(posterior$regions$region, c("region-z", "region-a")),
          isTRUE(all.equal(posterior$regions$normal_probability, c(0.65, 0.5))),
          identical(posterior$regions$posterior_max_states, c("3", "3")),
          identical(posterior$joins[[1L]]$genes, c("gene-m", "gene-z")),
          identical(posterior$joins[[1L]]$cells, c("cell-a", "cell-z")),
          identical(posterior$joins[[1L]]$theta_names, paste0("theta[", seq_len(6L), "]")),
          identical(posterior$cells$cell[seq_len(12L)], rep(c("cell-z", "cell-a"), each=6L)),
          identical(posterior$cells$epsilon_variable[seq_len(12L)],
                    rep(c("epsilon[2]", "epsilon[1]"), each=6L)),
          identical(posterior$cells$probability[c(3L, 9L, 15L)], c(0.75, 1, 0.5)))
cat("Posterior evidence preserves actual region/state order and rectangle identities\n")

final_joins <- joins[2L]
final_joins[[1L]]$State <- 3L
original <- matrix(c(2L, 3L, 2L, 2L, 4L, 2L), 3L, dimnames=dimnames(expression))
final <- matrix(3L, 3L, 2L, dimnames=dimnames(expression))
filtered <- bayes_filter_record(posterior, final_joins, original, final, 0.5)
stopifnot(identical(filtered$removed_region_ids, "region-z"),
          identical(filtered$retained_region_ids, "region-a"),
          identical(filtered$reassigned_region_ids, "region-a"),
          identical(filtered$regions$final_state, c(3L, 3L)),
          identical(filtered$regions$removed, c(TRUE, FALSE)),
          filtered$changed_entries == 5L,
          identical(expression, matrix(seq_len(6L), 3L, 2L, dimnames=dimnames(expression))))
bad <- final
bad[3L, 1L] <- 2L
failure <- tryCatch(bayes_filter_record(posterior, final_joins, original, bad, 0.5), error=identity)
stopifnot(inherits(failure, "error"))
cat("Observed filtering records strict-threshold removal and actual neutral reassignment, rejecting wrong rectangles\n")

calibration <- bayes_calibration_record(c(0.01, 0.5, 1, 1.5, 2, 3), rep(100, 6L))
stopifnot(identical(names(calibration), c("state", "mu", "raw_spike_precision")),
          identical(calibration$state, seq_len(6L)), nrow(calibration) == 6L,
          identical(calibration$mu, c(0.01, 0.5, 1, 1.5, 2, 3)))
cat("Calibration evidence does not require names on actual MCMC vectors\n")
