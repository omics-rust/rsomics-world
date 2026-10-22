# Native Ward tree-kernel test-first snapshot

Private product baseline dbb2c4f959c2e3156da2ab9b9c4a7afb63c7b8cf.
Original149 green14 source files are unchanged except the two already-reviewed
README files. New files are tests/cnv_samples_ward.rs and the exact generated
ward/trees.tsv and its README. Cargo.toml/Cargo.lock are unchanged; no new
dependency, feature, public API, module declaration, binary or release.

The new integration test imports src/cnv/samples/ward.rs, intentionally absent
in this immutable RED snapshot. Actual native Linux x86_64 compilation must
fail at that missing module after source/metadata checks; another failure is
not the intended RED. All152 real captured trees are retained in the fixture.

This snapshot is not native clustering, performance or product acceptance.
Plan: docs/plans/2026-10-22-infercnv-native-ward-plan.md.
