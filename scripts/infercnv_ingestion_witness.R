#!/usr/bin/env Rscript

args <- commandArgs(trailingOnly=TRUE)
if (length(args) < 2L || length(args) > 3L) {
    stop("usage: infercnv_ingestion_witness.R SYNTHETIC_BUNDLE OUTPUT_DIR [SHIPPED_BUNDLE]")
}
script_file <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
if (length(script_file) != 1L) stop("cannot identify script path")
source(file.path(dirname(normalizePath(sub("^--file=", "", script_file))),
                 "infercnv_oracle_io.R"), local=TRUE)

commit <- "b421d9405c97a309b081ef86d455e976df93eae4"
source_sha <- Sys.getenv("INFER_CNV_SOURCE_SHA256", unset="")
if (!identical(Sys.getenv("INFER_CNV_SOURCE_COMMIT", unset=""), commit) ||
    !grepl("^[0-9a-f]{64}$", source_sha) ||
    !identical(as.character(getRversion()), "4.6.1") ||
    !requireNamespace("infercnv", quietly=TRUE) ||
    !identical(as.character(utils::packageVersion("infercnv")), "1.28.0") ||
    !requireNamespace("jsonlite", quietly=TRUE) ||
    !requireNamespace("digest", quietly=TRUE)) {
    stop("pinned infercnv runtime or source identity unavailable")
}

output <- normalizePath(args[[2L]], mustWork=FALSE)
if (file.exists(output)) stop("output already exists: ", output)
dir.create(output, recursive=TRUE)
if (!dir.exists(output)) stop("cannot create output directory")
hash_file <- function(path) digest::digest(path, algo="sha256", file=TRUE)

gzip_copy <- function(input, output_path) {
    size <- file.info(input)$size
    if (is.na(size) || size > .Machine$integer.max) stop("unsupported count input size")
    input_con <- file(input, "rb")
    bytes <- readBin(input_con, what="raw", n=as.integer(size))
    close(input_con)
    if (length(bytes) != size) stop("count input truncated")
    output_con <- gzfile(output_path, "wb")
    writeBin(bytes, output_con)
    close(output_con)
    check_con <- gzfile(output_path, "rb")
    decoded <- readBin(check_con, what="raw", n=length(bytes) + 1L)
    close(check_con)
    if (!identical(bytes, decoded)) stop("gzip count bytes changed")
}

same_state <- function(actual, expected) {
    fields <- c("expr.data", "count.data", "gene_order",
                "reference_grouped_cell_indices", "observation_grouped_cell_indices")
    for (field in fields) {
        if (!identical(methods::slot(actual, field), methods::slot(expected, field))) {
            stop("creation state differs from accepted stage 1: ", field)
        }
    }
}

write_maps <- function(obj, path) {
    lines <- "role\tgroup\tindex\tcell"
    cells <- colnames(obj@expr.data)
    for (role in c("reference", "observation")) {
        maps <- if (role == "reference") obj@reference_grouped_cell_indices else
            obj@observation_grouped_cell_indices
        for (name in names(maps)) {
            for (index in maps[[name]]) {
                if (index < 1L || index > length(cells)) stop("invalid grouped index")
                lines <- c(lines, paste(role, name, index, cells[[index]], sep="\t"))
            }
        }
    }
    writeLines(lines, path)
}

export_state <- function(obj, directory) {
    dir.create(directory)
    write_expression(obj, file.path(directory, "expression.tsv"))
    write_identity(obj, file.path(directory, "genes.tsv"),
                   file.path(directory, "cells.tsv"))
    write_maps(obj, file.path(directory, "maps.tsv"))
    files <- c("expression.tsv", "genes.tsv", "cells.tsv", "maps.tsv")
    stats::setNames(as.list(vapply(file.path(directory, files), hash_file, character(1))), files)
}

create_case <- function(label, counts, positions, annotations, refs, expected=NULL) {
    case_dir <- file.path(output, label)
    dir.create(case_dir, recursive=TRUE)
    if (!dir.exists(case_dir)) stop("cannot create case directory")
    gzip <- file.path(case_dir, "counts.tsv.gz")
    gzip_copy(counts, gzip)
    options <- list(gene_order_file=positions, annotations_file=annotations,
                    ref_group_names=refs, delim="\t", max_cells_per_group=NULL,
                    min_max_counts_per_cell=c(1, Inf),
                    chr_exclude=c("chrX", "chrY", "chrM"))
    plain <- do.call(infercnv::CreateInfercnvObject,
                     c(list(raw_counts_matrix=counts), options))
    compressed <- do.call(infercnv::CreateInfercnvObject,
                          c(list(raw_counts_matrix=gzip), options))
    same_state(compressed, plain)
    if (!is.null(expected)) same_state(plain, expected)
    plain_hashes <- export_state(plain, file.path(case_dir, "plain"))
    gzip_hashes <- export_state(compressed, file.path(case_dir, "gzip"))
    if (!identical(plain_hashes, gzip_hashes)) stop("plain/gzip exports differ")
    list(counts_sha256=hash_file(counts), gzip_sha256=hash_file(gzip),
         positions_sha256=hash_file(positions), annotations_sha256=hash_file(annotations),
         genes=nrow(plain@expr.data), cells=ncol(plain@expr.data),
         references=as.list(refs), plain=plain_hashes, gzip=gzip_hashes,
         creation_arguments=list(raw_counts_matrix=normalizePath(counts),
             gene_order_file=normalizePath(positions),
             annotations_file=normalizePath(annotations), ref_group_names=as.list(refs),
             delim="\t", max_cells_per_group=NULL,
             min_max_counts_per_cell=list(1L, "Inf"),
             chr_exclude=as.list(c("chrX", "chrY", "chrM"))),
         reference_maps=plain@reference_grouped_cell_indices,
         observation_maps=plain@observation_grouped_cell_indices)
}

synthetic <- normalizePath(args[[1L]])
synthetic_record <- jsonlite::fromJSON(file.path(synthetic, "oracle.json"), simplifyVector=FALSE)
if (!identical(synthetic_record$package$source_commit, commit) ||
    !identical(synthetic_record$package$source_archive_sha256, source_sha)) {
    stop("synthetic bundle source identity differs")
}
cases <- list()
for (profile in c("grouped_bounds", "no_reference")) {
    entry <- synthetic_record$profiles[[profile]]
    source_stage <- entry$stages[["1"]]
    expected <- readRDS(file.path(synthetic, source_stage$checkpoint))
    refs <- unlist(entry$settings$ref_group_names, use.names=FALSE)
    cases[[paste0("synthetic_", profile)]] <- create_case(
        paste0("synthetic_", profile),
        file.path(synthetic, synthetic_record$inputs[["counts.tsv"]]),
        file.path(synthetic, synthetic_record$inputs[["gene_order.tsv"]]),
        file.path(synthetic, synthetic_record$inputs[["annotations.tsv"]]),
        if (length(refs)) refs else NULL, expected)
    cases[[paste0("synthetic_", profile)]]$stage1_sha256 <- hash_file(
        file.path(synthetic, source_stage$checkpoint))
    cases[[paste0("synthetic_", profile)]]$source_checkpoint <- source_stage$checkpoint
}

if (length(args) == 3L) {
    shipped <- normalizePath(args[[3L]])
    shipped_record <- jsonlite::fromJSON(file.path(shipped, "oracle.json"), simplifyVector=FALSE)
    if (!identical(shipped_record$package$source_commit, commit) ||
        !identical(shipped_record$package$source_archive_sha256, source_sha)) {
        stop("shipped bundle source identity differs")
    }
    full <- shipped_record$cases$full
    stage1 <- full$stages[["1"]]
    counts <- file.path(shipped, shipped_record$inputs[[full$input_counts]])
    refs <- unlist(full$reference_groups, use.names=FALSE)
    cases$shipped_full <- create_case(
        "shipped_full", counts,
        file.path(shipped, shipped_record$inputs$original_gene_order),
        file.path(shipped, shipped_record$inputs$original_annotations),
        refs, readRDS(file.path(shipped, stage1$checkpoint)))
    cases$shipped_full$stage1_sha256 <- hash_file(file.path(shipped, stage1$checkpoint))
    cases$shipped_full$source_checkpoint <- stage1$checkpoint
}

small <- file.path(output, "small_inputs")
dir.create(small)
writeLines(c(
    "gene\textra\tt_b\tref2\tobs_a\tref1",
    "G_C\t2\t2\t2\t2\t2",
    "G_X\t3\t3\t3\t3\t3",
    "G_B\t2\t0.076439\t2\t2\t2",
    "G_A\t2\t2\t2\t2\t2",
    "G_D\t2\t7.6439e-2\t2\t2\t2",
    "ONLY_COUNTS\t20\t20\t20\t20\t20"
), file.path(small, "counts.tsv"))
writeLines(c("POS_ONLY\tchr10\t1\t2", "G_B\tchr2\t20\t21",
             "G_A\tchr10\t30\t31", "G_X\tchrX\t10\t11",
             "G_C\tchr1\t5\t6", "G_D\tchr2\t10\t11"),
           file.path(small, "positions.tsv"))
writeLines(c("obs_a\ta_obs", "ref1\tz_ref", "t_b\tz_obs", "ref2\ta_ref"),
           file.path(small, "annotations.tsv"))
small_obj <- infercnv::CreateInfercnvObject(
    raw_counts_matrix=file.path(small, "counts.tsv"),
    gene_order_file=file.path(small, "positions.tsv"),
    annotations_file=file.path(small, "annotations.tsv"),
    ref_group_names=c("z_ref", "a_ref"), min_max_counts_per_cell=c(1, Inf))
if (!identical(rownames(small_obj@expr.data), c("G_A", "G_D", "G_B", "G_C")) ||
    !identical(colnames(small_obj@expr.data), c("t_b", "ref2", "obs_a", "ref1")) ||
    !identical(names(small_obj@reference_grouped_cell_indices), c("z_ref", "a_ref")) ||
    !identical(names(small_obj@observation_grouped_cell_indices), c("a_obs", "z_obs")) ||
    !identical(sprintf("%a", small_obj@expr.data["G_B", "t_b"]),
               "0x1.391819d2391d6p-4") ||
    !identical(sprintf("%a", small_obj@expr.data["G_D", "t_b"]),
               "0x1.391819d2391d6p-4")) {
    stop("small creation witness differs from pinned expectation")
}
cases$small <- create_case("small", file.path(small, "counts.tsv"),
                           file.path(small, "positions.tsv"),
                           file.path(small, "annotations.tsv"), c("z_ref", "a_ref"),
                           small_obj)
cases$small$decimal <- list(literal="0.076439",
                            numeric17=sprintf("%.17g", small_obj@expr.data["G_B", "t_b"]),
                            hex=sprintf("%a", small_obj@expr.data["G_B", "t_b"]))
writeLines(capture.output(sessionInfo()), file.path(output, "session.txt"))
record <- list(schema_version=1L,
               package=list(name="infercnv", version="1.28.0",
                            source_commit=commit, source_archive_sha256=source_sha),
               runtime=list(R_version=as.character(getRversion()), locale=Sys.getlocale()),
               cases=cases, session_sha256=hash_file(file.path(output, "session.txt")))
jsonlite::write_json(record, file.path(output, "witness.json"),
                     auto_unbox=TRUE, pretty=TRUE, null="null")
