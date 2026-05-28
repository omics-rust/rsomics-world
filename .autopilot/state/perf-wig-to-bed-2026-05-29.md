# perf-wig-to-bed — 2026-05-29

## Tool
rsomics-wig-to-bed 0.1.0

## Upstream reference
convert2bed (bedops 2.4.42, conda rs-up, aarch64-apple-darwin)

## Machine
mini_m2 — Apple M2, aarch64-apple-darwin, macOS 15.x

## Fixture
File: /tmp/perf_signal_large.wig
Size: ~7 MB
Rows: 1,000,000 signal values across 10 chromosomes (chr1–chr10)
Format: fixedStep, step=100bp, windowSize=100bp
Generator: Python seed=42

## Command
```
hyperfine --warmup 3 --runs 7 \
  'rsomics-wig-to-bed /tmp/perf_signal_large.wig > /dev/null' \
  'convert2bed -i wig < /tmp/perf_signal_large.wig > /dev/null'
```

## Results

| Command | Mean ± σ | Min | Max |
|---|---|---|---|
| rsomics-wig-to-bed | 237.5 ms ± 23.4 ms | 214.8 ms | 270.5 ms |
| convert2bed (bedops) | 2.134 s ± 0.717 s | 1.630 s | 3.392 s |

## Ratio
**8.99 ± 3.14× faster** — PASS (>1.0× gate met)

## Status
PASS
