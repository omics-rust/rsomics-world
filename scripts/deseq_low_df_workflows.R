#!/usr/bin/env Rscript

low_df_binary <- function(value, path, size) {
    connection <- file(path, open="wb")
    on.exit(close(connection))
    writeBin(value, connection, size=size, endian="little")
}

low_df_inputs <- function(root) {
    RNGkind("Mersenne-Twister", "Inversion", "Rejection")
    set.seed(20261030L)
    states <- list(initial=.Random.seed)
    genes <- sprintf("gene%04d", seq_len(1200L))
    pairs <- paste0("P", seq_len(5L))
    samples <- as.vector(rbind(paste0(pairs, "-control"), paste0(pairs, "-treated")))
    metadata <- data.frame(pair=factor(rep(pairs, each=2L), levels=pairs),
        condition=factor(rep(c("control", "treated"), 5L), levels=c("control", "treated")), row.names=samples)
    base_log2 <- stats::runif(length(genes), 3, 10)
    states$after_base <- .Random.seed
    base <- 2^base_log2
    dispersion_noise <- stats::rnorm(length(genes), 0, 1.2)
    states$after_dispersion <- .Random.seed
    dispersion <- (0.08 + 2 / base) * exp(dispersion_noise)
    condition_effect <- stats::rnorm(length(genes), 0, 0.3)
    states$after_condition <- .Random.seed
    pair_effect <- matrix(stats::rnorm(length(genes) * 5L, 0, 0.15), nrow=length(genes),
                          dimnames=list(genes, pairs))
    states$after_pair <- .Random.seed
    exposure <- stats::setNames(exp(seq(log(0.8), log(1.3), length.out=length(samples))), samples)
    mu <- exp(outer(log(base), rep(1, length(samples))) +
        outer(condition_effect, as.numeric(metadata$condition == "treated")) +
        pair_effect[, rep(seq_len(5L), each=2L), drop=FALSE] +
        outer(rep(1, length(genes)), log(exposure)))
    dimnames(mu) <- list(genes, samples)
    counts <- design_counts(mu, dispersion, genes, samples)
    states$after_counts <- .Random.seed
    directory <- file.path(root, "inputs/generation")
    if (!dir.create(directory, recursive=TRUE)) stop("cannot create generation directory")
    design_table(counts, file.path(directory, "counts.tsv"), genes)
    design_table(metadata, file.path(directory, "samples.tsv"), samples, "sample")
    design_table(data.frame(base_log2=base_log2, base_mean=base, dispersion_log_noise=dispersion_noise,
        true_dispersion=dispersion, condition_log_effect=condition_effect),
        file.path(directory, "gene-parameter-draws.tsv"), genes)
    design_table(pair_effect, file.path(directory, "pair-effect-draws.tsv"), genes)
    design_table(mu, file.path(directory, "negative-binomial-means.tsv"), genes)
    design_table(data.frame(exposure=unname(exposure)), file.path(directory, "library-exposure.tsv"), samples, "sample")
    saveRDS(list(counts=counts, metadata=metadata, base_log2=base_log2, dispersion_noise=dispersion_noise,
        condition_effect=condition_effect, pair_effect=pair_effect, exposure=exposure, mu=mu,
        RNG_states=states), file.path(directory, "all-generation-draws.rds"))
    deseq_write_json(list(seed=20261030L, RNGkind=array(RNGkind()), genes=1200L, samples=10L,
        base="2^runif(1200,3,10)", dispersion="(.08+2/base)*exp(rnorm(1200,0,1.2))",
        condition_effect="rnorm(1200,0,.3)", pair_effect="1200 by 5 independent rnorm(0,.15) draws",
        exposure="exp(seq(log(.8),log(1.3),length.out=10))",
        log_mean="log(base)+gene condition effect+gene/pair effect+log(exposure)",
        count_distribution="actual stats::rnbinom(mu,size=1/dispersion)", prefilter=FALSE),
        file.path(directory, "generator.json"))
    prepared <- list()
    for (n in 2:5) {
        selected <- seq_len(2L * n)
        input <- list(counts=counts[, selected, drop=FALSE],
            meta=droplevels(metadata[selected, , drop=FALSE]), sf=exposure[selected], nf=NULL)
        name <- paste0("paired-", n)
        directory <- file.path(root, "inputs", name)
        if (!dir.create(directory)) stop("cannot create paired input directory")
        design_table(input$counts, file.path(directory, "counts.tsv"), genes)
        design_table(input$meta, file.path(directory, "samples.tsv"), colnames(input$counts), "sample")
        matrix <- stats::model.matrix(~pair+condition, data=input$meta)
        forward <- stats::setNames(rep(0, ncol(matrix)), colnames(matrix))
        forward[["conditiontreated"]] <- 1
        mixed <- forward
        mixed[["pairP2"]] <- -1
        spec <- list(id=paste0("df-", n - 1L), input=name, formula="~pair+condition", route="formula",
            normalization="ratio", replacement="disabled", contrasts=list(forward=forward, mixed=mixed))
        prepared[[spec$id]] <- design_prepare_case(spec, input, root)
    }
    deseq_write_json(lapply(prepared, function(value) value$info), file.path(root, "inputs/case-inventory.json"))
    prepared
}

low_df_reference <- function(coarse, breaks, dfs=1:3) {
    old_seed <- .Random.seed
    on.exit(assign(".Random.seed", old_seed, envir=.GlobalEnv))
    result <- vector("list", length(dfs))
    for (index in seq_along(dfs)) {
        df <- dfs[index]
        set.seed(2)
        initial <- .Random.seed
        counts <- matrix(0L, nrow=length(coarse), ncol=length(breaks) - 1L)
        density <- matrix(0, nrow=nrow(counts), ncol=ncol(counts))
        retained <- integer(length(coarse))
        for (g in seq_along(coarse)) {
            random <- log(stats::rchisq(1e4, df=df)) + stats::rnorm(1e4, 0, sqrt(coarse[g])) - log(df)
            random <- random[random > min(breaks) & random < max(breaks)]
            histogram <- graphics::hist(random, breaks=breaks, plot=FALSE)
            counts[g, ] <- histogram$counts
            density[g, ] <- histogram$density
            retained[g] <- length(random)
            if (g == 1L) after_zero_sd <- .Random.seed
        }
        result[[index]] <- list(counts=counts, density=density, retained=retained,
            RNG_initial=initial, RNG_after_zero_sd=after_zero_sd, RNG_final=.Random.seed)
    }
    result
}

low_df_direct <- function(response, coarse, fine) {
    fit <- stats::loess(response ~ coarse, span=0.2)
    list(prediction=as.numeric(stats::predict(fit, fine)), pars=fit$pars,
        weights_one=all(fit$weights == 1), robust_one=all(fit$robust == 1), divisor_one=all(fit$divisor == 1),
        geometry=fit$kd[c("parameter", "a", "xi", "vert")])
}

low_df_apply <- function(operator, response) {
    result <- numeric(nrow(operator))
    for (i in seq_len(nrow(operator))) {
        value <- 0
        for (j in seq_len(ncol(operator))) value <- value + operator[i, j] * response[j]
        result[i] <- value
    }
    result
}

low_df_operator <- function(coarse, fine) {
    values <- matrix(0, nrow=length(fine), ncol=length(coarse))
    geometry <- NULL
    invariants <- c(geometry_equal=TRUE, gaussian_one_iteration=TRUE, degree_two=TRUE,
        interpolate=TRUE, unit_weights=TRUE, unit_robust_weights=TRUE, divisor_one=TRUE)
    for (j in seq_along(coarse)) {
        basis <- numeric(length(coarse))
        basis[j] <- 1
        fit <- low_df_direct(basis, coarse, fine)
        values[, j] <- fit$prediction
        if (is.null(geometry)) geometry <- fit$geometry
        invariants <- invariants & c(geometry_equal=identical(geometry, fit$geometry),
            gaussian_one_iteration=fit$pars$family == "gaussian" && fit$pars$iterations == 1L,
            degree_two=fit$pars$degree == 2L, interpolate=fit$pars$surface == "interpolate",
            unit_weights=fit$weights_one, unit_robust_weights=fit$robust_one, divisor_one=fit$divisor_one)
    }
    list(weights=values, invariants=invariants, controls=fit$pars,
         geometry_sha256=digest::digest(geometry, algo="sha256"))
}

low_df_prediction_check <- function(reference, candidate) {
    bound <- 2e-11 + 5e-13 * abs(reference)
    residual <- candidate - reference
    ok <- is.finite(reference) & is.finite(candidate) & abs(residual) <= bound
    list(passed=all(ok), failed_points=array(which(!ok)), max_abs=max(abs(residual)),
         max_scaled=max(abs(residual) / bound), residual=residual, bound=bound)
}

low_df_probes <- function(operator, responses, coarse, fine, root) {
    results <- list()
    for (name in names(responses)) {
        direct <- low_df_direct(responses[[name]], coarse, fine)$prediction
        projected <- low_df_apply(operator, responses[[name]])
        check <- low_df_prediction_check(direct, projected)
        design_table(data.frame(variance=fine, direct=direct, operator=projected,
            linearity_residual=check$residual, bound=check$bound), file.path(root, "probes", paste0(name, "-predictions.tsv")))
        saveRDS(list(response=responses[[name]], direct=direct, operator=projected, check=check),
            file.path(root, "probes", paste0(name, "-predictions.rds")))
        results[[name]] <- check[c("passed", "failed_points", "max_abs", "max_scaled")]
    }
    list(passed=all(vapply(results, function(value) value$passed, logical(1))), probes=results)
}

low_df_kl <- function(observed, reference, breaks) {
    density <- reference$counts / (reference$retained * rep(diff(breaks), each=nrow(reference$counts)))
    values <- numeric(nrow(density))
    small <- numeric(length(values))
    for (i in seq_along(values)) {
        z <- c(observed$density, density[i, ])
        small[i] <- min(z[z > 0])
        values[i] <- sum(observed$density * (log(observed$density + small[i]) - log(density[i, ] + small[i])))
    }
    list(values=values, small=small, density=density)
}

low_df_case_check <- function(prepared, prior_fun, references, operator, coarse, fine, breaks) {
    directory <- file.path(prepared$directory, "qualification")
    if (!dir.create(directory)) stop("cannot create prior qualification directory")
    fit <- readRDS(file.path(prepared$directory, "fit/DESeqDataSet.rds"))
    matrix <- attr(fit, "dispModelMatrix")
    df <- nrow(matrix) - ncol(matrix)
    stored_prior <- attr(DESeq2::dispersionFunction(fit), "dispPriorVar")
    seed <- .Random.seed
    actual_prior <- prior_fun(fit)
    seed_preserved <- identical(seed, .Random.seed)
    if (df == 4L) {
        variance <- attr(DESeq2::dispersionFunction(fit), "varLogDispEsts")
        replay_prior <- pmax(variance - trigamma(df / 2), 0.25)
        result <- list(id=prepared$spec$id, df=df, path="ordinary-non-Monte-Carlo", stored_prior=stored_prior,
            unmodified_source_prior=actual_prior, replay_prior=replay_prior, RNG_preserved=seed_preserved,
            passed=seed_preserved && identical(stored_prior, actual_prior) && identical(actual_prior, replay_prior))
        deseq_write_json(result, file.path(directory, "result.json"))
        return(result)
    }
    object <- fit[!S4Vectors::mcols(fit)$allZero, , drop=FALSE]
    row_data <- S4Vectors::mcols(object)
    residual <- log(row_data$dispGeneEst) - log(row_data$dispFit)
    above <- row_data$dispGeneEst >= 1e-8 * 100
    selected <- residual[above]
    selected <- selected[selected > min(breaks) & selected < max(breaks)]
    observed <- graphics::hist(selected, breaks=breaks, plot=FALSE)
    design_table(data.frame(gene_estimate=row_data$dispGeneEst, fitted=row_data$dispFit,
        residual=residual, above_min=above, in_outer_range=residual > min(breaks) & residual < max(breaks)),
        file.path(directory, "observed-residuals.tsv"), rownames(object))
    design_table(data.frame(lower=head(breaks, -1L), upper=tail(breaks, -1L),
        count=observed$counts, density=observed$density), file.path(directory, "observed-histogram.tsv"))
    reference <- references[[df]]
    from_counts <- low_df_kl(observed, reference, breaks)
    replay <- low_df_reference(coarse, breaks, dfs=df)[[1L]]
    replay_kl <- vapply(seq_along(coarse), function(i) {
        z <- c(observed$density, replay$density[i, ])
        small <- min(z[z > 0])
        sum(observed$density * (log(observed$density + small) - log(replay$density[i, ] + small)))
    }, numeric(1))
    direct <- low_df_direct(replay_kl, coarse, fine)$prediction
    projected <- low_df_apply(operator, from_counts$values)
    check <- low_df_prediction_check(direct, projected)
    direct_index <- which.min(direct)
    operator_index <- which.min(projected)
    replay_prior <- pmax(fine[direct_index], 0.25)
    operator_prior <- pmax(fine[operator_index], 0.25)
    histogram_equal <- identical(reference$counts, replay$counts) && identical(reference$retained, replay$retained)
    density_equal <- identical(from_counts$density, replay$density, num.eq=FALSE)
    kl_equal <- identical(from_counts$values, replay_kl, num.eq=FALSE)
    design_table(data.frame(variance=coarse, retained=reference$retained, small=from_counts$small,
        KL_from_counts=from_counts$values, KL_replay=replay_kl), file.path(directory, "KL-grid.tsv"))
    design_table(data.frame(variance=fine, direct=direct, operator=projected,
        residual=check$residual, bound=check$bound), file.path(directory, "fine-predictions.tsv"))
    saveRDS(list(observed=observed, reference_counts=reference$counts, replay_reference=replay,
        KL_from_counts=from_counts, KL_replay=replay_kl, direct=direct, operator=projected, check=check),
        file.path(directory, "complete-replay.rds"))
    result <- list(id=prepared$spec$id, df=df, path="source Monte Carlo prior",
        observed_filtered_vector_length=length(selected), observed_retained=sum(observed$counts),
        reference_histograms_equal=histogram_equal, reference_density_equal=density_equal, KL_equal=kl_equal,
        prediction_check=check[c("passed", "failed_points", "max_abs", "max_scaled")],
        source_replay_argmin_index=direct_index, operator_argmin_index=operator_index,
        source_replay_argmin=fine[direct_index], operator_argmin=fine[operator_index],
        source_nearest_minimum_gap=min(direct[-direct_index]) - direct[direct_index],
        operator_nearest_minimum_gap=min(projected[-operator_index]) - projected[operator_index],
        stored_prior=stored_prior, unmodified_source_prior=actual_prior, replay_prior=replay_prior,
        operator_prior=operator_prior, RNG_preserved=seed_preserved,
        private_argmin_observability="unmodified estimator returns the floored prior; unfloored argmin comes from the full algorithm replay",
        passed=seed_preserved && histogram_equal && density_equal && kl_equal && check$passed &&
            identical(direct_index, operator_index) && identical(stored_prior, actual_prior) &&
            identical(actual_prior, replay_prior) && identical(replay_prior, operator_prior))
    deseq_write_json(result, file.path(directory, "result.json"))
    result
}

low_df_workflows_main <- function() {
    args <- commandArgs(trailingOnly=TRUE)
    if (length(args) != 2L) stop("usage: deseq_low_df_workflows.R SOURCE_DIR OUTPUT_DIR")
    script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
    if (length(script) != 1L) stop("cannot identify source driver")
    script <- normalizePath(sub("^--file=", "", script), mustWork=TRUE)
    helpers <- file.path(dirname(script), c("deseq_workflow_oracle.R", "deseq_design_workflows.R"))
    for (helper in helpers) base::source(helper, local=.GlobalEnv)
    for (package in c("DESeq2", "digest", "jsonlite", "BiocManager")) {
        if (!requireNamespace(package, quietly=TRUE)) stop("required installed package absent: ", package)
    }
    pin <- list(name="DESeq2", version="1.52.0", bioconductor="3.23",
        source_commit="16aeab6d6158bd8cbb8e98764c2f33399b1a3fd3",
        source_archive_sha256="8c91699286336350e66eec132ce6fdf5bb4af78e2a4d015a5a61224f62a95984")
    if (as.character(getRversion()) != "4.6.1" || as.character(utils::packageVersion("DESeq2")) != pin$version ||
        as.character(BiocManager::version()) != pin$bioconductor) stop("installed runtime/package version differs")
    archive <- Sys.getenv("DESEQ2_SOURCE_ARCHIVE")
    if (!nzchar(archive) || !identical(deseq_hash(archive), pin$source_archive_sha256) ||
        !identical(Sys.getenv("DESEQ2_SOURCE_SHA256"), pin$source_archive_sha256) ||
        !identical(Sys.getenv("DESEQ2_SOURCE_COMMIT"), pin$source_commit)) stop("source acquisition identity differs")
    source_path <- normalizePath(args[[1L]], mustWork=TRUE)
    root <- normalizePath(args[[2L]], mustWork=FALSE)
    if (file.exists(root) || dir.exists(root)) stop("output directory already exists")
    if (!dir.create(root, recursive=TRUE)) stop("cannot create output directory")
    for (name in c("metadata", "inputs", "cases", "constants", "probes")) {
        if (!dir.create(file.path(root, name))) stop("cannot create output subdirectory")
    }
    options(digits=17, scipen=0, warn=1, contrasts=c("contr.treatment", "contr.poly"))
    if (!identical(Sys.setlocale("LC_COLLATE", "C"), "C") || !identical(Sys.setlocale("LC_NUMERIC", "C"), "C")) stop("cannot establish comparison locale")
    Sys.setenv(OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", MKL_NUM_THREADS="1")
    metadata <- file.path(root, "metadata")
    source_identity <- deseq_source_identity(source_path, metadata)
    prior_fun <- get("estimateDispersionsPriorVar", asNamespace("DESeq2"), inherits=FALSE)
    expected <- deseq_definition(file.path(source_path, "R/core.R"), "estimateDispersionsPriorVar")
    if (!identical(deparse(formals(prior_fun)), deparse(expected[[2L]])) ||
        !identical(deparse(body(prior_fun)), deparse(expected[[3L]]))) stop("installed prior estimator differs from the released source")
    writeLines(c("formals:", deparse(formals(prior_fun)), "body:", deparse(body(prior_fun))),
        file.path(metadata, "function-estimateDispersionsPriorVar.txt"))
    for (path in c(script, helpers)) if (!file.copy(path, file.path(metadata, basename(path)))) stop("cannot retain driver/helper")
    if (!file.copy(file.path(source_path, "DESCRIPTION"), file.path(metadata, "source-DESCRIPTION"))) stop("cannot retain source description")
    writeLines(capture.output(sessionInfo()), file.path(metadata, "session.txt"))
    functions <- list(hist.default=get("hist.default", asNamespace("graphics")), loess=stats::loess,
        loess.control=stats::loess.control, simpleLoess=get("simpleLoess", asNamespace("stats")),
        predict.loess=get("predict.loess", asNamespace("stats")), predLoess=get("predLoess", asNamespace("stats")))
    function_files <- character()
    for (name in names(functions)) {
        path <- file.path(metadata, paste0("R-function-", name, ".txt"))
        writeLines(c("formals:", deparse(formals(functions[[name]])), "body:", deparse(body(functions[[name]]))), path)
        function_files[name] <- deseq_hash(path)
    }
    rng_profile <- c("Mersenne-Twister", "Inversion", "Rejection")
    lib_r <- file.path(R.home("lib"), paste0("libR", .Platform$dynlib.ext))
    dll <- getLoadedDLLs()[["DESeq2"]][["path"]]
    runtime <- list(R=as.character(getRversion()), R_version=as.list(R.version),
        Bioconductor=as.character(BiocManager::version()), DESeq2=as.character(utils::packageVersion("DESeq2")),
        system=as.list(Sys.info()), BLAS_LAPACK=capture.output(La_library()),
        library_paths=array(.libPaths()), DESeq2_dll=dll, DESeq2_dll_sha256=deseq_hash(dll),
        thread_environment=as.list(Sys.getenv(c("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"))),
        libR_sha256=if (file.exists(lib_r)) deseq_hash(lib_r) else NULL,
        installed_function_sha256=as.list(function_files), loess_control=stats::loess.control(),
        R_tag_sources=array(c("https://svn.r-project.org/R/tags/R-4-6-1/src/library/graphics/R/hist.R",
            "https://svn.r-project.org/R/tags/R-4-6-1/src/library/stats/R/loess.R",
            "https://svn.r-project.org/R/tags/R-4-6-1/src/library/stats/src/loessf.f")),
        R_tag_comparison="optional tag bodies unavailable to the controller; actual installed bodies retained, no AST-equality claim")
    deseq_write_json(runtime, file.path(metadata, "runtime.json"))
    packages <- utils::installed.packages()[, c("Package", "Version", "LibPath"), drop=FALSE]
    write.table(packages[order(packages[, "Package"], method="radix"), , drop=FALSE],
        file.path(metadata, "packages.tsv"), sep="\t", row.names=FALSE, quote=TRUE)
    conditions <- list()
    observe <- function(id, label, expression) {
        note <- function(condition, kind) {
            row <- data.frame(index=length(conditions) + 1L, case=id, operation=label, kind=kind,
                class=paste(class(condition), collapse=","), message=conditionMessage(condition),
                call=paste(deparse(conditionCall(condition)), collapse=" "))
            conditions[[length(conditions) + 1L]] <<- row
            write.table(row, file.path(metadata, "conditions.tsv"), sep="\t", quote=TRUE, row.names=FALSE,
                col.names=length(conditions) == 1L, append=length(conditions) > 1L)
        }
        withCallingHandlers(force(expression), warning=function(value) note(value, "warning"),
            message=function(value) note(value, "message"), error=function(value) note(value, "error"))
    }
    prepared <- low_df_inputs(root)
    runtime$RNGkind <- array(RNGkind())
    deseq_write_json(runtime, file.path(metadata, "runtime.json"))
    coarse <- seq(from=0, to=8, length=200)
    fine <- seq(from=0, to=8, length.out=1000)
    breaks <- -20:20 / 2
    responses <- list(constant=rep(1, length(coarse)), linear=0.125 + 0.375 * coarse,
        quadratic=(coarse - 3.25)^2 / 7 - 0.5, localized_peak=exp(-((coarse - 5.1) / 0.35)^2),
        mixed_smooth=0.75 * sin(0.8 * coarse) + 0.2 * cos(2.3 * coarse) + 0.03 * coarse^2)
    bounds <- list(normalization=c(absolute=1e-10, relative=1e-10), dispersion=c(absolute=1e-6, relative=1e-4),
        coefficient=c(absolute=1e-4, relative=1e-5), probability=c(absolute=1e-7, relative=1e-4),
        mu=c(absolute=1e-6, relative=1e-5), diagnostic=c(absolute=1e-6, relative=1e-4), vst=c(absolute=1e-5, relative=1e-6))
    parameters <- list(design="~pair+condition", fitType="mean", test="Wald", sfType="ratio",
        betaPrior=FALSE, minReplicatesForReplace="Inf", useT=FALSE, minmu=0.5, parallel=FALSE,
        alpha=0.1, independentFiltering=TRUE, pAdjustMethod="BH", cooksCutoff="omitted: actual source default",
        transform="varianceStabilizingTransformation(blind=FALSE)", RNGkind=array(rng_profile),
        prediction_bound=c(absolute=2e-11, relative=5e-13), prior_floor=0.25,
        argmin="actual which.min; first minimum, no epsilon ties")
    deseq_write_json(bounds, file.path(root, "inputs/native-comparison-bounds.json"))
    deseq_write_json(parameters, file.path(root, "inputs/public-call-parameters.json"))
    algorithm <- list(schema_version=1L, status="source-only candidate constants, not adopted",
        package=pin, R="4.6.1", RNGkind=array(rng_profile), seed=2L, dfs=array(1:3),
        coarse_points=200L, fine_points=1000L, histogram_bins=40L, draws_per_grid=10000L,
        draw_order="rchisq(10000,df), then actual rnorm(10000,0,sqrt(variance)); including sd=0 call",
        outer_filter="strict -10 < value < 10 before actual graphics::hist",
        histogram="actual hist defaults; right-closed/include.lowest; densities divide by actual retained totals and unfuzzed widths",
        histogram_fuzz="source default 1e-7 times .5-width bins; no native binning implementation in this artifact",
        loess="actual loess(response~coarse,span=.2), all other Gaussian/interpolated defaults",
        operator_layout="1000 fine rows by 200 coarse columns, row-major IEEE754 f64 little-endian",
        reference_layout="df1,df2,df3; then coarse grid; then bins; uint32 little-endian",
        application="ascending coarse-column scalar f64 multiply then add, no FMA, pruning, clipping or normalization")
    deseq_write_json(algorithm, file.path(root, "constants/algorithm.json"))
    low_df_binary(coarse, file.path(root, "constants/coarse-grid.f64le"), 8L)
    low_df_binary(fine, file.path(root, "constants/fine-grid.f64le"), 8L)
    low_df_binary(breaks, file.path(root, "constants/histogram-breaks.f64le"), 8L)
    for (name in names(responses)) design_table(data.frame(coarse=coarse, response=responses[[name]]),
        file.path(root, "probes", paste0(name, "-response.tsv")))
    frozen <- sort(list.files(root, recursive=TRUE, full.names=TRUE), method="radix")
    frozen_hashes <- stats::setNames(vapply(frozen, deseq_hash, character(1)), substring(frozen, nchar(root) + 2L))
    deseq_write_json(list(status="all inputs, options and independent probes frozen before any fit/outcome",
        files=as.list(frozen_hashes)), file.path(metadata, "input-freeze.json"))
    cat("Frozen four paired source workflows, algorithm settings and five independent probes before fitting\n")
    references <- observe("constants", "actual R reference histograms", low_df_reference(coarse, breaks))
    reference_flat <- as.integer(unlist(lapply(references, function(value) as.vector(t(value$counts))), use.names=FALSE))
    low_df_binary(reference_flat, file.path(root, "constants/reference-counts.u32le"), 4L)
    saveRDS(lapply(references, function(value) value[c("RNG_initial", "RNG_after_zero_sd", "RNG_final")]),
        file.path(root, "constants/reference-RNG-states.rds"))
    low_df_binary(as.integer(unlist(lapply(references, function(value) value$retained), use.names=FALSE)),
        file.path(root, "constants/reference-retained-totals.u32le"), 4L)
    generated <- observe("constants", "actual R basis prediction operator", low_df_operator(coarse, fine))
    operator <- generated$weights
    low_df_binary(as.vector(t(operator)), file.path(root, "constants/prediction-operator.f64le"), 8L)
    deseq_write_json(list(controls=generated$controls, invariants=as.list(generated$invariants),
        geometry_sha256=generated$geometry_sha256, negative_weights=sum(operator < 0), zero_weights=sum(operator == 0),
        minimum_weight=min(operator), maximum_weight=max(operator)), file.path(root, "constants/operator-provenance.json"))
    constant_files <- sort(list.files(file.path(root, "constants"), full.names=TRUE), method="radix")
    constant_hashes <- stats::setNames(vapply(constant_files, deseq_hash, character(1)), basename(constant_files))
    deseq_write_json(list(status="generated from fixed algorithm settings only; no workflow prior or identity values",
        files=as.list(constant_hashes)), file.path(root, "constants/manifest.json"))
    probes <- observe("probes", "independent fixed-response qualification", low_df_probes(operator, responses, coarse, fine, root))
    cases <- list()
    qualification <- list()
    for (id in names(prepared)) {
        value <- prepared[[id]]
        cases[[id]] <- tryCatch(design_run_case(value, observe, fit_type="mean"), error=function(error) {
            failure <- list(id=id, status="failed", message=conditionMessage(error), class=array(class(error)))
            deseq_write_json(failure, file.path(value$directory, "failure.json"))
            failure
        })
        if (cases[[id]]$status == "complete") {
            qualification[[id]] <- tryCatch(observe(id, "unmodified prior and full replay qualification",
                low_df_case_check(value, prior_fun, references, operator, coarse, fine, breaks)), error=function(error) {
                    failure <- list(id=id, passed=FALSE, error=conditionMessage(error))
                    deseq_write_json(failure, file.path(value$directory, "qualification-failure.json"))
                    failure
                })
        }
    }
    if (!identical(frozen_hashes, stats::setNames(vapply(frozen, deseq_hash, character(1)), names(frozen_hashes)))) {
        stop("frozen input/probe/provenance bytes changed during qualification")
    }
    installed_prior <- get("estimateDispersionsPriorVar", asNamespace("DESeq2"), inherits=FALSE)
    if (!identical(formals(prior_fun), formals(installed_prior)) || !identical(body(prior_fun), body(installed_prior))) {
        stop("installed prior estimator changed during qualification")
    }
    source_ok <- length(cases) == 4L && all(vapply(cases, function(value) value$status == "complete", logical(1)))
    replay_ok <- length(qualification) == 4L && all(vapply(qualification, function(value) value$passed, logical(1)))
    positive <- names(Filter(function(value) !is.null(value$df) && value$df <= 3L &&
        !is.null(value$unmodified_source_prior) && value$unmodified_source_prior > 0.25, qualification))
    operator_ok <- all(generated$invariants) && probes$passed && replay_ok
    status <- if (!source_ok) "source-failed" else if (!length(positive)) "positive-prior-uncovered" else if (!operator_ok) "operator-rejected" else "complete"
    files <- sort(list.files(root, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
    hashes <- stats::setNames(vapply(file.path(root, files), deseq_hash, character(1)), files)
    witness <- list(schema_version=1L, status=status, package=pin, runtime=runtime, parameters=parameters,
        source=source_identity, prior_function_released_body_matched=TRUE,
        input_freeze_sha256=deseq_hash(file.path(metadata, "input-freeze.json")),
        constants_manifest_sha256=deseq_hash(file.path(root, "constants/manifest.json")),
        source_workflows=list(complete=source_ok, cases=cases),
        operator_qualification=list(complete=operator_ok && length(positive) > 0L,
            mathematical_checks_passed=operator_ok, conditionally_feasible_only=TRUE, adopted=FALSE,
            basis_invariants=as.list(generated$invariants), probes=probes, workflows=qualification,
            positive_nonfloor_MC_cases=array(positive), positive_prior_covered=length(positive) > 0L,
            no_universal_bitwise_equivalence_claim=TRUE),
        conditions=list(count=length(conditions), warnings=sum(vapply(conditions, function(value) value$kind == "warning", logical(1))),
            errors=sum(vapply(conditions, function(value) value$kind == "error", logical(1)))),
        excluded=array(c("native code or constant adoption", "native/four-platform compatibility or performance",
            "production prior answers, fixture identities or per-input branches in constants", "RNG profiles other than the recorded source profile")),
        files=as.list(hashes))
    deseq_write_json(witness, file.path(root, "witness.json"))
    manifest <- sort(c(files, "witness.json"), method="radix")
    write.table(data.frame(sha256=vapply(file.path(root, manifest), deseq_hash, character(1)), path=manifest),
        file.path(root, "sha256-manifest.tsv"), sep="\t", quote=FALSE, row.names=FALSE)
    if (status != "complete") stop("low-df source qualification did not pass: ", status)
    cat("Actual df1/2/3/4 mean-trend workflows and fixed-operator qualification complete; constants remain unadopted\n")
}

if (sys.nframe() == 0L) low_df_workflows_main()
