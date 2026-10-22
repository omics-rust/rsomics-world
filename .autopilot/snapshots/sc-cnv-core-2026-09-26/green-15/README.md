# Private native Ward.D2 tree-kernel candidate

Baseline private product dbb2c4f959c2e3156da2ab9b9c4a7afb63c7b8cf.
Compared with red-9, this adds src/cnv/samples/ward.rs and THIRD_PARTY_NOTICES.
The six-leaf test parses the two literal captured heights rather than a
long numeric source literal; the asserted binary64 values are unchanged.
Cargo.toml/Cargo.lock, all existing production files and all existing tests
remain unchanged. No public module/API, dependency, feature, binary or release.

Default private integration tests compile the kernel directly and compare
all152 captured real trees/107331 distances: exact merges/leaf order and
bitwise binary64 heights. Additional tests cover the complete requested
workspace boundary, invalid input/arithmetic, zero/equal ties, chain restart,
immutable input, dynamic range and rounded square-root ties.

Actual Linux x86_64 RED39767102778 reached the intended absent ward.rs error.
Fresh independent static implementation/spec review approves with no
Critical/Important finding. This snapshot is not yet native acceptance;
four-native debug/release, formatting, strict Clippy and source postchecks
remain required. No Euclidean/filter/group integration, step15 performance
or whole inferCNV conclusion follows. Original BSD notice is retained.

Plan: docs/plans/2026-10-22-infercnv-native-ward-plan.md.
