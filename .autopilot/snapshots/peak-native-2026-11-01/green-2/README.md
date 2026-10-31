# Private Peak live-annotation source snapshot: green-2

Clean owner main 5705a307ccd3a535610f8989583882bd805c8446; 34 exact Git-bound regular source files.
Parent 9431627255dc282992bd1380a41e0a5d729b9edc remains preserved as green-1.
Only four annotation files change: live-storage accounting/lifetimes and one
regression. All original 512 MiB budgets, complete input and scientific semantics
remain unchanged. This is modeled allocation admission, not a strict OS RSS cap.

Pinned local fmt/strict Clippy and full release suite pass 7+3+1 tests, zero
ignored. The same full real GTF now annotates successfully at 1/4 workers;
all 3440 source rows, nine core fields and nine masks agree, with byte-identical
worker TSVs. No real count, whole-workflow performance, new native-four/source,
public metadata or release qualification is inferred from this source freeze.
