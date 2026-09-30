#!/usr/bin/env Rscript

args <- commandArgs(trailingOnly=TRUE)
if (length(args) != 2L) stop("usage: infercnv_creation_edges.R SHIPPED_BUNDLE OUTPUT_DIR")
script_file <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
if (length(script_file) != 1L) stop("cannot identify script path")
source(file.path(dirname(normalizePath(sub("^--file=", "", script_file))),
                 "infercnv_oracle_io.R"), local=TRUE)
commit <- "b421d9405c97a309b081ef86d455e976df93eae4"
source_sha <- "b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207"
if (!identical(Sys.getenv("INFER_CNV_SOURCE_COMMIT", unset=""), commit) ||
    !identical(Sys.getenv("INFER_CNV_SOURCE_SHA256", unset=""), source_sha) ||
    !identical(as.character(getRversion()), "4.6.1") ||
    !requireNamespace("infercnv", quietly=TRUE) ||
    !identical(as.character(utils::packageVersion("infercnv")), "1.28.0") ||
    !requireNamespace("jsonlite", quietly=TRUE) ||
    !requireNamespace("digest", quietly=TRUE)) {
    stop("pinned infercnv runtime or source identity unavailable")
}
hash_file <- function(path) digest::digest(path, algo="sha256", file=TRUE)
shipped <- normalizePath(args[[1L]])
record <- jsonlite::fromJSON(file.path(shipped, "oracle.json"), simplifyVector=FALSE)
if (!identical(record$package$source_commit, commit) ||
    !identical(record$package$source_archive_sha256, source_sha)) stop("source bundle identity differs")
output <- normalizePath(args[[2L]], mustWork=FALSE)
if (file.exists(output)) stop("output already exists: ", output)
dir.create(output, recursive=TRUE)
if (!dir.exists(output)) stop("cannot create output directory")
input_dir <- file.path(output, "inputs")
dir.create(input_dir)

writeLines(c("gene\tlow\tmin\tmax\thigh\tzero\tunit\textra",
             "B\t1\t1\t2\t2\t0\t1\t1", "A\t1\t1\t2\t2\t0\t0\t1",
             "C\t0\t1\t1\t2\t0\t0\t1", "D\t0\t0\t0\t0\t0\t0\t1",
             "X\t1000000\t1000000\t1000000\t1000000\t1000000\t1000000\t1000000",
             "ONLY_COUNTS\t1000000\t1000000\t1000000\t1000000\t1000000\t1000000\t1000000",
             "ZERO_POS\t1000000\t1000000\t1000000\t1000000\t1000000\t1000000\t1000000"),
           file.path(input_dir, "counts.tsv"))
writeLines(c("POS_ONLY\tchr10\t1\t2", "A\tchr2\t10\t20", "B\tchr2\t10\t20",
             "C\tchr10\t30\t40", "D\tchr1\t1\t2", "X\tchrX\t10\t20",
             "ZERO_POS\tchr3\t0\t0"), file.path(input_dir, "positions.tsv"))
annotations <- c("high\tz_obs", "max\ta_obs", "min\tref", "low\ta_obs",
                 "zero\tz_obs", "unit\tz_obs")
writeLines(annotations, file.path(input_dir, "annotations.tsv"))
writeLines(c(annotations, "absent\tz_obs"), file.path(input_dir, "absent.tsv"))
writeLines(sub("low\ta_obs", "low\tlow_ref", annotations), file.path(input_dir, "removed.tsv"))
writeLines("UNMATCHED\tchr1\t1\t2", file.path(input_dir, "no_common.tsv"))

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

create_case <- function(label, counts, positions, annotations_path, refs, limits,
                        expect_error=FALSE, expected=NULL) {
    directory <- file.path(output, label)
    dir.create(directory)
    options <- list(raw_counts_matrix=normalizePath(counts),
                    gene_order_file=normalizePath(positions),
                    annotations_file=normalizePath(annotations_path),
                    ref_group_names=refs, delim="\t", max_cells_per_group=NULL,
                    min_max_counts_per_cell=limits, chr_exclude=c("chrX", "chrY", "chrM"))
    warnings <- character()
    execute <- function() withCallingHandlers(
        do.call(infercnv::CreateInfercnvObject, options), warning=function(condition) {
            warnings <<- c(warnings, conditionMessage(condition))
            invokeRestart("muffleWarning")
        })
    failure <- NULL
    failure_class <- failure_call <- NULL
    if (expect_error) {
        obj <- tryCatch(execute(), error=function(condition) {
            failure <<- conditionMessage(condition)
            failure_class <<- as.list(class(condition))
            failure_call <<- paste(deparse(conditionCall(condition)), collapse=" ")
            NULL
        })
        if (is.null(failure) || !nzchar(failure)) stop("expected error was not raised: ", label)
    } else {
        obj <- execute()
    }
    declared <- options
    declared$ref_group_names <- as.list(refs)
    declared$chr_exclude <- as.list(options$chr_exclude)
    declared$min_max_counts_per_cell <- if (is.null(limits)) NULL else
        lapply(limits, function(value) if (is.infinite(value)) "Inf" else value)
    paths <- options[c("raw_counts_matrix", "gene_order_file", "annotations_file")]
    result <- list(creation_arguments=declared, warnings=as.list(warnings),
                   input_sha256=lapply(paths, hash_file))
    if (expect_error) {
        result$error <- failure
        result$error_class <- failure_class
        result$error_call <- failure_call
        return(result)
    }
    if (!is.null(expected)) {
        for (field in c("expr.data", "count.data", "gene_order",
                        "reference_grouped_cell_indices", "observation_grouped_cell_indices")) {
            if (!identical(methods::slot(obj, field), methods::slot(expected, field))) {
                stop("original shipped creation differs from stage 1: ", field)
            }
        }
    }
    write_expression(obj, file.path(directory, "expression.tsv"))
    write_identity(obj, file.path(directory, "genes.tsv"), file.path(directory, "cells.tsv"))
    write_maps(obj, file.path(directory, "maps.tsv"))
    files <- c("expression.tsv", "genes.tsv", "cells.tsv", "maps.tsv")
    result$outputs <- stats::setNames(as.list(vapply(file.path(directory, files),
                                                   hash_file, character(1))), files)
    result
}

full <- record$cases$full
stage <- full$stages[["1"]]
checkpoint <- file.path(shipped, stage$checkpoint)
cases <- list(shipped_raw=create_case(
    "shipped_raw", file.path(shipped, record$inputs$original_counts),
    file.path(shipped, record$inputs$original_gene_order),
    file.path(shipped, record$inputs$original_annotations),
    unlist(full$reference_groups, use.names=FALSE), c(1, Inf), expected=readRDS(checkpoint)))
cases$shipped_raw$stage1_sha256 <- hash_file(checkpoint)
cases$shipped_raw$source_checkpoint <- stage$checkpoint
counts <- file.path(input_dir, "counts.tsv")
positions <- file.path(input_dir, "positions.tsv")
annotations_path <- file.path(input_dir, "annotations.tsv")
cases$integer_limits <- create_case("integer_limits", counts, positions, annotations_path, "ref", c(3, 5))
cases$min_zero <- create_case("min_zero", counts, positions, annotations_path, "ref", c(0, Inf))
cases$min_null <- create_case("min_null", counts, positions, annotations_path, "ref", NULL)
cases$absent_annotation <- create_case("absent_annotation", counts, positions,
                                       file.path(input_dir, "absent.tsv"), "ref", c(3, 5), TRUE)
cases$removed_reference <- create_case("removed_reference", counts, positions,
                                       file.path(input_dir, "removed.tsv"), "low_ref", c(3, 5), TRUE)
cases$no_common_genes <- create_case("no_common_genes", counts,
                                     file.path(input_dir, "no_common.tsv"), annotations_path,
                                     "ref", c(3, 5), TRUE)
writeLines(capture.output(sessionInfo()), file.path(output, "session.txt"))
receipt <- list(schema_version=1L, package=record$package,
                runtime=list(R_version=as.character(getRversion()), locale=Sys.getlocale(),
                             longdouble_digits=.Machine$longdouble.digits,
                             sizeof_longdouble=.Machine$sizeof.longdouble),
                session_sha256=hash_file(file.path(output, "session.txt")), cases=cases)
jsonlite::write_json(receipt, file.path(output, "edges.json"), auto_unbox=TRUE,
                     pretty=TRUE, null="null")
