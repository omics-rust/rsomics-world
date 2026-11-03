# BED reference-extraction source contract

Source inspection and seven real source workflows. No native implementation,
new foundation API, performance qualification or release is claimed.

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

## Output-routing discrepancy

In `ReportSeq`, `-bedOut` calls `_bed->reportBedTab(bed)`, which uses
`printf` to stdout, then writes sequence to `*_faOut`. The constructor points
`_faOut` at the requested `-fo` file when present. Source therefore routes
BED columns and sequence to different streams for this combination. This
is now confirmed by the real witness below, but is not an approved repair.
An async question requests unified complete BED-plus-sequence file output
with known incompatibility versus strict source routing. Do not reproduce
mixed routing accidentally or extend the user's map-specific decision.

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

## Seven real source workflows

Packet `/Volumes/KIOXIA/Developments/tmp/bed-getfasta-witness-20261103.5FzHQADd`.
Root authors and reads the complete2811B helper, fixed49B mixed-case/IUPAC
reference and78B two-row BED12 input. The owning BED guard verifies physical
external USB disk12s1, all Cargo/Rustup/tool/cache/tmp paths, actual resolved
Cargo target and27GiB free before execution. Each source call has10s timeout,
core0 and32KiB output-file limits; the packet has1MiB size/8GiB headroom
checks. No build, benchmark or retry. Exact existing BEDTools binary identity
is1691808B SHA256
fc994e1fcfdce8ce8591630d71f21fe67e2275c2a8d967dc0e3dfb4cfa4677a3.

Exactly seven calls all exit0: default; name-plus-strand; nameOnly/tab;
split/strand; RNA/strand; BED output/stdout; BED output/named file.
Root reads every complete stdout/stderr and the named file. All stderr
streams are empty. Default extraction is `AaCGtrYnUuBD`; DNA minus is
`HVaAnRyaCGtT`, RNA minus `HVaAnRyaCGuU`. Split output is `AaCGUuBD`
forward and `HVaACGtT` reverse. Names include coordinates for `-name`,
not for `-nameOnly`. The named BED-output call retains only26B of sequence
lines in its file and78B of concatenated BED columns without newline in
stdout; ordinary stdout mode produces104B of two complete BED-plus-sequence
rows. This confirms the source routing discrepancy on valid input.

Helper SHA256 f6b7f6a2d76482cf116d049283f2a4212526c5ffefc6c211444d2adbd8fb1525;
reference70ad872bf24323f6b134854da0128b2309ed2682e38d44b821b5e040d1d18530;
BEDb77fc21a793c96631c6a908b41ef220dba8b25b65db6acf0a6f18d930f2c71af;
ORACLEe8f56ce1b7add93616ea03098a49d5ccf4ce92943dca4ef9779a891fbfeac932.
The source-generated17B FAI and all full outputs remain in the packet.
This small witness establishes these semantics, not compressed-reference,
large-input, malformed-block, absent-reference or native correctness claims.
