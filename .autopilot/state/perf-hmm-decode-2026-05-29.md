# perf-hmm-decode — 2026-05-29

## Tool
rsomics-hmm-decode 0.1.0

## Upstream reference
Pure-Python Viterbi reference implementation (Python 3.12.7, aarch64-apple-darwin)
No CLI upstream exists; Python is the reference algorithm baseline for HMM decoding.

## Machine
mini_m2 — Apple M2, aarch64-apple-darwin, macOS 15.x

## Fixture
File: /tmp/perf_obs_large.txt (10,000 observation sequences, 1.25M total observations)
Model: 2-state HMM with 2 symbols (golden model.json)
Generator: Python seed=42

## Command
```
time rsomics-hmm-decode -m model.json obs_large.txt > /dev/null
time python3 viterbi_ref.py model.json obs_large.txt > /dev/null
```

## Results

| Command | Wall time |
|---|---|
| rsomics-hmm-decode | 0.207s |
| Python Viterbi reference | 2.830s |

## Ratio
**~13.7× faster** than pure-Python Viterbi — PASS (>1.0× gate met)

## Status
PASS
