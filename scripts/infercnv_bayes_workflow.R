#!/usr/bin/env Rscript

script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
if (length(script) != 1L) stop("cannot identify script")
script_dir <- dirname(normalizePath(sub("^--file=", "", script)))
source(file.path(script_dir, "infercnv_samples_clustering_witness.R"))
source(file.path(script_dir, "infercnv_oracle_io.R"))

bayes_posterior_record <- function(cell_gene, cnv_probabilities, cell_probabilities, expression) {
    if (length(cell_gene) != length(cnv_probabilities) ||
        length(cell_gene) != length(cell_probabilities)) stop("posterior/rectangle counts differ")
    regions <- probabilities <- cells <- joins <- vector("list", length(cell_gene))
    for (i in seq_along(cell_gene)) {
        join <- cell_gene[[i]]
        id <- as.character(join$cnv_regions)
        theta <- cnv_probabilities[[i]]
        epsilon <- cell_probabilities[[i]]
        epsilon_indices <- suppressWarnings(as.integer(sub("^epsilon\\[([0-9]+)\\]$", "\\1", colnames(epsilon))))
        state_names <- colnames(theta)
        states <- suppressWarnings(as.integer(sub("^theta\\[([0-9]+)\\]$", "\\1", state_names)))
        if (length(id) != 1L || !nzchar(id) || length(join$State) != 1L ||
            !is.matrix(theta) || nrow(theta) < 1L || !identical(states, seq_len(6L)) ||
            !identical(state_names, paste0("theta[", states, "]")) ||
            any(!is.finite(theta)) || any(theta < 0 | theta > 1) ||
            any(abs(rowSums(theta) - 1) > 1e-8) ||
            !is.matrix(epsilon) || !identical(dim(epsilon), c(6L, length(join$Cells))) ||
            !identical(rownames(epsilon), as.character(states)) ||
            !identical(sort(epsilon_indices), seq_along(join$Cells)) ||
            is.null(colnames(epsilon)) || any(!is.finite(epsilon)) ||
            any(epsilon < 0 | epsilon > 1) || any(abs(colSums(epsilon) - 1) > 1e-8) ||
            !length(join$Genes) || !length(join$Cells) ||
            anyNA(match(join$Genes, seq_len(nrow(expression)))) ||
            anyNA(match(join$Cells, seq_len(ncol(expression))))) stop("invalid actual i6 posterior rectangle")
        means <- colMeans(theta)
        maxima <- states[means == max(means)]
        regions[[i]] <- data.frame(region=id, original_state=as.integer(join$State),
            normal_probability=unname(means[[3L]]), posterior_max_states=paste(maxima, collapse=","),
            posterior_draws=nrow(theta), stringsAsFactors=FALSE)
        probabilities[[i]] <- data.frame(region=id, state=states, theta_variable=state_names,
                                         probability=unname(means), stringsAsFactors=FALSE)
        cell_indices <- join$Cells[epsilon_indices]
        cells[[i]] <- data.frame(region=id, cell=rep(colnames(expression)[cell_indices], each=6L),
            cell_index=rep(cell_indices, each=6L), epsilon_variable=rep(colnames(epsilon), each=6L),
            state=rep(states, length(join$Cells)), probability=as.vector(epsilon), stringsAsFactors=FALSE)
        joins[[i]] <- list(region=id, gene_indices=join$Genes, cell_indices=join$Cells,
            genes=rownames(expression)[join$Genes], cells=colnames(expression)[join$Cells],
            original_state=as.integer(join$State), theta_names=state_names,
            epsilon_names=colnames(epsilon), epsilon_state_names=rownames(epsilon))
    }
    result <- list(regions=do.call(rbind, regions), probabilities=do.call(rbind, probabilities),
                   cells=do.call(rbind, cells), joins=joins)
    if (!length(cell_gene) || anyDuplicated(result$regions$region)) stop("missing or duplicated actual CNV regions")
    result
}

bayes_calibration_record <- function(mu, precision) {
    if (length(mu) != 6L || length(precision) != 6L || any(!is.finite(mu)) ||
        any(!is.finite(precision) | precision <= 0)) stop("invalid actual raw-spike calibration")
    data.frame(state=seq_len(6L), mu=unname(mu), raw_spike_precision=unname(precision))
}

bayes_filter_record <- function(posterior, final_cell_gene, original_states, final_states, threshold) {
    before <- posterior$regions
    after_ids <- vapply(final_cell_gene, function(join) as.character(join$cnv_regions), character(1))
    if (anyDuplicated(after_ids) || any(!after_ids %in% before$region) ||
        !identical(after_ids, before$region[before$region %in% after_ids]) ||
        !identical(dimnames(original_states), dimnames(final_states)) ||
        !identical(dim(original_states), dim(final_states))) stop("filtered region/state identity differs")
    removed <- !before$region %in% after_ids
    if (!identical(removed, before$normal_probability > threshold)) {
        stop("actual region removals contradict the strict posterior-normal threshold")
    }
    expected <- original_states
    final_state <- rep(3L, nrow(before))
    for (i in seq_len(nrow(before))) {
        join <- posterior$joins[[i]]
        if (any(original_states[join$gene_indices, join$cell_indices, drop=FALSE] != before$original_state[[i]])) {
            stop("reported original region disagrees with actual HMM states")
        }
        if (!removed[[i]]) {
            actual <- final_cell_gene[[match(before$region[[i]], after_ids)]]
            if (!identical(actual$Genes, join$gene_indices) || !identical(actual$Cells, join$cell_indices) ||
                length(actual$State) != 1L || !actual$State %in% seq_len(6L) ||
                !as.character(actual$State) %in% strsplit(before$posterior_max_states[[i]], ",", fixed=TRUE)[[1L]]) {
                stop("actual reassignment disagrees with retained posterior rectangle")
            }
            final_state[[i]] <- as.integer(actual$State)
        }
        expected[join$gene_indices, join$cell_indices] <- final_state[[i]]
    }
    if (anyNA(final_states) || !all(final_states == expected)) stop("actual filtered state rectangles differ")
    regions <- before
    regions$removed <- removed
    regions$final_state <- final_state
    regions$reassigned <- !removed & final_state != before$original_state
    list(regions=regions, removed_region_ids=before$region[removed], retained_region_ids=after_ids,
         reassigned_region_ids=before$region[regions$reassigned],
         changed_entries=as.integer(sum(original_states != final_states)))
}

.bayes_sampler_exit <- function(frame, value) {
    capture <- get(".bayes_workflow_capture", envir=.GlobalEnv)
    index <- length(capture$samples) + 1L
    if (!inherits(value, "mcmc.list") || length(value) != 6L ||
        any(vapply(value, nrow, integer(1)) != 1000L)) stop("actual JAGS sample protocol differs")
    relative <- file.path("samples", sprintf("region-%04d.rds", index))
    saveRDS(list(samples=value, data=get("data", frame, inherits=FALSE),
                 inits=get("inits", frame, inherits=FALSE),
                 parameters=get("parameters", frame, inherits=FALSE)), file.path(capture$directory, relative))
    capture$samples[[index]] <- list(path=relative, chains=length(value), iterations=1000L,
        mcpar=lapply(value, function(chain) attr(chain, "mcpar")),
        variables=colnames(value[[1L]]))
}

bayes_load_hmm <- function(path, pin) {
    root <- normalizePath(path, mustWork=TRUE)
    witness <- bundle_file(root, "witness.json")
    record <- jsonlite::fromJSON(witness, simplifyVector=FALSE)
    if (!identical(record$schema_version, 1L) || !identical(record$status, "complete") ||
        !identical(record$package, pin) || !identical(record$cases$full$status, "success") ||
        !is.list(record$files) || is.null(names(record$files))) stop("HMM witness identity differs")
    paths <- names(record$files)
    if (anyDuplicated(paths) || !identical(paths, sort(paths, method="radix")) ||
        !identical(sort(list.files(root, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix"),
                   sort(c(paths, "witness.json"), method="radix"))) stop("HMM witness inventory differs")
    for (relative in paths) {
        if (!identical(hash_file(bundle_file(root, relative)), record$files[[relative]])) {
            stop("HMM witness SHA-256 differs: ", relative)
        }
    }
    list(root=root, record=record, witness_sha256=hash_file(witness))
}

bayes_validate_source <- function(ns, archive, pin, metadata) {
    if (!file.exists(archive) || !identical(hash_file(archive), pin$source_archive_sha256)) {
        stop("source archive bytes mismatch")
    }
    source_dir <- file.path(metadata, "source")
    dir.create(source_dir)
    if (!file.copy(archive, file.path(source_dir, "infercnv-source.tar.gz"))) stop("cannot preserve source archive")
    prefix <- paste0("infercnv-", pin$source_commit)
    files <- c("R/inferCNV_BayesNet.R", "R/inferCNV_HMM.R", "R/inferCNV_ops.R", "inst/BUGS_Mixture_Model")
    members <- paste0(prefix, "/", files)
    listing <- utils::untar(archive, list=TRUE)
    if (anyDuplicated(listing) || any(!members %in% listing)) stop("source archive members differ")
    utils::untar(archive, files=members, exdir=source_dir)
    source_root <- file.path(source_dir, prefix)
    installed <- list()
    for (name in c("inferCNVBayesNet", "filterHighPNormals", "run_gibb_sampling", "cnv_prob", "cell_prob",
                   "adjust_genes_regions_report")) {
        path <- if (name == "adjust_genes_regions_report") files[[2L]] else files[[1L]]
        definition <- function_definition(file.path(source_root, path), name)
        installed[[name]] <- list(actual=get(name, ns), formals=definition[[2L]], body=definition[[3L]])
    }
    for (name in names(installed)) {
        value <- installed[[name]]
        if (!identical(paste(deparse(formals(value$actual)), collapse="\n"), paste(deparse(value$formals), collapse="\n")) ||
            !identical(paste(deparse(body(value$actual)), collapse="\n"), paste(deparse(value$body), collapse="\n"))) {
            stop("installed Bayesian function differs from pinned source: ", name)
        }
        writeLines(deparse(value$actual), file.path(metadata, paste0("function-", name, ".txt")))
    }
    method_names <- c("initializeObject", "MeanSD", "getGenesCells", "runMCMC", "nonParallel",
                      "getProbabilities", "removeCNV", "reassignCNV")
    for (name in method_names) {
        writeLines(deparse(methods::getMethod(name, "MCMC_inferCNV", where=ns)),
                   file.path(metadata, paste0("function-", name, ".txt")))
    }
    model <- system.file("BUGS_Mixture_Model", package="infercnv", mustWork=TRUE)
    if (!identical(hash_file(model), hash_file(file.path(source_root, files[[4L]])))) {
        stop("installed BUGS model differs from pinned source")
    }
    if (!file.copy(model, file.path(metadata, "BUGS_Mixture_Model"))) stop("cannot preserve installed BUGS model")
    list(installed_model_sha256=hash_file(model), model="metadata/BUGS_Mixture_Model",
         source_archive="metadata/source/infercnv-source.tar.gz", functions=array(c(names(installed), method_names)))
}

bayes_workflow_main <- function() {
    args <- commandArgs(trailingOnly=TRUE)
    if (length(args) != 2L) stop("usage: infercnv_bayes_workflow.R HMM_WORKFLOWS OUTPUT")
    pin <- list(name="infercnv", version="1.28.0", source_commit="b421d9405c97a309b081ef86d455e976df93eae4",
                source_archive_sha256="b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207")
    if (!identical(as.character(getRversion()), "4.6.1") ||
        !identical(Sys.getenv("INFER_CNV_SOURCE_COMMIT"), pin$source_commit) ||
        !identical(Sys.getenv("INFER_CNV_SOURCE_SHA256"), pin$source_archive_sha256) ||
        !identical(as.character(utils::packageVersion("infercnv")), pin$version)) stop("runtime/source identity differs")
    for (package in c("rjags", "coda", "jsonlite", "digest")) {
        if (!requireNamespace(package, quietly=TRUE)) stop("required installed package absent: ", package)
    }
    source <- bayes_load_hmm(args[[1L]], pin)
    entry <- source$record$cases$full
    root <- normalizePath(args[[2L]], mustWork=FALSE)
    if (file.exists(root)) stop("output directory already exists")
    dir.create(root, recursive=TRUE)
    if (!dir.exists(root)) stop("cannot create output directory")
    metadata <- file.path(root, "metadata")
    dir.create(metadata)
    ns <- asNamespace("infercnv")
    source_identity <- bayes_validate_source(ns, Sys.getenv("INFER_CNV_SOURCE_ARCHIVE"), pin, metadata)
    for (name in c("infercnv_bayes_workflow.R", "test_infercnv_bayes_workflow.R",
                   "infercnv_samples_clustering_witness.R", "infercnv_oracle_io.R")) {
        if (!file.copy(file.path(script_dir, name), file.path(metadata, name))) stop("cannot retain script")
    }
    if (!file.copy(bundle_file(source$root, "witness.json"), file.path(metadata, "hmm-witness.json"))) {
        stop("cannot retain HMM witness")
    }
    writeLines(capture.output(sessionInfo()), file.path(metadata, "session.txt"))
    packages <- utils::installed.packages()[, c("Package", "Version", "LibPath"), drop=FALSE]
    write.table(packages[order(packages[, "Package"], method="radix"), , drop=FALSE],
                file.path(metadata, "packages.tsv"), sep="\t", quote=FALSE, row.names=FALSE)
    enforce_samples_locale()
    Sys.setenv(OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", MKL_NUM_THREADS="1")
    capture <- new.env(parent=emptyenv())
    assign(".bayes_workflow_capture", capture, envir=.GlobalEnv)
    trace("run_gibb_sampling", where=ns, print=FALSE,
          exit=quote(get(".bayes_sampler_exit", .GlobalEnv)(environment(), returnValue())))
    on.exit(untrace("run_gibb_sampling", where=ns))
    reports <- as.character(unlist(entry$reports, use.names=FALSE))
    report_names <- basename(reports)
    suffixes <- c(".cell_groupings", ".pred_cnv_regions.dat", ".pred_cnv_genes.dat", ".genes_used.dat")
    if (length(reports) != 4L || anyDuplicated(report_names)) stop("missing unique source HMM reports")
    prefix <- sub("\\.cell_groupings$", "", report_names[endsWith(report_names, ".cell_groupings")])
    if (length(prefix) != 1L || !startsWith(prefix, "17_HMM_pred") ||
        !setequal(report_names, paste0(prefix, suffixes))) stop("source HMM report prefix differs")
    token <- substring(prefix, nchar("17_HMM_pred") + 1L)
    if (!nzchar(token)) stop("missing source HMM resume token")
    seed <- 261027L
    threshold <- 0.5
    cases <- list()
    for (id in c("full_repeat1", "full_repeat2")) {
        relative <- file.path("cases", id)
        directory <- file.path(root, relative)
        dir.create(directory, recursive=TRUE)
        report_dir <- file.path(directory, "reports")
        dir.create(report_dir)
        dir.create(file.path(directory, "samples"))
        for (i in seq_along(reports)) {
            if (!file.copy(bundle_file(source$root, reports[[i]]), file.path(report_dir, report_names[[i]]))) {
                stop("cannot preserve original stage17 report")
            }
        }
        replay <- readRDS(bundle_file(source$root, entry$replay))
        if (!methods::is(replay$input, "infercnv") || !methods::is(replay$states, "infercnv") ||
            is.null(replay$input@.hspike) || !identical(dimnames(replay$input@expr.data), dimnames(replay$states@expr.data)) ||
            any(!replay$states@expr.data %in% seq_len(6L))) stop("invalid actual HMM replay objects")
        original_expression <- state_identity(replay$input)
        original_states <- replay$states@expr.data
        if (length(unique(apply(original_states, 2L, unique))) == 1L) stop("actual run stage18 condition is false")
        saveRDS(replay, file.path(directory, "original-input-and-hmm.rds"))
        original <- export_state(replay$states, file.path(relative, "original-hmm"), root)
        expression <- export_state(replay$input, file.path(relative, "expression"), root)
        capture$directory <- directory
        capture$samples <- list()
        parameters <- list(file_dir=report_dir, infercnv_obj=replay$input, HMM_states=original_states,
            out_dir=file.path(directory, "upstream"), resume_file_token=token, CORES=1L,
            postMcmcMethod="removeCNV", plotingProbs=FALSE, quietly=TRUE, diagnostics=FALSE,
            HMM_type="i6", k_obs_groups=as.integer(entry$run_parameters$k_obs_groups),
            cluster_by_groups=isTRUE(entry$run_parameters$cluster_by_groups), reassignCNVs=TRUE,
            no_plot=TRUE, useRaster=TRUE)
        set.seed(seed)
        cat("Running actual Bayesian post-HMM workflow: ", id, "\n", sep="")
        started <- proc.time()[["elapsed"]]
        con <- file(file.path(directory, "call.log"), "wt")
        warnings <- list()
        sink(con, type="output")
        sink(con, type="message")
        outcome <- tryCatch(withCallingHandlers({
            mcmc <- do.call(infercnv::inferCNVBayesNet, parameters)
            sampled_wall <- proc.time()[["elapsed"]] - started
            if (!identical(mcmc@expr.data, replay$input@expr.data)) stop("Bayesian sampling changed expression")
            saveRDS(mcmc, file.path(directory, "original-mcmc.rds"))
            posterior <- bayes_posterior_record(mcmc@cell_gene, mcmc@cnv_probabilities,
                                               mcmc@cell_probabilities, mcmc@expr.data)
            if (length(capture$samples) != length(mcmc@cell_gene)) stop("actual sampler trace inventory differs")
            filtered <- infercnv::filterHighPNormals(mcmc, original_states, threshold, useRaster=TRUE)
            filtered_wall <- proc.time()[["elapsed"]] - started - sampled_wall
            saveRDS(filtered, file.path(directory, "filtered-mcmc-and-states.rds"))
            filter <- bayes_filter_record(posterior, filtered[[1L]]@cell_gene, original_states, filtered[[2L]], threshold)
            get("adjust_genes_regions_report", ns)(filtered[[1L]], input_filename_prefix=prefix,
                output_filename_prefix=paste0("HMM_CNV_predictions.", token, ".Pnorm_0.5"), out_dir=report_dir)
            if (!identical(state_identity(replay$input), original_expression) ||
                !identical(replay$states@expr.data, original_states) ||
                !identical(filtered[[1L]]@expr.data, replay$input@expr.data)) stop("Bayesian workflow mutated original expression or states")
            final <- replay$states
            final@expr.data <- filtered[[2L]]
            saveRDS(final, file.path(directory, "final-hmm.rds"))
            for (name in c("regions", "probabilities", "cells")) {
                write.table(posterior[[name]], file.path(directory, paste0("posterior-", name, ".tsv")),
                            sep="\t", quote=FALSE, row.names=FALSE)
            }
            write.table(filter$regions, file.path(directory, "filter-regions.tsv"), sep="\t", quote=FALSE, row.names=FALSE)
            saveRDS(list(posterior=posterior, filter=filter), file.path(directory, "posterior-and-filter-record.rds"))
            calibration <- bayes_calibration_record(mcmc@mu, mcmc@sig)
            write.table(calibration, file.path(directory, "calibration.tsv"), sep="\t", quote=FALSE, row.names=FALSE)
            list(posterior=posterior, filter=filter, final=final, mcmc=mcmc,
                 sampled_wall=sampled_wall, filtered_wall=filtered_wall)
        }, warning=function(condition) {
            warnings[[length(warnings) + 1L]] <<- condition_record(condition)
            cat("OBSERVED WARNING: ", conditionMessage(condition), "\n", sep="")
            invokeRestart("muffleWarning")
        }), error=identity)
        sink(type="message")
        sink(type="output")
        close(con)
        if (inherits(outcome, "error")) {
            jsonlite::write_json(condition_record(outcome), file.path(directory, "failure.json"),
                                 auto_unbox=TRUE, pretty=TRUE, null="null")
            stop(outcome)
        }
        wall <- proc.time()[["elapsed"]] - started
        adjusted <- list.files(report_dir, pattern="^HMM_CNV_predictions.*\\.pred_cnv_(genes|regions)\\.dat$")
        if (length(adjusted) != 2L) stop("actual adjusted reports incomplete")
        final_export <- export_state(outcome$final, file.path(relative, "final-hmm"), root)
        samples <- lapply(seq_along(capture$samples), function(i) {
            value <- capture$samples[[i]]
            value$path <- file.path(relative, value$path)
            value$region <- outcome$posterior$regions$region[[i]]
            value$variables <- array(value$variables)
            value
        })
        recorded_parameters <- parameters[setdiff(names(parameters), c("infercnv_obj", "HMM_states"))]
        recorded_parameters$file_dir <- file.path(relative, "reports")
        recorded_parameters$out_dir <- file.path(relative, "upstream")
        cases[[id]] <- list(status="success", parameters=recorded_parameters,
            BayesMaxPNormal=threshold, original_expression=expression, original_hmm=original, final_hmm=final_export,
            original_input_and_hmm=file.path(relative, "original-input-and-hmm.rds"),
            original_mcmc=file.path(relative, "original-mcmc.rds"),
            filtered_mcmc_and_states=file.path(relative, "filtered-mcmc-and-states.rds"),
            final_hmm_object=file.path(relative, "final-hmm.rds"),
            posterior_regions=file.path(relative, "posterior-regions.tsv"),
            posterior_state_probabilities=file.path(relative, "posterior-probabilities.tsv"),
            posterior_cell_probabilities=file.path(relative, "posterior-cells.tsv"),
            posterior_and_filter_record=file.path(relative, "posterior-and-filter-record.rds"),
            calibration=file.path(relative, "calibration.tsv"),
            removed_region_ids=array(outcome$filter$removed_region_ids),
            reassigned_region_ids=array(outcome$filter$reassigned_region_ids),
            retained_region_ids=array(outcome$filter$retained_region_ids),
            changed_hmm_entries=outcome$filter$changed_entries,
            original_reports=array(file.path(relative, "reports", report_names)),
            adjusted_reports=array(file.path(relative, "reports", adjusted)), samples=samples, warnings=warnings,
            timing=list(sampling_and_public_call_wall_seconds=outcome$sampled_wall,
                        filtering_public_call_wall_seconds=outcome$filtered_wall,
                        total_post_hmm_wall_seconds=wall))
        cat("Completed actual Bayesian workflow: ", id, "; regions=", nrow(outcome$posterior$regions),
            "; removed=", length(outcome$filter$removed_region_ids), "; reassigned=",
            length(outcome$filter$reassigned_region_ids), "\n", sep="")
    }
    paths <- sort(list.files(root, recursive=TRUE, all.files=TRUE, no..=TRUE), method="radix")
    files <- stats::setNames(lapply(paths, function(path) hash_file(bundle_file(root, path))), paths)
    record <- list(schema_version=1L, status="complete", package=pin,
        scope="actual post-HMM Bayesian operation; not raw native acceptance or bitwise JAGS reproducibility",
        source_hmm_witness_sha256=source$witness_sha256, source_hmm_replay=entry$replay,
        source_hmm_replay_sha256=source$record$files[[entry$replay]], source_identity=source_identity,
        runtime=list(R_version=as.character(getRversion()), infercnv_version=pin$version,
            rjags_version=as.character(utils::packageVersion("rjags")), JAGS_version=as.character(rjags::jags.version()),
            coda_version=as.character(utils::packageVersion("coda")), locale=Sys.getlocale(), R_seed=seed,
            R_rng_kind=array(RNGkind()), JAGS_rng="upstream supplies no explicit JAGS RNG seeds; R seed does not control JAGS streams"),
        sampler=list(model="shared Dirichlet theta, categorical epsilon and fixed-precision normal likelihood; unused sigma node",
            chains=6L, adaptation_iterations=500L, update_iterations=200L, sample_iterations=1000L,
            requested_cores=1L, independent_repeats=2L, retained_samples=TRUE),
        timing="in-process actual post-HMM calls with trace and evidence I/O; not paired performance",
        index_base=1L, cases=cases, files=files)
    jsonlite::write_json(record, file.path(root, "witness.json"), auto_unbox=TRUE, pretty=TRUE,
                         null="null", na="null", digits=17)
}

if (sys.nframe() == 0L) bayes_workflow_main()
