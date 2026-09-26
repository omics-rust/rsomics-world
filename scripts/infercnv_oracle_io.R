write_expression <- function(obj, path) {
    matrix <- as.matrix(obj@expr.data)
    if (!is.numeric(matrix) || !all(is.finite(matrix)) ||
        !identical(rownames(matrix), rownames(obj@gene_order)) ||
        anyDuplicated(rownames(matrix)) || anyDuplicated(colnames(matrix))) {
        stop("invalid checkpoint expression identity or finite values")
    }
    con <- file(path, open="wt")
    on.exit(close(con))
    writeLines(paste(c("gene", colnames(matrix)), collapse="\t"), con)
    for (i in seq_len(nrow(matrix))) {
        writeLines(paste(c(rownames(matrix)[i],
                           formatC(matrix[i, ], digits=17, format="g")),
                         collapse="\t"), con)
    }
    invisible(matrix)
}

write_identity <- function(obj, gene_path, cell_path) {
    gene_order <- obj@gene_order
    write.table(data.frame(gene=rownames(gene_order),
                           chr=as.character(gene_order$chr),
                           start=gene_order$start, stop=gene_order$stop),
                gene_path, sep="\t", row.names=FALSE, quote=FALSE)
    cell_names <- colnames(obj@expr.data)
    group <- role <- rep(NA_character_, length(cell_names))
    for (name in names(obj@reference_grouped_cell_indices)) {
        idx <- obj@reference_grouped_cell_indices[[name]]
        if (any(!is.na(group[idx]))) stop("duplicate reference cell index")
        group[idx] <- name
        role[idx] <- "reference"
    }
    for (name in names(obj@observation_grouped_cell_indices)) {
        idx <- obj@observation_grouped_cell_indices[[name]]
        if (any(!is.na(group[idx]))) stop("reference/observation identity overlap")
        group[idx] <- name
        role[idx] <- "observation"
    }
    if (anyNA(group) || anyNA(role)) stop("unassigned checkpoint cell")
    write.table(data.frame(cell=cell_names, group=group, role=role),
                cell_path, sep="\t", row.names=FALSE, quote=FALSE)
}
