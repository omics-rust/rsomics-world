# PLINK complete output blocks

Status: integrated as9cc22cac4f71d7e9b249b1f4262d8d4b11869d7e;
new native/performance/publication qualification remains pending.
Plan: `docs/plans/2026-11-03-plink-complete-output-blocks.md`.
Owner base:499693cf5629420a0a5171e700afdda264e3aa0d.

External implementation packet:
`/Volumes/KIOXIA/Developments/tmp/plink-complete-blocks-20261103.itHdXe`.
Actual pre-change concurrency RED is retained in red-concurrency.log:
every_lane_completes_its_job_before_the_first_output_is_drained exits101
with a calculation job blocked before completion. A generous30s watchdog
detects stalled progress, not a throughput threshold. Consumer error releases
the old workers; no process is intentionally left hanging. Implementation and
whole-suite verification remain pending.

The new over-cap UTF-8 identifier fixture exposes a separate existing VCF
reader defect before LD execution: locked noodles-vcf0.90.0 record read_field validates
each fill_buf chunk as UTF-8 independently, rejecting a valid codepoint split
across chunks. Actual failure is retained in debug-lib.log. Root rules that
the output-budget regression use ASCII to isolate this wave; existing small
UTF-8 tests remain. This is not permission to ignore the valid-input defect:
a separate VCF reader correction and the retained reproducer must precede
publication. Do not silently bundle a dependency change into scheduling.
Root verifies Cargo.lock407 and the actual external registry source; the
initial worker's0.81.0 version attribution was wrong and is not adopted.

This follows source-confirmed job-internal blocking, not measured causal
attribution. The chosen private protocol transfers one completed payload per
lane and waits for writer return before reusing it. LD output jobs have an
admitted4MiB cap, a derived line bound and serial fallback for over-cap lines;
make migrates its existing completed blocks. Prune is independent/unchanged.
No larger queue, new public component, disk spool or numerical change.

One bounded implementer owns the complete source change and its tests/logs.
Root owns the complete review, independent verification and commit. New source
requires a new immutable/native gate; ongoing metadata green11 run41478980090
stays bound to499693cf and must not be represented as qualifying this work.
All external-device guards, preservation and exact-head gates remain required.

The full30-product goal remains active. This is a performance-relevant product
implementation wave, not a redefinition of portfolio completion.

## Source integration

Frozen four-file diff SHA256
4dbb0ff9bd9897d4ac9f5ed1d047f13992c97d6b636593fe21d16cae3ac11e28.
Root reads the complete delta/rulings, independently matches all4 source
hashes and runs the guarded full debug suite1093: exit0,12 library and3
integration tests, zero failures/ignored. Worker final debug/release each have
the same populations; strict all-target/all-feature Clippy/fmt exit0. An actual
long-ID table exceeds8MiB with serial/parallel identity; a single over-cap
ASCII ID returns unchanged output through serial fallback.

One fresh whole-feature review reads complete source and callers and finds
no actionable Critical/Important/Minor issue. Root rules on its declined
items: native-four/source qualification, representative throughput/RSS and
4MiB optimality, and publication remain future gates; the VCF UTF-8 defect
remains a separate publication hold. Reviewer deliberately did not rerun tools;
root's actual debug run and retained final tool logs supply execution evidence.
No claim is made that cache locality or all four-worker regressions are fixed.
