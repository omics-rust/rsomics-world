# `rsomics-vcf concat` candidate audit

Date: 2026-08-20

Status: retain and repair; do not commit, advertise, benchmark as current, or
publish yet.

## Candidate evidence

The dirty `rsomics-vcf` worktree is based on committed revision
`682942cfa69768dc3a127a8544f2f07213b704ea`. The candidate contains the
planned command, dispatcher, header, ordered and overlap stream, ligation,
naive BGZF, CLI, compatibility, and benchmark files. The concat-specific
surface is 4,872 lines across production code, tests, and the benchmark
harness. It includes 30 process-level CLI tests, five ignored bcftools 1.24
oracle groups, and four module tests.

This is implementation work rather than a placeholder. The typed header
preflight, four output encodings, BCF dictionary rebuilding, transactional
named output, retained ordinary stdin reader, cross-input duplicate scope,
fail-loud ordering checks, typed PQ/PS updates, and structural BGZF checks all
follow the accepted product design closely. The worktree remains uncommitted
so it cannot accidentally change the exact 0.6.0 publication candidate.

## Required repairs

1. Index preflight and query do not resolve the same file when both indexes
   exist. `require_fresh` currently checks CSI before TBI. Noodles 0.90 loads
   TBI before CSI for VCF and loads only CSI for BCF. The command can therefore
   validate one index and query another, or reject a stale CSI even though the
   VCF reader would use a valid TBI. Resolution must be format-aware and one
   chosen path must be used for both validation and query. Tests must cover
   valid, stale, and corrupt dual-index combinations for VCF and the BCF CSI
   restriction.

2. Indexed overlap mode starts one OS thread per input and does not apply an
   input-count or worker bound. The one-record channels bound queued records
   but not threads, stacks, descriptors, or indexed readers. Replace this with
   a bounded ingestion design, or define and enforce a measured resource cap
   whose failure contract is explicit. A many-input scaling fixture must
   measure threads, descriptors, RSS, ordering, cancellation, and early error
   propagation.

3. Naive mode performs a complete BGZF inflate and typed record pass during
   inspection and then reads every input again for raw copying. This satisfies
   the stronger corruption contract but has no evidence that the operation
   still serves the upstream mode's performance purpose. Remove redundant
   passes where possible and keep the full integrity contract. Do not weaken
   CRC, size, EOF, trailing-byte, schema, or BCF dictionary checks merely to
   manufacture a speed claim.

4. The oracle file proves useful normal cases, all duplicate spellings, one
   indexed region, principal ligation policies, and a partial encoding matrix.
   It does not yet satisfy the dossier's declared matrix for file lists,
   `--compact-PS`, every region-overlap rule, all input/output encoding
   combinations, three-or-more-input ties, genotype removal across VCF and
   BCF, or each expected divergence. Add exact fixtures and compare typed
   bodies, not only record counts or selected substrings.

5. The benchmark covers ordered VCF, ordered BCF, one two-input indexed
   overlap, naive VCF, and naive BCF. It omits the required many-sample
   ligation workload and does not expose the indexed-input thread scaling
   risk. The current decision rule also treats each workload independently;
   the final record must state the supported per-mode claim and must not turn
   the validated naive path into an unmeasured speed claim.

6. The design permits promotion of format-neutral BGZF framing to
   `rsomics-seqio` only after the VCF 0.6.0 publication gate and after BAM and
   VCF consumer tests plus a no-regression benchmark. Version 0.6.0 is still
   unpublished because the registry token is revoked. Keep both private
   implementations until that consumer-driven extraction gate is genuinely
   available.

## Verification state

No local build, test, package, or benchmark was run for this candidate during
the audit. The Mac boot disk is at 82%, above the mandatory 80% stop threshold.
The authorized Linux x86_64 host has its root filesystem at 98% and only
bcftools 1.13, so it was rejected as a build oracle as well. The control-plane
review is static evidence only.

After the storage gate is cleared, repair in the order above, then run format,
strict Clippy, debug and release tests, the complete pinned bcftools 1.24
oracle, benchmark smoke, formal performance, package verification, and
exact-head four-native-platform CI. Only then may the candidate become a
single coherent 0.7.0 feature commit and public command.
