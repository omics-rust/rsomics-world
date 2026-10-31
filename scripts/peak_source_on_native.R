#!/usr/bin/env Rscript

stopifnot(as.character(getRversion()) == "4.6.1",
          Sys.getenv("BIOCONDUCTOR_VERSION") == "3.23",
          as.character(packageVersion("ChIPseeker")) == "1.48.0",
          as.character(packageVersion("txdbmaker")) == "1.8.0")

args <- commandArgs(TRUE)
stopifnot(length(args) == 2L, !dir.exists(args[[2]]))
loadNamespace("ChIPseeker")
annotation <- readRDS(file.path(args[[1]], "csAnno.rds"))
data <- as.data.frame(annotation)
database <- AnnotationDbi::loadDb(file.path(args[[1]], "TxDb.sqlite"))
transcripts <- as.data.frame(GenomicFeatures::transcripts(database,
    columns=c("tx_id", "tx_name", "gene_id")))
stopifnot(!anyDuplicated(transcripts$tx_name))
index <- match(as.character(data$transcriptId), transcripts$tx_name)
stopifnot(!anyNA(index), identical(as.integer(data$row_ordinal), seq_len(nrow(data)) - 1L))
gene_chromosome <- as.character(transcripts$seqnames[index])
gene_strand <- as.character(transcripts$strand[index])
stopifnot(all(as.integer(data$geneChr) == match(gene_chromosome, GenomeInfoDb::seqlevels(database))),
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
dir.create(args[[2]], recursive=TRUE)
for (name in c("core", "detail"))
    write.table(get(name), file.path(args[[2]], paste0(name, ".tsv")), sep="\t", quote=TRUE,
                row.names=FALSE, na="NA")
saveRDS(list(core=core, detail=detail, normalization=list(
    geneChr="source numeric TxDb sequence identity to exact chromosome name",
    geneStrand="source1/2 to verified transcript +/-")), file.path(args[[2]], "models.rds"))
cat("actual same-native annotation rows:", nrow(data), "detail masks:", ncol(detail) - 2L, "\n")
