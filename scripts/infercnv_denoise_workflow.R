#!/usr/bin/env Rscript

script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
if (length(script) != 1L) stop("cannot identify script")
script_dir <- dirname(normalizePath(sub("^--file=", "", script)))
source(file.path(script_dir, "infercnv_samples_clustering_witness.R"))
source(file.path(script_dir, "infercnv_oracle_io.R"))

.denoise_trace_exit <- function(frame) {
    capture <- get(".samples_trace_capture", envir=.GlobalEnv)
    width <- if (exists("mean_ref_sd", frame, inherits=FALSE)) {
        get("mean_ref_sd", frame, inherits=FALSE)
    } else get("threshold", frame, inherits=FALSE)
    capture$noise <- list(center=get("mean_ref_vals", frame, inherits=FALSE), threshold=width)
    capture$noise_calls <- capture$noise_calls + 1L
}

denoise_workflow_main <- function() {
    args <- commandArgs(trailingOnly=TRUE)
    if (length(args) != 3L) stop("usage: infercnv_denoise_workflow.R SYNTHETIC SHIPPED OUTPUT")
    pin <- list(name="infercnv", version="1.28.0",
                source_commit="b421d9405c97a309b081ef86d455e976df93eae4",
                source_archive_sha256="b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207")
    if (!identical(as.character(getRversion()), "4.6.1") ||
        !identical(Sys.getenv("INFER_CNV_SOURCE_COMMIT"), pin$source_commit) ||
        !identical(Sys.getenv("INFER_CNV_SOURCE_SHA256"), pin$source_archive_sha256)) {
        stop("runtime/source identity differs")
    }
    if (as.character(utils::packageVersion("infercnv")) != pin$version) stop("infercnv version differs")
    synthetic <- load_bundle(args[[1L]], "synthetic", pin)
    shipped <- load_bundle(args[[2L]], "shipped", pin)
    root <- normalizePath(args[[3L]], mustWork=FALSE)
    if (file.exists(root)) stop("output directory already exists")
    dir.create(root, recursive=TRUE)
    if (!dir.exists(root)) stop("cannot create output directory")
    metadata <- file.path(root, "metadata")
    dir.create(metadata)
    writeLines(capture.output(sessionInfo()), file.path(metadata, "session.txt"))
    packages <- utils::installed.packages()[, c("Package", "Version", "LibPath"), drop=FALSE]
    write.table(packages, file.path(metadata, "packages.tsv"), sep="\t", quote=FALSE, row.names=FALSE)
    for (name in c("infercnv_denoise_workflow.R", "infercnv_samples_clustering_witness.R", "infercnv_oracle_io.R")) {
        if (!file.copy(file.path(script_dir, name), file.path(metadata, name))) stop("cannot retain script")
    }
    enforce_samples_locale()
    Sys.setenv(OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", MKL_NUM_THREADS="1")
    ns <- asNamespace("infercnv")
    context <- get("infercnv.env", envir=ns)
    capture <- new.env(parent=emptyenv())
    assign(".samples_trace_capture", capture, envir=.GlobalEnv)
    trace("define_signif_tumor_subclusters", where=ns, print=FALSE,
          tracer=quote(get(".samples_trace_enter", .GlobalEnv)(environment())),
          exit=quote(get(".samples_trace_exit", .GlobalEnv)(environment())))
    trace(".single_tumor_subclustering", where=ns, print=FALSE,
          tracer=quote(get(".samples_single_enter", .GlobalEnv)(environment())))
    for (name in c("clear_noise", "clear_noise_via_ref_mean_sd")) {
        trace(name, where=ns, print=FALSE,
              exit=quote(get(".denoise_trace_exit", .GlobalEnv)(environment())))
    }
    on.exit({
        for (name in c("define_signif_tumor_subclusters", ".single_tumor_subclustering",
                       "clear_noise", "clear_noise_via_ref_mean_sd")) untrace(name, where=ns)
    })
    sources <- list(grouped=list(bundle=synthetic, kind="synthetic", profile="grouped_bounds"),
                    proxy=list(bundle=synthetic, kind="synthetic", profile="no_reference"),
                    full=list(bundle=shipped, kind="shipped", profile="full"))
    modes <- list(off=list(enabled=FALSE, fixed=NULL, logistic=FALSE),
                  sd_hard=list(enabled=TRUE, fixed=NULL, logistic=FALSE),
                  fixed_hard=list(enabled=TRUE, fixed=0.1, logistic=FALSE),
                  sd_logistic=list(enabled=TRUE, fixed=NULL, logistic=TRUE),
                  fixed_logistic=list(enabled=TRUE, fixed=0.1, logistic=TRUE))
    cases <- list()
    for (source_name in names(sources)) for (mode_name in names(modes)) {
        source <- sources[[source_name]]
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
        mode <- modes[[mode_name]]
        id <- paste(source_name, mode_name, sep="_")
        relative <- file.path("cases", id)
        directory <- file.path(root, relative)
        dir.create(directory, recursive=TRUE)
        run_args <- run_parameters(settings, FALSE, 0.8, file.path(directory, "upstream"))
        run_args$save_rds <- FALSE
        run_args$up_to_step <- if (mode$enabled) 22L else 15L
        run_args$denoise <- mode$enabled
        run_args$noise_filter <- if (is.null(mode$fixed)) NA_real_ else mode$fixed
        run_args$sd_amplifier <- 1.5
        run_args$noise_logistic <- mode$logistic
        create_args <- list(raw_counts_matrix=bundle_file(bundle$root, inputs$counts),
                            gene_order_file=bundle_file(bundle$root, inputs$positions),
                            annotations_file=bundle_file(bundle$root, inputs$annotations),
                            ref_group_names=if (length(refs)) refs else NULL,
                            delim="\t", max_cells_per_group=NULL, min_max_counts_per_cell=c(1, Inf),
                            chr_exclude=c("chrX", "chrY", "chrM"))
        capture$noise <- NULL
        capture$noise_calls <- 0L
        started <- proc.time()[["elapsed"]]
        result <- capture_call(function() {
            obj <- do.call(infercnv::CreateInfercnvObject, create_args)
            do.call(infercnv::run, c(list(infercnv_obj=obj), run_args))
        }, file.path(directory, "call.log"), capture, context, 261026L)
        wall <- proc.time()[["elapsed"]] - started
        if (result$status != "success") {
            jsonlite::write_json(result, file.path(directory, "failure.json"), auto_unbox=TRUE, pretty=TRUE, null="null")
            stop("actual package workflow failed: ", id)
        }
        if (capture$entries != 1L || capture$noise_calls != as.integer(mode$enabled)) {
            stop("unexpected actual clustering/denoise invocation count")
        }
        parameters <- cluster_parameters(ns, FALSE, 0.8)
        before <- export_state(capture$input, file.path(relative, "before"), root)
        after <- export_state(result$value, file.path(relative, "after"), root)
        diagnostic <- export_diagnostic(capture, relative, root, parameters)
        groups <- export_groups(result$value, capture, ns)
        cases[[id]] <- list(id=id, status="success", mode=mode,
            inputs=c(list(bundle=source$kind, references=array(refs)), inputs),
            native=list(cutoff=settings$cutoff, min_cells_per_gene=settings$min_cells_per_gene,
                        window_length=settings$window_length, max_centered_threshold=settings$max_centered_threshold,
                        reference_mean=!isTRUE(settings$ref_subtract_use_mean_bounds)),
            before=before, after=after, diagnostic=diagnostic, groups=groups, noise=capture$noise,
            raw_create_to_return_wall_seconds=wall, warnings=result$warnings)
        cat("Completed actual raw workflow: ", id, " in ", wall, "s\n", sep="")
    }
    paths <- sort(list.files(root, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
    files <- stats::setNames(lapply(paths, function(path) hash_file(file.path(root, path))), paths)
    record <- list(schema_version=1L, status="complete", package=pin,
                   scope="raw CreateInfercnvObject plus samples run through15/22; HMM/masking/plots disabled",
                   timing="in-process R raw creation through return, including tracing; not fresh-process timing",
                   source_bundles=list(synthetic=synthetic$manifest_sha256, shipped=shipped$manifest_sha256),
                   cases=cases, files=files)
    jsonlite::write_json(record, file.path(root, "witness.json"), auto_unbox=TRUE, pretty=TRUE,
                         null="null", na="null", digits=17)
}

denoise_workflow_main()
