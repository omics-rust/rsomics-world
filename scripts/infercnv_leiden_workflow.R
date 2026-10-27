#!/usr/bin/env Rscript

script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
if (length(script) != 1L) stop("cannot identify script")
script_dir <- dirname(normalizePath(sub("^--file=", "", script)))
source(file.path(script_dir, "infercnv_samples_clustering_witness.R"))
source(file.path(script_dir, "infercnv_oracle_io.R"))

leiden_partition_record <- function(input_indices, vertices, membership, members, input_cells=names(input_indices)) {
    present <- !is.null(vertices) || !is.null(membership)
    global <- assigned <- NULL
    if (present) {
        positions <- match(vertices, input_cells)
        if (length(vertices) != length(input_indices) || length(membership) != length(vertices) ||
            anyNA(positions) || anyDuplicated(positions)) stop("graph vertices do not identify group input cells")
        global <- unname(input_indices[positions])
        assigned <- lapply(members, function(indices) unique(membership[match(unname(indices), global)]))
    }
    matches <- if (!present) NULL else all(lengths(assigned) == 1L) &&
        !anyNA(unlist(assigned)) && length(unique(unlist(assigned))) == length(assigned) &&
        setequal(unlist(members, use.names=FALSE), global)
    list(vertex_cells=nullable_array(vertices), global_indices=nullable_array(global),
         membership=nullable_array(membership), membership_names=nullable_array(names(membership)),
         matches_returned_subclusters=matches,
         subclusters=unname(lapply(seq_along(members), function(i) {
             list(name=names(members)[[i]], indices=array(unname(members[[i]])),
                  index_names=nullable_array(names(members[[i]])),
                  partition_labels=if (is.null(assigned)) NULL else array(unname(assigned[[i]])))
         })))
}

leiden_new_capture <- function() {
    capture <- new.env(parent=emptyenv())
    capture$depth <- capture$entries <- 0L
    capture$groups <- list()
    capture$active_group <- capture$input <- capture$before <- capture$local <- capture$result <- NULL
    capture
}

leiden_rng <- function() {
    list(kind=array(RNGkind()), seed=if (exists(".Random.seed", .GlobalEnv, inherits=FALSE))
        array(get(".Random.seed", .GlobalEnv)) else NULL)
}

leiden_frame_values <- function(frame, names) {
    present <- names[vapply(names, exists, logical(1), envir=frame, inherits=FALSE)]
    mget(present, envir=frame, inherits=FALSE)
}

.leiden_trace_enter <- function(frame, kind) {
    capture <- get(".leiden_workflow_capture", .GlobalEnv)
    if (kind == "outer") {
        capture$depth <- capture$depth + 1L
        if (capture$depth == 1L) {
            capture$entries <- capture$entries + 1L
            capture$input <- get("infercnv_obj", frame)
            capture$before <- state_identity(capture$input)
            capture$parameters <- leiden_frame_values(frame, c("p_val", "k_nn", "leiden_method",
                "leiden_function", "leiden_resolution", "hclust_method", "cluster_by_groups",
                "partition_method", "per_chr_hmm_subclusters", "per_chr_hmm_subclusters_references",
                "z_score_filter", "restrict_to_DE_genes"))
        }
        return(invisible(NULL))
    }
    if (capture$depth != 1L) return(invisible(NULL))
    if (kind == "single") {
        capture$active_group <- length(capture$groups) + 1L
        capture$groups[[capture$active_group]] <- list(name=get("tumor_group", frame),
            indices=get("tumor_group_idx", frame), matrix=get("tumor_expr_data", frame),
            arguments=leiden_frame_values(frame, c("k_nn", "leiden_resolution", "leiden_method",
                "leiden_function", "hclust_method")),
            started=proc.time()[["elapsed"]], rng_before=leiden_rng())
    } else {
        group <- capture$groups[[capture$active_group]]
        group[[kind]] <- list(arguments=leiden_frame_values(frame,
            c("k_nn", "resolution_parameter", "objective_function")),
            started=proc.time()[["elapsed"]], rng_before=leiden_rng())
        capture$groups[[capture$active_group]] <- group
    }
    invisible(NULL)
}

.leiden_trace_exit <- function(frame, kind, value) {
    capture <- get(".leiden_workflow_capture", .GlobalEnv)
    if (kind == "outer") {
        if (capture$depth == 1L) {
            capture$local <- leiden_frame_values(frame,
                c("expr.data", "gene_order", "outliers", "tumor_groups", "res", "subclusters_per_chr"))
            capture$result <- value
        }
        capture$depth <- capture$depth - 1L
        return(invisible(NULL))
    }
    if (capture$depth != 1L) return(invisible(NULL))
    group <- capture$groups[[capture$active_group]]
    if (kind == "single") {
        group$local <- leiden_frame_values(frame,
            c("used_leiden_resolution", "partition", "tmp_full_phylo", "tmp_phylo", "added_height", "res"))
        group$value <- value
        group$wall_seconds <- proc.time()[["elapsed"]] - group$started
        group$rng_after <- leiden_rng()
        capture$groups[[capture$active_group]] <- group
        capture$active_group <- NULL
    } else {
        route <- group[[kind]]
        fields <- if (kind == "pca") c("seurat_obs", "all.genes", "graph_obj", "partition_obj", "partition") else
            c("snn", "sparse_adjacency_matrix", "graph_obj", "partition_obj", "partition")
        route$local <- leiden_frame_values(frame, fields)
        route$value <- value
        route$wall_seconds <- proc.time()[["elapsed"]] - route$started
        route$rng_after <- leiden_rng()
        group[[kind]] <- route
        capture$groups[[capture$active_group]] <- group
    }
    invisible(NULL)
}

leiden_trace_targets <- c(define_signif_tumor_subclusters="outer",
    ".single_tumor_leiden_subclustering"="single", ".leiden_seurat_preprocess_routine"="pca",
    ".leiden_simple_snn"="simple")

leiden_install_traces <- function(ns, capture) {
    assign(".leiden_workflow_capture", capture, .GlobalEnv)
    for (name in names(leiden_trace_targets)) {
        kind <- leiden_trace_targets[[name]]
        trace(name, where=ns, print=FALSE,
            tracer=substitute(get(".leiden_trace_enter", .GlobalEnv)(environment(), KIND), list(KIND=kind)),
            exit=substitute(get(".leiden_trace_exit", .GlobalEnv)(environment(), KIND, returnValue()), list(KIND=kind)))
    }
    invisible(NULL)
}

leiden_remove_traces <- function(ns) {
    for (name in names(leiden_trace_targets)) untrace(name, where=ns)
    invisible(NULL)
}

leiden_save <- function(value, relative, root) {
    saveRDS(value, file.path(root, relative))
    relative
}

leiden_export_matrix <- function(value, relative, root) {
    rds <- leiden_save(value, paste0(relative, ".rds"), root)
    matrix <- as.matrix(value)
    tsv <- paste0(relative, ".tsv")
    con <- file(file.path(root, tsv), "wt")
    on.exit(close(con))
    columns <- colnames(matrix)
    rows <- rownames(matrix)
    writeLines(paste(c("row", if (is.null(columns)) seq_len(ncol(matrix)) else columns), collapse="\t"), con)
    for (i in seq_len(nrow(matrix))) {
        writeLines(paste(c(if (is.null(rows)) i else rows[[i]], sprintf("%.17g", matrix[i, ])), collapse="\t"), con)
    }
    list(rds=rds, tsv=tsv, class=array(class(value)), dimensions=array(dim(value)),
         row_names=nullable_array(rows), column_names=nullable_array(columns),
         tsv_unnamed_axes_use_positions=TRUE)
}

leiden_export_seurat <- function(object, relative, root) {
    checkpoint <- leiden_save(object, file.path(relative, "seurat.rds"), root)
    assays <- methods::slot(object, "assays")
    assay_records <- unname(lapply(seq_along(assays), function(i) {
        assay <- assays[[i]]
        layers <- SeuratObject::Layers(assay, search=NA)
        features <- SeuratObject::VariableFeatures(assay)
        feature_path <- file.path(relative, sprintf("assay-%03d-variable-features.txt", i))
        writeLines(features, file.path(root, feature_path))
        list(name=names(assays)[[i]], class=array(class(assay)), variable_features=feature_path,
             layers=unname(lapply(seq_along(layers), function(j) {
                 list(name=layers[[j]], matrix=leiden_export_matrix(
                     SeuratObject::LayerData(assay, layer=layers[[j]]),
                     file.path(relative, sprintf("assay-%03d-layer-%03d", i, j)), root))
             })))
    }))
    reductions <- methods::slot(object, "reductions")
    reduction_records <- unname(lapply(seq_along(reductions), function(i) {
        reduction <- reductions[[i]]
        deviations <- SeuratObject::Stdev(reduction)
        list(name=names(reductions)[[i]],
             scores=leiden_export_matrix(SeuratObject::Embeddings(reduction),
                 file.path(relative, sprintf("reduction-%03d-scores", i)), root),
             loadings=leiden_export_matrix(SeuratObject::Loadings(reduction),
                 file.path(relative, sprintf("reduction-%03d-loadings", i)), root),
             standard_deviations=numeric17(deviations), standard_deviation_names=nullable_array(names(deviations)))
    }))
    graphs <- methods::slot(object, "graphs")
    graph_records <- unname(lapply(seq_along(graphs), function(i) {
        list(name=names(graphs)[[i]], matrix=leiden_export_matrix(graphs[[i]],
            file.path(relative, sprintf("seurat-graph-%03d", i)), root))
    }))
    commands <- methods::slot(object, "commands")
    command_checkpoint <- leiden_save(commands, file.path(relative, "seurat-commands.rds"), root)
    command_records <- unname(lapply(seq_along(commands), function(i) {
        params <- methods::slot(commands[[i]], "params")
        path <- file.path(relative, sprintf("seurat-command-%03d-parameters.R", i))
        writeLines(capture.output(dput(params)), file.path(root, path))
        list(name=names(commands)[[i]], parameters=path)
    }))
    list(checkpoint=checkpoint, class=array(class(object)), default_assay=methods::slot(object, "active.assay"),
         assays=assay_records, reductions=reduction_records, graphs=graph_records,
         commands=list(checkpoint=command_checkpoint, recorded_parameters=command_records))
}

leiden_export_graph <- function(graph, input_indices, input_cells, relative, root) {
    checkpoint <- leiden_save(graph, file.path(relative, "igraph.rds"), root)
    vertices <- igraph::vertex_attr(graph, "name")
    positions <- match(vertices, input_cells)
    if (length(vertices) != length(input_indices) || anyNA(positions) || anyDuplicated(positions)) {
        stop("actual graph vertex identity differs from source group cells")
    }
    vertex_path <- file.path(relative, "graph-vertices.tsv")
    vertex_attributes <- leiden_save(igraph::vertex_attr(graph), file.path(relative, "vertex-attributes.rds"), root)
    edge_attributes <- leiden_save(igraph::edge_attr(graph), file.path(relative, "edge-attributes.rds"), root)
    write.table(data.frame(vertex=seq_along(vertices), cell=vertices, global_index=unname(input_indices[positions])),
        file.path(root, vertex_path), sep="\t", quote=FALSE, row.names=FALSE)
    endpoints <- igraph::as_edgelist(graph, names=FALSE)
    edges <- data.frame(edge=seq_len(nrow(endpoints)), from=endpoints[, 1L], to=endpoints[, 2L],
        from_cell=vertices[endpoints[, 1L]], to_cell=vertices[endpoints[, 2L]],
        loop=endpoints[, 1L] == endpoints[, 2L])
    weighted <- "weight" %in% igraph::edge_attr_names(graph)
    if (weighted) edges$weight <- sprintf("%.17g", igraph::edge_attr(graph, "weight"))
    edge_path <- file.path(relative, "graph-edges.tsv")
    write.table(edges, file.path(root, edge_path), sep="\t", quote=FALSE, row.names=FALSE)
    list(checkpoint=checkpoint, vertices=vertex_path, edges=edge_path,
         vertex_attributes=vertex_attributes, edge_attributes=edge_attributes,
         vertex_count=length(vertices), edge_count=nrow(edges), loop_count=sum(edges$loop),
         directed=igraph::is_directed(graph), weights_present=weighted,
         edge_attribute_names=array(igraph::edge_attr_names(graph)), index_base=1L)
}

leiden_export_group <- function(group, relative, root) {
    dir.create(file.path(root, relative), recursive=TRUE)
    raw <- leiden_save(group, file.path(relative, "actual-group-frames.rds"), root)
    members <- if (!is.null(group$value)) group$value$subclusters else group$local$res$subclusters
    complete <- !is.null(group$value)
    pca <- group$pca
    simple <- group$simple
    actual_seurat <- !is.null(pca) && inherits(pca$local$seurat_obs, "Seurat")
    route <- if (!is.null(simple)) {
        if (!is.null(pca)) "PCA_warning_fallback_simple" else "simple"
    } else if (!is.null(pca)) "PCA" else if (complete && length(group$indices) < 3L) {
        "fewer_than_three_cells"
    } else if (complete && group$arguments$k_nn >= length(group$indices)) "k_nn_at_least_group_size" else "not_reached"
    active <- if (!is.null(simple)) simple else pca
    graph <- active$local$graph_obj
    membership <- active$local$partition
    vertices <- if (is.null(graph)) NULL else igraph::vertex_attr(graph, "name")
    partition <- leiden_partition_record(group$indices,
        if (is.null(membership)) NULL else vertices, membership, members, colnames(group$matrix))
    tree <- if (complete) group$value$hc else group$local$res$hc
    tree_record <- NULL
    if (!is.null(tree)) {
        tree_record <- list(checkpoint=leiden_save(tree, file.path(relative, "stitched-hclust.rds"), root),
            merge=lapply(seq_len(nrow(tree$merge)), function(i) array(tree$merge[i, ])),
            height=numeric17(tree$height), order=array(tree$order), labels=nullable_array(tree$labels),
            method=tree$method, distance_method=tree$dist.method)
    }
    wrapper_record <- function(wrapper, kind) {
        if (is.null(wrapper)) return(NULL)
        list(arguments=wrapper$arguments, returned_membership=nullable_array(wrapper$value),
             partition_object=if (is.null(wrapper$local$partition_obj)) NULL else
                 leiden_save(wrapper$local$partition_obj,
                     file.path(relative, paste0(kind, "-partition.rds")), root),
             wall_seconds=wrapper$wall_seconds, rng_before=wrapper$rng_before, rng_after=wrapper$rng_after)
    }
    list(name=group$name, status=if (complete) "success" else "error", route=route,
         input_indices=array(unname(group$indices)), input_index_names=nullable_array(names(group$indices)),
         input_cells=array(colnames(group$matrix)), input_expression=leiden_export_matrix(group$matrix,
             file.path(relative, "input-expression"), root), raw_frames=raw,
         arguments=group$arguments, used_resolution=group$local$used_leiden_resolution,
         positive_pca_evidence=complete && actual_seurat && !is.null(pca$local$graph_obj) && !is.null(pca$value),
         pca=wrapper_record(pca, "pca"), simple=wrapper_record(simple, "simple"),
         seurat=if (!actual_seurat) NULL else leiden_export_seurat(pca$local$seurat_obs, relative, root),
         seurat_wrapper_object_present=!is.null(pca) && "seurat_obs" %in% names(pca$local),
         seurat_wrapper_class=if (is.null(pca) || !"seurat_obs" %in% names(pca$local)) NULL else array(class(pca$local$seurat_obs)),
         fallback_warning_log=if (route == "PCA_warning_fallback_simple")
             file.path(dirname(dirname(relative)), "call.log") else NULL,
         simple_knn=if (is.null(simple$local$snn)) NULL else leiden_export_matrix(simple$local$snn,
             file.path(relative, "actual-simple-knn"), root),
         simple_adjacency=if (is.null(simple$local$sparse_adjacency_matrix)) NULL else
             leiden_export_matrix(simple$local$sparse_adjacency_matrix,
                 file.path(relative, "actual-simple-adjacency"), root),
         graph=if (is.null(graph)) NULL else leiden_export_graph(graph, group$indices,
             colnames(group$matrix), relative, root), partition=partition, tree=tree_record,
         stitched_phylo=if (is.null(group$local$tmp_full_phylo)) NULL else
             leiden_save(group$local$tmp_full_phylo, file.path(relative, "stitched-phylo.rds"), root),
         wall_seconds=group$wall_seconds, rng_before=group$rng_before, rng_after=group$rng_after)
}

leiden_export_case <- function(result, capture, relative, root, parameters, wall_seconds) {
    raw <- leiden_save(as.list(capture, all.names=TRUE), file.path(relative, "actual-workflow-frames.rds"), root)
    input <- capture$input
    if (!is.null(input) && !identical(state_identity(input), capture$before)) {
        stop("original step14 input differs from its entry snapshot")
    }
    groups <- unname(lapply(seq_along(capture$groups), function(i) {
        leiden_export_group(capture$groups[[i]], file.path(relative, "groups", sprintf("%03d", i)), root)
    }))
    if (result$status == "success") {
        if (capture$entries != 1L || capture$depth != 0L || is.null(capture$result) ||
            !is.null(capture$result[[2L]])) stop("invalid outer Leiden capture contract")
        same_state(result$value, input)
        same_state(capture$result[[1L]], input)
        if (!identical(result$value@tumor_subclusters, capture$result[[1L]]@tumor_subclusters) ||
            !identical(vapply(groups, `[[`, character(1), "name"), names(capture$local$tumor_groups))) {
            stop("actual group order or outer/returned subclusters differ")
        }
    }
    step14 <- if (is.null(input)) NULL else leiden_save(input, file.path(relative, "step14-input.rds"), root)
    list(status=result$status, error=result$error, warnings=result$warnings,
         run_parameters=parameters, raw_frames=raw, call_log=file.path(relative, "call.log"),
         creation_checkpoint=if (is.null(capture$created)) NULL else
             leiden_save(capture$created, file.path(relative, "created-object.rds"), root),
         step14_checkpoint=step14,
         step14_state=if (is.null(input)) NULL else export_state(input, file.path(relative, "step14-state"), root),
         returned_checkpoint=if (result$status != "success") NULL else
             leiden_save(result$value, file.path(relative, "returned-object.rds"), root),
         returned_state=if (result$status != "success") NULL else
             export_state(result$value, file.path(relative, "returned-state"), root),
         discovery_parameters=capture$parameters,
         diagnostic=export_diagnostic(capture, relative, root, parameters),
         filtered_input=if (is.null(capture$local)) NULL else leiden_save(capture$local[
             intersect(c("expr.data", "gene_order", "outliers"), names(capture$local))],
             file.path(relative, "actual-filtered-input.rds"), root),
         original_expression_unchanged=if (result$status != "success") NULL else TRUE,
         groups=groups, positive_pca_groups=sum(vapply(groups, `[[`, logical(1), "positive_pca_evidence")),
         partition_mismatch_groups=array(vapply(Filter(function(group)
             identical(group$partition$matches_returned_subclusters, FALSE), groups), `[[`, character(1), "name")),
         rng_before=capture$run_rng_before, rng_after=capture$run_rng_after, raw_create_to_return_wall_seconds=wall_seconds)
}

leiden_write_witness <- function(record, root) {
    paths <- sort(list.files(root, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
    record$files <- stats::setNames(lapply(paths, function(path) hash_file(file.path(root, path))), paths)
    jsonlite::write_json(record, file.path(root, "witness.json"), auto_unbox=TRUE,
        pretty=TRUE, null="null", na="null", digits=17)
    paths <- sort(c(paths, "witness.json"), method="radix")
    writeLines(c("sha256\tpath", vapply(paths, function(path)
        paste(hash_file(file.path(root, path)), path, sep="\t"), character(1))),
        file.path(root, "sha256-manifest.tsv"))
    invisible(NULL)
}

leiden_validate_source <- function(ns, source_root, metadata) {
    validate_installed_source(ns, source_root, metadata)
    path <- file.path(source_root, "R/inferCNV_tumor_subclusters.R")
    definitions <- as.list(parse(path, keep.source=FALSE))
    for (name in c(names(leiden_trace_targets)[-1L], "add_single_branch_to_phylo")) {
        matches <- Filter(function(value) is.call(value) && as.character(value[[1L]]) %in% c("<-", "=") &&
            identical(value[[2L]], as.name(name)), definitions)
        if (length(matches) != 1L) stop("missing unique pinned definition: ", name)
        definition <- matches[[1L]][[3L]]
        installed <- get(name, ns)
        if (!identical(paste(deparse(formals(installed)), collapse="\n"), paste(deparse(definition[[2L]]), collapse="\n")) ||
            !identical(paste(deparse(body(installed)), collapse="\n"), paste(deparse(definition[[3L]]), collapse="\n"))) {
            stop("installed Leiden function differs from pinned source: ", name)
        }
        target <- file.path(metadata, paste0("function-", name, ".txt"))
        writeLines(deparse(installed), target)
        cat(paste(name, "R/inferCNV_tumor_subclusters.R", hash_file(path), basename(target), hash_file(target), sep="\t"),
            "\n", file=file.path(metadata, "functions.tsv"), append=TRUE, sep="")
    }
    imports <- c(CreateSeuratObject="Seurat", FindVariableFeatures="Seurat", ScaleData="Seurat",
        RunPCA="Seurat", FindNeighbors="Seurat", graph_from_adjacency_matrix="igraph",
        cluster_leiden="igraph", nn2="RANN", sparseMatrix="Matrix", as.phylo="ape")
    lines <- "function\tpackage\tenvironment\tinstalled_text\tsha256"
    for (name in names(imports)) {
        installed <- get(name, ns)
        if (!identical(installed, getExportedValue(imports[[name]], name))) stop("installed imported binding differs: ", name)
        target <- file.path(metadata, paste0("import-", name, ".R"))
        writeLines(deparse(installed), target)
        lines <- c(lines, paste(name, imports[[name]], environmentName(environment(installed)),
            basename(target), hash_file(target), sep="\t"))
        if (imports[[name]] == "Seurat") {
            for (class in c("Seurat", "Assay", "Assay5", "StdAssay", "default")) {
                method <- utils::getS3method(name, class, optional=TRUE, envir=asNamespace("Seurat"))
                if (is.null(method)) next
                target <- file.path(metadata, paste0("method-", name, ".", class, ".R"))
                writeLines(deparse(method), target)
                lines <- c(lines, paste(paste0(name, ".", class), "Seurat/SeuratObject",
                    environmentName(environment(method)), basename(target), hash_file(target), sep="\t"))
            }
        }
    }
    writeLines(lines, file.path(metadata, "imported-functions.tsv"))
    invisible(NULL)
}

leiden_knn_argument <- function(args) {
    if (length(args) == 2L) return(20L)
    if (length(args) != 3L || !args[[3L]] %in% c("5", "20")) {
        stop("usage: infercnv_leiden_workflow.R SHIPPED OUTPUT [5|20]")
    }
    as.integer(args[[3L]])
}

leiden_workflow_main <- function() {
    args <- commandArgs(trailingOnly=TRUE)
    k_nn <- leiden_knn_argument(args)
    pin <- list(name="infercnv", version="1.28.0",
        source_commit="b421d9405c97a309b081ef86d455e976df93eae4",
        source_archive_sha256="b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207")
    if (!identical(as.character(getRversion()), "4.6.1") ||
        !identical(Sys.getenv("INFER_CNV_SOURCE_COMMIT"), pin$source_commit) ||
        !identical(Sys.getenv("INFER_CNV_SOURCE_SHA256"), pin$source_archive_sha256)) stop("runtime/source environment identity differs")
    packages <- c("infercnv", "Seurat", "SeuratObject", "igraph", "RANN", "Matrix", "irlba",
        "ape", "fastcluster", "parallelDist", "jsonlite", "digest")
    for (package in packages) {
        if (!requireNamespace(package, quietly=TRUE)) stop("required installed package absent: ", package)
    }
    if (as.character(utils::packageVersion("infercnv")) != pin$version) stop("infercnv package identity differs")
    archive <- Sys.getenv("INFER_CNV_SOURCE_ARCHIVE")
    if (!file.exists(archive) || !identical(hash_file(archive), pin$source_archive_sha256)) stop("source archive bytes differ")
    bundle <- load_bundle(args[[1L]], "shipped", pin)
    root <- normalizePath(args[[2L]], mustWork=FALSE)
    if (file.exists(root)) stop("output directory already exists")
    dir.create(root, recursive=TRUE)
    if (!dir.exists(root)) stop("cannot create output directory")
    metadata <- file.path(root, "metadata")
    source_directory <- file.path(metadata, "source")
    dir.create(source_directory, recursive=TRUE)
    if (!file.copy(archive, file.path(source_directory, "infercnv-source.tar.gz"))) stop("cannot retain pinned source archive")
    prefix <- paste0("infercnv-", pin$source_commit)
    members <- paste0(prefix, "/", c("R/inferCNV_ops.R", "R/inferCNV_tumor_subclusters.R", "DESCRIPTION", "NAMESPACE"))
    listing <- utils::untar(archive, list=TRUE)
    if (anyDuplicated(listing) || any(!members %in% listing)) stop("source archive members differ")
    utils::untar(archive, files=members, exdir=source_directory)
    ns <- asNamespace("infercnv")
    leiden_validate_source(ns, file.path(source_directory, prefix), metadata)
    for (name in c("infercnv_leiden_workflow.R", "test_infercnv_leiden_workflow.R",
                   "infercnv_samples_clustering_witness.R", "infercnv_oracle_io.R")) {
        if (!file.copy(file.path(script_dir, name), file.path(metadata, name))) stop("cannot retain capture script: ", name)
    }
    for (name in c("sha256-manifest.tsv", "oracle.json")) {
        if (!file.copy(bundle_file(bundle$root, name), file.path(metadata, paste0("shipped-", name)))) {
            stop("cannot retain verified shipped bundle metadata")
        }
    }
    enforce_samples_locale()
    Sys.setenv(OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", MKL_NUM_THREADS="1")
    context <- get("infercnv.env", ns)
    context$GLOBAL_NUM_THREADS <- 1L
    leiden_install_traces(ns, leiden_new_capture())
    on.exit(leiden_remove_traces(ns), add=TRUE)
    entry <- bundle$record$cases$full
    refs <- as.character(unlist(entry$reference_groups, use.names=FALSE))
    inputs <- list(counts=bundle$record$inputs[[entry$input_counts]],
        gene_order=bundle$record$inputs$original_gene_order, annotations=bundle$record$inputs$original_annotations)
    input_records <- lapply(names(inputs), function(name) {
        relative <- file.path("metadata", paste0("input-", name, ".tsv"))
        if (!file.copy(bundle_file(bundle$root, inputs[[name]]), file.path(root, relative))) stop("cannot retain raw input")
        list(role=name, source_path=inputs[[name]], checkpoint=relative, sha256=bundle$hashes[[inputs[[name]]]])
    })
    seed <- 261027L
    cases <- list()
    for (method in c("PCA", "simple")) {
        relative <- file.path("cases", method)
        directory <- file.path(root, relative)
        dir.create(directory, recursive=TRUE)
        parameters <- run_parameters(bundle$record$run_settings, TRUE, 0.8, file.path(directory, "upstream"))
        parameters$analysis_mode <- "subclusters"
        parameters$tumor_subcluster_partition_method <- "leiden"
        parameters$k_nn <- k_nn
        parameters$leiden_method <- method
        parameters$leiden_function <- "CPM"
        parameters$leiden_resolution <- "auto"
        parameters$per_chr_hmm_subclusters <- FALSE
        parameters$per_chr_hmm_subclusters_references <- FALSE
        parameters$HMM <- FALSE
        parameters$BayesMaxPNormal <- 0
        parameters$up_to_step <- 15L
        parameters$save_rds <- FALSE
        capture <- leiden_new_capture()
        assign(".leiden_workflow_capture", capture, .GlobalEnv)
        cat("Running actual raw-input full184 Leiden workflow: ", method, "\n", sep="")
        started <- proc.time()[["elapsed"]]
        result <- capture_call(function() {
            capture$run_rng_before <- leiden_rng()
            object <- infercnv::CreateInfercnvObject(
                raw_counts_matrix=bundle_file(bundle$root, inputs$counts),
                gene_order_file=bundle_file(bundle$root, inputs$gene_order),
                annotations_file=bundle_file(bundle$root, inputs$annotations),
                ref_group_names=refs, delim="\t", max_cells_per_group=NULL,
                min_max_counts_per_cell=c(1, Inf), chr_exclude=c("chrX", "chrY", "chrM"))
            capture$created <- object
            if (ncol(object@expr.data) != 184L) stop("shipped full input does not contain the required 184 cells")
            do.call(infercnv::run, c(list(infercnv_obj=object), parameters))
        }, file.path(directory, "call.log"), capture, context, seed)
        wall_seconds <- proc.time()[["elapsed"]] - started
        capture$run_rng_after <- leiden_rng()
        recorded_parameters <- parameters
        recorded_parameters$out_dir <- file.path(relative, "upstream")
        replay <- leiden_save(list(parameters=parameters, result=result, frames=as.list(capture, all.names=TRUE)),
            file.path(relative, "actual-call-and-frames.rds"), root)
        export_warnings <- list()
        exported <- tryCatch(withCallingHandlers(
            leiden_export_case(result, capture, relative, root, recorded_parameters, wall_seconds),
            warning=function(condition) {
                export_warnings[[length(export_warnings) + 1L]] <<- condition_record(condition)
                invokeRestart("muffleWarning")
            }), error=function(condition) list(status="capture_error", source_status=result$status,
                error=result$error, warnings=result$warnings, export_error=condition_record(condition)))
        exported$inputs <- list(bundle="shipped", profile="full", references=array(refs), raw_inputs=input_records)
        exported$profile <- method
        exported$replay <- replay
        exported$export_warnings <- export_warnings
        cases[[method]] <- exported
        if (exported$status != "success") {
            jsonlite::write_json(list(status=exported$status, source_status=result$status,
                error=result$error, warnings=result$warnings, export_error=exported$export_error),
                file.path(directory, "failure.json"), auto_unbox=TRUE, pretty=TRUE, null="null")
        }
        cat("Recorded actual Leiden profile: ", method, "; outcome=", exported$status,
            "; positive PCA groups=", if (is.null(exported$positive_pca_groups)) "unavailable" else exported$positive_pca_groups,
            "\n", sep="")
    }
    writeLines(capture.output(sessionInfo()), file.path(metadata, "session.txt"))
    installed <- utils::installed.packages()[, c("Package", "Version", "LibPath"), drop=FALSE]
    write.table(installed[order(installed[, "Package"], method="radix"), , drop=FALSE],
        file.path(metadata, "packages.tsv"), sep="\t", quote=FALSE, row.names=FALSE)
    failures <- names(Filter(function(case) case$status != "success" || length(case$partition_mismatch_groups) > 0L, cases))
    record <- list(schema_version=1L, status=if (length(failures)) "captured_with_failures" else "complete",
        package=pin, index_base=1L,
        scope=sprintf("actual raw-input full184 PCA and simple Leiden workflows, k%d%s, through step15; not native acceptance",
                      k_nn, if (k_nn == 20L) " (default)" else " (non-default)"),
        source_bundles=list(shipped_manifest=bundle$manifest_sha256, shipped_oracle=bundle$oracle_sha256),
        runtime=list(R_version=as.character(getRversion()), seed=seed, rng_kind=array(RNGkind()), locale=Sys.getlocale(),
            requested_threads=1L, package_versions=stats::setNames(lapply(packages,
                function(package) as.character(getNamespaceVersion(package))), packages),
            package_library_paths=stats::setNames(lapply(packages,
                function(package) getNamespaceInfo(asNamespace(package), "path")), packages),
            thread_environment=as.list(Sys.getenv(c("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS")))),
        protocol=list(k_nn=k_nn, wrapper_frames="entry and exit of pinned discovery, single-group, PCA and simple functions",
            creation=list(delim="\t", max_cells_per_group=NULL, min_max_counts_per_cell=array(c(1, Inf)),
                chr_exclude=array(c("chrX", "chrY", "chrM"))),
            imported_method_tracing=FALSE, unchanged_source_validated_before_tracing=TRUE,
            internally_handled_warnings="source FindVariableFeatures warning handler logs its message in each actual call.log",
            seurat_parameters="original Seurat object and @commands plus readable command parameter dumps and installed method texts",
            simple_graph="actual source nn2 indices and binary sparse adjacency; no invented SNN Jaccard weights",
            timing="in-process raw creation through return including tracing; not paired performance"),
        successful_cases=array(setdiff(names(cases), failures)), failed_cases=array(failures), cases=cases)
    leiden_write_witness(record, root)
    if (length(failures)) stop("actual Leiden failures or mapping mismatches preserved: ", paste(failures, collapse=", "))
    cat("Captured both raw-input Leiden profiles; inspect real routes and positive PCA counts before claiming PCA evidence.\n")
}

if (sys.nframe() == 0L) leiden_workflow_main()
