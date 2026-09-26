#!/usr/bin/env Rscript

args <- commandArgs(trailingOnly=TRUE)
if (length(args) != 1L) stop("usage: infercnv_shipped_oracle.R OUTPUT_DIRECTORY")
script_file <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
if (length(script_file) != 1L) stop("cannot identify shipped oracle script path")
source(file.path(dirname(normalizePath(sub("^--file=", "", script_file))),
                 "infercnv_oracle_io.R"), local=TRUE)

commit <- "b421d9405c97a309b081ef86d455e976df93eae4"
archive_sha <- "b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207"
original_hashes <- c(
    original_counts="7b7c618e6b03d589ea36979b074d0b83e127dcaa01e059ba38f73900e7609ba3",
    original_annotations="345493e0686d75418427e9c4401f3f7bbb55ae3f61deee074ccee8882dc4dc63",
    original_gene_order="4ec63e049ea8299948fb730c2f60ca5a038a798b77cec3bb918fda572b2dc52e")
if (Sys.getenv("INFER_CNV_SOURCE_COMMIT") != commit ||
    Sys.getenv("INFER_CNV_SOURCE_SHA256") != archive_sha) {
    stop("pinned source identity environment mismatch")
}
if (!requireNamespace("infercnv", quietly=TRUE) ||
    as.character(utils::packageVersion("infercnv")) != "1.28.0" ||
    !requireNamespace("jsonlite", quietly=TRUE) ||
    !requireNamespace("digest", quietly=TRUE)) {
    stop("installed infercnv 1.28.0, jsonlite and digest are required")
}
source_archive <- Sys.getenv("INFER_CNV_SOURCE_ARCHIVE")
if (!file.exists(source_archive) ||
    digest::digest(source_archive, algo="sha256", file=TRUE) != archive_sha) {
    stop("pinned source archive bytes missing or wrong")
}

upstream_names <- c(original_counts="oligodendroglioma_expression_downsampled.counts.matrix.gz",
                    original_annotations="oligodendroglioma_annotations_downsampled.txt",
                    original_gene_order="gencode_downsampled.EXAMPLE_ONLY_DONT_REUSE.txt")
installed_paths <- vapply(upstream_names, function(name) {
    path <- system.file("extdata", name, package="infercnv")
    if (!nzchar(path)) stop("missing installed upstream input: ", name)
    path
}, character(1))
for (key in names(installed_paths)) {
    if (digest::digest(installed_paths[[key]], algo="sha256", file=TRUE) != original_hashes[[key]]) {
        stop("upstream input SHA-256 mismatch: ", key)
    }
}

root <- normalizePath(args[[1L]], mustWork=FALSE)
if (file.exists(root)) stop("output directory already exists: ", root)
dir.create(root, recursive=TRUE, showWarnings=FALSE)
if (!dir.exists(root)) stop("cannot create output directory")
dir.create(file.path(root, "inputs"))
dir.create(file.path(root, "metadata"))
dir.create(file.path(root, "source"))
inputs <- list(source_archive="source/infercnv-source.tar.gz",
               original_counts=paste0("inputs/", upstream_names[["original_counts"]]),
               original_annotations=paste0("inputs/", upstream_names[["original_annotations"]]),
               original_gene_order=paste0("inputs/", upstream_names[["original_gene_order"]]),
               canonical_full="inputs/canonical-full.tsv",
               canonical_subset="inputs/canonical-subset.tsv")
if (!file.copy(source_archive, file.path(root, inputs$source_archive))) stop("cannot preserve source archive")
for (key in names(upstream_names)) {
    if (!file.copy(installed_paths[[key]], file.path(root, inputs[[key]]))) {
        stop("cannot preserve original input: ", key)
    }
}

count_path <- file.path(root, inputs$original_counts)
header_con <- gzfile(count_path, open="rt")
header <- strsplit(readLines(header_con, n=1L), "\t", fixed=TRUE)[[1L]]
close(header_con)
if (length(header) != 184L || anyNA(header) || any(!nzchar(header)) || anyDuplicated(header)) {
    stop("original gzip must have exactly 184 unique cell-header fields")
}
counts_con <- gzfile(count_path, open="rt")
counts <- read.table(counts_con, sep="\t", header=FALSE, skip=1L,
                     row.names=1L, col.names=c("gene", header), check.names=FALSE,
                     stringsAsFactors=FALSE, quote="", comment.char="")
close(counts_con)
counts <- as.matrix(counts)
storage.mode(counts) <- "double"
if (nrow(counts) != 10338L || ncol(counts) != 184L ||
    anyDuplicated(rownames(counts)) || !identical(colnames(counts), header) ||
    any(!is.finite(counts)) || any(counts < 0)) {
    stop("original shipped matrix identity, dimensions or counts invalid")
}
order <- read.table(file.path(root, inputs$original_gene_order), sep="\t",
                    header=FALSE, row.names=1L, check.names=FALSE,
                    stringsAsFactors=FALSE, quote="", comment.char="")
colnames(order) <- c("chr", "start", "stop")
annotations <- read.table(file.path(root, inputs$original_annotations), sep="\t",
                          header=FALSE, row.names=1L, check.names=FALSE,
                          stringsAsFactors=FALSE, quote="", comment.char="")
if (anyDuplicated(rownames(order)) || anyDuplicated(rownames(annotations)) ||
    !setequal(rownames(order), rownames(counts)) ||
    !setequal(rownames(annotations), colnames(counts)) ||
    length(unique(annotations[[1L]])) != 6L) {
    stop("original coordinate/annotation identities invalid")
}
refs <- c("Microglia/Macrophage", "Oligodendrocytes (non-malignant)")
if (!identical(as.integer(table(factor(annotations[[1L]], levels=refs))), c(19L, 23L))) {
    stop("reference groups differ from upstream example")
}

subset_chrs <- c("chr1", "chr19", "chr21")
subset_ids <- rownames(counts)[rownames(counts) %in% rownames(order)[order$chr %in% subset_chrs]]
subset_counts <- counts[subset_ids, , drop=FALSE]
if (!setequal(unique(order[subset_ids, "chr"]), subset_chrs) ||
    nrow(subset_counts) < 1775L) {
    stop("subset gene-ID join or chromosome coverage invalid")
}
write_counts <- function(matrix, path) {
    con <- file(path, open="wt")
    on.exit(close(con))
    writeLines(paste(c("gene", colnames(matrix)), collapse="\t"), con)
    for (i in seq_len(nrow(matrix))) {
        writeLines(paste(c(rownames(matrix)[i], formatC(matrix[i, ], digits=17, format="g")),
                   collapse="\t"), con)
    }
}
write_counts(counts, file.path(root, inputs$canonical_full))
write_counts(subset_counts, file.path(root, inputs$canonical_subset))

metadata <- list(session="metadata/session.txt", packages="metadata/packages.tsv",
                 source="metadata/source.txt")
writeLines(capture.output(sessionInfo()), file.path(root, metadata$session))
installed <- utils::installed.packages()[, c("Package", "Version", "LibPath"), drop=FALSE]
write.table(installed[base::order(installed[, "Package"]), , drop=FALSE],
            file.path(root, metadata$packages), sep="\t", row.names=FALSE, quote=FALSE)
writeLines(c(paste0("source_commit\t", commit),
             paste0("source_archive_sha256\t", archive_sha),
             "source_url\thttps://github.com/bioconductor-source/infercnv"),
           file.path(root, metadata$source))

creation <- list(delim="\t", max_cells_per_group=NULL,
                 min_max_counts_per_cell=list(1L, "Inf"),
                 chr_exclude=c("chrX", "chrY", "chrM"),
                 ref_group_names=as.list(refs))
settings <- list(cutoff=1, min_cells_per_gene=3L, window_length=101L,
                 smooth_method="pyramidinal", max_centered_threshold=3,
                 scale_data=FALSE, HMM=FALSE, denoise=FALSE,
                 analysis_mode="samples", num_threads=1L,
                 resume_mode=FALSE, plot_steps=FALSE, no_plot=TRUE,
                 no_prelim_plot=TRUE, save_rds=TRUE,
                 remove_genes_at_chr_ends=FALSE, prune_outliers=FALSE,
                 mask_nonDE_genes=FALSE, up_to_step=14L,
                 num_ref_groups=NULL, cluster_by_groups=TRUE,
                 cluster_references=FALSE, ref_subtract_use_mean_bounds=TRUE,
                 tumor_subcluster_partition_method="leiden", BayesMaxPNormal=0.5)
steps <- c(1L, 2L, 3L, 4L, 8L, 9L, 10L, 11L, 12L, 14L)
oracle_getter <- getFromNamespace(".get_relevant_args_list", "infercnv")
cases <- list()
for (name in c("subset", "full")) {
    selected <- if (name == "subset") subset_chrs else paste0("chr", 1:22)
    case_dir <- file.path(root, name)
    upstream_dir <- file.path(case_dir, "upstream")
    dir.create(case_dir)
    input_key <- if (name == "subset") "canonical_subset" else "canonical_full"
    create_args <- list(raw_counts_matrix=file.path(root, inputs[[input_key]]),
                        gene_order_file=file.path(root, inputs$original_gene_order),
                        annotations_file=file.path(root, inputs$original_annotations),
                        ref_group_names=refs, delim=creation$delim,
                        max_cells_per_group=NULL, min_max_counts_per_cell=c(1, Inf),
                        chr_exclude=c("chrX", "chrY", "chrM"))
    obj <- do.call(infercnv::CreateInfercnvObject, create_args)
    incoming <- obj@expr.data
    source_matrix <- if (name == "subset") subset_counts else counts
    predicted_stage1 <- if (name == "subset") 1775L else 9939L
    if (ncol(incoming) != 184L || nrow(incoming) != predicted_stage1 ||
        !identical(colnames(incoming), colnames(counts)) ||
        !setequal(rownames(incoming), rownames(source_matrix)[rownames(source_matrix) %in% rownames(order)[order$chr %in% selected]]) ||
        !identical(unname(incoming), unname(source_matrix[rownames(incoming), , drop=FALSE])) ||
        !setequal(unique(as.character(obj@gene_order$chr)), selected) ||
        length(obj@reference_grouped_cell_indices[[refs[[1L]]]]) != 19L ||
        length(obj@reference_grouped_cell_indices[[refs[[2L]]]]) != 23L) {
        stop("independent incoming object check failed for ", name)
    }
    run_args <- c(list(out_dir=upstream_dir), settings)
    expected <- do.call(oracle_getter, run_args)$expected_file_names
    completed <- do.call(infercnv::run, c(list(infercnv_obj=obj), run_args))
    stages <- list()
    for (step in steps) {
        checkpoint <- expected[[step]]
        if (!nzchar(checkpoint) || !file.exists(checkpoint) ||
            dirname(normalizePath(checkpoint)) != normalizePath(upstream_dir)) {
            stop("missing or misplaced upstream checkpoint for ", name, " stage ", step)
        }
        stage_obj <- readRDS(checkpoint)
        if (ncol(stage_obj@expr.data) != 184L ||
            !identical(colnames(stage_obj@expr.data), colnames(counts))) {
            stop("checkpoint cell identity changed for ", name, " stage ", step)
        }
        prefix <- sprintf("%02d", step)
        expr_rel <- file.path(name, paste0(prefix, ".tsv"))
        gene_rel <- file.path(name, paste0(prefix, ".genes.tsv"))
        cell_rel <- file.path(name, paste0(prefix, ".cells.tsv"))
        write_expression(stage_obj, file.path(root, expr_rel))
        write_identity(stage_obj, file.path(root, gene_rel), file.path(root, cell_rel))
        stages[[as.character(step)]] <- list(expression=expr_rel, gene_order=gene_rel,
            cell_groups=cell_rel, checkpoint=file.path(name, "upstream", basename(checkpoint)),
            genes=nrow(stage_obj@expr.data), cells=ncol(stage_obj@expr.data))
    }
    stage1 <- readRDS(expected[[1L]])@expr.data
    stage2 <- readRDS(expected[[2L]])@expr.data
    retained <- rownames(stage1)[rowMeans(stage1) >= 1 & rowSums(stage1 > 0) >= 3]
    predicted_stage2 <- if (name == "subset") 1487L else 8508L
    if (!identical(rownames(stage2), retained) ||
        !identical(stage2, stage1[retained, , drop=FALSE]) ||
        nrow(stage2) != predicted_stage2 ||
        (name == "subset" && sum(as.character(readRDS(expected[[2L]])@gene_order$chr) == "chr21") != 90L)) {
        stop("stage-2 membership, values or predicted dimensions disagree for ", name)
    }
    final <- readRDS(expected[[14L]])
    if (!identical(completed@expr.data, final@expr.data) ||
        !identical(completed@gene_order, final@gene_order)) {
        stop("returned state differs from stage-14 checkpoint for ", name)
    }
    recorded <- vapply(stages, function(stage) basename(stage$checkpoint), character(1))
    extra <- setdiff(list.files(upstream_dir, recursive=TRUE), recorded)
    if (length(extra) && any(dir.exists(file.path(upstream_dir, extra)))) {
        stop("unexpected upstream directory")
    }
    cases[[name]] <- list(input_counts=input_key, chromosomes=as.list(selected),
                          reference_groups=as.list(refs), stages=stages,
                          additional_artifacts=as.list(file.path(name, "upstream", extra)))
}

all_paths <- c(unlist(inputs, use.names=FALSE), unlist(metadata, use.names=FALSE))
for (case in cases) {
    for (stage in case$stages) {
        all_paths <- c(all_paths, unlist(stage[c("expression", "gene_order", "cell_groups", "checkpoint")], use.names=FALSE))
    }
    all_paths <- c(all_paths, unlist(case$additional_artifacts, use.names=FALSE))
}
if (anyDuplicated(all_paths)) stop("duplicate oracle artifact path")
hashes <- as.list(vapply(all_paths, function(path) digest::digest(file.path(root, path),
                                algo="sha256", file=TRUE), character(1)))
names(hashes) <- all_paths
record <- list(schema_version=2L,
               package=list(name="infercnv", version="1.28.0", source_commit=commit,
                            source_archive_sha256=archive_sha),
               inputs=inputs, metadata=metadata, creation_settings=creation,
               run_settings=settings, cases=cases, sha256=hashes)
jsonlite::write_json(record, file.path(root, "oracle.json"),
                     auto_unbox=TRUE, pretty=TRUE, null="null")
