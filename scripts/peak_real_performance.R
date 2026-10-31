#!/usr/bin/env Rscript

stopifnot(as.character(getRversion()) == "4.6.1",
          Sys.getenv("BIOCONDUCTOR_VERSION") == "3.23",
          as.character(packageVersion("ChIPseeker")) == "1.48.0",
          as.character(packageVersion("txdbmaker")) == "1.8.0")
args <- commandArgs(TRUE)
stopifnot(length(args) == 4L, !dir.exists(args[[3]]))
dir.create(args[[3]], recursive=TRUE)
timings <- list()
mark <- function(name, started) {
    delta <- proc.time() - started
    timings[[name]] <<- data.frame(stage=name, user=delta[[1]], system=delta[[2]],
                                  elapsed=delta[[3]], child_user=delta[[4]], child_system=delta[[5]])
}

started <- proc.time()
chrominfo <- read.delim(args[[4]], check.names=FALSE)
txdb <- txdbmaker::makeTxDbFromGFF(args[[1]], format="gtf", chrominfo=chrominfo,
                                 dataSource="peak-workflow-input-only")
mark("TxDb-input-parse-and-construction", started)
started <- proc.time()
bed <- read.delim(file.path(args[[2]], "workflow.bed"), header=FALSE, quote="",
                  comment.char="", stringsAsFactors=FALSE)
ledger <- read.delim(file.path(args[[2]], "workflow-rows.tsv"), quote="\"", comment.char="",
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
mark("peak-input-parse", started)
started <- proc.time()
options(ChIPseeker.downstreamDistance=NULL)
annotation <- ChIPseeker::annotatePeak(peaks, TxDb=txdb, level="transcript", annoDb=NULL,
                                      tssRegion=c(-3000, 3000))
mark("annotatePeak-including-first-namespace-load", started)

started <- proc.time()
data <- as.data.frame(annotation)
transcripts <- as.data.frame(GenomicFeatures::transcripts(txdb,
    columns=c("tx_id", "tx_name", "gene_id")))
stopifnot(!anyDuplicated(transcripts$tx_name))
index <- match(as.character(data$transcriptId), transcripts$tx_name)
stopifnot(!anyNA(index), identical(as.integer(data$row_ordinal), seq_len(nrow(data)) - 1L))
gene_chromosome <- as.character(transcripts$seqnames[index])
gene_strand <- as.character(transcripts$strand[index])
stopifnot(all(as.integer(data$geneChr) == match(gene_chromosome, GenomeInfoDb::seqlevels(txdb))),
          all(as.integer(data$geneStrand) == ifelse(gene_strand == "+", 1L, 2L)))
core <- data[, c("row_key", "row_ordinal", "annotation", "geneChr", "geneStart", "geneEnd",
                 "geneLength", "geneStrand", "geneId", "transcriptId", "distanceToTSS"), drop=FALSE]
core$geneChr <- gene_chromosome
core$geneStrand <- gene_strand
source_names <- c("genic", "Intergenic", "Promoter", "fiveUTR", "threeUTR", "Exon", "Intron",
                  "downstream", "distal_intergenic")
native_names <- c("genic", "intergenic", "promoter", "five_utr", "three_utr", "exon", "intron",
                  "downstream", "distal_intergenic")
detail <- methods::slot(annotation, "detailGenomicAnnotation")
stopifnot(identical(names(detail), source_names), nrow(detail) == nrow(data),
          all(vapply(detail, is.logical, TRUE)), !anyNA(detail))
names(detail) <- native_names
detail <- data.frame(row_key=data$row_key, row_ordinal=data$row_ordinal, detail, check.names=FALSE)
for (name in c("core", "detail"))
    write.table(get(name), file.path(args[[3]], paste0(name, ".tsv")), sep="\t", quote=TRUE,
                row.names=FALSE, na="NA")
mark("matched-core-nine-mask-normalization-and-export", started)
write.table(do.call(rbind, timings), file.path(args[[3]], "timings.tsv"), sep="\t", quote=FALSE,
            row.names=FALSE)
