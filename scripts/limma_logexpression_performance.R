#!/usr/bin/env Rscript

ordinary_setup <- function() {
    script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
    if (length(script) != 1L) stop("cannot identify ordinary measurement driver")
    script <- normalizePath(sub("^--file=", "", script), mustWork=TRUE)
    for (name in c("deseq_workflow_oracle.R", "edger_workflow_oracle.R", "limma_workflow_oracle.R", "limma_logexpression_oracle.R"))
        base::source(file.path(dirname(script), name), local=.GlobalEnv)
    for (package in c("limma", "digest", "jsonlite", "BiocManager"))
        if (!requireNamespace(package, quietly=TRUE)) stop("required installed package absent: ", package)
    if (as.character(getRversion()) != "4.6.1" || as.character(BiocManager::version()) != "3.23" ||
        as.character(utils::packageVersion("limma")) != "3.68.5" || any(c("edgeR", "DESeq2") %in% loadedNamespaces()))
        stop("ordinary measurement runtime differs or loaded a count analysis package")
    if (!all(Sys.getenv(c("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS")) == "1")) stop("source thread environment differs")
    options(digits=17, scipen=0, warn=1)
    if (!identical(Sys.setlocale("LC_COLLATE", "C"), "C") || !identical(Sys.setlocale("LC_NUMERIC", "C"), "C")) stop("cannot establish comparison locale")
    script
}

ordinary_runtime <- function(qualification) {
    runtime <- jsonlite::read_json(file.path(qualification, "runtime.json"), simplifyVector=FALSE)
    if (!identical(runtime$R, as.character(getRversion())) ||
        !identical(runtime$limma$installed_path, system.file(package="limma", mustWork=TRUE)) ||
        !identical(runtime$limma$installed_dll_sha256, deseq_hash(getLoadedDLLs()[["limma"]][["path"]]))) stop("qualified ordinary runtime differs")
    definition <- c("formals:", deparse(formals(stats::lm.fit)), "body:", deparse(body(stats::lm.fit)))
    if (!identical(readLines(file.path(qualification, "stats-lm.fit.txt")), definition)) stop("qualified stats::lm.fit body differs")
}

ordinary_no_refit <- function() {
    blocked <- list(limma=c("lmFit", "lm.series", "gls.series", "mrlm", "voom", "vooma", "voomaLmFit",
        "normalizeBetweenArrays", "normalizeWithinArrays", "arrayWeights", "duplicateCorrelation"),
        stats=c("lm.fit", "lm.wfit", "lm", "glm.fit", "glm"))
    for (package in names(blocked)) {
        namespace <- asNamespace(package)
        for (name in blocked[[package]]) {
            if (!exists(name, namespace, inherits=FALSE)) next
            unlockBinding(name, namespace)
            assign(name, local({ label <- paste0(package, "::", name); function(...) stop("forbidden ordinary reload fit/preprocessing: ", label) }), namespace)
            lockBinding(name, namespace)
        }
    }
}

ordinary_test <- function(bundle, name) {
    contrast <- matrix(bundle$contrasts[[name]], ncol=1L, dimnames=list(colnames(bundle$design), name))
    fitted <- limma::contrasts.fit(bundle$base, contrasts=contrast)
    moderated <- limma::eBayes(fitted)
    table <- limma::topTable(moderated, coef=1L, number=Inf, sort.by="none", adjust.method="BH", p.value=1, lfc=0, confint=FALSE)
    list(contrast=fitted, moderated=moderated, table=table)
}

ordinary_performance_run <- function(witness, output, qualification) {
    ordinary_runtime(qualification)
    if (!dir.create(output)) stop("ordinary output already exists or cannot be created")
    parameters <- readRDS(file.path(witness, "metadata/settings.rds"))$parameters
    started <- proc.time()
    input <- logexpression_inputs(file.path(witness, "inputs/real"))
    if (!identical(dim(input$expression), c(8066L, 7L)) || !identical(names(input$contrasts), c("forward", "reverse", "mixed"))) stop("real ordinary input differs")
    base <- limma::lmFit(input$expression, input$design, method="ls", weights=NULL, block=NULL, ndups=1, spacing=1)
    bundle <- c(input, list(schema_version=1L, parameters=parameters, base=base))
    bundle$initial <- ordinary_test(bundle, "forward")
    saveRDS(bundle, file.path(output, "fit.rds"))
    rm(bundle, base, input)
    reloaded <- readRDS(file.path(output, "fit.rds"))
    ordinary_no_refit()
    for (name in names(reloaded$contrasts)) {
        state <- ordinary_test(reloaded, name)
        saveRDS(state, file.path(output, paste0(name, ".rds")))
        deseq_write_table(state$table, file.path(output, paste0(name, ".tsv")), rownames(state$table), "source_feature_id")
    }
    exports <- list(expression=reloaded$expression, fitted=stats::fitted(reloaded$base),
        residuals=stats::residuals(reloaded$base, reloaded$expression))
    for (name in names(exports)) {
        value <- exports[[name]]
        if (!identical(dimnames(value), dimnames(reloaded$expression))) stop("ordinary export identities differ")
        deseq_write_table(value, file.path(output, paste0(name, ".tsv")), rownames(value), "source_feature_id")
    }
    elapsed <- proc.time() - started
    edger_json(list(inner_elapsed_seconds=unname(elapsed[["elapsed"]]), inner_user_seconds=unname(elapsed[["user.self"]]),
        inner_system_seconds=unname(elapsed[["sys.self"]]),
        region="same supplied E/sample/design/contrast TSV read and named join, one public unweighted lmFit, initial forward contrast/default eBayes/topTable, full default-compressed RDS durable save/reload, three contrast/moderated/table RDS and six TSVs",
        excluded="package loading/runtime qualification and later actual-state inspection/diagnostic exports",
        serialization="default gzip-compressed complete input/base/initial analysis RDS plus three full derived RDS, not portable native JSON",
        no_refit="saved full base reused after namespace no-fit/preprocessing guards"), file.path(output, "inner-timing.json"))
}

ordinary_performance_inspect <- function(trial, output, qualification, witness) {
    ordinary_runtime(qualification)
    ordinary_no_refit()
    if (!dir.create(output)) stop("ordinary inspection output already exists or cannot be created")
    bounds <- jsonlite::read_json(file.path(dirname(qualification), "bounds.json"), simplifyVector=TRUE)
    failures <- metrics <- list()
    exact_fields <- 0L
    fail <- function(path, reason, details=NULL) failures[[length(failures)+1L]] <<- list(path=path, reason=reason, details=details)
    selected_kind <- function(path) {
        if (grepl("/parameters(/|$)|/input_kind$|/expression$|/samples(/|$)|/design(/|$)|/contrasts(/|$)|/df.residual$", path)) return(NULL)
        slot <- tail(strsplit(path, "/", fixed=TRUE)[[1L]], 1L)
        if (slot %in% c("Amean", "AveExpr", "E")) return("expression")
        if (slot %in% c("coefficients", "stdev.unscaled", "cov.coefficients", "logFC", "sigma")) return("linear")
        if (slot %in% c("df.prior", "df.total")) return("df")
        if (slot %in% c("s2.prior", "s2.post", "t", "F")) return("eb")
        if (slot %in% c("lods", "var.prior", "B")) return("odds")
        if (slot %in% c("p.value", "F.p.value", "P.Value", "adj.P.Val")) return("probability")
        NULL
    }
    numeric_class <- function(value) ifelse(is.nan(value), "NaN", ifelse(is.na(value), "NA",
        ifelse(is.infinite(value), ifelse(value > 0, "Inf", "-Inf"), "finite")))
    walk <- function(actual, expected, path) {
        if (!identical(typeof(actual), typeof(expected)) || !identical(attributes(actual), attributes(expected)) || length(actual) != length(expected)) {
            fail(path, "type/attributes/shape/names differ")
            return(invisible(NULL))
        }
        if (is.list(actual)) {
            labels <- names(actual)
            if (is.null(labels)) labels <- as.character(seq_along(actual))
            for (i in seq_along(actual)) walk(actual[[i]], expected[[i]], paste(path, labels[[i]], sep="/"))
            return(invisible(NULL))
        }
        kind <- if (is.numeric(actual)) selected_kind(path) else NULL
        if (is.null(kind)) {
            exact_fields <<- exact_fields+1L
            if (!identical(actual, expected)) fail(path, "exact field differs")
            return(invisible(NULL))
        }
        classes <- numeric_class(actual)
        reference <- numeric_class(expected)
        mismatch <- which(classes != reference)
        if (length(mismatch)) fail(path, "NA/NaN/infinity class differs", list(indices=array(mismatch)))
        indices <- which(classes == "finite" & reference == "finite")
        pair <- bounds[[kind]]
        scaled <- abs(actual[indices]-expected[indices])/(pair[[1L]]+pair[[2L]]*abs(expected[indices]))
        log_error <- rep(0, length(indices))
        positive <- actual[indices] > 0 & expected[indices] > 0
        if (kind == "probability") log_error[positive] <- abs(log(actual[indices][positive])-log(expected[indices][positive]))
        lost <- if (kind == "probability") expected[indices] > 0 & actual[indices] == 0 else rep(FALSE, length(indices))
        bad <- which(scaled > 1 | log_error > 1e-3 | lost)
        metrics[[path]] <<- list(kind=kind, count=length(indices), failed=length(bad),
            max_scaled=if (length(scaled)) max(scaled) else 0, max_log_probability_error=if (length(log_error)) max(log_error) else 0)
        if (length(bad)) fail(path, "fixed numerical bound failed", list(indices=array(indices[bad]), scaled=array(scaled[bad]), log_error=array(log_error[bad])))
    }
    bundle <- readRDS(file.path(trial, "fit.rds"))
    reference <- readRDS(file.path(witness, "cases/real/analysis-bundle.rds"))
    fields <- setdiff(names(reference), "initial")
    if (!identical(names(bundle), names(reference))) fail("bundle", "full input/base/initial inventory differs")
    walk(bundle[fields], reference[fields], "bundle")
    unwrap <- function(state) {
        if (!all(vapply(state, function(value) identical(value$status, "returned"), logical(1)))) stop("original real source derivation did not return")
        lapply(state, function(value) value$value)
    }
    fixed <- reference$initial
    walk(bundle$initial, unwrap(fixed$tests$forward), "initial")
    logexpression_write_fit(bundle$base, bundle$expression, file.path(output, "base"))
    discoveries <- list()
    for (name in names(bundle$contrasts)) {
        state <- readRDS(file.path(trial, paste0(name, ".rds")))
        expected <- unwrap(fixed$tests[[name]])
        walk(state, expected, paste0("results/", name))
        if (name == "forward" && !identical(bundle$initial, state)) fail(name, "initial/reloaded complete forward state differs")
        actual_ids <- rownames(state$table)[which(state$table$adj.P.Val <= 0.05)]
        expected_ids <- rownames(expected$table)[which(expected$table$adj.P.Val <= 0.05)]
        if (!identical(actual_ids, expected_ids)) fail(name, "discovery identities/order differ")
        discoveries[[name]] <- list(actual=length(actual_ids), source=length(expected_ids))
        for (stage in c("contrast", "moderated")) logexpression_write_fit(state[[stage]], bundle$expression, file.path(output, paste0(name, "-", stage)))
        copy <- file.path(output, paste0(name, ".tsv"))
        deseq_write_table(state$table, copy, rownames(state$table), "source_feature_id")
        if (!identical(deseq_hash(copy), deseq_hash(file.path(trial, paste0(name, ".tsv"))))) fail(name, "timed table differs from actual saved state")
    }
    stored <- list(expression=bundle$expression, fitted=stats::fitted(bundle$base), residuals=stats::residuals(bundle$base, bundle$expression))
    for (name in names(stored)) {
        actual <- as.matrix(read.delim(file.path(trial, paste0(name, ".tsv")), row.names=1L, check.names=FALSE))
        expected <- fixed[[if (name == "expression") "supplied_expression" else name]]
        walk(actual, expected, paste0("exports/", name, "/", if (name == "expression") "E" else "coefficients"))
        copy <- file.path(output, paste0(name, ".tsv"))
        deseq_write_table(stored[[name]], copy, rownames(stored[[name]]), "source_feature_id")
        if (!identical(deseq_hash(copy), deseq_hash(file.path(trial, paste0(name, ".tsv"))))) fail(name, "timed export differs from actual full base")
    }
    edger_json(failures, file.path(output, "failures.json"))
    edger_json(list(passed=length(failures) == 0L, failure_count=length(failures), bounds=bounds,
        positive_probability_log_bound=1e-3, metrics=metrics, exact_fields=exact_fields, discoveries=discoveries,
        inspection="actual full input/base/initial RDS, three complete derived RDS and six TSVs; namespace fitting/preprocessing blocked; no statistical re-derivation"), file.path(output, "comparison.json"))
    if (length(failures)) stop("actual measured ordinary source state/output comparison failed")
}

ordinary_performance_main <- function() {
    script <- ordinary_setup()
    args <- commandArgs(trailingOnly=TRUE)
    destination <- if (length(args) >= 3L) args[[3L]] else NULL
    index <- 0L
    note <- function(value, kind) {
        if (!is.null(destination) && dir.exists(destination)) {
            index <<- index+1L
            row <- data.frame(index=index, kind=kind, class=paste(class(value), collapse=","), message=conditionMessage(value), call=paste(deparse(conditionCall(value)), collapse=" "))
            write.table(row, file.path(destination, "conditions.tsv"), sep="\t", quote=TRUE, row.names=FALSE, col.names=index == 1L, append=index > 1L)
        }
    }
    withCallingHandlers({
        if (length(args) == 3L && args[[1L]] == "qualify") {
            if (!dir.create(args[[3L]])) stop("qualification output already exists")
            logexpression_identity(normalizePath(args[[2L]], mustWork=TRUE), args[[3L]])
            if (!file.copy(script, file.path(args[[3L]], basename(script)))) stop("cannot retain measurement driver")
        } else if (length(args) == 4L && args[[1L]] == "run") ordinary_performance_run(args[[2L]], args[[3L]], args[[4L]])
        else if (length(args) == 5L && args[[1L]] == "inspect") ordinary_performance_inspect(args[[2L]], args[[3L]], args[[4L]], args[[5L]])
        else stop("usage: limma_logexpression_performance.R qualify LIMMA OUTPUT | run WITNESS OUTPUT QUALIFICATION | inspect TRIAL OUTPUT QUALIFICATION WITNESS")
    }, warning=function(value) note(value, "warning"), message=function(value) note(value, "message"), error=function(value) note(value, "error"))
}

if (sys.nframe() == 0L) ordinary_performance_main()
