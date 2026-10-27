#!/usr/bin/env Rscript

script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
if (length(script) != 1L) stop("cannot identify script")
script_dir <- dirname(normalizePath(sub("^--file=", "", script)))
source(file.path(script_dir, "infercnv_hmm_workflow.R"))

subcluster_cut_record <- function(tree, input_indices, members, method, probability) {
    height <- NULL
    labels <- rep(1L, length(input_indices))
    if (!is.null(tree)) {
        height <- switch(method,
            pheight=probability * max(tree$height),
            qnorm=stats::qnorm(1-probability, mean(tree$height), stats::sd(tree$height)),
            stop("unsupported Ward characterization method"))
        labels <- unname(stats::cutree(tree, h=height))
    }
    assigned <- lapply(members, function(indices) unique(labels[match(indices, input_indices)]))
    matches <- all(lengths(assigned) == 1L) && !anyNA(unlist(assigned)) &&
        length(unique(unlist(assigned))) == length(assigned)
    list(cut_height=if (is.null(height)) NULL else sprintf("%.17g", height),
         cut_labels=array(labels), matches_cutree_partition=matches,
         subclusters=unname(lapply(seq_along(members), function(i) {
             list(name=names(members)[[i]], indices=array(unname(members[[i]])),
                  cut_labels=array(assigned[[i]]))
         })))
}

.subcluster_enter <- function(frame) {
    capture <- get(".samples_trace_capture", .GlobalEnv)
    capture$cluster_depth <- capture$cluster_depth + 1L
    if (capture$cluster_depth == 1L) .samples_trace_enter(frame)
}

.subcluster_exit <- function(frame, value) {
    capture <- get(".samples_trace_capture", .GlobalEnv)
    if (capture$cluster_depth == 1L) {
        .samples_trace_exit(frame)
        capture$cluster_result <- value
    }
    capture$cluster_depth <- capture$cluster_depth - 1L
}

.subcluster_single_enter <- function(frame) {
    if (get(".samples_trace_capture", .GlobalEnv)$cluster_depth == 1L) .samples_single_enter(frame)
}

.subcluster_hmm_exit <- function(frame, value) {
    if (!methods::is(value, "infercnv")) return(invisible(NULL))
    capture <- get(".hmm_workflow_capture", .GlobalEnv)
    capture$hmm_calls <- capture$hmm_calls + 1L
    capture$hmm <- list(input=capture$hmm_input, states=value,
        means=get("cnv_mean_sd", frame), fits=get("cnv_level_to_mean_sd_fit", frame),
        groups=unlist(capture$hmm_input@tumor_subclusters[["subclusters"]], recursive=FALSE))
}

subclusters_workflow_main <- function() {
    args <- commandArgs(trailingOnly=TRUE)
    if (length(args) != 2L) stop("usage: infercnv_subclusters_workflow.R SHIPPED OUTPUT")
    pin <- list(name="infercnv", version="1.28.0",
        source_commit="b421d9405c97a309b081ef86d455e976df93eae4",
        source_archive_sha256="b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207")
    if (!identical(as.character(getRversion()), "4.6.1") ||
        !identical(Sys.getenv("INFER_CNV_SOURCE_COMMIT"), pin$source_commit) ||
        !identical(Sys.getenv("INFER_CNV_SOURCE_SHA256"), pin$source_archive_sha256) ||
        as.character(utils::packageVersion("infercnv")) != pin$version) stop("runtime/source identity differs")
    bundle <- load_bundle(args[[1L]], "shipped", pin)
    root <- normalizePath(args[[2L]], mustWork=FALSE)
    if (file.exists(root)) stop("output directory already exists")
    dir.create(root, recursive=TRUE)
    if (!dir.exists(root)) stop("cannot create output directory")
    metadata <- file.path(root, "metadata")
    dir.create(metadata)
    for (name in c("infercnv_subclusters_workflow.R", "test_infercnv_subclusters_workflow.R",
                   "infercnv_hmm_workflow.R", "infercnv_samples_clustering_witness.R", "infercnv_oracle_io.R")) {
        if (!file.copy(file.path(script_dir, name), file.path(metadata, name))) stop("cannot retain script")
    }
    writeLines(capture.output(sessionInfo()), file.path(metadata, "session.txt"))
    write.table(utils::installed.packages()[, c("Package", "Version", "LibPath"), drop=FALSE],
                file.path(metadata, "packages.tsv"), sep="\t", quote=FALSE, row.names=FALSE)
    enforce_samples_locale()
    Sys.setenv(OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", MKL_NUM_THREADS="1")
    ns <- asNamespace("infercnv")
    context <- get("infercnv.env", ns)
    capture <- new.env(parent=emptyenv())
    capture$cluster_depth <- 0L
    assign(".samples_trace_capture", capture, .GlobalEnv)
    assign(".hmm_workflow_capture", capture, .GlobalEnv)
    trace("define_signif_tumor_subclusters", where=ns, print=FALSE,
        tracer=quote(get(".subcluster_enter", .GlobalEnv)(environment())),
        exit=quote(get(".subcluster_exit", .GlobalEnv)(environment(), returnValue())))
    on.exit(untrace("define_signif_tumor_subclusters", where=ns), add=TRUE)
    trace(".single_tumor_subclustering", where=ns, print=FALSE,
        tracer=quote(get(".subcluster_single_enter", .GlobalEnv)(environment())))
    on.exit(untrace(".single_tumor_subclustering", where=ns), add=TRUE)
    predictor <- "predict_CNV_via_HMM_on_tumor_subclusters"
    trace(predictor, where=ns, print=FALSE,
        tracer=quote(get(".hmm_workflow_enter", .GlobalEnv)(environment())),
        exit=quote(get(".subcluster_hmm_exit", .GlobalEnv)(environment(), returnValue())))
    on.exit(untrace(predictor, where=ns), add=TRUE)
    profiles <- list(pheight=list(method="pheight", probability=0.5),
                     qnorm=list(method="qnorm", probability=0.1))
    entry <- bundle$record$cases$full
    refs <- as.character(unlist(entry$reference_groups, use.names=FALSE))
    inputs <- list(counts=bundle$record$inputs[[entry$input_counts]],
                   positions=bundle$record$inputs$original_gene_order,
                   annotations=bundle$record$inputs$original_annotations)
    seed <- 261027L
    cases <- list()
    for (id in names(profiles)) {
        profile <- profiles[[id]]
        relative <- file.path("cases", id)
        directory <- file.path(root, relative)
        dir.create(directory, recursive=TRUE)
        run_args <- run_parameters(bundle$record$run_settings, TRUE, 0.8, file.path(directory, "upstream"))
        run_args$analysis_mode <- "subclusters"
        run_args$tumor_subcluster_partition_method <- profile$method
        run_args$tumor_subcluster_pval <- profile$probability
        run_args$HMM <- TRUE
        run_args$HMM_type <- "i6"
        run_args$HMM_report_by <- "subcluster"
        run_args$BayesMaxPNormal <- 0
        run_args$up_to_step <- 17L
        run_args$save_rds <- FALSE
        capture$hmm <- capture$cluster_result <- NULL
        capture$hmm_calls <- 0L
        capture$cluster_depth <- 0L
        cat("Running actual raw Ward/i6 workflow: ", id, "\n", sep="")
        started <- proc.time()[["elapsed"]]
        result <- capture_call(function() {
            object <- infercnv::CreateInfercnvObject(
                raw_counts_matrix=bundle_file(bundle$root, inputs$counts),
                gene_order_file=bundle_file(bundle$root, inputs$positions),
                annotations_file=bundle_file(bundle$root, inputs$annotations),
                ref_group_names=refs, delim="\t", max_cells_per_group=NULL,
                min_max_counts_per_cell=c(1, Inf), chr_exclude=c("chrX", "chrY", "chrM"))
            do.call(infercnv::run, c(list(infercnv_obj=object), run_args))
        }, file.path(directory, "call.log"), capture, context, seed)
        wall <- proc.time()[["elapsed"]] - started
        if (result$status != "success") {
            jsonlite::write_json(result, file.path(directory, "failure.json"), auto_unbox=TRUE,
                                 pretty=TRUE, null="null")
        }
        hmm_call_outcome(result)
        hmm <- capture$hmm
        if (capture$entries != 1L || capture$cluster_depth != 0L || capture$hmm_calls != 1L ||
            is.null(hmm) || !identical(result$value@expr.data, hmm$input@expr.data) ||
            !identical(dimnames(hmm$states@expr.data), dimnames(hmm$input@expr.data)) ||
            any(!hmm$states@expr.data %in% seq_len(6L))) stop("invalid actual subcluster result contract")
        groups <- export_groups(capture$cluster_result[[1L]], capture, ns)
        cuts <- unname(lapply(seq_along(groups), function(i) {
            name <- groups[[i]]$name
            members <- hmm$input@tumor_subclusters$subclusters[[name]]
            c(list(name=name), subcluster_cut_record(hmm$input@tumor_subclusters$hc[[name]],
                unlist(groups[[i]]$input_indices), members, profile$method, profile$probability))
        }))
        expression <- export_state(result$value, file.path(relative, "expression"), root)
        states <- export_state(hmm$states, file.path(relative, "states"), root)
        spike <- export_state(hmm$input@.hspike, file.path(relative, "spike"), root)
        diagnostic <- export_diagnostic(capture, relative, root, run_args)
        replay <- file.path(relative, "actual-subcluster-input-and-result.rds")
        saveRDS(list(input=hmm$input, states=hmm$states), file.path(root, replay))
        reports <- list.files(file.path(directory, "upstream"), pattern="^17_HMM_pred.*\\.(cell_groupings|pred_cnv_regions\\.dat|pred_cnv_genes\\.dat|genes_used\\.dat)$")
        if (length(reports) != 4L) stop("actual stage17 reports are incomplete")
        cases[[id]] <- list(status="success", inputs=c(list(bundle="shipped", references=array(refs)), inputs),
            run_parameters=run_args, expression=expression, states=states, spike=spike,
            groups=groups, cuts=cuts, hmm_groups=hmm_groups(hmm$groups), diagnostic=diagnostic,
            calibration=list(means=hmm$means, coefficients=lapply(hmm$fits, stats::coef),
                fitted_inputs=lapply(hmm$fits, function(fit) unname(as.matrix(fit$model)))),
            reports=array(file.path(relative, "upstream", reports)), replay=replay,
            warnings=result$warnings, raw_create_to_return_wall_seconds=wall)
        cat("Completed actual Ward/i6 workflow: ", id, "; subclusters=", length(hmm$groups),
            "; partition mismatches=", sum(!vapply(cuts, `[[`, logical(1), "matches_cutree_partition")), "\n", sep="")
    }
    probe <- capture_call(function() {
        get(".single_tumor_subclustering", ns)("probe", stats::setNames(1:3, c("a", "b", "c")),
            matrix(c(1, 11, 1.1), nrow=1L, dimnames=list("gene", c("a", "b", "c"))),
            p_val=0.5, hclust_method="ward.D2", partition_method="pheight")
    }, file.path(root, "membership-probe.log"), capture, context, seed)
    hmm_call_outcome(probe)
    saveRDS(probe$value, file.path(root, "actual-membership-probe.rds"))
    membership_probe <- subcluster_cut_record(probe$value$hc, 1:3, probe$value$subclusters, "pheight", 0.5)
    membership_probe$tree <- tree_signature(probe$value$hc)
    paths <- sort(list.files(root, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
    record <- list(schema_version=1L, status="complete", package=pin, seed=seed,
        rng_kind=array(RNGkind()), source_bundles=list(shipped=bundle$manifest_sha256),
        scope="actual raw Ward-subcluster i6 states and reports; membership characterization, not native acceptance",
        timing="in-process raw create through return including traces; not paired performance",
        successful_cases=array(names(cases)), cases=cases, membership_probe=membership_probe,
        files=stats::setNames(lapply(paths, function(path) hash_file(file.path(root, path))), paths))
    jsonlite::write_json(record, file.path(root, "witness.json"), auto_unbox=TRUE,
                         pretty=TRUE, null="null", na="null", digits=17)
}

if (sys.nframe() == 0L) subclusters_workflow_main()
