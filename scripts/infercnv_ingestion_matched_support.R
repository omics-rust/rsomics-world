input_receipt_sha256 <- "3bee1dc582a321f16cb173c757e955dc620a911a891f4051f8bae68dc856dc0b"
creation_functions <- c("CreateInfercnvObject", ".order_reduce", "validate_infercnv_obj")
creation_fields <- c("expr.data", "count.data", "gene_order",
                     "reference_grouped_cell_indices", "observation_grouped_cell_indices")
creation_outputs <- c("expression.tsv", "genes.tsv", "cells.tsv", "maps.tsv")

absolute_existing <- function(path, directory=FALSE) {
    if (!is.character(path) || length(path) != 1L || is.na(path) ||
        !startsWith(path, "/") || any(strsplit(path, "/", fixed=TRUE)[[1L]][-1L] %in% c(".", ".."))) {
        stop("absolute canonical path required")
    }
    current <- ""
    for (part in strsplit(path, "/", fixed=TRUE)[[1L]][-1L]) {
        current <- paste0(current, "/", part)
        link <- Sys.readlink(current)
        if (!is.na(link) && nzchar(link)) stop("symlink in owned path: ", path)
    }
    resolved <- normalizePath(path, mustWork=TRUE)
    if (!file.exists(resolved) || !identical(dir.exists(resolved), directory)) {
        stop("owned path type differs: ", path)
    }
    resolved
}

owned_input <- function(root, relative) {
    if (!is.character(relative) || length(relative) != 1L || is.na(relative) ||
        !nzchar(relative) || startsWith(relative, "/") || grepl("\\\\", relative) ||
        any(strsplit(relative, "/", fixed=TRUE)[[1L]] %in% c("", ".", ".."))) {
        stop("invalid owned input path")
    }
    root <- absolute_existing(root, TRUE)
    path <- absolute_existing(file.path(root, relative))
    if (!startsWith(path, paste0(root, "/"))) stop("input escapes artifact root")
    path
}

sha256_file <- function(path) digest::digest(path, algo="sha256", file=TRUE)

check_file_hashes <- function(paths, hashes) {
    if (!identical(names(paths), names(hashes)) || !length(paths) ||
        anyDuplicated(names(paths)) || any(!grepl("^[0-9a-f]{64}$", hashes))) {
        stop("input hash inventory differs")
    }
    for (name in names(paths)) {
        if (!identical(sha256_file(paths[[name]]), hashes[[name]])) stop("input bytes changed: ", name)
    }
    invisible(TRUE)
}

check_reused_scripts <- function(scripts_dir) {
    hashes <- c(infercnv_matched_support.R="0c3870354e5ac5bafedbd018148492bb5f339714b5412c409866825b2e783956",
                infercnv_oracle_io.R="5a01aeb620451d0254bbab775c53111a61eeefcaa32ac9b96ab142563ca5839d")
    paths <- stats::setNames(vapply(names(hashes), function(name) owned_input(scripts_dir, name),
                                    character(1)), names(hashes))
    check_file_hashes(paths, hashes)
    invisible(TRUE)
}

load_creation_record <- function(artifact, input_case, receipt_path, source_root) {
    if (!identical(input_case, "plain") && !identical(input_case, "gzip")) stop("unknown raw input case")
    artifact <- absolute_existing(artifact, TRUE)
    source_root <- absolute_existing(source_root, TRUE)
    receipt_path <- absolute_existing(receipt_path)
    if (!identical(sha256_file(receipt_path), input_receipt_sha256)) stop("input receipt bytes differ")
    receipt <- jsonlite::fromJSON(receipt_path, simplifyVector=FALSE)
    paths <- c(plain=owned_input(artifact, receipt$plain$path),
               gzip=owned_input(artifact, receipt$gzip$path),
               stats::setNames(vapply(names(receipt$files), function(rel) owned_input(artifact, rel),
                                      character(1)), names(receipt$files)),
               archive=owned_input(artifact, "infercnv-source.tar.gz"),
               stats::setNames(vapply(names(receipt$factory_source), function(rel) owned_input(source_root, rel),
                                      character(1)), names(receipt$factory_source)))
    hashes <- c(plain=receipt$plain$sha256, gzip=receipt$gzip$sha256,
                unlist(receipt$files, use.names=TRUE), archive=receipt$source_archive_sha256,
                unlist(receipt$factory_source, use.names=TRUE))
    check_file_hashes(paths, hashes)
    for (name in c("plain", "gzip")) {
        if (!identical(as.numeric(file.info(paths[[name]])$size), as.numeric(receipt[[name]]$bytes))) {
            stop("raw input byte count differs: ", name)
        }
    }
    cfg <- receipt$config
    list(receipt=receipt, receipt_path=receipt_path, paths=paths, hashes=hashes,
         artifact=artifact, source_root=source_root, input_case=input_case,
         counts=paths[[input_case]],
         positions=paths[["shipped-bundle/inputs/gencode_downsampled.EXAMPLE_ONLY_DONT_REUSE.txt"]],
         annotations=paths[["shipped-bundle/inputs/oligodendroglioma_annotations_downsampled.txt"]],
         checkpoint=paths[["shipped-bundle/full/upstream/01_incoming_data.infercnv_obj"]],
         expected_dir=absolute_existing(file.path(artifact, "shipped-bundle/full"), TRUE),
         maps=paths[["ingestion-witness/shipped_full/plain/maps.tsv"]],
         references=unname(unlist(cfg$reference_groups)),
         exclusions=unname(unlist(cfg$excluded_chromosomes)),
         depth=c(cfg$min_counts_per_cell, Inf))
}

function_definition <- function(path, name) {
    values <- as.list(parse(path, keep.source=FALSE))
    matches <- Filter(function(value) is.call(value) && identical(value[[1L]], as.name("<-")) &&
        identical(value[[2L]], as.name(name)) && is.call(value[[3L]]) &&
        identical(value[[3L]][[1L]], as.name("function")), values)
    if (length(matches) != 1L) stop("unique pinned function absent: ", name)
    matches[[1L]][[3L]]
}

check_function_source <- function(installed, path, name) {
    definition <- function_definition(path, name)
    text <- function(value) paste(deparse(value), collapse="\n")
    if (!is.function(installed) || !identical(text(formals(installed)), text(definition[[2L]])) ||
        !identical(text(body(installed)), text(definition[[3L]]))) {
        stop("installed function differs from pinned source: ", name)
    }
    invisible(TRUE)
}

check_namespace_source <- function(ns, record, output) {
    if (!identical(record$receipt$required_installed_functions, as.list(creation_functions))) {
        stop("factory function inventory differs")
    }
    source_file <- record$paths[["R/inferCNV.R"]]
    rows <- "function\tsource_path\tsource_sha256\tinstalled_text\tinstalled_text_sha256"
    for (name in creation_functions) {
        installed <- get(name, envir=ns, inherits=FALSE)
        check_function_source(installed, source_file, name)
        path <- file.path(output, paste0("function-", name, ".txt"))
        writeLines(deparse(installed), path)
        rows <- c(rows, paste(name, "R/inferCNV.R", sha256_file(source_file),
                              basename(path), sha256_file(path), sep="\t"))
    }
    for (name in c("C_CHR", "C_START", "C_STOP")) {
        wanted <- c(C_CHR="chr", C_START="start", C_STOP="stop")[[name]]
        if (!identical(get(name, envir=ns, inherits=FALSE), wanted)) stop("namespace constant differs: ", name)
    }
    if (!identical(get("digest", envir=ns), getExportedValue("digest", "digest"))) stop("digest import differs")
    writeLines(rows, file.path(output, "functions.tsv"))
    invisible(TRUE)
}

literal_tsv <- function(path, header) {
    lines <- readLines(path, warn=FALSE)
    fields <- strsplit(paste0(lines, "\t"), "\t", fixed=TRUE)
    if (length(fields) < 2L || !identical(fields[[1L]], header) ||
        any(lengths(fields) != length(header))) stop("invalid literal creation TSV: ", path)
    do.call(rbind, fields[-1L])
}

count_double_bits <- function(value) {
    if (!is.matrix(value) || !is.numeric(value) || anyNA(value) ||
        any(!is.finite(value)) || any(value < 0)) stop("invalid count matrix")
    totals <- colSums(value)
    if (!length(value) || any(!is.finite(totals)) || any(totals <= 0)) stop("invalid count depth")
    writeBin(as.double(value), raw(), size=8L, endian="big")
}

creation_parts <- function(obj) {
    if (!isS4(obj) || !all(creation_fields %in% methods::slotNames(obj))) stop("creation slots absent")
    expr <- methods::slot(obj, "expr.data")
    counts <- methods::slot(obj, "count.data")
    if (!identical(dim(expr), dim(counts)) || !identical(dimnames(expr), dimnames(counts))) {
        stop("count and expression identities differ")
    }
    ids <- rownames(expr)
    cells <- colnames(expr)
    if (is.null(ids) || is.null(cells) || anyNA(c(ids, cells)) || any(!nzchar(c(ids, cells))) ||
        anyDuplicated(ids) || anyDuplicated(cells)) stop("creation names invalid")
    genes <- methods::slot(obj, "gene_order")
    if (!is.data.frame(genes) || !identical(names(genes), c("chr", "start", "stop")) ||
        !identical(rownames(genes), ids) || anyNA(genes) || any(!nzchar(as.character(genes$chr))) ||
        !is.numeric(genes$start) || !is.numeric(genes$stop) ||
        any(!is.finite(genes$start)) || any(!is.finite(genes$stop)) ||
        any(genes$start < 0 | genes$stop < genes$start) ||
        any(genes$start != trunc(genes$start) | genes$stop != trunc(genes$stop)) ||
        any(genes$start == 0 & genes$stop == 0)) stop("creation genes invalid")
    groups <- roles <- rep(NA_character_, length(cells))
    map_rows <- list()
    for (role in c("reference", "observation")) {
        maps <- methods::slot(obj, paste0(role, "_grouped_cell_indices"))
        if (!is.list(maps) || length(maps) && (is.null(names(maps)) || anyNA(names(maps)) ||
            any(!nzchar(names(maps))) || anyDuplicated(names(maps)))) stop("invalid named group list")
        for (name in names(maps)) {
            indices <- maps[[name]]
            if (!is.numeric(indices) || !length(indices) || anyNA(indices) ||
                any(!is.finite(indices)) || any(indices != trunc(indices)) ||
                any(indices < 1 | indices > length(cells)) || anyDuplicated(indices) ||
                any(!is.na(groups[indices]))) stop("invalid grouped cell indices")
            groups[indices] <- name
            roles[indices] <- role
            for (index in indices) {
                map_rows[[length(map_rows) + 1L]] <- c(role, name, sprintf("%.0f", index), cells[[index]])
            }
        }
    }
    if (anyNA(groups) || anyNA(roles)) stop("incomplete group coverage")
    list(genes=cbind(ids, as.character(genes$chr), sprintf("%.0f", genes$start), sprintf("%.0f", genes$stop)),
         cells=cbind(cells, groups, roles), maps=do.call(rbind, map_rows),
         expression_bits=count_double_bits(expr), count_bits=count_double_bits(counts))
}

read_expected_parts <- function(record) {
    cells <- literal_tsv(file.path(record$expected_dir, "01.cells.tsv"), c("cell", "group", "role"))
    genes <- literal_tsv(file.path(record$expected_dir, "01.genes.tsv"), c("gene", "chr", "start", "stop"))
    expression <- literal_tsv(file.path(record$expected_dir, "01.tsv"), c("gene", cells[, 1L]))
    if (!identical(expression[, 1L], genes[, 1L])) stop("expected count genes differ")
    text <- expression[, -1L, drop=FALSE]
    text[] <- trimws(text, whitespace="[ \t\r\n\v\f]")
    number <- "^[+-]?(?:[0-9]+(?:\\.[0-9]*)?|\\.[0-9]+)(?:[eE][+-]?[0-9]+)?$"
    if (any(!grepl(number, text, perl=TRUE))) stop("invalid expected numeric spelling")
    values <- matrix(as.numeric(text), nrow=nrow(text), dimnames=list(genes[, 1L], cells[, 1L]))
    bits <- count_double_bits(values)
    if (!identical(dim(values), c(as.integer(record$receipt$created_genes), as.integer(record$receipt$cells)))) {
        stop("accepted creation dimensions differ")
    }
    list(genes=unname(genes), cells=unname(cells),
         maps=unname(literal_tsv(record$maps, c("role", "group", "index", "cell"))),
         expression_bits=bits, count_bits=bits)
}

validate_creation_object <- function(obj, expected) {
    actual <- creation_parts(obj)
    actual$genes <- unname(actual$genes)
    actual$cells <- unname(actual$cells)
    actual$maps <- unname(actual$maps)
    if (!identical(actual, expected)) stop("creation values, identities or ordered maps differ")
    invisible(TRUE)
}

creation_metadata <- function(obj) {
    list(object_class=unname(as.list(class(obj))),
         expression=list(type=typeof(obj@expr.data), classes=unname(as.list(class(obj@expr.data))),
                         dimensions=unname(as.list(dim(obj@expr.data)))),
         counts=list(type=typeof(obj@count.data), classes=unname(as.list(class(obj@count.data))),
                     dimensions=unname(as.list(dim(obj@count.data)))),
         options_dput=unname(as.list(capture.output(dput(obj@options)))))
}

export_creation <- function(obj, path) {
    if (file.exists(path) || !dir.create(path)) stop("new creation output directory required")
    write_expression(obj, file.path(path, "expression.tsv"))
    write_identity(obj, file.path(path, "genes.tsv"), file.path(path, "cells.tsv"))
    rows <- creation_parts(obj)$maps
    writeLines(c("role\tgroup\tindex\tcell", apply(rows, 1L, paste, collapse="\t")), file.path(path, "maps.tsv"))
    invisible(TRUE)
}

check_export_python <- function(path, record, scripts_dir, prefix) {
    python <- Sys.which("python3")
    if (!nzchar(python)) stop("independent Python checker unavailable")
    code <- paste("import pathlib,sys; sys.path.insert(0,sys.argv[1]);",
                  "from validate_infercnv_ingestion_measurement import validate_creation_export;",
                  "print(validate_creation_export(*map(pathlib.Path,sys.argv[2:5])))")
    status <- system2(python, c("-B", "-c", shQuote(code), shQuote(scripts_dir), shQuote(path),
                              shQuote(record$expected_dir), shQuote(record$maps)),
                      stdout=paste0(prefix, ".stdout"), stderr=paste0(prefix, ".stderr"))
    if (!identical(status, 0L)) stop("independent creation export check failed")
    invisible(TRUE)
}

read_rss <- function() parse_rss_bytes(paste(readLines("/proc/self/smaps_rollup", warn=FALSE), collapse="\n"))

parse_threads <- function(text) {
    lines <- strsplit(text, "\n", fixed=TRUE)[[1L]]
    matches <- grep("^Threads:[[:space:]]+[1-9][0-9]*$", lines, value=TRUE)
    if (length(matches) != 1L || length(grep("^Threads:", lines)) != 1L) stop("invalid process thread count")
    value <- as.numeric(sub("^Threads:[[:space:]]+", "", matches))
    if (!is.finite(value) || value > 2^53 - 1) stop("process thread count overflows")
    value
}

read_threads <- function() parse_threads(paste(readLines("/proc/self/status", warn=FALSE), collapse="\n"))

region_counters <- function(before, after) {
    keys <- c("user.self", "sys.self", "elapsed", "user.child", "sys.child")
    if (!identical(names(before), keys) || !identical(names(after), keys)) stop("CPU counter shape differs")
    values <- mapply(checked_delta, after, before)
    if (values[["elapsed"]] <= 0 || values[["user.child"]] != 0 || values[["sys.child"]] != 0) {
        stop("invalid measured factory counters")
    }
    stats::setNames(as.numeric(values[c("elapsed", "user.self", "sys.self", "user.child", "sys.child")]),
                   c("region_wall_seconds", "region_user_seconds", "region_system_seconds",
                     "region_child_user_seconds", "region_child_system_seconds"))
}
