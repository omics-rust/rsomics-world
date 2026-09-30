#!/usr/bin/env Rscript

hash_file <- function(path) digest::digest(path, algo="sha256", file=TRUE)
array <- function(value) unname(as.list(value))
nullable_array <- function(value) if (is.null(value)) NULL else array(value)
numeric17 <- function(value) array(sprintf("%.17g", as.numeric(value)))
call_text <- function(value) if (is.null(value)) NULL else paste(deparse(value), collapse=" ")

condition_record <- function(value) {
    list(message=conditionMessage(value), classes=array(class(value)),
         call=call_text(conditionCall(value)))
}

bundle_file <- function(root, relative) {
    if (!is.character(relative) || length(relative) != 1L || is.na(relative) ||
        !nzchar(relative) || startsWith(relative, "/") || grepl("\\\\", relative) ||
        any(strsplit(relative, "/", fixed=TRUE)[[1L]] %in% c("", ".", ".."))) {
        stop("unsafe relative file path")
    }
    parts <- strsplit(relative, "/", fixed=TRUE)[[1L]]
    current <- root
    for (part in parts) {
        current <- file.path(current, part)
        link <- Sys.readlink(current)
        if (!is.na(link) && nzchar(link)) stop("symlink in evidence path: ", relative)
    }
    path <- normalizePath(current, mustWork=TRUE)
    if (!startsWith(path, paste0(normalizePath(root), .Platform$file.sep)) ||
        dir.exists(path)) stop("evidence path escapes root or is not a file")
    path
}

load_bundle <- function(path, kind, package_pin) {
    root <- normalizePath(path, mustWork=TRUE)
    manifest_path <- bundle_file(root, "sha256-manifest.tsv")
    lines <- readLines(manifest_path, warn=FALSE)
    if (!length(lines) || lines[[1L]] != "sha256\tpath") stop("invalid bundle manifest")
    entries <- strsplit(lines[-1L], "\t", fixed=TRUE)
    if (!length(entries) || any(lengths(entries) != 2L)) stop("invalid manifest entries")
    names <- vapply(entries, `[[`, character(1), 2L)
    hashes <- vapply(entries, `[[`, character(1), 1L)
    if (anyDuplicated(names) || !identical(names, sort(names, method="radix")) ||
        any(!grepl("^[0-9a-f]{64}$", hashes))) stop("unsorted or duplicate bundle manifest")
    for (i in seq_along(names)) {
        if (!identical(hash_file(bundle_file(root, names[[i]])), hashes[[i]])) {
            stop("bundle manifest SHA-256 mismatch: ", names[[i]])
        }
    }
    actual <- sort(list.files(root, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
    if (!identical(setdiff(actual, "sha256-manifest.tsv"), names)) {
        stop("bundle manifest does not cover actual files")
    }
    record <- jsonlite::fromJSON(bundle_file(root, "oracle.json"), simplifyVector=FALSE)
    if (!identical(record$package, package_pin) ||
        !identical(record$schema_version, if (kind == "synthetic") 1L else 2L)) {
        stop("bundle package or schema pin mismatch")
    }
    list(root=root, record=record, hashes=stats::setNames(hashes, names),
         manifest_sha256=hash_file(manifest_path), oracle_sha256=hash_file(bundle_file(root, "oracle.json")))
}

read_checkpoint <- function(bundle, relative) {
    expected <- bundle$hashes[[relative]]
    path <- bundle_file(bundle$root, relative)
    if (is.null(expected) || !identical(hash_file(path), expected)) stop("unpinned checkpoint")
    obj <- readRDS(path)
    if (!methods::is(obj, "infercnv") || !is.null(obj@.hspike)) {
        stop("checkpoint is not a non-HMM infercnv object")
    }
    obj
}

state_identity <- function(obj) {
    list(expression=obj@expr.data, counts=obj@count.data, genes=obj@gene_order,
         references=obj@reference_grouped_cell_indices,
         observations=obj@observation_grouped_cell_indices, spike=obj@.hspike)
}

same_state <- function(actual, expected) {
    if (!identical(state_identity(actual), state_identity(expected))) {
        stop("expression, count, gene or creation-map state changed")
    }
    invisible(TRUE)
}

write_maps <- function(obj, path) {
    lines <- "role\tgroup\tindex\tcell"
    cells <- colnames(obj@expr.data)
    for (role in c("reference", "observation")) {
        maps <- if (role == "reference") obj@reference_grouped_cell_indices else
            obj@observation_grouped_cell_indices
        for (i in seq_along(maps)) {
            for (index in maps[[i]]) {
                if (!is.finite(index) || index != trunc(index) || index < 1L || index > length(cells)) {
                    stop("invalid creation group index")
                }
                lines <- c(lines, paste(role, names(maps)[[i]], index, cells[[index]], sep="\t"))
            }
        }
    }
    writeLines(lines, path)
}

export_state <- function(obj, relative, root) {
    directory <- file.path(root, relative)
    dir.create(directory, recursive=TRUE, showWarnings=FALSE)
    if (!dir.exists(directory)) stop("cannot create state export directory")
    paths <- list(expression=file.path(relative, "expression.tsv"),
                  genes=file.path(relative, "genes.tsv"), cells=file.path(relative, "cells.tsv"),
                  maps=file.path(relative, "maps.tsv"))
    write_expression(obj, file.path(root, paths$expression))
    write_identity(obj, file.path(root, paths$genes), file.path(root, paths$cells))
    write_maps(obj, file.path(root, paths$maps))
    paths
}

write_matrix <- function(matrix, path) {
    if (!is.matrix(matrix) || !is.numeric(matrix) || any(!is.finite(matrix))) {
        stop("invalid diagnostic expression matrix")
    }
    con <- file(path, "wt")
    on.exit(close(con))
    writeLines(paste(c("gene", colnames(matrix)), collapse="\t"), con)
    for (i in seq_len(nrow(matrix))) {
        writeLines(paste(c(rownames(matrix)[[i]], sprintf("%.17g", matrix[i, ])), collapse="\t"), con)
    }
}

.samples_trace_enter <- function(frame) {
    capture <- get(".samples_trace_capture", envir=.GlobalEnv)
    capture$entries <- capture$entries + 1L
    capture$input <- get("infercnv_obj", envir=frame, inherits=FALSE)
    capture$before <- state_identity(capture$input)
}

.samples_trace_exit <- function(frame) {
    capture <- get(".samples_trace_capture", envir=.GlobalEnv)
    local <- list()
    for (name in c("expr.data", "gene_order", "outliers", "tumor_groups", "res", "subclusters_per_chr")) {
        if (exists(name, envir=frame, inherits=FALSE)) local[name] <- list(get(name, envir=frame, inherits=FALSE))
    }
    capture$local <- local
}

.samples_single_enter <- function(frame) {
    capture <- get(".samples_trace_capture", envir=.GlobalEnv)
    capture$groups[[length(capture$groups) + 1L]] <- list(
        name=get("tumor_name", envir=frame, inherits=FALSE),
        indices=get("tumor_group_idx", envir=frame, inherits=FALSE),
        matrix=get("tumor_expr_data", envir=frame, inherits=FALSE))
}

reset_capture <- function(capture) {
    capture$entries <- 0L
    capture$groups <- list()
    capture$input <- capture$before <- capture$local <- NULL
}

capture_call <- function(fun, log_path, capture, context, seed) {
    reset_capture(capture)
    context$GLOBAL_NUM_THREADS <- 1L
    set.seed(seed)
    warnings <- list()
    con <- file(log_path, "wt")
    sink(con, type="output")
    sink(con, type="message")
    on.exit({ sink(type="message"); sink(type="output"); close(con) })
    result <- tryCatch(withCallingHandlers(list(status="success", value=fun()),
        warning=function(condition) {
            warnings[[length(warnings) + 1L]] <<- condition_record(condition)
            cat("OBSERVED WARNING: ", conditionMessage(condition), "\n", sep="")
            invokeRestart("muffleWarning")
        }), error=function(condition) {
            cat("OBSERVED ERROR: ", conditionMessage(condition), "\n", sep="")
            list(status="error", error=condition_record(condition))
        })
    result$warnings <- warnings
    result
}

export_diagnostic <- function(capture, relative, root, parameters) {
    local <- capture$local
    if (is.null(local) || !("expr.data" %in% names(local))) return(NULL)
    matrix <- local$expr.data
    selected <- match(rownames(matrix), rownames(capture$input@expr.data))
    if (anyNA(selected) || !identical(matrix, capture$input@expr.data[selected, , drop=FALSE])) {
        stop("captured filtering matrix is not an unchanged selection of source genes")
    }
    path <- file.path(relative, "filtered-expression.tsv")
    write_matrix(matrix, file.path(root, path))
    list(selected_gene_indices=array(selected), filtered_expression=path,
         outliers=nullable_array(local$outliers),
         filter_guard_active=parameters$z_score_filter > 0 &&
             length(capture$input@reference_grouped_cell_indices) > 0L,
         distance_ordering="lower-column-major")
}

tree_signature <- function(tree) tree[c("merge", "height", "order", "labels", "method", "dist.method")]

export_groups <- function(obj, capture, ns) {
    declared <- capture$local$tumor_groups
    if (length(declared) != length(capture$groups)) stop("traced source group call count mismatch")
    result <- obj@tumor_subclusters
    lapply(seq_along(capture$groups), function(i) {
        observed <- capture$groups[[i]]
        name <- observed$name
        if (!identical(name, names(declared)[[i]])) stop("source group call order mismatch")
        indices <- observed$indices
        tree_entry <- name %in% names(result$hc)
        tree <- result$hc[[name]]
        distance_values <- list()
        distance_labels <- NULL
        tree_json <- NULL
        if (!is.null(tree)) {
            distance <- get("parallelDist", envir=ns)(t(observed$matrix), threads=1L)
            rebuilt <- get("hclust", envir=ns)(distance, method="ward.D2")
            if (!identical(tree_signature(rebuilt), tree_signature(tree))) {
                stop("diagnostic distance and imported fastcluster tree do not bind to package-produced tree")
            }
            distance_values <- numeric17(distance)
            distance_labels <- nullable_array(attr(distance, "Labels"))
            tree_json <- list(merge=lapply(seq_len(nrow(tree$merge)), function(row) array(tree$merge[row, ])),
                              height=numeric17(tree$height), order=array(tree$order),
                              labels=nullable_array(tree$labels), method=tree$method)
        }
        members <- result$subclusters[[name]]
        subclusters <- lapply(seq_along(members), function(j) {
            list(name=names(members)[[j]], indices=array(unname(members[[j]])),
                 index_names=nullable_array(names(members[[j]])))
        })
        list(name=name, declared_indices=array(unname(declared[[i]])),
             input_indices=array(unname(indices)), input_index_names=nullable_array(names(indices)),
             tree_entry_present=tree_entry, tree_present=!is.null(tree), tree=tree_json,
             subclusters=subclusters, distances=distance_values, distance_labels=distance_labels)
    })
}

export_route <- function(call, capture, relative, root, parameters, ns, raw_list=FALSE) {
    if (call$status == "error") {
        return(list(status="error", error=call$error, warnings=call$warnings,
                    diagnostic=export_diagnostic(capture, relative, root, parameters)))
    }
    if (capture$entries != 1L || !identical(state_identity(capture$input), capture$before)) {
        stop("unexpected clustering invocation count or mutated original input")
    }
    if (raw_list && (!is.list(call$value) || length(call$value) != 2L)) stop("actual clustering return shape changed")
    obj <- if (raw_list) call$value[[1L]] else call$value
    if (!identical(state_identity(obj), capture$before)) stop("clustering changed original expression or creation state")
    checkpoint <- file.path(relative, "returned-object.rds")
    saveRDS(obj, file.path(root, checkpoint))
    result <- list(status="success", warnings=call$warnings,
                   state=export_state(obj, file.path(relative, "state"), root), checkpoint=checkpoint,
                   groups=export_groups(obj, capture, ns),
                   diagnostic=export_diagnostic(capture, relative, root, parameters),
                   hc_entry_exists="hc" %in% names(obj@tumor_subclusters),
                   result_tree_group_names=array(names(obj@tumor_subclusters$hc)),
                   result_subcluster_group_names=array(names(obj@tumor_subclusters$subclusters)))
    if (raw_list) {
        raw_path <- file.path(relative, "returned-list.rds")
        second_path <- file.path(relative, "second-result.rds")
        saveRDS(call$value, file.path(root, raw_path))
        saveRDS(call$value[[2L]], file.path(root, second_path))
        result$result_list <- raw_path
        result$second_result <- second_path
        result$second_result_is_null <- is.null(call$value[[2L]])
    }
    result
}

complete_parameters <- function(fun, overrides, enums=character()) {
    keys <- setdiff(names(formals(fun)), "infercnv_obj")
    values <- stats::setNames(lapply(keys, function(name) eval(formals(fun)[[name]], envir=environment(fun))), keys)
    for (name in enums) values[[name]] <- values[[name]][[1L]]
    for (name in names(overrides)) values[name] <- overrides[name]
    if (anyDuplicated(names(values)) || !identical(names(values), keys)) stop("explicit argument identity mismatch")
    values
}

cluster_parameters <- function(ns, grouped, z) {
    complete_parameters(get("define_signif_tumor_subclusters", envir=ns),
        list(hclust_method="ward.D2", cluster_by_groups=grouped, partition_method="none",
             z_score_filter=z, restrict_to_DE_genes=FALSE, per_chr_hmm_subclusters=FALSE,
             per_chr_hmm_subclusters_references=FALSE),
        c("leiden_method", "leiden_function", "leiden_method_per_chr", "leiden_function_per_chr"))
}

run_parameters <- function(settings, grouped, z, out_dir) {
    overrides <- settings
    overrides[c("out_dir", "analysis_mode", "hclust_method", "cluster_by_groups", "z_score_filter",
                "up_to_step", "num_ref_groups", "num_threads", "resume_mode", "save_rds",
                "HMM", "scale_data", "denoise", "prune_outliers", "mask_nonDE_genes",
                "remove_genes_at_chr_ends", "plot_steps", "inspect_subclusters", "no_plot",
                "no_prelim_plot", "save_final_rds", "write_expr_matrix", "write_phylo", "diagnostics")] <-
        list(out_dir, "samples", "ward.D2", grouped, z, 15L, NULL, 1L, FALSE, TRUE,
             FALSE, FALSE, FALSE, FALSE, FALSE, FALSE, FALSE, FALSE, TRUE, TRUE,
             FALSE, FALSE, FALSE, FALSE)
    complete_parameters(infercnv::run, overrides,
        c("smooth_method", "HMM_report_by", "HMM_type", "analysis_mode", "tumor_subcluster_partition_method",
          "leiden_method", "leiden_function", "leiden_method_per_chr", "leiden_function_per_chr"))
}

function_definition <- function(path, name) {
    definitions <- parse(path, keep.source=FALSE)
    matches <- Filter(function(value) is.call(value) && identical(value[[1L]], as.name("<-")) &&
        identical(value[[2L]], as.name(name)) && is.call(value[[3L]]) &&
        identical(value[[3L]][[1L]], as.name("function")), as.list(definitions))
    if (length(matches) != 1L) stop("missing unique pinned function definition: ", name)
    matches[[1L]][[3L]]
}

validate_installed_source <- function(ns, source_root, metadata_dir) {
    source_files <- c(run="R/inferCNV_ops.R", ".get_relevant_args_list"="R/inferCNV_ops.R",
                      define_signif_tumor_subclusters="R/inferCNV_tumor_subclusters.R",
                      ".single_tumor_subclustering"="R/inferCNV_tumor_subclusters.R")
    lines <- "function\tsource_path\tsource_sha256\tinstalled_text\tinstalled_text_sha256"
    for (name in names(source_files)) {
        source_path <- file.path(source_root, source_files[[name]])
        definition <- function_definition(source_path, name)
        installed <- get(name, envir=ns)
        expected_formals <- definition[[2L]]
        expected_body <- definition[[3L]]
        if (!identical(paste(deparse(formals(installed)), collapse="\n"), paste(deparse(expected_formals), collapse="\n")) ||
            !identical(paste(deparse(body(installed)), collapse="\n"), paste(deparse(expected_body), collapse="\n"))) {
            stop("installed function differs from pinned source: ", name)
        }
        text_path <- file.path(metadata_dir, paste0("function-", name, ".txt"))
        writeLines(deparse(installed), text_path)
        lines <- c(lines, paste(name, source_files[[name]], hash_file(source_path),
                               basename(text_path), hash_file(text_path), sep="\t"))
    }
    writeLines(lines, file.path(metadata_dir, "functions.tsv"))
    if (!identical(get("hclust", envir=ns), getExportedValue("fastcluster", "hclust")) ||
        !identical(get("parallelDist", envir=ns), getExportedValue("parallelDist", "parallelDist"))) {
        stop("namespace distance or tree binding differs from pinned imports")
    }
}

workflow_case <- function(bundle, kind, profile, grouped, z, root, ns, capture, context, seed) {
    entry <- if (kind == "synthetic") bundle$record$profiles[[profile]] else bundle$record$cases[[profile]]
    if (is.null(entry)) stop("missing workflow source profile")
    settings <- if (kind == "synthetic") bundle$record$settings else bundle$record$run_settings
    if (kind == "synthetic") settings$ref_subtract_use_mean_bounds <- entry$settings$ref_subtract_use_mean_bounds
    refs <- if (kind == "synthetic") unlist(entry$settings$ref_group_names, use.names=FALSE) else
        unlist(entry$reference_groups, use.names=FALSE)
    refs <- as.character(refs)
    stage1_rel <- entry$stages[["1"]]$checkpoint
    stage14_rel <- entry$stages[["14"]]$checkpoint
    creation <- read_checkpoint(bundle, stage1_rel)
    stage14 <- read_checkpoint(bundle, stage14_rel)
    if (!identical(names(creation@reference_grouped_cell_indices), if (length(refs)) refs else NULL)) {
        stop("source creation reference request order mismatch")
    }
    source_paths <- if (kind == "synthetic") {
        list(counts=bundle$record$inputs[["counts.tsv"]], positions=bundle$record$inputs[["gene_order.tsv"]],
             annotations=bundle$record$inputs[["annotations.tsv"]])
    } else {
        list(counts=bundle$record$inputs[[entry$input_counts]], positions=bundle$record$inputs$original_gene_order,
             annotations=bundle$record$inputs$original_annotations)
    }
    id <- paste(kind, profile, if (grouped) "grouped" else "pooled", if (z == 0) "filter0" else "filter08", sep="_")
    base <- file.path("cases", id)
    dir.create(file.path(root, base), recursive=TRUE)
    parameters <- cluster_parameters(ns, grouped, z)
    isolated_rel <- file.path(base, "isolated")
    full_rel <- file.path(base, "full_run")
    dir.create(file.path(root, isolated_rel))
    dir.create(file.path(root, full_rel))
    isolated_call <- capture_call(function() do.call(get("define_signif_tumor_subclusters", envir=ns),
        c(list(infercnv_obj=stage14), parameters)), file.path(root, isolated_rel, "call.log"), capture, context, seed)
    isolated <- export_route(isolated_call, capture, isolated_rel, root, parameters, ns, raw_list=TRUE)
    run_args <- run_parameters(settings, grouped, z, file.path(root, full_rel, "upstream"))
    expected <- do.call(get(".get_relevant_args_list", envir=ns), run_args)$expected_file_names
    full_call <- capture_call(function() do.call(infercnv::run, c(list(infercnv_obj=creation), run_args)),
        file.path(root, full_rel, "call.log"), capture, context, seed)
    full <- export_route(full_call, capture, full_rel, root, parameters, ns)
    agreement <- FALSE
    if (isolated$status == "success" && full$status == "success") {
        if (any(!file.exists(expected[c(14L, 15L)]))) stop("missing actual step14/15 checkpoint")
        preliminary <- file.path(run_args$out_dir, "preliminary.infercnv_obj")
        if (!file.exists(preliminary)) stop("missing preliminary checkpoint")
        actual14 <- readRDS(expected[[14L]])
        actual15 <- readRDS(expected[[15L]])
        actual_preliminary <- readRDS(preliminary)
        same_state(actual14, stage14)
        same_state(actual15, stage14)
        same_state(actual_preliminary, stage14)
        if (!identical(isolated_call$value[[1L]]@tumor_subclusters, full_call$value@tumor_subclusters) ||
            !identical(actual15@tumor_subclusters, full_call$value@tumor_subclusters) ||
            !identical(actual_preliminary@tumor_subclusters, full_call$value@tumor_subclusters) ||
            !identical(isolated$groups, full$groups) || !identical(isolated$diagnostic$selected_gene_indices,
                full$diagnostic$selected_gene_indices)) stop("isolated and full clustering routes differ")
        full$stage14_checkpoint <- file.path(full_rel, "upstream", basename(expected[[14L]]))
        full$stage15_checkpoint <- file.path(full_rel, "upstream", basename(expected[[15L]]))
        full$preliminary_checkpoint <- file.path(full_rel, "upstream", "preliminary.infercnv_obj")
        full$stage14_state <- export_state(actual14, file.path(full_rel, "stage14-state"), root)
        full$stage15_state <- export_state(actual15, file.path(full_rel, "stage15-state"), root)
        full$preliminary_state <- export_state(actual_preliminary, file.path(full_rel, "preliminary-state"), root)
        agreement <- TRUE
    }
    recorded_args <- run_args
    recorded_args$out_dir <- file.path(full_rel, "upstream")
    list(id=id, kind="workflow", inputs=list(bundle=kind, profile=profile, reference_groups=array(refs),
        stage1_checkpoint=stage1_rel, stage14_checkpoint=stage14_rel,
        sha256s=list(manifest=bundle$manifest_sha256, oracle=bundle$oracle_sha256,
                    stage1=bundle$hashes[[stage1_rel]], stage14=bundle$hashes[[stage14_rel]]),
        source_files=lapply(source_paths, function(path) list(path=path, sha256=bundle$hashes[[path]]))),
        arguments=list(isolated=parameters, full_run=recorded_args), isolated=isolated, full_run=full,
        route_agreement=agreement)
}

probe_object <- function(expression, groups, refs=character(), maps=NULL) {
    n <- ncol(expression)
    g <- nrow(expression)
    colnames(expression) <- paste0("cell", seq_len(n))
    rownames(expression) <- paste0("gene", seq_len(g))
    counts <- matrix(20, g, n, dimnames=dimnames(expression))
    order <- data.frame(chr=rep("chrProbe", g), start=seq_len(g) * 100L,
                        stop=seq_len(g) * 100L + 49L, row.names=rownames(expression))
    annotations <- data.frame(group=groups, row.names=colnames(expression))
    obj <- infercnv::CreateInfercnvObject(raw_counts_matrix=counts, gene_order_file=order,
        annotations_file=annotations, ref_group_names=if (length(refs)) refs else NULL,
        delim="\t", max_cells_per_group=NULL, min_max_counts_per_cell=c(1, Inf),
        chr_exclude=c("chrX", "chrY", "chrM"))
    obj@expr.data <- expression
    if (!is.null(maps)) {
        obj@reference_grouped_cell_indices <- maps$references
        obj@observation_grouped_cell_indices <- maps$observations
    }
    methods::validObject(obj)
    obj
}

probe_case <- function(id, obj, grouped, z, root, ns, capture, context, seed) {
    base <- file.path("cases", id)
    dir.create(file.path(root, base), recursive=TRUE)
    checkpoint <- file.path(base, "pre-call.rds")
    saveRDS(obj, file.path(root, checkpoint))
    before <- export_state(obj, file.path(base, "pre-call"), root)
    relative <- file.path(base, "isolated")
    dir.create(file.path(root, relative))
    parameters <- cluster_parameters(ns, grouped, z)
    call <- capture_call(function() do.call(get("define_signif_tumor_subclusters", envir=ns),
        c(list(infercnv_obj=obj), parameters)), file.path(root, relative, "call.log"), capture, context, seed)
    route <- export_route(call, capture, relative, root, parameters, ns, raw_list=TRUE)
    route$pre_call_checkpoint <- checkpoint
    list(id=id, kind="probe", inputs=list(reference_groups=array(names(obj@reference_grouped_cell_indices)),
        pre_call_checkpoint=checkpoint, pre_call_state=before), arguments=list(isolated=parameters), isolated=route)
}

main <- function() {
    args <- commandArgs(trailingOnly=TRUE)
    if (length(args) != 3L) stop("usage: infercnv_samples_clustering_witness.R SYNTHETIC_BUNDLE SHIPPED_BUNDLE NEW_OUTPUT_DIR")
    script_arg <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
    if (length(script_arg) != 1L) stop("cannot identify exporter script")
    script <- normalizePath(sub("^--file=", "", script_arg))
    script_dir <- dirname(script)
    source(file.path(script_dir, "infercnv_oracle_io.R"), local=.GlobalEnv)
    pin <- list(name="infercnv", version="1.28.0", source_commit="b421d9405c97a309b081ef86d455e976df93eae4",
                source_archive_sha256="b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207")
    if (!identical(as.character(getRversion()), "4.6.1") ||
        !identical(Sys.getenv("INFER_CNV_SOURCE_COMMIT"), pin$source_commit) ||
        !identical(Sys.getenv("INFER_CNV_SOURCE_SHA256"), pin$source_archive_sha256)) stop("runtime or source environment pin mismatch")
    for (package in c("infercnv", "jsonlite", "digest", "fastcluster", "parallelDist")) {
        if (!requireNamespace(package, quietly=TRUE)) stop("required installed package absent: ", package)
    }
    if (!identical(as.character(utils::packageVersion("infercnv")), pin$version)) stop("infercnv package version mismatch")
    archive <- Sys.getenv("INFER_CNV_SOURCE_ARCHIVE")
    if (!file.exists(archive) || !identical(hash_file(archive), pin$source_archive_sha256)) stop("source archive bytes mismatch")
    synthetic <- load_bundle(args[[1L]], "synthetic", pin)
    shipped <- load_bundle(args[[2L]], "shipped", pin)
    root <- normalizePath(args[[3L]], mustWork=FALSE)
    if (file.exists(root)) stop("output directory already exists: ", root)
    dir.create(root, recursive=TRUE)
    if (!dir.exists(root)) stop("cannot create output directory")
    metadata_dir <- file.path(root, "metadata")
    dir.create(metadata_dir)
    source_dir <- file.path(metadata_dir, "source")
    dir.create(source_dir)
    if (!file.copy(archive, file.path(source_dir, "infercnv-source.tar.gz"))) stop("cannot preserve source archive")
    prefix <- paste0("infercnv-", pin$source_commit)
    members <- paste0(prefix, "/", c("R/inferCNV_ops.R", "R/inferCNV_tumor_subclusters.R", "NAMESPACE", "DESCRIPTION"))
    listing <- utils::untar(archive, list=TRUE)
    if (anyDuplicated(listing) || any(!members %in% listing)) stop("source archive members differ")
    utils::untar(archive, files=members, exdir=source_dir)
    ns <- asNamespace("infercnv")
    validate_installed_source(ns, file.path(source_dir, prefix), metadata_dir)
    for (name in c("infercnv_samples_clustering_witness.R", "infercnv_oracle_io.R", "infercnv_oracle.R", "infercnv_shipped_oracle.R")) {
        if (!file.copy(file.path(script_dir, name), file.path(metadata_dir, name))) stop("cannot preserve exporter/source script: ", name)
    }
    if (!file.copy(file.path(synthetic$root, "sha256-manifest.tsv"), file.path(metadata_dir, "synthetic-manifest.tsv")) ||
        !file.copy(file.path(shipped$root, "sha256-manifest.tsv"), file.path(metadata_dir, "shipped-manifest.tsv"))) {
        stop("cannot preserve source bundle manifests")
    }
    Sys.setenv(OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", MKL_NUM_THREADS="1")
    if (!nzchar(Sys.setlocale("LC_ALL", "C"))) stop("cannot initialize C locale")
    for (category in c("LC_MESSAGES", "LC_PAPER", "LC_MEASUREMENT")) {
        if (!identical(Sys.setlocale(category, "C"), "C")) stop("cannot enforce C locale category: ", category)
    }
    if (!identical(Sys.getlocale(), "C")) stop("cannot enforce C locale")
    seed <- 260930L
    context <- get("infercnv.env", envir=ns)
    context$GLOBAL_NUM_THREADS <- 1L
    capture <- new.env(parent=emptyenv())
    assign(".samples_trace_capture", capture, envir=.GlobalEnv)
    trace("define_signif_tumor_subclusters", where=ns, print=FALSE,
          tracer=quote(get(".samples_trace_enter", envir=.GlobalEnv)(environment())),
          exit=quote(get(".samples_trace_exit", envir=.GlobalEnv)(environment())))
    trace(".single_tumor_subclustering", where=ns, print=FALSE,
          tracer=quote(get(".samples_single_enter", envir=.GlobalEnv)(environment())))
    on.exit({ untrace("define_signif_tumor_subclusters", where=ns);
              untrace(".single_tumor_subclustering", where=ns) })
    cases <- list()
    for (kind in c("synthetic", "shipped")) {
        bundle <- if (kind == "synthetic") synthetic else shipped
        profiles <- if (kind == "synthetic") c("grouped_bounds", "no_reference") else c("subset", "full")
        for (profile in profiles) for (grouped in c(TRUE, FALSE)) for (z in c(0, 0.8)) {
            entry <- workflow_case(bundle, kind, profile, grouped, z, root, ns, capture, context, seed)
            cases[[entry$id]] <- entry
        }
    }
    add_probe <- function(id, obj, grouped=TRUE, z=0) {
        entry <- probe_case(id, obj, grouped, z, root, ns, capture, context, seed)
        cases[[id]] <<- entry
    }
    mixed <- matrix(seq_len(24L) / 16 + 1, 4L, 6L)
    maps <- list(references=list(), observations=list(one=2L, two=c(6L, 1L), three=c(5L, 3L, 4L)))
    groups <- c("two", "one", "three", "three", "three", "two")
    obj <- probe_object(mixed, groups, maps=maps)
    add_probe("probe_small_groups_grouped_filter0", obj)
    add_probe("probe_small_groups_pooled_filter0", obj, FALSE)
    add_probe("probe_equal_profiles_grouped_filter0", probe_object(matrix(1, 4L, 3L), rep("obs", 3L)))
    ties <- diag(3L) + 1
    add_probe("probe_equal_distances_grouped_filter0", probe_object(ties, rep("obs", 3L)))
    add_probe("probe_equal_distances_permuted_grouped_filter0", probe_object(ties, rep("obs", 3L),
        maps=list(references=list(), observations=list(obs=c(3L, 1L, 2L)))))
    ref_groups <- rep(c("a_obs", "z_ref", "b_obs", "a_ref"), 3L)
    ref_obj <- probe_object(matrix(seq_len(48L) / 32 + 1, 4L, 12L), ref_groups, c("z_ref", "a_ref"))
    add_probe("probe_multiple_reference_order_grouped_filter0", ref_obj)
    add_probe("probe_multiple_reference_order_pooled_filter0", ref_obj, FALSE)
    offset <- c(-sqrt(1.829998), -0.801, -0.8, -0.799, 0.799, 0.8, 0.801, sqrt(1.829998))
    boundary <- cbind(2 + offset, 2 + offset, 2 + offset / 2, 2 - offset / 3, 2 + offset / 4)
    constant <- cbind(matrix(1, 4L, 3L), matrix(seq_len(12L) / 16 + 1, 4L, 3L))
    no_qualifying <- cbind(matrix(rep(c(rep(1, 7L), 9), each=4L), 4L, 8L), matrix(2, 4L, 3L))
    all_qualifying <- cbind(matrix(rep(c(1, 3, 1, 3), 2L), 4L, 2L), matrix(2, 4L, 3L))
    filter_objects <- list(
        boundary=probe_object(boundary, c(rep("ref", 2L), rep("obs", 3L)), "ref"),
        constant=probe_object(constant, c(rep("ref", 3L), rep("obs", 3L)), "ref"),
        no_qualifying=probe_object(no_qualifying, c(rep("ref", 8L), rep("obs", 3L)), "ref"),
        all_qualifying=probe_object(all_qualifying, c(rep("ref", 2L), rep("obs", 3L)), "ref"))
    for (name in names(filter_objects)) for (z in c(0, 0.2, 0.8)) {
        label <- if (z == 0) "filter0" else if (z == 0.2) "filter02" else "filter08"
        add_probe(paste0("probe_filter_", name, "_", label), filter_objects[[name]], z=z)
    }
    collision <- probe_object(matrix(seq_len(36L) / 16 + 1, 4L, 9L),
        rep(c("a_obs", "all_observations", "b_obs"), 3L), "all_observations")
    add_probe("probe_pooled_reference_name_collision_filter0", collision, FALSE)
    if (length(cases) != 36L) stop("required workflow/probe inventory is incomplete")
    writeLines(capture.output(sessionInfo()), file.path(metadata_dir, "session.txt"))
    packages <- utils::installed.packages()[, c("Package", "Version", "LibPath"), drop=FALSE]
    write.table(packages[order(packages[, "Package"], method="radix"), , drop=FALSE],
                file.path(metadata_dir, "packages.tsv"), sep="\t", row.names=FALSE, quote=FALSE)
    writeLines(c(paste0("source_commit\t", pin$source_commit),
                 paste0("source_archive_sha256\t", pin$source_archive_sha256),
                 "index_base\t1", "distance_ordering\tlower-column-major"), file.path(metadata_dir, "source.txt"))
    paths <- sort(list.files(root, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
    files <- stats::setNames(as.list(vapply(paths, function(path) hash_file(bundle_file(root, path)), character(1))), paths)
    record <- list(schema_version=1L, package=pin,
        runtime=list(R_version=as.character(getRversion()), locale=Sys.getlocale(), seed=seed, requested_threads=1L,
            package_versions=stats::setNames(lapply(c("infercnv", "fastcluster", "parallelDist"),
                function(package) as.character(getNamespaceVersion(package))), c("infercnv", "fastcluster", "parallelDist")),
            package_library_paths=stats::setNames(lapply(c("infercnv", "fastcluster", "parallelDist"),
                function(package) dirname(getNamespaceInfo(asNamespace(package), "path"))),
                c("infercnv", "fastcluster", "parallelDist")),
            thread_environment=as.list(Sys.getenv(c("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS")))),
        index_base=1L, metadata=list(session="metadata/session.txt", packages="metadata/packages.tsv",
            source="metadata/source.txt", functions="metadata/functions.tsv",
            exporter="metadata/infercnv_samples_clustering_witness.R", oracle_io="metadata/infercnv_oracle_io.R",
            synthetic_script="metadata/infercnv_oracle.R", shipped_script="metadata/infercnv_shipped_oracle.R",
            synthetic_manifest="metadata/synthetic-manifest.tsv", shipped_manifest="metadata/shipped-manifest.tsv",
            source_archive="metadata/source/infercnv-source.tar.gz"), cases=cases, files=files)
    jsonlite::write_json(record, file.path(root, "witness.json"), auto_unbox=TRUE, pretty=TRUE, null="null", na="string", digits=17)
    failures <- names(Filter(function(entry) entry$kind == "workflow" && !isTRUE(entry$route_agreement), cases))
    if (length(failures)) stop("workflow route failures preserved: ", paste(failures, collapse=", "))
    cat("Exported 16 workflow comparisons and 20 function-only probes; installed-package acceptance requires independent artifact validation.\n")
}

main()
