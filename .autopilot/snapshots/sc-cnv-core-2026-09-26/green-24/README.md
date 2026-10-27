# Integrated grouped i6 HMM candidate

Source baseline is rsomics-sc c5db7c74 with the complete native raw-to-HMM
grouped-samples candidate. It adds hidden-spike simulation/calibration,
R-aligned seeded draws, adjusted state decoding, deterministic CNV regions,
shared preprocessing-tail reuse and an opt-in CLI/JSON result. No dependency
or lockfile changes. Expression14 and optional denoise22 remain distinct.

This is an unaccepted implementation snapshot, not a publication or a claim
that inferCNV is complete. The single integrated gate covers four native
targets and actual retained R states/calibration/reports for full, proxy and
aggregated reference cases, plus 1/4-worker reproducibility. Subclusters,
i3/per-cell inference, Bayesian filtering and plots remain later work.
The owning product remains publish=false.
