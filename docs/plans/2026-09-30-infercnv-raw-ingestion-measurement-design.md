# Matched raw-ingestion measurement

Status: design only; no ingestion-speed or memory advantage is established.
This follows the accepted native strict-input reader, not a complete inferCNV
release. Samples/Ward evidence and implementation remain separate gates.

## Accepted implementation and inputs

Native owner: `rsomics-sc` at
`e62fc960aeeb9dc543298885213cda6a57196fe9`.
The accepted four-native reader run is `36696131908`; its creation values,
ordered identities and reference/observation maps were exact against the
bounded R witnesses. Production code must remain byte-identical while private
measurement support is added.

Use the full shipped canonical count input, both plain and its witnessed gzip
copy, with the same original gene-order and annotation files. The canonical
raw count matrix has 10338 genes and 184 cells; creation retains 9939 genes
and 184 cells. Both implementations must open the same files and use the
same ordered references, excluded chromosomes, annotation join and inclusive
depth range `[1, Inf]`. No in-memory matrix or prepared RDS replaces raw input.
Plain/gzip file bytes, decoded-byte equivalence and all auxiliary inputs must
be verified before measurement.

Canonical count-file generation is outside this gate. The native reader does
not yet support the original shipped implicit header, arbitrary R decimal
parsing or R's discarded first `V1` annotation row. Do not hide these exclusions
behind a purported original-input benchmark.

## Measured operations

Rust measures the actual public `cnv::create_input(InputPaths, CreationConfig)`.
R measures the actual installed `infercnv::CreateInfercnvObject` with the same
file paths and creation settings. Installed infercnv/source/runtime pins are
those of the accepted oracle. R namespace/source identity must be checked
before invocation; no parser or factory reimplementation is permitted.

The timed region begins immediately before the factory call and ends
immediately after it returns checked state. It includes opening, decoding,
record parsing, gene reduction/order, annotation joins, depth filtering,
ordered group construction and the production result allocation. It excludes
package startup, oracle loading, configuration construction, correctness
comparison, result serialization and artifact hashing. Production errors exit
non-zero and cannot become a timing sample.

The actual R factory also computes its internal `counts_md5` and validates
the S4 object before returning (`R/inferCNV.R:320–335`). These intrinsic factory
operations stay inside the timer; the artifact-hashing exclusion does not
remove them. Native creation does not promise the R resumability/cache options.
State parity here concerns count values, identities and ordered groups, not
the complete physical representation or every S4 option.

Measure each trial in a fresh process on the same native Linux x86_64 host.
Perform one whole factory warm-up, discard its result, and complete language-
appropriate reclamation before measuring once. Record that this is a warm
filesystem/allocation regime, not a controlled cold-cache measurement.
Validate and export the warm result before discarding it; release any loaded
validation fixture before measuring baseline RSS. Record those steps as
preparation, including any retained allocator effects.
Do not flush host caches or require privileged operations. Record actual
thread environment, namespace versions/library paths, BLAS and observed
process threads; requested limits are not proof of OS thread count.

For each plain/gzip case, exclude one warm-up pair and preserve seven measured
pairs with alternating Rust/R order. Never overlap either implementation or
reuse one implementation's prepared result as the other's input. Each sample
must carry its implementation, input case, pair index, order and warm-up flag.
The per-process factory warm-up and excluded pilot pair are distinct: two
cases times eight pairs times two implementations gives 32 fresh processes,
64 factory results to validate, and 28 accepted measured-region samples.

## Correctness and metrics

Validate every warm-up and measured result against the accepted step-1 state,
including exact f64 values, genes/cells and ordered reference/observation maps.
Inspect exports independently outside the timer. Verify raw input hashes
before and after all trials and preserve failures unchanged. A partially
valid set is not accepted by dropping failed or inconvenient samples.

Record factory-region wall, self user/system and child user/system time,
baseline RSS before the call, and retained-result RSS immediately afterward.
The returned object must remain alive for that sample; record RSS before any
oracle validation, export allocation or post-call reclamation. Preserve actual
R storage types/classes/options and the native f64 representation. R's
`expr.data` and `count.data` may share storage; two slots are not proof of two
independent matrix allocations.
Unexpected child CPU work fails the region contract. Record whole-process
wall/user/system and maximum RSS separately with an external process monitor.
Whole-process metrics include warm-up and post-call validation/export; they
must not be mislabeled as factory-only peaks. Baseline and returned-state RSS
also are not a transient peak bound. The native numeric-capacity budget does
not bound process RSS, metadata or decoder memory.

Record process I/O counters where available, their exact meanings and input
compressed/decoded byte counts. Warm-cache physical reads can be zero; do not
interpret that as zero input consumption or a portable I/O reduction. Preserve
raw timing distributions, process metrics, machine/runtime metadata, flags,
source snapshot/archive/lock hashes and exact executable invocation.
Factory-I/O claims require before/after deltas rather than lifetime counters;
record instrumentation reads or label the counters as whole-process only.

## Implementation and acceptance sequence

1. Add product-private measurement support and contract tests in the owning
   repository. Reuse private CPU/RSS/parsing support where its contracts match;
   do not alter the existing prepared-input benchmark or promote a foundation.
2. Freeze and independently review source, numerical hot paths and measurement
   boundaries. Run the complete ordinary/measurement suite, formatting and
   strict Clippy through four-native CI, verifying accepted production bytes
   are unchanged. The Mac APFS gate still forbids local builds.
3. Add an actual-package R trial and a paired driver with regression tests for
   case/ordering, missing or invalid counters, errors, omitted samples and
   mixed provenance. Reuse the existing external-artifact verification model
   without weakening its success, digest, CRC or path checks.
4. Extend the pinned installed-package oracle with a distinct selector for
   the accepted ingestion measurement snapshot. Do not substitute it for the
   existing prepared-input snapshot or change the meaning of older receipts.
   Verify exact-head controller CI before dispatch.
5. Verify original artifacts and every sample independently. Require strict
   region-wall advantage separately for plain and gzip: every one of the
   seven measured Rust samples must be faster than every measured R sample
   for the same case. Report medians, overlap and dispersion, not only one
   ratio. This gate chooses wall time before observing results; RSS/I/O are
   descriptive, not an opportunistic fallback or a claimed factory peak.
   Equal performance alone is not a replacement gate pass.

If this gate fails, preserve its measurements and improve the reader through
the observed bottleneck. Continue other unblocked inferCNV work without
publishing an unmeasured claim. Even a successful result establishes only the
canonical shipped warm-ingestion cases, not representative cohorts, original
header support, downstream clustering or whole-workflow performance.

The existing prepared-input driver is not the creation validator: it compares
step-14 values with a tolerance, omits group maps and does not independently
retain each warm output. Reuse its checked process-monitor/counter mechanics
only after review; implement the exact step-1/map and complete-sample gates
with mutation regressions.
