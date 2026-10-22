# Separate reference-only samples oracle

Builder check approved. Native distance was independently accepted at
5346f213; this separate reference-only oracle is now the active edge gate.
It does not change the distance candidate, source verifiers or fixtures.

## Decision and scope

The next group component must not invent zero-observation behavior. Existing
CreatedInput permits references-only input, but the accepted36-case witness
does not contain it. Capture the real installed R function's grouped and
pooled results using one tiny genuine all-reference object, without modifying
the canonical exporter, witness, validators, receipts or152-tree expectations.
This is evidence acquisition, not a public API decision or workflow claim.

Reuse the existing default-false `samples_clustering` workflow option. Add an
adjacent separate output step after canonical witness validation. No new
flag/workflow/source dependency/Rust feature is required; the full existing
oracle run is intentionally reused despite its higher run cost. Paired
ingestion selectors and literal code pins remain byte-identical.

## Contract

New test-only R entrypoint takes `SOURCE_ROOT NEW_OUTPUT_DIR`; source the
unchanged infercnv_oracle_io.R. Verify R4.6.1, installed infercnv1.28.0, source
commit/archive SHA from existing authenticated environment and exact installed
formals/body for define_signif_tumor_subclusters and its single-group helper.
Retain actual namespace distance/tree binding identities and source bytes.
Use explicit none/Ward.D2/no per-chromosome/no-DE/filter0, C locale, seed and
one requested thread. Do not source the canonical exporter: its main is
unconditional. Small test-only object/capture patterns may be reused directly.

Construct a real package object from a finite small matrix, ordinary reference
annotations for all three cells and no observation annotations; do not force
the observation slot empty by mutation. Assert the actual object has no
observations before entering either route, and retain its original class,
expression/count/gene/cell/maps and pre-call RDS. Use this same immutable input
for grouped and pooled calls. Record actual warnings/errors, not anticipated
success. Distinguish missing key, NULL, empty list and empty index vectors in
an explicit typed shape summary; preserve raw call/returned RDS so JSON
normalization cannot erase these distinctions. On successful calls verify
original state/maps unchanged. On error preserve its classes/message/call and
original input rather than deriving a fake output. Numeric tree values use
literal17-digit strings and actual labels/index order.

Use matrix(seq_len(12L)/8+1,4L,3L), counts20, gene1..4/cell1..3, one reference
named ref and all three annotations ref, matching the ordinary canonical
probe factory options. Only expression is assigned after factory creation;
never replace its group slots. A construction failure is a failed launch,
not two fabricated function-call outcomes.

Proposed record schema is version1 with exact top-level keys schema_version,
package, runtime, input, arguments, routes and files. Routes is an ordered
two-element list with identities grouped and pooled; each stores status,
warnings, error(nullable), log path, raw RDS path, result-shape(nullable) and
unchanged-state(nullable). Error records classes/message/call; success records
require unchanged-state=true. Raw RDS preserves the actual call wrapper,
including the returned object or original condition. Input binds the original
RDS and four exported state/map files. Runtime binds R, package versions,
loaded paths, source/function/exporter files, locale, seed and threads.

The typed result-shape recursively summarizes only the returned object's
tumor_subclusters and the second return element, retaining typeof, classes,
length and names (nullable). List items remain ordered; atomic values remain
typed literal strings, with doubles17-digit, explicit NA/NaN/Inf tokens;
language nodes retain deparsed text. Thus absent keys differ from present
NULL, and NULL differs from empty list/integer vectors. The original RDS,
not this diagnostic summary, remains authoritative for unmodeled R objects.
Validator shape tests must cover these distinctions, not normalize them.

Write a versioned edge-only JSON record and complete sorted relative-file
SHA256 inventory, including runtime/source/function/exporter/arguments/log
bindings. Output directory must be new, paths regular and safe, no append or
mutation of accepted inputs. A Python stdlib validator authenticates all
listed files/runtime/source pin/required two route identities and typed
observed results. It must not assert an unknown pooled outcome before the
first real run. Later acceptance pins the original observed bytes and exact
status/shape for integration tests. Schema validation alone is not acceptance.

## Execution plan

### Task 1: Add separate test-first edge capture

Completed implementation at d7d808039e213840a63df0673074403e0940ca93.
Fresh independent spec/source/quality review approved with no important
finding. Coherent root controller run421tests64.227s passes, two existing
skips, after syncing the exact embedded workflow mirror; no test loosened.
Actual R execution and edge outcome acceptance remain Task2, not established.

Files: scripts/infercnv_samples_reference_edges.R,
scripts/validate_infercnv_samples_reference_edges.py,
scripts/test_infercnv_samples_reference_edges.py and one adjacent step in
.github/workflows/infercnv-oracle.yml. No other implementation files.

Interfaces: Rscript --vanilla scripts/infercnv_samples_reference_edges.R
SOURCE_ROOT NEW_OUTPUT_DIR; Python validator validate(root: Path) -> dict and
CLI positional ROOT. Generated record witness.json uses the contract above.
The witness itself is not part of its nonrecursive files inventory.

- [x] Add tiny controller fixtures for hash/path/count/type/source/runtime/
  shape omissions, both success/error forms, typed NULL/empty distinctions,
  no omitted route, exclusive output and real workflow opt-in/order/Bash
  routing. Observe missing validator/behavior RED before implementation.
- [x] Implement small R capture and Python validator. Fix exact emitted key
  names consistently in tests; record them in the task report. Keep original
  files/pins untouched. Safely source unchanged oracle_io only; no eval of the
  canonical exporter and no replacement of actual reference/observation maps.
- [x] Focused then full controller tests, architecture/whitespace checks,
  self-review and exact owned-file commit. No local R/build/install. Fresh
  independent R/source/validator/workflow review before push/admission.

### Task 2: Run and admit actual separate evidence

Full launch39781233583 and intended early locale RED39785979929 are retained
as failed evidence, not edge outcomes. Repair3b65144e resets the three
previously omitted locale categories without changing the regression or
strict C predicate. Fresh scoped review and424 controller tests pass; actual
native GREEN and both edge outcomes remain pending. Detailed provenance:
`.autopilot/state/infercnv-reference-edge-locale-2026-10-22.md`.

- [ ] Require exact-head Control plane CI; dispatch existing shipped samples
  oracle option with other optional contracts/snapshots off. No local R/build.
- [ ] Authenticate original run/jobs/ZIP/log/source/runtime/record bytes and
  raw RDS presence; inspect complete actual outcomes independently. Preserve
  failed launches. Schema-validator success is not oracle acceptance.
- [ ] Pin observed status/shape and record reference-only contract without
  generalizing to filtering, whole step15 or performance. Only then implement
  groups against both original36 cases and this separate edge. Continue with
  unblocked product-local work if external setup is unavailable.

The user already decided pooled reference-name collision all_observations
must fail early as a known incompatibility. This edge does not reopen or
silently broaden that decision. Group implementation otherwise remains
product-private, based on ordered existing CellGroup maps and borrowed state.

## Primary evidence

Original source b421d9405c97a309b081ef86d455e976df93eae4,
R/inferCNV_tumor_subclusters.R34-40,75-81,189-193,229-264.
Existing canonical exporter probe_object411-430, probe_case433-447,
installed-source checks302-335, runtime450-486; unconditional main587.
Existing workflow samples option19-22, exact source/runtime348-408,
canonical capture729-747, unconditional upload861-869. Source audit details:
`.autopilot/state/infercnv-samples-group-contract-2026-10-22.md`.
