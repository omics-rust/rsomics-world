#!/usr/bin/env Rscript

limma_source_identity <- function(source, package, pin, output) {
    shared <- edger_source_identity(source, package, pin, output)
    if (package == "edgeR") {
        files <- c("R/voomLmFit.R"="5f44d088c8f09d580845a9f74f63cbb0ae70298d488ba3b248082affb1a92a35",
            "R/effectiveLibSizes.R"="51a74655823e12e282d0300214c645e7fa08069b0034104cd7cc027a7accc4e6")
        functions <- c(voomLmFit="R/voomLmFit.R", getNormLibSizes="R/effectiveLibSizes.R",
            getNormLibSizes.DGEList="R/effectiveLibSizes.R")
    } else {
        files <- c("R/lmfit.R"="4a3e2a37c58b5e8ef5a82c6aefa39db6a8336cae34dcfe3db0c4b0ae166b40c1",
            "R/contrasts.R"="e53dc9ae12fe9042309381ded7585d99abde28b29ba5948507db6327b5f66853",
            "R/ebayes.R"="26ce27167f85523f1a8916df5de8a6c029b3e5450b816c710a4ab284f93ef590",
            "R/decidetests.R"="ad3e838939d608ac9955498ccc7342ef32e8af828a6334f677d9272b67c1fa9b",
            "R/fitFDist.R"="c2538746ccf7220be379bc4b592da133c8b1beaea56ac536077da3ceba707af4",
            "R/norm.R"="98914d8c485164afdf95c4b0881ae48e95591e3edcf5ae9543e2d8b1f0544c61",
            "R/toptable.R"="ccbbe4104ffe9d1fff7d4cab27601d9257d140227289df376f72f9d4c8a8e1dc")
        functions <- c(lmFit="R/lmfit.R", lm.series="R/lmfit.R", getEAWP="R/lmfit.R",
            is.fullrank="R/lmfit.R", nonEstimable="R/lmfit.R", fitted.MArrayLM="R/lmfit.R",
            residuals.MArrayLM="R/lmfit.R", contrasts.fit="R/contrasts.R", eBayes="R/ebayes.R",
            .ebayes="R/ebayes.R", tmixture.matrix="R/ebayes.R", tmixture.vector="R/ebayes.R",
            fitFDist="R/fitFDist.R", normalizeBetweenArrays="R/norm.R",
            topTable="R/toptable.R", .topTableT="R/toptable.R", classifyTestsF="R/decidetests.R")
    }
    for (name in unique(c(names(files), unname(functions)))) {
        if (!basename(name) %in% list.files(file.path(source, dirname(name)))) stop("source member case differs: ", name)
    }
    for (name in names(files)) {
        if (!identical(deseq_hash(file.path(source, name)), unname(files[[name]]))) stop("source differs from release: ", package, "/", name)
    }
    for (name in names(functions)) {
        expected <- edger_definition(file.path(source, functions[[name]]), name)
        installed <- get(name, asNamespace(package), inherits=FALSE)
        if (!identical(deparse(formals(installed)), deparse(expected[[2L]])) ||
            !identical(deparse(body(installed)), deparse(expected[[3L]]))) stop("installed function differs from release: ", package, "::", name)
        writeLines(c("formals:", deparse(formals(installed)), "body:", deparse(body(installed))),
            file.path(output, paste0(package, "-function-", name, ".txt")))
    }
    list(shared=shared, source_files=as.list(files), checked_functions=as.list(functions))
}

limma_shape <- function(value) {
    result <- list(class=array(class(value)), length=length(value), dimensions=dim(value),
        names=names(value), dimnames=dimnames(value))
    if (is.numeric(value)) result$locations <- list(indexing="1-based R column-major",
        zero=array(which(value == 0)), "NA"=array(which(is.na(value) & !is.nan(value))),
        "NaN"=array(which(is.nan(value))), positive_infinity=array(which(value == Inf)),
        negative_infinity=array(which(value == -Inf)))
    result
}

limma_write_fit <- function(fit, directory) {
    if (!dir.create(directory)) stop("cannot create fit export directory")
    genes <- rownames(fit$coefficients)
    saveRDS(fit, file.path(directory, "complete-MArrayLM.rds"))
    for (name in c("coefficients", "stdev.unscaled", "t", "p.value", "lods")) {
        if (!is.null(fit[[name]])) deseq_write_table(fit[[name]], file.path(directory, paste0(name, ".tsv")), genes)
    }
    if (!is.null(fit$cov.coefficients)) deseq_write_table(fit$cov.coefficients,
        file.path(directory, "cov.coefficients.tsv"), rownames(fit$cov.coefficients), "coefficient")
    fields <- c("Amean", "sigma", "df.residual", "s2.prior", "df.prior", "s2.post", "df.total", "F", "F.p.value")
    fields <- fields[vapply(fields, function(name) !is.null(fit[[name]]), logical(1))]
    rows <- stats::setNames(lapply(fields, function(name) edger_row_vector(fit[[name]], length(genes))), fields)
    deseq_write_table(as.data.frame(rows, check.names=FALSE), file.path(directory, "row-diagnostics.tsv"), genes)
    edger_json(list(fields=array(names(fit)), shapes=lapply(fit, limma_shape), rank=fit$rank,
        pivot=fit$pivot, method=fit$method, var.prior=fit$var.prior, proportion=fit$proportion),
        file.path(directory, "state-descriptors.json"))
}

limma_derive <- function(bundle, observe, label) {
    tests <- lapply(names(bundle$contrasts), function(name) {
        contrast <- matrix(bundle$contrasts[[name]], ncol=1L,
            dimnames=list(colnames(bundle$design), name))
        fit <- observe(paste(label, "contrasts.fit", name), limma::contrasts.fit(bundle$base, contrasts=contrast))
        moderated <- observe(paste(label, "eBayes", name), limma::eBayes(fit))
        table <- observe(paste(label, "topTable", name), limma::topTable(moderated, coef=1L,
            number=Inf, sort.by="none", adjust.method="BH", p.value=1, lfc=0, confint=FALSE))
        list(contrast=fit, moderated=moderated, table=table)
    })
    names(tests) <- names(bundle$contrasts)
    list(tests=tests, logCPM=bundle$base$EList$E, weights=bundle$base$EList$weights,
        fitted=observe(paste(label, "fitted base model"), stats::fitted(bundle$base)),
        residuals=observe(paste(label, "residuals base model"), stats::residuals(bundle$base, bundle$base$EList$E)))
}

limma_write_derived <- function(derived, bundle, directory) {
    genes <- rownames(bundle$base$coefficients)
    for (name in names(derived$tests)) {
        value <- derived$tests[[name]]
        if (!identical(rownames(value$table), genes)) stop("result feature order differs")
        deseq_write_table(value$table, file.path(directory, paste0(name, ".tsv")), genes)
        limma_write_fit(value$contrast, file.path(directory, paste0(name, "-contrast")))
        limma_write_fit(value$moderated, file.path(directory, paste0(name, "-moderated")))
    }
    for (name in c("logCPM", "weights", "fitted", "residuals")) {
        value <- derived[[name]]
        if (!identical(dim(value), dim(bundle$normalized$counts))) stop("observation export dimensions differ")
        colnames(value) <- colnames(bundle$normalized$counts)
        deseq_write_table(value, file.path(directory, paste0(name, ".tsv")), genes)
    }
    saveRDS(derived, file.path(directory, "complete-derived-state.rds"))
}

limma_workflow_main <- function() {
    args <- commandArgs(trailingOnly=TRUE)
    if (length(args) != 4L) stop("usage: limma_workflow_oracle.R EDGER_SOURCE LIMMA_SOURCE DATA_SOURCE OUTPUT_DIR")
    script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
    if (length(script) != 1L) stop("cannot identify source driver")
    script <- normalizePath(sub("^--file=", "", script), mustWork=TRUE)
    helpers <- file.path(dirname(script), c("deseq_workflow_oracle.R", "edger_workflow_oracle.R"))
    for (helper in helpers) base::source(helper, local=.GlobalEnv)
    for (package in c("edgeR", "limma", "digest", "jsonlite", "BiocManager")) {
        if (!requireNamespace(package, quietly=TRUE)) stop("required installed package absent: ", package)
    }
    pins <- list(edgeR=list(name="edgeR", version="4.10.7", bioconductor="3.23",
        source_commit="27ba7b39d819e68df2ecdea9f32a4b6039b2a0b5",
        source_archive_sha256="a3cc0f66be29f1a61a74021f9848ec940794e653905c4332327a719dd9ad8393"),
        limma=list(name="limma", version="3.68.5", bioconductor="3.23",
        source_commit="825d1c8330d024adb6c54af66d79b0f4ec258530",
        source_archive_sha256="40b1d2059a76ad98db4bf80139760acf8e8bae92a8265aaaf16ac199831533f4"))
    carrier_hash <- "8c91699286336350e66eec132ce6fdf5bb4af78e2a4d015a5a61224f62a95984"
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
    runtime <- source_identity <- parameters <- checks <- coverage <- NULL
    conditions <- list()
    on.exit({
        files <- sort(list.files(root, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
        hashes <- stats::setNames(vapply(file.path(root, files), deseq_hash, character(1)), files)
        witness <- list(schema_version=1L, status=status, packages=pins, runtime=runtime, source=source_identity,
            data_carrier=list(package="DESeq2", version="1.52.0", source_archive_sha256=carrier_hash,
                installed_by_workflow=FALSE, loaded_for_fit=FALSE), parameters=parameters, checks=checks,
            branch_coverage=coverage, failure=failure, conditions=list(count=length(conditions),
                warnings=sum(vapply(conditions, function(value) value$kind == "warning", logical(1))),
                errors=sum(vapply(conditions, function(value) value$kind == "error", logical(1)))),
            nonfinite_encoding="TSV/RDS preserve NA, NaN, Inf, -Inf; JSON nonfinite numeric values use explicit string tokens",
            exclusions=array(c("native implementation or compatibility acceptance", "performance, release or biological claims",
                "log-expression or missing-count input", "sample quality weights, donors, blocks and correlation",
                "prior weights, supplied offsets or library sizes, other normalizations", "robust, trend or forced-legacy moderation",
                "TREAT, public multi-term F, plotting UI, batch removal, splicing, genesets and cross-product bundles")),
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
        if (as.character(getRversion()) != "4.6.1" || as.character(BiocManager::version()) != "3.23") stop("source runtime differs")
        for (package in names(pins)) {
            prefix <- toupper(package)
            pin <- pins[[package]]
            archive <- Sys.getenv(paste0(prefix, "_SOURCE_ARCHIVE"))
            if (as.character(utils::packageVersion(package)) != pin$version || !nzchar(archive) ||
                !identical(deseq_hash(archive), pin$source_archive_sha256) ||
                !identical(Sys.getenv(paste0(prefix, "_SOURCE_SHA256")), pin$source_archive_sha256) ||
                !identical(Sys.getenv(paste0(prefix, "_SOURCE_COMMIT")), pin$source_commit)) stop("source acquisition identity differs: ", package)
        }
        if (!identical(deseq_hash(Sys.getenv("DESEQ2_SOURCE_ARCHIVE")), carrier_hash) ||
            "DESeq2" %in% loadedNamespaces()) stop("raw-data carrier identity or source-only use differs")
        source_identity <- lapply(names(sources), function(package) limma_source_identity(sources[[package]], package, pins[[package]], metadata))
        names(source_identity) <- names(sources)
        for (path in c(script, helpers)) if (!file.copy(path, file.path(metadata, basename(path)))) stop("cannot retain driver/helper")
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
        if (!identical(dim(counts), c(14599L, 7L)) || !is.integer(counts) || anyDuplicated(rownames(counts)) ||
            anyDuplicated(colnames(counts)) || anyDuplicated(mapped) || !setequal(mapped, colnames(counts))) stop("input identities differ")
        deseq_write_table(data.frame(original_id=rownames(annotation), sample=mapped), file.path(root, "inputs/sample-id-mapping.tsv"))
        rownames(annotation) <- mapped
        samples <- annotation[colnames(counts), c("condition", "type"), drop=FALSE]
        samples$condition <- factor(samples$condition, levels=c("untreated", "treated"))
        samples$type <- factor(samples$type, levels=c("paired-end", "single-read"))
        if (anyNA(samples)) stop("unknown input factor level")
        formula <- stats::as.formula("~type+condition", env=baseenv())
        design <- stats::model.matrix(formula, data=samples)
        terms <- stats::terms(formula, data=samples)
        if (!identical(dim(design), c(7L, 3L)) || qr(design)$rank != 3L) stop("source design is not full rank")
        forward <- stats::setNames(c(0, 0, 1), colnames(design))
        contrasts <- list(forward=forward, reverse=-forward, mixed=stats::setNames(c(0, -1, 1), colnames(design)))
        parameters <- list(construction=list(lib.size=NULL, norm.factors=NULL, group=NULL, genes=NULL, remove.zeros=FALSE),
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
        deseq_write_table(counts, file.path(root, "inputs/counts.tsv"), rownames(counts))
        deseq_write_table(samples, file.path(root, "inputs/samples.tsv"), rownames(samples), "sample")
        deseq_write_table(design, file.path(root, "inputs/model-matrix.tsv"), rownames(design), "sample")
        deseq_write_table(attr(terms, "factors"), file.path(root, "inputs/terms-factors.tsv"), rownames(attr(terms, "factors")), "variable")
        deseq_write_table(do.call(cbind, contrasts), file.path(root, "inputs/contrasts.tsv"), colnames(design), "coefficient")
        for (name in names(samples)) deseq_write_table(stats::contrasts(samples[[name]]),
            file.path(root, "inputs", paste0(name, "-coding.tsv")), levels(samples[[name]]), "level")
        edger_json(parameters, file.path(root, "inputs/public-call-parameters.json"))
        saveRDS(list(counts=counts, samples=samples, annotation=annotation, design=design, terms=terms,
            contrasts=contrasts, parameters=parameters), file.path(root, "inputs/prefit-state.rds"))
        frozen <- sort(list.files(file.path(root, "inputs"), full.names=TRUE), method="radix")
        frozen_hashes <- stats::setNames(vapply(frozen, deseq_hash, character(1)), basename(frozen))
        edger_json(list(status="original inputs, mapping, design, coding, contrasts and options frozen before fitting",
            files=as.list(frozen_hashes)), file.path(metadata, "input-freeze.json"))
        raw <- observe("DGEList", edgeR::DGEList(counts=counts, samples=samples))
        keep <- observe("filterByExpr explicit design", edgeR::filterByExpr(raw, design=design))
        filtered <- observe("subset recompute library sizes", raw[keep, , keep.lib.sizes=FALSE])
        normalized <- observe("TMM normalization", edgeR::normLibSizes(filtered, method="TMM"))
        base <- observe("current voomLmFit", edgeR::voomLmFit(normalized, design=design, save.plot=TRUE, keep.EList=TRUE))
        bundle <- list(schema_version=1L, packages=pins, input=list(counts=counts, samples=samples, annotation=annotation),
            raw=raw, keep=keep, filtered=filtered, normalized=normalized, design=design, terms=terms,
            contrasts=contrasts, parameters=parameters, base=base)
        direct <- limma_derive(bundle, observe, "initial")
        bundle$contrast <- direct$tests$forward$contrast
        bundle$moderated <- direct$tests$forward$moderated
        bundle_path <- file.path(root, "fit/analysis-bundle.rds")
        saveRDS(bundle, bundle_path)
        bundle_hash <- deseq_hash(bundle_path)
        limma_write_fit(base, file.path(root, "fit/base"))
        genes <- rownames(base$coefficients)
        deseq_write_table(data.frame(keep=keep, status=ifelse(keep, "kept", "filtered"), fitted_row=match(rownames(counts), genes)),
            file.path(root, "fit/filter-mask.tsv"), rownames(counts))
        deseq_write_table(normalized$counts, file.path(root, "fit/kept-counts.tsv"), genes)
        deseq_write_table(data.frame(raw_library_size=raw$samples$lib.size, postfilter_library_size=filtered$samples$lib.size,
            TMM_factor=normalized$samples$norm.factors, effective_library_size=edgeR::getNormLibSizes(normalized),
            log_offset=edgeR::getOffset(normalized)), file.path(root, "fit/library-state.tsv"), colnames(counts), "sample")
        deseq_write_table(base$targets, file.path(root, "fit/targets.tsv"), rownames(base$targets), "sample")
        deseq_write_table(base$design, file.path(root, "fit/design.tsv"), rownames(base$design), "sample")
        deseq_write_table(data.frame(x=base$voom.xy$x, y=base$voom.xy$y), file.path(root, "fit/voom-points.tsv"),
            seq_along(base$voom.xy$x), "point")
        deseq_write_table(data.frame(x=base$voom.line$x, y=base$voom.line$y), file.path(root, "fit/voom-line.tsv"),
            seq_along(base$voom.line$x), "point")
        edger_json(list(derived_span=limma::chooseLowessSpan(nrow(normalized$counts), small.n=50, min.span=0.3, power=1/3),
            span_returned_by_fit=FALSE, targets_lib_size="post-filter raw library size, not TMM effective size",
            E="source voom log2((count+.5)/(effective library size+1)*1e6), not ordinary edgeR logCPM",
            plot_point_identity="source voom.xy/line supplies coordinates without feature IDs; retain point indices",
            sigma_and_df="structural-zero correction changes sigma/df only; NA sigma at zero df is retained",
            diagnostics="source-returned fields only; no invented convergence/iteration fields",
            observations=list(logCPM=limma_shape(direct$logCPM), weights=limma_shape(direct$weights))),
            file.path(root, "fit/state-semantics.json"))
        limma_write_derived(direct, bundle, file.path(root, "results"))
        reloaded <- readRDS(bundle_path)
        before <- serialize(reloaded, NULL, version=3)
        after_reload <- limma_derive(reloaded, observe, "reload")
        limma_write_derived(after_reload, reloaded, file.path(root, "reload"))
        comparison_files <- sort(list.files(file.path(root, "results"), pattern="\\.tsv$", recursive=TRUE), method="radix")
        checks <- list(original_count_values=identical(raw$counts, counts), kept_count_values=identical(normalized$counts, counts[keep, , drop=FALSE]),
            input_order=identical(genes, rownames(counts)[keep]), design=identical(base$design, design),
            targets_order=identical(rownames(base$targets), colnames(counts)),
            library_recomputed=identical(unname(filtered$samples$lib.size), unname(colSums(filtered$counts))),
            full_reload_state=identical(bundle, reloaded), reload_state_unchanged=identical(before, serialize(reloaded, NULL, version=3)),
            original_state_unchanged=identical(bundle, readRDS(bundle_path)), bundle_bytes_unchanged=identical(bundle_hash, deseq_hash(bundle_path)),
            derived_state=identical(direct, after_reload), no_invented_block_or_sample_weights=is.null(base$block) &&
                is.null(base$correlation) && is.null(base$targets$sample.weight), data_carrier_not_loaded=!"DESeq2" %in% loadedNamespaces(),
            reload_TSV_bytes=all(vapply(comparison_files, function(name) identical(deseq_hash(file.path(root, "results", name)),
                deseq_hash(file.path(root, "reload", name))), logical(1))),
            frozen_inputs=identical(frozen_hashes, stats::setNames(vapply(frozen, deseq_hash, character(1)), names(frozen_hashes))))
        positive_df <- base$df.residual[base$df.residual > 0]
        observed <- list(structural_zero_residual_df_loss=any(base$df.residual < nrow(design)-base$rank),
            unequal_positive_residual_df=length(unique(positive_df)) > 1L)
        coverage <- list(observed=observed, uncovered=array(names(observed)[!unlist(observed)]),
            selected_moderation=if (length(unique(positive_df)) == 1L) "fitFDist legacy equal-positive-df" else "fitFDistUnequalDF1",
            excluded=array(c("sample weights", "donor/block/correlation", "missing-count input")),
            policy="unobserved branches remain uncovered; no input changes or broader acceptance claim")
        edger_json(list(checks=checks, branch_coverage=coverage, original_genes=nrow(counts), kept_genes=length(genes),
            filtered_genes=sum(!keep), raw_all_zero_genes=sum(rowSums(counts) == 0), samples=ncol(counts),
            design_columns=ncol(design), residual_df_distribution=as.list(table(base$df.residual, useNA="ifany")),
            nonfinite_and_zero_locations=list(sigma=limma_shape(base$sigma), df.residual=limma_shape(base$df.residual)),
            significant_FDR_0_05=lapply(direct$tests, function(value) sum(value$table$adj.P.Val <= 0.05, na.rm=TRUE)),
            no_estimates_for_filtered_rows=TRUE), file.path(root, "summary.json"))
        if (!all(vapply(checks, isTRUE, logical(1)))) stop("source fitted state, reload or export checks failed")
        status <- "complete"
        cat("Actual current limma voom workflow, full base fit, moderation and reload exports captured\n")
    }, error=function(error) {
        failure <<- list(message=conditionMessage(error), class=array(class(error)), call=paste(deparse(conditionCall(error)), collapse=" "))
        edger_json(failure, file.path(metadata, "failure.json"))
        stop(error)
    })
}

if (sys.nframe() == 0L) limma_workflow_main()
