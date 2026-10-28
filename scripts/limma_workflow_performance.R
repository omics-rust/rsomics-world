#!/usr/bin/env Rscript

limma_performance_setup <- function() {
    script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
    if (length(script) != 1L) stop("cannot identify measurement driver")
    script <- normalizePath(sub("^--file=", "", script), mustWork=TRUE)
    for (helper in c("deseq_workflow_oracle.R", "edger_workflow_oracle.R", "limma_workflow_oracle.R")) {
        base::source(file.path(dirname(script), helper), local=.GlobalEnv)
    }
    for (package in c("edgeR", "limma", "digest", "jsonlite", "BiocManager")) {
        if (!requireNamespace(package, quietly=TRUE)) stop("required installed package absent: ", package)
    }
    if (as.character(getRversion()) != "4.6.1" || as.character(BiocManager::version()) != "3.23" ||
        as.character(utils::packageVersion("edgeR")) != "4.10.7" ||
        as.character(utils::packageVersion("limma")) != "3.68.5" ||
        "DESeq2" %in% loadedNamespaces()) stop("measurement package/runtime identity differs")
    if (!all(Sys.getenv(c("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS")) == "1")) stop("source thread environment differs")
    options(digits=17, scipen=0, warn=1, contrasts=c("contr.treatment", "contr.poly"))
    if (!identical(Sys.setlocale("LC_COLLATE", "C"), "C") || !identical(Sys.setlocale("LC_NUMERIC", "C"), "C")) stop("cannot establish comparison locale")
    script
}

limma_performance_pins <- function() {
    list(edgeR=list(name="edgeR", version="4.10.7", bioconductor="3.23",
        source_commit="27ba7b39d819e68df2ecdea9f32a4b6039b2a0b5",
        source_archive_sha256="a3cc0f66be29f1a61a74021f9848ec940794e653905c4332327a719dd9ad8393"),
        limma=list(name="limma", version="3.68.5", bioconductor="3.23",
        source_commit="825d1c8330d024adb6c54af66d79b0f4ec258530",
        source_archive_sha256="40b1d2059a76ad98db4bf80139760acf8e8bae92a8265aaaf16ac199831533f4"))
}

limma_performance_qualify <- function(edgeR, limma, output, script) {
    if (!dir.create(output)) stop("qualification output already exists or cannot be created")
    pins <- limma_performance_pins()
    sources <- list(edgeR=normalizePath(edgeR, mustWork=TRUE), limma=normalizePath(limma, mustWork=TRUE))
    identities <- lapply(names(pins), function(package) {
        pin <- pins[[package]]
        prefix <- toupper(package)
        if (!identical(Sys.getenv(paste0(prefix, "_SOURCE_COMMIT")), pin$source_commit) ||
            !identical(Sys.getenv(paste0(prefix, "_SOURCE_SHA256")), pin$source_archive_sha256) ||
            !identical(deseq_hash(Sys.getenv(paste0(prefix, "_SOURCE_ARCHIVE"))), pin$source_archive_sha256)) stop("source acquisition differs: ", package)
        limma_source_identity(sources[[package]], package, pin, output)
    })
    names(identities) <- names(pins)
    if (!identical(deseq_hash(Sys.getenv("DESEQ2_SOURCE_ARCHIVE")),
        "8c91699286336350e66eec132ce6fdf5bb4af78e2a4d015a5a61224f62a95984")) stop("data-only source carrier differs")
    writeLines(capture.output(sessionInfo()), file.path(output, "session.txt"))
    packages <- utils::installed.packages()[, c("Package", "Version", "LibPath"), drop=FALSE]
    write.table(packages[order(packages[, "Package"], method="radix"), , drop=FALSE],
        file.path(output, "packages.tsv"), sep="\t", row.names=FALSE, quote=TRUE)
    if (!file.copy(script, file.path(output, basename(script)))) stop("cannot retain source measurement driver")
    runtime <- list(R=as.character(getRversion()), R_version=unclass(R.version),
        Bioconductor=as.character(BiocManager::version()), platform=R.version$platform,
        system=as.list(Sys.info()), source=identities, packages=pins,
        edgeR=edger_runtime_package("edgeR", output), limma=edger_runtime_package("limma", output),
        BLAS_LAPACK=capture.output(La_library()), locale=Sys.getlocale(), external_software=as.list(extSoftVersion()),
        data_carrier=list(package="DESeq2", installed_for_analysis=FALSE, loaded_for_analysis=FALSE),
        thread_environment=as.list(Sys.getenv(c("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"))))
    edger_json(runtime, file.path(output, "runtime.json"))
}

limma_performance_runtime <- function(qualification) {
    qualified <- jsonlite::read_json(file.path(qualification, "runtime.json"), simplifyVector=FALSE)
    for (package in c("edgeR", "limma")) {
        dll <- getLoadedDLLs()[[package]][["path"]]
        if (!identical(qualified[[package]]$installed_path, system.file(package=package, mustWork=TRUE)) ||
            !identical(qualified[[package]]$installed_dll_sha256, deseq_hash(dll)) ||
            !identical(qualified[[package]]$version, as.character(utils::packageVersion(package)))) stop("loaded runtime differs from qualification: ", package)
    }
    if (!identical(qualified$R, as.character(getRversion()))) stop("qualified R runtime differs")
}

limma_performance_parameters <- function(samples, design) {
    list(construction=list(lib.size=NULL, norm.factors=NULL, group=NULL, genes=NULL, remove.zeros=FALSE),
        formula="~type+condition", levels=lapply(samples, levels), assign=attr(design, "assign"),
        factor_coding=getOption("contrasts"), coefficients=array(colnames(design)),
        filtering=list(explicit_design=TRUE, group=NULL, lib.size=NULL, min.count=10, min.total.count=15, large.n=10, min.prop=0.7),
        subset=list(keep.lib.sizes=FALSE), normalization=list(method="TMM", refColumn=NULL, logratioTrim=0.3,
            sumTrim=0.05, doWeighting=TRUE, Acutoff=-1e10, p=0.75),
        voom=list(block=NULL, prior.weights=NULL, sample.weights=FALSE, var.design=NULL, var.group=NULL, prior.n=10,
            lib.size=NULL, offset=NULL, offset.prior=NULL, normalize.method="none", span=0.5, adaptive.span=TRUE,
            plot=FALSE, save.plot=TRUE, keep.EList=TRUE, derived_span_rule="chooseLowessSpan(kept genes, small.n=50, min.span=.3, power=1/3)"),
        moderation=list(proportion=0.01, stdev.coef.lim=c(0.1, 4), trend=FALSE, span=NULL, robust=FALSE,
            winsor.tail.p=c(0.05, 0.1), legacy=NULL),
        topTable=list(coef=1L, number=Inf, genelist="fit$genes", adjust.method="BH", sort.by="none", resort.by=NULL,
            p.value=1, fc=NULL, lfc=0, confint=FALSE),
        contrast_semantics=list(alignment="all model columns explicitly named in design order",
            mixed="algebraic diagnostic, not paired biology; retain contrasts.fit weighted nonorthogonal SE approximation"),
        reload="derive contrasts and moderation from saved base; no filter, TMM, voom or linear-model rerun")
}

limma_performance_run <- function(witness, output, qualification) {
    limma_performance_runtime(qualification)
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
    formula <- stats::as.formula("~type+condition", env=baseenv())
    design <- stats::model.matrix(formula, data=samples)
    terms <- stats::terms(formula, data=samples)
    forward <- stats::setNames(c(0, 0, 1), colnames(design))
    contrasts <- list(forward=forward, reverse=-forward, mixed=stats::setNames(c(0, -1, 1), colnames(design)))
    parameters <- limma_performance_parameters(samples, design)
    raw <- edgeR::DGEList(counts=counts, samples=samples)
    keep <- edgeR::filterByExpr(raw, design=design)
    filtered <- raw[keep, , keep.lib.sizes=FALSE]
    normalized <- edgeR::normLibSizes(filtered, method="TMM")
    base <- edgeR::voomLmFit(normalized, design=design, save.plot=TRUE, keep.EList=TRUE)
    contrast <- limma::contrasts.fit(base, contrasts=matrix(forward, ncol=1L, dimnames=list(colnames(design), "forward")))
    moderated <- limma::eBayes(contrast)
    initial <- limma::topTable(moderated, coef=1L, number=Inf, sort.by="none", adjust.method="BH", p.value=1, lfc=0, confint=FALSE)
    bundle <- list(schema_version=1L, packages=limma_performance_pins(), input=list(counts=counts, samples=samples, annotation=annotation),
        raw=raw, keep=keep, filtered=filtered, normalized=normalized, design=design, terms=terms,
        contrasts=contrasts, parameters=parameters, base=base, contrast=contrast, moderated=moderated, initial_results=initial)
    saveRDS(bundle, file.path(output, "fit.rds"))
    rm(bundle, raw, filtered, normalized, base, contrast, moderated, initial)
    reloaded <- readRDS(file.path(output, "fit.rds"))
    derived <- limma_derive(reloaded, function(label, value) force(value), "measured reload")
    genes <- rownames(reloaded$base$coefficients)
    for (name in names(derived$tests)) {
        saveRDS(derived$tests[[name]], file.path(output, paste0(name, ".rds")))
        deseq_write_table(derived$tests[[name]]$table, file.path(output, paste0(name, ".tsv")), genes)
    }
    for (name in c("logCPM", "weights", "fitted", "residuals")) {
        value <- derived[[name]]
        colnames(value) <- colnames(reloaded$normalized$counts)
        deseq_write_table(value, file.path(output, paste0(name, ".tsv")), genes)
    }
    elapsed <- proc.time() - started
    edger_json(list(inner_elapsed_seconds=unname(elapsed[["elapsed"]]),
        inner_user_seconds=unname(elapsed[["user.self"]]), inner_system_seconds=unname(elapsed[["sys.self"]]),
        region="original gzip/CSV read and join, source design, default filter/TMM/current voom, initial forward contrast/EB/table, complete RDS save/reload, three contrasts/EB/tables/full RDS states and four observation/model TSV exports",
        excluded="package loading and identity checks; validation and extra oracle diagnostic exports run in a different untimed process",
        serialization="default gzip-compressed full RDS analysis plus three contrast/moderated/table RDS; not the portable uncompressed native JSON encoding"),
        file.path(output, "inner-timing.json"))
}

limma_performance_inspect <- function(trial, output, qualification, witness) {
    limma_performance_runtime(qualification)
    if (!dir.create(output)) stop("inspection output already exists or cannot be created")
    bounds <- jsonlite::read_json(file.path(dirname(qualification), "bounds.json"), simplifyVector=TRUE)
    failures <- list()
    metrics <- list()
    exact_fields <- 0L
    fail <- function(path, reason, details=NULL) {
        failures[[length(failures)+1L]] <<- list(path=path, reason=reason, details=details)
    }
    kind <- function(path) {
        if (grepl("/parameters(/|$)|/input(/|$)|/design(/|$)|/terms(/|$)|/contrasts(/|$)|/counts$|/df.residual$", path)) return(NULL)
        slot <- tail(strsplit(path, "/", fixed=TRUE)[[1L]], 1L)
        if (slot == "norm.factors") return("tmm")
        if (grepl("/voom[.]line/delta$", path)) return("tmm")
        if (slot == "lib.size") return("library")
        if (slot %in% c("E", "Amean", "AveExpr")) return("expression")
        if (slot %in% c("coefficients", "stdev.unscaled", "cov.coefficients", "logFC")) return("linear")
        if (slot == "sigma") return("trend")
        if (slot == "weights") return("weights")
        if (slot %in% c("df.prior", "df.total")) return("df")
        if (slot %in% c("s2.prior", "s2.post", "t", "F")) return("eb")
        if (slot %in% c("lods", "var.prior", "B")) return("odds")
        if (slot %in% c("p.value", "F.p.value", "P.Value", "adj.P.Val")) return("probability")
        if (grepl("/voom[.]xy/x$|/voom[.]line/x$", path)) return("expression")
        if (grepl("/voom[.]xy/y$|/voom[.]line/y$", path)) return("trend")
        NULL
    }
    numeric_class <- function(value) {
        ifelse(is.nan(value), "NaN", ifelse(is.na(value), "NA", ifelse(is.infinite(value),
            ifelse(value > 0, "Inf", "-Inf"), ifelse(value == 0, "zero", "finite"))))
    }
    walk <- function(actual, expected, path) {
        if (!identical(typeof(actual), typeof(expected)) || !identical(attributes(actual), attributes(expected)) ||
            length(actual) != length(expected)) {
            fail(path, "type/attributes/shape/names differ")
            return(invisible(NULL))
        }
        if (is.list(actual)) {
            labels <- names(actual)
            if (is.null(labels)) labels <- as.character(seq_along(actual))
            for (i in seq_along(actual)) walk(actual[[i]], expected[[i]], paste(path, labels[[i]], sep="/"))
            return(invisible(NULL))
        }
        selected <- if (is.numeric(actual)) kind(path) else NULL
        if (is.null(selected)) {
            exact_fields <<- exact_fields+1L
            equal <- if (is.language(actual)) identical(deparse(actual), deparse(expected)) else identical(actual, expected)
            if (!equal) fail(path, "exact field differs")
            return(invisible(NULL))
        }
        classes <- numeric_class(actual)
        reference_classes <- numeric_class(expected)
        mismatch <- which(classes != reference_classes)
        if (length(mismatch)) fail(path, "zero/NA/NaN/infinity class differs", list(indices=array(mismatch)))
        finite <- which(classes == reference_classes & classes %in% c("zero", "finite"))
        pair <- bounds[[selected]]
        scaled <- abs(actual[finite]-expected[finite])/(pair[[1L]]+pair[[2L]]*abs(expected[finite]))
        log_error <- rep(0, length(finite))
        positive <- actual[finite] > 0 & expected[finite] > 0
        if (selected == "probability") log_error[positive] <- abs(log(actual[finite][positive]/expected[finite][positive]))
        bad <- which(scaled > 1 | log_error > 1e-3)
        metrics[[path]] <<- list(kind=selected, count=length(finite), failed=length(bad),
            max_scaled=if (length(scaled)) max(scaled) else 0, max_abs=if (length(finite)) max(abs(actual[finite]-expected[finite])) else 0,
            max_log_probability_error=if (length(log_error)) max(log_error) else 0)
        if (length(bad)) fail(path, "fixed numerical bound failed", list(indices=array(finite[bad]),
            actual=array(actual[finite[bad]]), expected=array(expected[finite[bad]]), scaled=array(scaled[bad]), log_error=array(log_error[bad])))
    }
    bundle <- readRDS(file.path(trial, "fit.rds"))
    reference <- readRDS(file.path(witness, "fit/analysis-bundle.rds"))
    if (!identical(names(bundle), c(names(reference), "initial_results"))) fail("bundle", "complete base/input/provenance inventory differs")
    walk(bundle[names(reference)], reference, "bundle")
    fixed <- readRDS(file.path(witness, "results/complete-derived-state.rds"))
    walk(bundle$initial_results, fixed$tests$forward$table, "initial/table")
    limma_write_fit(bundle$base, file.path(output, "base"))
    limma_write_fit(bundle$contrast, file.path(output, "initial-contrast"))
    limma_write_fit(bundle$moderated, file.path(output, "initial-moderated"))
    discoveries <- list()
    for (name in names(bundle$contrasts)) {
        state <- readRDS(file.path(trial, paste0(name, ".rds")))
        walk(state, fixed$tests[[name]], paste0("results/", name))
        if (name == "forward" && (!identical(bundle$contrast, state$contrast) ||
            !identical(bundle$moderated, state$moderated) || !identical(bundle$initial_results, state$table))) fail(name, "initial/reloaded full forward state differs")
        actual_ids <- rownames(state$table)[which(state$table$adj.P.Val <= 0.05)]
        expected_ids <- rownames(fixed$tests[[name]]$table)[which(fixed$tests[[name]]$table$adj.P.Val <= 0.05)]
        if (!identical(actual_ids, expected_ids)) fail(name, "discovery identities/order differ")
        discoveries[[name]] <- list(actual=length(actual_ids), source=length(expected_ids))
        limma_write_fit(state$contrast, file.path(output, paste0(name, "-contrast")))
        limma_write_fit(state$moderated, file.path(output, paste0(name, "-moderated")))
        copy <- file.path(output, paste0(name, ".tsv"))
        deseq_write_table(state$table, copy, rownames(state$table))
        if (!identical(deseq_hash(copy), deseq_hash(file.path(trial, paste0(name, ".tsv"))))) fail(name, "timed table differs from actual measured RDS state")
    }
    for (name in c("logCPM", "weights", "fitted", "residuals")) {
        actual <- read.delim(file.path(trial, paste0(name, ".tsv")), row.names=1L, check.names=FALSE)
        expected <- read.delim(file.path(witness, "results", paste0(name, ".tsv")), row.names=1L, check.names=FALSE)
        if (!identical(dimnames(actual), dimnames(expected))) fail(name, "timed export identities/dimensions differ")
        for (column in seq_along(actual)) {
            slot <- switch(name, logCPM="E", weights="weights", fitted="coefficients", residuals="coefficients")
            walk(actual[[column]], expected[[column]], paste0("exports/", name, "/", colnames(actual)[[column]], "/", slot))
        }
        if (name %in% c("logCPM", "weights")) {
            stored <- if (name == "logCPM") bundle$base$EList$E else bundle$base$EList$weights
            colnames(stored) <- colnames(bundle$normalized$counts)
            copy <- file.path(output, paste0(name, ".tsv"))
            deseq_write_table(stored, copy, rownames(bundle$base$coefficients))
            if (!identical(deseq_hash(copy), deseq_hash(file.path(trial, paste0(name, ".tsv"))))) fail(name, "timed export differs from serialized base observation state")
        }
    }
    edger_json(failures, file.path(output, "failures.json"))
    edger_json(list(passed=length(failures) == 0L, failure_count=length(failures), bounds=bounds,
        positive_probability_log_bound=1e-3, metrics=metrics, exact_fields=exact_fields,
        discoveries=discoveries, raw_genes=nrow(bundle$raw), kept_genes=nrow(bundle$base$coefficients),
        residual_df_distribution=as.list(table(bundle$base$df.residual)),
        inspection="actual measured complete RDS/base/initial/three contrast states and seven tables; no fitting, normalization, voom or statistical re-derivation"),
        file.path(output, "comparison.json"))
    if (length(failures)) stop("actual measured source state/output comparison failed; all failures retained")
}

limma_workflow_performance_main <- function() {
    args <- commandArgs(trailingOnly=TRUE)
    script <- limma_performance_setup()
    destination <- if (length(args) == 4L && args[[1L]] == "qualify") args[[4L]] else if (length(args) >= 3L) args[[3L]] else NULL
    index <- 0L
    note <- function(condition, kind) {
        if (!is.null(destination) && dir.exists(destination)) {
            index <<- index+1L
            row <- data.frame(index=index, kind=kind, class=paste(class(condition), collapse=","),
                message=conditionMessage(condition), call=paste(deparse(conditionCall(condition)), collapse=" "))
            write.table(row, file.path(destination, "conditions.tsv"), sep="\t", quote=TRUE,
                row.names=FALSE, col.names=index == 1L, append=index > 1L)
        }
    }
    withCallingHandlers({
        if (length(args) == 4L && args[[1L]] == "qualify") {
            limma_performance_qualify(args[[2L]], args[[3L]], args[[4L]], script)
        } else if (length(args) == 4L && args[[1L]] == "run") {
            limma_performance_run(args[[2L]], args[[3L]], args[[4L]])
        } else if (length(args) == 5L && args[[1L]] == "inspect") {
            limma_performance_inspect(args[[2L]], args[[3L]], args[[4L]], args[[5L]])
        } else stop("usage: limma_workflow_performance.R qualify EDGER LIMMA OUTPUT | run WITNESS OUTPUT QUALIFICATION | inspect TRIAL OUTPUT QUALIFICATION WITNESS")
    }, warning=function(value) note(value, "warning"), message=function(value) note(value, "message"), error=function(value) note(value, "error"))
}

if (sys.nframe() == 0L) limma_workflow_performance_main()
