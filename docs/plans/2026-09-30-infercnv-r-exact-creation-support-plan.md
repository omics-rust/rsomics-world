# R exact creation support implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans. Steps use checkbox tracking. Root integrates; bounded read-only agents may propose/review.

**Goal:** Prove the package-free R comparison/export helper against literal count bits, ordered identities/maps and checked counters without claiming an installed factory trial.

**Architecture:** Add one private controller helper and one package-free contract script. Reuse the unchanged R counter and oracle-export modules plus the accepted independent Python checker. The real-factory trial and package contracts remain frozen external proposals until a separate complete installed-package/driver plan closes their runtime and executed-code provenance gates.

**Tech Stack:** R 4.6.1 base/recommended methods/stats; Python >=3.11 standard library; ssh 4090 native Linux. No R installation, package build or dependency download.

**Spec:** [Matched raw-ingestion measurement](2026-09-30-infercnv-raw-ingestion-measurement-design.md), specifically the independently testable R comparison/export component.

## Global constraints

- Boot APFS remains over 80%; no local R/Cargo execution or installs.
- World source lives on Zane HDD, local scratch under /Volumes/KIOXIA/Developments/tmp, remote scratch under /dev/shm.
- All expected/actual count values compare binary64 bytes including signed zero; integer storage may convert exactly to double. No tolerance.
- Preserve literal TSV IDs/coordinates, roles and ordered maps; never substitute membership sets.
- Warm/measured factory lifecycle, actual installed package/source pins and paired performance acceptance are not proved by this package-free script.
- Pure tests may use a test-only S4 class to exercise helper contracts, never a fake infercnv factory.
- Rare comments; all errors propagate. Existing R/Python helpers and old prepared measurement bytes remain unchanged.

## Review focus

- Actual integer/double storage must not turn state comparison into approximate equality: explicit integer/double success and one-ULP failures.
- Metadata must not hold live matrix/S4/function/environment references across a later GC baseline: recursive metadata-state check.
- Literal quoted/trailing-empty fields and signed zero must not collapse through generic CSV formatting: literal TSV and raw-byte regressions.
- Coverage, member order and group order must be tested separately: reordered reference/member and overlap mutants.
- Package-free success must not masquerade as actual factory/source authentication: pinned package tests/trial stay unexecuted proposals and remain separate gates.

---

### Task 1: Genuine missing-helper RED on remote R

**Files:** Create scripts/test_infercnv_ingestion_matched_pure.R.

**Interfaces:** Contract CLI takes one existing scripts directory; consumes helper functions defined below plus unchanged counter/oracle IO modules and independent Python checker.

- [x] **Step 1: Add the complete test while the helper is absent.**

```r
#!/usr/bin/env Rscript

args <- commandArgs(trailingOnly=TRUE)
if (length(args) != 1L) stop("usage: test_infercnv_ingestion_matched_pure.R EXISTING_SCRIPTS_DIR")
flag <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
if (length(flag) != 1L) stop("cannot identify contract script")
proposal_dir <- dirname(normalizePath(sub("^--file=", "", flag)))
source(file.path(proposal_dir, "infercnv_ingestion_matched_support.R"), local=TRUE)
scripts_dir <- absolute_existing(args[[1L]], TRUE)
source(file.path(scripts_dir, "infercnv_matched_support.R"), local=TRUE)
source(file.path(scripts_dir, "infercnv_oracle_io.R"), local=TRUE)
if (!nzchar(Sys.getenv("TMPDIR"))) stop("explicit external TMPDIR required")
root <- tempfile("ingestion-pure-", tmpdir=absolute_existing(Sys.getenv("TMPDIR"), TRUE))
if (!dir.create(root)) stop("cannot create contract scratch")
fails <- function(action, message) {
    error <- tryCatch({force(action); NULL}, error=identity)
    if (!inherits(error, "error") || !grepl(message, conditionMessage(error), fixed=TRUE)) {
        stop("expected bounded rejection absent: ", message)
    }
}
expect <- function(actual, wanted, label) if (!identical(actual, wanted)) stop("contract differs: ", label)
hex_raw <- function(text) {
    starts <- seq.int(1L, nchar(text), 2L)
    as.raw(strtoi(substring(text, starts, starts + 1L), 16L))
}
bits <- hex_raw(paste0("3ff0000000000000", "4010000000000000", "4000000000000000",
                       "4014000000000000", "4008000000000000", "4018000000000000"))
expected <- list(
    genes=matrix(c("g1", "chr1", "1", "2", "g2", "chr2", "3", "4"), ncol=4L, byrow=TRUE),
    cells=matrix(c("c_a", "r_a", "reference", "c_obs", "obs", "observation", "c_b", "r_b", "reference"), ncol=3L, byrow=TRUE),
    maps=matrix(c("reference", "r_b", "3", "c_b", "reference", "r_a", "1", "c_a",
                  "observation", "obs", "2", "c_obs"), ncol=4L, byrow=TRUE),
    expression_bits=bits, count_bits=bits)
methods::setClass("CreationFixture", slots=c(expr.data="matrix", count.data="matrix", gene_order="data.frame",
    reference_grouped_cell_indices="list", observation_grouped_cell_indices="list", options="list"))
values <- matrix(c(1L, 4L, 2L, 5L, 3L, 6L), nrow=2L,
                 dimnames=list(c("g1", "g2"), c("c_a", "c_obs", "c_b")))
obj <- methods::new("CreationFixture", expr.data=values, count.data=values,
    gene_order=data.frame(chr=c("chr1", "chr2"), start=c(1L, 3L), stop=c(2L, 4L), row.names=c("g1", "g2")),
    reference_grouped_cell_indices=list(r_b=3L, r_a=1L), observation_grouped_cell_indices=list(obs=2L),
    options=list(min_max_counts_per_cell=c(1, Inf), counts_md5="fixture-only"))
expect(validate_creation_object(obj, expected), TRUE, "literal creation parts")
expect(count_double_bits(values), bits, "column-major binary64")
double_obj <- obj
storage.mode(double_obj@expr.data) <- "double"
storage.mode(double_obj@count.data) <- "double"
expect(validate_creation_object(double_obj, expected), TRUE, "safe integer-to-double equality")
metadata <- creation_metadata(obj)
has_state <- function(value) is.matrix(value) || isS4(value) || is.environment(value) || is.function(value) ||
    is.list(value) && any(vapply(value, has_state, logical(1)))
expect(has_state(metadata), FALSE, "metadata does not retain state")
expect(metadata$expression$type, "integer", "actual storage type")
if (!any(grepl("Inf", unlist(metadata$options_dput), fixed=TRUE))) stop("Inf options lost")
changed <- double_obj
changed@expr.data[1L, 1L] <- 1 + .Machine$double.eps
fails(validate_creation_object(changed, expected), "creation values, identities or ordered maps differ")
changed <- obj
changed@count.data[1L, 1L] <- 2L
fails(validate_creation_object(changed, expected), "creation values, identities or ordered maps differ")
changed <- obj
changed@reference_grouped_cell_indices <- rev(changed@reference_grouped_cell_indices)
fails(validate_creation_object(changed, expected), "creation values, identities or ordered maps differ")
changed <- obj
changed@reference_grouped_cell_indices$r_a <- c(1L, 2L)
fails(validate_creation_object(changed, expected), "invalid grouped cell indices")
changed <- obj
changed@gene_order$stop[1L] <- 0L
fails(validate_creation_object(changed, expected), "creation genes invalid")
changed <- obj
colnames(changed@count.data)[1L] <- "wrong"
fails(validate_creation_object(changed, expected), "count and expression identities differ")
for (bad in list(matrix(c(0, 0), ncol=1L), matrix(c(1e308, 1e308), ncol=1L),
                 matrix(c(1, Inf), ncol=1L), matrix(c(1, -1), ncol=1L))) {
    fails(count_double_bits(bad), "invalid count")
}
zero <- matrix(c(0, 1), ncol=1L)
negative_zero <- zero
negative_zero[1L] <- -as.double(0)
if (identical(count_double_bits(zero), count_double_bits(negative_zero))) stop("signed zero bits collapsed")
full <- file.path(root, "full")
if (!dir.create(full)) stop("cannot create expected fixture")
writeLines(c("gene\tc_a\tc_obs\tc_b", "g1\t 1 \t2\t3", "g2\t4\t5\t6"), file.path(full, "01.tsv"))
writeLines(c("gene\tchr\tstart\tstop", "g1\tchr1\t1\t2", "g2\tchr2\t3\t4"), file.path(full, "01.genes.tsv"))
writeLines(c("cell\tgroup\trole", "c_a\tr_a\treference", "c_obs\tobs\tobservation", "c_b\tr_b\treference"), file.path(full, "01.cells.tsv"))
maps <- file.path(root, "maps.tsv")
writeLines(c("role\tgroup\tindex\tcell", "reference\tr_b\t3\tc_b", "reference\tr_a\t1\tc_a", "observation\tobs\t2\tc_obs"), maps)
record <- list(expected_dir=full, maps=maps, receipt=list(created_genes=2L, cells=3L))
expect(read_expected_parts(record), expected, "independent literal expected tables")
export <- file.path(root, "export")
export_creation(obj, export)
expect(sort(list.files(export)), sort(creation_outputs), "four exported files")
check_export_python(export, record, scripts_dir, file.path(root, "python-check"))
fails(export_creation(obj, export), "new creation output directory required")
writeLines(c("gene\tc_a\tc_obs\tc_b", "g1\t1_0\t2\t3", "g2\t4\t5\t6"), file.path(full, "01.tsv"))
fails(read_expected_parts(record), "invalid expected numeric spelling")
literal <- file.path(root, "literal.tsv")
writeLines(c("a\tb", '"quoted"\t'), literal)
expect(literal_tsv(literal, c("a", "b")), matrix(c('"quoted"', ""), nrow=1L), "literal quotes and trailing empty field")
source_path <- file.path(root, "source.R")
writeLines("f <- function(a, b=2) { a + b }", source_path)
f <- function(a, b=2) { a + b }
expect(check_function_source(f, source_path, "f"), TRUE, "AST formals and body")
fails(check_function_source(function(a, b=3) { a + b }, source_path, "f"), "installed function differs")
fails(check_function_source(function(a, b=2) { a - b }, source_path, "f"), "installed function differs")
writeLines(c("f <- function(a) a", "f <- function(a) a"), source_path)
fails(function_definition(source_path, "f"), "unique pinned function absent")
expect(owned_input(root, "literal.tsv"), literal, "owned input")
fails(absolute_existing("relative"), "absolute canonical path required")
fails(owned_input(root, "../elsewhere"), "invalid owned input path")
link <- file.path(root, "alias")
if (!file.symlink(full, link)) stop("cannot create symlink contract fixture")
fails(absolute_existing(link, TRUE), "symlink in owned path")
expect(parse_threads("Name:\tR\nThreads:\t1\n"), 1, "observed threads")
for (text in c("", "Threads: 0", "Threads: 1\nThreads: 2", "Threads: 1.5")) fails(parse_threads(text), "invalid process thread count")
before <- c(user.self=0, sys.self=0, elapsed=0, user.child=0, sys.child=0)
after <- c(user.self=0.5, sys.self=0.25, elapsed=1, user.child=0, sys.child=0)
expect(region_counters(before, after), c(region_wall_seconds=1, region_user_seconds=0.5,
    region_system_seconds=0.25, region_child_user_seconds=0, region_child_system_seconds=0), "region CPU units")
actual_shape <- unclass(proc.time())
shape_after <- actual_shape
shape_after[["elapsed"]] <- actual_shape[["elapsed"]] + 1
region_counters(actual_shape, shape_after)
for (key in c("user.child", "sys.child")) {
    bad <- after
    bad[[key]] <- 1
    fails(region_counters(before, bad), "invalid measured factory counters")
}
fails(region_counters(before, before), "invalid measured factory counters")
fails(region_counters(before, after[-1L]), "CPU counter shape differs")
expect(parse_rss_bytes("Rss: 17 kB\n"), 17408, "RSS bytes")
fails(parse_rss_bytes("Rss: 1 MB"), "invalid RSS field")
fails(checked_delta(1, 2), "invalid clock delta")
cat("Package-free creation helper contracts passed; no upstream factory or performance assertion.\n")
```

- [x] **Step 2: Freeze test and unchanged dependencies in the exclusive remote gate directory.**

The already verified runtime is /usr/bin/Rscript 4.6.1 on ssh 4090. The exclusive directory /dev/shm/rsomics-ingestion-r-20260930-gate was created empty with proposal/scripts/tmp children; do not remove or reuse a prior failed gate. Copy only:

```bash
scp scripts/test_infercnv_ingestion_matched_pure.R 4090:/dev/shm/rsomics-ingestion-r-20260930-gate/proposal/
scp scripts/infercnv_matched_support.R scripts/infercnv_oracle_io.R scripts/validate_infercnv_ingestion_measurement.py scripts/validate_infercnv_samples_clustering_witness.py 4090:/dev/shm/rsomics-ingestion-r-20260930-gate/scripts/
```

Record local and remote source hashes before execution; helper must be absent remotely. Retain exit/stdout/stderr externally, including a failed launch.

- [x] **Step 3: Run RED without accepting an arbitrary failure.**

```bash
ssh 4090 'test ! -e /dev/shm/rsomics-ingestion-r-20260930-gate/proposal/infercnv_ingestion_matched_support.R; TMPDIR=/dev/shm/rsomics-ingestion-r-20260930-gate/tmp PYTHONDONTWRITEBYTECODE=1 Rscript --vanilla /dev/shm/rsomics-ingestion-r-20260930-gate/proposal/test_infercnv_ingestion_matched_pure.R /dev/shm/rsomics-ingestion-r-20260930-gate/scripts > /dev/shm/rsomics-ingestion-r-20260930-gate/red.stdout 2> /dev/shm/rsomics-ingestion-r-20260930-gate/red.stderr'
```

Expected nonzero with R source/file error naming the absent infercnv_ingestion_matched_support.R. An SSH/runtime/source-transfer failure is not RED. Keep the complete diagnostic and its original hash.

### Task 2: Minimal helper, package-free GREEN and scope review

**Files:** Create scripts/infercnv_ingestion_matched_support.R.

**Interfaces:** load_creation_record binds one frozen receipt plus paths; check_namespace_source compares three installed function formals/body and constants/digest; read_expected_parts and creation_parts produce exact genes/cells/maps/raw-double parts; validate_creation_object compares the entire result; export_creation emits four files; check_export_python calls the accepted independent checker; creation_metadata returns scalar/list-only storage metadata; region_counters rejects malformed/nonfinite/backward or child-work samples. The namespace/receipt functions are not invoked by this pure gate.

- [x] **Step 1: Add the complete helper only after the observed RED.**

```r
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
```

- [x] **Step 2: Copy the helper, parse both files, then run GREEN.**

```bash
scp scripts/infercnv_ingestion_matched_support.R 4090:/dev/shm/rsomics-ingestion-r-20260930-gate/proposal/
ssh 4090 'TMPDIR=/dev/shm/rsomics-ingestion-r-20260930-gate/tmp PYTHONDONTWRITEBYTECODE=1 Rscript --vanilla -e "parse(file=\"/dev/shm/rsomics-ingestion-r-20260930-gate/proposal/infercnv_ingestion_matched_support.R\"); parse(file=\"/dev/shm/rsomics-ingestion-r-20260930-gate/proposal/test_infercnv_ingestion_matched_pure.R\")" > /dev/shm/rsomics-ingestion-r-20260930-gate/parse.stdout 2> /dev/shm/rsomics-ingestion-r-20260930-gate/parse.stderr'
ssh 4090 'TMPDIR=/dev/shm/rsomics-ingestion-r-20260930-gate/tmp PYTHONDONTWRITEBYTECODE=1 Rscript --vanilla /dev/shm/rsomics-ingestion-r-20260930-gate/proposal/test_infercnv_ingestion_matched_pure.R /dev/shm/rsomics-ingestion-r-20260930-gate/scripts > /dev/shm/rsomics-ingestion-r-20260930-gate/green.stdout 2> /dev/shm/rsomics-ingestion-r-20260930-gate/green.stderr'
```

Expected full package-free success marker, zero exit, preserved Python checker outputs and exact unchanged before/after source hashes. Any real failure gets diagnosed with original evidence and a narrow failing regression; no skipped assertion or package substitution.

Root runtime ruling: the first unchanged-helper run reached the independent
Python checker and failed at hashlib.file_digest because 4090's default
/usr/bin/python3 is 3.10.12. Exact export comparison had completed, but that
failed run is not accepted. Python >=3.11 is already required by the accepted
controller's file_digest/tomllib APIs. An existing Python 3.12 is available at
/home/liangjy/.local/bin/python3.12; select it through the gate-owned
python-bin/python3 alias and PATH for the second run. Preserve the original
green.stdout/stderr and complete failed fixture; use green-2.stdout/stderr
without changing any checker/R bytes. No software installation is needed.

- [ ] **Step 3: Complete scope review and controller checks; commit only the two R files and owned plan/state.**

Independent complete-source review already judged the frozen proposal suitable for integration. It explicitly did not execute R or accept timings. Root rereads actual integrated hashes/results and runs the unchanged 223 Python controller tests, architecture check and whitespace check with external TMPDIR. Commit subject: test(sc): verify exact R creation support. Push main and verify exact-head Control plane CI.

## Explicit subsequent gates, not this helper's acceptance

Do not run/accept the real-factory proposal yet. Its separate complete integration plan must close:

- Freeze ALL executed R/Python scripts and imported checker module identities before launch and after trials. End-of-trial hashes alone are not executed-code authentication.
- Fix/verify actual C locale categories, R 4.6.1/Bioc 3.23/infercnv source, namespace versions/paths and thread environment on every accepted sample.
- Preserve actual proc.time documentation: installed R 4.6.1 says Unix-like values are rounded down to milliseconds; host accuracy is system-specific. No nanosecond claim.
- Original API/digest/CRC/safe artifact authentication and decoded plain/gzip equivalence precede trials.
- Real installed-package source/function and plain/gzip contract tests, complete trial inventory and counters/provenance tests.
- Fresh 32-process schedule/64 independent outputs, external whole-process monitor, strict preselected per-case wall gate and preserved failures. No whole-process peak may be called a factory peak.
