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
