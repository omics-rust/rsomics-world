#!/usr/bin/env Rscript

bayes_reduce_samples <- function(directory) {
    directory <- normalizePath(directory, mustWork=TRUE)
    if (!dir.exists(directory)) stop("sample input is not a directory")
    files <- list.files(directory, pattern="^region-[0-9]{4}\\.rds$", full.names=TRUE)
    if (!length(files)) stop("no region-####.rds sample files")
    indices <- as.integer(sub("^region-([0-9]{4})\\.rds$", "\\1", basename(files)))
    if (any(indices < 1L) || anyDuplicated(indices)) stop("invalid one-based sample region indices")
    order <- order(indices)
    files <- files[order]
    indices <- indices[order]
    rows <- list()
    for (i in seq_along(files)) {
        record <- readRDS(files[[i]])
        chains <- record$samples
        if (!is.list(chains) || length(chains) != 6L) stop("region samples must contain six chains: ", files[[i]])
        variables <- colnames(chains[[1L]])
        if (!length(variables) || anyNA(variables) || anyDuplicated(variables) ||
            any(!grepl("^(theta|epsilon)\\[[1-9][0-9]*\\]$", variables))) {
            stop("invalid posterior variable identities: ", files[[i]])
        }
        for (chain in chains) {
            if (!is.matrix(chain) || !is.numeric(chain) || nrow(chain) != 1000L ||
                any(!is.finite(chain)) || anyDuplicated(colnames(chain)) ||
                !setequal(colnames(chain), variables)) stop("invalid 1000-draw chain: ", files[[i]])
        }
        for (variable in variables) {
            theta <- startsWith(variable, "theta[")
            state <- as.integer(sub("^[^[]+\\[([0-9]+)\\]$", "\\1", variable))
            values <- lapply(chains, function(chain) as.numeric(chain[, variable]))
            if (theta) {
                if (!state %in% seq_len(6L) || any(vapply(values,
                    function(value) any(value < 0 | value > 1), logical(1)))) {
                    stop("invalid theta draws: ", variable)
                }
            } else if (any(vapply(values, function(value) any(!value %in% seq_len(6L)), logical(1)))) {
                stop("invalid epsilon states: ", variable)
            }
            for (target in if (theta) state else seq_len(6L)) {
                draws <- if (theta) values else lapply(values, function(value) as.numeric(value == target))
                means <- vapply(draws, mean, numeric(1))
                batches <- unlist(lapply(draws, function(value) colMeans(matrix(value, 50L, 20L))), use.names=FALSE)
                row <- data.frame(region_index=indices[[i]], variable=variable, state=target,
                    posterior_mean=mean(means), batch_standard_error=stats::sd(batches) / sqrt(120),
                    stringsAsFactors=FALSE)
                for (j in seq_len(6L)) row[[paste0("chain_mean_", j)]] <- means[[j]]
                rows[[length(rows) + 1L]] <- row
            }
        }
    }
    do.call(rbind, rows)
}

bayes_uncertainty_main <- function() {
    args <- commandArgs(trailingOnly=TRUE)
    if (length(args) != 2L) stop("usage: infercnv_bayes_uncertainty.R INPUT_SAMPLES_DIRECTORY OUTPUT_TSV")
    if (file.exists(args[[2L]])) stop("output already exists")
    result <- bayes_reduce_samples(args[[1L]])
    numeric_columns <- c("posterior_mean", "batch_standard_error", paste0("chain_mean_", seq_len(6L)))
    for (name in numeric_columns) result[[name]] <- sprintf("%.17g", result[[name]])
    write.table(result, args[[2L]], sep="\t", quote=FALSE, row.names=FALSE)
    message("Descriptive MC diagnostic: six chains, twenty nonoverlapping fifty-draw batches per chain; ",
            "batch SD/sqrt(120), not proof of convergence or bitwise JAGS compatibility.")
}

if (sys.nframe() == 0L) bayes_uncertainty_main()
