# rsomics-vcf annotate family design

Status: the released transfer operation, complete tag-filling and distance
surfaces, upstream authorities, historical dispositions, safety corrections,
foundation use, compatibility matrix, and release gates are defined. The
target release is 0.15.0 after the complete statistics slice.

## Product boundary

`annotate` owns VCF/BCF transformations whose primary result is adding,
removing, renaming, or deriving typed record fields without changing the
variant alleles or genotype calls. Three user-recognizable operations share
the product's header, record, expression, sample, region, output, and help
layers:

- transfer typed fields from a second variant or tabular source;
- derive standard and expression-based INFO or FORMAT fields from each record;
- annotate each sorted record with positional distance to neighboring records.

Reference-orientation diagnosis and repair remain a separate `fixref`
operation because they can rewrite REF, ALT, and GT. Consequence prediction and
VEP/BCSQ field expansion belong to `rsomics-annotation`. Genotype QC, HWE
reports, LD, and per-sample analysis remain in `rsomics-plink`; only the
record-local HWE value requested by `fill-tags` is written here. Normalization,
filtering, querying, and set operations retain their current commands.

No retired annotation operation becomes another crate.

## Released transfer base

The current flat `annotate` operation shipped in `rsomics-vcf` 0.4.0. Its
release revision is `203b11974adf719f24ac485fbcc8d02fa77e5423`, and the
current product head before this dossier is
`682942cfa69768dc3a127a8544f2f07213b704ea`.

The released implementation already provides typed VCF/BCF and tabular
sources, checked header edits, fixed/INFO/FORMAT transfer, sample mapping,
allele-aware cardinality projection, position/interval/allele matching,
expressions, regions, atomic named output, bounded overlap buffering, unified
help, differential fixtures, and measured bounded-memory evidence. It is the
implementation base, not a historical source to reconstruct.

Release 0.15 reorganizes that operation under a nested family without
rewriting its proven transfer engine. New work must preserve every 0.4 through
0.6 transfer regression and exact output contract.

## Upstream authority

The behavior oracle is bcftools and HTSlib 1.24 at revision
`fb9f0f783e0f67d734f6fa7fe4df9d230522f196`:

- `bcftools annotate` and `vcfannotate.c` for transfer and header editing;
- `+fill-tags` and `plugins/fill-tags.c` for standard fields, population
  groups, and calculated fields;
- `+variant-distance` and `plugins/variant-distance.c` for positional
  neighborhood semantics.

Audited SHA-256 values are:

- `vcfannotate.c`:
  `6be47073e1d549f2bcded27f4cf8952ccd03f90ad537088f418dfe8a5d645730`;
- `plugins/fill-tags.c`:
  `78f9048d5ea73e6b42be2a25e91a016b31c6762cbefd159adcc10eab866c15dc`;
- `plugins/variant-distance.c`:
  `d8effda38450e4ffe79388feb937d412eb6fffd1a56c636902e072bc5f149412`.

The sources are MIT licensed. VCF 4.5 and BCF2 remain the type and cardinality
authorities. The Wigginton exact test defines HWE and excess-heterozygosity
probabilities. Upstream source and official documentation are behavior
evidence; no permissive parser, warning-only policy, or undefined result is
inherited merely for byte similarity.

## Historical assets

| Repository | Revision | Version | Tracked Rust, manifest, and test lines | Disposition |
|---|---|---:|---:|---|
| `rsomics-vcf-annotate` | `c958d89eeb5ff8ec0ce343ded3ab9ddfe10e957a` | 0.1.3 | 623 | compact BED and replacement fixtures only; implementation already replaced by 0.4 |
| `rsomics-vcf-fill-tags` | `a28b803cebc218468fd53280f5166ea76198f03a` | 0.1.2 | 1,525 | refactor HWE call, genotype fixtures, formatting expectations, and corrected algorithms; replace parser and orchestration |
| `rsomics-vcf-variant-distance` | `b9a86dd089539bd9d3147acae72f3b19bfe8015a` | 0.1.1 | 975 | retain compact positional and INFO fixtures; replace whole-file implementation |

All three retired clones are clean. Their standalone binaries, Clap surfaces,
direct writers, duplicated help descriptions, audit narration, and process
launch assumptions are discarded.

### Historical fill-tags

The old implementation contains useful diploid and haploid count cases, the
`rsomics-stats::hwe_exact` call, float-format expectations, malformed-input
seeds, and small outputs that still match the selected bcftools 1.24 rows. The
representative 1.24 row SHA-256 is
`5ca7cc5e85b4fe6314011fe58f199516aa878c901ae40779f1ff6b48431096d7`.

It is not a complete or safe replacement. It line-parses only plain or generic
gzip VCF, has no BCF or standard input, silently maps invalid generic-ploidy
alleles to reference or ignores out-of-range indices, leaves sites-only input
unchanged, and implements only ten fields. Population groups, F_MISSING, END,
TYPE, VAF/VAF1, ADF+ADR, calculated assignments, selections, and grouped index
output are absent. Its multi-worker route reads and decompresses the complete
file into memory. Named output is created directly.

Its claimed 2.48-times result compared an eight-thread whole-file path with
single-threaded bcftools 1.23.1, omitted complete semantic equality, input and
binary hashes, repeated raw rounds, RSS, and the unsupported operations. The
claim is rejected; only the fixture generator remains useful.

### Historical variant distance

The old distance implementation preserves nearest-distinct-position semantics,
same-position groups, one-position contigs, existing INFO replacement, and
several compact header cases. It supports only nearest distance and plain VCF.
It packs every record, offset, chromosome name, and position into memory;
therefore its stated single pass is not streaming. The position parser accepts
a numeric prefix, does not validate positive coordinates or ordering, and the
writer reconstructs untyped text.

Its Criterion benchmark measures an in-memory Rust function over generated
text. It neither invokes the upstream executable nor includes file IO,
semantic hashes, RSS, binary revisions, or repeated paired trials. The
unspecified faster-than-upstream statement is rejected.

## Command tree and migration

The canonical command tree is:

```text
rsomics-vcf annotate transfer [OPTIONS] [INPUT]
rsomics-vcf annotate fill-tags [OPTIONS] [INPUT]
rsomics-vcf annotate distance [OPTIONS] [INPUT]
```

The released `rsomics-vcf annotate [TRANSFER OPTIONS] [INPUT]` grammar remains
an accepted compatibility route to `annotate transfer` through 1.0. It shares
the same argument and execution types; no second transfer implementation or
duplicate help specification is retained. Canonical help, examples, and shell
completion show the nested commands plus one migration line.

Every leaf uses `rsomics-help`. Family help first distinguishes transfer,
calculated fields, and positional distance. Leaf help shows only relevant
fields, selection, grouping, direction, source, and output options.

## Shared input and output contract

All leaves accept plain VCF, BGZF VCF, raw BCF, BGZF BCF, and standard input by
content. They share:

- `--include EXPR` or `--exclude EXPR`, never both;
- indexed regions and streaming targets with explicit overlap policy;
- the product's typed record validation and expression engine;
- `-O v|z|b|u`, bounded BGZF workers, and atomic named output;
- optional grouped TBI or CSI creation for compatible named output;
- JSON summaries separated from the variant stream;
- checked output and index aliases, counts, coordinates, and cardinalities.

Selections are applied before annotation. `transfer` and `fill-tags` retain
`--keep-sites` to emit expression-rejected records unchanged. `distance`
matches the upstream filtering model: rejected records are absent and do not
define neighbors. This avoids an ambiguous stateful mixture in which retained
but unannotated records may or may not change the measured distance.

Named variant output and an optional index commit together through the existing
multi-file transaction. Standard output cannot request an index. A broken pipe
is distinct from invalid input. No named predecessor is replaced until the
complete stream and index validate and close.

## Transfer contract

`annotate transfer` retains the complete released 0.4 contract and its current
modules. Release 0.15 adds only the family routing and grouped output-index
transaction. In particular:

- VCF/BCF, BED, and one-based tabular source coordinates remain distinct;
- fixed fields and typed INFO/FORMAT values retain explicit replacement,
  missing, append, and existing-only policies;
- allele-numbered values use checked source-to-target mappings;
- sample mapping, header removal and rename, ID derivation, site marks,
  pairing logic, and overlap fractions keep their existing semantics;
- source and target ordering, collisions, declarations, and type/cardinality
  mismatches fail loud.

The experimental upstream merge-logic expression language and `--force`
remain excluded. Every stable operation already declared in 0.4 remains
present; excluded flags are not accepted and ignored.

## Fill-tags contract

`annotate fill-tags` replaces `+fill-tags` and the retired micro-crate. The
predefined fields are:

| Field | Number | Source and meaning |
|---|---|---|
| INFO/AC | A | called alternate-allele copies |
| INFO/AC_Hom | A | copies in fully called homogeneous genotypes |
| INFO/AC_Het | A | copies in fully called heterogeneous genotypes |
| INFO/AC_Hemi | A | copies in haploid or compatibility half-missing genotypes |
| INFO/AF | A | AC divided by AN, or checked INFO/AC and INFO/AN without samples |
| INFO/AN | 1 | all called allele copies |
| INFO/ExcHet | A | one-tailed excess-heterozygosity probability |
| INFO/HWE | A | two-sided exact HWE probability |
| INFO/MAF | 1 | second-largest frequency across REF and every ALT |
| INFO/NS | 1 | samples with at least one called allele |
| INFO/END | 1 | checked one-based inclusive record end |
| INFO/F_MISSING | 1 | samples with any missing GT allele divided by samples |
| INFO/TYPE | . | typed REF, SNP, MNP, INDEL, OTHER, BND, or OVERLAP classes |
| FORMAT/VAF | A | per-sample alternate depth divided by total allele depth |
| FORMAT/VAF1 | 1 | per-sample combined alternate depth fraction |

`--tags all` follows the recognizable upstream set and excludes END and TYPE;
those two fields require explicit selection. Field names are case-insensitive
only while parsing the predefined list and serialize with canonical spelling.
Unknown or duplicate requests fail.

### Genotypes and populations

AN and AC count actual called copies for every checked ploidy. A triploid
`1/1/1` therefore contributes three, not two. Fully called homozygous and
heterozygous category counts retain allele-copy meaning at polyploid sites.
Out-of-range, malformed, negative, or structurally inconsistent GT fails with
sample and record context.

The default compatibility policy counts the one called allele of a half-missing
diploid genotype as hemizygous. `--drop-missing` still permits that called copy
in AN and AC but excludes it from AC_Hemi. Fully missing genotypes contribute
only to F_MISSING. The summary records fully called, partial, and fully missing
sample counts so the policy is visible.

HWE and ExcHet are calculated per ALT from fully called diploid genotypes whose
alleles are limited to REF and that ALT. Genotypes containing another ALT,
different ploidy, or missing alleles are excluded from that ALT's exact test
and counted. This is identical to bcftools for valid biallelic diploid input
and avoids its silent multiallelic approximation.

`--groups FILE` accepts exactly two columns: sample and a comma-separated list
of population labels. Every sample and label must be valid and unique within
its declared relation; unknown samples, duplicate rows, empty groups, and tag
suffix collisions fail. Unsuffixed fields summarize all selected samples and
`_GROUP` fields summarize each named group. Header and output group order
follow first declaration.

### Depth and missing sources

VAF and VAF1 prefer valid Number=R FORMAT/AD. If AD is absent, compatible
Number=R FORMAT/ADF and FORMAT/ADR are summed with checked arithmetic. Present
fields require exact per-sample cardinality, nonnegative integers, and totals
that do not overflow. A zero total produces a declared zero fraction, matching
the sound upstream behavior.

Requesting a field whose required GT or depth source is unavailable fails by
default. `--missing-source clear` removes any stale destination and writes a
typed missing value while incrementing an unavailable counter. There is no
warning-only mode that leaves an old derived value looking current.

Existing destination definitions must have the same Number and Type. A
conflicting declaration fails before output. Existing values are replaced only
for requested fields; unrelated INFO and FORMAT order remains stable where the
encoding permits it.

### Calculated assignments

The tag list also accepts the established calculated assignment grammar:

```text
INFO/DP:1=int(sum(FORMAT/DP))
FORMAT/DP:1=int(smpl_sum(FORMAT/AD))
INFO/good:1=int(N_PASS(binom(FORMAT/AD[:0],FORMAT/AD[:1]) >= 1e-5))
```

Assignments compile once through the existing typed VCF expression engine.
Destination scope, Number, and Integer or Float type are explicit unless an
identical existing header definition supplies them. Result shape must match
exactly; values are never silently truncated or padded. Integer conversion
requires finite in-range values and the declared rounding function. Population
groups constrain sample reductions and produce the same deterministic suffixes
as predefined fields.

## Distance contract

`annotate distance` measures the difference between one-based POS values at
neighboring distinct positions on the same contig. Records at the same position
form one block and do not measure zero to each other. Supported directions are
`nearest`, `forward`, `reverse`, and `both`; `fwd` and `rev` are accepted input
aliases. The default destination is INFO/DIST, and `--tag` selects another
valid INFO identifier.

For `nearest`, ties choose the forward distance to match the upstream
decision, although the numeric value is equal. A single-position contig has no
distance and leaves the destination missing. `both` writes previous then next;
an unavailable side is a typed missing value rather than the integer zero.

Input must be nondecreasing by declared contig order and position. A coordinate
regression, reopened contig block, nonpositive coordinate, or checked distance
overflow fails. The implementation buffers one distinct-position block plus
the state required for its neighbors, so memory depends on maximum duplicate
multiplicity rather than total records.

An existing destination definition must be Integer Number=1, or Number=2 for
`both`. A conflicting header fails. Existing destination values are replaced;
unrelated INFO fields are preserved. Region and target selection occurs before
distance state, matching a stream that physically contains only selected
records.

## Deliberate safety differences

Live bcftools 1.24 probes establish these explicit corrections:

| Probe | Upstream result | rsomics contract |
|---|---|---|
| one `1/1/1` and one `0/1/1` genotype | AN=4 and AC=3 although six alleles are called and five are ALT | count actual ploidy: AN=6 and AC=5 |
| multiallelic HWE | counts other-ALT genotypes through an approximation noted in source | use only valid per-ALT diploid REF/ALT projections and report exclusions |
| requested VAF with missing or malformed depth | warn once or silently retain no new value | fail, or clear stale output only under explicit policy |
| unknown or duplicate population sample | warn and continue | fail before variant output |
| unsorted positions 20 then 10 | exit 0 and write DIST=-10 to both records | fail on the coordinate regression |
| `both` at the first and last position | write `0,next` and `previous,0` | write missing,next and previous,missing |

Invalid GT allele indices already fail in bcftools 1.24 and remain hard errors.
Sound diploid/haploid, standard field, population, expression, and sorted
distance behavior retains exact or typed-equivalent compatibility.

## Foundation decision

No new public foundation item or crate is required.

- `rsomics-help` supplies the nested family, leaf help, diagnostics, version,
  completion, and JSON-result conventions.
- `rsomics-common::AtomicFile::commit_all` supplies the variant-plus-index
  transaction.
- `rsomics-stats::hwe_exact` remains the policy-free Wigginton kernel. The
  named consumers are `rsomics-vcf annotate fill-tags` and the near-term
  `rsomics-plink stats hardy` implementation. Both require differential tests
  over the same validated `(nref, nalt, nhet)` contract before the dependency
  is released.
- Variant types, allele-copy categories, partial missingness, population
  suffixes, field assignments, and distance buffering remain private VCF
  product policy.

The old fill-tags parser does not justify moving genotype logic into
`rsomics-stats`. The numerical foundation receives counts, not VCF records,
headers, samples, or ploidy decisions.

## Product structure

```text
src/
├── annotate.rs
├── annotate/
│   ├── columns.rs
│   ├── distance.rs
│   ├── edit.rs
│   ├── fill_tags.rs
│   ├── genotype_counts.rs
│   ├── header.rs
│   ├── matching.rs
│   ├── set_id.rs
│   └── source.rs
└── commands/
    ├── annotate.rs
    └── annotate/
        ├── distance.rs
        ├── fill_tags.rs
        └── transfer.rs
```

The existing transfer modules move only as required by the command family.
`genotype_counts.rs` owns one checked per-record count model shared by standard
fields and HWE projections. `fill_tags.rs` owns field plans and record-local
updates. `distance.rs` owns the bounded ordered state. Command adapters contain
no genotype math, expression evaluation, header mutation, or direct file
creation.

## Compatibility matrix

The pinned matrix covers:

- every released transfer edit, source type, column policy, sample mapping,
  pair mode, overlap rule, selection, region, and output encoding;
- all predefined fields singly, in combinations, and under `all`;
- no-ALT, biallelic, multiallelic, symbolic, breakend, spanning-deletion,
  gVCF, mixed-type, and malformed records;
- haploid, diploid, triploid, tetraploid, mixed-ploidy, homogeneous,
  heterogeneous, half-missing, fully missing, phased, and invalid GT;
- zero samples, missing per-record GT, INFO AC/AN fallback, empty and zero
  allele counts, checked overflow, and stale destination handling;
- AD and ADF+ADR, zero depth, missing vectors, cardinality mismatch, negative
  values, overflow, VAF, and VAF1;
- HWE and ExcHet biallelic exact values, multiallelic projections, excluded
  genotypes, monomorphic alleles, and large counts;
- group order, overlapping groups, sample subsets, unknown and duplicate
  samples, invalid labels, empty groups, and destination collisions;
- calculated INFO and FORMAT assignments, reductions, per-sample results,
  binomial, Fisher, phred, explicit casts, result shapes, nonfinite values,
  and group masks;
- distance directions, ties, same-position blocks, first and last positions,
  single-position contigs, custom tags, selections, contig changes, coordinate
  regression, reopened contigs, and overflow;
- all four encodings, standard input, indexed regions, streaming targets,
  serial and bounded parallel output, grouped indexes, aliases, write failure,
  broken pipes, and JSON separation.

## Tests

Unit tests cover genotype copies and categories, partial-missing policy, HWE
projection, depth fractions, field planning, assignment result shapes, group
maps, distance transitions, header compatibility, and checked arithmetic.

Golden tests import only the historical fixtures selected above and rerun them
against 1.24. Differential tests compare all sound standard fields, population
groups, calculated assignments, and distance directions against the pinned
oracle. Corrected cases assert the documented rsomics values and upstream
evidence separately rather than weakening equality checks.

Property tests generate valid mixed-ploidy genotypes and require AN to equal
the number of called copies, AC plus reference copies to equal AN, AF to equal
AC/AN when defined, and category totals never to exceed allele counts. Distance
properties compare the streaming reducer with a simple sorted distinct-position
model under randomized duplicates and contig boundaries.

Malformed and fault-injection suites require no complete-looking stdout prefix
or replaced named output after an invalid header, record, expression, group,
coordinate, write, close, or index operation.

## Performance gates

Formal comparisons pin source and binary hashes, machine, filesystem, input
hashes, flags, worker counts, warmups, alternating raw rounds, output semantic
hashes, wall time, CPU, peak RSS, and bytes read and written.

Representative workloads are:

- the existing two-million-record transfer interval and typed-source fixtures;
- ten million mixed-ploidy records and 2,000 samples for standard fields;
- five million records with AD, ten population groups, and calculated fields;
- twenty million records with duplicate-position bursts across all distance
  directions;
- BGZF VCF and BCF variants of the fill-tags and distance hot paths.

Transfer must retain its established bounded-memory advantage without a
material regression. Fill-tags and distance must each show a strict throughput
or peak-memory advantage over their equivalent bcftools 1.24 operation while
producing the same sound values. Parallel fill-tags uses bounded ordered batches;
it may not regain the historical whole-file memory strategy. A speed result
that omits fields, populations, expressions, typed input, or output validation
is not accepted.

## Release gate

Release 0.15.0 is complete only when:

- the nested command family and flat transfer compatibility route share one
  implementation and one unified help contract;
- transfer retains every released compatibility, failure, transaction, and
  performance guarantee;
- all predefined fields, populations, calculated assignments, and distance
  directions are implemented without placeholders;
- historical assets are imported only according to this disposition and
  source comments are limited to stable non-obvious invariants;
- all standard, corrected, malformed, property, differential, transaction,
  encoding, index, and fault-injection tests pass;
- both new hot paths pass their formal performance decision;
- the `rsomics-stats::hwe_exact` call receives an API review and consumer-side
  compatibility test without moving VCF policy into the foundation;
- package contents, metadata, README, nested help, licenses, and attribution
  pass a fresh public API and production hot-path review;
- native Linux and macOS CI pass on `x86_64` and `aarch64` at the exact head;
- publication occurs only after every earlier declared release slice and this
  complete family are present.

Audit evidence is retained outside the repository at:

```text
/Volumes/KIOXIA/Developments/tmp/rsomics-vcf-annotate-audit-20260819
```

Primary references are the
[bcftools 1.24 annotate manual](https://samtools.github.io/bcftools/bcftools.html#annotate),
the [official fill-tags guide](https://samtools.github.io/bcftools/howtos/plugin.fill-tags.html),
the [bcftools plugin list](https://samtools.github.io/bcftools/howtos/plugins.html),
the [VCF and BCF specifications](https://github.com/samtools/hts-specs), and
Wigginton et al. 2005 (PMID: 15789306).
