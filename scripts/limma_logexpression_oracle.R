#!/usr/bin/env Rscript

logexpression_observer <- function(directory) {
    index <- 0L
    function(label, expression) {
        note <- function(condition, kind) {
            index <<- index + 1L
            row <- data.frame(index=index, operation=label, kind=kind,
                class=paste(class(condition), collapse=","), message=conditionMessage(condition),
                call=paste(deparse(conditionCall(condition)), collapse=" "))
            write.table(row, file.path(directory, "conditions.tsv"), sep="\t", quote=TRUE,
                row.names=FALSE, col.names=index == 1L, append=index > 1L)
        }
        withCallingHandlers(force(expression), warning=function(value) note(value, "warning"),
            message=function(value) note(value, "message"), error=function(value) note(value, "error"))
    }
}

logexpression_attempt <- function(expression) {
    tryCatch(list(status="returned", value=force(expression)), error=function(error) {
        list(status="failed", failure=list(class=array(class(error)), message=conditionMessage(error),
            call=paste(deparse(conditionCall(error)), collapse=" ")))
    })
}

logexpression_manifest <- function(root) {
    files <- sort(list.files(root, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
    files <- setdiff(files, "sha256-manifest.tsv")
    write.table(data.frame(sha256=vapply(file.path(root, files), deseq_hash, character(1)), path=files),
        file.path(root, "sha256-manifest.tsv"), sep="\t", quote=FALSE, row.names=FALSE)
}

logexpression_identity <- function(source, metadata) {
    pin <- list(name="limma", version="3.68.5", bioconductor="3.23",
        source_commit="825d1c8330d024adb6c54af66d79b0f4ec258530",
        source_archive_sha256="40b1d2059a76ad98db4bf80139760acf8e8bae92a8265aaaf16ac199831533f4")
    for (package in c("limma", "digest", "jsonlite", "BiocManager")) {
        if (!requireNamespace(package, quietly=TRUE)) stop("required package absent: ", package)
    }
    if (as.character(getRversion()) != "4.6.1" || as.character(BiocManager::version()) != "3.23" ||
        as.character(utils::packageVersion("limma")) != pin$version) stop("source runtime differs")
    if (!identical(Sys.getenv("LIMMA_SOURCE_COMMIT"), pin$source_commit) ||
        !identical(Sys.getenv("LIMMA_SOURCE_SHA256"), pin$source_archive_sha256) ||
        !identical(deseq_hash(Sys.getenv("LIMMA_SOURCE_ARCHIVE")), pin$source_archive_sha256)) stop("source archive identity differs")
    library <- normalizePath(Sys.getenv("R_LIBS_USER"), mustWork=TRUE)
    installed <- normalizePath(system.file(package="limma", mustWork=TRUE), mustWork=TRUE)
    if (!startsWith(installed, paste0(library, "/"))) stop("limma was not loaded from the exact installed user library")
    source_identity <- limma_source_identity(source, "limma", pin, metadata)
    expected <- edger_definition(file.path(source, "R/fitFDist.R"), "trigammaInverse")
    installed_function <- get("trigammaInverse", asNamespace("limma"), inherits=FALSE)
    if (!identical(deparse(formals(installed_function)), deparse(expected[[2L]])) ||
        !identical(deparse(body(installed_function)), deparse(expected[[3L]]))) stop("installed trigammaInverse differs from release")
    writeLines(c("formals:", deparse(formals(installed_function)), "body:", deparse(body(installed_function))),
        file.path(metadata, "limma-function-trigammaInverse.txt"))
    source_identity$extra_checked_function <- "trigammaInverse from R/fitFDist.R"
    writeLines(capture.output(sessionInfo()), file.path(metadata, "session.txt"))
    packages <- utils::installed.packages()[, c("Package", "Version", "LibPath"), drop=FALSE]
    write.table(packages[order(packages[, "Package"], method="radix"), , drop=FALSE],
        file.path(metadata, "packages.tsv"), sep="\t", quote=TRUE, row.names=FALSE)
    stats_fit <- stats::lm.fit
    stats_path <- file.path(metadata, "stats-lm.fit.txt")
    writeLines(c("formals:", deparse(formals(stats_fit)), "body:", deparse(body(stats_fit))), stats_path)
    runtime <- list(R=as.character(getRversion()), Bioconductor=as.character(BiocManager::version()),
        platform=R.version$platform, system=as.list(Sys.info()), external_software=as.list(extSoftVersion()),
        BLAS_LAPACK=capture.output(La_library()), locale=Sys.getlocale(),
        limma=edger_runtime_package("limma", metadata),
        stats_lm_fit=list(package_version=as.character(utils::packageVersion("stats")),
            installed_definition="stats-lm.fit.txt", definition_sha256=deseq_hash(stats_path),
            qualification="installed function captured under exact R4.6.1; no R source-tag comparison"),
        thread_environment=as.list(Sys.getenv(c("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"))))
    edger_json(runtime, file.path(metadata, "runtime.json"))
    list(pin=pin, source=source_identity, runtime=runtime)
}

logexpression_inputs <- function(directory) {
    read <- function(name) read.delim(file.path(directory, paste0(name, ".tsv")), row.names=1L,
        check.names=FALSE, stringsAsFactors=FALSE)
    expression <- as.matrix(read("expression"))
    design <- as.matrix(read("design"))
    samples <- read("samples")
    contrasts <- as.matrix(read("contrasts"))
    unique_ids <- function(ids) !is.null(ids) && length(ids) > 0L && !anyNA(ids) &&
        all(nzchar(ids)) && !anyDuplicated(ids)
    if (!is.numeric(expression) || !all(is.finite(expression)) || !unique_ids(rownames(expression)) ||
        !unique_ids(colnames(expression))) stop("invalid finite named supplied expression")
    ids <- colnames(expression)
    if (!is.numeric(design) || !all(is.finite(design)) || !unique_ids(rownames(design)) ||
        !unique_ids(colnames(design)) || !setequal(rownames(design), ids) ||
        !unique_ids(rownames(samples)) || !setequal(rownames(samples), ids)) stop("input identities or design differ")
    design <- design[ids, , drop=FALSE]
    samples <- samples[ids, , drop=FALSE]
    if (nrow(design) <= ncol(design) || qr(design)$rank != ncol(design)) stop("design must be full rank with n>p")
    if (!is.numeric(contrasts) || !all(is.finite(contrasts)) || !unique_ids(rownames(contrasts)) ||
        !unique_ids(colnames(contrasts)) || !setequal(rownames(contrasts), colnames(design)) ||
        any(colSums(abs(contrasts)) == 0)) stop("contrasts must completely name the design and be nonzero")
    contrasts <- contrasts[colnames(design), , drop=FALSE]
    named <- lapply(seq_len(ncol(contrasts)), function(index) contrasts[, index])
    names(named) <- colnames(contrasts)
    list(input_kind="log-expression", expression=expression, design=design, samples=samples, contrasts=named)
}

logexpression_derive <- function(bundle, observe, label) {
    tests <- lapply(names(bundle$contrasts), function(name) {
        contrast <- matrix(bundle$contrasts[[name]], ncol=1L,
            dimnames=list(colnames(bundle$design), name))
        fitted <- logexpression_attempt(observe(paste(label, "contrasts.fit", name),
            limma::contrasts.fit(bundle$base, contrasts=contrast)))
        if (fitted$status == "failed") return(list(contrast=fitted))
        moderated <- logexpression_attempt(observe(paste(label, "eBayes", name), limma::eBayes(fitted$value)))
        if (moderated$status == "failed") return(list(contrast=fitted, moderated=moderated))
        table <- logexpression_attempt(observe(paste(label, "topTable", name),
            limma::topTable(moderated$value, coef=1L, number=Inf, sort.by="none", adjust.method="BH",
                p.value=1, lfc=0, confint=FALSE)))
        list(contrast=fitted, moderated=moderated, table=table)
    })
    names(tests) <- names(bundle$contrasts)
    list(tests=tests, supplied_expression=bundle$expression,
        fitted=observe(paste(label, "fitted base"), stats::fitted(bundle$base)),
        residuals=observe(paste(label, "residuals base"), stats::residuals(bundle$base, bundle$expression)),
        squeezeVar=logexpression_attempt(observe(paste(label, "public squeezeVar diagnostic"),
            limma::squeezeVar(bundle$base$sigma^2, bundle$base$df.residual))))
}

logexpression_write_mapping <- function(value, expression, directory, name) {
    edger_json(list(input_shape=limma_shape(expression), source_shape=limma_shape(value),
        input_feature_ids=array(rownames(expression)), source_feature_ids=rownames(value),
        alignment="positional source-row to input-row mapping; raw source names are not repaired"),
        file.path(directory, paste0(name, "-identity.json")))
    if (nrow(value) != nrow(expression)) stop("source/input feature dimensions differ")
    if (nrow(expression) > 1L && !identical(rownames(value), rownames(expression))) stop("multirow source feature order differs")
    mapping <- data.frame(input_feature_id=rownames(expression),
        source_feature_id=if (is.null(rownames(value))) rep(NA_character_, nrow(value)) else rownames(value))
    deseq_write_table(mapping, file.path(directory, paste0(name, "-input-mapping.tsv")), seq_len(nrow(value)), "source_row")
}

logexpression_write_fit <- function(fit, expression, directory) {
    if (!dir.create(directory)) stop("cannot create ordinary fit export directory")
    saveRDS(fit, file.path(directory, "complete-MArrayLM.rds"))
    edger_json(list(fields=array(names(fit)), shapes=lapply(fit, limma_shape), rank=fit$rank,
        pivot=fit$pivot, method=fit$method, var.prior=fit$var.prior, proportion=fit$proportion),
        file.path(directory, "state-descriptors.json"))
    logexpression_write_mapping(fit$coefficients, expression, directory, "features")
    ids <- rownames(fit$coefficients)
    id_name <- if (is.null(ids)) "source_row" else "source_feature_id"
    if (is.null(ids)) ids <- seq_len(nrow(fit$coefficients))
    for (name in c("coefficients", "stdev.unscaled", "t", "p.value", "lods")) {
        if (!is.null(fit[[name]])) deseq_write_table(fit[[name]], file.path(directory, paste0(name, ".tsv")), ids, id_name)
    }
    if (!is.null(fit$cov.coefficients)) deseq_write_table(fit$cov.coefficients,
        file.path(directory, "cov.coefficients.tsv"), rownames(fit$cov.coefficients), "coefficient")
    fields <- c("Amean", "sigma", "df.residual", "s2.prior", "df.prior", "s2.post", "df.total", "F", "F.p.value")
    fields <- fields[vapply(fields, function(name) !is.null(fit[[name]]), logical(1))]
    rows <- stats::setNames(lapply(fields, function(name) edger_row_vector(fit[[name]], nrow(fit$coefficients))), fields)
    deseq_write_table(as.data.frame(rows, check.names=FALSE), file.path(directory, "row-diagnostics.tsv"), ids, id_name)
}

logexpression_write_derived <- function(derived, bundle, directory) {
    if (!dir.create(directory)) stop("cannot create derived directory")
    saveRDS(derived, file.path(directory, "complete-derived-state.rds"))
    for (name in names(derived$tests)) {
        value <- derived$tests[[name]]
        if (!is.null(value$table) && value$table$status == "returned") saveRDS(value$table$value,
            file.path(directory, paste0(name, "-returned-table.rds")))
    }
    for (name in names(derived$tests)) {
        value <- derived$tests[[name]]
        for (stage in intersect(c("contrast", "moderated"), names(value))) {
            result <- value[[stage]]
            if (result$status == "returned") logexpression_write_fit(result$value, bundle$expression,
                file.path(directory, paste0(name, "-", stage)))
        }
        if (!is.null(value$table) && value$table$status == "returned") {
            logexpression_write_mapping(value$table$value, bundle$expression, directory, name)
            deseq_write_table(value$table$value, file.path(directory, paste0(name, ".tsv")),
                rownames(value$table$value), "source_feature_id")
        }
        edger_json(lapply(value, function(result) result[setdiff(names(result), "value")]),
            file.path(directory, paste0(name, "-status.json")))
    }
    for (name in c("supplied_expression", "fitted", "residuals")) {
        value <- derived[[name]]
        logexpression_write_mapping(value, bundle$expression, directory, name)
        if (!identical(dim(value), dim(bundle$expression)) ||
            !identical(colnames(value), colnames(bundle$expression))) stop("observation export sample identities differ")
        ids <- rownames(value)
        deseq_write_table(value, file.path(directory, paste0(name, ".tsv")),
            if (is.null(ids)) seq_len(nrow(value)) else ids, if (is.null(ids)) "source_row" else "source_feature_id")
    }
    saveRDS(derived$squeezeVar, file.path(directory, "public-squeezeVar-diagnostic.rds"))
    edger_json(list(status=derived$squeezeVar$status,
        values=if (derived$squeezeVar$status == "returned") derived$squeezeVar$value else NULL,
        shapes=if (derived$squeezeVar$status == "returned") lapply(derived$squeezeVar$value, limma_shape) else NULL,
        failure=derived$squeezeVar$failure), file.path(directory, "public-squeezeVar-diagnostic.json"))
}

logexpression_complete <- function(derived) {
    all(vapply(derived$tests, function(value) !is.null(value$table) && value$table$status == "returned", logical(1)))
}

logexpression_case <- function(input, root, parameters, observe) {
    if (!dir.create(root)) stop("cannot create case directory")
    saveRDS(c(input, list(parameters=parameters)), file.path(root, "prefit-state.rds"))
    for (name in c("expression", "design", "samples")) {
        ids <- if (name == "expression") rownames(input$expression) else colnames(input$expression)
        deseq_write_table(input[[name]], file.path(root, paste0("joined-", name, ".tsv")), ids,
            if (name == "expression") "gene" else "sample")
    }
    deseq_write_table(do.call(cbind, input$contrasts), file.path(root, "named-contrasts.tsv"), colnames(input$design), "coefficient")
    frozen <- sort(list.files(root, full.names=TRUE), method="radix")
    hashes <- stats::setNames(vapply(frozen, deseq_hash, character(1)), basename(frozen))
    edger_json(list(status="supplied E, joined samples/design, named contrasts and options frozen before lmFit",
        files=as.list(hashes)), file.path(root, "input-freeze.json"))
    base <- observe("ordinary unweighted lmFit", limma::lmFit(input$expression, input$design,
        method="ls", weights=NULL, block=NULL, ndups=1, spacing=1))
    bundle <- c(input, list(schema_version=1L, parameters=parameters, base=base))
    saveRDS(bundle, file.path(root, "uncontrasted-base-bundle.rds"))
    derived <- logexpression_derive(bundle, observe, "initial")
    bundle$initial <- derived
    saveRDS(bundle, file.path(root, "analysis-bundle.rds"))
    logexpression_write_derived(derived, bundle, file.path(root, "results"))
    logexpression_write_fit(base, input$expression, file.path(root, "base"))
    checks <- list(supplied_E_unchanged=identical(input$expression, derived$supplied_expression),
        base_feature_order=nrow(base$coefficients) == nrow(input$expression) &&
            (nrow(input$expression) == 1L || identical(rownames(base$coefficients), rownames(input$expression))),
        base_design=identical(base$design, input$design), ordinary_df=all(base$df.residual == nrow(input$design)-ncol(input$design)),
        no_count_state=all(vapply(c("EList", "targets", "weights", "voom.xy", "voom.line", "block", "correlation"),
            function(name) is.null(base[[name]]), logical(1))),
        frozen_inputs=identical(hashes, stats::setNames(vapply(frozen, deseq_hash, character(1)), names(hashes))))
    moderation <- lapply(derived$tests, function(value) {
        if (is.null(value$moderated) || value$moderated$status != "returned") return(NULL)
        list(prior_df=limma_shape(value$moderated$value$df.prior),
            positive_infinite_prior=any(value$moderated$value$df.prior == Inf),
            nonfinite=lapply(value$moderated$value[c("t", "p.value", "lods", "F", "F.p.value")], limma_shape))
    })
    summary <- list(checks=checks, complete_derivation=logexpression_complete(derived),
        features=nrow(input$expression), samples=ncol(input$expression), coefficients=ncol(input$design),
        branch=list(equal_positive_df=length(unique(base$df.residual[base$df.residual > 0])) == 1L,
            selected_moderation=if (nrow(input$expression) < 3L) "no-shrink n<3" else "auto legacy fitFDist",
            zero_residual_variance=any(base$sigma == 0), moderation=moderation))
    edger_json(summary, file.path(root, "summary.json"))
    if (!all(vapply(checks, isTRUE, logical(1)))) stop("ordinary source input/base invariants failed")
    summary
}

logexpression_reload <- function(root, source, script) {
    metadata <- file.path(root, "reload-process")
    if (!dir.create(metadata)) stop("reload output already exists")
    observe <- logexpression_observer(metadata)
    status <- "failed"
    checks <- list()
    on.exit({
        edger_json(list(status=status, checks=checks, process=Sys.getpid(),
            mode="separate process; no fitting or preprocessing"), file.path(metadata, "witness.json"))
        logexpression_manifest(root)
    })
    identity <- logexpression_identity(source, metadata)
    initial_process <- readRDS(file.path(root, "metadata/settings.rds"))$initial_process
    if (identical(initial_process, Sys.getpid())) stop("reload must use an independent R process")
    blocked <- list(limma=c("lmFit", "lm.series", "gls.series", "mrlm", "voom", "vooma", "voomaLmFit",
        "normalizeBetweenArrays", "normalizeWithinArrays", "arrayWeights", "duplicateCorrelation"),
        stats=c("lm.fit", "lm.wfit", "lm", "glm.fit", "glm"))
    for (package in names(blocked)) {
        namespace <- asNamespace(package)
        for (name in blocked[[package]]) {
            if (!exists(name, namespace, inherits=FALSE)) next
            unlockBinding(name, namespace)
            assign(name, local({ label <- paste0(package, "::", name); function(...) stop("forbidden reload fit/preprocessing: ", label) }), namespace)
            lockBinding(name, namespace)
        }
    }
    edger_json(list(blocked=blocked, source_qualified_before_guards=identity$pin,
        initial_process=initial_process, reload_process=Sys.getpid()), file.path(metadata, "no-refit-guards.json"))
    for (name in c("real", "equalvar", "constvar", "two-row", "one-zero")) {
        directory <- file.path(root, "cases", name)
        path <- file.path(directory, "analysis-bundle.rds")
        if (!file.exists(path)) stop("initial case has no saved base bundle: ", name)
        bundle_hash <- deseq_hash(path)
        bundle <- readRDS(path)
        before <- serialize(bundle, NULL, version=3)
        result <- logexpression_derive(bundle, observe, paste("reload", name))
        logexpression_write_derived(result, bundle, file.path(directory, "reload"))
        names <- sort(list.files(file.path(directory, "results"), pattern="\\.tsv$", recursive=TRUE), method="radix")
        checks[[name]] <- list(full_derived_state=identical(bundle$initial, result),
            base_bundle_unchanged=identical(before, serialize(bundle, NULL, version=3)),
            saved_bundle_bytes_unchanged=identical(bundle_hash, deseq_hash(path)),
            saved_bundle_state_unchanged=identical(bundle, readRDS(path)),
            TSV_bytes=all(vapply(names, function(member) identical(deseq_hash(file.path(directory, "results", member)),
                deseq_hash(file.path(directory, "reload", member))), logical(1))))
    }
    if (!all(vapply(unlist(checks, recursive=FALSE), isTRUE, logical(1)))) stop("independent no-refit reload checks failed")
    status <- "complete"
    cat("Independent ordinary-expression reload captured; no fit/preprocessing permitted\n")
}

limma_logexpression_main <- function() {
    args <- commandArgs(trailingOnly=TRUE)
    script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
    if (length(script) != 1L) stop("cannot identify source driver")
    script <- normalizePath(sub("^--file=", "", script), mustWork=TRUE)
    helpers <- file.path(dirname(script), c("deseq_workflow_oracle.R", "edger_workflow_oracle.R", "limma_workflow_oracle.R"))
    for (helper in helpers) base::source(helper, local=.GlobalEnv)
    options(digits=17, scipen=0, warn=1)
    if (!identical(Sys.setlocale("LC_COLLATE", "C"), "C") || !identical(Sys.setlocale("LC_NUMERIC", "C"), "C")) stop("cannot establish comparison locale")
    Sys.setenv(OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", MKL_NUM_THREADS="1")
    if (length(args) == 2L && args[[1L]] == "--reload") {
        root <- normalizePath(args[[2L]], mustWork=TRUE)
        source <- readRDS(file.path(root, "metadata/settings.rds"))$source
        return(logexpression_reload(root, source, script))
    }
    if (length(args) != 3L) stop("usage: limma_logexpression_oracle.R LIMMA_SOURCE INPUT_DIR OUTPUT_DIR; or --reload OUTPUT_DIR")
    source <- normalizePath(args[[1L]], mustWork=TRUE)
    inputs <- normalizePath(args[[2L]], mustWork=TRUE)
    root <- normalizePath(args[[3L]], mustWork=FALSE)
    if (file.exists(root) || !dir.create(root, recursive=TRUE)) stop("source output already exists or cannot be created")
    for (name in c("metadata", "inputs", "cases")) if (!dir.create(file.path(root, name))) stop("cannot create source output subdirectory")
    metadata <- file.path(root, "metadata")
    observe <- logexpression_observer(metadata)
    status <- "failed"
    summaries <- list()
    failure <- identity <- NULL
    on.exit({
        edger_json(list(schema_version=1L, status=status, identity=identity, cases=summaries, failure=failure,
            input_kind="supplied finite log-expression", reload="separate process required; see reload-process/witness.json",
            nonfinite_encoding="RDS/TSV retain NA, NaN, Inf, -Inf; JSON uses distinct string tokens and exact locations",
            exclusions=array(c("count preprocessing/voom/vooma", "weights/block/correlation", "missing E",
                "native compatibility/performance/release/biological acceptance"))), file.path(root, "witness.json"))
        logexpression_manifest(root)
    })
    tryCatch({
        identity <- logexpression_identity(source, metadata)
        for (path in c(script, helpers)) if (!file.copy(path, file.path(metadata, basename(path)))) stop("cannot retain controller source")
        if (!file.copy(file.path(source, "DESCRIPTION"), file.path(metadata, "limma-source-DESCRIPTION"))) stop("cannot retain source metadata")
        manifest <- read.delim(file.path(inputs, "manifest.tsv"), check.names=FALSE, stringsAsFactors=FALSE)
        if (anyDuplicated(manifest$path) || nrow(manifest) != 21L ||
            !setequal(list.files(inputs, recursive=TRUE), c(manifest$path, "manifest.tsv"))) stop("input-only manifest membership differs")
        for (index in seq_len(nrow(manifest))) {
            name <- manifest$path[[index]]
            if (grepl("(^/|(^|/)\\.\\.(/|$))", name) || !identical(deseq_hash(file.path(inputs, name)), manifest$sha256[[index]])) stop("frozen input hash differs")
            destination <- file.path(root, "inputs", name)
            dir.create(dirname(destination), recursive=TRUE, showWarnings=FALSE)
            if (!file.copy(file.path(inputs, name), destination)) stop("cannot preserve original input bytes")
        }
        if (!file.copy(file.path(inputs, "manifest.tsv"), file.path(root, "inputs/manifest.tsv"))) stop("cannot retain input manifest")
        parameters <- list(input_kind="log-expression; supplied values already logged and unchanged",
            lmFit=list(method="ls", weights=NULL, block=NULL, ndups=1L, spacing=1L, correlation="unused"),
            moderation=list(proportion=0.01, stdev.coef.lim=c(0.1, 4), trend=FALSE, span=NULL,
                robust=FALSE, winsor.tail.p=c(0.05, 0.1), legacy=NULL),
            topTable=list(coef=1L, number=Inf, sort.by="none", adjust.method="BH", p.value=1, lfc=0, confint=FALSE),
            probes="actual one/two-row source outcome retained; no native fallback or source-input tuning",
            named_join="expression sample order; exact metadata/design identity sets; complete named contrast rows")
        edger_json(parameters, file.path(metadata, "public-call-parameters.json"))
        saveRDS(list(source=source, parameters=parameters, initial_process=Sys.getpid()), file.path(metadata, "settings.rds"))
        for (name in c("real", "equalvar", "constvar", "two-row", "one-zero")) {
            directory <- file.path(root, "cases", name)
            captured <- logexpression_attempt(logexpression_case(logexpression_inputs(file.path(root, "inputs", name)),
                directory, parameters, function(label, expression) observe(paste(name, label), expression)))
            if (captured$status == "returned") {
                summaries[[name]] <- captured$value
            } else {
                dir.create(directory, showWarnings=FALSE)
                edger_json(captured$failure, file.path(directory, "failure.json"))
                summaries[[name]] <- captured
            }
        }
        if (any(vapply(summaries, function(value) identical(value$status, "failed"), logical(1)))) stop("source case or capture failed outside a retained derivation probe")
        core <- c("real", "equalvar", "constvar")
        if (!all(vapply(summaries[core], function(value) isTRUE(value$complete_derivation), logical(1)))) stop("required ordinary source case failed; original state retained")
        inf_observed <- any(vapply(summaries$equalvar$branch$moderation,
            function(value) !is.null(value) && isTRUE(value$positive_infinite_prior), logical(1)))
        if (!inf_observed) stop("fixed equal-variance source probe did not exercise infinite prior df")
        status <- "captured-awaiting-independent-reload"
        cat("Ordinary supplied-expression source captured; independent reload is still required\n")
    }, error=function(error) {
        failure <<- list(class=array(class(error)), message=conditionMessage(error), call=paste(deparse(conditionCall(error)), collapse=" "))
        edger_json(failure, file.path(metadata, "failure.json"))
        stop(error)
    })
}

if (sys.nframe() == 0L) limma_logexpression_main()
