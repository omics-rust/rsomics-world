# Closed private ingestion trial metadata

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development.

Spec: `2026-09-30-infercnv-raw-ingestion-measurement-design.md`.
Prerequisites: independently accepted native green14 source and installed
R creation contracts38700520489. This component is not a launcher, public API
or performance acceptance. Existing exact exports/counters remain separate.

## Global Constraints

- Only two new private world script/test files; no producer, Rust, R, existing
  prepared-driver, foundation, workflow or public API changes.
- Direct main is authorized; commit only the assigned files, no push/delete.
- No local R/Cargo/build/install/download. Check boot APFS/external paths/free
  space before stdlib-only tests; use external TMPDIR and disable bytecode.
- Preserve failed evidence and unrelated user changes; no Co-Authored-By.
- Keep comments rare and contract-focused; no source narration/history.
- Callers authenticate immutable inputs/source/receipts/executables and the
  independent current-host runtime profile. Never claim that self-reported
  metadata, this checker or the first trial authenticates that trust root.

## Task 1: Typed metadata gate with closed inventories

Add only `scripts/validate_infercnv_trial_metadata.py` and
`scripts/test_validate_infercnv_trial_metadata.py`.

Read the complete frozen producer
`/Volumes/KIOXIA/Documents/omics-rust/rsomics-sc/benches/ingestion_trial/mod.rs`,
the complete `scripts/infercnv_ingestion_matched.R`, and the input/installed-R
receipts. Actual accepted R metadata is available read-only at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-ingestion-measurement-2026-09-30/run-38700520489/artifact/ingestion-package/fresh-plain`.

Expose frozen dataclasses `TrialBindings` and `CheckedMetadata`, and
`validate_trial_metadata(root, implementation, bindings) -> CheckedMetadata`.
Bindings contain case, canonical artifact/source/receipt/scripts/output paths,
exact12 input paths/hashes,7 code paths/hashes, native executable/version,
and an independently authenticated current-host R profile. Use field names
input_case, artifact_root, source_root, receipt_path, scripts_dir, output_dir,
input_paths, input_hashes, code_paths, code_hashes, native_executable,
native_version, r_profile. Paths are Paths; named path/hash collections are
immutable-by-contract mappings. Validate their shape without pretending to
authenticate caller-supplied expectations. r_profile has exactly8 keys: system,
Python, namespaces, blas, options_dput, object_options_dput, clock_source,
functions. Their shapes mirror corresponding R runtime fields; functions maps
the3 names to expected installed-text hashes. This profile comes from an
independent current-host preflight and frozen source proofs, never a timed
trial chosen as its own oracle. Unit tests may inject small trusted profiles;
integration must use actual admitted profiles. Result contains
implementation/case, observed before/after thread counts, seven lifetime I/O
counters and file hashes. Keep interfaces narrow; no CLI/network/launch/build
or new format producer. The later owner supplies trust and process chronology.

Before opening any result file, recursively lstat the root, ancestors and all
entries. Reject symlinks, missing/extra entries, FIFO/socket/device/special
files and unexpected nested directories. Require exact four regular phase
files in warm/measured. Native root adds only metrics.tsv/runtime.tsv/
io.lifetime.txt. R root adds exactly15 regular files from the frozen producer:
clock-proc-time.txt, function-.order_reduce.txt,
function-CreateInfercnvObject.txt, function-validate_infercnv_obj.txt,
functions.tsv, io.lifetime.txt, measured-checker.stderr,
measured-checker.stdout, warm-checker.stderr, warm-checker.stdout,
metrics.tsv, packages.tsv, provenance.json, runtime.json, session.txt.
Do not open a path learned from metadata before comparing it to the expected
closed filename. Hash all admitted regular output files after validation.

Reuse `parse_region_metrics` for literal12 metrics and bind implementation/
case. Do not reimplement exact count/map comparison or campaign scheduling.
Parse other TSV literally (no CSV unquoting/whitespace normalization); reject
CR, duplicate/missing/extra keys and malformed rows. Strict JSON must reject
duplicate keys, NaN/Infinity and wrong nested key inventories/types. Booleans
must not alias integers. Positive numeric identifiers/threads are boundedu64;
I/O counters may be zero. I/O is exactly the seven Linux procfs keys, no deltas
or inference of zero input consumption.

Native runtime is exactly65 fields emitted by its producer:50 fixed named
fields plus argv_0..3,2 ordered references,3 excluded chromosomes and6 emitted
thread variables. Compare all policy/scope literals to frozen producer values,
not presence-only checks. Bind schema/raw_creation/rust/factory, Cargo product
version, linux/x86_64/release assertionsfalse, exact accepted receipt/run/
artifact/hash identities, canonical input/expected/output/executable paths,
argv, byte lengths,256MiB numeric/1MiB record budgets,1/Inf depth, f64
representation,9939x184 result, reference/exclusion order and six thread limits1.
PID/observed threads must be positive bounded decimal integers. Native PID is
not GNU time's wrapper PID; do not claim authenticated PID from metadata alone.
The native producer does not emit VECLIB_MAXIMUM_THREADS: do not invent it.

R runtime has exactly20 top-level fields; provenance exactly13. Validate all
constant schema/contract fields and complete nested shapes from producers.
R4.6.1/Bioc3.23/infercnv1.28, Linux/x86_64, six C locale categories,7 requested
thread values1, positive observed threads. Preserve observed counts, never
require one. Bind Python executable/version/hash projection to independent full
identity (empty libpython mappings are legitimate), all6 namespace versions/
library paths, full8-field current-host Sys.info, BLAS and global options to
the supplied profile. Both object metadata records require infercnv class,
double matrix/array9939x184 and exact canonical options dput, not merely Inf
substring. Exact timer/preparation/RSS/I/O/lifetime policy strings are frozen.

Bind clock path/function text/hash to independently authenticated profile AND
actual file bytes. Bind3 ordered functions.tsv rows, source/hash, exact proof
filenames and both actual/profile function-text hashes. Current accepted clock
SHA09efec79c3b4ef0d1df8667e4fa026ec17ca5c425f08b4db7d774628bdcbaf3f
and primitive text `.Primitive("proc.time")` are evidence, not permission to
accept a self-consistent changed document. Profile function hashes must match
the accepted installed/source proof. Preserve packages.tsv as literal3-column
Package/Version/LibPath table; unique(Package,LibPath), not unique names; each
loaded namespace must have its matching version/library row. session/checker
logs remain supporting raw files, not brittle parsed correctness evidence.

R provenance binds exact case, receipt/run/artifact/source,12 input path/hash
entries, all8 creation arguments (ordered refs/exclusions, tab,NULL,[1,"Inf"]),
7 code paths and identical frozen before/after hashes, exact5 reused fixed
hashes and literal authentication scope. R emits no argv/PID: do not invent
them or confuse metadata checking with process-owner proof.

Write tests first with genuine missing-module RED. Use small closed synthetic
file trees whose metadata faithfully matches the frozen contract; large count
matrices are unnecessary because exact exports are a separate existing gate.
Cover both implementations/cases, every binding category, quoted/CR TSV,
duplicate keys, bool-as-int, NaN/Infinity, nested key additions, missing/extra/
symlink/FIFO/socket/nested entries, swapping case/path/argv, noncanonical IDs,
native nonexistent thread field, duplicate installed names at distinct libs,
wrong namespace lib, malformed I/O and self-consistently changed clock/function
file plus reported hash. Ensure special-file rejection cannot hang.

Run focused tests while iterating, full controller suite once before commit,
self-review and report genuine RED/GREEN with commands/output and limitations.
Commit only these two files; root owns push/exact-head Linux CI/task review and
whole-launcher review. Do not dispatch agents. Raise material interface ambiguity
to root instead of introducing a framework or expanding producer scope.
