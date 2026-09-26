# External shipped-example conformance candidate: green-5

This snapshot adds the explicit `external-infercnv-oracle` test feature to the
accepted synthetic native core and the four-native-verified test reader.
It requires `RSOMICS_INFER_CNV_ORACLE_ROOT` for its external test, compares
subset/full across all ten stages at the unchanged numerical criterion, and
checks returned state and immutable prepared input. The fixed oracle is run
`36221554790`, selected separately by its committed receipt in candidate CI.

Compared with green-4, only Cargo.toml changes and tests/README.md plus
tests/cnv_external_oracle.rs are added. Production code, dependency lock and
all original fixture bytes are unchanged. Product source review is clean.

Hosted missing-configuration failure, four-native debug/release conformance,
formatting and strict Clippy are pending for this snapshot. This is not a
performance, downstream-workflow, CLI or publication claim.

`files.sha256` identifies source bytes; `checksums.sha256` covers this receipt,
the manifest and archive. Preserve all earlier snapshots and evidence.
