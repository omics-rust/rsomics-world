#!/usr/bin/env Rscript

args <- commandArgs(trailingOnly=TRUE)
if (length(args) != 1L) stop("usage: infercnv_oracle.R OUTPUT_DIRECTORY")
root <- normalizePath(args[[1L]], mustWork=FALSE)
if (file.exists(root)) stop("output directory already exists: ", root)
dir.create(root, recursive=TRUE, showWarnings=FALSE)
if (!dir.exists(root)) stop("cannot create output directory")

commit <- "b421d9405c97a309b081ef86d455e976df93eae4"
source_commit <- Sys.getenv("INFER_CNV_SOURCE_COMMIT", unset="")
source_sha <- Sys.getenv("INFER_CNV_SOURCE_SHA256", unset="")
if (!identical(source_commit, commit) || !grepl("^[0-9a-f]{64}$", source_sha)) {
    stop("pinned infercnv source identity environment is missing or wrong")
}
if (!requireNamespace("infercnv", quietly=TRUE) ||
    as.character(utils::packageVersion("infercnv")) != "1.28.0") {
    stop("infercnv 1.28.0 must be installed")
}
if (!requireNamespace("jsonlite", quietly=TRUE) || !requireNamespace("digest", quietly=TRUE)) {
    stop("jsonlite and digest are required for provenance")
}

seed <- 260926L
set.seed(seed)
genes <- c(sprintf("N%03d", 1:300), sprintf("G%03d", 1:128),
           sprintf("L%03d", 1:17), "S001", "LOW", "SPARSE")
chromosomes <- c(rep("chrNeutral", 300), rep("chrGain", 128),
                 rep("chrLoss", 17), "chrSolo", "chrGain", "chrGain")
positions <- c(seq_len(300), seq_len(128), seq_len(17), 1L, 129L, 130L)
order <- data.frame(gene=genes, chr=chromosomes,
                    start=positions * 100L, stop=positions * 100L + 49L,
                    stringsAsFactors=FALSE)
cells <- c(sprintf("A%d", 1:3), sprintf("B%d", 1:4),
           sprintf("GAIN%d", 1:3), sprintf("LOSS%d", 1:3))
groups <- c(rep("normal_a", 3), rep("normal_b", 4),
            rep("gain_obs", 3), rep("loss_obs", 3))
counts <- matrix(0L, nrow=length(genes) + 1L, ncol=length(cells),
                 dimnames=list(c(genes, "ABSENT_FROM_ORDER"), cells))
for (j in seq_along(cells)) {
    counts[, j] <- 20L + (seq_len(nrow(counts)) + j) %% 3L
    if (groups[j] == "normal_b") counts[order$chr == "chrGain", j] <- 26L + j %% 2L
    if (groups[j] == "gain_obs") counts[order$chr == "chrGain", j] <- 48L + j %% 3L
    if (groups[j] == "gain_obs") counts[order$chr == "chrLoss", j] <- 4L + j %% 2L
    if (groups[j] == "loss_obs") counts[order$chr == "chrLoss", j] <- 4L + j %% 2L
}
counts["LOW", ] <- 0L
counts["SPARSE", ] <- 0L
counts["SPARSE", "A1"] <- 100L
counts <- counts[sample(seq_len(nrow(counts))), sample(seq_len(ncol(counts))), drop=FALSE]
order <- order[sample(seq_len(nrow(order))), , drop=FALSE]
annotations <- data.frame(cell=sample(cells), stringsAsFactors=FALSE)
annotations$group <- groups[match(annotations$cell, cells)]

input_dir <- file.path(root, "inputs")
dir.create(input_dir)
counts_path <- file.path(input_dir, "counts.tsv")
annotations_path <- file.path(input_dir, "annotations.tsv")
order_path <- file.path(input_dir, "gene_order.tsv")
write.table(cbind(gene=rownames(counts), counts), counts_path, sep="\t",
            row.names=FALSE, col.names=TRUE, quote=FALSE)
write.table(annotations, annotations_path, sep="\t", row.names=FALSE,
            col.names=FALSE, quote=FALSE)
write.table(order, order_path, sep="\t", row.names=FALSE,
            col.names=FALSE, quote=FALSE)

metadata_dir <- file.path(root, "metadata")
dir.create(metadata_dir)
writeLines(capture.output(sessionInfo()), file.path(metadata_dir, "session.txt"))
installed <- utils::installed.packages()[, c("Package", "Version", "LibPath"), drop=FALSE]
write.table(installed[base::order(installed[, "Package"]), , drop=FALSE],
            file.path(metadata_dir, "packages.tsv"), sep="\t", row.names=FALSE,
            quote=FALSE)
writeLines(c(paste0("source_commit\t", commit),
             paste0("source_archive_sha256\t", source_sha),
             "source_url\thttps://github.com/bioconductor-source/infercnv"),
           file.path(metadata_dir, "source.txt"))

settings <- list(cutoff=0.1, min_cells_per_gene=3L, window_length=101L,
                 smooth_method="pyramidinal", max_centered_threshold=3,
                 scale_data=FALSE, HMM=FALSE, denoise=FALSE,
                 analysis_mode="samples", num_threads=1L,
                 resume_mode=FALSE, plot_steps=FALSE, no_plot=TRUE,
                 no_prelim_plot=TRUE, save_rds=TRUE,
                 remove_genes_at_chr_ends=FALSE, prune_outliers=FALSE,
                 mask_nonDE_genes=FALSE, up_to_step=14L,
                 num_ref_groups=NULL, cluster_by_groups=TRUE,
                 cluster_references=TRUE,
                 tumor_subcluster_partition_method="leiden",
                 BayesMaxPNormal=0.5)
profiles <- list(single_reference=list(ref_group_names=c("normal_a"),
                                       ref_subtract_use_mean_bounds=TRUE),
                 grouped_bounds=list(ref_group_names=c("normal_a", "normal_b"),
                                     ref_subtract_use_mean_bounds=TRUE),
                 grouped_mean=list(ref_group_names=c("normal_a", "normal_b"),
                                   ref_subtract_use_mean_bounds=FALSE),
                 no_reference=list(ref_group_names=character(0),
                                   ref_subtract_use_mean_bounds=TRUE))
steps <- c(1L, 2L, 3L, 4L, 8L, 9L, 10L, 11L, 12L, 14L)
oracle_getter <- getFromNamespace(".get_relevant_args_list", "infercnv")

write_expression <- function(obj, path) {
    matrix <- as.matrix(obj@expr.data)
    if (!is.numeric(matrix) || !all(is.finite(matrix)) ||
        !identical(rownames(matrix), rownames(obj@gene_order)) ||
        anyDuplicated(rownames(matrix)) || anyDuplicated(colnames(matrix))) {
        stop("invalid checkpoint expression identity or finite values")
    }
    con <- file(path, open="wt")
    on.exit(close(con))
    writeLines(paste(c("gene", colnames(matrix)), collapse="\t"), con)
    for (i in seq_len(nrow(matrix))) {
        writeLines(paste(c(rownames(matrix)[i],
                           formatC(matrix[i, ], digits=17, format="g")),
                         collapse="\t"), con)
    }
    invisible(matrix)
}

write_identity <- function(obj, gene_path, cell_path) {
    gene_order <- obj@gene_order
    write.table(data.frame(gene=rownames(gene_order),
                           chr=as.character(gene_order$chr),
                           start=gene_order$start, stop=gene_order$stop),
                gene_path, sep="\t", row.names=FALSE, quote=FALSE)
    cell_names <- colnames(obj@expr.data)
    group <- role <- rep(NA_character_, length(cell_names))
    for (name in names(obj@reference_grouped_cell_indices)) {
        idx <- obj@reference_grouped_cell_indices[[name]]
        if (any(!is.na(group[idx]))) stop("duplicate reference cell index")
        group[idx] <- name
        role[idx] <- "reference"
    }
    for (name in names(obj@observation_grouped_cell_indices)) {
        idx <- obj@observation_grouped_cell_indices[[name]]
        if (any(!is.na(group[idx]))) stop("reference/observation identity overlap")
        group[idx] <- name
        role[idx] <- "observation"
    }
    if (anyNA(group) || anyNA(role)) stop("unassigned checkpoint cell")
    write.table(data.frame(cell=cell_names, group=group, role=role),
                cell_path, sep="\t", row.names=FALSE, quote=FALSE)
}

results <- list()
numerical_checks <- list()
for (name in names(profiles)) {
    profile <- profiles[[name]]
    profile_dir <- file.path(root, name)
    upstream_dir <- file.path(profile_dir, "upstream")
    dir.create(profile_dir)
    create_args <- list(raw_counts_matrix=counts_path,
                        gene_order_file=order_path,
                        annotations_file=annotations_path,
                        ref_group_names=if (length(profile$ref_group_names)) profile$ref_group_names else NULL,
                        delim="\t", max_cells_per_group=NULL,
                        min_max_counts_per_cell=c(1, Inf),
                        chr_exclude=c("chrX", "chrY", "chrM"))
    obj <- do.call(infercnv::CreateInfercnvObject, create_args)
    run_args <- c(list(out_dir=upstream_dir), settings,
                  list(ref_subtract_use_mean_bounds=profile$ref_subtract_use_mean_bounds))
    expected <- do.call(oracle_getter, run_args)$expected_file_names
    completed <- do.call(infercnv::run, c(list(infercnv_obj=obj), run_args))
    stage_map <- list()
    for (step in steps) {
        checkpoint <- expected[[step]]
        if (!nzchar(checkpoint) || !file.exists(checkpoint)) {
            stop("missing upstream checkpoint for ", name, " stage ", step)
        }
        stage_obj <- readRDS(checkpoint)
        prefix <- sprintf("%02d", step)
        expression_rel <- file.path(name, paste0(prefix, ".tsv"))
        gene_rel <- file.path(name, paste0(prefix, ".genes.tsv"))
        cell_rel <- file.path(name, paste0(prefix, ".cells.tsv"))
        write_expression(stage_obj, file.path(root, expression_rel))
        write_identity(stage_obj, file.path(root, gene_rel), file.path(root, cell_rel))
        stage_map[[as.character(step)]] <- list(
            expression=expression_rel, gene_order=gene_rel,
            cell_groups=cell_rel,
            checkpoint=file.path(name, "upstream", basename(checkpoint)))
    }
    if (!identical(rownames(completed@expr.data),
                   rownames(readRDS(expected[[14L]])@expr.data))) {
        stop("returned step 14 object differs from checkpoint")
    }
    stage2 <- readRDS(expected[[2L]])@expr.data
    stage3 <- readRDS(expected[[3L]])@expr.data
    stage4 <- readRDS(expected[[4L]])@expr.data
    stage12 <- readRDS(expected[[12L]])@expr.data
    stage14 <- readRDS(expected[[14L]])@expr.data
    if (!isTRUE(all.equal(as.numeric(colSums(stage3)),
                          rep(median(colSums(stage2)), ncol(stage3)),
                          tolerance=1e-9)) ||
        !isTRUE(all.equal(stage4, log2(stage3 + 1), tolerance=1e-10)) ||
        !isTRUE(all.equal(stage14, 2^stage12, tolerance=1e-10))) {
        stop("checkpoint arithmetic invariant failed for ", name)
    }
    json_profile <- profile
    json_profile$ref_group_names <- as.list(profile$ref_group_names)
    results[[name]] <- list(settings=json_profile, stages=stage_map)
    numerical_checks[[name]] <- list(
        stage8=readRDS(expected[[8L]])@expr.data,
        stage12=stage12,
        stage14=stage14,
        gene_order=readRDS(expected[[14L]])@gene_order,
        observations=readRDS(expected[[14L]])@observation_grouped_cell_indices)
}

single <- numerical_checks$single_reference
gain_rows <- which(as.character(single$gene_order$chr) == "chrGain")
loss_rows <- which(as.character(single$gene_order$chr) == "chrLoss")
gain_cells <- single$observations$gain_obs
loss_cells <- single$observations$loss_obs
if (!length(gain_rows) || !length(loss_rows) || !length(gain_cells) || !length(loss_cells) ||
    median(single$stage12[gain_rows, gain_cells, drop=FALSE]) <= 0 ||
    median(single$stage12[loss_rows, gain_cells, drop=FALSE]) >= 0 ||
    median(single$stage14[gain_rows, gain_cells, drop=FALSE]) <= 1 ||
    median(single$stage14[loss_rows, loss_cells, drop=FALSE]) >= 1) {
    stop("fixture gain/loss signal did not survive upstream processing")
}
if (identical(numerical_checks$grouped_bounds$stage8,
              numerical_checks$grouped_mean$stage8) ||
    identical(numerical_checks$single_reference$stage8,
              numerical_checks$grouped_bounds$stage8) ||
    identical(numerical_checks$single_reference$stage8,
              numerical_checks$no_reference$stage8)) {
    stop("oracle reference profiles collapsed to identical step-8 output")
}

inputs <- list("counts.tsv"="inputs/counts.tsv",
               "annotations.tsv"="inputs/annotations.tsv",
               "gene_order.tsv"="inputs/gene_order.tsv")
metadata <- list("session.txt"="metadata/session.txt",
                 "packages.tsv"="metadata/packages.tsv",
                 "source.txt"="metadata/source.txt")
sha256 <- lapply(inputs, function(path) digest::digest(file.path(root, path),
                                                      algo="sha256", file=TRUE))
record <- list(schema_version=1L,
               package=list(name="infercnv", version="1.28.0",
                            source_commit=commit,
                            source_archive_sha256=source_sha),
               seed=seed, inputs=inputs, input_sha256=sha256,
               metadata=metadata, settings=settings,
               creation_settings=list(max_cells_per_group=NULL,
                                      min_max_counts_per_cell=list(1L, "Inf"),
                                      chr_exclude=c("chrX", "chrY", "chrM")),
               profiles=results)
jsonlite::write_json(record, file.path(root, "oracle.json"),
                     auto_unbox=TRUE, pretty=TRUE, null="null")
