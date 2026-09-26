matched_args <- function() {
    list(cutoff=1, min_cells_per_gene=3, window_length=101,
         smooth_method="pyramidinal", max_centered_threshold=3,
         num_ref_groups=NULL, ref_subtract_use_mean_bounds=TRUE,
         cluster_by_groups=TRUE, cluster_references=FALSE,
         analysis_mode="samples", tumor_subcluster_partition_method="leiden",
         HMM=FALSE, BayesMaxPNormal=0.5,
         scale_data=FALSE, denoise=FALSE, remove_genes_at_chr_ends=FALSE,
         prune_outliers=FALSE, mask_nonDE_genes=FALSE,
         num_threads=1, debug=FALSE, resume_mode=FALSE,
         plot_steps=FALSE, inspect_subclusters=FALSE,
         no_plot=TRUE, no_prelim_plot=TRUE,
         save_rds=FALSE, save_final_rds=FALSE,
         write_expr_matrix=FALSE, write_phylo=FALSE, diagnostics=FALSE,
         up_to_step=14)
}

validate_run_formals <- function() {
    keys <- c("infercnv_obj", "out_dir", names(matched_args()))
    if (anyDuplicated(keys) || !all(keys %in% names(formals(infercnv::run)))) {
        stop("fixed run arguments differ from installed infercnv::run formals")
    }
    invisible(TRUE)
}

owned_bundle_file <- function(root, rel) {
    if (!is.character(rel) || length(rel) != 1L || is.na(rel) ||
        !nzchar(rel) || grepl("\\\\", rel) || grepl("^/", rel) ||
        any(strsplit(rel, "/", fixed=TRUE)[[1L]] %in% c("", ".", ".."))) {
        stop("unsafe bundle path")
    }
    root <- normalizePath(root, mustWork=TRUE)
    path <- normalizePath(file.path(root, rel), mustWork=TRUE)
    if (!startsWith(path, paste0(root, .Platform$file.sep)) || dir.exists(path)) {
        stop("bundle path escapes root or is not a file")
    }
    path
}

sha256_file <- function(path) digest::digest(path, algo="sha256", file=TRUE)

load_prepared_record <- function(bundle_root, script_dir) {
    receipt_path <- file.path(dirname(script_dir), ".autopilot", "oracles",
                              "infercnv-measurement-input-2026-09-26.json")
    if (!identical(sha256_file(receipt_path),
                   "33d598ba61c998a03189cbb2794a8b4655478b798141b2919f30244559a2e034")) {
        stop("trusted measurement receipt bytes differ")
    }
    receipt <- jsonlite::fromJSON(receipt_path, simplifyVector=FALSE)
    if (!identical(receipt$accepted_oracle_receipt_sha256,
                   "be0342e7c3f701e98b2f4db5199230de542a6f9bbfb6ea3379c1958068094476") ||
        !identical(receipt$source_commit, "b421d9405c97a309b081ef86d455e976df93eae4") ||
        !identical(receipt$case, "full") || !identical(receipt$incoming_genes, 9939L) ||
        !identical(receipt$retained_genes, 8508L) || !identical(receipt$cells, 184L)) {
        stop("measurement receipt identity mismatch")
    }
    required <- c("full/01.tsv", "full/01.genes.tsv", "full/01.cells.tsv",
                  "full/14.tsv", "full/14.genes.tsv", "full/14.cells.tsv")
    if (!identical(sort(names(receipt$files)), sort(required))) stop("receipt file set mismatch")
    oracle <- jsonlite::fromJSON(owned_bundle_file(bundle_root, "oracle.json"), simplifyVector=FALSE)
    if (!identical(oracle$package$source_commit, receipt$source_commit)) stop("fresh source mismatch")
    stages <- oracle$cases$full$stages
    for (step in c("1", "14")) {
        stage <- stages[[step]]
        prefix <- sprintf("full/%02d", as.integer(step))
        paths <- c(stage$expression, stage$gene_order, stage$cell_groups)
        if (!identical(paths, paste0(prefix, c(".tsv", ".genes.tsv", ".cells.tsv"))) ||
            !identical(stage$genes, if (step == "1") 9939L else 8508L) ||
            !identical(stage$cells, 184L)) stop("fresh stage path or dimensions mismatch")
    }
    for (rel in required) {
        if (!identical(oracle$sha256[[rel]], receipt$files[[rel]]) ||
            !identical(sha256_file(owned_bundle_file(bundle_root, rel)), receipt$files[[rel]])) {
            stop("fresh accepted TSV pin mismatch: ", rel)
        }
    }
    checkpoint <- stages[["1"]]$checkpoint
    if (!identical(checkpoint, "full/upstream/01_incoming_data.infercnv_obj") ||
        !identical(sha256_file(owned_bundle_file(bundle_root, checkpoint)),
                   oracle$sha256[[checkpoint]])) stop("fresh stage-1 checkpoint mismatch")
    list(receipt=receipt, checkpoint=checkpoint,
         checkpoint_sha256=oracle$sha256[[checkpoint]])
}

prepared_identity <- function(obj) {
    list(expr=obj@expr.data, counts=obj@count.data, genes=obj@gene_order,
         references=obj@reference_grouped_cell_indices,
         observations=obj@observation_grouped_cell_indices, spike=obj@.hspike)
}

read_pinned_matrix <- function(path) {
    data <- read.table(path, header=TRUE, sep="\t", row.names=1L,
                       check.names=FALSE, quote="", comment.char="")
    result <- as.matrix(data)
    storage.mode(result) <- "double"
    result
}

validate_group_map <- function(actual, expected_names, cells, groups, role, expected_role) {
    if (!is.list(actual) || !identical(names(actual), expected_names)) stop("group list order mismatch")
    covered <- integer()
    for (name in expected_names) {
        indices <- actual[[name]]
        wanted <- which(groups == name & role == expected_role)
        if (!is.numeric(indices) || anyNA(indices) || any(!is.finite(indices)) ||
            any(indices != trunc(indices)) || !identical(as.integer(indices), wanted) ||
            any(indices < 1L | indices > length(cells))) stop("group index mismatch: ", name)
        covered <- c(covered, as.integer(indices))
    }
    covered
}

validate_prepared <- function(obj, bundle_root, record) {
    if (!methods::is(obj, "infercnv")) stop("not an infercnv object")
    if (!is.null(obj@.hspike)) stop("hidden spike is present")
    if (!identical(record$checkpoint, "full/upstream/01_incoming_data.infercnv_obj") ||
        !identical(sha256_file(owned_bundle_file(bundle_root, record$checkpoint)),
                   record$checkpoint_sha256)) stop("checkpoint provenance mismatch")
    receipt <- record$receipt
    for (rel in names(receipt$files)) {
        if (!identical(sha256_file(owned_bundle_file(bundle_root, rel)), receipt$files[[rel]])) {
            stop("accepted TSV pin mismatch: ", rel)
        }
    }
    expected <- read_pinned_matrix(owned_bundle_file(bundle_root, "full/01.tsv"))
    if (!identical(dim(expected), c(9939L, 184L)) ||
        anyNA(expected) || any(!is.finite(expected)) || any(expected < 0)) stop("pinned stage 1 invalid")
    for (slot in list(obj@expr.data, obj@count.data)) {
        if (!is.matrix(slot) || !is.numeric(slot) || !identical(dim(slot), dim(expected)) ||
            !identical(dimnames(slot), dimnames(expected)) || anyNA(slot) ||
            any(!is.finite(slot)) || any(slot < 0) ||
            !identical(unname(slot), unname(expected))) stop("prepared count slot mismatch")
    }
    genes <- read.table(owned_bundle_file(bundle_root, "full/01.genes.tsv"),
                        header=TRUE, sep="\t", check.names=FALSE, quote="", comment.char="")
    order <- obj@gene_order
    if (!identical(names(genes), c("gene", "chr", "start", "stop")) ||
        !identical(rownames(order), as.character(genes$gene)) ||
        !identical(rownames(order), rownames(expected)) ||
        !identical(as.character(order$chr), as.character(genes$chr)) ||
        !identical(as.numeric(order$start), as.numeric(genes$start)) ||
        !identical(as.numeric(order$stop), as.numeric(genes$stop))) stop("prepared gene order mismatch")
    cells <- read.table(owned_bundle_file(bundle_root, "full/01.cells.tsv"),
                        header=TRUE, sep="\t", check.names=FALSE, quote="", comment.char="")
    if (!identical(names(cells), c("cell", "group", "role")) ||
        !identical(as.character(cells$cell), colnames(expected))) stop("pinned cell identity mismatch")
    groups <- as.character(cells$group)
    roles <- as.character(cells$role)
    refs <- c("Microglia/Macrophage", "Oligodendrocytes (non-malignant)")
    obs <- c("malignant_93", "malignant_97", "malignant_MGH36", "malignant_MGH53")
    if (anyNA(groups) || anyNA(roles) || !setequal(unique(groups), c(refs, obs)) ||
        any(!(roles %in% c("reference", "observation")))) stop("pinned group roles invalid")
    covered <- c(validate_group_map(obj@reference_grouped_cell_indices, refs,
                                    cells$cell, groups, roles, "reference"),
                 validate_group_map(obj@observation_grouped_cell_indices, obs,
                                    cells$cell, groups, roles, "observation"))
    if (length(covered) != 184L || !identical(sort(covered), seq_len(184L))) {
        stop("incomplete or overlapping group coverage")
    }
    invisible(TRUE)
}

run_prepared <- function(obj, out_dir) {
    if (!dir.exists(out_dir)) stop("pre-created run output directory required")
    do.call(infercnv::run, c(list(infercnv_obj=obj, out_dir=out_dir), matched_args()))
}

parse_rss_bytes <- function(text) {
    lines <- strsplit(text, "\n", fixed=TRUE)[[1L]]
    matches <- grep("^Rss:[[:space:]]+[0-9]+[[:space:]]+kB$", lines, value=TRUE)
    if (length(matches) != 1L || length(grep("^Rss:", lines)) != 1L) stop("invalid RSS field")
    kib <- as.numeric(sub("^Rss:[[:space:]]+([0-9]+)[[:space:]]+kB$", "\\1", matches))
    if (!is.finite(kib) || kib <= 0 || kib > (2^53 - 1) / 1024) stop("RSS overflow or zero")
    kib * 1024
}

checked_delta <- function(after, before) {
    if (length(after) != 1L || length(before) != 1L || !is.numeric(after) ||
        !is.numeric(before) || !is.finite(after) || !is.finite(before) ||
        after < before || before < 0) stop("invalid clock delta")
    after - before
}
