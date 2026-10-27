#!/usr/bin/env Rscript

design_table <- function(value, path, ids=NULL, id_name="gene") {
    if (methods::is(value, "DataFrame")) value <- as.data.frame(value)
    deseq_write_table(value, path, ids, id_name)
}

design_counts <- function(mu, dispersion, genes, samples) {
    value <- matrix(as.integer(stats::rnbinom(length(mu), mu=as.vector(mu),
        size=rep(1 / dispersion, ncol(mu)))), nrow=nrow(mu), dimnames=list(genes, samples))
    if (anyNA(value) || any(value < 0L)) stop("count generation exceeded integer range")
    value
}

design_inputs <- function(root) {
    RNGkind("Mersenne-Twister", "Inversion", "Rejection")
    set.seed(20261028L)
    before <- .Random.seed
    ngenes <- 1200L
    samples <- sprintf("sample%02d", seq_len(18L))
    meta <- expand.grid(x=c(-0.75, 0.125, 1), group=c("A", "B", "C"),
        batch=c("b0", "b1"), KEEP.OUT.ATTRS=FALSE, stringsAsFactors=FALSE)
    meta$group <- factor(meta$group, levels=c("A", "B", "C"))
    meta$batch <- factor(meta$batch, levels=c("b0", "b1"))
    rownames(meta) <- samples
    genes <- sprintf("gene%04d", seq_len(ngenes))
    base <- 2^stats::runif(ngenes, 3, 10)
    dispersion <- 0.08 + 2 / base
    effect_b <- stats::rnorm(ngenes, 0, 0.35)
    effect_c <- stats::rnorm(ngenes, 0, 0.45)
    effect_batch <- stats::rnorm(ngenes, 0, 0.12)
    slope <- stats::rnorm(ngenes, 0, 0.15)
    slope_b <- stats::rnorm(ngenes, 0, 0.1)
    slope_c <- stats::rnorm(ngenes, 0, 0.1)
    true_sf <- exp(seq(log(0.8), log(1.3), length.out=length(samples)))
    eta <- outer(log(base), rep(1, length(samples))) +
        outer(effect_b, as.numeric(meta$group == "B")) +
        outer(effect_c, as.numeric(meta$group == "C")) +
        outer(effect_batch, as.numeric(meta$batch == "b1")) + outer(slope, meta$x) +
        outer(slope_b, meta$x * (meta$group == "B")) +
        outer(slope_c, meta$x * (meta$group == "C")) +
        outer(rep(1, ngenes), log(true_sf))
    counts <- design_counts(exp(eta), dispersion, genes, samples)
    sf <- stats::setNames(exp(seq(log(0.7), log(1.7), length.out=length(samples))), samples)
    nf <- outer(exp(0.2 * sin(seq_len(ngenes) / 11)),
                sf * exp(0.12 * cos(seq_along(samples) / 3))) *
          exp(0.1 * outer(sin(seq_len(ngenes) / 17), cos(seq_along(samples) / 5)))
    dimnames(nf) <- dimnames(counts)
    general <- list(counts=counts, meta=meta, sf=sf, nf=nf,
        truth=data.frame(base_mean=base, dispersion=dispersion, group_B=effect_b,
            group_C=effect_c, batch_b1=effect_batch, x=slope,
            group_B_x=slope_b, group_C_x=slope_c),
        generation=list(seed=20261028L, RNGkind=array(RNGkind()), genes=ngenes, samples=18L,
            mean="exp(log(base)+group effects+batch effect+x and group:x slopes+log(true SF))",
            base="2^runif(1200,3,10)", dispersion="0.08+2/base",
            count_distribution="stats::rnbinom(mu, size=1/dispersion)",
            x=array(c(-0.75, 0.125, 1)), true_size_factors=array(true_sf),
            supplied_size_factors="exp(seq(log(.7),log(1.7),length.out=18)); not recentered",
            supplied_nf="gene/sample-dependent positive matrix; distinct from supplied SF",
            prefilter=FALSE), rng_before=before, rng_after=.Random.seed)

    set.seed(20261029L)
    before <- .Random.seed
    cook_samples <- c(sprintf("A%02d", seq_len(7L)), sprintf("B%02d", seq_len(7L)))
    cook_meta <- data.frame(group=factor(rep(c("A", "B"), each=7L), levels=c("A", "B")),
                            row.names=cook_samples)
    cook_base <- 2^stats::runif(ngenes, 3, 10)
    cook_disp <- 0.08 + 2 / cook_base
    cook_effect <- stats::rnorm(ngenes, 0, 0.4)
    cook_mu <- exp(outer(log(cook_base), rep(1, 14L)) +
                   outer(cook_effect, as.numeric(cook_meta$group == "B")))
    cook_counts <- design_counts(cook_mu, cook_disp, genes, cook_samples)
    special <- rbind(
        "eligible-high-A"=c(1000000L, rep(30L, 6L), rep(40L, 7L)),
        "ineligible-high-B"=c(rep(30L, 7L), 1000000L, rep(40L, 6L)),
        "replace-zero-A"=c(1000000L, rep(0L, 13L)),
        "low-count-B"=c(rep(60000L, 7L), 0L, rep(10000L, 6L)),
        "all-zero"=rep(0L, 14L),
        "stable"=c(rep(100L, 7L), rep(200L, 7L)))
    colnames(special) <- cook_samples
    cook_counts <- rbind(cook_counts, special)
    cook <- list(counts=cook_counts, meta=cook_meta,
        sf=stats::setNames(rep(1, 14L), cook_samples), nf=NULL,
        truth=data.frame(base_mean=cook_base, dispersion=cook_disp, group_B=cook_effect),
        generation=list(seed=20261029L, RNGkind=array(RNGkind()), background_genes=ngenes,
            samples=14L, count_distribution="stats::rnbinom(mu, size=1/dispersion)",
            base="2^runif(1200,3,10)", dispersion="0.08+2/base",
            group_B="rnorm(1200,0,.4)", supplied_size_factors="all one, explicitly supplied",
            preset_gene_ids=array(rownames(special)), preset_counts=unname(special),
            purpose="source-only branch qualification; presets frozen before the first fit",
            prefilter=FALSE), rng_before=before, rng_after=.Random.seed)
    smaller <- cook
    smaller$counts <- cook$counts[, c(seq_len(7L), 8:10), drop=FALSE]
    smaller$meta <- cook$meta[colnames(smaller$counts), , drop=FALSE]
    smaller$sf <- cook$sf[colnames(smaller$counts)]
    smaller$generation$parent <- "cook-7x7"
    smaller$generation$samples <- 10L
    smaller$generation$selection <- array(colnames(smaller$counts))
    values <- list(general=general, "cook-7x7"=cook, "cook-7x3"=smaller)
    for (name in names(values)) {
        value <- values[[name]]
        directory <- file.path(root, "inputs", name)
        if (!dir.create(directory, recursive=TRUE)) stop("cannot create input directory")
        design_table(value$counts, file.path(directory, "counts.tsv"), rownames(value$counts))
        design_table(value$meta, file.path(directory, "samples.tsv"), rownames(value$meta), "sample")
        order <- rev(seq_along(value$sf))
        design_table(data.frame(factor=unname(value$sf[order])), file.path(directory, "size-factors.tsv"),
                     names(value$sf)[order], "sample")
        if (!is.null(value$nf)) {
            reversed <- value$nf[rev(seq_len(nrow(value$nf))), rev(seq_len(ncol(value$nf))), drop=FALSE]
            design_table(reversed, file.path(directory, "normalization-factors.tsv"), rownames(reversed))
        }
        design_table(value$truth, file.path(directory, "generation-parameters.tsv"), genes)
        deseq_write_json(value$generation, file.path(directory, "generation.json"))
        saveRDS(list(before=value$rng_before, after=value$rng_after), file.path(directory, "RNG-state.rds"))
        saveRDS(list(counts=value$counts, coldata=value$meta, size_factors=value$sf,
                     normalization_factors=value$nf), file.path(directory, "generated-inputs.rds"))
    }
    values
}

design_case_inventory <- function() {
    formula <- "~batch*group + group*x"
    cases <- list(
        list(id="interaction-ratio-formula", input="general", formula=formula, route="formula", normalization="ratio", replacement="disabled"),
        list(id="interaction-ratio-matrix", input="general", formula=formula, route="matrix", normalization="ratio", replacement="disabled"),
        list(id="interaction-sf-formula", input="general", formula=formula, route="formula", normalization="sf", replacement="disabled"),
        list(id="interaction-nf-formula", input="general", formula=formula, route="formula", normalization="nf", replacement="disabled"),
        list(id="interaction-sf-nf-formula", input="general", formula=formula, route="formula", normalization="sf-nf", replacement="disabled"),
        list(id="nointercept-sf-formula", input="general", formula="~0+group+batch+x", route="formula", normalization="sf", replacement="disabled"),
        list(id="intercept-sf-formula", input="general", formula="~1", route="formula", normalization="sf", replacement="disabled"))
    for (input in c("cook-7x7", "cook-7x3")) {
        for (replacement in c("default", "disabled")) {
            for (route in c("formula", "matrix")) {
                cases[[length(cases) + 1L]] <- list(id=paste(input, replacement, route, sep="-"),
                    input=input, formula="~group", route=route, normalization="sf", replacement=replacement)
            }
        }
    }
    cases
}

design_prepare_case <- function(spec, input, root) {
    formula <- stats::as.formula(spec$formula, env=baseenv())
    matrix <- stats::model.matrix(formula, data=input$meta)
    terms <- stats::terms(formula, data=input$meta)
    if (qr(matrix)$rank != ncol(matrix) || nrow(matrix) <= ncol(matrix)) stop("source design is not full rank")
    forward <- stats::setNames(rep(0, ncol(matrix)), colnames(matrix))
    mixed <- forward
    if (!is.null(spec$contrasts)) {
        align <- function(weights) {
            if (!is.numeric(weights) || any(!is.finite(weights)) || is.null(names(weights)) ||
                anyDuplicated(names(weights)) || !setequal(names(weights), colnames(matrix)) ||
                !any(weights != 0)) stop("explicit contrast must name every model column once")
            weights[colnames(matrix)]
        }
        forward <- align(spec$contrasts$forward)
        mixed <- if (is.null(spec$contrasts$mixed)) NULL else align(spec$contrasts$mixed)
    } else if (spec$formula == "~1") {
        forward[["(Intercept)"]] <- 1
        mixed <- NULL
    } else if (spec$formula == "~0+group+batch+x") {
        forward[c("groupA", "groupB")] <- c(-1, 1)
        mixed[c("groupB", "groupC")] <- c(-1, 1)
    } else if (spec$input == "general") {
        forward[["groupB"]] <- 1
        mixed[c("groupB", "groupC")] <- c(1, -1)
    } else {
        forward[["groupB"]] <- 1
        mixed[c("(Intercept)", "groupB")] <- c(-1, 1)
    }
    if (length(forward) != ncol(matrix) || (!is.null(mixed) && length(mixed) != ncol(matrix))) {
        stop("planned contrast does not match the actual R model columns")
    }
    directory <- file.path(root, "cases", spec$id)
    if (!dir.create(file.path(directory, "inputs"), recursive=TRUE)) stop("cannot create case input directory")
    design_table(matrix, file.path(directory, "inputs/model-matrix.tsv"), rownames(matrix), "sample")
    factors <- attr(terms, "factors")
    if (is.matrix(factors) && length(factors)) {
        design_table(factors, file.path(directory, "inputs/terms-factors.tsv"), rownames(factors), "variable")
    }
    factor_names <- names(input$meta)[vapply(input$meta, is.factor, logical(1))]
    factor_coding <- list()
    for (name in factor_names) {
        coding <- stats::contrasts(input$meta[[name]])
        design_table(coding, file.path(directory, "inputs", paste0("factor-", name, "-coding.tsv")),
                     rownames(coding), "level")
        dummy <- stats::contrasts(input$meta[[name]], contrasts=FALSE)
        design_table(dummy, file.path(directory, "inputs", paste0("factor-", name, "-dummy-coding.tsv")),
                     rownames(dummy), "level")
        term <- match(name, attr(terms, "term.labels"))
        columns <- if (is.na(term)) integer() else which(attr(matrix, "assign") == term)
        if (length(columns)) {
            design_table(matrix[, columns, drop=FALSE],
                file.path(directory, "inputs", paste0("factor-", name, "-design-columns.tsv")),
                rownames(matrix), "sample")
        }
        factor_coding[[name]] <- list(default_contrast_columns=array(colnames(coding)),
            dummy_columns=array(colnames(dummy)), actual_design_columns=array(colnames(matrix)[columns]))
    }
    factor_levels <- lapply(input$meta[factor_names], levels)
    info <- list(id=spec$id, input=spec$input, formula=spec$formula, route=spec$route,
        count_input=paste0("inputs/", spec$input, "/counts.tsv"),
        metadata_input=paste0("inputs/", spec$input, "/samples.tsv"),
        normalization=spec$normalization, supplied_values_recentered=FALSE,
        size_factor_input=if (spec$normalization %in% c("sf", "sf-nf")) paste0("inputs/", spec$input, "/size-factors.tsv") else NULL,
        normalization_factor_input=if (spec$normalization %in% c("nf", "sf-nf")) paste0("inputs/", spec$input, "/normalization-factors.tsv") else NULL,
        replacement=spec$replacement, minReplicatesForReplace=if (spec$replacement == "disabled") "Inf" else 7L,
        factor_levels=factor_levels, factor_coding=factor_coding,
        numeric_columns=array(names(input$meta)[vapply(input$meta, is.numeric, logical(1))]),
        contrast_options=as.list(getOption("contrasts")), raw_columns=array(colnames(matrix)),
        assign=array(attr(matrix, "assign")), model_matrix_contrasts=as.list(attr(matrix, "contrasts")),
        terms=list(labels=array(attr(terms, "term.labels")), order=array(attr(terms, "order")),
            intercept=attr(terms, "intercept"), response=attr(terms, "response"),
            variables=deparse(attr(terms, "variables"))),
        contrasts=list(forward=as.list(forward), reverse=as.list(-forward), mixed=if (is.null(mixed)) NULL else as.list(mixed)),
        mixed_contrast_exclusion=if (is.null(mixed)) "intercept-only design has one coefficient" else NULL)
    deseq_write_json(info, file.path(directory, "inputs/case.json"))
    design_table(data.frame(forward=unname(forward), reverse=unname(-forward),
        mixed=if (is.null(mixed)) rep(NA_real_, length(forward)) else unname(mixed)),
        file.path(directory, "inputs/contrasts.tsv"), names(forward), "coefficient")
    saveRDS(list(formula=formula, terms=terms, model_matrix=matrix, contrasts=list(forward=forward, mixed=mixed)),
            file.path(directory, "inputs/design.rds"))
    list(spec=spec, input=input, formula=formula, matrix=matrix, forward=forward, mixed=mixed,
         info=info, directory=directory)
}

design_cook_state <- function(fit, forward, unmasked, spec, input, directory) {
    counts <- DESeq2::counts(fit)
    assay_names <- SummarizedExperiment::assayNames(fit)
    cooks <- SummarizedExperiment::assays(fit)[["cooks"]]
    matrix <- attr(fit, "dispModelMatrix")
    cutoff <- stats::qf(0.99, ncol(matrix), nrow(matrix) - ncol(matrix))
    finite <- is.finite(cooks)
    raw_max <- vapply(seq_len(nrow(cooks)), function(g) {
        if (any(finite[g, ])) max(cooks[g, finite[g, ]]) else NA_real_
    }, numeric(1))
    replacements <- if ("replaceCounts" %in% assay_names) SummarizedExperiment::assays(fit)[["replaceCounts"]] else counts
    changed <- rowSums(replacements != counts) > 0L
    row_data <- as.data.frame(S4Vectors::mcols(fit))
    flagged <- if ("replace" %in% names(row_data)) row_data$replace else rep(FALSE, nrow(counts))
    flagged[is.na(flagged)] <- FALSE
    original_nonzero <- rowSums(counts) > 0L
    now_zero <- changed & rowSums(replacements) == 0L & original_nonzero
    masked <- is.na(forward$pvalue) & !is.na(unmasked$pvalue)
    low_counts <- vapply(seq_len(nrow(counts)), function(g) {
        if (!any(finite[g, ])) return(FALSE)
        largest <- which.max(replace(cooks[g, ], !finite[g, ], -Inf))
        sum(counts[g, ] > counts[g, largest]) >= 3L
    }, logical(1))
    post_max <- if ("maxCooks" %in% names(row_data)) row_data$maxCooks else rep(NA_real_, nrow(counts))
    low_exempt <- is.finite(post_max) & post_max > cutoff & low_counts &
        is.finite(forward$pvalue) & is.finite(unmasked$pvalue) & spec$route == "formula"
    zero_result <- now_zero & !is.na(forward$pvalue) & forward$pvalue == 1 &
        !is.na(forward$log2FoldChange) & forward$log2FoldChange == 0 &
        !is.na(forward$lfcSE) & forward$lfcSE == 0 & !is.na(forward$stat) & forward$stat == 0
    table <- data.frame(raw_max_cooks=raw_max, cutoff=cutoff, post_max_cooks=post_max,
        source_replace_flag=flagged, actual_count_change=changed,
        original_all_zero=!original_nonzero, replaced_to_zero=now_zero,
        masked_pvalue=masked, three_counts_larger=low_counts, low_count_exemption=low_exempt,
        replacement_zero_result=zero_result, pvalue=forward$pvalue, unmasked_pvalue=unmasked$pvalue)
    design_table(table, file.path(directory, "results/cook-branches.tsv"), rownames(counts))
    if (!identical(counts, input$counts)) stop("DESeq did not preserve original counts")
    replaceable <- SummarizedExperiment::colData(fit)$replaceable
    list(cutoff=cutoff, assay_names=array(assay_names),
        replaceable_samples=if (is.null(replaceable)) NULL else as.list(stats::setNames(as.logical(replaceable), colnames(fit))),
        raw_outliers=array(rownames(counts)[is.finite(raw_max) & raw_max > cutoff]),
        source_flagged_genes=array(rownames(counts)[flagged]),
        count_changed_genes=array(rownames(counts)[changed]),
        replaced_to_zero=array(rownames(counts)[now_zero]),
        replacement_zero_result=array(rownames(counts)[zero_result]),
        masked_genes=array(rownames(counts)[masked]),
        low_count_exemptions=array(rownames(counts)[low_exempt]), original_counts_preserved=TRUE)
}

design_run_case <- function(prepared, observe, fit_type="parametric") {
    spec <- prepared$spec
    input <- prepared$input
    directory <- prepared$directory
    for (name in c("fit", "results", "reload")) {
        if (!dir.create(file.path(directory, name))) stop("cannot create case output directory")
    }
    call <- function(label, expression) observe(spec$id, label, expression)
    dataset <- call("construct", DESeq2::DESeqDataSetFromMatrix(countData=input$counts, colData=input$meta,
        design=if (spec$route == "formula") prepared$formula else prepared$matrix))
    if (spec$normalization %in% c("sf", "sf-nf")) {
        DESeq2::sizeFactors(dataset) <- unname(input$sf[colnames(dataset)])
    }
    if (spec$normalization %in% c("nf", "sf-nf")) {
        DESeq2::normalizationFactors(dataset) <- input$nf[rownames(dataset), colnames(dataset), drop=FALSE]
    }
    fit <- call("DESeq", DESeq2::DESeq(dataset, test="Wald", fitType=fit_type, sfType="ratio",
        betaPrior=FALSE, minReplicatesForReplace=if (spec$replacement == "disabled") Inf else 7L,
        useT=FALSE, minmu=0.5, parallel=FALSE))
    before <- deseq_fit_state(fit)
    if (!identical(dim(before$model_matrix), dim(prepared$matrix)) ||
        !all(before$model_matrix == prepared$matrix)) stop("fitted model matrix differs from the supplied design")
    derive <- function(object, weights, label, ignore_cooks=FALSE) {
        args <- list(object=object, contrast=unname(weights), alpha=0.1,
            independentFiltering=TRUE, pAdjustMethod="BH", parallel=FALSE)
        if (ignore_cooks) args$cooksCutoff <- FALSE
        call(label, do.call(DESeq2::results, args))
    }
    forward <- derive(fit, prepared$forward, "forward results")
    mixed <- if (is.null(prepared$mixed)) NULL else derive(fit, prepared$mixed, "mixed results")
    unmasked <- if (spec$input == "general") NULL else derive(fit, prepared$forward, "unmasked Cook diagnostics", TRUE)
    normalized <- call("normalized counts", DESeq2::counts(fit, normalized=TRUE))
    transformed <- call("fit-reusing VST", DESeq2::varianceStabilizingTransformation(fit, blind=FALSE))
    if (!identical(before, deseq_fit_state(fit))) stop("derived outputs changed the fitted state")
    if (!identical(DESeq2::counts(fit), input$counts)) stop("fitted state does not retain original counts")
    saveRDS(fit, file.path(directory, "fit/DESeqDataSet.rds"))
    saveRDS(before, file.path(directory, "fit/complete-state.rds"))
    saveRDS(list(forward=forward, mixed=mixed, unmasked=unmasked, normalized=normalized, vst=transformed),
            file.path(directory, "results/derived-objects.rds"))
    for (entry in c("forward", "mixed", "unmasked")) {
        value <- get(entry)
        if (!is.null(value)) {
            design_table(value, file.path(directory, "results", paste0(entry, ".tsv")), rownames(fit))
            deseq_write_json(S4Vectors::metadata(value),
                file.path(directory, "results", paste0(entry, "-metadata.json")))
        }
    }
    design_table(normalized, file.path(directory, "results/normalized-counts.tsv"), rownames(fit))
    design_table(SummarizedExperiment::assay(transformed), file.path(directory, "results/vst.tsv"), rownames(fit))
    omitted <- design_table(S4Vectors::mcols(fit), file.path(directory, "fit/row-diagnostics.tsv"), rownames(fit))
    design_table(stats::coef(fit), file.path(directory, "fit/coefficients.tsv"), rownames(fit))
    design_table(S4Vectors::mcols(S4Vectors::mcols(fit)), file.path(directory, "fit/column-metadata.tsv"),
                 names(S4Vectors::mcols(fit)), "column")
    design_table(attr(fit, "modelMatrix"), file.path(directory, "fit/model-matrix.tsv"), colnames(fit), "sample")
    design_table(attr(fit, "dispModelMatrix"), file.path(directory, "fit/dispersion-model-matrix.tsv"), colnames(fit), "sample")
    coefficient_names <- DESeq2::resultsNames(fit)
    design_table(data.frame(raw=colnames(prepared$matrix), coefficient=coefficient_names,
        fitted=colnames(attr(fit, "modelMatrix")), assign=attr(prepared$matrix, "assign")),
        file.path(directory, "fit/coefficient-map.tsv"))
    sf <- DESeq2::sizeFactors(fit)
    nf <- DESeq2::normalizationFactors(fit)
    if (!is.null(sf)) design_table(data.frame(factor=sf), file.path(directory, "fit/size-factors.tsv"), colnames(fit), "sample")
    if (!is.null(nf)) design_table(nf, file.path(directory, "fit/normalization-factors.tsv"), rownames(fit))
    deseq_write_json(list(mode=spec$normalization, size_factors=if (is.null(sf)) NULL else as.list(sf),
        normalization_factors_present=!is.null(nf), fit_offsets_use=if (is.null(nf)) "size factors" else "normalization factors",
        supplied_values_recentered=FALSE), file.path(directory, "fit/normalization.json"))
    assays <- SummarizedExperiment::assays(fit)
    for (name in names(assays)) {
        if (!grepl("^[A-Za-z0-9_.-]+$", name)) stop("unexpected assay name")
        design_table(assays[[name]], file.path(directory, "fit", paste0("assay-", name, ".tsv")), rownames(fit))
    }
    trend <- DESeq2::dispersionFunction(fit)
    saveRDS(attributes(trend), file.path(directory, "fit/dispersion-attributes.rds"))
    deseq_write_json(attributes(trend), file.path(directory, "fit/dispersion-attributes.json"))
    writeLines(deparse(body(trend)), file.path(directory, "fit/dispersion-body.txt"))
    reloaded <- readRDS(file.path(directory, "fit/DESeqDataSet.rds"))
    if (!identical(before, deseq_fit_state(reloaded))) stop("reloaded fitted state differs")
    repeat_forward <- derive(reloaded, prepared$forward, "reloaded forward results")
    repeat_mixed <- if (is.null(mixed)) NULL else derive(reloaded, prepared$mixed, "reloaded mixed results")
    reverse <- derive(reloaded, -prepared$forward, "reloaded reverse results")
    repeat_normalized <- call("reloaded normalized counts", DESeq2::counts(reloaded, normalized=TRUE))
    repeat_vst <- call("reloaded fit-reusing VST", DESeq2::varianceStabilizingTransformation(reloaded, blind=FALSE))
    if (!identical(as.data.frame(forward), as.data.frame(repeat_forward)) ||
        !identical(S4Vectors::metadata(forward), S4Vectors::metadata(repeat_forward)) ||
        (!is.null(mixed) && (!identical(as.data.frame(mixed), as.data.frame(repeat_mixed)) ||
            !identical(S4Vectors::metadata(mixed), S4Vectors::metadata(repeat_mixed)))) ||
        !identical(normalized, repeat_normalized) ||
        !identical(SummarizedExperiment::assay(transformed), SummarizedExperiment::assay(repeat_vst)) ||
        !identical(before, deseq_fit_state(reloaded))) stop("reloaded outputs or fitted state differ")
    saveRDS(list(forward=repeat_forward, mixed=repeat_mixed, reverse=reverse,
        normalized=repeat_normalized, vst=repeat_vst), file.path(directory, "reload/derived-objects.rds"))
    design_table(repeat_forward, file.path(directory, "reload/forward.tsv"), rownames(fit))
    if (!is.null(repeat_mixed)) design_table(repeat_mixed, file.path(directory, "reload/mixed.tsv"), rownames(fit))
    design_table(reverse, file.path(directory, "reload/reverse.tsv"), rownames(fit))
    deseq_write_json(S4Vectors::metadata(reverse), file.path(directory, "reload/reverse-metadata.json"))
    design_table(repeat_normalized, file.path(directory, "reload/normalized-counts.tsv"), rownames(fit))
    design_table(SummarizedExperiment::assay(repeat_vst), file.path(directory, "reload/vst.tsv"), rownames(fit))
    row_data <- as.data.frame(S4Vectors::mcols(fit))
    cooks <- if (is.null(unmasked)) NULL else design_cook_state(fit, forward, unmasked, spec, input, directory)
    result <- list(id=spec$id, status="complete", input=spec$input, parameters=prepared$info,
        genes=nrow(fit), samples=ncol(fit), raw_columns=array(colnames(prepared$matrix)),
        coefficient_names=array(coefficient_names), model_rank=qr(prepared$matrix)$rank,
        model_matrix_type=attr(fit, "modelMatrixType"), actual_dispersion_fit=attr(trend, "fitType"),
        beta_nonconverged_genes=array(rownames(fit)[!is.na(row_data$betaConv) & !row_data$betaConv]),
        omitted_plain_diagnostic_columns=array(omitted),
        forward_na_counts=as.list(vapply(as.data.frame(forward), function(value) sum(is.na(value)), integer(1))),
        cooks=cooks, reload=list(fitted_state_identical=TRUE, forward_identical=TRUE,
            mixed_identical=if (is.null(mixed)) NULL else TRUE, normalized_identical=TRUE,
            vst_identical=TRUE, refit=FALSE))
    deseq_write_json(result, file.path(directory, "case.json"))
    result
}

design_coverage <- function(cases) {
    cook <- Filter(function(value) value$status == "complete" && !is.null(value$cooks), cases)
    by_id <- stats::setNames(cook, vapply(cook, function(value) value$id, character(1)))
    contains <- function(id, field, gene) {
        !is.null(by_id[[id]]) && gene %in% by_id[[id]]$cooks[[field]]
    }
    replacement_cases <- Filter(function(value) value$parameters$replacement == "default", cook)
    disabled <- Filter(function(value) value$parameters$replacement == "disabled", cook)
    required <- c(
        positive_raw_cooks=any(vapply(cook, function(value) length(value$cooks$raw_outliers) > 0L, logical(1))),
        eligible_replacement=contains("cook-7x7-default-formula", "count_changed_genes", "eligible-high-A") &&
            contains("cook-7x3-default-formula", "count_changed_genes", "eligible-high-A"),
        ineligible_outlier_retained=contains("cook-7x3-default-formula", "raw_outliers", "ineligible-high-B") &&
            !contains("cook-7x3-default-formula", "count_changed_genes", "ineligible-high-B") &&
            contains("cook-7x3-default-formula", "masked_genes", "ineligible-high-B"),
        positive_pvalue_mask=any(vapply(disabled, function(value) length(value$cooks$masked_genes) > 0L, logical(1))),
        low_count_formula_exemption=contains("cook-7x3-disabled-formula", "low_count_exemptions", "low-count-B") &&
            contains("cook-7x3-disabled-matrix", "masked_genes", "low-count-B"),
        replacement_to_zero=contains("cook-7x7-default-formula", "replacement_zero_result", "replace-zero-A") &&
            contains("cook-7x3-default-formula", "replacement_zero_result", "replace-zero-A"),
        disabled_replacement=length(disabled) == 4L && all(vapply(disabled, function(value) {
            length(value$cooks$count_changed_genes) == 0L && !"replaceCounts" %in% value$cooks$assay_names
        }, logical(1))),
        formula_and_matrix_replacement=length(replacement_cases) == 4L && all(vapply(replacement_cases, function(value) {
            "eligible-high-A" %in% value$cooks$count_changed_genes
        }, logical(1))))
    list(complete=all(required), observed=as.list(required), uncovered=array(names(required)[!required]),
        interpretation="actual public-call outcomes; a missing trigger is uncovered, not accepted",
        per_case=as.list(stats::setNames(lapply(cook, function(value) value$cooks), names(by_id))))
}

design_workflows_main <- function() {
    args <- commandArgs(trailingOnly=TRUE)
    if (length(args) != 2L) stop("usage: deseq_design_workflows.R SOURCE_DIR OUTPUT_DIR")
    script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
    if (length(script) != 1L) stop("cannot identify source driver")
    script <- normalizePath(sub("^--file=", "", script), mustWork=TRUE)
    helper <- file.path(dirname(script), "deseq_workflow_oracle.R")
    base::source(helper, local=.GlobalEnv)
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
    for (name in c("metadata", "inputs", "cases")) if (!dir.create(file.path(root, name))) stop("cannot create output subdirectory")
    options(digits=17, scipen=0, warn=1, contrasts=c("contr.treatment", "contr.poly"))
    if (!identical(Sys.setlocale("LC_COLLATE", "C"), "C") || !identical(Sys.setlocale("LC_NUMERIC", "C"), "C")) {
        stop("cannot establish source comparison locale")
    }
    Sys.setenv(OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", MKL_NUM_THREADS="1")
    metadata <- file.path(root, "metadata")
    identity <- deseq_source_identity(source_path, metadata)
    for (path in c(script, helper)) {
        if (!file.copy(path, file.path(metadata, basename(path)))) stop("cannot retain source driver/helper")
    }
    if (!file.copy(file.path(source_path, "DESCRIPTION"), file.path(metadata, "source-DESCRIPTION"))) stop("cannot retain source description")
    installed <- system.file(package="DESeq2", mustWork=TRUE)
    installed_paths <- sort(list.files(installed, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
    installed_hashes <- stats::setNames(vapply(file.path(installed, installed_paths), deseq_hash, character(1)), installed_paths)
    write.table(data.frame(sha256=unname(installed_hashes), path=names(installed_hashes)),
        file.path(metadata, "installed-DESeq2-sha256.tsv"), sep="\t", row.names=FALSE, quote=FALSE)
    dll <- getLoadedDLLs()[["DESeq2"]][["path"]]
    runtime <- list(R=as.character(getRversion()), Bioconductor=as.character(BiocManager::version()),
        DESeq2=as.character(utils::packageVersion("DESeq2")), platform=R.version$platform,
        system=as.list(Sys.info()), external_software=as.list(extSoftVersion()),
        BLAS_LAPACK=capture.output(La_library()), locale=Sys.getlocale(),
        contrasts=as.list(getOption("contrasts")), installed_path=installed,
        installed_dll=dll, installed_dll_sha256=deseq_hash(dll),
        thread_environment=as.list(Sys.getenv(c("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"))))
    deseq_write_json(runtime, file.path(metadata, "runtime.json"))
    writeLines(capture.output(sessionInfo()), file.path(metadata, "session.txt"))
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
            write.table(row, file.path(metadata, "conditions.tsv"), sep="\t", quote=TRUE,
                row.names=FALSE, col.names=length(conditions) == 1L, append=length(conditions) > 1L)
        }
        withCallingHandlers(force(expression), warning=function(value) note(value, "warning"),
            message=function(value) note(value, "message"), error=function(value) note(value, "error"))
    }
    values <- design_inputs(root)
    inventory <- design_case_inventory()
    prepared <- lapply(inventory, function(spec) design_prepare_case(spec, values[[spec$input]], root))
    deseq_write_json(lapply(prepared, function(value) value$info), file.path(root, "inputs/case-inventory.json"))
    parameters <- list(test="Wald", fitType="parametric with actual source fallback", sfType="ratio",
        betaPrior=FALSE, useT=FALSE, minmu=0.5, parallel=FALSE, alpha=0.1,
        independentFiltering=TRUE, pAdjustMethod="BH", cooksCutoff="omitted: actual source default",
        unmasked_diagnostic_cooksCutoff=FALSE, transform="varianceStabilizingTransformation(blind=FALSE)")
    deseq_write_json(parameters, file.path(root, "inputs/public-call-parameters.json"))
    bounds <- list(normalization=c(absolute=1e-10, relative=1e-10), dispersion=c(absolute=1e-6, relative=1e-4),
        coefficient=c(absolute=1e-4, relative=1e-5), probability=c(absolute=1e-7, relative=1e-4),
        mu=c(absolute=1e-6, relative=1e-5), diagnostic=c(absolute=1e-6, relative=1e-4), vst=c(absolute=1e-5, relative=1e-6))
    deseq_write_json(bounds, file.path(root, "inputs/native-comparison-bounds.json"))
    frozen_files <- sort(c(list.files(file.path(root, "inputs"), recursive=TRUE, full.names=TRUE),
        unlist(lapply(prepared, function(value) list.files(file.path(value$directory, "inputs"), recursive=TRUE, full.names=TRUE)))), method="radix")
    frozen_hashes <- stats::setNames(vapply(frozen_files, deseq_hash, character(1)), substring(frozen_files, nchar(root) + 2L))
    deseq_write_json(list(status="frozen-before-first-fit", driver_sha256=deseq_hash(script),
        helper_sha256=deseq_hash(helper), case_count=length(inventory), general_case_count=7L, cook_case_count=8L,
        files=as.list(frozen_hashes)), file.path(metadata, "input-freeze.json"))
    cat("Frozen all inputs and 15 cases before the first source fit\n")
    cases <- lapply(prepared, function(value) {
        tryCatch(design_run_case(value, observe), error=function(error) {
            failure <- list(id=value$spec$id, status="failed", class=array(class(error)),
                message=conditionMessage(error), call=paste(deparse(conditionCall(error)), collapse=" "))
            deseq_write_json(failure, file.path(value$directory, "failure.json"))
            failure
        })
    })
    names(cases) <- vapply(inventory, function(value) value$id, character(1))
    if (!identical(frozen_hashes, stats::setNames(vapply(frozen_files, deseq_hash, character(1)), names(frozen_hashes)))) {
        stop("frozen input bytes changed during source fits")
    }
    if (!identical(installed_hashes, stats::setNames(vapply(file.path(installed, installed_paths), deseq_hash, character(1)), installed_paths))) {
        stop("installed package bytes changed during source fits")
    }
    coverage <- design_coverage(cases)
    failed <- names(cases)[vapply(cases, function(value) value$status != "complete", logical(1))]
    files <- sort(list.files(root, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
    hashes <- stats::setNames(vapply(file.path(root, files), deseq_hash, character(1)), files)
    record <- list(schema_version=1L,
        status=if (length(failed)) "failed" else if (!coverage$complete) "incomplete-coverage" else "complete",
        package=pin, runtime=runtime, source=identity, parameters=parameters,
        inputs_frozen_before_first_fit=TRUE, input_freeze_sha256=deseq_hash(file.path(metadata, "input-freeze.json")),
        case_count=length(cases), general_case_count=7L, cook_case_count=8L, failed_cases=array(failed),
        cases=cases, cook_qualification=coverage,
        conditions=list(count=length(conditions), warnings=sum(vapply(conditions, function(value) value$kind == "warning", logical(1))),
            errors=sum(vapply(conditions, function(value) value$kind == "error", logical(1)))),
        exclusions=array(c("native implementation or worker/platform acceptance", "performance or biological validation",
            "local trend and low-residual-df release coverage", "formula grammar outside the declared product interface")),
        files=as.list(hashes))
    deseq_write_json(record, file.path(root, "witness.json"))
    manifest_files <- sort(c(files, "witness.json"), method="radix")
    write.table(data.frame(sha256=vapply(file.path(root, manifest_files), deseq_hash, character(1)), path=manifest_files),
        file.path(root, "sha256-manifest.tsv"), sep="\t", quote=FALSE, row.names=FALSE)
    if (length(failed)) stop("actual source workflows failed: ", paste(failed, collapse=", "))
    if (!coverage$complete) stop("actual source branch coverage is uncovered: ", paste(coverage$uncovered, collapse=", "))
    cat("Completed seven general-design and eight Cook workflows with observed source branch coverage\n")
}

if (sys.nframe() == 0L) design_workflows_main()
