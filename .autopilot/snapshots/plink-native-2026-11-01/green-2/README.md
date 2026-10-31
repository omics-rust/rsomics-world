# Private PLINK final guard-only native source snapshot: green-2

Owner repository: omics-rust/rsomics-plink, clean main.
Owner commit: 1d199b541ea014bdff487b99ee6669873f533b0d
Owner Git tree: 4c3a89149dba750a0c3b64e0d9831b7148b9c5e4
Parent/baseline commit: a1c50116e52c71f8cd6bbb65ac94ddfb4418dd94
Tracked regular files: 302, all Git mode100644; exact actual working-tree bytes
were checked against every immutable-commit Git blob before freezing.

Only build.rs, README.md and MERGE_RECORD.md changed from the baseline. The
build change makes RSOMICS_GUARDED_TARGET_DIR validation opt-in for downstream
users; dependency, fixture and scientific implementation files are unchanged.
files.sha256 covers every tracked source body. The archive stores original
relative paths and Git executable modes, with no .git, ignored or untracked
files. Tar member timestamps use the owner commit time; gzip metadata is fixed.

This is a new actual source snapshot, not a rewrite of green-1 or a claim that
the final guard-only commit has passed native-four/same-native-source gates.
Those exact-snapshot workflows remain pending. It confers no new real-cohort
performance, publication or source/relink-distribution qualification.
Original Rust and external LGPL pgenlib/SIMDe terms remain distinct.
