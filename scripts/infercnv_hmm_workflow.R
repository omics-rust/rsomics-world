#!/usr/bin/env Rscript

script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
if (length(script) != 1L) stop("cannot identify script")
script_dir <- dirname(normalizePath(sub("^--file=", "", script)))
source(file.path(script_dir, "infercnv_samples_clustering_witness.R"))
source(file.path(script_dir, "infercnv_oracle_io.R"))

.hmm_workflow_exit <- function(frame, value) {
    if (!methods::is(value, "infercnv")) return(invisible(NULL))
    capture <- get(".hmm_workflow_capture", envir=.GlobalEnv)
    capture$hmm_calls <- capture$hmm_calls + 1L
    capture$hmm <- list(input=get("infercnv_obj", frame), states=value,
                        means=get("cnv_mean_sd", frame),
                        fits=get("cnv_level_to_mean_sd_fit", frame),
                        groups=get("tumor_samples", frame))
}

hmm_groups <- function(groups) {
    unname(lapply(seq_along(groups), function(i) {
        list(name=names(groups)[[i]], indices=array(groups[[i]]))
    }))
}

hmm_workflow_main <- function() {
    args <- commandArgs(trailingOnly=TRUE)
    if (length(args) != 3L) stop("usage: infercnv_hmm_workflow.R SYNTHETIC SHIPPED OUTPUT")
    pin <- list(name="infercnv", version="1.28.0",
                source_commit="b421d9405c97a309b081ef86d455e976df93eae4",
                source_archive_sha256="b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207")
    if (!identical(as.character(getRversion()), "4.6.1") ||
        !identical(Sys.getenv("INFER_CNV_SOURCE_COMMIT"), pin$source_commit) ||
        !identical(Sys.getenv("INFER_CNV_SOURCE_SHA256"), pin$source_archive_sha256) ||
        as.character(utils::packageVersion("infercnv")) != pin$version) stop("runtime/source identity differs")
    synthetic <- load_bundle(args[[1L]], "synthetic", pin)
    shipped <- load_bundle(args[[2L]], "shipped", pin)
    root <- normalizePath(args[[3L]], mustWork=FALSE)
    if (file.exists(root)) stop("output directory already exists")
    dir.create(root, recursive=TRUE)
    if (!dir.exists(root)) stop("cannot create output directory")
    metadata <- file.path(root, "metadata")
    dir.create(metadata)
    for (name in c("infercnv_hmm_workflow.R", "infercnv_samples_clustering_witness.R", "infercnv_oracle_io.R")) {
        if (!file.copy(file.path(script_dir, name), file.path(metadata, name))) stop("cannot retain script")
    }
    writeLines(capture.output(sessionInfo()), file.path(metadata, "session.txt"))
    packages <- utils::installed.packages()[, c("Package", "Version", "LibPath"), drop=FALSE]
    write.table(packages, file.path(metadata, "packages.tsv"), sep="\t", quote=FALSE, row.names=FALSE)
    enforce_samples_locale()
    Sys.setenv(OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", MKL_NUM_THREADS="1")
    ns <- asNamespace("infercnv")
    context <- get("infercnv.env", ns)
    capture <- new.env(parent=emptyenv())
    assign(".hmm_workflow_capture", capture, envir=.GlobalEnv)
    predictor <- "predict_CNV_via_HMM_on_whole_tumor_samples"
    trace(predictor, where=ns, print=FALSE,
          exit=quote(get(".hmm_workflow_exit", .GlobalEnv)(environment(), returnValue())))
    on.exit(untrace(predictor, where=ns))
    sources <- list(grouped=list(bundle=synthetic, kind="synthetic", profile="grouped_bounds"),
                    proxy=list(bundle=synthetic, kind="synthetic", profile="no_reference"),
                    full=list(bundle=shipped, kind="shipped", profile="full"))
    seed <- 261027L
    cases <- list()
    for (id in names(sources)) {
        source <- sources[[id]]
        bundle <- source$bundle
        entry <- if (source$kind == "synthetic") bundle$record$profiles[[source$profile]] else bundle$record$cases[[source$profile]]
        settings <- if (source$kind == "synthetic") bundle$record$settings else bundle$record$run_settings
        if (source$kind == "synthetic") settings$ref_subtract_use_mean_bounds <- entry$settings$ref_subtract_use_mean_bounds
        refs <- if (source$kind == "synthetic") entry$settings$ref_group_names else entry$reference_groups
        refs <- as.character(unlist(refs, use.names=FALSE))
        inputs <- if (source$kind == "synthetic") {
            list(counts=bundle$record$inputs[["counts.tsv"]], positions=bundle$record$inputs[["gene_order.tsv"]],
                 annotations=bundle$record$inputs[["annotations.tsv"]])
        } else list(counts=bundle$record$inputs[[entry$input_counts]], positions=bundle$record$inputs$original_gene_order,
                    annotations=bundle$record$inputs$original_annotations)
        relative <- file.path("cases", id)
        directory <- file.path(root, relative)
        dir.create(directory, recursive=TRUE)
        run_args <- run_parameters(settings, TRUE, 0.8, file.path(directory, "upstream"))
        run_args$save_rds <- FALSE
        run_args$up_to_step <- 17L
        run_args$HMM <- TRUE
        run_args$HMM_type <- "i6"
        run_args$HMM_report_by <- "subcluster"
        run_args$BayesMaxPNormal <- 0
        create_args <- list(raw_counts_matrix=bundle_file(bundle$root, inputs$counts),
                            gene_order_file=bundle_file(bundle$root, inputs$positions),
                            annotations_file=bundle_file(bundle$root, inputs$annotations),
                            ref_group_names=if (length(refs)) refs else NULL,
                            delim="\t", max_cells_per_group=NULL, min_max_counts_per_cell=c(1, Inf),
                            chr_exclude=c("chrX", "chrY", "chrM"))
        capture$hmm <- NULL
        capture$hmm_calls <- 0L
        started <- proc.time()[["elapsed"]]
        result <- capture_call(function() {
            obj <- do.call(infercnv::CreateInfercnvObject, create_args)
            do.call(infercnv::run, c(list(infercnv_obj=obj), run_args))
        }, file.path(directory, "call.log"), capture, context, seed)
        wall <- proc.time()[["elapsed"]] - started
        if (result$status != "success") {
            jsonlite::write_json(result, file.path(directory, "failure.json"), auto_unbox=TRUE, pretty=TRUE, null="null")
            stop("actual raw HMM workflow failed: ", id)
        }
        hmm <- capture$hmm
        if (capture$hmm_calls != 1L || is.null(hmm) || is.null(hmm$input@.hspike) ||
            !identical(result$value@expr.data, hmm$input@expr.data) ||
            !identical(dimnames(hmm$states@expr.data), dimnames(hmm$input@expr.data)) ||
            any(!hmm$states@expr.data %in% seq_len(6L))) stop("invalid actual HMM result contract")
        expression <- export_state(result$value, file.path(relative, "expression"), root)
        states <- export_state(hmm$states, file.path(relative, "states"), root)
        spike <- export_state(hmm$input@.hspike, file.path(relative, "spike"), root)
        replay <- file.path(relative, "actual-hmm-input-and-result.rds")
        saveRDS(list(input=hmm$input, states=hmm$states), file.path(root, replay))
        reports <- list.files(file.path(directory, "upstream"), pattern="^17_HMM_pred.*\\.(cell_groupings|pred_cnv_regions\\.dat|pred_cnv_genes\\.dat|genes_used\\.dat)$")
        if (length(reports) != 4L) stop("actual stage17 reports are incomplete")
        calibration <- list(means=hmm$means, coefficients=lapply(hmm$fits, stats::coef),
                            fitted_inputs=lapply(hmm$fits, function(fit) unname(as.matrix(fit$model))))
        capture$hmm <- NULL
        pooled <- capture_call(function() {
            get(predictor, ns)(hmm$input, cluster_by_groups=FALSE, cnv_mean_sd=hmm$means,
                               cnv_level_to_mean_sd_fit=hmm$fits, t=run_args$HMM_transition_prob)
        }, file.path(directory, "pooled-call.log"), capture, context, seed)
        pooled_record <- list(status=pooled$status, warnings=pooled$warnings)
        if (pooled$status == "success") {
            pooled_record$groups <- hmm_groups(capture$hmm$groups)
            pooled_record$states <- export_state(pooled$value, file.path(relative, "pooled-states"), root)
        } else pooled_record$error <- pooled$error
        cases[[id]] <- list(status="success", inputs=c(list(bundle=source$kind, references=array(refs)), inputs),
            run_parameters=run_args, expression=expression, states=states, spike=spike,
            calibration=calibration, groups=hmm_groups(hmm$groups), replay=replay,
            reports=array(file.path(relative, "upstream", reports)), pooled=pooled_record,
            warnings=result$warnings, raw_create_to_return_wall_seconds=wall)
        cat("Completed actual raw i6 workflow: ", id, "; pooled route=", pooled$status,
            "; genes=", nrow(hmm$states@expr.data), "; cells=", ncol(hmm$states@expr.data), "\n", sep="")
    }
    paths <- sort(list.files(root, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
    files <- stats::setNames(lapply(paths, function(path) hash_file(file.path(root, path))), paths)
    record <- list(schema_version=1L, status="complete", package=pin,
        scope="actual raw-to-i6 grouped samples, calibrated states and stage17 reports; no Bayesian or native acceptance",
        seed=seed, rng_kind=array(RNGkind()), timing="in-process raw create through return including trace; not paired performance",
        source_bundles=list(synthetic=synthetic$manifest_sha256, shipped=shipped$manifest_sha256),
        cases=cases, files=files)
    jsonlite::write_json(record, file.path(root, "witness.json"), auto_unbox=TRUE, pretty=TRUE,
                         null="null", na="null", digits=17)
}

hmm_workflow_main()
