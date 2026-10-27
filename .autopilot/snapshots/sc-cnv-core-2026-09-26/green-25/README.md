# Grouped i6 HMM Clippy correction

Compared with green24, only src/cnv/hmm/spline.rs changes: collapse a nested
else/if and express one R reference-test literal at its representable binary64
precision. The numerical algorithm, draw order, interfaces, dependencies and
comparison tolerances are unchanged. Green24 already matched all three real
HMM workflows at1/4 workers on Linux aarch64; its Linux x86_64 formatting and
debug/release tests pass, but strict Clippy reports these two style warnings.

This remains the same integrated feature campaign. Run all four native targets
with the actual retained R witness; no R rerun or per-helper campaign. Keep
expression14, HMM states17 and denoise22 separate. No complete-inferCNV or
publication claim; rsomics-sc remains publish=false.
