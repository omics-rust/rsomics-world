#!/usr/bin/env Rscript

edger_json <- function(value, path) {
    safe <- function(value) {
        if (is.list(value)) return(lapply(value, safe))
        if (is.numeric(value) && any(!is.finite(value))) {
            result <- lapply(value, function(number) {
                if (is.nan(number)) "NaN" else if (is.na(number)) "NA" else if (is.infinite(number)) {
                    if (number > 0) "Inf" else "-Inf"
                } else number
            })
            if (length(result) == 1L && is.null(names(value))) return(result[[1L]])
            return(result)
        }
        value
    }
    deseq_write_json(safe(value), path)
}

edger_definition <- function(path, name) {
    find <- function(expression) {
        if (!is.call(expression)) return(NULL)
        if (identical(expression[[1L]], as.name("assign")) && length(expression) >= 3L &&
            identical(expression[[2L]], name)) return(expression[[3L]])
        if (length(expression) == 3L && as.character(expression[[1L]]) %in% c("<-", "=")) {
            if (is.symbol(expression[[2L]]) && identical(as.character(expression[[2L]]), name)) {
                value <- expression[[3L]]
                while (is.call(value) && length(value) == 3L && as.character(value[[1L]]) %in% c("<-", "=")) value <- value[[3L]]
                return(value)
            }
            return(find(expression[[3L]]))
        }
        NULL
    }
    for (expression in parse(path, keep.source=FALSE)) {
        value <- find(expression)
        if (!is.null(value) && is.call(value) && identical(value[[1L]], as.name("function"))) return(value)
    }
    stop("missing released function definition: ", name)
}

edger_source_identity <- function(source, package, pin, output) {
    description <- read.dcf(file.path(source, "DESCRIPTION"))
    if (description[1L, "Version"] != pin$version || description[1L, "git_branch"] != "RELEASE_3_23" ||
        description[1L, "git_last_commit"] != substr(pin$source_commit, 1L, 7L)) stop("released package metadata differs: ", package)
    if (package == "edgeR") {
        files <- c(
            "R/DGEList.R"="45468b3c2f5aaed81145659129be285f3474d2b4f92419e8f8e2e80dd66375f4",
            "R/filterByExpr.R"="7c72593262c2b4c8082c7f316e8d27f3d936cb448a99f812cd24e03fa33e8c93",
            "R/subsetting.R"="40e97bd91bd4792d31d9d7289249a3663ac545c90f2d8a7dd3164c3b999357da",
            "R/normLibSizes.R"="2495e88f5346d53b3bf89edf768548821f77fbd68671207cee88e415b7a0207a",
            "R/calcNormFactors.R"="af94689615bd68b4a68d70bde306e3e4bd34207ee861b98892b53377aa07eed1",
            "R/glmQLFTest.R"="693b7c70bdb246532cf9088f8fcbb1af9bd93c337c0e38ba34bd43b98c0de269",
            "R/glmfit.R"="ebdc2f8fb338b79a1a0fce7b978f6399a9105c14bacd951b6f988a52f35fdb98",
            "R/mglmLevenberg.R"="7743c75f75e7bf844eede263e0365fbcd03a18c974f1f9e7331b45c7c0c6b3de",
            "R/estimateGLMCommonDisp.R"="019c2aeb60b54dbce40d07da9e3071eccbaf24db4f6b07dcbc201264b7701878",
            "R/dispCoxReid.R"="2445afd31b4ae4e6f353fcb8a3a84c990c85850ef41cfafd8a19dc0772b41571",
            "R/adjustedProfileLik.R"="3c65fdfb57e5d4f0fbf4f671afb6684e8b9461e0660ff17e5f77f874bc638535",
            "R/cpm.R"="54ff5f77505fa328b7c7e239e6113d5c2a0bc3213c37fd53165d1f1e468388f1",
            "R/aveLogCPM.R"="7a3ad33840ee2e41e1ff210d796212911d602e59d3e41de380b1641f2fd56659",
            "R/getOffset.R"="9cb70fc8a52d82bd57668f7d9094845d60f62b3134d1c7bccb91b886681582de",
            "R/topTags.R"="f0e4346478fdc401dac7d64fb77504495c03e065db4925bec032daad518bb033",
            "src/glm.c"="b3662748b5fed05d3f7d5b11a17f98e01be928a38ef6a1a6c8ea8a9975efdb4d",
            "src/ql_glm.c"="90fa2c6d4bd36d0cb2cbb32d08343139e3e3786711ed300373080e62a92d246b",
            "src/ql_weights.c"="4a2ad18c8793f96ea5ff9f7a53277244f49af8ea5202473425be38d505258e47",
            "src/R_exports.c"="e999ba0f5860561532d43f25271e902023dee896976727d940306281fcd2228b",
            "src/compute_cpm.c"="5d17e75b3b6a578a0a9971a99de0ca4f3ebab388e32ac71ea5116ecf336b7f02",
            "src/compute_apl.c"="742daa33811fe5bc160557cbf0bc7c7374278acd20d99a1d1883a282ce4e1139",
            "src/compute_nbdev.c"="e6285f00ec06f93efb35adc907fb0ffe9cf68347a0a30fbd848b0bed31bef97b")
        functions <- c(DGEList="R/DGEList.R", DGEList.default="R/DGEList.R",
            filterByExpr="R/filterByExpr.R", filterByExpr.DGEList="R/filterByExpr.R", filterByExpr.default="R/filterByExpr.R",
            "[.DGEList"="R/subsetting.R", normLibSizes="R/normLibSizes.R",
            normLibSizes.DGEList="R/normLibSizes.R", normLibSizes.default="R/normLibSizes.R",
            .calcFactorTMM="R/calcNormFactors.R", .calcFactorQuantile="R/calcNormFactors.R",
            glmQLFit="R/glmQLFTest.R", glmQLFit.DGEList="R/glmQLFTest.R", glmQLFit.default="R/glmQLFTest.R",
            glmQLFTest="R/glmQLFTest.R", glmFit="R/glmfit.R", glmFit.default="R/glmfit.R", glmLRT="R/glmfit.R",
            mglmLevenberg="R/mglmLevenberg.R", estimateGLMCommonDisp.default="R/estimateGLMCommonDisp.R",
            dispCoxReid="R/dispCoxReid.R", adjustedProfileLik="R/adjustedProfileLik.R",
            cpm="R/cpm.R", cpm.DGEList="R/cpm.R", cpm.default="R/cpm.R",
            aveLogCPM.DGEList="R/aveLogCPM.R", aveLogCPM.default="R/aveLogCPM.R", getOffset="R/getOffset.R", topTags="R/topTags.R")
    } else {
        files <- c(
            "R/squeezeVar.R"="237d96decee3360c552080276711ccbb720501829f96cea9b4e0cb1e3209f3d5",
            "R/fitFDistUnequalDF1.R"="4749ecc1fa0d5a48d991542faaf74d17c045f3ca0da2594c6774301a203f9c9c",
            "R/chooseLowessSpan.R"="2a970a8c8746498fc280e00619901bb20a153d3152c7a503bd34a111f0ec0406",
            "R/loessFit.R"="ae3403e10e0d8e832e02e9e9b04856d10966670a1f75de50a3bd1a06ca648329",
            "R/weightedLowess.R"="76ded7ec8bde5e2bca2082d678f5a1f3a8bc0f28dcf438308beb6adecb4898fd")
        functions <- c(squeezeVar="R/squeezeVar.R", .squeezeVar="R/squeezeVar.R",
            fitFDistUnequalDF1="R/fitFDistUnequalDF1.R", chooseLowessSpan="R/chooseLowessSpan.R",
            loessFit="R/loessFit.R", weightedLowess="R/weightedLowess.R")
    }
    for (name in names(files)) {
        if (!identical(deseq_hash(file.path(source, name)), unname(files[[name]]))) stop("source differs from release: ", package, "/", name)
    }
    for (name in names(functions)) {
        expected <- edger_definition(file.path(source, functions[[name]]), name)
        installed <- get(name, asNamespace(package), inherits=FALSE)
        if (!identical(deparse(formals(installed)), deparse(expected[[2L]])) ||
            !identical(deparse(body(installed)), deparse(expected[[3L]]))) stop("installed function differs from release: ", package, "::", name)
        filename <- if (name == "[.DGEList") "subset-DGEList" else name
        writeLines(c("formals:", deparse(formals(installed)), "body:", deparse(body(installed))),
            file.path(output, paste0(package, "-function-", filename, ".txt")))
    }
    list(source_files=as.list(files), checked_functions=as.list(functions))
}

edger_runtime_package <- function(package, output) {
    installed_root <- system.file(package=package, mustWork=TRUE)
    files <- sort(list.files(installed_root, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
    hashes <- vapply(file.path(installed_root, files), deseq_hash, character(1))
    write.table(data.frame(sha256=hashes, path=files), file.path(output, paste0("installed-", package, "-sha256.tsv")),
        sep="\t", quote=FALSE, row.names=FALSE)
    dll <- getLoadedDLLs()[[package]][["path"]]
    if (!file.copy(dll, file.path(output, paste0(package, "-", basename(dll))))) stop("cannot retain loaded package DLL")
    list(version=as.character(utils::packageVersion(package)), installed_path=installed_root,
        installed_dll=dll, installed_dll_sha256=deseq_hash(dll))
}

edger_row_vector <- function(value, n) {
    if (!length(value) %in% c(1L, n)) stop("source diagnostic length differs from feature rows")
    rep_len(value, n)
}

edger_derive <- function(bundle, observe, label) {
    tests <- lapply(names(bundle$contrasts), function(name) {
        test <- observe(paste(label, "glmQLFTest", name), edgeR::glmQLFTest(bundle$fit, contrast=bundle$contrasts[[name]]))
        tags <- observe(paste(label, "topTags", name), edgeR::topTags(test, n=Inf, sort.by="none", adjust.method="BH"))
        list(test=test, tags=tags)
    })
    names(tests) <- names(bundle$contrasts)
    list(tests=tests, observed_cpm=observe(paste(label, "observed CPM"), edgeR::cpm(bundle$normalized)),
        observed_log_cpm=observe(paste(label, "observed logCPM"), edgeR::cpm(bundle$normalized, log=TRUE)),
        held_AveLogCPM=bundle$fit$AveLogCPM)
}

edger_write_derived <- function(derived, bundle, directory) {
    genes <- rownames(bundle$fit$counts)
    for (name in names(derived$tests)) {
        value <- derived$tests[[name]]
        saveRDS(value, file.path(directory, paste0(name, ".rds")))
        deseq_write_table(value$tags$table, file.path(directory, paste0(name, ".tsv")), genes)
        deseq_write_table(data.frame(df_test=value$test$df.test, df_total=value$test$df.total),
            file.path(directory, paste0(name, "-test-df.tsv")), genes)
    }
    for (name in c("observed_cpm", "observed_log_cpm")) {
        value <- derived[[name]]
        if (!identical(dim(value), dim(bundle$fit$counts))) stop("source export dimensions changed")
        colnames(value) <- colnames(bundle$fit$counts)
        deseq_write_table(value, file.path(directory, paste0(name, ".tsv")), genes)
    }
    deseq_write_table(data.frame(AveLogCPM=derived$held_AveLogCPM), file.path(directory, "held-AveLogCPM.tsv"), genes)
    saveRDS(derived, file.path(directory, "complete-derived-state.rds"))
}

edger_workflow_main <- function() {
    args <- commandArgs(trailingOnly=TRUE)
    if (length(args) != 4L) stop("usage: edger_workflow_oracle.R EDGER_SOURCE LIMMA_SOURCE DATA_SOURCE OUTPUT_DIR")
    script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
    if (length(script) != 1L) stop("cannot identify source driver")
    script <- normalizePath(sub("^--file=", "", script), mustWork=TRUE)
    helper <- file.path(dirname(script), "deseq_workflow_oracle.R")
    base::source(helper, local=.GlobalEnv)
    for (package in c("edgeR", "limma", "digest", "jsonlite", "BiocManager")) {
        if (!requireNamespace(package, quietly=TRUE)) stop("required installed package absent: ", package)
    }
    pins <- list(edgeR=list(name="edgeR", version="4.10.7", bioconductor="3.23",
        source_commit="27ba7b39d819e68df2ecdea9f32a4b6039b2a0b5",
        source_archive_sha256="a3cc0f66be29f1a61a74021f9848ec940794e653905c4332327a719dd9ad8393"),
        limma=list(name="limma", version="3.68.5", bioconductor="3.23",
        source_commit="825d1c8330d024adb6c54af66d79b0f4ec258530",
        source_archive_sha256="40b1d2059a76ad98db4bf80139760acf8e8bae92a8265aaaf16ac199831533f4"))
    if (as.character(getRversion()) != "4.6.1" || as.character(BiocManager::version()) != "3.23") stop("source runtime differs")
    for (package in names(pins)) {
        prefix <- toupper(package)
        archive <- Sys.getenv(paste0(prefix, "_SOURCE_ARCHIVE"))
        pin <- pins[[package]]
        if (as.character(utils::packageVersion(package)) != pin$version || !nzchar(archive) ||
            !identical(deseq_hash(archive), pin$source_archive_sha256) ||
            !identical(Sys.getenv(paste0(prefix, "_SOURCE_SHA256")), pin$source_archive_sha256) ||
            !identical(Sys.getenv(paste0(prefix, "_SOURCE_COMMIT")), pin$source_commit)) stop("source acquisition identity differs: ", package)
    }
    carrier_hash <- "8c91699286336350e66eec132ce6fdf5bb4af78e2a4d015a5a61224f62a95984"
    if (!identical(deseq_hash(Sys.getenv("DESEQ2_SOURCE_ARCHIVE")), carrier_hash) ||
        "DESeq2" %in% loadedNamespaces()) stop("raw-data carrier identity or source-only use differs")
    sources <- list(edgeR=normalizePath(args[[1L]], mustWork=TRUE), limma=normalizePath(args[[2L]], mustWork=TRUE))
    data_source <- normalizePath(args[[3L]], mustWork=TRUE)
    root <- normalizePath(args[[4L]], mustWork=FALSE)
    if (file.exists(root) || dir.exists(root)) stop("output directory already exists")
    if (!dir.create(root, recursive=TRUE)) stop("cannot create output directory")
    for (name in c("metadata", "inputs", "fit", "results", "reload")) {
        if (!dir.create(file.path(root, name))) stop("cannot create output subdirectory")
    }
    metadata <- file.path(root, "metadata")
    status <- "failed"
    failure <- NULL
    runtime <- source_identity <- parameters <- checks <- NULL
    conditions <- list()
    on.exit({
        files <- sort(list.files(root, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
        hashes <- stats::setNames(vapply(file.path(root, files), deseq_hash, character(1)), files)
        witness <- list(schema_version=1L, status=status, packages=pins, runtime=runtime, source=source_identity,
            data_carrier=list(package="DESeq2", version="1.52.0", source_archive_sha256=carrier_hash, installed_or_used_for_fit=FALSE),
            parameters=parameters, checks=checks, failure=failure,
            conditions=list(count=length(conditions), warnings=sum(vapply(conditions, function(value) value$kind == "warning", logical(1))),
                errors=sum(vapply(conditions, function(value) value$kind == "error", logical(1)))),
            nonfinite_encoding="TSV and RDS preserve NA, NaN, Inf, -Inf; JSON nonfinite numbers are explicit string tokens, never null",
            exclusions=array(c("native implementation or compatibility acceptance", "performance or biological claims",
                "robust, legacy, classic, LRT and Treat workflows", "supplied offsets, weights or library sizes",
                "other normalizations, RPKM, splicing, single-cell imports and cross-product fitted-state interchange")),
            files=as.list(hashes))
        edger_json(witness, file.path(root, "witness.json"))
        manifest <- sort(c(files, "witness.json"), method="radix")
        write.table(data.frame(sha256=vapply(file.path(root, manifest), deseq_hash, character(1)), path=manifest),
            file.path(root, "sha256-manifest.tsv"), sep="\t", quote=FALSE, row.names=FALSE)
    })
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
            message=function(value) note(value, "message"), error=function(value) note(value, "error"))
    }
    tryCatch({
        options(digits=17, scipen=0, warn=1, contrasts=c("contr.treatment", "contr.poly"))
        if (!identical(Sys.setlocale("LC_COLLATE", "C"), "C") || !identical(Sys.setlocale("LC_NUMERIC", "C"), "C")) stop("cannot establish comparison locale")
        Sys.setenv(OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", MKL_NUM_THREADS="1")
        source_identity <- lapply(names(sources), function(package) edger_source_identity(sources[[package]], package, pins[[package]], metadata))
        names(source_identity) <- names(sources)
        for (path in c(script, helper)) if (!file.copy(path, file.path(metadata, basename(path)))) stop("cannot retain driver/helper")
        for (package in names(sources)) {
            if (!file.copy(file.path(sources[[package]], "DESCRIPTION"), file.path(metadata, paste0(package, "-source-DESCRIPTION")))) stop("cannot retain source description")
        }
        writeLines(capture.output(sessionInfo()), file.path(metadata, "session.txt"))
        packages <- utils::installed.packages()[, c("Package", "Version", "LibPath"), drop=FALSE]
        write.table(packages[order(packages[, "Package"], method="radix"), , drop=FALSE],
            file.path(metadata, "packages.tsv"), sep="\t", row.names=FALSE, quote=TRUE)
        runtime <- list(R=as.character(getRversion()), Bioconductor=as.character(BiocManager::version()),
            platform=R.version$platform, system=as.list(Sys.info()), external_software=as.list(extSoftVersion()),
            BLAS_LAPACK=capture.output(La_library()), locale=Sys.getlocale(),
            edgeR=edger_runtime_package("edgeR", metadata), limma=edger_runtime_package("limma", metadata),
            thread_environment=as.list(Sys.getenv(c("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"))))
        edger_json(runtime, file.path(metadata, "runtime.json"))
        input_pins <- c(pasilla_gene_counts.tsv.gz="4b5570008936786da365d110df1c83c8cffc646d6c667d6b15ecbfb3d9ee735d",
            pasilla_sample_annotation.csv="1844f871ca5d750f41685c73b87bd7dda291ec0d52304c1afb46a17ae88db745")
        for (name in names(input_pins)) {
            original <- file.path(data_source, "inst/extdata", name)
            if (!identical(deseq_hash(original), unname(input_pins[[name]])) ||
                !file.copy(original, file.path(root, "inputs", name))) stop("original shipped data differs: ", name)
        }
        counts <- as.matrix(read.delim(file.path(root, "inputs/pasilla_gene_counts.tsv.gz"), row.names="gene_id", check.names=FALSE))
        annotation <- read.csv(file.path(root, "inputs/pasilla_sample_annotation.csv"), row.names=1L, check.names=FALSE)
        mapped <- sub("fb$", "", rownames(annotation))
        if (!identical(dim(counts), c(14599L, 7L)) || !is.integer(counts) || sum(rowSums(counts) == 0) != 2240L ||
            anyDuplicated(rownames(counts)) || anyDuplicated(colnames(counts)) || anyDuplicated(mapped) ||
            !setequal(mapped, colnames(counts))) stop("real input counts/metadata identities differ")
        deseq_write_table(data.frame(original_id=rownames(annotation), sample=mapped), file.path(root, "inputs/sample-id-mapping.tsv"))
        rownames(annotation) <- mapped
        samples <- annotation[colnames(counts), c("condition", "type"), drop=FALSE]
        samples$condition <- factor(samples$condition, levels=c("untreated", "treated"))
        samples$type <- factor(samples$type, levels=c("paired-end", "single-read"))
        if (anyNA(samples)) stop("unknown input factor level")
        design <- stats::model.matrix(~type+condition, data=samples)
        if (!identical(dim(design), c(7L, 3L)) || qr(design)$rank != 3L) stop("source design is not full rank")
        forward <- stats::setNames(rep(0, ncol(design)), colnames(design))
        forward[["conditiontreated"]] <- 1
        contrasts <- list(forward=forward, reverse=-forward)
        parameters <- list(construction=list(lib.size=NULL, norm.factors=NULL, group=NULL, genes=NULL, remove.zeros=FALSE),
            formula="~type+condition", levels=list(type=array(levels(samples$type)), condition=array(levels(samples$condition))),
            filtering=list(group=NULL, explicit_design=TRUE, min.count=10, min.total.count=15, large.n=10, min.prop=0.7),
            subset=list(keep.lib.sizes=FALSE), normalization=list(method="TMM", refColumn=NULL, logratioTrim=0.3,
                sumTrim=0.05, doWeighting=TRUE, Acutoff=-1e10, p=0.75),
            ql=list(dispersion=NULL, abundance.trend=TRUE, robust=FALSE, winsor.tail.p=c(0.05, 0.1),
                legacy=FALSE, top.proportion=NULL, keep.unit.mat=FALSE, prior.count=0.125,
                glm_solver="glmFit.default general Levenberg, maxit=250, tol=1e-6"),
            results=list(contrast_alignment="all model columns named and reordered to design", poisson.bound_requested=TRUE,
                poisson.bound_effective=FALSE, topTags=list(n=Inf, sort.by="none", adjust.method="BH", p.value=1)),
            exports=list(CPM="observed cpm(DGEList), normalized.lib.sizes=TRUE, log=FALSE",
                logCPM="observed cpm(DGEList), normalized.lib.sizes=TRUE, log=TRUE, prior.count=2",
                AveLogCPM="held fit$AveLogCPM from the original DGEList aveLogCPM default call, not recomputed"),
            reload="full saved DGEList+DGEGLM; contrast-specific null GLMs allowed, no repeat filter/TMM/full QL fit")
        deseq_write_table(counts, file.path(root, "inputs/counts.tsv"), rownames(counts))
        deseq_write_table(samples, file.path(root, "inputs/samples.tsv"), rownames(samples), "sample")
        deseq_write_table(design, file.path(root, "inputs/model-matrix.tsv"), rownames(design), "sample")
        deseq_write_table(do.call(cbind, contrasts), file.path(root, "inputs/contrasts.tsv"), colnames(design), "coefficient")
        edger_json(parameters, file.path(root, "inputs/public-call-parameters.json"))
        frozen <- sort(list.files(file.path(root, "inputs"), full.names=TRUE), method="radix")
        frozen_hashes <- stats::setNames(vapply(frozen, deseq_hash, character(1)), basename(frozen))
        edger_json(list(status="original inputs, identity joins, levels, design, contrasts and options frozen before fitting",
            files=as.list(frozen_hashes)), file.path(metadata, "input-freeze.json"))
        raw <- observe("DGEList", edgeR::DGEList(counts=counts, samples=samples))
        keep <- observe("filterByExpr explicit design", edgeR::filterByExpr(raw, design=design))
        filtered <- observe("subset recompute library sizes", raw[keep, , keep.lib.sizes=FALSE])
        normalized <- observe("TMM normalization", edgeR::normLibSizes(filtered, method="TMM"))
        fit <- observe("current glmQLFit", edgeR::glmQLFit(normalized, design=design, dispersion=NULL,
            abundance.trend=TRUE, robust=FALSE, winsor.tail.p=c(0.05, 0.1), legacy=FALSE,
            top.proportion=NULL, keep.unit.mat=FALSE))
        bundle <- list(schema_version=1L, packages=pins, input=list(counts=counts, samples=samples, annotation=annotation),
            raw=raw, keep=keep, filtered=filtered, normalized=normalized, design=design, contrasts=contrasts,
            parameters=parameters, fit=fit)
        saveRDS(bundle, file.path(root, "fit/analysis-bundle.rds"))
        saveRDS(fit, file.path(root, "fit/DGEGLM.rds"))
        genes <- rownames(fit$counts)
        n <- length(genes)
        row_map <- match(rownames(counts), genes)
        deseq_write_table(data.frame(keep=keep, status=ifelse(keep, "kept", "filtered"), fitted_row=row_map),
            file.path(root, "fit/filter-mask.tsv"), rownames(counts))
        deseq_write_table(data.frame(raw_library_size=raw$samples$lib.size, postfilter_library_size=filtered$samples$lib.size,
            TMM_factor=normalized$samples$norm.factors, effective_library_size=normalized$samples$lib.size * normalized$samples$norm.factors,
            log_offset=edgeR::getOffset(normalized)), file.path(root, "fit/library-state.tsv"), colnames(counts), "sample")
        deseq_write_table(fit$counts, file.path(root, "fit/kept-counts.tsv"), genes)
        deseq_write_table(fit$coefficients, file.path(root, "fit/shrunken-coefficients.tsv"), genes)
        deseq_write_table(fit$unshrunk.coefficients, file.path(root, "fit/unshrunk-coefficients.tsv"), genes)
        deseq_write_table(fit$fitted.values, file.path(root, "fit/fitted-means.tsv"), genes)
        offsets <- as.matrix(fit$offset)
        colnames(offsets) <- colnames(counts)
        deseq_write_table(offsets, file.path(root, "fit/offset-matrix.tsv"), genes)
        fields <- c("AveLogCPM", "dispersion", "average.ql.dispersion", "deviance", "df.residual", "deviance.adj",
            "df.residual.adj", "s2.prior", "s2.post", "df.prior", "iter", "failed")
        row_data <- lapply(fields, function(name) edger_row_vector(fit[[name]], n))
        names(row_data) <- fields
        row_data$working.NB.dispersion <- edger_row_vector(fit$dispersion / fit$average.ql.dispersion, n)
        row_data$raw.QL.dispersion <- ifelse(fit$df.residual.adj < 1e-4, 0, fit$deviance.adj / fit$df.residual.adj)
        deseq_write_table(as.data.frame(row_data, check.names=FALSE), file.path(root, "fit/row-diagnostics.tsv"), genes)
        edger_json(list(raw_deviance="final working-NB fit$deviance, not the hidden first-pass NB fit",
            working_NB_dispersion="fit$dispersion / fit$average.ql.dispersion",
            raw_QL_dispersion="source src/ql_glm.c: df.adj < 1e-4 -> 0, otherwise deviance.adj/df.adj",
            actual_fields=array(names(fit)), top_proportion=fit$top.proportion,
            iterations="source-reported diagnostics; no cross-BLAS exact gate",
            convergence="failed/iter retained without fabrication; release C cap-exit status is not an independent convergence proof",
            all_QL_state_in_RDS=TRUE), file.path(root, "fit/state-semantics.json"))
        direct <- edger_derive(bundle, observe, "initial")
        edger_write_derived(direct, bundle, file.path(root, "results"))
        reloaded <- readRDS(file.path(root, "fit/analysis-bundle.rds"))
        before <- serialize(reloaded, NULL, version=3)
        after_reload <- edger_derive(reloaded, observe, "reload")
        edger_write_derived(after_reload, reloaded, file.path(root, "reload"))
        checks <- list(input_order=identical(rownames(fit$counts), rownames(counts)[keep]),
            count_values=identical(fit$counts, counts[keep, , drop=FALSE]),
            design=identical(fit$design, design), full_reload_state=identical(bundle, reloaded),
            reload_state_unchanged=identical(before, serialize(reloaded, NULL, version=3)),
            original_state_unchanged=identical(bundle, readRDS(file.path(root, "fit/analysis-bundle.rds"))),
            derived_state=identical(direct, after_reload),
            library_recomputation=identical(unname(filtered$samples$lib.size), unname(colSums(filtered$counts))),
            modern_QL=identical(fit$method, "levenberg") && is.null(fit$df.residual.zeros) && !is.null(fit$df.residual.adj),
            source_reported_failed=sum(fit$failed != FALSE), source_reported_iteration_range=range(fit$iter),
            finite_fits=all(is.finite(c(fit$coefficients, fit$unshrunk.coefficients, fit$fitted.values,
                fit$deviance, fit$df.residual, fit$deviance.adj, fit$df.residual.adj, fit$s2.prior, fit$s2.post,
                fit$dispersion, fit$average.ql.dispersion, fit$AveLogCPM))),
            prior_df_valid=all(!is.na(fit$df.prior) & fit$df.prior >= 0),
            finite_results=all(vapply(direct$tests, function(value) all(is.finite(as.matrix(value$tags$table))), logical(1))),
            finite_exports=all(is.finite(c(direct$observed_cpm, direct$observed_log_cpm, direct$held_AveLogCPM))))
        comparison_files <- sort(list.files(file.path(root, "results"), pattern="\\.tsv$"), method="radix")
        checks$reload_TSV_bytes <- all(vapply(comparison_files, function(name) {
            identical(deseq_hash(file.path(root, "results", name)), deseq_hash(file.path(root, "reload", name)))
        }, logical(1)))
        checks$frozen_inputs <- identical(frozen_hashes, stats::setNames(vapply(frozen, deseq_hash, character(1)), names(frozen_hashes)))
        edger_json(list(checks=checks, original_genes=nrow(counts), kept_genes=n, filtered_genes=sum(!keep),
            samples=ncol(counts), design_columns=ncol(design), filter_hat=stats::hat(design),
            filter_effective_min_sample_size=1/max(stats::hat(design)), actual_prior_df=fit$df.prior,
            significant_FDR_0_05=lapply(direct$tests, function(value) sum(value$tags$table$FDR <= 0.05)),
            prior_df_infinite=sum(is.infinite(fit$df.prior)), no_estimates_for_filtered_rows=TRUE), file.path(root, "summary.json"))
        gates <- checks[setdiff(names(checks), c("source_reported_failed", "source_reported_iteration_range"))]
        if (!all(vapply(gates, isTRUE, logical(1))) || anyNA(fit$failed) || any(fit$failed) ||
            anyNA(fit$iter) || any(fit$iter < 1L | fit$iter > 250L)) stop("source workflow state, output or reported solver checks failed")
        status <- "complete"
        cat("Actual current edgeR QL workflow, complete fitted state and reload contrasts/exports captured\n")
    }, error=function(error) {
        failure <<- list(message=conditionMessage(error), class=array(class(error)), call=paste(deparse(conditionCall(error)), collapse=" "))
        edger_json(failure, file.path(metadata, "failure.json"))
        stop(error)
    })
}

if (sys.nframe() == 0L) edger_workflow_main()
