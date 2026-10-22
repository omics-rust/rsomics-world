#!/usr/bin/env Rscript

array <- function(value) unname(as.list(value))
hash_file <- function(path) digest::digest(path, algo="sha256", file=TRUE)
call_text <- function(value) if (is.null(value)) NULL else paste(deparse(value), collapse="\n")

typed_shape <- function(value) {
    result <- list(typeof=typeof(value), classes=array(class(value)), length=length(value),
                   names=if (is.null(names(value))) NULL else array(names(value)))
    if (is.null(value)) return(result)
    if (typeof(value) %in% c("list", "expression", "pairlist")) {
        result$items <- unname(lapply(as.list(value), typed_shape))
    } else if (is.language(value)) {
        result$text <- call_text(value)
    } else {
        literal <- function(item) {
            if (is.double(item)) {
                if (is.nan(item)) return("NaN")
                if (is.na(item)) return("NA")
                return(sprintf("%.17g", item))
            }
            if (is.na(item)) return("NA")
            if (is.character(item)) return(encodeString(item, quote='"'))
            as.character(item)
        }
        result$values <- array(vapply(as.list(value), literal, character(1)))
    }
    result
}

state_identity <- function(obj) {
    list(classes=class(obj), expression=obj@expr.data, counts=obj@count.data,
         genes=obj@gene_order, references=obj@reference_grouped_cell_indices,
         observations=obj@observation_grouped_cell_indices, spike=obj@.hspike)
}

save_raw <- function(value, path) {
    saveRDS(value, path)
    if (!identical(readRDS(path), value)) stop("raw RDS read-back differs: ", path)
}

function_definition <- function(path, name) {
    definitions <- as.list(parse(path, keep.source=FALSE))
    matches <- Filter(function(value) is.call(value) && identical(value[[1L]], as.name("<-")) &&
        identical(value[[2L]], as.name(name)) && is.call(value[[3L]]) &&
        identical(value[[3L]][[1L]], as.name("function")), definitions)
    if (length(matches) != 1L) stop("missing unique source function: ", name)
    matches[[1L]][[3L]]
}

preserve <- function(from, to) {
    if (!file.copy(from, to, overwrite=FALSE)) stop("cannot preserve file: ", from)
    if (!identical(hash_file(from), hash_file(to))) stop("preserved bytes differ: ", from)
}

probe_object <- function() {
    expression <- matrix(seq_len(12L)/8+1, 4L, 3L)
    dimnames(expression) <- list(paste0("gene", seq_len(4L)), paste0("cell", seq_len(3L)))
    counts <- matrix(20, 4L, 3L, dimnames=dimnames(expression))
    order <- data.frame(chr=rep("chrProbe", 4L), start=seq_len(4L)*100L,
                        stop=seq_len(4L)*100L+49L, row.names=rownames(expression))
    annotations <- data.frame(group=rep("ref", 3L), row.names=colnames(expression))
    obj <- infercnv::CreateInfercnvObject(raw_counts_matrix=counts, gene_order_file=order,
        annotations_file=annotations, ref_group_names="ref", delim="\t",
        max_cells_per_group=NULL, min_max_counts_per_cell=c(1, Inf),
        chr_exclude=c("chrX", "chrY", "chrM"))
    obj@expr.data <- expression
    methods::validObject(obj)
    if (length(obj@observation_grouped_cell_indices) != 0L ||
        length(unlist(obj@observation_grouped_cell_indices, use.names=FALSE)) != 0L ||
        !identical(names(obj@reference_grouped_cell_indices), "ref") ||
        !identical(as.integer(obj@reference_grouped_cell_indices[["ref"]]), seq_len(3L))) {
        stop("factory did not create the all-reference object")
    }
    obj
}

capture_route <- function(id, obj, parameters, ns, root, seed) {
    log <- paste0(id, ".log")
    raw_rds <- paste0(id, ".rds")
    before <- state_identity(obj)
    if (!identical(obj, readRDS(file.path(root, "input.rds")))) stop("input differs from original RDS")
    if (length(obj@observation_grouped_cell_indices) != 0L) stop("input has observations")
    arguments <- c(list(infercnv_obj=obj), parameters, list(cluster_by_groups=identical(id, "grouped")))
    actual_call <- as.call(c(list(as.name("define_signif_tumor_subclusters")), arguments))
    warnings <- character()
    connection <- file(file.path(root, log), "wt")
    sink(connection)
    sink(connection, type="message")
    captured <- tryCatch({
        set.seed(seed)
        value <- withCallingHandlers(do.call(get("define_signif_tumor_subclusters", envir=ns), arguments),
            warning=function(condition) {
                warnings <<- c(warnings, conditionMessage(condition))
                message("Warning: ", conditionMessage(condition))
                invokeRestart("muffleWarning")
            })
        list(status="success", value=value, condition=NULL)
    }, error=function(condition) list(status="error", value=NULL, condition=condition),
       finally={sink(type="message"); sink(); close(connection)})
    raw <- c(list(call=actual_call, input=obj, warnings=warnings), captured)
    save_raw(raw, file.path(root, raw_rds))
    if (!identical(obj, readRDS(file.path(root, "input.rds")))) stop("call mutated original input")
    result <- list(id=id, status=captured$status, warnings=array(warnings), error=NULL,
                   log=log, raw_rds=raw_rds, result_shape=NULL, unchanged_state=NULL)
    if (captured$status == "error") {
        condition <- captured$condition
        result$error <- list(classes=array(class(condition)), message=conditionMessage(condition),
                             call=call_text(conditionCall(condition)))
        return(result)
    }
    value <- captured$value
    if (!is.list(value) || length(value) != 2L || !methods::is(value[[1L]], "infercnv")) {
        stop("successful clustering returned an unrecognized object")
    }
    if (!identical(state_identity(obj), before) || !identical(state_identity(value[[1L]]), before)) {
        stop("successful clustering changed original input state")
    }
    result$result_shape <- list(tumor_subclusters=typed_shape(value[[1L]]@tumor_subclusters),
                                second=typed_shape(value[[2L]]))
    result$unchanged_state <- TRUE
    result
}

initialize_samples_locale <- function() {
    if (!nzchar(Sys.setlocale("LC_ALL", "C"))) stop("cannot initialize C locale")
    for (category in c("LC_MESSAGES", "LC_PAPER", "LC_MEASUREMENT")) {
        if (!identical(Sys.setlocale(category, "C"), "C")) stop("cannot enforce C locale category: ", category)
    }
    if (!identical(Sys.getlocale(), "C")) stop("cannot enforce C locale")
}

main <- function() {
    args <- commandArgs(trailingOnly=TRUE)
    if (length(args) != 2L) stop("usage: infercnv_samples_reference_edges.R SOURCE_ROOT NEW_OUTPUT_DIR")
    source_root <- normalizePath(args[[1L]], mustWork=TRUE)
    root <- normalizePath(args[[2L]], mustWork=FALSE)
    if (file.exists(root)) stop("output directory already exists: ", root)
    script_arg <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
    if (length(script_arg) != 1L) stop("cannot identify exporter script")
    script <- normalizePath(sub("^--file=", "", script_arg), mustWork=TRUE)
    script_dir <- dirname(script)
    source(file.path(script_dir, "infercnv_oracle_io.R"), local=.GlobalEnv)
    pin <- list(name="infercnv", version="1.28.0", source_commit="b421d9405c97a309b081ef86d455e976df93eae4",
                source_archive_sha256="b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207")
    if (!identical(as.character(getRversion()), "4.6.1") ||
        !identical(Sys.getenv("INFER_CNV_SOURCE_COMMIT"), pin$source_commit) ||
        !identical(Sys.getenv("INFER_CNV_SOURCE_SHA256"), pin$source_archive_sha256)) {
        stop("runtime or source environment pin mismatch")
    }
    packages <- c("infercnv", "jsonlite", "digest", "fastcluster", "parallelDist")
    for (package in packages) {
        if (!requireNamespace(package, quietly=TRUE)) stop("required installed package absent: ", package)
    }
    if (!identical(as.character(utils::packageVersion("infercnv")), pin$version)) stop("package version mismatch")
    archive <- Sys.getenv("INFER_CNV_SOURCE_ARCHIVE")
    if (!file.exists(archive) || !identical(hash_file(archive), pin$source_archive_sha256)) stop("source archive mismatch")
    source_path <- file.path(source_root, "R/inferCNV_tumor_subclusters.R")
    if (!identical(hash_file(source_path), "68ceb769fc96f12d16ef73f7c0c8beac7e1e32c21bd057355ce0c23bbdf6f9fb")) {
        stop("source function bytes mismatch")
    }
    if (!dir.create(root, recursive=TRUE)) stop("cannot create exclusive output directory")
    ns <- asNamespace("infercnv")
    functions <- c("define_signif_tumor_subclusters", ".single_tumor_subclustering")
    function_paths <- c("function-main.txt", "function-single.txt")
    for (i in seq_along(functions)) {
        installed <- get(functions[[i]], envir=ns)
        definition <- function_definition(source_path, functions[[i]])
        if (!identical(call_text(formals(installed)), call_text(definition[[2L]])) ||
            !identical(call_text(body(installed)), call_text(definition[[3L]]))) {
            stop("installed function differs from source: ", functions[[i]])
        }
        writeLines(deparse(installed), file.path(root, function_paths[[i]]))
    }
    if (!identical(get("hclust", envir=ns), getExportedValue("fastcluster", "hclust")) ||
        !identical(get("parallelDist", envir=ns), getExportedValue("parallelDist", "parallelDist"))) {
        stop("namespace distance or tree binding differs")
    }
    preserve(source_path, file.path(root, "source.R"))
    preserve(archive, file.path(root, "source.tar.gz"))
    preserve(script, file.path(root, "exporter.R"))
    preserve(file.path(script_dir, "infercnv_oracle_io.R"), file.path(root, "io.R"))
    Sys.setenv(OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", MKL_NUM_THREADS="1")
    initialize_samples_locale()
    context <- get("infercnv.env", envir=ns)
    context$GLOBAL_NUM_THREADS <- 1L
    seed <- 260930L
    set.seed(seed)
    obj <- probe_object()
    save_raw(obj, file.path(root, "input.rds"))
    write_expression(obj, file.path(root, "expression.tsv"))
    write_identity(obj, file.path(root, "genes.tsv"), file.path(root, "cells.tsv"))
    writeLines(c("role\tgroup\tindex\tcell", vapply(seq_len(3L), function(i)
        paste("reference", "ref", i, colnames(obj@expr.data)[[i]], sep="\t"), character(1))), file.path(root, "maps.tsv"))
    parameters <- list(p_val=0.1, k_nn=20L, leiden_method="PCA", leiden_function="CPM",
        leiden_resolution="auto", leiden_method_per_chr="simple", leiden_function_per_chr="modularity",
        leiden_resolution_per_chr=1, hclust_method="ward.D2", partition_method="none",
        per_chr_hmm_subclusters=FALSE, per_chr_hmm_subclusters_references=FALSE,
        z_score_filter=0, restrict_to_DE_genes=FALSE)
    jsonlite::write_json(parameters, file.path(root, "arguments.json"), auto_unbox=TRUE, null="null", digits=NA)
    routes <- lapply(c("grouped", "pooled"), capture_route, obj=obj, parameters=parameters, ns=ns, root=root, seed=seed)
    runtime <- list(R_version=as.character(getRversion()), locale=Sys.getlocale(), seed=seed, threads=1L,
        package_versions=stats::setNames(lapply(packages, function(p) as.character(utils::packageVersion(p))), packages),
        loaded_paths=stats::setNames(lapply(packages, function(p) getNamespaceInfo(asNamespace(p), "path")), packages),
        source_root=source_root, source_archive="source.tar.gz", source_files=list("source.R"),
        functions=array(function_paths), bindings=list(hclust="fastcluster::hclust", parallelDist="parallelDist::parallelDist"),
        exporter="exporter.R", io="io.R", arguments_file="arguments.json")
    paths <- sort(list.files(root, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
    files <- lapply(paths, function(path) list(path=path, sha256=hash_file(file.path(root, path))))
    record <- list(schema_version=1L, package=pin, runtime=runtime,
        input=list(classes=array(class(obj)), genes=nrow(obj@expr.data), cells=ncol(obj@expr.data),
            reference_groups=array(names(obj@reference_grouped_cell_indices)),
            observation_groups=array(names(obj@observation_grouped_cell_indices)), rds="input.rds",
            state=list(expression="expression.tsv", genes="genes.tsv", cells="cells.tsv", maps="maps.tsv")),
        arguments=parameters, routes=routes, files=files)
    jsonlite::write_json(record, file.path(root, "witness.json"), auto_unbox=TRUE, pretty=TRUE,
                         null="null", na="null", digits=NA)
}

if (sys.nframe() == 0L) main()
