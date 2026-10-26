# Samples CLI performance implementation plan

Use executing-plans inline. The user authorizes routine choices and requests
workflow-first implementation, a few meaningful checks and one overall review.

Goal: obtain actual whole-process time and peak RSS for the accepted raw-to-
samples CLI at1 and4 workers, then use the evidence to guide optimization.
This is one step toward the full portfolio goal, not a replacement release gate.

Spec: the accepted2026-10-23 samples workflow plan, AGENTS.md, and this bounded
measurement contract. Product source is unchanged green20, committed through
f8a10a0. No local Rust/R build while APFS boot remains over80%.

## Measurement contract

- Canonical real shipped input:9939 incoming genes/184 cells; retained8508,
  selected6187, reference groups Microglia19 and Oligodendrocytes23, pooled
  observations142. Plain input first; no cold-I/O or larger-cohort claim.
- Time a fresh release CLI process from raw TSV through preprocessing,
  filtering, distances, Ward and its default JSON report. Do not include
  compilation or validation of the report; expression export stays disabled.
- One warm-up per worker count followed by five alternating measured pairs.
  Keep every output, command, stdout/stderr, time/RSS result and process record.
- Reuse infercnv_trial_process.run_child for bounded process-group ownership
  and existing GNU-time format/parser from measure_infercnv. No new runner
  or per-helper process-contract framework.
- Report GNU-time process wall/user/system/maxRSS plus high-resolution owner
  elapsed time; the latter includes observer overhead. Record binary/input/
  controller hashes, flags, CPU affinity/count/quota, host/load and versions.
- Every report must pass canonical shape/group/selection checks and have the
  same normalized content after removing only settings.threads. Failure leaves
  raw partial results and exits nonzero; no complete summary is fabricated.
- Report all measured values with median/min/max and serial/parallel ratio.
  Slower or noisy parallel execution is a valid result, not a failed benchmark.
  This does not establish an advantage over R or finish full inferCNV.

## Files and tasks

- [x] Reconcile the accepted green20 outcome with tracked control-plane state.
- [x] Add scripts/measure_infercnv_samples_cli.py and a small test module for
  report normalization, summary statistics and incomplete/invalid results.
  Run focused Python tests locally with external TMPDIR; no local compilation.
- [x] Add an explicit samples_benchmark input to the existing native workflow.
  It selects Linuxx86 only, requires the original samples receipt, rejects RED/
  mixed measurement modes, reuses source/oracle admission and native tests,
  then measures the real binary. Normal runs retain all four native targets.
  Retain the actual executable with benchmark evidence for subsequent work.
- [ ] Run controller verification and one overall review, push exact source,
  wait for exact-head CI, execute the actual hosted benchmark, inspect raw
  output/metrics and record the real result. Do not infer speed from tests.

## Review focus and continuation

One whole-feature static review completed. Its actionable retention finding
is fixed: copy/hash the executable before invoking the benchmark, then run
that retained copy. Failure therefore preserves the actual binary alongside
partial logs/reports. No production algorithm or old oracle was changed.

Check timing boundary, discarded warm-ups, thread-only normalization, paired
order, interrupted partial runs and the distinction between requested threads
and available CPUs. Existing process-helper tests cover lifecycle details;
do not duplicate them here. The benchmark mode is not the four-native release
gate. All binaries/data/scratch remain external locally or in runner-temp.

Then address any demonstrated hot-path problem, measure a genuinely larger
cohort and compare the equivalent scope against the installed pinned R tool
on the same host. Existing prepared-only and warm-ingestion speedups remain
separate. Continue HMM/denoise and other missing inferCNV workflows, followed
by the rest of the product portfolio; no unfinished operation is advertised.
