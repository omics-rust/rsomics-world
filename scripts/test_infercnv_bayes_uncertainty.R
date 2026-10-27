#!/usr/bin/env Rscript

script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
script_dir <- dirname(normalizePath(sub("^--file=", "", script)))
driver <- file.path(script_dir, "infercnv_bayes_uncertainty.R")
if (file.exists(driver)) source(driver)
if (!exists("bayes_reduce_samples")) stop("Bayesian uncertainty reducer is missing")

directory <- tempfile("rsomics-bayes-uncertainty-")
dir.create(directory)
samples <- lapply(seq_len(6L), function(chain) {
    values <- cbind("theta[2]"=rep(((chain - 1L) * 20L + seq_len(20L)) / 200, each=50L),
                    "epsilon[12]"=rep(c(rep(1L, chain), rep(2L, 20L - chain)), each=50L),
                    "epsilon[2]"=rep(if (chain == 6L) 6L else 3L, 1000L),
                    "theta[1]"=rep(0.8, 1000L))
    values[, ((seq_len(4L) + chain - 2L) %% 4L) + 1L, drop=FALSE]
})
saveRDS(list(samples=samples, data=list(C=12L)), file.path(directory, "region-0010.rds"))
saveRDS(list(samples=samples, data=list(C=12L)), file.path(directory, "region-0002.rds"))
saveRDS("not a region sample", file.path(directory, "region-1.rds"))
before <- tools::md5sum(list.files(directory, full.names=TRUE))
result <- bayes_reduce_samples(directory)
stopifnot(identical(unique(result$region_index), c(2L, 10L)), nrow(result) == 28L,
          identical(result$variable[seq_len(14L)], c("theta[2]", rep("epsilon[12]", 6L),
                                                      rep("epsilon[2]", 6L), "theta[1]")))
theta <- result[result$region_index == 2L & result$variable == "theta[2]", ]
epsilon <- result[result$region_index == 2L & result$variable == "epsilon[12]", ]
other <- result[result$region_index == 2L & result$variable == "epsilon[2]", ]
near <- function(actual, expected) isTRUE(all.equal(unname(actual), expected, tolerance=1e-12))
stopifnot(theta$state == 2L, near(theta$posterior_mean, 0.3025),
          near(theta$batch_standard_error, 11 / (200 * sqrt(12))),
          near(as.numeric(theta[paste0("chain_mean_", seq_len(6L))]),
               c(0.0525, 0.1525, 0.2525, 0.3525, 0.4525, 0.5525)),
          identical(epsilon$state, seq_len(6L)),
          near(epsilon$posterior_mean, c(0.175, 0.825, 0, 0, 0, 0)),
          near(epsilon$batch_standard_error[[1L]], sqrt(0.175 * 0.825 / 119)),
          near(as.numeric(epsilon[1L, paste0("chain_mean_", seq_len(6L))]),
               c(0.05, 0.10, 0.15, 0.20, 0.25, 0.30)),
          near(other$posterior_mean, c(0, 0, 5 / 6, 0, 0, 1 / 6)),
          near(as.numeric(other[3L, paste0("chain_mean_", seq_len(6L))]), c(1, 1, 1, 1, 1, 0)),
          identical(tools::md5sum(list.files(directory, full.names=TRUE)), before))
output <- file.path(directory, "summary.tsv")
status <- system2(file.path(R.home("bin"), "Rscript"), c("--vanilla", shQuote(driver),
                  shQuote(directory), shQuote(output)))
written <- read.delim(output, check.names=FALSE)
stopifnot(status == 0L, identical(names(written), c("region_index", "variable", "state",
          "posterior_mean", "batch_standard_error", paste0("chain_mean_", seq_len(6L)))),
          near(written$posterior_mean, result$posterior_mean),
          near(written$batch_standard_error, result$batch_standard_error),
          identical(tools::md5sum(names(before)), before))
cat("Bayesian reduction preserves variable identities, six-chain batch uncertainty, file order and original samples\n")
