#!/usr/bin/env Rscript

if (!identical(as.character(getRversion()), "4.6.1") ||
    !identical(Sys.getenv("BIOCONDUCTOR_VERSION"), "3.23"))
    stop("R/Bioconductor image profile differs")

export_table <- function(value, path) {
    value <- as.data.frame(value)
    for (name in names(value)) {
        if (is.list(value[[name]]))
            value[[name]] <- vapply(value[[name]], function(x) paste(as.character(x), collapse=";"), "")
    }
    write.table(value, path, sep="\t", quote=TRUE, row.names=FALSE, na="NA")
}

snapshot <- function(output) {
    dir.create(output, recursive=TRUE, showWarnings=FALSE)
    saveRDS(list(session=sessionInfo(), options=options(), locale=Sys.getlocale(),
                 environment=Sys.getenv(c("R_VERSION", "BIOCONDUCTOR_VERSION", "BIOCONDUCTOR_DOCKER_VERSION",
                     "PEAK_CONTAINER_DIGEST", "R_LIBS_USER", "R_LIBS_SITE", "R_USER_CACHE_DIR",
                     "TMPDIR", "XDG_CACHE_HOME", "R_ENVIRON_USER", "R_PROFILE_USER",
                     "OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS")),
                 extSoftVersion=extSoftVersion(),
                 namespaces=loadedNamespaces(), DLLs=getLoadedDLLs()),
            file.path(output, "session-options-DLLs.rds"))
    capture.output(sessionInfo(), file=file.path(output, "session.txt"))
    packages <- installed.packages(fields=c("Priority", "Depends", "Imports", "LinkingTo"))
    fields <- c("Package", "Version", "Priority", "LibPath", "Depends", "Imports", "LinkingTo")
    write.table(packages[, fields, drop=FALSE], file.path(output, "installed.tsv"),
                sep="\t", quote=TRUE, qmethod="double", row.names=FALSE, na="")
}

capture_state <- function(environment, output) {
    state <- list()
    for (name in c("x0", "z0", "d0", "constant", "fdr"))
        if (exists(name, envir=environment, inherits=FALSE))
            state[[name]] <- get(name, envir=environment, inherits=FALSE)
    saveRDS(state, file.path(output, "threshold-objects.rds"))
    rows <- list()
    for (name in names(state)) {
        values <- state[[name]]
        rows[[name]] <- data.frame(object=name, index=seq_along(values),
            value=as.character(values),
            number_class=ifelse(is.na(values), ifelse(is.nan(values), "NaN", "NA"),
                ifelse(is.infinite(values), ifelse(values > 0, "+Inf", "-Inf"), "finite")))
    }
    if (!length(rows)) stop("source threshold objects are absent")
    export_table(do.call(rbind, rows), file.path(output, "thresholds.tsv"))
    state
}

export_annotation <- function(annotation, output) {
    data <- as.data.frame(annotation)
    saveRDS(data, file.path(output, "annotation-data.rds"))
    export_table(data, file.path(output, "annotation.tsv"))
    for (name in slotNames(annotation))
        saveRDS(slot(annotation, name), file.path(output, paste0("slot-", name, ".rds")))
    if ("annotation" %in% names(data))
        export_table(as.data.frame(table(data$annotation, useNA="always")),
                     file.path(output, "observed-annotation-classes.tsv"))
    data
}

annotate <- function(model, rows, output, format, downstream) {
    dir.create(output, recursive=TRUE, showWarnings=FALSE)
    warnings <- character()
    on.exit(saveRDS(warnings, file.path(output, "warnings.rds")))
    withCallingHandlers({
        if (!identical(as.character(packageVersion("ChIPseeker")), "1.48.0") ||
            !identical(as.character(packageVersion("txdbmaker")), "1.8.0"))
            stop("annotation package versions differ")
        chrominfo <- data.frame(chrom=c("chrA", "chrB"), length=c(10000000, 10000000),
                                is_circular=c(FALSE, FALSE))
        txdb <- txdbmaker::makeTxDbFromGFF(model, format=format, chrominfo=chrominfo,
                                         dataSource="peak-workflow-input-only")
        AnnotationDbi::saveDb(txdb, file.path(output, "TxDb.sqlite"))
        export_table(GenomicFeatures::transcripts(txdb, columns=c("tx_id", "tx_name", "gene_id")),
                     file.path(output, "transcripts.tsv"))
        export_table(GenomicFeatures::exons(txdb), file.path(output, "exons.tsv"))
        export_table(GenomicFeatures::cds(txdb), file.path(output, "CDS.tsv"))
        saveRDS(GenomicFeatures::exonsBy(txdb, by="tx", use.names=TRUE),
                file.path(output, "exons-by-transcript.rds"))
        saveRDS(GenomeInfoDb::seqinfo(txdb), file.path(output, "seqinfo.rds"))
        bed <- read.delim(file.path(rows, "workflow.bed"), header=FALSE, quote="",
                          comment.char="", stringsAsFactors=FALSE)
        ledger <- read.delim(file.path(rows, "workflow-rows.tsv"), quote="\"", comment.char="",
                             check.names=FALSE, stringsAsFactors=FALSE)
        stopifnot(nrow(bed) == nrow(ledger), !anyDuplicated(ledger$row_key))
        peaks <- GenomicRanges::GRanges(seqnames=bed[[1]],
            ranges=IRanges::IRanges(start=as.integer(bed[[2]]) + 1L, end=as.integer(bed[[3]])),
            strand=ifelse(bed[[6]] == ".", "*", bed[[6]]))
        names(peaks) <- ledger$row_key
        S4Vectors::mcols(peaks)$row_key <- ledger$row_key
        S4Vectors::mcols(peaks)$row_ordinal <- ledger$row_ordinal
        S4Vectors::mcols(peaks)$display_id <- bed[[4]]
        S4Vectors::mcols(peaks)$input_score <- bed[[5]]
        S4Vectors::mcols(peaks)$source <- ledger$source
        S4Vectors::mcols(peaks)$source_row <- ledger$source_row
        options(ChIPseeker.downstreamDistance=NULL)
        if (!identical(downstream, "unset"))
            options(ChIPseeker.downstreamDistance=as.numeric(downstream))
        saveRDS(list(downstream_option=getOption("ChIPseeker.downstreamDistance"),
                     tssRegion=c(-3000, 3000), level="transcript", annoDb=NULL),
                file.path(output, "invocation-options.rds"))
        saveRDS(peaks, file.path(output, "input-peaks.rds"))
        annotation <- ChIPseeker::annotatePeak(peaks, TxDb=txdb, level="transcript", annoDb=NULL)
        saveRDS(annotation, file.path(output, "csAnno.rds"))
        export_annotation(annotation, output)
        for (name in c("annotatePeak", "getGenomicAnnotation", "getNearestFeatureIndicesAndDistances"))
            capture.output(get(name, envir=asNamespace("ChIPseeker")),
                           file=file.path(output, paste0("installed-source-", name, ".R")))
        snapshot(output)
    }, warning=function(condition) warnings <<- c(warnings, conditionMessage(condition)))
}

reload_annotation <- function(original, output, downstream) {
    dir.create(output, recursive=TRUE, showWarnings=FALSE)
    stopifnot(identical(as.character(packageVersion("ChIPseeker")), "1.48.0"))
    object <- readRDS(file.path(original, "csAnno.rds"))
    expected <- readRDS(file.path(original, "annotation-data.rds"))
    stopifnot(identical(as.data.frame(object), expected))
    txdb <- AnnotationDbi::loadDb(file.path(original, "TxDb.sqlite"))
    peaks <- readRDS(file.path(original, "input-peaks.rds"))
    options(ChIPseeker.downstreamDistance=NULL)
    if (!identical(downstream, "unset"))
        options(ChIPseeker.downstreamDistance=as.numeric(downstream))
    actual <- ChIPseeker::annotatePeak(peaks, TxDb=txdb, level="transcript", annoDb=NULL)
    stopifnot(identical(as.data.frame(actual), expected))
    saveRDS(actual, file.path(output, "reannotated-csAnno.rds"))
    export_annotation(object, output)
    snapshot(output)
    writeLines("same-profile RDS/SQLite reload and fresh re-annotation identical",
               file.path(output, "reload-status.txt"))
}

main <- function(args) {
    if (length(args) < 2L) stop("mode and output arguments required")
    mode <- args[[1]]
    if (mode == "inventory") {
        snapshot(args[[2]])
    } else if (mode == "runtime") {
        stopifnot(identical(as.character(packageVersion("ChIPseeker")), "1.48.0"),
                  identical(as.character(packageVersion("txdbmaker")), "1.8.0"))
        loadNamespace("ChIPseeker")
        loadNamespace("txdbmaker")
        snapshot(args[[2]])
    } else if (mode == "annotate") {
        stopifnot(length(args) == 6L)
        annotate(args[[2]], args[[3]], args[[4]], args[[5]], args[[6]])
    } else if (mode == "reload") {
        stopifnot(length(args) == 4L)
        reload_annotation(args[[2]], args[[3]], args[[4]])
    } else if (mode == "seacr-reload") {
        stopifnot(length(args) == 3L)
        original <- args[[2]]
        output <- args[[3]]
        dir.create(output, recursive=TRUE, showWarnings=FALSE)
        environment <- new.env(parent=baseenv())
        load(file.path(original, "globals.RData"), envir=environment)
        restored <- capture_state(environment, output)
        stopifnot(identical(restored, readRDS(file.path(original, "threshold-objects.rds"))))
        snapshot(output)
        writeLines("same-profile full source-global reload identical", file.path(output, "reload-status.txt"))
    } else {
        stop("unknown source-witness mode")
    }
}

capture_seacr <- function() {
    capture_directory <- Sys.getenv("PEAK_SEACR_CAPTURE")
    dir.create(capture_directory, recursive=TRUE, showWarnings=FALSE)
    sys.source(Sys.getenv("PEAK_SEACR_SOURCE_R"), envir=.GlobalEnv)
    capture_state(.GlobalEnv, capture_directory)
    snapshot(capture_directory)
    save(list=ls(envir=.GlobalEnv, all.names=TRUE), file=file.path(capture_directory, "globals.RData"), envir=.GlobalEnv)
}

if (identical(Sys.getenv("PEAK_R_MODE"), "seacr-capture")) {
    capture_seacr()
} else {
    main(commandArgs(TRUE))
}
