# Installed inferCNV raw-creation contracts

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans task-by-task.

**Goal:** Execute the actual pinned installed R factory on the authenticated canonical plain/gzip files and check exact creation state, fresh-process instrumentation and bootstrap failures.

**Architecture:** Extend the existing pinned-oracle workflow with default-false ingestion_contracts, mutually exclusive with prepared matched trials and samples clustering. Reuse the accepted original-artifact verifier, creation/R runtime helpers and independent literal Python validators. No new public foundation, product API, launcher or performance acceptance.

**Tech Stack:** R4.6.1, Bioconductor3.23, infercnv1.28.0 from b421d9405c97a309b081ef86d455e976df93eae4, Python3.12, native Linux x86_64.

**Spec:** `2026-09-30-infercnv-raw-ingestion-measurement-design.md`.

## Global constraints

- Production Rust, accepted helpers/reused scripts and old oracle receipts remain byte-identical.
- Local APFS occupancy93.22% forbids Cargo/R/build/install/dependency downloads; hosted native execution only. All local evidence/scratch stays on external disks.
- Original run36707344893/artifact11092988090 is authenticated by its original API, receipt, size, digest, safe ZIP/CRC and manifest before extraction is consumed.
- The artifact verifier returns shipped-bundle; trial ARTIFACT_ROOT is its parent, including ingestion-witness and the source archive.
- All file arguments are absolute before either R program changes directory.
- LC_ALL=C and seven thread limits1 are set before R starts; record actual namespaces and observed threads, not a claim of continuous single-thread execution.
- Fixed reviewed code pins precede launch and are rechecked afterward. Self-recorded hashes are drift evidence, not authenticity.
- Scrub PYTHONPATH/PYTHONHOME; disable user site, bytecode, unsafe current-directory Python imports; reject existing pyc/pyo files. Insert only authenticated scripts for the two explicit project imports.
- Preserve original failures, package logs/warnings, bootstrap mutation files and every result. No source-substring tests or numerical tolerance relaxation.
- Single-writer unique output roots; the runtime helper is not an atomic-exclusive writer and its RSS argument-shape limitation remains deferred. Real callers use scalar-validated RSS.
- Direct scoped main commits/push with exact-head controller CI, as explicitly authorized. No PR, deletion, publication or automatic reminders.

## Review focus

1. Wrong source/imported checker before bootstrap: five actual mutated-code child runs must reject before creating destination.
2. Wrong values, cells, group/role ordering or coordinates: actual objects and exports are independently compared; mutation tests must reject each.
3. Wrong runtime/receipt/namespace implementation: preflight independently proves installed formals/body/constants/digest and fixed raw/source bytes before test execution.
4. Invalid real counters/metadata: actual fresh plain/gzip children must export all four warm/measured states; the accepted Python literal twelve-field parser supplements the R package parser.
5. Misleading RED or old selector regression: execute all16 selector combinations with actual Bash and preserve old run blocks; only a precisely named missing trial after successful prerequisites counts as intended RED.

## Task 1: Installed-package test and hosted genuine RED

**Files:** Create scripts/test_infercnv_ingestion_matched_package.R; modify .github/workflows/infercnv-oracle.yml; record .autopilot/state/infercnv-r-package-contracts-2026-10-15.md.

**Interfaces:** Consumes the accepted input receipt, original artifact and unchanged helpers. Produces original installed-package evidence and a failing test requiring the missing six-argument trial. Existing scripts are both adjacent and reused; their fixed identities remain unchanged.

- [x] Add the complete package test below with apply_patch; trial stays physically absent.
- [x] Apply the complete workflow below. Validate YAML and every actual Bash block with bash -n; execute selector matrix (2 datasets ×2 matched presence ×2 clustering ×2 ingestion) and compare observable exit status with the approved truth table.
- [x] Parse the actual new R test on4090 with existing R4.6.1, no package install/local R. Run all274 Python tests, architecture and whitespace checks with external TMPDIR/bytecode disabled.
- [x] Commit only package test, workflow and this plan/ledger: test(sc): exercise installed creation contracts. Push main and wait exact-head Control plane CI (f42ec6a8,38689987652 success).
- [x] Dispatch infercnv-oracle.yml dataset=shipped ingestion_contracts=true; other selectors empty/false (38690064479). Original run/jobs/artifact/log acquisition follows completion; no RED acceptance yet.
- [x] Accept RED only after original artifact/input proof, runtime and installed namespace prerequisites pass, every adjacent/reused file except trial matches its fixed hash and is readable, output fixture directories are writable, and stderr explicitly reports actual creation trial absent followed by the exact absolute infercnv_ingestion_matched.R path (38695221700, original API/ZIP/log audit). Original generic-copy run38690064479 is not intended RED; installation/runtime failures are not RED.
- [x] Original run38690064479 emits only generic copy failure. Added explicit missing-component assertion to the package contract, updated fixed hash/complete plan and require a new proper RED; no waiver.

### Complete package test

```r
#!/usr/bin/env Rscript

args <- commandArgs(trailingOnly=TRUE)
if (length(args) != 5L) {
    stop("usage: test_infercnv_ingestion_matched_package.R ARTIFACT_ROOT SOURCE_ROOT INPUT_RECEIPT EXISTING_SCRIPTS_DIR NEW_OUTPUT_DIR")
}
flag <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
if (length(flag) != 1L) stop("cannot identify package contract script")
proposal_dir <- dirname(normalizePath(sub("^--file=", "", flag)))
for (package in c("infercnv", "digest", "jsonlite", "Matrix", "BiocManager", "BiocVersion")) {
    if (!requireNamespace(package, quietly=TRUE)) stop("installed package missing: ", package)
}
accepted_support <- file.path(proposal_dir, "infercnv_ingestion_matched_support.R")
if (!identical(digest::digest(accepted_support, algo="sha256", file=TRUE),
               "d583b156e9a08f3422969697b4524469b8eb05bb92d6c65d41e2aa1df7990faa")) stop("accepted helper bytes differ")
source(accepted_support, local=TRUE)
source(file.path(proposal_dir, "infercnv_ingestion_runtime_support.R"), local=TRUE)
scripts_dir <- absolute_existing(args[[4L]], TRUE)
check_reused_scripts(scripts_dir)
source(file.path(scripts_dir, "infercnv_matched_support.R"), local=TRUE)
source(file.path(scripts_dir, "infercnv_oracle_io.R"), local=TRUE)
actual_runtime <- list(R_version=as.character(getRversion()), Bioconductor_version=as.character(BiocManager::version()),
    infercnv_version=as.character(getNamespaceVersion("infercnv")), system=Sys.info()[["sysname"]],
    architecture=Sys.info()[["machine"]], locale_all=Sys.getlocale(),
    locale=stats::setNames(vapply(locale_categories, Sys.getlocale, character(1)), locale_categories),
    thread_environment=Sys.getenv(thread_keys))
require_runtime_contract(actual_runtime)
output <- args[[5L]]
parent <- absolute_existing(dirname(output), TRUE)
if (!startsWith(output, "/") || file.exists(output) ||
    !identical(file.path(parent, basename(output)), output) || !dir.create(output)) {
    stop("new absolute package-test output directory required")
}
setwd(output)
fails <- function(action, message) {
    error <- tryCatch({force(action); NULL}, error=identity)
    if (!inherits(error, "error") || !grepl(message, conditionMessage(error), fixed=TRUE)) {
        stop("expected package rejection absent: ", message)
    }
}
trial_path <- file.path(proposal_dir, "infercnv_ingestion_matched.R")
if (!file.exists(trial_path)) stop("actual creation trial absent: ", trial_path)
for (target in c("creation_support", "counter_support", "oracle_io", "creation_checker", "imported_validator")) {
    fixture <- file.path(output, paste0("bad-code-", target))
    adjacent <- file.path(fixture, "trial-scripts")
    reused <- file.path(fixture, "reused-scripts")
    if (!dir.create(fixture) || !dir.create(adjacent) || !dir.create(reused)) stop("cannot create bootstrap fixture")
    new_names <- c("infercnv_ingestion_matched.R", "infercnv_ingestion_runtime_support.R", "infercnv_ingestion_matched_support.R")
    old_names <- c("infercnv_matched_support.R", "infercnv_oracle_io.R",
                   "validate_infercnv_ingestion_measurement.py", "validate_infercnv_samples_clustering_witness.py")
    if (!all(file.copy(file.path(proposal_dir, new_names), adjacent)) ||
        !all(file.copy(file.path(scripts_dir, old_names), reused))) stop("cannot copy bootstrap fixture")
    targets <- c(creation_support=file.path(adjacent, new_names[[3L]]),
                 stats::setNames(file.path(reused, old_names), c("counter_support", "oracle_io", "creation_checker", "imported_validator")))
    writeLines("mutated bootstrap bytes", targets[[target]])
    destination <- file.path(fixture, "must-not-exist")
    stdout <- file.path(fixture, "stdout")
    stderr <- file.path(fixture, "stderr")
    status <- system2(file.path(R.home("bin"), "Rscript"), c("--vanilla",
        shQuote(file.path(adjacent, new_names[[1L]])), shQuote(args[[1L]]), "plain", shQuote(args[[2L]]),
        shQuote(args[[3L]]), shQuote(reused), shQuote(destination)), stdout=stdout, stderr=stderr)
    if (status == 0L || file.exists(destination) ||
        !any(grepl("fixed code bytes differ before factory", readLines(stderr, warn=FALSE), fixed=TRUE))) {
        stop("actual bootstrap failed to reject changed code before factory: ", target)
    }
}
record <- load_creation_record(args[[1L]], "plain", args[[3L]], args[[2L]])
check_namespace_source(asNamespace("infercnv"), record, output)
expected <- read_expected_parts(record)
expected_obj <- readRDS(record$checkpoint)
validate_creation_object(expected_obj, expected)
factory <- getExportedValue("infercnv", "CreateInfercnvObject")
plain <- factory(raw_counts_matrix=record$counts, gene_order_file=record$positions,
                 annotations_file=record$annotations, ref_group_names=record$references,
                 delim="\t", max_cells_per_group=NULL,
                 min_max_counts_per_cell=record$depth, chr_exclude=record$exclusions)
gzip_record <- load_creation_record(args[[1L]], "gzip", args[[3L]], args[[2L]])
compressed <- factory(raw_counts_matrix=gzip_record$counts, gene_order_file=gzip_record$positions,
                      annotations_file=gzip_record$annotations, ref_group_names=gzip_record$references,
                      delim="\t", max_cells_per_group=NULL,
                      min_max_counts_per_cell=gzip_record$depth, chr_exclude=gzip_record$exclusions)
for (obj in list(plain, compressed)) {
    if (!methods::is(obj, "infercnv")) stop("factory class differs")
    validate_creation_object(obj, expected)
}
export_creation(plain, file.path(output, "plain"))
export_creation(compressed, file.path(output, "gzip"))
check_export_python(file.path(output, "plain"), record, scripts_dir, file.path(output, "plain-checker"))
check_export_python(file.path(output, "gzip"), record, scripts_dir, file.path(output, "gzip-checker"))
for (slot in c("expr.data", "count.data")) {
    changed <- plain
    values <- methods::slot(changed, slot)
    values[1L, 1L] <- values[1L, 1L] + 1
    methods::slot(changed, slot) <- values
    fails(validate_creation_object(changed, expected), "creation values, identities or ordered maps differ")
}
changed <- plain
changed@reference_grouped_cell_indices <- rev(changed@reference_grouped_cell_indices)
fails(validate_creation_object(changed, expected), "creation values, identities or ordered maps differ")
changed <- plain
changed@reference_grouped_cell_indices[[1L]] <- rev(changed@reference_grouped_cell_indices[[1L]])
fails(validate_creation_object(changed, expected), "creation values, identities or ordered maps differ")
changed <- plain
ri <- changed@reference_grouped_cell_indices[[1L]][1L]
oi <- changed@observation_grouped_cell_indices[[1L]][1L]
changed@reference_grouped_cell_indices[[1L]][1L] <- oi
changed@observation_grouped_cell_indices[[1L]][1L] <- ri
fails(validate_creation_object(changed, expected), "creation values, identities or ordered maps differ")
changed <- plain
colnames(changed@expr.data)[1L] <- "wrong-cell"
fails(validate_creation_object(changed, expected), "count and expression identities differ")
changed <- plain
changed@gene_order$stop[1L] <- changed@gene_order$stop[1L] + 1
fails(validate_creation_object(changed, expected), "creation values, identities or ordered maps differ")
changed <- plain
changed@count.data[1L, 1L] <- Inf
fails(validate_creation_object(changed, expected), "invalid count matrix")
bad_factory <- factory
body(bad_factory) <- quote(stop("different implementation"))
fails(check_function_source(bad_factory, record$paths[["R/inferCNV.R"]], "CreateInfercnvObject"),
      "installed function differs")
bad_hashes <- record$hashes
bad_hashes[["plain"]] <- paste(rep("0", 64L), collapse="")
fails(check_file_hashes(record$paths, bad_hashes), "input bytes changed")
bad_receipt <- file.path(output, "bad-input-receipt.json")
writeLines("{}", bad_receipt)
fails(load_creation_record(args[[1L]], "plain", bad_receipt, args[[2L]]), "input receipt bytes differ")
fails(load_creation_record(args[[1L]], "invalid", args[[3L]], args[[2L]]), "unknown raw input case")
rscript <- file.path(R.home("bin"), "Rscript")
if (any(Sys.getenv(thread_keys) != "1")) stop("set actual thread environment before starting R")
for (input_case in c("plain", "gzip")) {
    trial <- file.path(output, paste0("fresh-", input_case))
    status <- system2(rscript, c("--vanilla", shQuote(file.path(proposal_dir, "infercnv_ingestion_matched.R")),
        shQuote(args[[1L]]), input_case, shQuote(args[[2L]]), shQuote(args[[3L]]), shQuote(scripts_dir), shQuote(trial)),
        stdout=file.path(output, paste0(input_case, "-trial.stdout")),
        stderr=file.path(output, paste0(input_case, "-trial.stderr")))
    if (!identical(status, 0L)) stop("actual fresh factory trial failed: ", input_case)
    for (phase in c("warm", "measured")) {
        check_export_python(file.path(trial, phase), record, scripts_dir,
                            file.path(output, paste0(input_case, "-", phase, "-independent")))
    }
    metrics <- read.table(file.path(trial, "metrics.tsv"), header=TRUE, sep="\t", quote="", comment.char="",
                          colClasses="character", check.names=FALSE)
    if (!identical(names(metrics), c("key", "value")) || anyDuplicated(metrics$key)) stop("trial metrics malformed")
    values <- stats::setNames(metrics$value, metrics$key)
    needed <- c("schema_version", "measurement", "implementation", "input_case", "preparation_wall_seconds", "region_wall_seconds",
                "region_user_seconds", "region_system_seconds", "region_child_user_seconds",
                "region_child_system_seconds", "baseline_rss_bytes", "returned_rss_bytes")
    if (!identical(names(values), needed) || values[["implementation"]] != "infercnv" ||
        values[["schema_version"]] != "1" || values[["measurement"]] != "raw_creation" ||
        values[["input_case"]] != input_case) stop("trial metric inventory differs")
    numbers <- as.numeric(values[-seq_len(4L)])
    if (any(!is.finite(numbers)) || any(numbers < 0) || as.numeric(values[["region_wall_seconds"]]) <= 0 ||
        as.numeric(values[["baseline_rss_bytes"]]) <= 0 || as.numeric(values[["returned_rss_bytes"]]) <= 0 ||
        as.numeric(values[["region_child_user_seconds"]]) != 0 || as.numeric(values[["region_child_system_seconds"]]) != 0) {
        stop("actual trial counter contract differs")
    }
    runtime <- jsonlite::fromJSON(file.path(trial, "runtime.json"), simplifyVector=FALSE)
    packages <- c("infercnv", "digest", "jsonlite", "Matrix", "BiocManager", "BiocVersion")
    if (!identical(names(runtime$namespaces), packages) ||
        !identical(runtime$Bioconductor_version, "3.23") ||
        !identical(runtime$locale_all, "C") ||
        !identical(unlist(runtime$locale, use.names=TRUE), actual_runtime$locale) ||
        !identical(names(runtime$thread_environment), thread_keys) ||
        !identical(runtime$warm$expression$type, typeof(plain@expr.data)) ||
        !identical(runtime$measured$counts$type, typeof(plain@count.data)) ||
        any(unlist(runtime$thread_environment) != "1") || runtime$observed_threads_before < 1 ||
        runtime$observed_threads_after < 1) stop("actual runtime metadata differs")
    for (package in packages) {
        actual <- runtime$namespaces[[package]]
        if (!identical(actual$version, as.character(getNamespaceVersion(package))) ||
            !identical(actual$library_path, dirname(getNamespaceInfo(asNamespace(package), "path")))) {
            stop("actual loaded namespace version or library differs: ", package)
        }
    }
    if (!any(grepl("Inf", unlist(runtime$warm$options_dput), fixed=TRUE)) ||
        !any(grepl("Inf", unlist(runtime$measured$options_dput), fixed=TRUE))) stop("actual Inf options lost")
    if (!file.exists(file.path(trial, "io.lifetime.txt")) ||
        !identical(sha256_file(file.path(trial, "clock-proc-time.txt")), runtime$clock_source$sha256)) {
        stop("actual lifetime I/O or clock provenance absent")
    }
    provenance <- jsonlite::fromJSON(file.path(trial, "provenance.json"), simplifyVector=FALSE)
    if (!identical(provenance$input_case, input_case) ||
        !identical(provenance$receipt_sha256, input_receipt_sha256) ||
        !identical(provenance$creation_arguments$min_max_counts_per_cell, list(1L, "Inf")) ||
        !identical(unlist(provenance$sha256, use.names=TRUE), record$hashes)) stop("actual trial provenance differs")
    code_paths <- unlist(provenance$code_paths, use.names=TRUE)
    before <- unlist(provenance$code_sha256_before, use.names=TRUE)
    after <- unlist(provenance$code_sha256_after, use.names=TRUE)
    check_file_hashes(code_paths, before)
    if (!identical(before, after)) stop("trial code drifted")
    fixed <- unlist(provenance$fixed_code_sha256, use.names=TRUE)
    if (!identical(before[names(fixed)], fixed)) stop("trial fixed code binding differs")
}
check_file_hashes(record$paths, record$hashes)
cat("Actual package creation contracts passed; two fresh trials/four verified results are not a paired performance acceptance.\n")
```

## Task 2: Actual trial GREEN

**Files:** Create scripts/infercnv_ingestion_matched.R. No other implementation file changes.

**Interfaces:** Six positional arguments ARTIFACT_ROOT plain|gzip SOURCE_ROOT INPUT_RECEIPT EXISTING_SCRIPTS_DIR NEW_OUTPUT_DIR. Fresh output contains warm/measured exact four-file exports, twelve metrics, runtime/provenance, actual installed clock documentation, package/library inventory and raw lifetime IO.

- [x] Only after observed intended RED, add the complete trial below. Its SHA256 must be eb09009172c905363407f49c63bfa164da8d32a29ea8ec8a17a8701dc6e231d0; strengthened package test SHA256f5244184ae90276b7d2df27a3e8f9d149449702cffcd64f6036189544da25f83.
- [x] Parse actual R on4090; compare complete proposal/owning bytes. Run full controller (283)/architecture/whitespace checks; all12 actual prelaunch code/receipt pins match.
- [x] Commit trial and owned ledger: test(sc): run actual installed raw creation trials (56e70d79). Exact-head controller38696213008 success; installed-creation run38696325035 dispatched with the same selectors.
- [ ] Require successful installed-package mutants and two actual fresh trial children, actual parent plain/gzip objects, independent warm/measured exact checks, literal counters and all provenance/clock/source/executable checks. No timer, scope or assertion weakening on failure.

### Complete actual trial

```r
#!/usr/bin/env Rscript

args <- commandArgs(trailingOnly=TRUE)
if (length(args) != 6L) {
    stop("usage: infercnv_ingestion_matched.R ARTIFACT_ROOT plain|gzip SOURCE_ROOT INPUT_RECEIPT EXISTING_SCRIPTS_DIR NEW_OUTPUT_DIR")
}
flag <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
if (length(flag) != 1L) stop("cannot identify trial script")
proposal_dir <- dirname(normalizePath(sub("^--file=", "", flag)))
script_path <- normalizePath(sub("^--file=", "", flag), mustWork=TRUE)
scripts_dir <- normalizePath(args[[5L]], mustWork=TRUE)
packages <- c("infercnv", "digest", "jsonlite", "Matrix", "BiocManager", "BiocVersion")
for (package in packages) {
    if (!requireNamespace(package, quietly=TRUE)) stop("required namespace unavailable: ", package)
}
code_paths <- c(trial=script_path, runtime_helper=file.path(proposal_dir, "infercnv_ingestion_runtime_support.R"),
    creation_support=file.path(proposal_dir, "infercnv_ingestion_matched_support.R"),
    counter_support=file.path(scripts_dir, "infercnv_matched_support.R"), oracle_io=file.path(scripts_dir, "infercnv_oracle_io.R"),
    creation_checker=file.path(scripts_dir, "validate_infercnv_ingestion_measurement.py"),
    imported_validator=file.path(scripts_dir, "validate_infercnv_samples_clustering_witness.py"))
code_hashes <- vapply(code_paths, function(path) digest::digest(path, algo="sha256", file=TRUE), character(1))
fixed_code <- c(creation_support="d583b156e9a08f3422969697b4524469b8eb05bb92d6c65d41e2aa1df7990faa",
    counter_support="0c3870354e5ac5bafedbd018148492bb5f339714b5412c409866825b2e783956",
    oracle_io="5a01aeb620451d0254bbab775c53111a61eeefcaa32ac9b96ab142563ca5839d",
    creation_checker="0fb1c968e46ff4dd041a42907ab70efaee9fae8beadf1254c81b1edf3c19cb36",
    imported_validator="682d2a41bbe901a02b6687756e64cc1359fa1e3881d49af40eaa5fc725211222")
if (!identical(code_hashes[names(fixed_code)], fixed_code)) stop("fixed code bytes differ before factory")
source(code_paths[["creation_support"]], local=TRUE)
source(code_paths[["runtime_helper"]], local=TRUE)
proposal_dir <- absolute_existing(proposal_dir, TRUE)
scripts_dir <- absolute_existing(scripts_dir, TRUE)
for (path in code_paths) absolute_existing(path)
check_reused_scripts(scripts_dir)
source(file.path(scripts_dir, "infercnv_matched_support.R"), local=TRUE)
source(file.path(scripts_dir, "infercnv_oracle_io.R"), local=TRUE)
runtime_contract <- list(R_version=as.character(getRversion()), Bioconductor_version=as.character(BiocManager::version()),
    infercnv_version=as.character(getNamespaceVersion("infercnv")), system=Sys.info()[["sysname"]],
    architecture=Sys.info()[["machine"]], locale_all=Sys.getlocale(),
    locale=stats::setNames(vapply(locale_categories, Sys.getlocale, character(1)), locale_categories),
    thread_environment=Sys.getenv(thread_keys))
require_runtime_contract(runtime_contract)
if (!file.exists("/proc/self/smaps_rollup")) stop("required procfs unavailable")
python_command <- Sys.which("python3")
if (!nzchar(python_command)) stop("independent Python runtime absent")
python_path <- normalizePath(python_command, mustWork=TRUE)
python_version <- system2(python_path, c("-I", "-B", "--version"), stdout=TRUE, stderr=TRUE)
if (length(python_version) != 1L || !grepl("^Python 3\\.12\\.[0-9]+$", python_version) ||
    !is.null(attr(python_version, "status"))) stop("independent checker requires Python 3.12")
python_sha256 <- sha256_file(python_path)
clock_documentation <- local({
    document <- tools::Rd_db("base")[["proc.time.Rd"]]
    if (is.null(document)) stop("installed proc.time documentation absent")
    capture.output(tools::Rd2txt(document))
})
output <- args[[6L]]
parent <- absolute_existing(dirname(output), TRUE)
if (!startsWith(output, "/") || file.exists(output) ||
    !identical(file.path(parent, basename(output)), output) || !dir.create(output)) {
    stop("new absolute output directory required")
}
setwd(output)
prepare_start <- as.numeric(proc.time()[["elapsed"]])
record <- load_creation_record(args[[1L]], args[[2L]], args[[4L]], args[[3L]])
ns <- asNamespace("infercnv")
check_namespace_source(ns, record, output)
factory <- getExportedValue("infercnv", "CreateInfercnvObject")
counts <- record$counts
positions <- record$positions
annotations <- record$annotations
references <- record$references
exclusions <- record$exclusions
depth <- record$depth
warm_metadata <- local({
    expected_obj <- readRDS(record$checkpoint)
    if (!methods::is(expected_obj, "infercnv")) stop("accepted checkpoint class differs")
    expected <- read_expected_parts(record)
    validate_creation_object(expected_obj, expected)
    warm <- factory(raw_counts_matrix=counts, gene_order_file=positions,
                    annotations_file=annotations, ref_group_names=references,
                    delim="\t", max_cells_per_group=NULL,
                    min_max_counts_per_cell=depth, chr_exclude=exclusions)
    if (!methods::is(warm, "infercnv")) stop("factory returned unexpected class")
    validate_creation_object(warm, expected)
    export_creation(warm, file.path(output, "warm"))
    check_export_python(file.path(output, "warm"), record, scripts_dir, file.path(output, "warm-checker"))
    creation_metadata(warm)
})
invisible(gc(full=TRUE))
threads_before <- read_threads()
baseline_rss_bytes <- read_rss()
before <- unclass(proc.time())
measured <- factory(raw_counts_matrix=counts, gene_order_file=positions,
                    annotations_file=annotations, ref_group_names=references,
                    delim="\t", max_cells_per_group=NULL,
                    min_max_counts_per_cell=depth, chr_exclude=exclusions)
after <- unclass(proc.time())
retained_result_rss_bytes <- read_rss()
counters <- region_counters(before, after)
threads_after <- read_threads()
preparation_wall_seconds <- checked_delta(before[["elapsed"]], prepare_start)
local({
    expected_obj <- readRDS(record$checkpoint)
    if (!methods::is(expected_obj, "infercnv") || !methods::is(measured, "infercnv")) {
        stop("measured or accepted checkpoint class differs")
    }
    expected <- read_expected_parts(record)
    validate_creation_object(expected_obj, expected)
    validate_creation_object(measured, expected)
})
export_creation(measured, file.path(output, "measured"))
check_export_python(file.path(output, "measured"), record, scripts_dir, file.path(output, "measured-checker"))
check_file_hashes(record$paths, record$hashes)
check_reused_scripts(scripts_dir)
check_file_hashes(code_paths, code_hashes)
if (!identical(sha256_file(record$receipt_path), input_receipt_sha256)) stop("input receipt changed")
capture_lifetime_io("/proc/self/io", file.path(output, "io.lifetime.txt"))
metrics <- creation_metrics(record$input_case, preparation_wall_seconds, counters, baseline_rss_bytes, retained_result_rss_bytes)
writeLines(clock_documentation, file.path(output, "clock-proc-time.txt"))
runtime <- list(R_version=runtime_contract$R_version, Bioconductor_version=runtime_contract$Bioconductor_version,
    system=as.list(Sys.info()), locale_all=runtime_contract$locale_all, locale=as.list(runtime_contract$locale),
    Python=list(executable=python_path, version=python_version, sha256=python_sha256),
    namespaces=stats::setNames(lapply(packages, function(package) {
        list(version=as.character(getNamespaceVersion(package)),
             library_path=dirname(getNamespaceInfo(asNamespace(package), "path")))
    }), packages), thread_environment=as.list(runtime_contract$thread_environment),
    observed_threads_before=threads_before, observed_threads_after=threads_after,
    blas=extSoftVersion()[["BLAS"]], options_dput=as.list(capture.output(dput(options()[c("scipen", "digits", "warn", "OutDec")]))),
    warm=warm_metadata, measured=creation_metadata(measured),
    preparation_policy="verified raw/expected/source bytes; validated/exported warm factory result; released local warm/oracle states; gc(full=TRUE); baseline",
    region_clock="proc.time elapsed/self/child counters around actual CreateInfercnvObject; seconds rounded down to milliseconds per installed R documentation; actual accuracy is system-dependent, not assumed nanoseconds",
    clock_source=list(path="clock-proc-time.txt", sha256=sha256_file(file.path(output, "clock-proc-time.txt")),
                      function_text=as.list(deparse(base::proc.time))),
    rss_scope="baseline then returned object live before delta formatting, validation, export or post-call gc",
    io_scope="unmodified /proc/self/io lifetime snapshot after both verified exports and rechecks, before final metadata writes; includes startup, source-proof writes, warmup, validators, exports and instrumentation; not factory-region deltas",
    lifetime_scope="external monitor includes package startup, warm-up, independent Python validators and exports; not factory peak")
jsonlite::write_json(runtime, file.path(output, "runtime.json"), auto_unbox=TRUE, pretty=TRUE, null="null")
provenance <- list(input_case=record$input_case, receipt_sha256=input_receipt_sha256,
    original_run=record$receipt$accepted_original_run, original_artifact=record$receipt$accepted_original_artifact,
    source_commit=record$receipt$source_commit, paths=as.list(record$paths), sha256=as.list(record$hashes),
    creation_arguments=list(raw_counts_matrix=counts, gene_order_file=positions,
        annotations_file=annotations, ref_group_names=unname(as.list(references)), delim="\t",
        max_cells_per_group=NULL, min_max_counts_per_cell=list(1, "Inf"), chr_exclude=unname(as.list(exclusions))),
    code_paths=as.list(code_paths), code_sha256_before=as.list(code_hashes), code_sha256_after=as.list(vapply(code_paths, sha256_file, character(1))),
    fixed_code_sha256=as.list(fixed_code),
    code_authentication="external controller MUST authenticate exact trial/helper/reused-R/Python bytes against independently frozen source before process launch; self/runtime-helper hashes here are drift baselines, not circular authenticity pins")
jsonlite::write_json(provenance, file.path(output, "provenance.json"), auto_unbox=TRUE, pretty=TRUE, null="null")
writeLines(capture.output(sessionInfo()), file.path(output, "session.txt"))
write.table(utils::installed.packages()[, c("Package", "Version", "LibPath"), drop=FALSE],
            file.path(output, "packages.tsv"), sep="\t", row.names=FALSE, quote=FALSE)
check_file_hashes(code_paths, code_hashes)
if (!identical(sha256_file(python_path), python_sha256)) stop("independent Python executable changed")
write_metrics(file.path(output, "metrics.tsv"), metrics)
```

## Task 3: Original evidence acceptance and fresh review

**Interfaces:** Consumes original API/ZIP/logs and frozen source; produces only bounded actual-R package/trial acceptance, not a performance result.

- [ ] Fetch original run/jobs/artifact API replies, original ZIP and full logs externally; require exact head/attempt/repository/workflow and all intended steps success. Verify size/digest/CRC/safe unique REGTYPE paths before reading exports.
- [ ] Independently authenticate all copied fixed pins against owning source, actual R/Python before/after hashes, raw/source input identities and decoded gzip proof. Review original installed package/version/library/function records and real clock documentation bytes.
- [ ] Re-run the accepted literal metrics/export validators locally with external TMPDIR/bytecode off on the real four outputs; inspect parent plain/gzip and mutation failure records, observed storage types/options/Inf and raw lifetime-I/O labeling.
- [x] Fresh-context whole-change review (most capable available reviewer) at56e70d79: no Critical/Important; existing helper Minor limits safely consumed and ledgered. Root original evidence/execution gates remain separate; review-only assurance is not execution.
- [ ] Record accepted receipt/ledger/source/results with exact controller CI and continue the separately planned authenticated paired32-process/64-state launcher. No whole inferCNV/publication/speed/peak-RSS/cold-cache/general-R-parser claim.

## Complete pinned-oracle workflow

Old prepared and clustering paths retain their code and default selection; only the new route bypasses old fixture regeneration. An absent trial is recorded but never treated as a successful contract. Initial RED deliberately has no trial pin to execute; GREEN requires the actual pinned file via the unchanged package test and mutation fixtures.

```yaml
name: inferCNV pinned oracle

on:
  workflow_dispatch:
    inputs:
      dataset:
        description: Oracle dataset
        type: choice
        required: true
        default: synthetic
        options:
          - synthetic
          - shipped
      matched_snapshot:
        description: Optional verified measurement snapshot (shipped dataset only)
        type: string
        required: false
        default: ''
      samples_clustering:
        description: Execute samples Ward clustering witnesses (shipped dataset only)
        type: boolean
        default: false

      ingestion_contracts:
        description: Validate installed raw-creation contracts, not paired performance
        type: boolean
        default: false

permissions:
  contents: read
  actions: read

jobs:
  oracle:
    runs-on: ubuntu-24.04
    timeout-minutes: 240
    env:
      INFER_CNV_SOURCE_COMMIT: b421d9405c97a309b081ef86d455e976df93eae4
      ORACLE_DATASET: ${{ inputs.dataset || 'synthetic' }}
      MATCHED_SNAPSHOT: ${{ inputs.matched_snapshot || '' }}
      SAMPLES_CLUSTERING: ${{ inputs.samples_clustering || false }}
      INGESTION_CONTRACTS: ${{ inputs.ingestion_contracts || false }}
    steps:
      - uses: actions/checkout@v4
        with:
          persist-credentials: false

      - name: Prepare runner-temp directories
        run: |
          set -euo pipefail
          export R_LIBS_USER="$RUNNER_TEMP/R/library"
          export R_USER_CACHE_DIR="$RUNNER_TEMP/R/cache"
          export XDG_CACHE_HOME="$RUNNER_TEMP/xdg-cache"
          export TMPDIR="$RUNNER_TEMP/tmp"
          export EVIDENCE_DIR="$RUNNER_TEMP/infercnv-oracle"
          export ORACLE_OUTPUT="$EVIDENCE_DIR/bundle"
          export SHIPPED_OUTPUT="$EVIDENCE_DIR/shipped-bundle"
          export RUST_SOURCE="$EVIDENCE_DIR/verified-source"
          export CARGO_HOME="$RUNNER_TEMP/cargo-home"
          export RUSTUP_HOME="$RUNNER_TEMP/rustup-home"
          export CARGO_TARGET_DIR="$RUNNER_TEMP/cargo-target"
          printf 'R_LIBS_USER=%s\nR_USER_CACHE_DIR=%s\nXDG_CACHE_HOME=%s\nTMPDIR=%s\nEVIDENCE_DIR=%s\nORACLE_OUTPUT=%s\nSHIPPED_OUTPUT=%s\nRUST_SOURCE=%s\nCARGO_HOME=%s\nRUSTUP_HOME=%s\nCARGO_TARGET_DIR=%s\n' \
            "$R_LIBS_USER" "$R_USER_CACHE_DIR" "$XDG_CACHE_HOME" "$TMPDIR" "$EVIDENCE_DIR" "$ORACLE_OUTPUT" "$SHIPPED_OUTPUT" "$RUST_SOURCE" "$CARGO_HOME" "$RUSTUP_HOME" "$CARGO_TARGET_DIR" >> "$GITHUB_ENV"
          mkdir -p "$R_LIBS_USER" "$R_USER_CACHE_DIR" "$XDG_CACHE_HOME" "$TMPDIR" "$EVIDENCE_DIR"

      - name: Validate optional matched snapshot selector
        shell: bash
        run: |
          set -euo pipefail
          if [[ "$INGESTION_CONTRACTS" == true ]]; then
            test "$ORACLE_DATASET" = shipped
            test -z "$MATCHED_SNAPSHOT"
            test "$SAMPLES_CLUSTERING" = false
            printf '%s\n' LC_ALL=C OPENBLAS_NUM_THREADS=1 OMP_NUM_THREADS=1 \
              MKL_NUM_THREADS=1 BLIS_NUM_THREADS=1 VECLIB_MAXIMUM_THREADS=1 \
              NUMEXPR_NUM_THREADS=1 RAYON_NUM_THREADS=1 PYTHONNOUSERSITE=1 \
              PYTHONDONTWRITEBYTECODE=1 PYTHONSAFEPATH=1 >> "$GITHUB_ENV"
          fi
          if [[ "$SAMPLES_CLUSTERING" == true ]]; then
            test "$ORACLE_DATASET" = shipped
          fi
          if [[ -n "$MATCHED_SNAPSHOT" ]]; then
            test "$ORACLE_DATASET" = shipped
            python3 -B -c 'import json,os,pathlib; r=json.loads(pathlib.Path(".autopilot/oracles/infercnv-measurement-candidate-2026-09-26.json").read_text()); assert os.environ["MATCHED_SNAPSHOT"] == r["snapshot_name"]'
            printf 'dataset=%s\nmatched_snapshot=%s\n' "$ORACLE_DATASET" "$MATCHED_SNAPSHOT" > "$EVIDENCE_DIR/matched-selection.tsv"
          fi

      - name: Fetch native run and verify frozen measurement source
        if: env.MATCHED_SNAPSHOT != ''
        shell: bash
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          set -euo pipefail
          receipt=.autopilot/oracles/infercnv-measurement-candidate-2026-09-26.json
          read -r run_id run_attempt < <(python3 -B -c 'import json; r=json.load(open(".autopilot/oracles/infercnv-measurement-candidate-2026-09-26.json")); print(r["run_id"],r["run_attempt"])')
          cp "$receipt" "$EVIDENCE_DIR/measurement-candidate-receipt.json"
          gh api "repos/omics-rust/rsomics-world/actions/runs/$run_id" > "$EVIDENCE_DIR/native-run.json"
          gh api "repos/omics-rust/rsomics-world/actions/runs/$run_id/attempts/$run_attempt/jobs?per_page=100" > "$EVIDENCE_DIR/native-jobs.json"
          PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s scripts -p test_verify_infercnv_measurement_snapshot.py \
            2>&1 | tee "$EVIDENCE_DIR/snapshot-verifier-tests.log"
          PYTHONDONTWRITEBYTECODE=1 python3 -B scripts/verify_infercnv_measurement_snapshot.py \
            --receipt "$receipt" --run-json "$EVIDENCE_DIR/native-run.json" \
            --jobs-json "$EVIDENCE_DIR/native-jobs.json" \
            --snapshot-dir ".autopilot/snapshots/sc-cnv-core-2026-09-26/$MATCHED_SNAPSHOT" \
            --output-dir "$RUST_SOURCE" 2>&1 | tee "$EVIDENCE_DIR/snapshot-verification.log"
          PYTHONDONTWRITEBYTECODE=1 python3 -B - "$receipt" ".autopilot/snapshots/sc-cnv-core-2026-09-26/$MATCHED_SNAPSHOT" "$RUST_SOURCE" <<'PY' > "$EVIDENCE_DIR/source-before.json"
          import json, pathlib, sys
          sys.path.insert(0, str(pathlib.Path("scripts").resolve()))
          from verify_infercnv_measurement_snapshot import verify_extracted
          print(json.dumps(verify_extracted(*sys.argv[1:]), sort_keys=True))
          PY

      - name: Authenticate installed-creation contract code
        if: env.INGESTION_CONTRACTS == 'true'
        shell: bash
        run: |
          set -euo pipefail
          unset PYTHONPATH PYTHONHOME
          test -z "$(find scripts -type f \( -name '*.pyc' -o -name '*.pyo' \) -print -quit)"
          cat > "$EVIDENCE_DIR/ingestion-fixed.sha256" <<'PINS'
          f5244184ae90276b7d2df27a3e8f9d149449702cffcd64f6036189544da25f83  scripts/test_infercnv_ingestion_matched_package.R
          bcbbd0e3b6a2d053562345cd9ee59a4ff3e9dc4c8d4e6e2a7db66097d5787418  scripts/infercnv_ingestion_runtime_support.R
          d583b156e9a08f3422969697b4524469b8eb05bb92d6c65d41e2aa1df7990faa  scripts/infercnv_ingestion_matched_support.R
          0c3870354e5ac5bafedbd018148492bb5f339714b5412c409866825b2e783956  scripts/infercnv_matched_support.R
          5a01aeb620451d0254bbab775c53111a61eeefcaa32ac9b96ab142563ca5839d  scripts/infercnv_oracle_io.R
          0fb1c968e46ff4dd041a42907ab70efaee9fae8beadf1254c81b1edf3c19cb36  scripts/validate_infercnv_ingestion_measurement.py
          682d2a41bbe901a02b6687756e64cc1359fa1e3881d49af40eaa5fc725211222  scripts/validate_infercnv_samples_clustering_witness.py
          212bf00ad68c59e652f220eacdf76350ca4e56d48752c3b1592fd46dba5c4dea  scripts/infercnv_ingestion_campaign.py
          651ac3082d92f952bf026f134bfb0482f50b118dcfab367a497bbb0fe863571f  scripts/verify_infercnv_oracle_artifact.py
          3bee1dc582a321f16cb173c757e955dc620a911a891f4051f8bae68dc856dc0b  .autopilot/oracles/infercnv-ingestion-measurement-input-2026-09-30.json
          41284ea567fd2c41a48e22eef3c9063a59944607828ebc9cdea8a3f6da82e526  .autopilot/oracles/infercnv-samples-clustering-witness-2026-09-30.json
          PINS
          if [[ -e scripts/infercnv_ingestion_matched.R ]]; then
            printf '%s\n' 'eb09009172c905363407f49c63bfa164da8d32a29ea8ec8a17a8701dc6e231d0  scripts/infercnv_ingestion_matched.R' >> "$EVIDENCE_DIR/ingestion-fixed.sha256"
            printf 'present\n' > "$EVIDENCE_DIR/ingestion-trial-presence.txt"
          else
            test ! -L scripts/infercnv_ingestion_matched.R
            printf 'absent; package contract must fail\n' > "$EVIDENCE_DIR/ingestion-trial-presence.txt"
          fi
          sha256sum --check "$EVIDENCE_DIR/ingestion-fixed.sha256" | tee "$EVIDENCE_DIR/ingestion-code-before.log"
          git rev-parse HEAD > "$EVIDENCE_DIR/ingestion-controller-head.txt"

      - name: Select Python 3.12 for installed-creation contracts
        if: env.INGESTION_CONTRACTS == 'true'
        shell: bash
        run: |
          set -euo pipefail
          test -x /usr/bin/python3.12
          /usr/bin/python3.12 -I -B -c 'import sys; assert sys.version_info[:2] == (3, 12)'
          mkdir "$RUNNER_TEMP/ingestion-python"
          ln -s /usr/bin/python3.12 "$RUNNER_TEMP/ingestion-python/python3"
          printf '%s\n' "$RUNNER_TEMP/ingestion-python" >> "$GITHUB_PATH"

      - name: Verify original installed-creation artifact and raw inputs
        if: env.INGESTION_CONTRACTS == 'true'
        shell: bash
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          set -euo pipefail
          unset PYTHONPATH PYTHONHOME
          df -h "$RUNNER_TEMP" > "$EVIDENCE_DIR/ingestion-disk-before.txt"
          test "$(df --output=avail -B1 "$RUNNER_TEMP" | tail -1)" -ge 12000000000
          gh api repos/omics-rust/rsomics-world/actions/runs/36707344893 > "$EVIDENCE_DIR/ingestion-original-run.json"
          gh api repos/omics-rust/rsomics-world/actions/artifacts/11092988090 > "$EVIDENCE_DIR/ingestion-original-artifact.json"
          gh api repos/omics-rust/rsomics-world/actions/artifacts/11092988090/zip > "$RUNNER_TEMP/ingestion-original.zip"
          python3 -I -B --version | tee "$EVIDENCE_DIR/ingestion-python-version.txt"
          python3 -B -m unittest discover -s scripts -p test_verify_infercnv_oracle_artifact.py \
            2>&1 | tee "$EVIDENCE_DIR/ingestion-artifact-verifier-tests.log"
          python3 -B scripts/verify_infercnv_oracle_artifact.py \
            --receipt .autopilot/oracles/infercnv-samples-clustering-witness-2026-09-30.json \
            --run-json "$EVIDENCE_DIR/ingestion-original-run.json" \
            --artifact-json "$EVIDENCE_DIR/ingestion-original-artifact.json" \
            --zip "$RUNNER_TEMP/ingestion-original.zip" --output-dir "$RUNNER_TEMP/ingestion-original" \
            2>&1 | tee "$EVIDENCE_DIR/ingestion-artifact-verification.log"
          export INGESTION_ARTIFACT_ROOT="$RUNNER_TEMP/ingestion-original"
          printf 'INGESTION_ARTIFACT_ROOT=%s\n' "$INGESTION_ARTIFACT_ROOT" >> "$GITHUB_ENV"
          python3 -B - <<'PY' > "$EVIDENCE_DIR/ingestion-input-proof.json"
          import gzip, hashlib, json, os, pathlib
          root = pathlib.Path(os.environ["INGESTION_ARTIFACT_ROOT"])
          receipt = json.loads(pathlib.Path(".autopilot/oracles/infercnv-ingestion-measurement-input-2026-09-30.json").read_text())
          oracle = json.loads(pathlib.Path(".autopilot/oracles/infercnv-samples-clustering-witness-2026-09-30.json").read_text())
          assert (receipt["accepted_original_run"], receipt["accepted_original_artifact"], receipt["accepted_artifact_sha256"]) == (oracle["run_id"], oracle["artifact_id"], oracle["artifact_sha256"])
          def digest(path):
              with path.open("rb") as stream:
                  return hashlib.file_digest(stream, "sha256").hexdigest()
          files = dict(receipt["files"])
          files["infercnv-source.tar.gz"] = receipt["source_archive_sha256"]
          for case in ("plain", "gzip"):
              item = receipt[case]
              path = root / item["path"]
              assert path.stat().st_size == item["bytes"]
              files[item["path"]] = item["sha256"]
          for name, wanted in files.items():
              path = root / name
              assert path.is_file() and not path.is_symlink() and digest(path) == wanted, name
          decoded = hashlib.sha256()
          count = 0
          with gzip.open(root / receipt["gzip"]["path"], "rb") as stream:
              for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                  decoded.update(chunk)
                  count += len(chunk)
          assert count == receipt["gzip"]["decoded_bytes"] == receipt["plain"]["bytes"]
          assert decoded.hexdigest() == receipt["gzip"]["decoded_sha256"] == receipt["plain"]["sha256"]
          print(json.dumps({"verified_files": files, "decoded_bytes": count, "decoded_sha256": decoded.hexdigest()}, sort_keys=True))
          PY

      - name: Test bundle validator
        shell: bash
        run: |
          set -euo pipefail
          PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s scripts -p test_infercnv_oracle.py \
            2>&1 | tee "$EVIDENCE_DIR/checker-tests.log"
          PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s scripts -p test_infercnv_shipped_oracle.py \
            2>&1 | tee "$EVIDENCE_DIR/shipped-checker-tests.log"

      - uses: r-lib/actions/setup-r@v2
        with:
          r-version: '4.6.1'
          use-public-rspm: true

      - name: Verify Python runtime identity across R launch
        if: env.INGESTION_CONTRACTS == 'true'
        shell: bash
        run: |
          set -euo pipefail
          unset PYTHONPATH PYTHONHOME
          python3 -I -B -c 'import pathlib,sys; assert sys.version_info[:2] == (3,12); print(pathlib.Path(sys.executable).resolve())' > "$EVIDENCE_DIR/ingestion-python-executable.txt"
          realpath "$(command -v Rscript)" > "$EVIDENCE_DIR/ingestion-rscript-executable.txt"
          Rscript --vanilla -e 'cat(normalizePath(file.path(R.home("bin"), "exec", "R")), "\n", sep="")' > "$EVIDENCE_DIR/ingestion-r-executable.txt"
          sha256sum "$(cat "$EVIDENCE_DIR/ingestion-python-executable.txt")" \
            "$(cat "$EVIDENCE_DIR/ingestion-rscript-executable.txt")" \
            "$(cat "$EVIDENCE_DIR/ingestion-r-executable.txt")" > "$EVIDENCE_DIR/ingestion-executables.sha256"
          sha256sum --check "$EVIDENCE_DIR/ingestion-executables.sha256" > "$EVIDENCE_DIR/ingestion-executables-preflight.log"
          probe_code="import hashlib,json,pathlib,sys; libraries=sorted({line.split()[-1] for line in pathlib.Path('/proc/self/maps').read_text().splitlines() if 'libpython' in line}); hashes={name:hashlib.sha256(pathlib.Path(name).read_bytes()).hexdigest() for name in libraries}; executable=str(pathlib.Path(sys.executable).resolve()); print(json.dumps({'executable':executable,'executable_sha256':hashlib.sha256(pathlib.Path(executable).read_bytes()).hexdigest(),'version':sys.version,'prefix':sys.prefix,'base_prefix':sys.base_prefix,'stdlib_json':json.__file__,'libpython_sha256':hashes},sort_keys=True))"
          python3 -I -B -c "$probe_code" > "$EVIDENCE_DIR/ingestion-python-shell-runtime.json"
          Rscript --vanilla - "$probe_code" <<'RS' 2>&1 | tee "$EVIDENCE_DIR/ingestion-python-r-boundary.log"
          python <- normalizePath(Sys.which("python3"), mustWork=TRUE)
          expected <- readLines(file.path(Sys.getenv("EVIDENCE_DIR"), "ingestion-python-shell-runtime.json"), warn=FALSE)
          cat("R LD_LIBRARY_PATH: ", Sys.getenv("LD_LIBRARY_PATH"), "\n", sep="")
          cat("Actual R ldpaths configuration:\n")
          cat(readLines(file.path(R.home("etc"), "ldpaths"), warn=FALSE), sep="\n")
          observed <- system2(python, c("-I", "-B", "-c", shQuote(commandArgs(trailingOnly=TRUE)[[1L]])),
                              stdout=TRUE, stderr=TRUE)
          cat("\nShell Python identity:\n", expected, "\nR-child Python identity:\n", observed, "\n", sep="")
          if (length(expected) != 1L || length(observed) != 1L || !is.null(attr(observed, "status")) ||
              !identical(expected, observed)) {
              stop("Python runtime identity differs across R launch")
          }
          cat("Python runtime identity agrees across shell and R launch.\n")
          RS

      - name: Install JAGS and system headers
        run: |
          sudo apt-get update
          sudo apt-get install -y --no-install-recommends \
            jags libcurl4-openssl-dev libssl-dev libxml2-dev \
            libfontconfig1-dev libfreetype6-dev libpng-dev libtiff-dev \
            libjpeg-dev libharfbuzz-dev libfribidi-dev libgit2-dev \
            libudunits2-dev libhdf5-dev libgsl-dev libglpk-dev

      - name: Download pinned source and record byte identity
        run: |
          set -euo pipefail
          archive="$EVIDENCE_DIR/infercnv-source.tar.gz"
          curl --fail --location --retry 3 \
            "https://github.com/bioconductor-source/infercnv/archive/$INFER_CNV_SOURCE_COMMIT.tar.gz" \
            --output "$archive"
          printf '%s  %s\n' b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207 "$archive" | sha256sum --check -
          export INFER_CNV_SOURCE_SHA256="$(sha256sum "$archive" | cut -d' ' -f1)"
          printf 'commit\t%s\nsha256\t%s\nurl\t%s\n' \
            "$INFER_CNV_SOURCE_COMMIT" "$INFER_CNV_SOURCE_SHA256" \
            "https://github.com/bioconductor-source/infercnv/archive/$INFER_CNV_SOURCE_COMMIT.tar.gz" \
            > "$EVIDENCE_DIR/source-identity.tsv"
          printf 'INFER_CNV_SOURCE_SHA256=%s\n' "$INFER_CNV_SOURCE_SHA256" >> "$GITHUB_ENV"
          printf 'INFER_CNV_SOURCE_ARCHIVE=%s\n' "$archive" >> "$GITHUB_ENV"
          tar -xzf "$archive" -C "$EVIDENCE_DIR"
          test "$(sed -n 's/^Version: //p' "$EVIDENCE_DIR/infercnv-$INFER_CNV_SOURCE_COMMIT/DESCRIPTION")" = '1.28.0'

      - name: Install R dependencies from Bioconductor 3.23
        shell: bash
        run: |
          set -euo pipefail
          Rscript --vanilla -e '
            cran <- Sys.getenv("RSPM")
            if (!nzchar(cran)) stop("setup-r did not provide RSPM")
            options(repos=c(CRAN=cran))
            print(getOption("repos"))
            install.packages("BiocManager", lib=Sys.getenv("R_LIBS_USER"))
            BiocManager::install(version="3.23", ask=FALSE, update=FALSE)
            desc <- read.dcf(file.path(Sys.getenv("EVIDENCE_DIR"),
                                       paste0("infercnv-", Sys.getenv("INFER_CNV_SOURCE_COMMIT")),
                                       "DESCRIPTION"))
            imports <- trimws(strsplit(desc[1, "Imports"], ",", fixed=TRUE)[[1]])
            imports <- sub("[[:space:]]*\\(.*", "", imports)
            BiocManager::install(c(imports, "jsonlite"), ask=FALSE, update=FALSE,
                                 dependencies=c("Depends", "Imports", "LinkingTo"))
            missing <- setdiff(imports, rownames(installed.packages()))
            if (length(missing)) stop("missing infercnv imports: ", paste(missing, collapse=", "))
            unloadable <- vapply(c(imports, "jsonlite"), function(pkg) {
              tryCatch({loadNamespace(pkg); ""}, error=function(e) conditionMessage(e))
            }, character(1))
            if (any(nzchar(unloadable))) {
              print(unloadable[nzchar(unloadable)])
              stop("installed dependencies failed namespace loading")
            }
          ' 2>&1 | tee "$EVIDENCE_DIR/install-dependencies.log"

      - name: Install exact source and verify package
        shell: bash
        run: |
          set -euo pipefail
          R CMD INSTALL --library="$R_LIBS_USER" \
            "$EVIDENCE_DIR/infercnv-$INFER_CNV_SOURCE_COMMIT" \
            2>&1 | tee "$EVIDENCE_DIR/install-infercnv.log"
          Rscript --vanilla -e '
            stopifnot(as.character(packageVersion("infercnv")) == "1.28.0")
            cat("infercnv_version\t", as.character(packageVersion("infercnv")), "\n", sep="")
            cat("infercnv_path\t", system.file(package="infercnv"), "\n", sep="")
            cat("Bioconductor_version\t", as.character(BiocManager::version()), "\n", sep="")
            stopifnot(as.character(BiocManager::version()) == "3.23")
          ' 2>&1 | tee "$EVIDENCE_DIR/installed-identity.log"

      - name: Verify installed creation source and runtime prerequisites
        if: env.INGESTION_CONTRACTS == 'true'
        shell: bash
        run: |
          set -euo pipefail
          unset PYTHONPATH PYTHONHOME
          scripts_dir="$(realpath scripts)"
          receipt="$(realpath .autopilot/oracles/infercnv-ingestion-measurement-input-2026-09-30.json)"
          source_root="$EVIDENCE_DIR/infercnv-$INFER_CNV_SOURCE_COMMIT"
          mkdir "$EVIDENCE_DIR/ingestion-prerequisites"
          sha256sum --check "$EVIDENCE_DIR/ingestion-fixed.sha256" > "$EVIDENCE_DIR/ingestion-code-prelaunch.log"
          Rscript --vanilla - "$scripts_dir" "$INGESTION_ARTIFACT_ROOT" "$receipt" "$source_root" \
            "$EVIDENCE_DIR/ingestion-prerequisites" <<'RS' 2>&1 | tee "$EVIDENCE_DIR/ingestion-prerequisites.log"
          args <- commandArgs(trailingOnly=TRUE)
          source(file.path(args[[1L]], "infercnv_ingestion_matched_support.R"))
          source(file.path(args[[1L]], "infercnv_ingestion_runtime_support.R"))
          for (package in c("infercnv", "digest", "jsonlite", "Matrix", "BiocManager", "BiocVersion")) {
              stopifnot(requireNamespace(package, quietly=TRUE))
          }
          require_runtime_contract(list(R_version=as.character(getRversion()),
              Bioconductor_version=as.character(BiocManager::version()),
              infercnv_version=as.character(getNamespaceVersion("infercnv")),
              system=Sys.info()[["sysname"]], architecture=Sys.info()[["machine"]],
              locale_all=Sys.getlocale(),
              locale=stats::setNames(vapply(locale_categories, Sys.getlocale, character(1)), locale_categories),
              thread_environment=Sys.getenv(thread_keys)))
          record <- load_creation_record(args[[2L]], "plain", args[[3L]], args[[4L]])
          check_namespace_source(asNamespace("infercnv"), record, args[[5L]])
          cat(capture.output(sessionInfo()), sep="\n")
          cat("\nInstalled source/runtime/input prerequisites passed.\n")
          RS
          printf 'artifact_root=%s\nsource_root=%s\nreceipt=%s\nscripts_dir=%s\noutput=%s\n' \
            "$INGESTION_ARTIFACT_ROOT" "$source_root" "$receipt" "$scripts_dir" "$EVIDENCE_DIR/ingestion-package" \
            > "$EVIDENCE_DIR/ingestion-invocation.tsv"
          sha256sum --check "$EVIDENCE_DIR/ingestion-executables.sha256" > "$EVIDENCE_DIR/ingestion-executables-before.log"

      - name: Test installed plain and gzip creation factories
        if: env.INGESTION_CONTRACTS == 'true'
        shell: bash
        run: |
          set -euo pipefail
          unset PYTHONPATH PYTHONHOME
          Rscript --vanilla scripts/test_infercnv_ingestion_matched_package.R \
            "$INGESTION_ARTIFACT_ROOT" "$EVIDENCE_DIR/infercnv-$INFER_CNV_SOURCE_COMMIT" \
            "$(realpath .autopilot/oracles/infercnv-ingestion-measurement-input-2026-09-30.json)" \
            "$(realpath scripts)" "$EVIDENCE_DIR/ingestion-package" \
            2>&1 | tee "$EVIDENCE_DIR/ingestion-package.log"

      - name: Independently parse real creation counters and exports
        if: env.INGESTION_CONTRACTS == 'true'
        shell: bash
        run: |
          set -euo pipefail
          unset PYTHONPATH PYTHONHOME
          python3 -B - <<'PY' > "$EVIDENCE_DIR/ingestion-real-results.json"
          import dataclasses, json, os, pathlib, sys
          sys.path.insert(0, str(pathlib.Path("scripts").resolve()))
          from infercnv_ingestion_campaign import parse_region_metrics
          from validate_infercnv_ingestion_measurement import validate_factory_results
          root = pathlib.Path(os.environ["INGESTION_ARTIFACT_ROOT"])
          output = pathlib.Path(os.environ["EVIDENCE_DIR"]) / "ingestion-package"
          expected = root / "shipped-bundle/full"
          maps = root / "ingestion-witness/shipped_full/plain/maps.tsv"
          result = {}
          for case in ("plain", "gzip"):
              trial = output / ("fresh-" + case)
              metrics = parse_region_metrics((trial / "metrics.tsv").read_bytes().decode("utf-8"))
              assert metrics.input_case == case and metrics.implementation == "infercnv"
              checks = validate_factory_results(trial, expected, maps)
              result[case] = {"metrics": dataclasses.asdict(metrics), "checks": checks}
          print(json.dumps(result, sort_keys=True))
          PY

      - name: Verify installed-creation code and executables after contracts
        if: ${{ always() && inputs.ingestion_contracts }}
        shell: bash
        run: |
          set -euo pipefail
          sha256sum --check "$EVIDENCE_DIR/ingestion-fixed.sha256" | tee "$EVIDENCE_DIR/ingestion-code-after.log"
          sha256sum --check "$EVIDENCE_DIR/ingestion-executables.sha256" | tee "$EVIDENCE_DIR/ingestion-executables-after.log"
          test -z "$(find scripts -type f \( -name '*.pyc' -o -name '*.pyo' \) -print -quit)"
          df -h "$RUNNER_TEMP" > "$EVIDENCE_DIR/ingestion-disk-after.txt"

      - name: Execute synthetic infercnv through step 14
        if: env.INGESTION_CONTRACTS != 'true'
        shell: bash
        run: |
          set -euo pipefail
          Rscript --vanilla scripts/infercnv_oracle.R "$ORACLE_OUTPUT" \
            2>&1 | tee "$EVIDENCE_DIR/oracle-run.log"

      - name: Validate synthetic exports and create sorted manifest
        if: env.INGESTION_CONTRACTS != 'true'
        shell: bash
        run: |
          set -euo pipefail
          PYTHONDONTWRITEBYTECODE=1 python3 -B scripts/validate_infercnv_oracle.py \
            "$ORACLE_OUTPUT" --manifest 2>&1 | tee "$EVIDENCE_DIR/validation.log"

      - name: Execute shipped infercnv through step 14
        if: env.ORACLE_DATASET == 'shipped' && env.INGESTION_CONTRACTS != 'true'
        shell: bash
        run: |
          set -euo pipefail
          Rscript --vanilla scripts/infercnv_shipped_oracle.R "$SHIPPED_OUTPUT" \
            2>&1 | tee "$EVIDENCE_DIR/shipped-oracle-run.log"

      - name: Validate shipped exports and create sorted manifest
        if: env.ORACLE_DATASET == 'shipped' && env.INGESTION_CONTRACTS != 'true'
        shell: bash
        run: |
          set -euo pipefail
          PYTHONDONTWRITEBYTECODE=1 python3 -B scripts/validate_infercnv_shipped_oracle.py \
            "$SHIPPED_OUTPUT" --manifest 2>&1 | tee "$EVIDENCE_DIR/shipped-validation.log"

      - name: Test ingestion witness checker
        shell: bash
        run: |
          set -euo pipefail
          PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s scripts \
            -p test_infercnv_ingestion_witness.py \
            2>&1 | tee "$EVIDENCE_DIR/ingestion-checker-tests.log"

      - name: Capture creation-stage raw-input witnesses
        if: env.INGESTION_CONTRACTS != 'true'
        shell: bash
        run: |
          set -euo pipefail
          args=("$ORACLE_OUTPUT" "$EVIDENCE_DIR/ingestion-witness")
          if [[ "$ORACLE_DATASET" == shipped ]]; then args+=("$SHIPPED_OUTPUT"); fi
          sha256sum scripts/infercnv_ingestion_witness.R \
            scripts/validate_infercnv_ingestion_witness.py \
            scripts/test_infercnv_ingestion_witness.py \
            > "$EVIDENCE_DIR/ingestion-code-sha256.txt"
          Rscript --vanilla scripts/infercnv_ingestion_witness.R "${args[@]}" \
            2>&1 | tee "$EVIDENCE_DIR/ingestion-witness-run.log"
          checker=("$ORACLE_OUTPUT" "$EVIDENCE_DIR/ingestion-witness")
          if [[ "$ORACLE_DATASET" == shipped ]]; then checker+=(--shipped "$SHIPPED_OUTPUT"); fi
          PYTHONDONTWRITEBYTECODE=1 python3 -B scripts/validate_infercnv_ingestion_witness.py \
            "${checker[@]}" 2>&1 | tee "$EVIDENCE_DIR/ingestion-witness-validation.log"

      - name: Capture creation format and filter edge probes
        if: env.ORACLE_DATASET == 'shipped' && env.INGESTION_CONTRACTS != 'true'
        shell: bash
        run: |
          set -euo pipefail
          PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s scripts \
            -p test_infercnv_creation_edges.py \
            2>&1 | tee "$EVIDENCE_DIR/creation-edge-checker-tests.log"
          sha256sum scripts/infercnv_creation_edges.R \
            scripts/validate_infercnv_creation_edges.py scripts/test_infercnv_creation_edges.py \
            > "$EVIDENCE_DIR/creation-edge-code-sha256.txt"
          Rscript --vanilla scripts/infercnv_creation_edges.R \
            "$SHIPPED_OUTPUT" "$EVIDENCE_DIR/creation-edges" \
            2>&1 | tee "$EVIDENCE_DIR/creation-edges-run.log"
          PYTHONDONTWRITEBYTECODE=1 python3 -B scripts/validate_infercnv_creation_edges.py \
            "$SHIPPED_OUTPUT" "$EVIDENCE_DIR/creation-edges" \
            2>&1 | tee "$EVIDENCE_DIR/creation-edges-validation.log"

      - name: Capture samples-clustering witnesses
        if: env.SAMPLES_CLUSTERING == 'true'
        shell: bash
        run: |
          set -euo pipefail
          test "$ORACLE_DATASET" = shipped
          PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s scripts \
            -p test_infercnv_samples_clustering_witness.py \
            2>&1 | tee "$EVIDENCE_DIR/samples-clustering-checker-tests.log"
          sha256sum scripts/infercnv_samples_clustering_witness.R \
            scripts/validate_infercnv_samples_clustering_witness.py \
            scripts/test_infercnv_samples_clustering_witness.py \
            > "$EVIDENCE_DIR/samples-clustering-code-sha256.txt"
          Rscript --vanilla scripts/infercnv_samples_clustering_witness.R \
            "$ORACLE_OUTPUT" "$SHIPPED_OUTPUT" "$EVIDENCE_DIR/samples-clustering" \
            2>&1 | tee "$EVIDENCE_DIR/samples-clustering-run.log"
          PYTHONDONTWRITEBYTECODE=1 python3 -B scripts/validate_infercnv_samples_clustering_witness.py \
            "$ORACLE_OUTPUT" "$SHIPPED_OUTPUT" "$EVIDENCE_DIR/samples-clustering" \
            2>&1 | tee "$EVIDENCE_DIR/samples-clustering-validation.log"

      - name: Validate fresh full-case measurement pins
        if: env.MATCHED_SNAPSHOT != ''
        shell: bash
        run: |
          set -euo pipefail
          PYTHONDONTWRITEBYTECODE=1 python3 -B - "$SHIPPED_OUTPUT" <<'PY' > "$EVIDENCE_DIR/measurement-input-validation.json"
          import json, pathlib, sys
          sys.path.insert(0, str(pathlib.Path("scripts").resolve()))
          from measure_infercnv import validate_pins
          receipt = json.loads(pathlib.Path(".autopilot/oracles/infercnv-measurement-input-2026-09-26.json").read_text())
          print(json.dumps(validate_pins(pathlib.Path(sys.argv[1]), receipt), sort_keys=True))
          PY

      - name: Test matched R preparation against installed infercnv
        if: env.MATCHED_SNAPSHOT != ''
        shell: bash
        run: |
          set -euo pipefail
          Rscript --vanilla scripts/test_infercnv_matched.R "$SHIPPED_OUTPUT" \
            2>&1 | tee "$EVIDENCE_DIR/matched-r-tests.log"

      - name: Install Rust 1.91 and build exact matched bench
        if: env.MATCHED_SNAPSHOT != ''
        shell: bash
        run: |
          set -euo pipefail
          mkdir -p "$CARGO_HOME" "$RUSTUP_HOME" "$CARGO_TARGET_DIR"
          rustup toolchain install 1.91.0 --profile minimal --no-self-update \
            2>&1 | tee "$EVIDENCE_DIR/rust-install.log"
          (
            cd "$RUST_SOURCE"
            cargo +1.91.0 build --locked --release --features infercnv-measurement \
              --bench cnv_matched --message-format=json \
              > "$EVIDENCE_DIR/cargo-build.jsonl" 2> "$EVIDENCE_DIR/cargo-build.stderr"
          )
          PYTHONDONTWRITEBYTECODE=1 python3 -B - "$EVIDENCE_DIR/cargo-build.jsonl" "$EVIDENCE_DIR/binary-identity.json" "$CARGO_TARGET_DIR" "$GITHUB_ENV" <<'PY'
          import hashlib, json, os, pathlib, stat, sys
          messages = [json.loads(line) for line in pathlib.Path(sys.argv[1]).read_text().splitlines()]
          paths = [pathlib.Path(m["executable"]) for m in messages
                   if m.get("reason") == "compiler-artifact" and
                   m.get("target", {}).get("name") == "cnv_matched" and m.get("executable")]
          if len(paths) != 1:
              raise ValueError(f"expected one exact bench executable, found {len(paths)}")
          binary = paths[0]
          if (not stat.S_ISREG(binary.lstat().st_mode) or not os.access(binary, os.X_OK) or
                  pathlib.Path(sys.argv[3]).resolve() not in binary.resolve().parents):
              raise ValueError("bench executable is missing, linked, non-executable or outside target")
          identity = {"path": str(binary.resolve()), "sha256": hashlib.sha256(binary.read_bytes()).hexdigest()}
          pathlib.Path(sys.argv[2]).write_text(json.dumps(identity, indent=2) + "\n")
          with open(sys.argv[4], "a") as env:
              env.write(f"RUST_BINARY={binary.resolve()}\n")
          PY

      - name: Record matched runtime provenance
        if: env.MATCHED_SNAPSHOT != ''
        shell: bash
        run: |
          set -euo pipefail
          uname -a > "$EVIDENCE_DIR/machine.txt"
          lscpu > "$EVIDENCE_DIR/cpu.txt"
          cat /proc/loadavg > "$EVIDENCE_DIR/load-before.txt"
          rustc +1.91.0 -vV > "$EVIDENCE_DIR/rustc-version.txt"
          cargo +1.91.0 -vV > "$EVIDENCE_DIR/cargo-version.txt"
          /usr/bin/time --version > "$EVIDENCE_DIR/gnu-time-version.txt"
          Rscript --vanilla -e 'cat(capture.output(sessionInfo()), sep="\n")' > "$EVIDENCE_DIR/r-session.txt"
          printf 'OPENBLAS_NUM_THREADS=1\nOMP_NUM_THREADS=1\nMKL_NUM_THREADS=1\nBLIS_NUM_THREADS=1\nVECLIB_MAXIMUM_THREADS=1\n' > "$EVIDENCE_DIR/thread-limits.txt"
          printf 'source=%s\nbinary=%s\nprofile=release\nfeatures=infercnv-measurement\nbench=cnv_matched\n' \
            "$RUST_SOURCE" "$RUST_BINARY" > "$EVIDENCE_DIR/build-configuration.tsv"
          sha256sum scripts/infercnv_matched.R scripts/infercnv_matched_support.R \
            scripts/measure_infercnv.py scripts/verify_infercnv_measurement_snapshot.py \
            scripts/infercnv_source_snapshot.py \
            .autopilot/oracles/infercnv-measurement-input-2026-09-26.json \
            .autopilot/oracles/infercnv-measurement-candidate-2026-09-26.json \
            "$RUST_BINARY" > "$EVIDENCE_DIR/measurement-code-sha256.txt"
          printf 'CARGO_HOME=%s\nRUSTUP_HOME=%s\nCARGO_TARGET_DIR=%s\nTMPDIR=%s\nRUSTFLAGS=%s\n' \
            "$CARGO_HOME" "$RUSTUP_HOME" "$CARGO_TARGET_DIR" "$TMPDIR" "${RUSTFLAGS:-}" \
            > "$EVIDENCE_DIR/build-environment.tsv"

      - name: Run same-host matched trials
        if: env.MATCHED_SNAPSHOT != ''
        shell: bash
        run: |
          set -euo pipefail
          PYTHONDONTWRITEBYTECODE=1 python3 -B scripts/measure_infercnv.py \
            --bundle "$SHIPPED_OUTPUT" --binary "$RUST_BINARY" \
            --output "$EVIDENCE_DIR/matched-results" \
            --input-receipt .autopilot/oracles/infercnv-measurement-input-2026-09-26.json \
            --r-script scripts/infercnv_matched.R \
            2>&1 | tee "$EVIDENCE_DIR/matched-driver.log"

      - name: Verify measurement source after trials
        if: ${{ always() && inputs.matched_snapshot != '' }}
        shell: bash
        run: |
          set -euo pipefail
          cat /proc/loadavg > "$EVIDENCE_DIR/load-after.txt"
          if [[ -d "$RUST_SOURCE" ]]; then
            PYTHONDONTWRITEBYTECODE=1 python3 -B - \
              .autopilot/oracles/infercnv-measurement-candidate-2026-09-26.json \
              ".autopilot/snapshots/sc-cnv-core-2026-09-26/$MATCHED_SNAPSHOT" \
              "$RUST_SOURCE" "$EVIDENCE_DIR" <<'PY' 2>&1 | tee "$EVIDENCE_DIR/source-after.log"
          import pathlib, sys
          sys.path.insert(0, str(pathlib.Path("scripts").resolve()))
          from verify_infercnv_measurement_snapshot import record_extracted_check
          record_extracted_check(*sys.argv[1:])
          PY
          else
            printf 'failed\n' > "$EVIDENCE_DIR/source-after.status"
            printf 'verified source never created\n' > "$EVIDENCE_DIR/source-after.error.txt"
            exit 1
          fi

      - name: Upload raw evidence even on failure
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: infercnv-oracle-${{ github.run_id }}-${{ github.run_attempt }}
          path: ${{ runner.temp }}/infercnv-oracle/
          if-no-files-found: error
          retention-days: 30
```

## Pre-flight and self-review

Task1 produces authenticated raw-root paths and installed source/runtime plus package CLI5; Task2 consumes the same absolute paths and emits six-argument children. Package test passes reused scripts and adjacent helper paths consistently. Original bundle parent (not shipped-bundle) is used. Fixed checker import includes the samples validator; campaign import adds only these two project modules. Both new R sources are completely specified above; no public API/production Rust changes. The campaign schedule/performance requirement belongs to the subsequent driver, not to this two-child contract gate.
