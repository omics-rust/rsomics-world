# Test-first native CNV snapshot: red-1

Frozen from the unpublished local `rsomics-sc` repository before implementation.
It has no product commit yet. `files.sha256` identifies every archived source
file; `checksums.sha256` identifies this receipt, that manifest, and the archive.

The expected failure is Rust E0583: `src/lib.rs` declares `cnv`, but no
`src/cnv.rs` or `src/cnv/mod.rs` exists. Dependency, runner, or fixture failures
do not satisfy this gate. The remote run generates the first Cargo.lock;
that lockfile must return to the product before any green snapshot.

Thirteen contract tests and a four-profile, ten-stage numerical regression
precede the core. Fixture TSVs are byte-identical selected outputs from
accepted infercnv 1.28.0 oracle run 36216764707, source commit
`b421d9405c97a309b081ef86d455e976df93eae4`. Their own manifest and provenance
are included. No RDS, upstream package, build products, or Git directory is
included. This source has no binary, is not publishable, and starts from
already prepared stage-1 counts rather than a complete raw-input workflow.
