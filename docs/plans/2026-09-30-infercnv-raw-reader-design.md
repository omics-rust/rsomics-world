# inferCNV raw TSV ingestion

## Scope

Add a product-local reader in the unpublished `rsomics-sc` repository. It
transforms counts, genomic positions and cell annotations into `PreparedCounts`
and ordered reference/observation groups. It does not add a CLI, public
foundation, RDS import, sampling, clustering, HMM or a release claim.

The source contract is installed infercnv 1.28.0, commit
`b421d9405c97a309b081ef86d455e976df93eae4`, `CreateInfercnvObject` and
`.order_reduce` in `R/inferCNV.R`. The inspected creation witness is Actions
run `36685366497`; its receipt binds original archives and source checkpoints.
Native comparison must preserve the existing downstream tolerance rather than
adjusting it to accommodate a reader.

## Data and ordering

Counts are strict UTF-8 TSV with explicit gene-column header and unique,
nonempty cell/gene IDs. CRLF and a final unterminated record are supported.
The three input files may be plain or gzip; decoding must reach EOF and report
truncation/checksum errors. Numeric fields accept surrounding ASCII spaces:
the accepted R `formatC` canonical files contain fixed-width leading padding.
Identity fields are never trimmed. Other whitespace, comment and quoting
transformations from R's general `read.table` dialect are not inferred. An implicit count
header is added only after a separate real-package probe.

Positions have four headerless fields: gene ID, chromosome, unsigned start,
unsigned end. Annotations have two headerless fields: cell ID and group name.
All records are validated, even if later excluded. Duplicate IDs, empty
fields, negative/nonfinite counts, invalid coordinates, malformed rows and
unknown annotation cells fail with path and record context.

Annotation IDs are literal: native ingestion retains a valid first cell named
`V1` and errors if it is unknown. It deliberately does not reproduce upstream's
unconditional first-`V1` annotation deletion. This strict headerless dialect
must be explicit rather than silently treating a real cell as a header.

Remove configured chromosome labels and `(0,0)` positions, then derive
chromosome ranks from the entire remaining position table, including genes
absent from counts. Join in expression-row order and stably sort by chromosome
rank, start and end without a gene-name tie-breaker. Validate annotation IDs
against the original count header before any cell filtering. Compute cell
totals over the reduced genes, apply inclusive limits with minimum clamped to
one, then restrict to annotated cells in original matrix-column order.

Reference groups preserve the requested order; every requested group must
retain cells. Observation groups use explicit bytewise lexical ordering, not
machine locale. Group indices are zero-based into the final matrix. Values
are assembled in the existing column-major layout and checked by
`PreparedCounts::new` once. Group objects expose read-only accessors; ownership
of prepared counts can transfer without a matrix clone.

## Numerical boundary

Use ordinary deterministic Rust `f64` parsing. Exact arbitrary fractional
lexical parity with R is excluded: the actual Linux R witness parses `0.076439`
to `0x1.391819d2391d6p-4`, one ULP above direct binary64 parsing. Do not insert
literal substitutions, universal ULP allowances, copy R's parser, or claim
cross-platform R-decimal equivalence. Canonical binary64-roundtrip input and
exactly representable integer-count cases remain exact comparison gates.

Cell totals use compensated summation in genomic order. Tests must distinguish
native deterministic summation from R's long-double accumulator. Integer total
thresholds within the exact binary64 range and focused exactly representable
fractional thresholds are real-package comparison gates; arbitrary fractional
threshold parity remains excluded unless independently demonstrated.

## Resource boundary

One `CreationConfig` controls count limits, excluded chromosomes, reference
names, numeric workspace bytes and maximum decoded record bytes. Avoid storing
unjoined numerical rows. The budget covers retained row-major values, totals,
and final column-major values at their overlapping peak. Check dimensions and
byte arithmetic before allocation; fallible reservation errors propagate.
The record limit bounds transient text, not total metadata memory. State this
distinction rather than claiming a whole-process memory cap.

## Verification and delivery

1. Add small tests for stable tied ordering, position-only chromosome rank,
   annotation validation before filtering, inclusive post-reduction limits,
   no-reference grouping, failures on discarded rows, gzip corruption and
   numeric/record-budget exhaustion.
2. Implement only the owning reader module and those tests. Resolve the gzip
   dependency lock remotely; do not build/download on the current Mac boot
   disk (APFS occupancy remains over 80%).
3. Test frozen source on native Linux/macOS x86_64 and aarch64, formatting and
   strict Clippy; retain test-first failure and exact source hashes.
4. Compare actual accepted synthetic and shipped canonical inputs, stage-1
   identities/maps and existing step-14 checkpoints. Obtain extra real-package
   probes for threshold/filter failures and implicit header behavior before
   advertising those capabilities.
5. Review API and memory/error paths independently. Commit only accepted
   product bytes; record native gates and numerical exclusions in the dossier.
   Measure ingestion separately before making throughput claims.
