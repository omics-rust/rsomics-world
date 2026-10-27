#!/usr/bin/env Rscript

script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
script_dir <- dirname(normalizePath(sub("^--file=", "", script)))
driver <- file.path(script_dir, "infercnv_bayes_performance.R")
if (file.exists(driver)) source(driver)
if (!exists("bayes_performance_parameters") || !exists("bayes_performance_outputs")) {
    stop("whole raw Bayesian performance contract helpers are missing")
}

entry <- list(run_parameters=list(analysis_mode="samples", HMM=TRUE, HMM_type="i6",
    cluster_by_groups=TRUE, HMM_transition_prob=1e-6, BayesMaxPNormal=0,
    up_to_step=17L, num_threads=4L, resume_mode=TRUE, save_rds=TRUE,
    save_final_rds=TRUE, no_plot=FALSE, no_prelim_plot=FALSE, plot_steps=TRUE,
    diagnostics=TRUE, denoise=FALSE),
    reports=paste0("cases/full/upstream/17_HMM_predHMMi6.hmm_mode-samples",
        c(".cell_groupings", ".genes_used.dat", ".pred_cnv_genes.dat", ".pred_cnv_regions.dat")))
root <- tempfile("bayes-performance-")
dir.create(root)
upstream <- file.path(root, "upstream")
parameters <- bayes_performance_parameters(entry, upstream)
stopifnot(parameters$up_to_step == 19L, parameters$BayesMaxPNormal == 0.5,
          parameters$num_threads == 1L, parameters$HMM_transition_prob == 1e-6,
          !parameters$resume_mode, !parameters$save_rds, !parameters$save_final_rds,
          parameters$no_plot, parameters$no_prelim_plot, !parameters$plot_steps,
          !parameters$diagnostics, !parameters$denoise, parameters$out_dir == upstream)
cat("Actual-run parameters include Bayesian filtering, not the retained stage17 replay\n")

original <- paste0("upstream/17_HMM_predHMMi6.hmm_mode-samples",
    c(".cell_groupings", ".genes_used.dat", ".pred_cnv_genes.dat", ".pred_cnv_regions.dat"))
adjusted <- paste0("upstream/HMM_CNV_predictions.HMMi6.hmm_mode-samples.Pnorm_0.5",
    c(".pred_cnv_regions.dat", ".pred_cnv_genes.dat"))
mcmc <- "upstream/BayesNetOutput.HMMi6.hmm_mode-samples/MCMC_inferCNV_obj.rds"
for (relative in c(original, adjusted, mcmc)) {
    destination <- file.path(root, relative)
    dir.create(dirname(destination), recursive=TRUE, showWarnings=FALSE)
    writeLines("retained source output", destination)
}
outputs <- bayes_performance_outputs(root, entry)
stopifnot(identical(unlist(outputs$original_reports, use.names=FALSE), original),
          identical(unlist(outputs$adjusted_reports, use.names=FALSE), adjusted),
          identical(outputs$mcmc_rds, mcmc))
incomplete <- tempfile("bayes-performance-incomplete-")
dir.create(incomplete)
failure <- tryCatch(bayes_performance_outputs(incomplete, entry), error=identity)
stopifnot(inherits(failure, "error"))
cat("Evidence paths require actual original, adjusted and automatic MCMC outputs\n")
