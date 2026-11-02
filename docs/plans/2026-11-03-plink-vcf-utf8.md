# PLINK VCF record-boundary UTF-8 repair

> One coherent private repair, meaningful RED/GREEN regression and one fresh whole-feature review. Existing unattended authority applies. Root owns integration, immutable capture, native qualification and performance.

**Goal:** Accept valid UTF-8 VCF fields regardless of underlying I/O chunk
boundaries, without changing VCF interpretation or adding a dependency.

**Architecture:** Retain the existing noodles builder, header, decompression
and record decoder. Collect one complete raw record line in a reusable buffer,
then call the same decoder on its complete byte slice. Metadata scanning
retains maximum observed raw record width for later reader admission.

**Tech stack:** Rust1.91, std BufRead and locked noodles-vcf0.90.0.

**Spec:** AGENTS.md; genotype/popgen dossier; actual locked dependency
`src/io/reader/record.rs::read_field`; retained failure in
`plink-complete-blocks-20261103.itHdXe/debug-lib.log` on external scratch.

## Constraints

- Owner base9cc22cac4f71d7e9b249b1f4262d8d4b11869d7e, clean source/index.
- Only `src/io/vcf.rs`, `src/dataset/mod.rs`, `src/make/mod.rs`, the affected
  pre-allocation estimate in `src/stats/mod.rs` and relevant
  existing consumer tests. No library/version/CLI/schema/other parser change.
- All builds/tests/cache/scratch use verified external devices and existing
  guard; scratch `/Volumes/KIOXIA/Developments/tmp`. Preserve old evidence.
- Current valid GT/DS/HDS, headers, PR, half-call, split-PAR, unknown fields,
  compressed input, LF/CRLF and final-record-without-newline semantics stay.
- Invalid UTF-8 remains an explicit error; never use lossy decoding.
- `prepared_reader_bytes()` must remain PGEN-only: its Some value enables
  the prepared-reader parallel make pipeline. VCF must not enter that path.
- No representative performance acceptance or publication before new actual
  native/source gates and measured performance. Prior green11 qualifies base
  metadata only, not scheduling or this repair.

## Design and accounting

Use a reusable raw byte line buffer in metadata and genotype passes. Parse
the complete slice with a temporary noodles Reader into a reused Record;
do not hand-copy fields or switch to the different typed RecordBuf parser.
Track maximum observed raw record width during metadata scan and carry it in
private Source::Vcf state, not a published metadata schema. Both reader memory
estimate and actual reader workspace include raw buffer, conservative record
storage and serializer transient, alongside existing owned variant clone and
I/O allowance. A narrow general/VCF estimate is preferable to changing the
meaning of prepared_reader_bytes. Preserve pre-allocation budget admission.

Second-pass reads must not silently grow past observed/reserved record width.
Read at most that width plus one to detect changed/oversized input and fail
before uncontrolled growth. Preserve existing metadata cross-pass comparison.
Do not add a public record-size flag or broad allocation framework.

Root scope ruling: stats::visit_batches is a concrete consumer used by VCF
genotype filters. Its current PGEN-only preflight must use the new general
reader estimate too, or VCF line allocation precedes budget admission. This
is an accounting call-site change, not a new statistical operation or policy.

## One task

- [x] Add a small real valid UTF-8 record whose multibyte ID is split at an
  actual buffer boundary. Observe the unchanged parser fail before repair.
  Use plain and existing BGZF input paths; construct a BGZF block split with
  the existing writer where useful, no dependency or downloaded fixture.
- [x] Exercise successful metadata plus actual make/genotype reopen, not just
  a helper getter. Pin literal Unicode ID and raw genotype contract; invalid
  UTF-8 still errors. Reuse existing lifecycle evidence for unrelated policies.
- [x] Implement complete-line decoding and maximum-width propagation with
  reader/preflight accounting. Keep source mutation and budget failures loud.
- [x] Run guarded all-feature debug/release, strict all-target Clippy and fmt;
  expected complete suites pass, no ignored/suppressed diagnostics. Record
  actual library/integration populations, including duplicated module tests.
- [x] Freeze complete delta/hashes/logs, leave index empty and root-ready;
  no commits/push/download/native dispatch/cohort timing/publication/deletion.

## Review focus and root integration

Check complete-line lifetime and UTF-8 validation, raw/BGZF read boundaries,
record byte limits and memory ownership/preflight, EOF/newline semantics,
cross-pass detection, and preservation of PGEN-only parallel reader admission.
Root reads the whole change, obtains one fresh review and independently
verifies before commit. Capture the combined committed source and qualify
all four native targets/upstream consumers before representative measurement.
