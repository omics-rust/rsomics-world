#!/usr/bin/env Rscript

stopifnot(as.character(getRversion()) == "4.6.1",
          Sys.getenv("BIOCONDUCTOR_VERSION") == "3.23",
          as.character(packageVersion("ChIPseeker")) == "1.48.0",
          as.character(packageVersion("txdbmaker")) == "1.8.0")

export_table <- function(value, path) {
    value <- as.data.frame(value)
    for (name in names(value))
        if (is.list(value[[name]]))
            value[[name]] <- vapply(value[[name]], function(x) paste(as.character(x), collapse=";"), "")
    write.table(value, path, sep="\t", quote=TRUE, row.names=FALSE, na="NA")
}

snapshot <- function(output) {
    dir.create(output, recursive=TRUE, showWarnings=FALSE)
    saveRDS(list(session=sessionInfo(), options=options(), locale=Sys.getlocale(),
                 namespaces=loadedNamespaces(), DLLs=getLoadedDLLs(),
                 environment=Sys.getenv(c("R_LIBS_USER", "R_LIBS_SITE", "TMPDIR",
                     "R_USER_CACHE_DIR", "OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS",
                     "MKL_NUM_THREADS", "PEAK_CONTAINER_DIGEST"))),
            file.path(output, "session-options-DLLs.rds"))
    capture.output(sessionInfo(), file=file.path(output, "session.txt"))
    packages <- installed.packages(fields="Priority")
    export_table(packages[, c("Package", "Version", "Priority", "LibPath"), drop=FALSE],
                 file.path(output, "installed.tsv"))
    dlls <- getLoadedDLLs()
    export_table(data.frame(name=names(dlls),
        path=vapply(dlls, function(x) x[["path"]], "")), file.path(output, "DLLs.tsv"))
    export_table(data.frame(namespace=loadedNamespaces(),
        version=vapply(loadedNamespaces(), function(x) as.character(packageVersion(x)), "")),
        file.path(output, "namespaces.tsv"))
}

export_annotation <- function(annotation, output) {
    data <- as.data.frame(annotation)
    saveRDS(data, file.path(output, "annotation-data.rds"))
    export_table(data, file.path(output, "annotation.tsv"))
    for (name in slotNames(annotation))
        saveRDS(slot(annotation, name), file.path(output, paste0("slot-", name, ".rds")))
    export_table(as.data.frame(table(data$annotation, useNA="always")),
                 file.path(output, "observed-annotation-classes.tsv"))
}

annotate <- function(database, inputs, output, profile) {
    dir.create(output, recursive=TRUE, showWarnings=FALSE)
    warnings <- character()
    on.exit(saveRDS(warnings, file.path(output, "warnings.rds")))
    withCallingHandlers({
        txdb <- AnnotationDbi::loadDb(database)
        bed <- read.delim(file.path(inputs, "annotation.bed"), header=FALSE,
                          quote="", comment.char="", stringsAsFactors=FALSE)
        ledger <- read.delim(file.path(inputs, "annotation-rows.tsv"), quote="",
                             comment.char="", stringsAsFactors=FALSE)
        stopifnot(nrow(bed) == 10L, nrow(ledger) == nrow(bed), !anyDuplicated(ledger$row_key))
        peaks <- GenomicRanges::GRanges(seqnames=bed[[1]],
            ranges=IRanges::IRanges(start=as.integer(bed[[2]]) + 1L, end=as.integer(bed[[3]])),
            strand=bed[[6]])
        names(peaks) <- ledger$row_key
        S4Vectors::mcols(peaks)$row_key <- ledger$row_key
        S4Vectors::mcols(peaks)$row_ordinal <- ledger$row_ordinal
        S4Vectors::mcols(peaks)$display_id <- bed[[4]]
        options(ChIPseeker.downstreamDistance=NULL)
        arguments <- list(peak=peaks, TxDb=txdb, level="transcript", annoDb=NULL)
        if (identical(profile, "narrow")) {
            arguments$tssRegion <- c(-10, 10)
        } else {
            stopifnot(identical(profile, "default"))
        }
        saveRDS(list(database=normalizePath(database), profile=profile,
                     passed_tss=arguments$tssRegion,
                     effective_tss=if (profile == "narrow") c(-10, 10) else
                         eval(formals(ChIPseeker::annotatePeak)$tssRegion),
                     downstream_option=getOption("ChIPseeker.downstreamDistance"),
                     level="transcript", annoDb=NULL), file.path(output, "invocation-options.rds"))
        saveRDS(peaks, file.path(output, "input-peaks.rds"))
        annotation <- do.call(ChIPseeker::annotatePeak, arguments)
        saveRDS(annotation, file.path(output, "csAnno.rds"))
        export_annotation(annotation, output)
        snapshot(output)
    }, warning=function(condition) warnings <<- c(warnings, conditionMessage(condition)))
}

reload <- function(calls) {
    output <- file.path(calls, "reload", "objects")
    dir.create(output, recursive=TRUE, showWarnings=FALSE)
    for (name in c("A-gtf-narrow", "A-gff3-narrow", "A-gtf-default")) {
        input <- file.path(calls, name)
        object <- readRDS(file.path(input, "csAnno.rds"))
        expected <- readRDS(file.path(input, "annotation-data.rds"))
        options <- readRDS(file.path(input, "invocation-options.rds"))
        peaks <- readRDS(file.path(input, "input-peaks.rds"))
        stopifnot(identical(as.data.frame(object), expected))
        txdb <- AnnotationDbi::loadDb(options$database)
        base::options(ChIPseeker.downstreamDistance=NULL)
        arguments <- list(peak=peaks, TxDb=txdb, level="transcript", annoDb=NULL)
        if (identical(options$profile, "narrow"))
            arguments$tssRegion <- c(-10, 10)
        fresh <- do.call(ChIPseeker::annotatePeak, arguments)
        stopifnot(identical(as.data.frame(fresh), expected))
        for (slot in slotNames(object)) {
            stopifnot(identical(methods::slot(object, slot), readRDS(file.path(input, paste0("slot-", slot, ".rds")))))
            stopifnot(identical(methods::slot(object, slot), methods::slot(fresh, slot)))
        }
        saveRDS(fresh, file.path(output, paste0(name, ".rds")))
    }
    snapshot(output)
    writeLines("all three csAnno/slot/SQLite fresh annotations identical", file.path(output, "reload-status.txt"))
}

qualify <- function(calls) {
    gtf <- readRDS(file.path(calls, "A-gtf-narrow", "annotation-data.rds"))
    gff <- readRDS(file.path(calls, "A-gff3-narrow", "annotation-data.rds"))
    default <- readRDS(file.path(calls, "A-gtf-default", "annotation-data.rds"))
    stopifnot(identical(gtf, gff), nrow(gtf) == 10L, nrow(default) == 10L,
              identical(as.character(gtf$row_key), paste0("annotation:", 0:9)),
              identical(gtf$row_key, default$row_key),
              identical(as.integer(gtf$row_ordinal), 0:9),
              identical(as.character(gtf$transcriptId), rep(c("g066.t1", "g065.t1"), each=5)),
              identical(as.character(gtf$geneId), rep(c("g066", "g065"), each=5)))
    patterns <- rep(c("^Promoter", "^5' UTR", "^Exon", "^Intron", "^3' UTR"), 2)
    stopifnot(all(vapply(seq_along(patterns), function(i) grepl(patterns[i], gtf$annotation[i]), TRUE)),
              all(grepl("^Promoter", default$annotation)),
              identical(gtf$distanceToTSS, default$distanceToTSS),
              identical(gtf$geneStrand, rep(c(1L, 2L), each=5)))
    export_table(data.frame(row_key=gtf$row_key, strand=gtf$strand,
        narrow=gtf$annotation, default=default$annotation,
        transcript=gtf$transcriptId, distance=gtf$distanceToTSS),
        file.path(calls, "annotation-branch-observations.tsv"))
    writeLines("actual both-strand UTR/exon/intron and default promoter-priority contrast",
               file.path(calls, "annotation-qualification.txt"))
}

args <- commandArgs(TRUE)
stopifnot(length(args) >= 2L)
if (args[[1]] == "annotate") {
    stopifnot(length(args) == 5L)
    annotate(args[[2]], args[[3]], args[[4]], args[[5]])
} else if (args[[1]] == "reload") {
    stopifnot(length(args) == 2L)
    reload(args[[2]])
} else if (args[[1]] == "qualify") {
    stopifnot(length(args) == 2L)
    qualify(args[[2]])
} else if (args[[1]] == "admit") {
    stopifnot(length(args) == 3L)
    loadNamespace("ChIPseeker")
    loadNamespace("txdbmaker")
    AnnotationDbi::loadDb(file.path(args[[2]], "witness/calls/A-gtf-default/TxDb.sqlite"))
    snapshot(args[[3]])
    for (name in c("annotatePeak", "getGenomicAnnotation", "getNearestFeatureIndicesAndDistances"))
        capture.output(get(name, envir=asNamespace("ChIPseeker")),
                       file=file.path(args[[3]], paste0("installed-source-", name, ".R")))
} else {
    stop("unknown source-view action")
}
