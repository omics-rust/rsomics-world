# R raw-creation runtime helper implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans. Root integrates/observes real RED/GREEN; reviewers remain bounded read-only.

**Goal:** Add the minimal private runtime/metric/lifetime-I/O helper used by the reviewed actual R factory trial; execute only its package-free contracts now.

**Architecture:** Leave accepted creation support and reused R/Python unchanged. New helper validates the runtime record, twelve typed factory counters, literal TSV writing with existing-path rejection and raw lifetime-I/O copying. Outputs require the campaign's isolated single-writer directory; creation is not atomic-exclusive. The actual installed-package source/trial and external prelaunch authentication remain separate.

**Tech stack:** Existing remote4090 Linux R4.6.1/base/stats/utils, external local/remote scratch, no installation/build/download. Mac APFS>80% prohibits local R/Cargo execution.

**Spec:** docs/plans/2026-09-30-infercnv-raw-ingestion-measurement-design.md. Revised R source reviewed at external revised-trial-20260930; its earlier8group worker pass is not an owning gate.

## Scope and source review

Root fully read both complete files and the revised trial/package proposal. Fresh independent source review found no critical/important lifecycle/counter/source/metric defect. Exact helper SHA bcbbd0e3b6a2d053562345cd9ee59a4ff3e9dc4c8d4e6e2a7db66097d5787418; pure test35442d32734d592dd21a305e6e4527bf33994da8917e93e4d3d9b7807eb657b4.

Records require R4.6.1/Bioc3.23/infercnv1.28/Linuxx86, aggregate C and six named C locale categories, seven thread-limit strings1. This function tests records; a later real trial must populate them from actual runtime. No fake factory or loading infercnv occurs in pure tests.

Twelve native-compatible key/value metric fields; positive finite wall, nonnegative preparation/selfCPU, zero childCPU, positive exact-integer R RSS<=2^53-1. All literal metric fields have no tab/CR/LF. Existing/broken-symlink destinations rejected; I/O copy preserves raw bytes rather than reinterpretation or factory-delta claim.

Minor review limits: package test uses as.numeric and must be supplemented by the accepted Python literal parser on real outputs; category vector excludes LC_PAPER/LC_MEASUREMENT but actual aggregate locale must remain C. These are not upgraded to guarantees by this helper gate.

### Task 1: Genuine missing-helper RED

**Files:** scripts/test_infercnv_ingestion_runtime_pure.R only.

- [x] Add exact complete test below, runtime helper physically absent.
- [x] Create a new remote-owned /dev/shm/rsomics-ingestion-runtime-root-20260930.XXXXXX directory with mktemp; preserve it and all outputs. Check native R4.6.1 and free /dev/shm; no packages/install.
- [x] Copy only the test; execute actual Rscript --vanilla under LC_ALL=C/explicit TMPDIR, preserve exit/status/stdout/stderr and exact missing infercnv_ingestion_runtime_support.R diagnostic. Environment/parse failure is not RED.
- [x] Copy raw evidence back to new external r-runtime-4090 directory. Never overwrite worker or earlier root evidence.

Complete test:

```r
#!/usr/bin/env Rscript

flag <- grep("^--file=", commandArgs(FALSE), value=TRUE)
if (length(flag) != 1L) stop("cannot identify runtime contract test")
script_dir <- dirname(normalizePath(sub("^--file=", "", flag)))
source(file.path(script_dir, "infercnv_ingestion_runtime_support.R"), local=TRUE)
if (!identical(as.character(getRversion()), "4.6.1")) stop("pure gate requires existing R 4.6.1")
root <- tempfile("ingestion-runtime-", tmpdir=Sys.getenv("TMPDIR"))
if (!nzchar(Sys.getenv("TMPDIR")) || !dir.create(root)) stop("explicit scratch directory required")
count <- 0L
check <- function(label, action) {
    force(action)
    count <<- count + 1L
    cat(label, "passed\n")
}
fails <- function(action, message) {
    error <- tryCatch({force(action); NULL}, error=identity)
    if (!inherits(error, "error") || !grepl(message, conditionMessage(error), fixed=TRUE)) stop("bounded rejection absent: ", message)
}
runtime <- list(R_version="4.6.1", Bioconductor_version="3.23", infercnv_version="1.28.0",
                system="Linux", architecture="x86_64", locale_all="C",
                locale=stats::setNames(rep("C", length(locale_categories)), locale_categories),
                thread_environment=stats::setNames(rep("1", length(thread_keys)), thread_keys))
check("accepted runtime record", stopifnot(require_runtime_contract(runtime)))
check("wrong actual runtime rejected", {
    for (name in c("R_version", "Bioconductor_version", "infercnv_version", "system", "architecture")) {
        changed <- runtime
        changed[[name]] <- "wrong"
        fails(require_runtime_contract(changed), "runtime requirement differs")
    }
})
check("non-C locale rejected", {
    changed <- runtime
    changed$locale[["LC_CTYPE"]] <- "C.UTF-8"
    fails(require_runtime_contract(changed), "locale categories differ")
    changed$locale <- runtime$locale[-1L]
    fails(require_runtime_contract(changed), "locale categories differ")
    changed <- runtime
    changed$locale_all <- "C.UTF-8"
    fails(require_runtime_contract(changed), "locale categories differ")
})
check("unset or non-one thread limit rejected", {
    changed <- runtime
    changed$thread_environment[[1L]] <- "2"
    fails(require_runtime_contract(changed), "thread environment differs")
    changed$thread_environment <- runtime$thread_environment[-1L]
    fails(require_runtime_contract(changed), "thread environment differs")
})
counters <- c(region_wall_seconds=0.25, region_user_seconds=0.125, region_system_seconds=0,
              region_child_user_seconds=0, region_child_system_seconds=0)
metrics <- creation_metrics("plain", 1, counters, 1024, 2048)
check("literal native-compatible metric bytes", {
    path <- file.path(root, "metrics.tsv")
    write_metrics(path, metrics)
    wanted <- c("key\tvalue", "schema_version\t1", "measurement\traw_creation", "implementation\tinfercnv", "input_case\tplain",
        "preparation_wall_seconds\t1.00000000000000000e+00", "region_wall_seconds\t2.50000000000000000e-01",
        "region_user_seconds\t1.25000000000000000e-01", "region_system_seconds\t0.00000000000000000e+00",
        "region_child_user_seconds\t0.00000000000000000e+00", "region_child_system_seconds\t0.00000000000000000e+00",
        "baseline_rss_bytes\t1024", "returned_rss_bytes\t2048")
    stopifnot(identical(readLines(path), wanted))
    fails(write_metrics(path, metrics), "new metadata file required")
    stopifnot(identical(readLines(path), wanted))
})
check("invalid timing or RSS rejected", {
    for (value in c(0, -1, Inf, NaN)) fails(creation_metrics("plain", 1, replace(counters, 1L, value), 1, 1), "invalid creation metrics")
    for (value in c(0, -1, 1.5, Inf)) fails(creation_metrics("plain", 1, counters, value, 1), "invalid creation metrics")
    fails(creation_metrics("unknown", 1, counters, 1, 1), "invalid creation metrics")
    fails(creation_metrics("plain", -1, counters, 1, 1), "invalid creation metrics")
    fails(creation_metrics("plain", 1, replace(counters, 4L, 1), 1, 1), "invalid creation metrics")
})
check("metric keys and separators rejected before creation", {
    path <- file.path(root, "invalid.tsv")
    changed <- metrics
    names(changed)[[2L]] <- names(changed)[[1L]]
    fails(write_metrics(path, changed), "metric table contract differs")
    changed <- metrics
    changed[[3L]] <- "bad\tvalue"
    fails(write_metrics(path, changed), "metric table contract differs")
    stopifnot(!file.exists(path))
})
check("lifetime I/O bytes copied without interpretation", {
    source_path <- file.path(root, "io-source")
    wanted <- charToRaw("rchar: 123\nsyscr: 2\nread_bytes: 0\n")
    writeBin(wanted, source_path)
    output <- file.path(root, "io.lifetime.txt")
    capture_lifetime_io(source_path, output)
    stopifnot(identical(readBin(output, "raw", n=1000L), wanted))
    fails(capture_lifetime_io(source_path, output), "new metadata file required")
})
cat("Runtime helper contracts passed:", count, "; package-free, no factory or performance claim.\n")
```

### Task 2: Reviewed helper GREEN

**Files:** scripts/infercnv_ingestion_runtime_support.R.

- [x] After observed missing-file RED, add exact complete helper below with apply_patch.
- [x] Compare owning/proposal SHA; copy helper to same remote gate. Parse both actual files and execute unchanged complete test. Require8 actual behavior-group success markers/exit0; preserve full outputs. No source/controller weakening or numerical/timer claim.
- [x] Check local/remote before/after source identities, original accepted helper/reused scripts unchanged. Run all274 current Python controller tests/architecture/whitespace with external TMPDIR/bytecode disabled. Count actual output; no local R run.

Complete helper:

```r
locale_categories <- c("LC_COLLATE", "LC_CTYPE", "LC_MONETARY", "LC_NUMERIC", "LC_TIME", "LC_MESSAGES")
thread_keys <- c("OPENBLAS_NUM_THREADS", "OMP_NUM_THREADS", "MKL_NUM_THREADS", "BLIS_NUM_THREADS",
                 "VECLIB_MAXIMUM_THREADS", "NUMEXPR_NUM_THREADS", "RAYON_NUM_THREADS")
metric_keys <- c("schema_version", "measurement", "implementation", "input_case", "preparation_wall_seconds",
                 "region_wall_seconds", "region_user_seconds", "region_system_seconds",
                 "region_child_user_seconds", "region_child_system_seconds", "baseline_rss_bytes", "returned_rss_bytes")

require_runtime_contract <- function(value) {
    wanted <- list(R_version="4.6.1", Bioconductor_version="3.23", infercnv_version="1.28.0",
                   system="Linux", architecture="x86_64")
    if (any(!vapply(names(wanted), function(name) identical(value[[name]], wanted[[name]]), logical(1)))) {
        stop("runtime requirement differs")
    }
    if (!identical(value$locale_all, "C") || !identical(names(value$locale), locale_categories) ||
        anyNA(value$locale) || any(value$locale != "C")) {
        stop("locale categories differ")
    }
    if (!identical(names(value$thread_environment), thread_keys) || anyNA(value$thread_environment) ||
        any(value$thread_environment != "1")) stop("thread environment differs")
    invisible(TRUE)
}

creation_metrics <- function(input_case, preparation, counters, baseline, returned) {
    rss <- c(baseline, returned)
    if (!is.character(input_case) || length(input_case) != 1L || is.na(input_case) ||
        !(input_case %in% c("plain", "gzip")) || !is.numeric(preparation) || length(preparation) != 1L ||
        !is.finite(preparation) || preparation < 0 || !is.numeric(counters) ||
        !identical(names(counters), metric_keys[6L:10L]) || any(!is.finite(counters)) || any(counters < 0) ||
        counters[[1L]] <= 0 || any(counters[4L:5L] != 0) || !is.numeric(rss) || length(rss) != 2L ||
        any(!is.finite(rss)) || any(rss <= 0 | rss != trunc(rss) | rss > 2^53 - 1)) stop("invalid creation metrics")
    stats::setNames(c("1", "raw_creation", "infercnv", input_case, sprintf("%.17e", preparation),
                      sprintf("%.17e", counters), sprintf("%.0f", rss)), metric_keys)
}

write_metrics <- function(path, metrics) {
    if (!is.character(metrics) || !identical(names(metrics), metric_keys) || anyNA(metrics) ||
        any(grepl("[\t\r\n]", metrics))) stop("metric table contract differs")
    link <- Sys.readlink(path)
    if (file.exists(path) || (!is.na(link) && nzchar(link))) stop("new metadata file required")
    write.table(data.frame(key=names(metrics), value=unname(metrics)), path, sep="\t", row.names=FALSE, quote=FALSE)
    invisible(TRUE)
}

capture_lifetime_io <- function(source_path, output_path) {
    link <- Sys.readlink(output_path)
    if (file.exists(output_path) || (!is.na(link) && nzchar(link))) stop("new metadata file required")
    input <- file(source_path, "rb")
    on.exit(close(input))
    chunks <- list()
    repeat {
        chunk <- readBin(input, "raw", n=65536L)
        if (!length(chunk)) break
        chunks[[length(chunks) + 1L]] <- chunk
    }
    bytes <- do.call(c, chunks)
    if (!is.raw(bytes) || !length(bytes)) stop("empty lifetime I/O snapshot")
    writeBin(bytes, output_path)
    invisible(TRUE)
}
```

Run remotely with explicit actual paths derived from mktemp result, not a guessed old gate:

```text
env LC_ALL=C TMPDIR=REMOTE_GATE PYTHONDONTWRITEBYTECODE=1 /usr/bin/Rscript --vanilla REMOTE_GATE/test_infercnv_ingestion_runtime_pure.R
```

### Task 3: Scoped delivery and real factory next

- [x] Fresh complete owning source/results review. Commit only2 new R files and owned plan/state: test(sc): validate R creation runtime contracts (1a364674).
- [x] Push main; wait exact-head Control plane CI and record actual hashes/run (38688021631, exact head36299d7f includes1a364674).
- [x] Continue separate installed-package/source/trial/bootstrap/workflow plan without an automatic pause (2026-10-15-infercnv-r-package-contracts.md; running38690064479).

Pure gate is not infercnv execution, actual runtime authentication, clock-documentation verification, factory performance or complete root inventory. Revised real trial152lines/package194lines remain scratch-only until their own complete plan. That gate must authenticate original accepted artifact and code/imports BEFORE launch/after, pin runtime/namespace/source functions/digest, preserve actual clock docs/warnings/Inf options/lifetime IO and independently check real plain/gzip warm/measured states. Paired32-process/64-state performance, native Ward/HMM/Leiden and public delivery remain separate.
