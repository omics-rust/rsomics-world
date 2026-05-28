# perf-seacr — 2026-05-29

## Tool
rsomics-seacr 0.1.0

## Upstream reference
SEACR 1.3 (bioconda seacr-1.3-hdfd78af_2, bash + Rscript, aarch64-apple-darwin)

## Machine
mini_m2 — Apple M2, aarch64-apple-darwin, macOS 15.x

## Fixture
File: /tmp/perf_exp_large.bedgraph (10,000 lines, 257 KB, 5 chromosomes)
File: /tmp/perf_igg_large.bedgraph (3,000 lines, control)
Generator: Python seed=42

## Command
```
hyperfine --warmup 2 --runs 5 \
  'rsomics-seacr exp.bedgraph --control igg.bedgraph --norm non --mode stringent --output /tmp/ours' \
  'bash SEACR_1.3.sh exp.bedgraph igg.bedgraph non stringent /tmp/ref'
```

Note: SEACR_1.3.sh requires a patched version on macOS due to `head /dev/urandom | LC_CTYPE=C tr`
producing binary characters in filenames on macOS (known Linux-targeting issue). Patched version
uses `cat /dev/urandom | LC_ALL=C tr -dc 'A-Za-z0-9'`. Computational output is identical.

## Results

| Command | Mean ± σ | Min | Max |
|---|---|---|---|
| rsomics-seacr | 3.3 ms ± 0.9 ms | 2.3 ms | 4.5 ms |
| SEACR_1.3.sh (R+awk) | 3.866 s ± 1.331 s | 2.420 s | 5.735 s |

## Ratio
**~1180× faster** (SEACR baseline dominated by Rscript startup + awk pipeline)

## Status
PASS — rsomics-seacr is a compiled Rust binary doing the same AUC + threshold computation
without R or awk overhead. The comparison is fair: same algorithm, same inputs, same outputs.
