#!/usr/bin/env Rscript

deseq_performance_setup <- function() {
    script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
    if (length(script) != 1L) stop("cannot identify measurement driver")
    script <- normalizePath(sub("^--file=", "", script), mustWork=TRUE)
    base::source(file.path(dirname(script), "deseq_workflow_oracle.R"), local=.GlobalEnv)
    for (package in c("DESeq2", "digest", "jsonlite", "BiocManager")) {
        if (!requireNamespace(package, quietly=TRUE)) stop("required installed package absent: ", package)
    }
    if (as.character(getRversion()) != "4.6.1" || as.character(BiocManager::version()) != "3.23" ||
        as.character(utils::packageVersion("DESeq2")) != "1.52.0") stop("measurement package/runtime identity differs")
    options(digits=17, scipen=0, warn=1, contrasts=c("contr.treatment", "contr.poly"))
    if (!identical(Sys.setlocale("LC_COLLATE", "C"), "C") || !identical(Sys.setlocale("LC_NUMERIC", "C"), "C")) stop("cannot establish comparison locale")
    script
}

deseq_performance_qualify <- function(source, output, script) {
    if (!identical(Sys.getenv("DESEQ2_SOURCE_COMMIT"), "16aeab6d6158bd8cbb8e98764c2f33399b1a3fd3") ||
        !identical(deseq_hash(Sys.getenv("DESEQ2_SOURCE_ARCHIVE")),
            "8c91699286336350e66eec132ce6fdf5bb4af78e2a4d015a5a61224f62a95984")) stop("released source acquisition differs")
    if (!dir.create(output)) stop("qualification output already exists or cannot be created")
    identity <- deseq_source_identity(source, output)
    installed <- system.file(package="DESeq2", mustWork=TRUE)
    files <- sort(list.files(installed, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
    write.table(data.frame(sha256=vapply(file.path(installed, files), deseq_hash, character(1)), path=files),
        file.path(output, "installed-DESeq2-sha256.tsv"), sep="\t", row.names=FALSE, quote=FALSE)
    dll <- getLoadedDLLs()[["DESeq2"]][["path"]]
    if (!file.copy(dll, file.path(output, basename(dll))) || !file.copy(script, file.path(output, basename(script)))) stop("cannot retain loaded driver/DLL")
    writeLines(capture.output(sessionInfo()), file.path(output, "session.txt"))
    packages <- utils::installed.packages()[, c("Package", "Version", "LibPath"), drop=FALSE]
    write.table(packages[order(packages[, "Package"], method="radix"), , drop=FALSE],
        file.path(output, "packages.tsv"), sep="\t", row.names=FALSE, quote=TRUE)
    runtime <- list(R=as.character(getRversion()), Bioconductor=as.character(BiocManager::version()),
        DESeq2=as.character(utils::packageVersion("DESeq2")), platform=R.version$platform,
        installed_path=installed, installed_dll=dll, installed_dll_sha256=deseq_hash(dll), source=identity,
        BLAS_LAPACK=capture.output(La_library()), locale=Sys.getlocale(), external_software=as.list(extSoftVersion()),
        thread_environment=as.list(Sys.getenv(c("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"))))
    deseq_write_json(runtime, file.path(output, "runtime.json"))
}

deseq_performance_runtime <- function(qualification) {
    qualified <- jsonlite::read_json(file.path(qualification, "runtime.json"), simplifyVector=FALSE)
    dll <- getLoadedDLLs()[["DESeq2"]][["path"]]
    if (!identical(qualified$R, as.character(getRversion())) ||
        !identical(qualified$installed_path, system.file(package="DESeq2", mustWork=TRUE)) ||
        !identical(qualified$installed_dll_sha256, deseq_hash(dll))) stop("loaded measurement runtime differs from qualification")
}

deseq_performance_run <- function(witness, output, qualification) {
    deseq_performance_runtime(qualification)
    if (!dir.create(output)) stop("trial output already exists or cannot be created")
    started <- proc.time()
    counts <- as.matrix(read.delim(file.path(witness, "inputs/pasilla_gene_counts.tsv.gz"), row.names="gene_id", check.names=FALSE))
    annotation <- read.csv(file.path(witness, "inputs/pasilla_sample_annotation.csv"), row.names=1L, check.names=FALSE)
    mapped <- sub("fb$", "", rownames(annotation))
    if (!identical(dim(counts), c(14599L, 7L)) || !is.integer(counts) || anyDuplicated(mapped) ||
        !setequal(mapped, colnames(counts))) stop("original pasilla identities differ")
    rownames(annotation) <- mapped
    samples <- annotation[colnames(counts), c("condition", "type"), drop=FALSE]
    samples$condition <- factor(samples$condition, levels=c("untreated", "treated"))
    samples$type <- factor(samples$type, levels=c("paired-end", "single-read"))
    design <- stats::as.formula("~ type + condition", env=baseenv())
    dataset <- DESeq2::DESeqDataSetFromMatrix(countData=counts, colData=samples, design=design)
    fit <- DESeq2::DESeq(dataset, test="Wald", fitType="parametric", sfType="ratio", betaPrior=FALSE,
        minReplicatesForReplace=7L, useT=FALSE, minmu=0.5, parallel=FALSE)
    initial <- DESeq2::results(fit, contrast=c("condition", "treated", "untreated"), alpha=0.1,
        independentFiltering=TRUE, pAdjustMethod="BH", parallel=FALSE)
    analysis <- list(fit=fit, initial_results=initial, schema_version=1L,
        parameters=list(design="~type+condition", test="Wald", fitType="parametric with actual source fallback",
            sfType="ratio", betaPrior=FALSE, minReplicatesForReplace=7L, useT=FALSE, minmu=0.5,
            alpha=0.1, independentFiltering=TRUE, pAdjustMethod="BH", parallel=FALSE, blind=FALSE))
    saveRDS(analysis, file.path(output, "fit.rds"))
    rm(analysis, initial, fit)
    reloaded <- readRDS(file.path(output, "fit.rds"))
    forward <- DESeq2::results(reloaded$fit, contrast=c("condition", "treated", "untreated"), alpha=0.1,
        independentFiltering=TRUE, pAdjustMethod="BH", parallel=FALSE)
    reverse <- DESeq2::results(reloaded$fit, contrast=c("condition", "untreated", "treated"), alpha=0.1,
        independentFiltering=TRUE, pAdjustMethod="BH", parallel=FALSE)
    normalized <- DESeq2::counts(reloaded$fit, normalized=TRUE)
    vst <- DESeq2::varianceStabilizingTransformation(reloaded$fit, blind=FALSE)
    deseq_write_table(forward, file.path(output, "forward.tsv"), rownames(forward))
    deseq_write_table(reverse, file.path(output, "reverse.tsv"), rownames(reverse))
    deseq_write_table(normalized, file.path(output, "normalized-counts.tsv"), rownames(normalized))
    deseq_write_table(SummarizedExperiment::assay(vst), file.path(output, "vst.tsv"), rownames(vst))
    elapsed <- proc.time() - started
    deseq_write_json(list(inner_elapsed_seconds=unname(elapsed[["elapsed"]]),
        inner_user_seconds=unname(elapsed[["user.self"]]), inner_system_seconds=unname(elapsed[["sys.self"]]),
        source_DLL_sha256=deseq_hash(getLoadedDLLs()[["DESeq2"]][["path"]]),
        region="original gzip counts/CSV metadata read and identity join, named design, complete fit, initial contrast, RDS save/reload, forward/reverse results, normalized counts and fitted VST, all four TSV writes",
        excluded="package loading and identity qualification; no fit validation or extra diagnostic export in this timed process",
        encoding="full fitted RDS bundle, default gzip compression; not the native portable uncompressed JSON encoding"),
        file.path(output, "inner-timing.json"))
}

deseq_performance_inspect <- function(trial, output, qualification) {
    deseq_performance_runtime(qualification)
    if (!dir.create(output)) stop("inspection output already exists or cannot be created")
    analysis <- readRDS(file.path(trial, "fit.rds"))
    fit <- analysis$fit
    deseq_write_table(S4Vectors::mcols(fit), file.path(output, "row-diagnostics.tsv"), rownames(fit))
    deseq_write_table(stats::coef(fit), file.path(output, "coefficients.tsv"), rownames(fit))
    deseq_write_table(data.frame(size_factor=DESeq2::sizeFactors(fit)), file.path(output, "size-factors.tsv"), colnames(fit), "sample")
    deseq_write_table(attr(fit, "modelMatrix"), file.path(output, "model-matrix.tsv"), colnames(fit), "sample")
    for (name in names(SummarizedExperiment::assays(fit))) {
        deseq_write_table(SummarizedExperiment::assay(fit, name), file.path(output, paste0("assay-", name, ".tsv")), rownames(fit))
    }
    deseq_write_json(S4Vectors::metadata(analysis$initial_results), file.path(output, "filter-and-result-metadata.json"))
    deseq_write_json(list(genes=nrow(fit), samples=ncol(fit), results_names=array(DESeq2::resultsNames(fit)),
        actual_dispersion_fit=attr(DESeq2::dispersionFunction(fit), "fitType"),
        dispersion_attributes=attributes(DESeq2::dispersionFunction(fit)),
        initial_result_na_counts=as.list(vapply(as.data.frame(analysis$initial_results), function(value) sum(is.na(value)), integer(1))),
        initial_discoveries=sum(analysis$initial_results$padj < 0.1, na.rm=TRUE),
        options=analysis$parameters, inspection="read saved measured fit only; no DESeq, normalization or dispersion/GLM refit"),
        file.path(output, "fit-summary.json"))
}

deseq_workflow_performance_main <- function() {
    args <- commandArgs(trailingOnly=TRUE)
    script <- deseq_performance_setup()
    if (length(args) == 3L && args[[1L]] == "qualify") {
        deseq_performance_qualify(args[[2L]], args[[3L]], script)
    } else if (length(args) == 4L && args[[1L]] == "run") {
        deseq_performance_run(args[[2L]], args[[3L]], args[[4L]])
    } else if (length(args) == 4L && args[[1L]] == "inspect") {
        deseq_performance_inspect(args[[2L]], args[[3L]], args[[4L]])
    } else stop("usage: deseq_workflow_performance.R qualify SOURCE OUTPUT | run WITNESS OUTPUT QUALIFICATION | inspect TRIAL OUTPUT QUALIFICATION")
}

if (sys.nframe() == 0L) deseq_workflow_performance_main()
