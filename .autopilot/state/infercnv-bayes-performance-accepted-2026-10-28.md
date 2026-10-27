# Bounded grouped-i6 Bayesian performance acceptance

Actual same-host campaign40514973287/job122226430314 succeeds on
worldd7d233d449a0057a6560760d37a2a819b59fd609. The measured native binary is
the accepted Linux x86_64 binary from40510071393, SHA
1c28b34984934199fe35451ec811e528da7226a17da3d621a923ef620be8cbe6.
No rebuilt or unaccepted discovery binary is substituted.

Original684193165-byte artifact12742461145 is retained at
rsomics-fixtures/evidence/infercnv-bayes-performance-2026-10-27/
run-40514973287/. Run/job/artifact API, original ZIP, raw measured outputs and
normal source-bundle verification are retained. ZIP SHA
11eec312ddb0bb41364be2390c74b47b28235efa276d23ba0af5dcb244d94b37;
summary SHA090ab52f9c19a93e515c19e493ffe6a5e3e49760605225e6d4b80061d849760b.
Receipt: .autopilot/oracles/infercnv-bayes-performance-2026-10-27.json.

## Equivalent scope and observed results

The real shipped raw input has8508 genes,184 cells and six original groups.
R invokes actual CreateInfercnvObject plus run through19 with grouped i6,
Bayesian normal threshold0.5, no denoise, no resume and no stage-RDS reload.
The performance driver has no tracing or matrix export within the timed call.
R startup is excluded; the native timing includes its whole CLI and JSON.
Both use the same warm-cache inputs and four-CPU affinity on one Linux
x86_64 AMD EPYC9V45 runner, with R/BLAS/OMP constrained to one worker.
Input/code/tool versions, exact argv and per-process CPU/RSS are retained.

One R warmup precedes three measured source calls. Each native1/4-worker
profile has one warmup and five measured calls, with worker order alternated.

| Complete operation | Median seconds | Observed range | Median process peak RSS |
|---|---:|---:|---:|
| R public Create+run19 |402.619|387.966–419.891|1694285824 bytes|
| Native CLI,1 worker |1.38|1.35–1.43|109416448 bytes|
| Native CLI,4 workers |1.08|1.06–1.14|109555712 bytes|

The conservative source-public/native-whole-process median ratio is291.75x;
the slowest native serial call remains below the fastest source public call.
Native4-worker timings are strictly below the serial range, a1.278x median
gain. Observed serial process peak RSS is15.48x lower than source.
The relevant bounded complete-operation replacement gate passes; this is not
a claim about arbitrary inferCNV workloads or all Rust/R implementations.

## Correctness and limits

Every measured native report has the same fingerprint. Original expression,
HMM states,82 original/81 retained regions, report structure and all stable
Bayesian decisions remain accepted. Source warmup and measured trial2 choose
the other state for chr16-region_182, already one of the three predeclared
source-indeterminate state3/4 maxima. The retained comparator labels these
differences rather than relaxing a criterion or claiming exact stochastic
identity. No other stable/structural difference is observed.

This is a nontrivial184-cell shipped-input profile, not a large-cohort or
cold-I/O benchmark, default discovery acceptance, biological validation or
general MCMC convergence proof. GNU time peak RSS measures each process,
not aggregate fork memory or a hard memory bound. The separately retained
4103-cell samples benchmark is not silently extended to Bayesian refinement.

Continue the whole native PCA/simple discovery gate and the remaining CNV/
portfolio operations. This accepted performance slice does not complete
inferCNV, create a public release or narrow/pause the full30-family goal.
