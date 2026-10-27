#!/usr/bin/env Rscript

deseq_hash <- function(path) digest::digest(path, algo="sha256", file=TRUE)

deseq_write_json <- function(value, path) {
    jsonlite::write_json(value, path, auto_unbox=TRUE, pretty=TRUE,
                         null="null", na="null", digits=17)
}

deseq_write_table <- function(value, path, ids=NULL, id_name="gene") {
    frame <- as.data.frame(value, check.names=FALSE, stringsAsFactors=FALSE)
    simple <- vapply(frame, function(column) is.atomic(column) && is.null(dim(column)), logical(1))
    omitted <- names(frame)[!simple]
    frame <- frame[, simple, drop=FALSE]
    for (name in names(frame)) {
        if (is.numeric(frame[[name]])) {
            frame[[name]] <- formatC(frame[[name]], digits=17, format="g", decimal.mark=".")
        } else {
            frame[[name]] <- as.character(frame[[name]])
        }
    }
    if (!is.null(ids)) {
        if (length(ids) != nrow(frame)) stop("table identities differ from its rows")
        frame <- cbind(stats::setNames(data.frame(as.character(ids)), id_name), frame)
    }
    write.table(frame, path, sep="\t", quote=TRUE, row.names=FALSE, na="NA", fileEncoding="UTF-8")
    unname(omitted)
}

deseq_definition <- function(path, name) {
    for (expression in parse(path, keep.source=FALSE)) {
        if (is.call(expression) && length(expression) == 3L && is.symbol(expression[[1L]]) &&
            as.character(expression[[1L]]) %in% c("<-", "=") &&
            is.symbol(expression[[2L]]) && identical(as.character(expression[[2L]]), name) &&
            is.call(expression[[3L]]) && identical(expression[[3L]][[1L]], as.name("function"))) {
            return(expression[[3L]])
        }
    }
    stop("missing pinned function definition: ", name)
}

deseq_source_identity <- function(source, output) {
    files <- c(
        "R/AllClasses.R"="63c4d06f34b27dd56898fb67b827686f8087382787a0a142e9e38e01d2110e8a",
        "R/core.R"="1aa7f02de40d176c6930e0990697f1db59ab5b49b8a4bd730bc9de8660042aec",
        "R/results.R"="bf23751da82f2e1483d7a9b66c8464608b4fcd9494d64bda78fb778be305ea47",
        "R/vst.R"="859e61a270307fdc6c634f15a6412c20aa606284fad344d451cebd7382bc833f",
        "R/methods.R"="dd8320dc0ea8634ee91748bb5a10d4e466fd7aa0c4e31001ac053481b60798e8",
        "R/fitNbinomGLMs.R"="c23e6e61799b4b036804f323995d75104804694d3fc91965d6cd85476d9173f1",
        "src/DESeq2.cpp"="5ab1b09544566649a5c0909b16900f3f09d61d93aea000eee18c51bfb2dad895")
    for (name in names(files)) {
        if (!identical(deseq_hash(file.path(source, name)), unname(files[[name]]))) {
            stop("source file differs from release: ", name)
        }
    }
    description <- read.dcf(file.path(source, "DESCRIPTION"))
    if (description[1L, "Version"] != "1.52.0" || description[1L, "git_branch"] != "RELEASE_3_23" ||
        description[1L, "git_last_commit"] != "16aeab6") stop("source package metadata differs")
    functions <- c(DESeqDataSetFromMatrix="R/AllClasses.R", DESeq="R/core.R",
        results="R/results.R", varianceStabilizingTransformation="R/vst.R",
        counts.DESeqDataSet="R/methods.R", estimateSizeFactors.DESeqDataSet="R/methods.R",
        estimateDispersions.DESeqDataSet="R/methods.R", estimateDispersionsGeneEst="R/core.R",
        estimateDispersionsFit="R/core.R", estimateDispersionsMAP="R/core.R",
        nbinomWaldTest="R/core.R", getContrast="R/results.R", pvalueAdjustment="R/results.R",
        getVarianceStabilizedData="R/vst.R", fitNbinomGLMs="R/fitNbinomGLMs.R",
        fitNbinomGLMsOptim="R/fitNbinomGLMs.R", recordMaxCooks="R/core.R",
        replaceOutliers="R/core.R", refitWithoutOutliers="R/core.R")
    namespace <- asNamespace("DESeq2")
    for (name in names(functions)) {
        expected <- deseq_definition(file.path(source, functions[[name]]), name)
        installed <- get(name, namespace, inherits=FALSE)
        if (!identical(deparse(formals(installed)), deparse(expected[[2L]])) ||
            !identical(deparse(body(installed)), deparse(expected[[3L]]))) {
            stop("installed function differs from release source: ", name)
        }
        writeLines(c("formals:", deparse(formals(installed)), "body:", deparse(body(installed))),
                   file.path(output, paste0("function-", name, ".txt")))
    }
    writeLines(deparse(methods::getMethod("counts", "DESeqDataSet", where=namespace)),
               file.path(output, "method-counts-DESeqDataSet.txt"))
    list(source_files=as.list(files), checked_functions=as.list(functions))
}

deseq_fit_state <- function(object) {
    trend <- DESeq2::dispersionFunction(object)
    list(counts=DESeq2::counts(object), coldata=as.data.frame(SummarizedExperiment::colData(object)),
         design=paste(deparse(DESeq2::design(object)), collapse=" "),
         model_matrix=attr(object, "modelMatrix"), dispersion_model_matrix=attr(object, "dispModelMatrix"),
         size_factors=DESeq2::sizeFactors(object), normalization_factors=DESeq2::normalizationFactors(object),
         row_data=as.data.frame(S4Vectors::mcols(object)),
         assays=lapply(SummarizedExperiment::assays(object), as.matrix),
         dispersion_attributes=attributes(trend), dispersion_body=deparse(body(trend)))
}

deseq_workflow_main <- function() {
    args <- commandArgs(trailingOnly=TRUE)
    if (length(args) != 2L) stop("usage: deseq_workflow_oracle.R SOURCE_DIR OUTPUT_DIR")
    for (package in c("DESeq2", "digest", "jsonlite", "BiocManager")) {
        if (!requireNamespace(package, quietly=TRUE)) stop("required installed package absent: ", package)
    }
    pin <- list(name="DESeq2", version="1.52.0", bioconductor="3.23",
                source_commit="16aeab6d6158bd8cbb8e98764c2f33399b1a3fd3",
                source_archive_sha256="8c91699286336350e66eec132ce6fdf5bb4af78e2a4d015a5a61224f62a95984")
    if (as.character(getRversion()) != "4.6.1" ||
        as.character(utils::packageVersion("DESeq2")) != pin$version ||
        as.character(BiocManager::version()) != pin$bioconductor) stop("installed runtime/package version differs")
    archive <- Sys.getenv("DESEQ2_SOURCE_ARCHIVE")
    if (!nzchar(archive) || !identical(deseq_hash(archive), pin$source_archive_sha256) ||
        !identical(Sys.getenv("DESEQ2_SOURCE_SHA256"), pin$source_archive_sha256) ||
        !identical(Sys.getenv("DESEQ2_SOURCE_COMMIT"), pin$source_commit)) stop("source acquisition identity differs")
    source <- normalizePath(args[[1L]], mustWork=TRUE)
    root <- normalizePath(args[[2L]], mustWork=FALSE)
    if (file.exists(root) || dir.exists(root)) stop("output directory already exists")
    if (!dir.create(root, recursive=TRUE)) stop("cannot create output directory")
    directories <- c("metadata", "inputs", "fit", "results", "reload")
    for (name in directories) if (!dir.create(file.path(root, name))) stop("cannot create output subdirectory")
    options(digits=17, scipen=0, warn=1)
    if (!identical(Sys.setlocale("LC_COLLATE", "C"), "C") ||
        !identical(Sys.setlocale("LC_NUMERIC", "C"), "C")) {
        stop("cannot establish source comparison locale")
    }
    Sys.setenv(OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", MKL_NUM_THREADS="1")
    metadata <- file.path(root, "metadata")
    source_identity <- deseq_source_identity(source, metadata)
    script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
    if (length(script) != 1L || !file.copy(sub("^--file=", "", script),
        file.path(metadata, "deseq_workflow_oracle.R"))) stop("cannot preserve driver")
    if (!file.copy(file.path(source, "DESCRIPTION"), file.path(metadata, "source-DESCRIPTION"))) {
        stop("cannot preserve source metadata")
    }
    writeLines(capture.output(sessionInfo()), file.path(metadata, "session.txt"))
    packages <- utils::installed.packages()[, c("Package", "Version", "LibPath"), drop=FALSE]
    write.table(packages[order(packages[, "Package"], method="radix"), , drop=FALSE],
                file.path(metadata, "packages.tsv"), sep="\t", row.names=FALSE, quote=TRUE)
    installed_root <- system.file(package="DESeq2", mustWork=TRUE)
    installed_paths <- sort(list.files(installed_root, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
    installed_hashes <- stats::setNames(vapply(file.path(installed_root, installed_paths), deseq_hash, character(1)), installed_paths)
    write.table(data.frame(sha256=unname(installed_hashes), path=names(installed_hashes)),
                file.path(metadata, "installed-DESeq2-sha256.tsv"), sep="\t", row.names=FALSE, quote=FALSE)
    dll <- getLoadedDLLs()[["DESeq2"]][["path"]]
    runtime <- list(R=as.character(getRversion()), Bioconductor=as.character(BiocManager::version()),
        DESeq2=as.character(utils::packageVersion("DESeq2")), platform=R.version$platform,
        system=as.list(Sys.info()), external_software=as.list(extSoftVersion()),
        BLAS_LAPACK=capture.output(La_library()), locale=Sys.getlocale(), installed_path=installed_root,
        installed_dll=dll, installed_dll_sha256=deseq_hash(dll),
        thread_environment=as.list(Sys.getenv(c("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"))))
    deseq_write_json(runtime, file.path(metadata, "runtime.json"))
    conditions <- list()
    observe <- function(label, expression) {
        note <- function(condition, kind) {
            row <- data.frame(index=length(conditions) + 1L, operation=label, kind=kind,
                class=paste(class(condition), collapse=","), message=conditionMessage(condition),
                call=paste(deparse(conditionCall(condition)), collapse=" "))
            conditions[[length(conditions) + 1L]] <<- row
            write.table(row, file.path(metadata, "conditions.tsv"), sep="\t", quote=TRUE,
                        row.names=FALSE, col.names=length(conditions) == 1L, append=length(conditions) > 1L)
        }
        withCallingHandlers(force(expression), warning=function(value) note(value, "warning"),
                            message=function(value) note(value, "message"))
    }
    input_pins <- c(pasilla_gene_counts.tsv.gz="4b5570008936786da365d110df1c83c8cffc646d6c667d6b15ecbfb3d9ee735d",
                    pasilla_sample_annotation.csv="1844f871ca5d750f41685c73b87bd7dda291ec0d52304c1afb46a17ae88db745")
    for (name in names(input_pins)) {
        original <- system.file("extdata", name, package="DESeq2", mustWork=TRUE)
        if (!identical(deseq_hash(original), unname(input_pins[[name]])) ||
            !identical(deseq_hash(file.path(source, "inst/extdata", name)), unname(input_pins[[name]])) ||
            !file.copy(original, file.path(root, "inputs", name))) stop("shipped input identity differs: ", name)
    }
    counts <- as.matrix(read.delim(file.path(root, "inputs/pasilla_gene_counts.tsv.gz"),
                                  row.names="gene_id", check.names=FALSE))
    annotation <- read.csv(file.path(root, "inputs/pasilla_sample_annotation.csv"),
                           row.names=1L, check.names=FALSE)
    mapped <- sub("fb$", "", rownames(annotation))
    if (!identical(dim(counts), c(14599L, 7L)) || !is.integer(counts) ||
        sum(rowSums(counts) == 0) != 2240L || anyDuplicated(rownames(counts)) ||
        anyDuplicated(colnames(counts)) || anyDuplicated(mapped) ||
        !setequal(mapped, colnames(counts))) stop("real pasilla count/metadata identities differ")
    deseq_write_table(data.frame(original_id=rownames(annotation), sample=mapped),
                      file.path(root, "inputs/sample-id-mapping.tsv"))
    rownames(annotation) <- mapped
    coldata <- annotation[colnames(counts), c("condition", "type"), drop=FALSE]
    coldata$condition <- factor(coldata$condition, levels=c("untreated", "treated"))
    coldata$type <- factor(coldata$type, levels=c("paired-end", "single-read"))
    if (anyNA(coldata)) stop("unknown pasilla factor level")
    formula <- stats::as.formula("~ type + condition", env=baseenv())
    matrix <- stats::model.matrix(formula, data=coldata)
    if (qr(matrix)$rank != ncol(matrix) || ncol(matrix) != 3L) stop("real source design is not full rank")
    deseq_write_table(counts, file.path(root, "inputs/counts.tsv"), rownames(counts))
    deseq_write_table(coldata, file.path(root, "inputs/samples.tsv"), rownames(coldata), "sample")
    deseq_write_table(matrix, file.path(root, "inputs/model-matrix.tsv"), rownames(matrix), "sample")
    dataset <- observe("construct", DESeq2::DESeqDataSetFromMatrix(countData=counts, colData=coldata, design=formula))
    fit <- observe("DESeq", DESeq2::DESeq(dataset, test="Wald", fitType="parametric", sfType="ratio",
        betaPrior=FALSE, minReplicatesForReplace=7, useT=FALSE, minmu=0.5, parallel=FALSE))
    before <- deseq_fit_state(fit)
    if (!identical(dim(before$model_matrix), dim(matrix)) || !all(before$model_matrix == matrix) ||
        is.null(before$dispersion_attributes$fitType)) stop("fitted model or reusable dispersion trend missing")
    contrast <- c("condition", "treated", "untreated")
    result <- observe("results", DESeq2::results(fit, contrast=contrast, alpha=0.1,
        independentFiltering=TRUE, pAdjustMethod="BH", parallel=FALSE))
    normalized <- observe("normalized counts", DESeq2::counts(fit, normalized=TRUE))
    transformed <- observe("design-aware VST", DESeq2::varianceStabilizingTransformation(fit, blind=FALSE))
    if (!identical(before, deseq_fit_state(fit))) stop("result/export calls changed the fitted state")
    saveRDS(fit, file.path(root, "fit/DESeqDataSet.rds"))
    saveRDS(result, file.path(root, "results/forward.rds"))
    saveRDS(transformed, file.path(root, "results/vst.rds"))
    deseq_write_table(result, file.path(root, "results/forward.tsv"), rownames(result))
    deseq_write_table(normalized, file.path(root, "results/normalized-counts.tsv"), rownames(normalized))
    deseq_write_table(SummarizedExperiment::assay(transformed), file.path(root, "results/vst.tsv"), rownames(transformed))
    omitted <- deseq_write_table(S4Vectors::mcols(fit), file.path(root, "fit/row-diagnostics.tsv"), rownames(fit))
    deseq_write_table(stats::coef(fit), file.path(root, "fit/coefficients.tsv"), rownames(fit))
    deseq_write_table(data.frame(size_factor=DESeq2::sizeFactors(fit)),
                      file.path(root, "fit/size-factors.tsv"), colnames(fit), "sample")
    deseq_write_table(attr(fit, "modelMatrix"), file.path(root, "fit/model-matrix.tsv"), colnames(fit), "sample")
    saveRDS(S4Vectors::metadata(result), file.path(root, "results/filter-and-result-metadata.rds"))
    deseq_write_json(S4Vectors::metadata(result), file.path(root, "results/filter-and-result-metadata.json"))
    assays <- SummarizedExperiment::assays(fit)
    for (name in names(assays)) {
        if (!grepl("^[A-Za-z0-9_.-]+$", name)) stop("unexpected assay name")
        deseq_write_table(assays[[name]], file.path(root, "fit", paste0("assay-", name, ".tsv")), rownames(fit))
    }
    reloaded <- readRDS(file.path(root, "fit/DESeqDataSet.rds"))
    if (!identical(before, deseq_fit_state(reloaded))) stop("reloaded fitted state differs")
    repeat_result <- observe("reloaded results", DESeq2::results(reloaded, contrast=contrast, alpha=0.1,
        independentFiltering=TRUE, pAdjustMethod="BH", parallel=FALSE))
    repeat_normalized <- observe("reloaded normalized counts", DESeq2::counts(reloaded, normalized=TRUE))
    repeat_vst <- observe("reloaded design-aware VST", DESeq2::varianceStabilizingTransformation(reloaded, blind=FALSE))
    if (!identical(as.data.frame(result), as.data.frame(repeat_result)) ||
        !identical(S4Vectors::metadata(result), S4Vectors::metadata(repeat_result)) ||
        !identical(normalized, repeat_normalized) ||
        !identical(SummarizedExperiment::assay(transformed), SummarizedExperiment::assay(repeat_vst))) {
        stop("reloaded complete results or exports differ")
    }
    reverse <- observe("reloaded reverse contrast", DESeq2::results(reloaded,
        contrast=c("condition", "untreated", "treated"), alpha=0.1,
        independentFiltering=TRUE, pAdjustMethod="BH", parallel=FALSE))
    if (!identical(before, deseq_fit_state(reloaded))) stop("reloaded result derivation changed fitted state")
    saveRDS(list(forward=repeat_result, reverse=reverse, normalized=repeat_normalized, vst=repeat_vst),
            file.path(root, "reload/derived-objects.rds"))
    deseq_write_table(repeat_result, file.path(root, "reload/forward.tsv"), rownames(repeat_result))
    deseq_write_table(reverse, file.path(root, "reload/reverse.tsv"), rownames(reverse))
    deseq_write_table(repeat_normalized, file.path(root, "reload/normalized-counts.tsv"), rownames(repeat_normalized))
    deseq_write_table(SummarizedExperiment::assay(repeat_vst), file.path(root, "reload/vst.tsv"), rownames(repeat_vst))
    if (!identical(installed_hashes, stats::setNames(vapply(file.path(installed_root, installed_paths),
        deseq_hash, character(1)), installed_paths))) stop("installed source package files changed during execution")
    row_data <- as.data.frame(S4Vectors::mcols(fit))
    cell_sizes <- as.integer(table(apply(matrix, 1L, paste, collapse="_")))
    files <- sort(list.files(root, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
    hashes <- stats::setNames(vapply(file.path(root, files), deseq_hash, character(1)), files)
    record <- list(schema_version=1L, status="complete", package=pin, runtime=runtime, source=source_identity,
        inputs=list(count_gzip_sha256=unname(input_pins[[1L]]), annotation_sha256=unname(input_pins[[2L]]),
            genes=nrow(counts), samples=ncol(counts), all_zero_rows=sum(rowSums(counts) == 0), prefilter=FALSE,
            sample_ids=array(colnames(counts)), factor_levels=lapply(coldata, levels),
            sample_identity_mapping="documented terminal fb removal; identity join to original count-column order"),
        parameters=list(design="~ type + condition", test="Wald", fitType="parametric with upstream fallback",
            sfType="ratio", betaPrior=FALSE, minReplicatesForReplace=7L, minmu=0.5, parallel=FALSE,
            contrast=array(contrast), alpha=0.1, independentFiltering=TRUE, pAdjustMethod="BH",
            cooksCutoff="omitted: actual source default", transform="varianceStabilizingTransformation(blind=FALSE)"),
        fitted=list(results_names=array(DESeq2::resultsNames(fit)), model_rank=qr(matrix)$rank,
            actual_dispersion_fit=attr(DESeq2::dispersionFunction(fit), "fitType"),
            dispersion_attributes=attributes(DESeq2::dispersionFunction(fit)),
            assay_names=array(names(assays)), omitted_plain_diagnostic_columns=array(omitted),
            beta_nonconverged=if ("betaConv" %in% names(row_data)) sum(!row_data$betaConv, na.rm=TRUE) else NULL,
            finite_max_cooks=if ("maxCooks" %in% names(row_data)) sum(is.finite(row_data$maxCooks)) else NULL,
            replaced_genes=if ("replace" %in% names(row_data)) sum(row_data$replace, na.rm=TRUE) else 0L,
            forward_na_counts=as.list(vapply(as.data.frame(result), function(column) sum(is.na(column)), integer(1)))),
        reload=list(fitted_state_identical=TRUE, forward_results_identical=TRUE,
            result_metadata_identical=TRUE, normalized_counts_identical=TRUE, vst_identical=TRUE,
            reverse_contrast=array(c("condition", "untreated", "treated")), refit=FALSE),
        condition_count=length(conditions), warnings=sum(vapply(conditions, function(row) row$kind == "warning", logical(1))),
        scope=list(real_input="unfiltered shipped pasilla, 14599 genes and7 samples",
            design_cell_sizes=array(cell_sizes), cooks_positive_coverage=FALSE, replacement_positive_coverage=FALSE,
            excluded=array(c("numeric/interactions and general contrast coverage", "difficult convergence and trend fallback coverage",
                "native implementation and four-platform acceptance", "performance or biological validation"))),
        files=as.list(hashes))
    deseq_write_json(record, file.path(root, "witness.json"))
    manifest_files <- sort(c(files, "witness.json"), method="radix")
    write.table(data.frame(sha256=vapply(file.path(root, manifest_files), deseq_hash, character(1)), path=manifest_files),
                file.path(root, "sha256-manifest.tsv"), sep="\t", quote=FALSE, row.names=FALSE)
    cat("Completed actual DESeq2 fitted workflow: ", nrow(counts), " genes, ", ncol(counts), " samples\n", sep="")
}

if (sys.nframe() == 0L) deseq_workflow_main()
