# Same-host raw grouped-i6 Bayesian performance

Extend the accepted functional workflow with a bounded useful-performance
decision, not another per-helper or code-review campaign. Product code
42988589ac4708c560407649d9a1c86f8b6a47be matches native green27 except README.
The full30-family goal remains active; this campaign does not publish SC.

## Scope and fair boundary

Use one Linux x86_64 hosted runner, the pinned infercnv1.28.0/R4.6.1/JAGS
source and the same canonical raw full184-cell shipped input. Run actual
CreateInfercnvObject plus infercnv::run through19 with grouped samples, i6,
seed261027, transition1e-6, posterior normal threshold0.5, no denoise/plots,
no resume and no optional stage RDS. Do not substitute a stage17 replay or
time an exporter/traced coda collector as if it were the actual operation.
The source's unconditional MCMC object/report writes remain part of R work.

R public-call timing excludes package startup and after-call evidence
export. Native timing includes the entire command, parsing and its complete
JSON report including original expression/HMM and Bayesian output. Require
the slowest measured native process to be strictly faster than the fastest
measured R public call; this avoids attributing package startup to an
algorithmic speedup. Record whole R process time separately. GNU time RSS
is observed process peak, not an aggregate fork/process-tree memory claim.

Use R num_threads1 and BLAS/OMP1. Native1 worker is the fair serial comparison;
native4 workers is a separate scaling observation. Preserve actual available
CPU/affinity, OS/CPU/memory, input/source/binary/driver hashes, all flags,
versions, logs, return codes, user/system time and RSS. Use the accepted
Linux binary from run40510071393/artifact12737096223 with its exact digest,
not a fresh unqualified build. All acquisition verification precedes timing.

## Execution

1. An isolated worker owns only the R whole-run driver and one small critical
   source/record test. It reuses existing bundle/run-parameter helpers,
   retains actual original/adjusted report paths and automatic MCMC output,
   records timing after the actual call, and performs heavy export outside
   the measured process. It does not change upstream bodies or install on Mac.
2. Root owns the Python serial campaign, default-false hosted route, receipt,
   retained evidence and decision. Run one R/native warmup, then three R
   trials interleaved with native1/4 trials, plus two extra native1/4 pairs.
   No duplicate oracle/HMM/denoise campaigns or local compilation.
3. Require complete source calls and original/adjusted reports. Original HMM
   reports retain the accepted stable contract; Bayesian removals/IDs/report
   structure must agree, with the existing source-indeterminate state3/4
   policy kept explicit. Native fingerprints must agree across every trial
   and both worker counts. Do not accept partial or failed timing samples.
4. Record full distributions and memory provenance. Claim only this real,
   non-trivial small-cohort profile; larger cohorts, cold-I/O, default
   subclusters, general convergence and biological validity remain excluded.

The existing external-disk gate remains: boot APFS>80% prohibits local
build/test/install. Hosted execution uses runner-temp storage. Do not delete,
publish, create reminders/automations or request repeated human approval.
