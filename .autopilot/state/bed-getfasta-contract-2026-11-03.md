# BED reference-extraction source contract

Source inspection only. No native implementation, new foundation API,
runtime oracle result, performance qualification or release is claimed.

## Upstream contract

Root reads complete `src/fastaFromBed/{fastaFromBed.cpp,fastaFromBed.h,
fastaFromBedMain.cpp}` and `src/utils/sequenceUtilities/sequenceUtils.cpp`
from BEDTools2.31.1 source archive SHA256
1db4e57c1899aede13ef516c61c9b5819cf2790994a556dbb28972488f9946e6,
prefix `bedtools2-705ccfdf2c9a77d71560c8adcece0663c2f5e18e`.
The BED `reportBedTab` implementation is also inspected directly.

The operation extracts zero-based half-open reference intervals in input
order. Default output is a coordinate FASTA header and one unwrapped sequence
line. `-name` and deprecated `-name+` both produce
`name::chrom:start-end`; `-nameOnly` produces the name alone. With `-s`,
the header receives the strand suffix, and only minus-strand sequence is
reverse-complemented. `-rna` changes A/a complements to U/u during that
reverse complement; it does not transliterate forward-strand sequence.
The complete source complement preserves case, maps IUPAC pairs, treats
T/t and U/u as complements of A/a, and leaves unrecognized bytes unchanged.

`-split` concatenates BED blocks in their stored order before applying
strand reversal to the assembled sequence. `-tab` writes header plus sequence;
`-bedOut` writes BED columns plus sequence and takes precedence over naming
and FASTA/tab formatting. Missing-reference and reference-overrun records
are intentionally warned/skipped; zero-length points are also warned/skipped.
These behaviors are not covered by the approved map point-loss repair.
GFF/VCF input and malformed-block semantics need their own explicit scope
decision and evidence; BED support must not silently imply them.

## Output-routing discrepancy requiring a real witness

In `ReportSeq`, `-bedOut` calls `_bed->reportBedTab(bed)`, which uses
`printf` to stdout, then writes sequence to `*_faOut`. The constructor points
`_faOut` at the requested `-fo` file when present. Source therefore routes
BED columns and sequence to different streams for this combination. This
is a source-level finding, not a captured runtime oracle or approved repair.
Before implementing named-output behavior, capture both full streams/file
and resolve compatibility policy. Do not reproduce mixed routing accidentally
or silently extend the user's map-specific defect decision.

## Historical asset disposition

Root reads the complete old `rsomics-bed-getfasta/src/lib.rs` at
82783b773f900d8d3a2ce6eac1f1367cbfcdd16e. Refactor-then-merge remains
the disposition, not direct adoption. Its whole-reference HashMap and
whole-BED string loading are unsuitable for this product's execution model.
It silently overwrites duplicate reference names, clamps out-of-range
coordinates instead of source skipping, interprets `use_name` as name-only,
and uppercases/replaces IUPAC bases with N during reverse complement.
It lacks block, tab, BED-output and RNA contracts. Preserve useful fixtures
after oracle comparison; replace the parser/loader and incorrect policies.

## Existing foundation, not another reference crate

Root rereads complete published `rsomics-seqio` IndexedFasta implementation.
The existing CALL consumer and registry0.6.0 identity are pinned in
`bed-nuc-contract-2026-11-03.md`. BED extraction and nucleotide profiling
should reuse that reader rather than revive private FAI/whole-genome loaders.
Strand/header/block/output policy belongs inside BED, not in Layer A.

Important memory constraint: `cache_capacity` is a prefetch granularity,
not a maximum allocation. `load` computes the cached end using
`max(range.end)`, and the noodles query constructs the complete requested
sequence. A long fetch can therefore allocate much more than the default
1MiB. Any bounded-memory extraction/profile implementation must issue
bounded queries or explicitly account for the full interval and assembled
blocks; do not advertise constant memory from the cache setting alone.

Plain indexed files are supported by the current File-backed reader.
Auto-index creation, compressed indexed access, full-header naming and
transactional diagnostic policy remain explicit integration questions.
No speculative public API or dependency change is authorized by this note.
