# FASTQ quality-window frontier

Owner-read-only frontier work after Table 0.2 GitHub delivery. Owner remains clean at
`89d4f534f90eca7ce84cf0929474047bd1139d38`, version0.1.1. No FASTQ source,
dependency, public API, package or release was changed in this inventory.

## Existing consumer and source contract

`run` and `trim` already share the ordered SE/PE engine, explicit quality
encoding, checked whole-read filters and coordinated outputs. Quality-window
trimming belongs in that product's private transform layer, not a new crate.
Historical `rsomics-fastq-quality` at
`711e32cac0f45072d038171f42d6d4a831112cff` remains evidence-only: its mixed
Trimmomatic/fastp semantics cannot define the new workflow.

Current owning CI and PROVENANCE both pin fastp1.3.6 to
`23d6211d4f05d61f561899f1b7702435a4b5d408`. The prior inventory observed
ambient fastp1.4.0; it is not the declared oracle. The exact local executable is
now established below. Do not silently run ambient compatibility tests or
report upstream-only witnesses as a native implementation comparison.

Retrieved official commit-addressed source to external scratch:
`/Volumes/KIOXIA/Developments/tmp/fastq-quality-source-20261103.98ZifXEs`.

| File | SHA-256 |
|---|---|
| `filter.cpp` | `a17fc2d9469b04642cd01142670b38f136dd2c1bbd5ec576703b6646ff2fc0c5` |
| `options.h` | `dd9941894b58f4a90095127e03a4d936b961266634334a0431f0b2b668eaab74` |
| `main.cpp` | `ae4d0a53a39d619d73b238fd23c51b890ce04573bb966f7b7f501ea1db7f746b` |

Origin for each: `https://raw.githubusercontent.com/OpenGene/fastp/23d6211d4f05d61f561899f1b7702435a4b5d408/src/FILE`.
Guard checked actual external devices, cache/scratch resolution and headroom
before downloads. Three source files and the exact full source archive were
downloaded; no system tool or dependency was installed.

## Actual source contract, not yet native compatibility acceptance

Read `filter.cpp:68-205`, `options.h:132-170`, relevant option declarations and
`main.cpp:267-322` and the native `process_single`/`process_pair` call sites.

- One `trimAndCut` handles fixed clipping and front/tail/right quality modes;
  windows default to4 and mean quality20. Shared option values are inherited
  unless mode-specific values are supplied.
- The window sum uses integer ASCII qualities; the rolling threshold includes
  the Phred33 offset. Equality passes. Native Phred64 handling must follow the
  existing explicit encoding contract, not saturating subtraction/detection.
- Right mode scans with a strict end condition, preserves the initial
  individually passing bases inside the first failing window, and disables
  backward tail cutting when enabled. A textbook average-window implementation
  that cuts at the window start would disagree.
- Short/equal-window reads and final-window handling have source-specific
  boundaries. Capture real retained sequences/qualities, not merely an
  all-discarded golden, before deciding implementation details.
- The native engine currently combines fixed and poly trimming before whole-
  read filtering. Upstream SE/PE pipeline order, Phred conversion, null-read
  classification and metric behavior must be traced before adding quality
  cutting; inserting it after the existing combined trim is not justified yet.

## Exact local oracle established

Official codeload archive SHA-256
`f48bb574f05800d63b0fc3958bea3faf6b2db6e6f49f0c15a210029e34b88550`.
All79 members were checked for safe regular/directory paths and bounded size
before external extraction. The three source files above match archive bodies.
Read the complete Makefile before building; only relative `obj`/`fastp` outputs
are used. `make -j2` succeeds with existing read-only Homebrew highway1.4.0,
isa-l2.32.1 and libdeflate1.26. No `make install` or brew install was run.
Project outputs, compiler/module scratch and build logs stay on the external
disk; this is not a claim of zero OS/application footprint.

Actual `fastp --version` is `fastp 1.3.6`; executable:
`fastp-23d6211d4f05d61f561899f1b7702435a4b5d408/fastp` under the packet,
SHA-256 `7fb979ed4d571be495a0f5cc210122cc98a4e8f1bb1224d2aa5120947f34401d`.

`witness.py` executes five actual SE workflows: front, tail, right, front+right,
front+tail. The joint input includes retained good reads, low-quality front,
middle and tail, and a read exactly as long as the window. Full output FASTQ,
original stdout/stderr, JSON/HTML reports and argv are preserved. These are
retained-sequence/quality witnesses, not all-discarded fixtures. Right keeps
the good prefix inside its first low window; the equal-window read disappears
even with ordinary quality/length filtering disabled. Record this contract
before coding rather than assuming a textbook loop.

`WITNESS.json` SHA-256
`5d2dad3063ae2c78394f305a341afc577ee9a997255517ef0088845269e99942`;
tracked copy `.autopilot/oracles/fastq-quality-upstream-2026-11-03.json`.
No native FASTQ comparison or performance claim is made yet. Also read actual
SE/PE call sites: fixed+quality cutting precedes poly-G, then adapter handling,
then poly-X and whole-read filtering. Trace Phred reader conversion and null-read
classification as the next implementation prerequisite.

## Next implementation unit

Establish the exact oracle and the full quality-window workflow, including
front/tail/right selection and their interactions, inside the existing
`run`/`trim` pipeline. Do not turn an individual flag into a product. Trace
upstream caller order and parameter validation, then write one joint SE/PE
retained-read differential and atomic-failure workflow. Reuse the current
ordered local Rayon engine and report model; do not speculate a new Layer A
quality API without the second concrete product consumer.

Measure the integrated quality workflow on nontrivial reads with complete
sequence/quality/order equivalence, CPU/wall/RSS provenance, and representative
serial/parallel execution. Old fixed/poly performance does not qualify this
new hot path. One whole-feature review and the existing four-native gate apply.
Nothing in this inventory exposes or advertises unfinished behavior.
