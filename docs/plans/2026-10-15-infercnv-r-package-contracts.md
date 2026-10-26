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

## Pinned-oracle workflow source

The last pre-denoise workflow definition remains available at
[its exact commit](https://github.com/omics-rust/rsomics-world/blob/45cf3f4df688356a1ebbae619f1582548747ab9e/.github/workflows/infercnv-oracle.yml).
The [current workflow](../../.github/workflows/infercnv-oracle.yml) is the
single maintained executable definition. Its real selector, fixed-code,
postcheck and upload behavior is tested directly; this historical plan no
longer duplicates hundreds of executable YAML lines or pins future features
to a prose copy. Original run/source evidence is unchanged.

## Pre-flight and self-review

Task1 produces authenticated raw-root paths and installed source/runtime plus package CLI5; Task2 consumes the same absolute paths and emits six-argument children. Package test passes reused scripts and adjacent helper paths consistently. Original bundle parent (not shipped-bundle) is used. Fixed checker import includes the samples validator; campaign import adds only these two project modules. Both new R sources are completely specified above; no public API/production Rust changes. The campaign schedule/performance requirement belongs to the subsequent driver, not to this two-child contract gate.
