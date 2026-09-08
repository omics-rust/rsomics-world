# Sketch product contract recheck

Status: signature scale recovery and mixed-scale collection comparison are
repaired in source with four-native CI. Neither change is registry-delivered. The
separate
[kmer short-input repair and construction measurements](../01-foundations/kmer-consumer-review-2026-09-09.md)
remain open release gates.

## Scope and decision

An independent source review of sketch
`3802b1aaca54a95c14dbdeb6571aff4eb5ebafc3` found two Important product-local
correctness defects. They do not justify another foundation, public API or
product boundary. Signature types, profile validation and comparison policy
remain private modules inside `rsomics-sketch`.

The CLI still uses registry `rsomics-help 0.4.0` recursively and
`rsomics-common 0.11.1` for global JSON reporting and exit/error handling.
Review checked those exact locked sources, not only newer foundation heads.
No duplicate product-specific CLI layer was introduced.

## Signature scale recovery

At `scaled=93`, the forward threshold is `198352086814081216`. Reversing the
floating-point division gives `92.99999999999999`; truncation selects 92 and
the loader's exact forward check rejects its own generated signature. Plain
and compressed output both reach the same validation path. Inspect, compare
and search therefore inherit the rejection.

Diagnostic commit `63abafaef56098179ee9a42c4e01a53b4f604446` and CI
[34266897367](https://github.com/omics-rust/rsomics-sketch/actions/runs/34266897367)
reproduced that exact loader failure on all four native platforms. Its green
workflow explicitly required the regression to fail; it was not a repair.

Fix `db594726e92ff08fcee85ebc333600e44268ae59` recovers the scale from the
truncated inverse and checked adjacent candidates, requiring an exact forward
threshold match. It preserves serialized thresholds and rejects values outside
the existing integer-scaled profile. Rounding alone is insufficient:
`scaled=3583599928` has threshold `5147545608` and inverse
`3583599928.68072`, whose rounded candidate is incorrect.

Normal exact-head CI
[34267432784](https://github.com/omics-rust/rsomics-sketch/actions/runs/34267432784)
passed four-native tests and the pinned sourmash oracle, plus strict Clippy,
formatting, rustdoc, package reconstruction and construction benchmark smoke.
The regression runs unignored. Tests cover 1, 93, 99, 1000, the high-scale
counterexample and `u32::MAX`, plain/gzip reloads, threshold-boundary hashes,
abundances and invalid thresholds. Live construction cases at 93 and the
high value compare complete signature bytes with sourmash 4.9.4 and then reload
the product's metadata. Independent source review approved the bounded
neighbor recovery and confirmed that no threshold or public layout changed.

This deliberately corrects an upstream metadata edge case, not the file format.
The pinned sourmash
[inverse and deserializer](https://github.com/sourmash-bio/sourmash/blob/v4.9.4/src/core/src/sketch/minhash.rs#L28)
truncate the reported scale but retain the serialized threshold. Signatures
created at 92 and 93 can consequently report the same scale yet fail its
threshold compatibility check. The product README excludes exact metadata
and mixed-scale comparison equivalence where that upstream truncation occurs.

## Mixed-scale collection comparison

The previous `metrics` selected a separate maximum scale for every pair.
Sourmash's pinned
[collection comparison](https://github.com/sourmash-bio/sourmash/blob/v4.9.4/src/sourmash/commands.py#L161)
instead downsamples all inputs to one common maximum scale before scoring.

The regression uses A at scale 1 with hashes `[1, 9223372036854775809]`,
B at scale 1 with `[1]`, and C at scale 2 with `[1]`. The common scale-2
threshold discards A's larger hash, making every pair's Jaccard and containment
one. The old A/B pair instead reports scale 1 and Jaccard 0.5.

Diagnostic commit `a19e8170a97666d77b3d89d4b3754e311a4ee330`, CI
[34267814268](https://github.com/omics-rust/rsomics-sketch/actions/runs/34267814268),
reproduced the expected scale-1-versus-2 failure on all four native platforms.
The unit regression covers pair output and both matrix measures through the
public comparison entry point.

Fix `f430522bb3d3c6fd38e08af758dca11e5262f47b` validates collection
compatibility, computes the common maximum scale once, and uses it for both
output shapes. Search retains the existing query/target-local scale. Its new
regression checks the distinct scores, scales and resulting ranking.

Normal exact-head CI
[34268416605](https://github.com/omics-rust/rsomics-sketch/actions/runs/34268416605)
passed all four native targets plus the ordinary release checks. The comparison
regression runs unignored. A new pinned live oracle loads literal, independently
digested fixtures and compares complete Jaccard and containment matrices against
sourmash 4.9.4. The older asymmetric containment case still protects matrix
orientation. Independent review found no blocker in types, thresholds,
directional metrics, error propagation or transactional output. Construction
performance was not remeasured at this repaired product head.

## Retained evidence and remaining gates

Raw red/green CI job logs and exact-head metadata are collected under
`/Volumes/KIOXIA/Developments/tmp/sketch-scaled-review-20260909-x3pR1j/`.
A copy is preserved at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/sketch-contract-review-2026-09-09/`
and was recursively compared with scratch.
No local Rust compilation or product execution was performed: physical boot
APFS usage remains approximately 94.5%. All local project files and scratch
remain on external disks. Existing user changes and old measurements were not
removed or staged.

Before product publication, adopt the corrected registry kmer dependency,
remove its old-panic diagnostic, run full exact-head
four-native validation and complete the construction performance decision.
The current measurements pin the earlier product head `3802b1a`; they do not
silently become measurements of a later repaired product binary.
