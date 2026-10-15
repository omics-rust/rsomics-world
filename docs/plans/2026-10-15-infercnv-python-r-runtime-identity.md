# Python identity across the installed R boundary

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans.

**Goal:** Require the same Python patch runtime outside and inside R before any installed creation contracts are accepted.

**Architecture:** Keep the actual R trial, package test, helpers and twelve code/input pins unchanged. Add one real cross-process preflight with dynamic-library provenance to the installed-oracle workflow. After observed mismatch RED, select the native Ubuntu24.04 Python3.12 instead of mixing a hosted-toolcache executable with R-prepended system libpython.

**Spec:** `2026-09-30-infercnv-raw-ingestion-measurement-design.md`, source/runtime provenance requirements; `2026-10-15-infercnv-r-package-contracts.md`.

## Evidence and hypothesis

Candidate run38696325035 succeeds, but actual R-child metadata records a toolcache3.12.15 executable path/hash with Python3.12.3 version; outside-R version is3.12.15. Both are within the existing3.12 minor requirement, so value/counter checks do not detect this weaker provenance contract. Do not accept runtime identity or paired timing from the green UI alone. Root independently rechecks the exact parent/four child outputs and preserves this candidate.

Hypothesis: R's ldpaths prepends the system library directory before toolcache's LD_LIBRARY_PATH, causing a toolcache executable to load Ubuntu's libpython. Existing4090 R4.6.1 ldpaths demonstrates this prefix mechanism; the actual hosted R configuration/maps must prove it before choosing the fix.

## Constraints and review focus

- No production Rust, numerical/timer boundary, accepted R helper, package assertion or trial-source changes.
- No local R/build/install/download; all evidence/scratch external, native hosted execution.
- Keep candidate38696325035 original APIs/ZIP/logs and results unchanged; this is not final runtime acceptance.
- Probe executes the actual Python from R, captures exact version, executable, only LD_LIBRARY_PATH, mapped libpython paths/hashes and actual R ldpaths. Do not dump credentials/full environment.
- Regression compares shell and R-spawned versions; inconsistent identity rejects before expensive dependency installation/factory execution.
- Capture the existing three executable baselines immediately after R setup, before the new probe. Keep the later pre-factory recheck and always-postcheck; an intended early runtime rejection must still leave valid executable/source postchecks, not a missing downstream baseline error.
- Native interpreter selection is conditional on ingestion_contracts only; old prepared/default oracle behavior stays unchanged.
- Do not set global R loader paths to make Python work, alter factory runtime library resolution, wrap an executable while labeling the wrapper a binary, or accept version mismatch.
- Full controller tests, YAML/Bash validation, exact-head CI and original evidence audit remain required.

## Task 1: Actual boundary RED

- [x] Add the complete preflight below after setup-r and before Install JAGS/system headers in the workflow and its complete package-plan copy. Existing executable baseline capture moves here; later rechecks remain.
- [ ] Validate YAML/Bash, run controller/architecture/whitespace checks; commit only workflow/owned plan/state, push and wait exact-head CI.
- [ ] Dispatch ingestion-only shipped mode. Require the exact runtime mismatch after raw input and R setup prerequisites succeed; inspect actual R ldpaths and mapped library provenance. No factory rerun may precede that rejected boundary.
- [ ] Preserve original run/jobs/artifact/logs externally. If hypothesis is wrong, investigate the actual loaded runtime rather than install another workaround.

```yaml
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

```

## Task 2: Native interpreter GREEN

- [ ] Only after hosted RED proves the runtime/library conflict, replace setup-python action with the complete conditional native selection below. It uses an explicit Ubuntu-system executable and prepends a symlink directory only to PATH; no R LD_LIBRARY_PATH changes.
- [ ] Keep the boundary preflight unchanged; validate complete workflow/package-plan copy and all twelve code/receipt pins.
- [ ] Full controller/YAML/Bash/architecture/whitespace; commit scoped fix, push and wait exact-head CI.
- [ ] Dispatch the same ingestion-only mode. Require preflight identity agreement, actual package/factory tests, independent outputs/counters and source/executable postchecks all successful.
- [ ] Authenticate original APIs/ZIP/CRC/safe inventory and recheck all four warm/measured states, parent plain/gzip states, twelve metrics, Python consistency and actual runtime/clock/code/source provenance. Update the installed-package acceptance ledger only after that.
- [ ] Fresh whole-change review of this workflow-only correction; ledger declined scope and any minor limits. Paired32/64 performance and full inferCNV/publication remain separate.

```yaml
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
```

## Ruling

This strengthens the private runtime-authentication boundary, not the public tool's numerical semantics or its Python-minor requirement. Cost if wrong: native system Python may be absent or another loader conflict remains; explicit executable/version tests and the unchanged R-spawned probe reject it before package execution. Do not waive the mismatch because both versions happen to satisfy Python3.12.
