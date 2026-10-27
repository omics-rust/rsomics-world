#!/usr/bin/env Rscript

script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
if (length(script) != 1L) stop("cannot identify script")
script_dir <- dirname(normalizePath(sub("^--file=", "", script)))
source(file.path(script_dir, "infercnv_bayes_workflow.R"))

bayes_performance_parameters <- function(entry, out_dir) {
    parameters <- entry$run_parameters
    if (!identical(parameters$analysis_mode, "samples") || !isTRUE(parameters$HMM) ||
        !identical(parameters$HMM_type, "i6") || !isTRUE(parameters$cluster_by_groups) ||
        isTRUE(parameters$denoise)) stop("performance profile is not the accepted grouped i6 workflow")
    parameters$out_dir <- out_dir
    parameters$up_to_step <- 19L
    parameters$BayesMaxPNormal <- 0.5
    parameters$num_threads <- 1L
    for (name in c("resume_mode", "save_rds", "save_final_rds", "plot_steps", "diagnostics")) {
        parameters[[name]] <- FALSE
    }
    parameters$no_plot <- TRUE
    parameters$no_prelim_plot <- TRUE
    parameters
}

bayes_performance_outputs <- function(root, entry) {
    originals <- basename(as.character(unlist(entry$reports, use.names=FALSE)))
    grouping <- originals[endsWith(originals, ".cell_groupings")]
    prefix <- sub("\\.cell_groupings$", "", grouping)
    suffixes <- c(".cell_groupings", ".genes_used.dat", ".pred_cnv_genes.dat", ".pred_cnv_regions.dat")
    if (length(prefix) != 1L || !startsWith(prefix, "17_HMM_pred") ||
        length(originals) != 4L || !setequal(originals, paste0(prefix, suffixes))) {
        stop("original HMM report identities differ")
    }
    token <- substring(prefix, nchar("17_HMM_pred") + 1L)
    adjusted <- paste0("HMM_CNV_predictions.", token, ".Pnorm_0.5",
                       c(".pred_cnv_regions.dat", ".pred_cnv_genes.dat"))
    result <- list(original_reports=array(file.path("upstream", originals)),
                   adjusted_reports=array(file.path("upstream", adjusted)),
                   mcmc_rds=file.path("upstream", paste0("BayesNetOutput.", token),
                                      "MCMC_inferCNV_obj.rds"))
    for (relative in unlist(result, use.names=FALSE)) bundle_file(root, relative)
    result
}

bayes_performance_main <- function() {
    args <- commandArgs(trailingOnly=TRUE)
    if (length(args) != 3L) stop("usage: infercnv_bayes_performance.R SHIPPED HMM_WORKFLOWS OUTPUT")
    pin <- list(name="infercnv", version="1.28.0",
                source_commit="b421d9405c97a309b081ef86d455e976df93eae4",
                source_archive_sha256="b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207")
    if (!identical(as.character(getRversion()), "4.6.1") ||
        !identical(Sys.getenv("INFER_CNV_SOURCE_COMMIT"), pin$source_commit) ||
        !identical(Sys.getenv("INFER_CNV_SOURCE_SHA256"), pin$source_archive_sha256) ||
        as.character(utils::packageVersion("infercnv")) != pin$version) stop("runtime/source identity differs")
    shipped <- load_bundle(args[[1L]], "shipped", pin)
    hmm <- bayes_load_hmm(args[[2L]], pin)
    if (!identical(hmm$record$source_bundles$shipped, shipped$manifest_sha256)) {
        stop("HMM and raw shipped inputs differ")
    }
    entry <- hmm$record$cases$full
    inputs <- entry$inputs
    if (!identical(inputs$bundle, "shipped")) stop("full performance profile is not raw shipped input")
    root <- normalizePath(args[[3L]], mustWork=FALSE)
    if (file.exists(root)) stop("output directory already exists")
    dir.create(root, recursive=TRUE)
    if (!dir.exists(root)) stop("cannot create output directory")
    metadata <- file.path(root, "metadata")
    dir.create(metadata)
    ns <- asNamespace("infercnv")
    source_identity <- bayes_validate_source(ns, Sys.getenv("INFER_CNV_SOURCE_ARCHIVE"), pin, metadata)
    for (name in c("infercnv_bayes_performance.R", "test_infercnv_bayes_performance.R",
                   "infercnv_bayes_workflow.R", "infercnv_samples_clustering_witness.R", "infercnv_oracle_io.R")) {
        if (!file.copy(file.path(script_dir, name), file.path(metadata, name))) stop("cannot preserve driver")
    }
    writeLines(capture.output(sessionInfo()), file.path(metadata, "session.txt"))
    packages <- utils::installed.packages()[, c("Package", "Version", "LibPath"), drop=FALSE]
    write.table(packages[order(packages[, "Package"], method="radix"), , drop=FALSE],
                file.path(metadata, "packages.tsv"), sep="\t", quote=FALSE, row.names=FALSE)
    versions <- list(R=as.character(getRversion()), infercnv=as.character(utils::packageVersion("infercnv")),
                     rjags=as.character(utils::packageVersion("rjags")), JAGS=as.character(rjags::jags.version()))
    enforce_samples_locale()
    Sys.setenv(OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", MKL_NUM_THREADS="1")
    references <- as.character(unlist(inputs$references, use.names=FALSE))
    create_args <- list(raw_counts_matrix=bundle_file(shipped$root, inputs$counts),
                        gene_order_file=bundle_file(shipped$root, inputs$positions),
                        annotations_file=bundle_file(shipped$root, inputs$annotations),
                        ref_group_names=if (length(references)) references else NULL,
                        delim="\t", max_cells_per_group=NULL, min_max_counts_per_cell=c(1, Inf),
                        chr_exclude=c("chrX", "chrY", "chrM"))
    parameters <- bayes_performance_parameters(entry, file.path(root, "upstream"))
    seed <- 261027L
    set.seed(seed)
    invisible(gc())
    started <- proc.time()
    created <- do.call(infercnv::CreateInfercnvObject, create_args)
    value <- do.call(infercnv::run, c(list(infercnv_obj=created), parameters))
    elapsed <- proc.time() - started
    if (!methods::is(value, "infercnv")) stop("public run did not return an infercnv object")
    outputs <- bayes_performance_outputs(root, entry)
    recorded_create <- list(raw_counts_matrix=inputs$counts, gene_order_file=inputs$positions,
        annotations_file=inputs$annotations, ref_group_names=array(references), delim="\t",
        max_cells_per_group=NULL, min_max_counts_per_cell=list(1, "Inf"),
        chr_exclude=array(c("chrX", "chrY", "chrM")))
    recorded_parameters <- parameters
    recorded_parameters$out_dir <- "upstream"
    input_hashes <- lapply(inputs[c("counts", "positions", "annotations")],
                           function(relative) shipped$hashes[[relative]])
    record <- c(list(schema_version=1L, status="complete", package=pin, versions=versions,
        source=source_identity, source_hmm_witness_sha256=hmm$witness_sha256,
        shipped_manifest_sha256=shipped$manifest_sha256, inputs=inputs, input_sha256=input_hashes,
        seed=seed, JAGS_seed="not explicitly set", create_parameters=recorded_create,
        run_parameters=recorded_parameters,
        public_call_seconds=unname(elapsed[["elapsed"]]),
        user_seconds=unname(sum(elapsed[c("user.self", "user.child")])),
        system_seconds=unname(sum(elapsed[c("sys.self", "sys.child")])),
        expression=list(genes=nrow(value@expr.data), cells=ncol(value@expr.data),
                        gene_ids=array(rownames(value@expr.data)), cell_ids=array(colnames(value@expr.data))),
        timing_scope="raw CreateInfercnvObject plus run through19; excludes startup and evidence export",
        memory_scope="whole measured R process; no MCMC reload or matrix export"), outputs)
    jsonlite::write_json(record, file.path(root, "trial.json"), auto_unbox=TRUE, pretty=TRUE,
                         null="null", na="null", digits=17)
    cat("Completed actual raw grouped-i6 Bayesian workflow in ",
        record$public_call_seconds, " seconds\n", sep="")
}

if (sys.nframe() == 0L) bayes_performance_main()
