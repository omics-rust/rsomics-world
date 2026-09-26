# inferCNV shipped-data correctness continuation

## Purpose

Extend the unpublished native pre-clustering core from the accepted synthetic
oracle to the actual example distributed with pinned infercnv 1.28.0. This
does not add a user CLI, raw-input compatibility claim, clustering, HMM,
denoising, plotting, public foundation, or release. Performance measurement
follows correctness; it is not established by this design.

Input/provenance and the prospective timing boundary are in
`../../.autopilot/state/infercnv-real-data-preflight-2026-09-26.md`.
The package source is commit `b421d9405c97a309b081ef86d455e976df93eae4`.
Use only its three matched oligodendroglioma count/annotation/coordinate files.
Preserve the warning that the coordinate file is abridged for this example.

Pin the source archive to SHA-256
`b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207`
independently of the generator's JSON. Pin each original shipped file too:

| File | SHA-256 |
|---|---|
| Compressed expression matrix | `7b7c618e6b03d589ea36979b074d0b83e127dcaa01e059ba38f73900e7609ba3` |
| Cell annotations | `345493e0686d75418427e9c4401f3f7bbb55ae3f61deee074ccee8882dc4dc63` |
| Example gene coordinates | `4ec63e049ea8299948fb730c2f60ca5a038a798b77cec3bb918fda572b2dc52e` |

The original gzip header contains 184 cell names without a leading `gene`
field; each data row contains a gene ID plus 184 values. Parse that declared
dialect, then write a canonical `gene`-headed TSV and record its derivation
and hash. Subset by gene ID joined to coordinates, never row position.

Canonical values are the pinned R runtime's interpretation of the original
decimal text, not an assumption that R and Python round every input identically.
The independently diagnosed five adjacent-representable-value conversions and
both reviewed canonical byte hashes are recorded in
`../../.autopilot/state/infercnv-shipped-parser-audit-2026-09-26.md`.
Pin those canonical hashes outside generated JSON. Audit equality or numerical
adjacency only at the original-to-canonical bridge; retain exact identities and
all post-canonical value checks. A changed canonical hash requires a new review,
not automatic acceptance of arbitrary one-ULP changes. No downstream numerical
tolerance is widened.

## Bounded cases

Generate independent R runs for two cases, retaining all 184 cells and the
same six annotations in each:

1. A chr1+chr19+chr21 subset chosen before `CreateInfercnvObject`.
2. The complete shipped matrix, excluding chrX/chrY/chrM in object creation.

Both use the two upstream example reference groups with bounds subtraction,
cutoff 1, minimum detection 3, pyramidinal window 101, clamp 3, no reference
regrouping, and one thread. Disable optional transformations and plots,
disable resume, and stop at stage 14. State every relevant creation/run
argument explicitly in the evidence. Generate each case from its own raw
input; do not slice a previously normalized full output.

The preflight's 1,775/1,487 subset and 9,939/8,508 full stage-1/stage-2 gene
counts are predictions, not accepted oracle values. Assert them during the R
run and inspect any disagreement before freezing new fixtures. Check exact
cell identities and group sizes, finite nonnegative prepared inputs, positive
post-filter depths, and the short chr21 block. Fractional counts are valid;
this fixture is not an integer-UMI or biological ground-truth dataset.
Independently derive and verify stage-1/2 gene identities, not just counts.
Require the subset's chromosome set to equal chr1/chr19/chr21, all 184
annotated cells to remain, and both declared reference groups to be nonempty.
The independent validator must match every stage-1 count exactly to the
canonical raw matrix by gene and cell ID, then require stage 2 to contain
exactly the unchanged values of the retained genes. Identity and downstream
arithmetic checks alone must not accept a self-consistent wrong starting matrix.

## Oracle harness and artifacts

Reuse the existing pinned R/runtime installation workflow. Add an explicit
manual dataset choice, defaulting to the existing synthetic mode; keep its
established validator and semantics intact. Extract expression/identity export
helpers into one sourced R helper used by both generators, with no change to
their output contract. Do not duplicate an R installation workflow or copy
the synthetic fixture's scenario-specific checks into the real-data path.
In shipped-data mode, run and validate the old synthetic generator too, so
shared-exporter changes get an actual upstream regression in the same runtime.

The shipped-data generator uses the installed package's actual run and
filename builder, preserving the ten original RDS checkpoints per case plus
17-digit TSV expression, gene/coordinate and cell/group exports. Its separate
schema records source byte identity, package/runtime metadata, input hashes,
complete parameters, actual dimensions, and all checkpoint paths. A dedicated
validator checks schema/paths/hashes, identity/order, grouping, dimensions,
finite values and stage arithmetic relationships. It must reject missing,
extra, malformed, mismatched, or traversal-addressed required artifacts.
Negative checker tests precede implementation. Accepted synthetic validation
continues to run unchanged after the shared exporter extraction.
Use streamed/pairwise checks or compact numeric arrays; do not retain all ten
full matrices as Python float objects simultaneously.

Keep large outputs and original ZIPs under external fixtures, not product or
control-plane Git. Retain exact-head job metadata, archive hashes, sorted
file manifests, logs, original checkpoints and extraction verification. If an
oracle step fails, preserve the real failure instead of substituting output.

## Native conformance

Extend the test-only fixture reader to accept an explicit fixture root without
changing production parsers or public core policy. The existing small checked-in
synthetic fixtures continue to run by default. A separate explicit external
oracle test must error when its configured evidence is absent, not silently
skip or report compatibility. Keep the large cases outside the repository.

After accepting the R oracle, commit a small receipt pinning the exact run,
head, artifact ID/name, ZIP digest and bundle-manifest digest. Extend candidate
CI with an explicit receipt selector, not a latest-artifact lookup. It fetches
that exact immutable artifact into runner-temp with read-only Actions access,
checks run identity and both digests before extraction/fixture use, verifies
safe archive paths and the bundle manifest, and sets the external test's
input path explicitly. Preserve acquisition/checker logs alongside the Rust
source/test evidence. A missing or mismatched oracle fails the job.

For each case, construct prepared counts from stage 1 and run the native
pipeline once, observing all ten stages. Compare genes, coordinates, cell IDs,
groups and dimensions exactly, including the returned final state. Start
with the existing numerical criterion `abs(delta) <= 1e-12 + 1e-12*abs(expected)`
as a prospective check, not a universal accuracy guarantee. Report per-stage
maximum absolute and scaled errors. Investigate discrepancies; do not relax
the gate to obtain a pass. Add malformed-reader tests where the generalized
test-only loader introduces new inputs.

Validate the frozen product source and exact large-oracle hashes on Linux and
macOS, x86_64 and aarch64, with locked dependencies and debug/release tests.
Formatting, strict Clippy and all existing synthetic/contract/kernel tests
remain required. No local Cargo execution while boot APFS exceeds 80%.

## Acceptance and next work

Accept only after independent harness review, real R execution, raw artifact
inspection, native four-platform results, and a fresh implementation review.
Commit the verified product changes locally on main and the control-plane
receipts on main, preserving all earlier red/green snapshots. Public source
or registry publication still waits for a coherent workflow release.

Then implement the separately matched performance runner described by the
preflight, using the accepted full example and no timed exports. A strict
measured advantage on this example is useful evidence but does not replace a
large representative fixture or validate downstream CNV calling. Raw-state
integration and the source-audited stage-15+/HMM/denoise workflows remain open.
