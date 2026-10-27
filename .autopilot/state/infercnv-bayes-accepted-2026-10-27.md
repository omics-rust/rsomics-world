# Accepted native grouped-i6 Bayesian workflow

Bounded functional acceptance, not full inferCNV, a convergence claim or a
public release. The complete raw grouped-i6 CLI now preserves expression14
and original HMM17 while adding separate posterior/filter/reassignment19.
No foundation, dependency, lockfile or public repository was added.
Owning product implementation and README are committed as
42988589ac4708c560407649d9a1c86f8b6a47be; product worktree is clean and
publish=false. There is not yet a public SC remote to push to.

Source: green27 on world92e21b3983e6563193a1f3f2e46a4fab17ef2dd4.
Archive SHA dc13d219dc63b5cda894ed6751706b369f28696f37a9efe6cfbf9e16f07c909c;
182-file manifest bb92aeb6c7d04119c63735c7886e0fa1f533303b5f6d73d528baac747b5c15a5.
Root verifies all candidate files against the owning product before its
documentation update. Exact-head controller40509412691 and native
40510071393 both pass. All four native Linux/macOS x86_64/aarch64 targets
pass136 debug and136 release tests, formatting and strict Clippy.

Actual source oracle40499067515/artifact12735046764 is pinned by
.autopilot/oracles/infercnv-bayes-workflow-2026-10-27.json. Its two independent
real JAGS repetitions and coda-derived uncertainty remain in the original
verified artifact; witness SHA
69ca75877f3960ed4b8b20b26bb2db14cfe92357ee7d4c069fed9e5f77397fb6.
Native comparison first rechecks the accepted original expression and HMM
against40477088139, then compares the Bayesian operation against both
repetitions. Eight full raw-input native processes cover1/4 workers on four
platforms, each retaining8508 genes,184 cells, six groups and82 regions.

Every comparison removes the same1 region and retains81; final reassigned
states, original IDs/ranges, memberships, coordinates and report ordering
agree. The same three actual state3/4 ambiguities remain explicitly labeled
(chr3-region_59, chr5-region_161, chr16-region_182); zero observed state
differences does not turn those uncertain source maxima into exact general
contracts. Largest scaled mixture error is0.40327197 and cell-frequency
error0.47 under the predeclared statistical contract; no tolerance changed
after native execution. Native1/4 fingerprints agree within each platform.
Cross-platform report byte identity is not claimed.

Root retains original run/job API, native ZIPs and safely extracted evidence
under rsomics-fixtures/evidence/infercnv-bayes-native-2026-10-27/
run-40510071393/. ZIP CRC/API digest/size, source archive/manifest, binary
hashes, all eight complete comparisons and test counts have been checked.

| Platform | Artifact | ZIP SHA256 | Binary SHA256 |
|---|---|---|---|
| Linux x86_64 |12737096223|cce5f5902e7cd6b9068422258c63d4be266972e9c27584b152f096cdb60e7637|1c28b34984934199fe35451ec811e528da7226a17da3d621a923ef620be8cbe6|
| Linux aarch64 |12737060691|5fa6b79074b643c27a19eed453e1ddabb5e7cdf3dbea175cdf443cff442ab25d|36368513324246ac0e6e73c25dd549fcbfbc8e3adcf3cfec743fd98d5c6f07e3|
| macOS x86_64 |12737985196|1c8e3fb8085db78b6e56c760f91a3b5f1da4c7a7227d368e0da3e8cf1d9e0c07|8f63321203cdc27e45eaf4dbd442c2aec234ed5c88f06fccc113deb25bfe55bc|
| macOS aarch64 |12736647767|a6243ba5b2cb166afe398660fd6fdffbfce4dddaf4c29174efd1cb20b1ee6653|8a27eab0ae931e2f797677c62923f464fea26f034763ad029fcca22bc097a30a|

The single whole-feature review's global-single-state skip and zero-disables
help findings are fixed and exercised by this native gate. No second feature
review or per-helper campaign is needed. The remaining declined-to-judge
boundaries and stochastic limits stay in infercnv-bayesian-review-2026-10-27.md
and infercnv-bayesian-statistical-contract-2026-10-27.md.

Linux x86_64 native owner wall observations are1.409/1.101s with about
104.4/104.6MiB peak RSS at1/4 workers. These are not paired/distribution-based
replacement-performance evidence. Next: same-host full raw R/native grouped
i6+Bayesian timing and memory, actual default Leiden/PCA source inspection,
and the remaining CNV/product-family work. Ward membership policy is held
only for its pending user choice; the full portfolio goal remains active.
