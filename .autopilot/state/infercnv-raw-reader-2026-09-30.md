# inferCNV raw reader — resumed candidate

Spec: `docs/plans/2026-09-30-infercnv-raw-reader-design.md`.
The user explicitly resumed on 2026-09-30 at 16:35 CST. The earlier pause
receipt is historical and does not instruct the controller to stop again.
The goal tool still exposes the old `paused` state and has no agent-side resume
operation; continuation follows the user's explicit instruction, not a second
or completed goal.

## Storage and scope

Fresh APFS size/free: 245,107,195,904 / 29,228,732,416 bytes (about 88% used).
No local build or dependency download. Cargo/test execution remains on hosted
native runners, with externally stored controller source and evidence. Product
is unpublished at `3715e55b`; no CLI, release, ingestion-speed or full-CNV
claim follows from this reader candidate.

## Accepted test-first failure

World `47dc2475db4e1538e90a524767b1a1b391a0f205` passed exact-head control
run `36688482518`. Frozen `red-6` adds only the 21 input contract tests.
Actual Linux x86_64 run `36688501838` failed solely at debug compilation with
E0432: missing `CreationConfig`, `CreatedInput`, `InputPaths`, `create_input`.
This is an absent-feature compile failure, not an executed assertion failure.
Dependencies and resolved output paths passed; both before/after source checks
verified all 143 files. The original artifact's size/API digest/CRC and source
archive/manifest/checksum/README bytes were independently verified.

Original evidence is preserved at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-ingestion-2026-09-30/run-36688501838/`.
Artifact `11084363788` has SHA-256
`bb8e7e70652dc900621af9e4fb10ac2501b561248b095f9fe27c36fab6bd2366`;
original logs SHA-256 is
`b05c38b7b42dcedd1f598a1b09747eee31c70c4b47ddcfd426b0285c4d30ea28`.

## Current work

The bounded product implementer resumed after red acceptance. The controller
owns frozen source, dependency-lock resolution, Git and native CI. A separate
agent owns the three creation edge scripts; source-derived failure classifiers
and input-path provenance are being strengthened before actual package tests.
The controller's new gzip lock verifier has five targeted tests passing after
the missing-module red; remote lock resolution and workflow integration are
pending. Existing packages may not change: only flate2 1.1.9 and its Rust gzip
dependency nodes are allowed to be added.

Ruling: no local compile despite external Cargo paths — APFS remains above
the manual's 80% gate — hosted execution preserves the native-platform gates
at the cost of longer source snapshot and artifact round trips.

Ruling: keep ingestion policy internal to `rsomics-sc` — one actual consumer,
inferCNV-specific joins/filter/group semantics — no speculative foundation API
or revival of a deleted micro-crate.

The inherited VCF preflight and loose test data remain untouched and unstaged.
