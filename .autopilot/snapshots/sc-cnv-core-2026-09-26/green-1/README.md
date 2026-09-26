# Native CNV candidate: green-1

Frozen from the unpublished local `rsomics-sc` repository after verified
test-first run 36218066698, exact control-plane head
`4e8c25c7c475c855f7175a954c5bf2731a45cc74`. That run failed with E0583 for
the absent `cnv` module; dependency resolution and source checks passed.

This candidate adds the real prepared-counts pre-clustering core and 23 tests.
It addresses the test-only review's length, callback-count, returned-identity,
and non-square accessor gaps. Numerical tolerance remains unchanged.
Cargo.lock is the byte-identical remotely generated red-run lockfile, SHA-256
`636ead1961f573b303be8c02f6068201acd11dde1d0cb9bab3d81ccf86bf5554`.

`files.sha256` identifies every archived file. `checksums.sha256` identifies
this receipt, that manifest, and the archive. Fixture provenance is retained
inside the source. No build products, Git metadata, RDS, or upstream package
are included. The candidate has not yet passed native tests or independent
implementation review. It is not a complete raw-input CNV workflow, a public
CLI, a publication candidate, or performance evidence.
