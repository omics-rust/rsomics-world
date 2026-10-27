#!/usr/bin/env Rscript

script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
script_dir <- dirname(normalizePath(sub("^--file=", "", script)))
driver <- file.path(script_dir, "infercnv_leiden_workflow.R")
if (file.exists(driver)) source(driver)
if (!exists("leiden_partition_record", mode="function")) stop("Leiden vertex/global-cell recorder is missing")

indices <- c(cellB=7L, cellC=4L, cellA=12L, cellD=3L)
vertices <- c("cellD", "cellB", "cellA", "cellC")
membership <- c(9L, 2L, 9L, 9L)
members <- list(tumor_s9=c(cellA=12L, cellC=4L, cellD=3L), tumor_s2=c(cellB=7L))
record <- leiden_partition_record(indices, vertices, membership, members)
stopifnot(identical(unlist(record$global_indices), c(3L, 7L, 12L, 4L)),
          identical(unlist(record$vertex_cells), vertices),
          identical(unlist(record$membership), membership),
          record$matches_returned_subclusters,
          identical(vapply(record$subclusters, `[[`, character(1), "name"), c("tumor_s9", "tumor_s2")),
          identical(unlist(record$subclusters[[1L]]$indices), c(12L, 4L, 3L)))
positional <- list(tumor_s9=c(cellB=7L, cellA=12L, cellD=3L), tumor_s2=c(cellC=4L))
stopifnot(!leiden_partition_record(indices, vertices, membership, positional)$matches_returned_subclusters)
unnamed <- leiden_partition_record(unname(indices), vertices, membership, members, input_cells=names(indices))
stopifnot(identical(unlist(unnamed$global_indices), c(3L, 7L, 12L, 4L)), unnamed$matches_returned_subclusters)
cat("Leiden memberships map graph identities to global cells without relabeling package subgroups\n")

small <- leiden_partition_record(c(cellD=3L, cellB=7L), NULL, NULL,
                                 list(tumor_s1=c(cellD=3L, cellB=7L)))
stopifnot(is.null(small$vertex_cells), is.null(small$global_indices), is.null(small$membership),
          is.null(small$matches_returned_subclusters),
          identical(unlist(small$subclusters[[1L]]$indices), c(3L, 7L)))
cat("Small-group records retain actual subgroups without inventing graph vertices or a partition\n")

archive <- Sys.getenv("INFER_CNV_SOURCE_ARCHIVE")
if (nzchar(archive)) {
    member <- paste0("infercnv-b421d9405c97a309b081ef86d455e976df93eae4/R/inferCNV_tumor_subclusters.R")
    directory <- tempfile("leiden-source-")
    dir.create(directory)
    utils::untar(archive, files=member, exdir=directory)
    probe <- new.env(parent=.GlobalEnv)
    source(file.path(directory, member), local=probe)
    probe$flog.info <- function(...) invisible(NULL)
    methods::setClass("LeidenRecordProbe", slots=c(expr.data="matrix", count.data="matrix",
        gene_order="data.frame", reference_grouped_cell_indices="list",
        observation_grouped_cell_indices="list", .hspike="ANY", tumor_subclusters="list"))
    expression <- matrix(c(1, 0, 1, 1, 0, 5, 1, 0, 1, 1, 0, 5), nrow=3L,
                         dimnames=list(c("geneA", "geneB", "geneC"), paste0("cell", 1:4)))
    object <- methods::new("LeidenRecordProbe", expr.data=expression, count.data=expression,
        gene_order=data.frame(chr="chr1", start=1:3, stop=2:4, row.names=rownames(expression)),
        reference_grouped_cell_indices=list(ref=c(4L, 2L)),
        observation_grouped_cell_indices=list(obs=c(3L, 1L)), .hspike=NULL, tumor_subclusters=list())
    capture <- leiden_new_capture()
    leiden_install_traces(probe, capture)
    result <- probe$define_signif_tumor_subclusters(object, partition_method="leiden", z_score_filter=0.8)
    leiden_remove_traces(probe)
    stopifnot(capture$entries == 1L, capture$depth == 0L,
              identical(capture$input@expr.data, expression),
              identical(capture$local$expr.data, expression[1L, , drop=FALSE]),
              identical(capture$local$outliers, c(geneB=2L, geneC=3L)),
              identical(vapply(capture$groups, `[[`, character(1), "name"), c("obs", "ref")),
              identical(capture$groups[[1L]]$indices, c(3L, 1L)),
              identical(capture$groups[[1L]]$value$subclusters$obs_s1, c(3L, 1L)),
              is.null(capture$groups[[1L]]$pca), is.null(capture$groups[[1L]]$simple),
              identical(state_identity(result[[1L]]), state_identity(object)))
    cat("Unmodified pinned outer/single source retains step14, temporary filtering and real small-group bypasses\n")
    capture$created <- object
    output <- tempfile("leiden-record-")
    dir.create(output)
    dir.create(file.path(output, "cases", "probe"), recursive=TRUE)
    exported <- leiden_export_case(list(status="success", value=result[[1L]], warnings=list()),
        capture, "cases/probe", output, list(z_score_filter=0.8), 0)
    leiden_write_witness(list(schema_version=1L, status="complete", cases=list(probe=exported)), output)
    witness <- jsonlite::fromJSON(file.path(output, "witness.json"), simplifyVector=FALSE)
    manifest <- utils::read.delim(file.path(output, "sha256-manifest.tsv"), stringsAsFactors=FALSE)
    stopifnot(witness$cases$probe$status == "success", length(witness$cases$probe$groups) == 2L,
              witness$cases$probe$positive_pca_groups == 0L,
              witness$cases$probe$groups[[1L]]$route == "fewer_than_three_cells",
              is.null(witness$cases$probe$groups[[1L]]$graph),
              is.null(witness$cases$probe$groups[[1L]]$partition$membership),
              identical(readRDS(file.path(output, exported$step14_checkpoint))@expr.data, expression),
              "witness.json" %in% manifest$path,
              all(vapply(seq_len(nrow(manifest)), function(i) identical(manifest$sha256[[i]],
                  digest::digest(file.path(output, manifest$path[[i]]), algo="sha256", file=TRUE)), logical(1))))
    cat("The source-based no-graph capture exports replay objects and an independently checksummed witness\n")
    changed_capture <- list2env(as.list(capture), parent=emptyenv())
    changed_capture$input <- result[[1L]]
    changed_capture$input@expr.data[1L, 1L] <- 99
    changed_capture$result <- list(changed_capture$input, NULL)
    dir.create(file.path(output, "cases", "changed"))
    changed <- tryCatch(leiden_export_case(list(status="success", value=changed_capture$input, warnings=list()),
        changed_capture, "cases/changed", output, list(z_score_filter=0.8), 0), error=function(condition) condition)
    stopifnot(inherits(changed, "error"), grepl("entry snapshot", conditionMessage(changed), fixed=TRUE))
    cat("An altered entry/return pair cannot be labeled unchanged when the entry snapshot disagrees\n")

    larger <- object
    larger@expr.data <- larger@count.data <- matrix(seq_len(66L), nrow=3L,
        dimnames=list(rownames(expression), paste0("cell", 1:22)))
    larger@reference_grouped_cell_indices <- list()
    larger@observation_grouped_cell_indices <- list(obs=seq_len(22L))
    failure_capture <- leiden_new_capture()
    failure_capture$created <- larger
    leiden_install_traces(probe, failure_capture)
    failed <- capture_call(function() probe$define_signif_tumor_subclusters(larger,
        partition_method="leiden", leiden_method="PCA", z_score_filter=0)[[1L]],
        file.path(output, "source-error.log"), failure_capture, new.env(), 261027L)
    leiden_remove_traces(probe)
    dir.create(file.path(output, "cases", "error"))
    failed_export <- leiden_export_case(failed, failure_capture, "cases/error", output, list(z_score_filter=0), 0)
    stopifnot(failed_export$status == "error", failure_capture$depth == 0L,
              failed_export$positive_pca_groups == 0L,
              failed_export$groups[[1L]]$route == "PCA",
              !failed_export$groups[[1L]]$seurat_wrapper_object_present,
              is.null(failed_export$groups[[1L]]$graph),
              is.null(failed_export$returned_checkpoint),
              !is.null(readRDS(file.path(output, failed_export$raw_frames))$groups[[1L]]$pca))
    cat("A real source call with its Seurat dependency absent preserves the failed PCA frame without positive graph evidence\n")
} else {
    cat("Pinned-source trace test skipped: INFER_CNV_SOURCE_ARCHIVE is not set\n")
}
