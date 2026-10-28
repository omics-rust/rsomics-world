#!/usr/bin/env Rscript

local_definition <- function(path, name) {
    for (expression in parse(path, keep.source=FALSE)) {
        if (is.call(expression) && length(expression) == 3L && is.symbol(expression[[1L]]) &&
            as.character(expression[[1L]]) %in% c("<-", "=") &&
            (is.symbol(expression[[2L]]) || is.character(expression[[2L]])) &&
            identical(as.character(expression[[2L]]), name) && is.call(expression[[3L]]) &&
            identical(expression[[3L]][[1L]], as.name("function"))) return(expression[[3L]])
    }
    stop("missing pinned function definition: ", name)
}

local_hashes <- function(root) {
    files <- sort(list.files(root, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
    stats::setNames(vapply(file.path(root, files), deseq_hash, character(1)), files)
}

local_assert_hashes <- function(root, hashes) {
    current <- stats::setNames(vapply(file.path(root, names(hashes)), deseq_hash, character(1)), names(hashes))
    if (!identical(hashes, current)) stop("frozen input/source/model bytes changed: ", root)
}

local_function_identity <- function(name, installed, expected, output) {
    if (!identical(deparse(formals(installed)), deparse(expected[[2L]])) ||
        !identical(deparse(body(installed)), deparse(expected[[3L]]))) {
        stop("installed function differs from pinned source: ", name)
    }
    writeLines(c("formals:", deparse(formals(installed)), "body:", deparse(body(installed))),
        file.path(output, paste0("function-", name, ".txt")))
}

local_source_identity <- function(deseq_source, locfit_source, output) {
    if (as.character(getRversion()) != "4.6.1" ||
        as.character(utils::packageVersion("DESeq2")) != "1.52.0" ||
        utils::packageDescription("locfit", fields="Version") != "1.5-9.12" ||
        as.character(BiocManager::version()) != "3.23") stop("installed runtime/package version differs")
    pins <- c(DESEQ2_SOURCE_ARCHIVE="8c91699286336350e66eec132ce6fdf5bb4af78e2a4d015a5a61224f62a95984",
        LOCFIT_SOURCE_ARCHIVE="fa31b9d407f3557a6fcb7105cbd7b16a32a6fb868a7c60bc38a3e33cb7a39343")
    for (name in names(pins)) {
        path <- Sys.getenv(name)
        if (!nzchar(path) || !identical(deseq_hash(path), unname(pins[[name]]))) stop("source archive identity differs: ", name)
    }
    if (Sys.getenv("DESEQ2_SOURCE_COMMIT") != "16aeab6d6158bd8cbb8e98764c2f33399b1a3fd3" ||
        file.info(Sys.getenv("LOCFIT_SOURCE_ARCHIVE"))$size != 207112) stop("source acquisition identity differs")
    identity <- deseq_source_identity(deseq_source, output)
    for (name in c("parametricDispersionFit", "localDispersionFit", "momentsDispEstimate")) {
        local_function_identity(name, get(name, asNamespace("DESeq2"), inherits=FALSE),
            deseq_definition(file.path(deseq_source, "R/core.R"), name), output)
    }
    files <- c(DESCRIPTION="08f53ba618ceed9b19936d312ae817885e3cf18ed16f31f06ce6c4ab52804586",
        NAMESPACE="a949dbe1ae2a1490f93b746065f9378435003770a49c23b9d293cf210beb253b",
        "R/locfit.r"="fb81214de10e54bd41334c12fc6a7059b04305f0c88dbe8eb2e13192d3db6895")
    local_assert_hashes(locfit_source, files)
    if (read.dcf(file.path(locfit_source, "DESCRIPTION"))[1L, "Version"] != "1.5-9.12") stop("locfit source version differs")
    namespace <- asNamespace("locfit")
    functions <- c("locfit", "locfit.raw", "rbox", "predict.locfit", "preplot.locfit", "preplot.locfit.raw")
    for (name in functions) {
        local_function_identity(paste0("locfit-", name), get(name, namespace, inherits=FALSE),
            local_definition(file.path(locfit_source, "R/locfit.r"), name), output)
    }
    for (generic in c("predict", "preplot")) {
        if (!identical(utils::getS3method(generic, "locfit", envir=namespace),
            get(paste0(generic, ".locfit"), namespace, inherits=FALSE))) stop("registered locfit method differs: ", generic)
    }
    if (!identical(eval(formals(stats::splinefun)$method)[[1L]], "fmm")) stop("unexpected R splinefun default")
    writeLines(c("formals:", deparse(formals(stats::splinefun)), "body:", deparse(body(stats::splinefun))),
        file.path(output, "function-stats-splinefun.txt"))
    for (entry in list(c(deseq_source, "DESeq2"), c(locfit_source, "locfit"))) {
        if (!file.copy(file.path(entry[[1L]], "DESCRIPTION"), file.path(output, paste0(entry[[2L]], "-source-DESCRIPTION")))) {
            stop("cannot preserve source description")
        }
    }
    list(DESeq2=identity, locfit=list(version="1.5-9.12", files=as.list(files), functions=array(functions)),
        archives=as.list(pins), runtime="R4.6.1/Bioc3.23", local_source_modified=FALSE)
}

local_runtime <- function(output) {
    packages <- lapply(c("DESeq2", "locfit"), function(name) {
        path <- normalizePath(system.file(package=name, mustWork=TRUE), mustWork=TRUE)
        hashes <- local_hashes(path)
        dll <- normalizePath(getLoadedDLLs()[[name]][["path"]], mustWork=TRUE)
        write.table(data.frame(sha256=unname(hashes), path=names(hashes)),
            file.path(output, paste0("installed-", name, "-sha256.tsv")), sep="\t", quote=FALSE, row.names=FALSE)
        list(path=path, files=hashes, dll=dll, dll_sha256=deseq_hash(dll),
             version=utils::packageDescription(name, fields="Version"))
    })
    names(packages) <- c("DESeq2", "locfit")
    libraries <- lapply(getLoadedDLLs(), function(value) {
        path <- value[["path"]]
        list(path=path, sha256=if (file.exists(path)) deseq_hash(path) else NULL)
    })
    libR <- normalizePath(file.path(R.home("lib"), "libR.so"), mustWork=TRUE)
    maps <- readLines("/proc/self/maps", warn=FALSE)
    if (!any(grepl(libR, maps, fixed=TRUE))) stop("pinned libR is not mapped in the source process")
    writeLines(maps, file.path(output, "loaded-process-maps.txt"))
    writeLines(capture.output(getDLLRegisteredRoutines("locfit")), file.path(output, "locfit-registered-routines.txt"))
    writeLines(capture.output(sessionInfo()), file.path(output, "session.txt"))
    installed <- utils::installed.packages()[, c("Package", "Version", "LibPath"), drop=FALSE]
    write.table(installed[order(installed[, "Package"], method="radix"), , drop=FALSE],
        file.path(output, "packages.tsv"), sep="\t", quote=TRUE, row.names=FALSE)
    makeconf <- file.path(R.home("etc"), "Makeconf")
    if (!file.copy(makeconf, file.path(output, "R-Makeconf"))) stop("cannot preserve R compiler configuration")
    runtime <- list(R=as.character(getRversion()), Bioconductor=as.character(BiocManager::version()),
        platform=R.version$platform, system=as.list(Sys.info()), process_id=Sys.getpid(),
        external_software=as.list(extSoftVersion()), BLAS_LAPACK=capture.output(La_library()),
        locale=Sys.getlocale(), contrasts=as.list(getOption("contrasts")), packages=packages,
        libR=list(path=libR, sha256=deseq_hash(libR)), loaded_libraries=libraries,
        thread_environment=as.list(Sys.getenv(c("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"))),
        compiler_environment=as.list(Sys.getenv(c("CC", "CFLAGS", "CPPFLAGS", "LDFLAGS", "R_MAKEVARS_USER"))))
    deseq_write_json(runtime, file.path(output, "runtime.json"))
    saveRDS(runtime, file.path(output, "runtime.rds"))
    runtime
}

local_assert_packages <- function(runtime) {
    for (package in runtime$packages) local_assert_hashes(package$path, package$files)
    if (!identical(deseq_hash(runtime$libR$path), runtime$libR$sha256)) stop("libR bytes changed")
}

local_prepare_inputs <- function(source, root) {
    pins <- c(pasilla_gene_counts.tsv.gz="4b5570008936786da365d110df1c83c8cffc646d6c667d6b15ecbfb3d9ee735d",
        pasilla_sample_annotation.csv="1844f871ca5d750f41685c73b87bd7dda291ec0d52304c1afb46a17ae88db745")
    directory <- file.path(root, "inputs/pasilla")
    if (!dir.create(directory, recursive=TRUE)) stop("cannot create pasilla input directory")
    for (name in names(pins)) {
        original <- system.file("extdata", name, package="DESeq2", mustWork=TRUE)
        if (!identical(deseq_hash(original), unname(pins[[name]])) ||
            !identical(deseq_hash(file.path(source, "inst/extdata", name)), unname(pins[[name]])) ||
            !file.copy(original, file.path(directory, name))) stop("shipped pasilla input differs: ", name)
    }
    counts <- as.matrix(read.delim(file.path(directory, names(pins)[[1L]]), row.names="gene_id", check.names=FALSE))
    meta <- read.csv(file.path(directory, names(pins)[[2L]]), row.names=1L, check.names=FALSE)
    mapped <- sub("fb$", "", rownames(meta))
    if (!identical(dim(counts), c(14599L, 7L)) || !is.integer(counts) || sum(rowSums(counts) == 0) != 2240L ||
        anyDuplicated(mapped) || !setequal(mapped, colnames(counts))) stop("pasilla identity/shape differs")
    design_table(data.frame(original_id=rownames(meta), sample=mapped), file.path(directory, "sample-id-mapping.tsv"))
    rownames(meta) <- mapped
    meta <- meta[colnames(counts), c("type", "condition"), drop=FALSE]
    meta$type <- factor(meta$type, levels=c("paired-end", "single-read"))
    meta$condition <- factor(meta$condition, levels=c("untreated", "treated"))
    if (anyNA(meta)) stop("unknown pasilla factor level")
    values <- list(pasilla=list(counts=counts, meta=meta, sf=NULL, nf=NULL))

    RNGkind("Mersenne-Twister", "Inversion", "Rejection")
    set.seed(20261028L)
    before <- .Random.seed
    genes <- sprintf("gene%04d", seq_len(1200L))
    samples <- c(sprintf("control%02d", seq_len(6L)), sprintf("treated%02d", seq_len(6L)))
    metadata <- data.frame(condition=factor(rep(c("control", "treated"), each=6L), levels=c("control", "treated")), row.names=samples)
    t <- (seq_len(1184L) - 1) / 1183
    base <- exp(log(2) + t * log(2500))
    dispersion <- 0.03 + 0.55 * t
    exposure <- stats::setNames(c(.70,.85,1,1.15,1.30,1.45,.75,.90,1.05,1.20,1.35,1.50), samples)
    mu <- outer(base, exposure)
    selected <- seq_len(1184L) %% 4L == 0L
    mu[selected, 7:12] <- mu[selected, 7:12] * 1.4
    dimnames(mu) <- list(genes[seq_len(1184L)], samples)
    drawn <- design_counts(mu, dispersion, rownames(mu), samples)
    counts <- rbind(drawn, matrix(0L, nrow=16L, ncol=12L, dimnames=list(genes[1185:1200], samples)))
    after <- .Random.seed
    nf <- exp(.18 * outer(sin(seq_len(1200L) / 11), (seq_len(12L) - 6.5) / 5.5))
    nf <- sweep(nf, 2L, exposure, "*")
    dimnames(nf) <- dimnames(counts)
    values$increasing <- list(counts=counts, meta=metadata, sf=2 * exposure, nf=nf)
    generation <- file.path(root, "inputs/generation")
    if (!dir.create(generation, recursive=TRUE)) stop("cannot create generation directory")
    design_table(data.frame(t=t, baseline=base, dispersion=dispersion, treated_multiplier=ifelse(selected, 1.4, 1)),
        file.path(generation, "parameters.tsv"), genes[seq_len(1184L)])
    design_table(mu, file.path(generation, "negative-binomial-means.tsv"), rownames(mu))
    design_table(data.frame(exposure=unname(exposure)), file.path(generation, "exposures.tsv"), samples, "sample")
    saveRDS(list(before=before, after=after), file.path(generation, "RNG-state.rds"))
    deseq_write_json(list(seed=20261028L, RNGkind=array(RNGkind()), NB_rows=1184L, preset_zero_rows=16L,
        draw_order="column-major, gene1..1184 for control01..06 then treated01..06",
        counts="rnbinom(mu,size=1/(.03+.55*t))", baseline="exp(log(2)+t*log(2500))",
        t="(g-1)/1183", treated_multiplier="1.4 if g divisible by4, else1",
        NF="E[j]*exp(.18*sin(g/11)*((j-6.5)/5.5)), including zero rows", SF="2*E; not recentered",
        prefilter=FALSE, reseed_or_retune=FALSE), file.path(generation, "generator.json"))
    for (name in names(values)) {
        value <- values[[name]]
        directory <- file.path(root, "inputs", name)
        if (!dir.exists(directory) && !dir.create(directory)) stop("cannot create input directory")
        design_table(value$counts, file.path(directory, "counts.tsv"), rownames(value$counts))
        design_table(value$meta, file.path(directory, "samples.tsv"), rownames(value$meta), "sample")
        if (!is.null(value$sf)) design_table(data.frame(factor=unname(value$sf)), file.path(directory, "size-factors.tsv"), names(value$sf), "sample")
        if (!is.null(value$nf)) design_table(value$nf, file.path(directory, "normalization-factors.tsv"), rownames(value$nf))
        saveRDS(value, file.path(directory, "inputs.rds"))
    }
    ids <- c("pasilla-local-ratio", "increasing-default-ratio", "increasing-local-nf", "increasing-local-sf-nf")
    prepared <- lapply(seq_along(ids), function(index) {
        name <- if (index == 1L) "pasilla" else "increasing"
        formula <- if (index == 1L) "~type+condition" else "~condition"
        matrix <- stats::model.matrix(stats::as.formula(formula), values[[name]]$meta)
        forward <- stats::setNames(rep(0, ncol(matrix)), colnames(matrix))
        forward[["conditiontreated"]] <- 1
        mixed <- forward
        mixed[["(Intercept)"]] <- -1
        spec <- list(id=ids[[index]], input=name, formula=formula, route="formula",
            normalization=c("ratio", "ratio", "nf", "sf-nf")[[index]], replacement="default",
            fit_type=if (index == 2L) "parametric" else "local", contrasts=list(forward=forward, mixed=mixed))
        design_prepare_case(spec, values[[name]], root)
    })
    names(prepared) <- ids
    queries <- c(.1,.5,1,2,5,10,20,50,100,200,500,1000,2000,5000,10000,50000,100000)
    design_table(data.frame(mean=queries), file.path(root, "inputs/positive-queries.tsv"), seq_along(queries), "query")
    saveRDS(queries, file.path(root, "inputs/positive-queries.rds"))
    deseq_write_json(list(extra=array(queries), gene_queries="all original nonzero gene baseMeans",
        VST_queries="source's999 positive sinh/asinh grid; max normalized count determines endpoint",
        log_trend_at_zero=FALSE), file.path(root, "inputs/query-policy.json"))
    deseq_write_json(lapply(prepared, function(value) c(value$info, list(requested_fit=value$spec$fit_type))),
        file.path(root, "inputs/case-inventory.json"))
    saveRDS(prepared, file.path(root, "inputs/prepared-cases.rds"))
    prepared
}

local_state <- function(fit) {
    trend <- DESeq2::dispersionFunction(fit)
    model <- if (identical(attr(trend, "fitType"), "local")) get("fit", environment(trend), inherits=FALSE) else NULL
    list(fit=deseq_fit_state(fit), locfit=if (is.null(model)) NULL else
        model[c("eva", "cell", "nvc", "box", "sty", "deriv", "mi", "dp", "vnames", "yname")])
}

local_vst_trace <- function(fit, normalized, authoritative, directory) {
    if (!identical(attr(DESeq2::dispersionFunction(fit), "fitType"), "local")) return(list(status="not-local"))
    if (!dir.create(directory)) stop("cannot create VST trace directory")
    sf <- if (is.null(DESeq2::sizeFactors(fit))) exp(MatrixGenerics::colMeans(log(DESeq2::normalizationFactors(fit)))) else DESeq2::sizeFactors(fit)
    xg <- sinh(seq(asinh(0), asinh(max(normalized)), length.out=1000))[-1]
    xim <- mean(1 / sf)
    trend <- DESeq2::dispersionFunction(fit)(xg)
    base_vars <- trend * xg^2 + xim * xg
    integrand <- 1 / sqrt(base_vars)
    knots <- asinh((xg[-1] + xg[-length(xg)]) / 2)
    integral <- cumsum((xg[-1] - xg[-length(xg)]) * (integrand[-1] + integrand[-length(integrand)]) / 2)
    spline <- stats::splinefun(knots, integral)
    h1 <- stats::quantile(MatrixGenerics::rowMeans(normalized), .95)
    h2 <- stats::quantile(MatrixGenerics::rowMeans(normalized), .999)
    eta <- (log2(h2) - log2(h1)) / (spline(asinh(h2)) - spline(asinh(h1)))
    xi <- log2(h1) - eta * spline(asinh(h1))
    reconstructed <- sapply(colnames(DESeq2::counts(fit)), function(sample) eta * spline(asinh(normalized[, sample])) + xi)
    rownames(reconstructed) <- rownames(DESeq2::counts(fit))
    coefficients <- get("z", environment(spline), inherits=FALSE)
    saveRDS(spline, file.path(directory, "FMM-spline.rds"))
    saveRDS(coefficients, file.path(directory, "FMM-coefficients.rds"))
    for (name in names(coefficients)) design_table(data.frame(value=coefficients[[name]]),
        file.path(directory, paste0("FMM-", name, ".tsv")))
    design_table(data.frame(mean=xg, dispersion=trend, base_variance=base_vars, integrand=integrand),
        file.path(directory, "variance-grid.tsv"), seq_along(xg), "grid")
    design_table(data.frame(asinh_midpoint=knots, cumulative_trapezoid=integral),
        file.path(directory, "integral-knots.tsv"), seq_along(knots), "knot")
    design_table(data.frame(h1=unname(h1), h2=unname(h2), xim=xim, eta=unname(eta), xi=unname(xi)),
        file.path(directory, "calibration.tsv"))
    design_table(data.frame(factor=unname(sf)), file.path(directory, "VST-size-factors.tsv"), colnames(fit), "sample")
    design_table(reconstructed, file.path(directory, "reconstructed-vst.tsv"), rownames(fit))
    nf <- DESeq2::normalizationFactors(fit)
    nz <- !S4Vectors::mcols(fit)$allZero
    moments_xim <- if (is.null(nf)) mean(1 / DESeq2::sizeFactors(fit)) else mean(1 / MatrixGenerics::colMeans(nf[nz, , drop=FALSE]))
    trace <- list(status="local", sf=sf, sf_policy=if (is.null(DESeq2::sizeFactors(fit))) "NF-geometric-all-rows" else "existing-SF",
        xim=xim, fit_moments_xim=moments_xim, grid=xg, dispersion=trend, base_variance=base_vars,
        integrand=integrand, knots=knots, integral=integral, FMM=coefficients,
        h1=h1, h2=h2, eta=eta, xi=xi, reconstructed=reconstructed,
        authoritative_identical=identical(authoritative, reconstructed))
    saveRDS(trace, file.path(directory, "trace.rds"))
    if (!trace$authoritative_identical) stop("local VST source-expression trace differs from actual public VST")
    trace
}

local_derive <- function(fit, prepared, queries, output, observe) {
    if (!dir.create(output)) stop("cannot create derived output directory")
    id <- prepared$spec$id
    before <- local_state(fit)
    results <- list()
    contrasts <- list(forward=prepared$forward, reverse=-prepared$forward, mixed=prepared$mixed)
    for (name in names(contrasts)) {
        value <- observe(id, paste(name, "results"), DESeq2::results(fit, contrast=unname(contrasts[[name]]),
            alpha=.1, independentFiltering=TRUE, pAdjustMethod="BH", parallel=FALSE))
        results[[name]] <- list(table=as.data.frame(value), metadata=S4Vectors::metadata(value))
        saveRDS(value, file.path(output, paste0(name, ".rds")))
        design_table(value, file.path(output, paste0(name, ".tsv")), rownames(fit))
        saveRDS(S4Vectors::metadata(value), file.path(output, paste0(name, "-metadata.rds")))
        deseq_write_json(S4Vectors::metadata(value), file.path(output, paste0(name, "-metadata.json")))
    }
    normalized <- observe(id, "normalized counts", DESeq2::counts(fit, normalized=TRUE))
    saveRDS(normalized, file.path(output, "normalized-counts.rds"))
    design_table(normalized, file.path(output, "normalized-counts.tsv"), rownames(fit))
    transformed <- observe(id, "fitted VST", DESeq2::varianceStabilizingTransformation(fit, blind=FALSE))
    vst <- SummarizedExperiment::assay(transformed)
    saveRDS(transformed, file.path(output, "vst.rds"))
    design_table(vst, file.path(output, "vst.tsv"), rownames(fit))
    trace <- observe(id, "local VST numerical trace", local_vst_trace(fit, normalized, vst, file.path(output, "vst-trace")))
    data <- S4Vectors::mcols(fit)
    gene_means <- data$baseMean[!data$allZero]
    grid <- sinh(seq(asinh(0), asinh(max(normalized)), length.out=1000))[-1]
    q <- c(gene_means, grid, queries)
    predicted <- observe(id, "saved trend queries", DESeq2::dispersionFunction(fit)(q))
    if (length(predicted) != length(q)) stop("source trend prediction length differs")
    table <- data.frame(kind=c(rep("gene", length(gene_means)), rep("VST-grid", length(grid)), rep("frozen-extra", length(queries))),
        identity=c(rownames(fit)[!data$allZero], as.character(seq_along(grid)), as.character(seq_along(queries))), mean=q, dispersion=predicted)
    if (identical(attr(DESeq2::dispersionFunction(fit), "fitType"), "local")) {
        model <- get("fit", environment(DESeq2::dispersionFunction(fit)), inherits=FALSE)
        table$predicted_log_dispersion <- observe(id, "locfit registered prediction", stats::predict(model, data.frame(logMeans=log(q))))
        if (!identical(exp(table$predicted_log_dispersion), predicted)) stop("locfit prediction differs from saved trend closure")
    }
    design_table(table, file.path(output, "trend-queries.tsv"))
    value <- list(results=results, normalized=normalized, vst=vst, trend=table, VST_trace=trace)
    saveRDS(value, file.path(output, "derived-objects.rds"))
    if (!identical(before, local_state(fit))) stop("derived source calls changed fitted state")
    value
}

local_fit_case <- function(prepared, queries, observe) {
    spec <- prepared$spec
    input <- prepared$input
    directory <- file.path(prepared$directory, "fit")
    if (!dir.create(directory)) stop("cannot create fitted model directory")
    dataset <- observe(spec$id, "construct", DESeq2::DESeqDataSetFromMatrix(input$counts, input$meta, prepared$formula))
    if (spec$normalization == "sf-nf") DESeq2::sizeFactors(dataset) <- unname(input$sf[colnames(dataset)])
    if (spec$normalization %in% c("nf", "sf-nf")) DESeq2::normalizationFactors(dataset) <- input$nf[rownames(dataset), colnames(dataset), drop=FALSE]
    fit <- observe(spec$id, "DESeq", DESeq2::DESeq(dataset, test="Wald", fitType=spec$fit_type, sfType="ratio",
        betaPrior=FALSE, minReplicatesForReplace=7L, useT=FALSE, minmu=.5, parallel=FALSE))
    saveRDS(fit, file.path(directory, "DESeqDataSet.rds"))
    state <- local_state(fit)
    saveRDS(state, file.path(directory, "complete-state.rds"))
    saveRDS(DESeq2::dispersionFunction(fit), file.path(directory, "dispersion-function.rds"))
    if (!identical(DESeq2::counts(fit), input$counts) ||
        !identical(dim(attr(fit, "modelMatrix")), dim(prepared$matrix)) ||
        !all(attr(fit, "modelMatrix") == prepared$matrix)) {
        stop("actual source fit changed original counts or model matrix")
    }
    omitted <- design_table(S4Vectors::mcols(fit), file.path(directory, "row-diagnostics.tsv"), rownames(fit))
    design_table(stats::coef(fit), file.path(directory, "coefficients.tsv"), rownames(fit))
    design_table(attr(fit, "modelMatrix"), file.path(directory, "model-matrix.tsv"), colnames(fit), "sample")
    design_table(attr(fit, "dispModelMatrix"), file.path(directory, "dispersion-model-matrix.tsv"), colnames(fit), "sample")
    design_table(data.frame(raw=colnames(prepared$matrix), coefficient=DESeq2::resultsNames(fit)),
        file.path(directory, "coefficient-map.tsv"))
    for (name in SummarizedExperiment::assayNames(fit)) {
        if (!grepl("^[A-Za-z0-9_.-]+$", name)) stop("unexpected assay name")
        design_table(SummarizedExperiment::assays(fit)[[name]], file.path(directory, paste0("assay-", name, ".tsv")), rownames(fit))
    }
    sf <- DESeq2::sizeFactors(fit)
    nf <- DESeq2::normalizationFactors(fit)
    if (!is.null(sf)) design_table(data.frame(factor=sf), file.path(directory, "size-factors.tsv"), colnames(fit), "sample")
    if (!is.null(nf)) design_table(nf, file.path(directory, "normalization-factors.tsv"), rownames(fit))
    trend <- DESeq2::dispersionFunction(fit)
    actual <- attr(trend, "fitType")
    saveRDS(attributes(trend), file.path(directory, "dispersion-attributes.rds"))
    if (identical(actual, "local")) {
        model <- get("fit", environment(trend), inherits=FALSE)
        means <- get("means", environment(trend), inherits=FALSE)
        disps <- get("disps", environment(trend), inherits=FALSE)
        min_disp <- get("minDisp", environment(trend), inherits=FALSE)
        row <- S4Vectors::mcols(fit)
        selected <- !row$allZero & !is.na(row$dispGeneEst) & row$dispGeneEst > 100 * min_disp
        if (!identical(unname(means), unname(row$baseMean[selected])) ||
            !identical(unname(disps), unname(row$dispGeneEst[selected])) || !inherits(model, "locfit")) stop("captured local training state differs")
        saveRDS(model, file.path(directory, "locfit-model.rds"))
        design_table(data.frame(mean=means, dispersion=disps, log_mean=log(means), log_dispersion=log(disps),
            mean_weight=means, internal_selected=disps >= 10 * min_disp), file.path(directory, "locfit-training.tsv"), rownames(fit)[selected])
        design_table(model$eva$coef, file.path(directory, "locfit-evaluation-coefficients.tsv"))
        for (name in c("nvc", "box", "sty", "deriv", "mi", "dp")) {
            value <- model[[name]]
            labels <- names(value)
            if (is.null(labels)) labels <- as.character(seq_along(value))
            design_table(data.frame(name=labels, value=unname(value)), file.path(directory, paste0("locfit-", name, ".tsv")))
        }
    }
    derived <- local_derive(fit, prepared, queries, file.path(prepared$directory, "results"), observe)
    list(id=spec$id, status="complete", requested_fit=spec$fit_type, actual_fit=actual,
        omitted_plain_diagnostics=array(omitted), genes=nrow(fit), samples=ncol(fit),
        local_VST_trace=identical(derived$VST_trace$status, "local") && derived$VST_trace$authoritative_identical,
        nonfinite_trend_queries=sum(!is.finite(derived$trend$dispersion)))
}

local_failure <- function(id, error, path) {
    failure <- list(id=id, status="failed", class=array(class(error)), message=conditionMessage(error),
        call=paste(deparse(conditionCall(error)), collapse=" "))
    deseq_write_json(failure, path)
    failure
}

local_reload_case <- function(prepared, queries, observe) {
    fit_path <- file.path(prepared$directory, "fit/DESeqDataSet.rds")
    before <- readRDS(file.path(prepared$directory, "fit/complete-state.rds"))
    fit_hash <- deseq_hash(fit_path)
    fit <- readRDS(fit_path)
    if (!identical(before, local_state(fit))) stop("fresh-process reloaded fit state differs")
    repeated <- local_derive(fit, prepared, queries, file.path(prepared$directory, "reload"), observe)
    original <- readRDS(file.path(prepared$directory, "results/derived-objects.rds"))
    if (!identical(original, repeated) || !identical(fit_hash, deseq_hash(fit_path))) stop("fresh-process outputs/model bytes differ")
    list(id=prepared$spec$id, status="complete", full_state_identical=TRUE, all_derived_outputs_identical=TRUE,
        saved_model_bytes_unchanged=TRUE, fitting_calls=FALSE, input_regeneration=FALSE)
}

local_coverage <- function(prepared, fits, reloads, conditions) {
    complete <- vapply(fits, function(value) value$status == "complete", logical(1))
    required <- c(all_four_fits=all(complete), all_four_fresh_reloads=all(vapply(reloads, function(value) value$status == "complete", logical(1))),
        explicit_local=all(vapply(fits[c(1L, 3L, 4L)], function(value) identical(value$actual_fit, "local"), logical(1))),
        actual_default_fallback=FALSE, NF_priority_same_inference=FALSE, NF_only_VST_proxy=FALSE,
        supplied_SF_VST_preference=FALSE, VST_xim_distinct=FALSE, finite_VST_values_distinct=FALSE)
    fallback_message <- any(vapply(conditions, function(value) value$case == "increasing-default-ratio" &&
        value$operation == "DESeq" && value$kind == "message" &&
        grepl("a local regression fit was automatically substituted.", value$message, fixed=TRUE), logical(1)))
    required[["actual_default_fallback"]] <- identical(fits[[2L]]$actual_fit, "local") && fallback_message
    nf <- list(status="unavailable")
    if (all(complete[c(3L, 4L)])) {
        left <- readRDS(file.path(prepared[[3L]]$directory, "fit/complete-state.rds"))
        right <- readRDS(file.path(prepared[[4L]]$directory, "fit/complete-state.rds"))
        a <- readRDS(file.path(prepared[[3L]]$directory, "results/derived-objects.rds"))
        b <- readRDS(file.path(prepared[[4L]]$directory, "results/derived-objects.rds"))
        fields <- c("counts", "design", "model_matrix", "dispersion_model_matrix", "normalization_factors", "row_data", "assays", "dispersion_attributes", "dispersion_body")
        required[["NF_priority_same_inference"]] <- identical(left$fit[fields], right$fit[fields]) &&
            identical(left$locfit, right$locfit) && identical(a$normalized, b$normalized) && identical(a$results, b$results)
        required[["NF_only_VST_proxy"]] <- is.null(left$fit$size_factors) && identical(a$VST_trace$sf_policy, "NF-geometric-all-rows")
        required[["supplied_SF_VST_preference"]] <- identical(unname(right$fit$size_factors), unname(prepared[[4L]]$input$sf)) &&
            identical(b$VST_trace$sf_policy, "existing-SF")
        required[["VST_xim_distinct"]] <- !identical(a$VST_trace$xim, b$VST_trace$xim)
        delta <- a$vst - b$vst
        finite <- is.finite(delta)
        required[["finite_VST_values_distinct"]] <- any(delta[finite] != 0)
        nf <- list(status="observed", NF_only_xim=a$VST_trace$xim, SF_NF_xim=b$VST_trace$xim,
            NF_moments_xim=a$VST_trace$fit_moments_xim, finite_differences=sum(finite & delta != 0),
            max_abs_finite_difference=if (any(finite)) max(abs(delta[finite])) else NULL)
    }
    list(complete=all(required), observed=as.list(required), uncovered=array(names(required)[!required]),
        fallback_message_observed=fallback_message, normalization=nf, retuning_performed=FALSE)
}

local_workflows_main <- function() {
    args <- commandArgs(trailingOnly=TRUE)
    if (length(args) != 4L || !args[[1L]] %in% c("fit", "reload")) {
        stop("usage: deseq_local_trend_workflows.R fit|reload DESEQ_SOURCE_DIR LOCFIT_SOURCE_DIR OUTPUT_DIR")
    }
    mode <- args[[1L]]
    script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
    if (length(script) != 1L) stop("cannot identify source driver")
    script <- normalizePath(sub("^--file=", "", script), mustWork=TRUE)
    helpers <- file.path(dirname(script), c("deseq_workflow_oracle.R", "deseq_design_workflows.R"))
    for (helper in helpers) base::source(helper, local=.GlobalEnv)
    for (package in c("DESeq2", "locfit", "digest", "jsonlite", "BiocManager")) {
        if (!requireNamespace(package, quietly=TRUE)) stop("required installed package absent: ", package)
    }
    source <- normalizePath(args[[2L]], mustWork=TRUE)
    locfit_source <- normalizePath(args[[3L]], mustWork=TRUE)
    root <- normalizePath(args[[4L]], mustWork=FALSE)
    if (mode == "fit") {
        if (file.exists(root)) stop("output directory already exists")
        if (!dir.create(root, recursive=TRUE)) stop("cannot create output directory")
    } else root <- normalizePath(root, mustWork=TRUE)
    metadata <- file.path(root, "metadata", mode)
    if (!dir.create(metadata, recursive=TRUE)) stop("source phase output already exists or cannot be created")
    options(digits=17, scipen=0, warn=1, contrasts=c("contr.treatment", "contr.poly"))
    if (!identical(Sys.setlocale("LC_COLLATE", "C"), "C") || !identical(Sys.setlocale("LC_NUMERIC", "C"), "C")) stop("cannot establish comparison locale")
    Sys.setenv(OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", MKL_NUM_THREADS="1")
    identity <- local_source_identity(source, locfit_source, metadata)
    runtime <- local_runtime(metadata)
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
    if (mode == "fit") {
        for (path in c(script, helpers)) if (!file.copy(path, file.path(metadata, basename(path)))) stop("cannot preserve source controller")
        prepared <- local_prepare_inputs(source, root)
        parameters <- list(test="Wald", betaPrior=FALSE, useT=FALSE, minmu=.5, minReplicatesForReplace=7L,
            sfType="ratio", parallel=FALSE, alpha=.1, independentFiltering=TRUE, pAdjustMethod="BH",
            cooksCutoff="omitted: actual source default", VST="varianceStabilizingTransformation(blind=FALSE)")
        deseq_write_json(parameters, file.path(root, "inputs/public-call-parameters.json"))
        bounds <- list(normalization=c(absolute=1e-10, relative=1e-10), dispersion=c(absolute=1e-6, relative=1e-4),
            coefficient=c(absolute=1e-4, relative=1e-5), probability=c(absolute=1e-7, relative=1e-4),
            mu=c(absolute=1e-6, relative=1e-5), diagnostic=c(absolute=1e-6, relative=1e-4), vst=c(absolute=1e-5, relative=1e-6))
        deseq_write_json(bounds, file.path(root, "inputs/native-comparison-bounds.json"))
        frozen <- local_hashes(root)
        saveRDS(frozen, file.path(root, "metadata/input-freeze.rds"))
        deseq_write_json(list(status="frozen-before-first-fit", case_count=4L, files=as.list(frozen)), file.path(root, "metadata/input-freeze.json"))
        cat("Frozen all four cases, source identities, RNG state and queries before the first source fit\n")
        queries <- readRDS(file.path(root, "inputs/positive-queries.rds"))
        fits <- lapply(prepared, function(value) tryCatch(local_fit_case(value, queries, observe),
            error=function(error) local_failure(value$spec$id, error, file.path(value$directory, "fit-failure.json"))))
        local_assert_hashes(root, frozen)
        local_assert_packages(runtime)
        stage <- list(process_id=Sys.getpid(), cases=fits, conditions=conditions, source=identity)
        saveRDS(stage, file.path(root, "fit-stage.rds"))
        deseq_write_json(stage, file.path(root, "fit-stage.json"))
        quit(status=if (all(vapply(fits, function(value) value$status == "complete", logical(1)))) 0L else 1L, save="no")
    }
    frozen <- readRDS(file.path(root, "metadata/input-freeze.rds"))
    local_assert_hashes(root, frozen)
    original_runtime <- readRDS(file.path(root, "metadata/fit/runtime.rds"))
    local_assert_packages(original_runtime)
    stage <- readRDS(file.path(root, "fit-stage.rds"))
    if (identical(stage$process_id, Sys.getpid())) stop("reload requires a fresh R process")
    prepared <- readRDS(file.path(root, "inputs/prepared-cases.rds"))
    queries <- readRDS(file.path(root, "inputs/positive-queries.rds"))
    reloads <- lapply(prepared, function(value) tryCatch(local_reload_case(value, queries, observe),
        error=function(error) local_failure(value$spec$id, error, file.path(value$directory, "reload-failure.json"))))
    coverage <- local_coverage(prepared, stage$cases, reloads, stage$conditions)
    local_assert_hashes(root, frozen)
    local_assert_packages(runtime)
    local_assert_packages(original_runtime)
    failed <- names(stage$cases)[vapply(seq_along(stage$cases), function(i) stage$cases[[i]]$status != "complete" || reloads[[i]]$status != "complete", logical(1))]
    witness <- list(schema_version=1L, status=if (length(failed)) "failed" else if (!coverage$complete) "incomplete-coverage" else "complete",
        source=identity, input_freeze_sha256=deseq_hash(file.path(root, "metadata/input-freeze.json")),
        all_inputs_frozen_before_fitting=TRUE, case_count=4L, fits=stage$cases, reloads=reloads, failed_cases=array(failed),
        fit_process_id=stage$process_id, reload_process_id=Sys.getpid(), fresh_process=TRUE, qualification=coverage,
        source_conditions=list(fit=stage$conditions, reload=conditions), nonfinite_numeric_state="preserved in RDS and17-digit text tables",
        exclusions=array(c("native implementation/dependency adoption", "performance/four-native/release acceptance",
            "positive local-trend Cook replacement combination", "new low-df/glmGamPoi/blind or subsampled VST coverage")))
    deseq_write_json(witness, file.path(root, "witness.json"))
    hashes <- local_hashes(root)
    write.table(data.frame(sha256=unname(hashes), path=names(hashes)), file.path(root, "sha256-manifest.tsv"), sep="\t", quote=FALSE, row.names=FALSE)
    cat("Source coverage status: ", witness$status, "; uncovered: ", paste(coverage$uncovered, collapse=", "), "\n", sep="")
    quit(status=if (length(failed)) 1L else if (!coverage$complete) 2L else 0L, save="no")
}

if (sys.nframe() == 0L) local_workflows_main()
