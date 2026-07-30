# Workflow boundary review

Status: audit complete. `rsomics-workflow` is rejected as a current public
product boundary.

## Decision

The accepted portfolio does not include `rsomics-workflow`.

The only routed historical candidate is a 176-line
`rsomics-sample-sheet` implementation. It validates a private three-column TSV
by checking whether two paths exist. It is not a workflow engine, a workflow
description implementation, an Illumina sample-sheet tool, or a portable
experiment-metadata product.

A 2026-07-31 live check returned 404 from both the
`omics-rust/rsomics-workflow` GitHub repository endpoint and the crates.io
package endpoint. The planning-only name can therefore leave the current
allowlist without deleting, yanking, renaming, or breaking an installed
product.

Established workflow systems already provide recognizable and materially
different products:

- [Snakemake](https://snakemake.readthedocs.io/) owns rule evaluation, DAG
  construction, scheduling, resources, executors, caching, reports, profiles,
  and schema-aware configuration;
- [Nextflow](https://www.nextflow.io/docs/latest/) owns a dataflow language,
  processes/channels, executors, containers, caching/resume, configuration
  profiles, provenance, and the nf-core ecosystem;
- WDL/Cromwell, CWL runners, Galaxy, and Toil own other established language,
  portability, UI, or execution identities.

An rsomics-native engine would be a substantial new product, not a
consolidation of the current source pool. Interest in a static Rust workflow
runner is insufficient reason to reserve a public product name.

## Why sample sheets do not define one product

“Sample sheet” refers to incompatible, policy-bearing formats:

- [Illumina BCL Convert `SampleSheet.csv` v1 and v2](https://support-docs.illumina.com/SW/BCL_Convert/Content/SW/BCLConvert/SampleSheets_swBCL.htm)
  are sectioned CSV documents whose settings, required sections, lane/index
  rules, cycle definitions, and failure behavior depend on the BCL Convert and
  instrument profile.
- An nf-core pipeline normally defines its own CSV columns and JSON Schema.
  [nf-core demultiplexing](https://nf-co.re/demultiplex/latest/docs/usage/)
  distinguishes a pipeline sample sheet from a flowcell `SampleSheet.csv`.
- [Snakemake](https://snakemake.readthedocs.io/page/snakefiles/configuration.html)
  validates configuration dictionaries or sample tables against a
  workflow-provided JSON Schema and separately supports portable encapsulated
  projects.
- PEP, SDRF, and other metadata standards have their own identity, schema,
  transformation, and provenance contracts.
- A product-local `sample_id`, `r1`, `r2` manifest still needs product-specific
  rules for pairedness, lanes, technical replicates, read groups, conditions,
  references, remote objects, and relative paths.

The correct abstraction is a typed input schema owned by the consuming
workflow. A generic parser or path resolver becomes shared only when two
implemented products demonstrate the same policy-free contract.

## Operation routing

| Operation | Current owner | Decision |
|---|---|---|
| product batch manifest | the consuming Layer B product | define and validate the exact schema beside the command that consumes it |
| Illumina BCL Convert sample sheet | future demultiplexing/BCL-conversion scope | no current accepted owner; review with the real upstream and instrument profiles |
| generic CSV/TSV manipulation | `rsomics-table` | table structure only; no sequencing or workflow policy |
| FASTA/FASTQ path and format validation | the consuming product through `rsomics-seqio` | existence alone is not record-format or pairedness validation |
| Snakemake/Nextflow/Galaxy wrappers | integration assets, not Rust products | add only when implemented rsomics binaries and user demand make the wrapper useful |
| pipeline execution | established external workflow engines | adopt and integrate; no current rsomics replacement |
| control-plane campaign state | `rsomics-world` | repository automation metadata, not an installable Layer B tool |

This routing does not add sample-sheet types to `rsomics-common`.
`rsomics-common` owns process-level execution and output contracts, not every
product's input manifest.

## Historical asset disposition

| Historical asset | Audited revision | Disposition |
|---|---|---|
| `rsomics-sample-sheet` | `0312315de44ebe04ad10b9fd387733d5728fe451` | discard the production CLI and public-library shape; retain the two-line TSV only if useful as a malformed/missing-path fixture when a real product schema is implemented |

The source clone and registry-reset archive remain on external storage. The
repository is not revived.

## Audit findings

1. The package description claims “Illumina/custom TSV,” but the parser accepts
   only a tab-separated first field plus optional `r1` and `r2` fields.
   Illumina sample sheets are sectioned CSV and are not parsed at all.
2. There is no README, origin, upstream format version, schema, specification,
   or behavior oracle.
3. The first physical line is skipped whenever it contains the substring
   `sample`. A leading comment makes the real header become data, while a
   header not containing that substring is also treated as data.
4. Trimming the entire line changes leading and trailing field content.
   Quoting, escaped delimiters, duplicate headers, extra columns, ragged rows,
   comments after whitespace, and non-UTF-8 paths have no declared policy.
5. Duplicate and empty sample identities are not checked. Extra columns are
   silently ignored.
6. Relative paths resolve against the process working directory rather than a
   declared sheet or launch base. The code checks only `Path::exists`; it does
   not require a file, readability, FASTA/FASTQ validity, mate compatibility,
   unique inputs, or stable canonical identity.
7. Public fields allow callers to construct entries whose `valid` boolean
   contradicts `errors`.
8. The implementation writes the complete report and only then returns an
   error for invalid rows. With `--json`, the report is sent to a sink, so the
   structured result does not contain the row diagnostics.
9. Named output is opened with direct `File::create`. A late read or write
   failure can truncate an existing destination.
10. The CLI uses the obsolete duplicated `HelpSpec`, not the mandatory current
    `rsomics-help` layer.
11. The “compatibility” test checks only that stdout contains two header words.
    It does not compare any real sample-sheet implementation or specification.
12. The only fixture contains one nonexistent R1 path. The smoke test correctly
    expects failure, but the Criterion benchmark invokes the same fixture and
    asserts success, so the benchmark panics if run.
13. CI runs format, Clippy, and tests only on Ubuntu. It does not run the broken
    benchmark or any native macOS/aarch64 target.
14. The repository contains no DAG, dependency resolution, scheduler,
    resources, retry, cache, resume, environment, container, remote-storage,
    secrets, provenance, workflow language, or execution model.

## Consumer-owned manifest contract

When a product needs batch input, its dossier and implementation define:

- an explicit format/version and exact column or section schema;
- unique stable sample and unit identities;
- single-end, paired-end, lane, replicate, and read-group semantics where
  relevant;
- path base, local/remote URI policy, existence/readability timing, and
  symlink behavior;
- format checks through the appropriate foundation rather than extension or
  existence alone;
- duplicate inputs, missing values, extra fields, quoting, comments, and
  encoding behavior;
- schema and semantic diagnostics with row/field locations;
- normalized structured output and provenance when conversion occurs;
- transactional named output.

Two products may later expose an identical schema-independent table reader,
diagnostic type, or path-resolution primitive. Promotion still requires two
consumer-side contracts and must exclude the products' biological policy.

## External workflow integration

Rsomics products should be easy to orchestrate without an rsomics engine:

- stable exit codes and fail-loud stderr diagnostics;
- stdin/stdout where streaming is meaningful;
- transactional named outputs;
- machine-readable result and provenance schemas;
- deterministic behavior under declared seeds and thread counts;
- clear resource and temporary-storage controls;
- version and upstream-compatibility metadata;
- small end-to-end examples usable from Snakemake, Nextflow, CWL/WDL, or shell.

Wrappers belong with the ecosystem that consumes them. An nf-core module,
Snakemake wrapper, or Galaxy tool definition can be versioned independently
from a Rust crate and does not justify `rsomics-workflow`.

## Reconsideration gate

A workflow product can return to portfolio review only if all of the following
exist:

1. a documented user problem not adequately served by integrating an
   established engine;
2. a recognizable execution or workflow-language identity, not a collection
   of unrelated utilities;
3. a complete operation map covering DAG construction, scheduling, resources,
   failures/retries, cache/resume, environments/containers, provenance,
   temporary data, remote storage, and security boundaries;
4. representative workflows and comparative correctness, restart, portability,
   performance, and resource evidence;
5. an implementation plan large enough to form a coherent first release
   without placeholder operations.

Until then, there is no target repository, public crate, release sequence, or
reserved CLI surface.

## License and attribution

The historical Rust code is team-owned and remains recoverable. Any future
integration asset records the corresponding workflow system, version, format
or module specification, and license. Workflow configuration, container
images, sample data, and institutional profiles receive separate provenance
and redistribution review.
