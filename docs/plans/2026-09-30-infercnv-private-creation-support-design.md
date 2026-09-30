# Private exact creation support

## Boundary

This is the Rust comparison/export component of the
[raw-ingestion measurement design](2026-09-30-infercnv-raw-ingestion-measurement-design.md).
It produces working contract-tested support, not a timed factory or speed
result. The accepted reader at product `e62fc960aeeb9dc543298885213cda6a57196fe9`
and all existing source, fixtures, prepared benchmark/support and lock bytes
remain unchanged. Add only one opt-in test target and two private files;
no dependency or public-foundation API is needed.

Owner: `/Volumes/KIOXIA/Documents/omics-rust/rsomics-sc`.
Private module: `benches/ingestion_support/mod.rs`.
Contract tests: `tests/cnv_ingestion_measurement_contract.rs`, enabled by the
existing `infercnv-measurement` feature. Existing default test selection and
prepared measurement remain unchanged.

## Interfaces

- `load_expected(full_dir: &Path, maps_path: &Path) -> Result<ExpectedCreation, String>`
  loads only the accepted `01.tsv`, `01.genes.tsv`, `01.cells.tsv` and the
  separate witnessed maps table.
- `ExpectedCreation::reference_names(&self) -> Vec<String>` returns requested
  reference order for configuration outside the measured region.
- `validate_created(created: &CreatedInput, expected: &ExpectedCreation) -> Result<(), String>`
  compares genes/cells, every `f64::to_bits()` and ordered reference/observation
  maps. No numeric tolerance or membership-set substitution is allowed.
- `write_creation(root: &Path, created: &CreatedInput) -> Result<(), String>`
  creates a new absent phase directory containing exactly `expression.tsv`,
  `genes.tsv`, `cells.tsv` and `maps.tsv` on success.

Reuse `tests/support/mod.rs::ExpectedStage::load(full_dir, 1)` privately for
the trusted accepted numerical fixture. Validate its shape/count/coordinate
invariants through `PreparedCounts::new`, not a second numerical data model.
The separate cell/map parser retains roles, which the existing stage loader
does not. Input authentication belongs to the later complete measurement
driver; this helper is not an archive/provenance verifier or general R parser.
The independent controller checker separately uses literal TSV and exact
binary64 values for each warm/measured export.

Expected maps use header `role\tgroup\tindex\tcell` and positive one-based
indices converted to zero-based typed indices internally. Every cell appears
once, matches its literal ID/group/role and is covered. Groups are contiguous,
their member indices increase in source column order, names do not recur,
and all references precede observations. Requested reference order is the map
group order, not an alphabetical sort. Zero reference or zero observation
groups are valid; an empty created matrix is not.

## Actual state and export

`CreatedInput` has private fields and only the checked factory constructs it.
Use its typed getters without rechecking group bounds/coverage or rebuilding
the factory's invariants. Exported cell roles start as observations and are
marked from actual reference indices; do not hardcode the shipped group names.
Maps enumerate actual ordered groups and actual member indices.

Identities, chromosome and group names remain literal, including quotes,
spaces and Unicode. Reject tabs and CR/LF in fields before creating any output
directory, since those cannot be faithfully represented in this TSV contract.
Use 17 decimal places in scientific notation for bit-roundtrippable count
values, including negative zero. Coordinates remain unsigned integers.
Writers use `create_new`, propagate opening/write/flush errors and never
overwrite or delete prior output. A failed export may leave partial evidence;
no successful phase/receipt may be reported for it. Missing-parent and
existing-file/directory failures must preserve pre-existing bytes.

## Tests and gates

Use actual `create_input` with two genes and interleaved reference/observation
columns, non-alphabetical requested reference order, a multi-member reference
group and a negative-zero value. Tests prove exact signed-zero/one-ULP
rejection, column-major exports and re-loading exports, literal identities,
reference/member order, no/all reference configurations, missing/malformed
expected files/maps/roles/coordinates and non-destructive export failures.

Test-first snapshot `red-7` contains the complete new test and registered
target but no helper. Linux must fail specifically on the missing helper
module after dependency metadata succeeds; an environment failure is not RED.
Then implement and review the helper, freeze `green-11`, and run four native
Linux/macOS x86_64/aarch64 debug/release targets with the new dedicated test,
unchanged ordinary/prepared/external tests, formatting and strict Clippy.
No test passes by cross-compilation alone.

The candidate workflow gets a separate default-false `ingestion_measurement`
selector requiring the existing `measurement` selector. Original snapshot
semantics and prepared benchmark remain unchanged. Preserve original APIs,
logs, artifacts and per-source hashes before accepting the new support.
All `src/`, existing tests/fixtures, old bench/support and Cargo.lock must match
the accepted product bytes before and after the gate.

Local Cargo execution remains forbidden at boot APFS 88.68% occupancy.
Controller TMPDIR and standalone rustfmt are external; actual compilation and
native tests use hosted runner scratch. No storage deletion is authorized.
Actual R/Rust trial counters, paired-process ordering, source/input pins and
the 32-process/64-output wall-performance gate remain separate work.
