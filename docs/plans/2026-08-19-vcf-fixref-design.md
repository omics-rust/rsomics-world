# rsomics-vcf fixref design

Status: the complete orientation-inspection and repair boundary, upstream
authority, historical disposition, typed allele policy, safety corrections,
compatibility matrix, performance evidence, and release gates are defined.
The target release is 0.16.0 after the unified annotation family.

## Product boundary

`fixref` diagnoses and repairs strand convention and reference-orientation
problems in biallelic SNP records. It can complement alleles, exchange REF and
ALT, remap allele-indexed values, or relocate records from an authoritative ID
source. Those changes are scientifically different from representation
normalization and merit a visible operation with an inspection-first UX.

`norm` owns left alignment, REF checking, multiallelic splitting and joining,
duplicate removal, and representation normalization. Its `--check-ref fix`
mode repairs locally provable REF/ALT representation mistakes; it is not a
strand inference tool. The bcftools guide explicitly warns that using the
corresponding normalization switch to repair strand problems can create
nonsensical genotypes. `fixref` therefore remains separate while sharing one
private typed allele-transformation primitive with `norm`.

`fixref` does not infer a reference build, liftover coordinates between builds,
normalize indels, repair arbitrary malformed VCF, or replace array-genotyping
cluster normalization. Build conversion remains in `rsomics-liftover`.
Genotype QC and strand checks across PLINK datasets remain in `rsomics-plink`.

## Upstream authority

The compatibility oracle is bcftools and HTSlib 1.24 at revision
`fb9f0f783e0f67d734f6fa7fe4df9d230522f196`. The audited
`plugins/fixref.c` SHA-256 is
`ea74c8217e425539b767aa20454e453399e9b1d16606bb0b72c8f98e0a44e666`.
The source is MIT licensed. VCF 4.5 and BCF2 define typed field cardinality,
genotype ordering, phase, and missing values; indexed FASTA defines the
reference byte at each checked coordinate.

The recognizable upstream modes are statistics, Illumina TOP conversion,
ID-guided repair, unambiguous flip, all-SNP flip, allele-only REF/ALT repair,
and swap-only repair. Sound observable behavior is retained. Warning-only,
silent-loss, annotation-corrupting, and undefined arithmetic behavior is not.

## Historical asset

The retired `rsomics-vcf-fixref` clone is clean at revision
`d6efd2bd79067b2b7b2f738703e428ca40dc56f1`, version 0.1.1. It contains
1,194 tracked Rust, manifest, and test lines. Its compact four-action SNP
fixture, complement table, action-count expectations, and a subset of its
algorithm branches are test seeds.

The standalone implementation is discarded. It accepts only text VCF, has no
BCF, compressed-output, standard-input, index, selection, or atomic-write
contract, caches complete FASTA contigs, directly creates output, and treats
malformed or truncated records and missing reference data as counted
pass-through cases. Its genotype rewriter handles only one separator and does
not support general ploidy. On REF/ALT exchange it rewrites only GT, leaving
allele-indexed INFO and FORMAT values stale. The verbose audit narration,
crate-wide lint allowances, duplicated CLI, and custom parser are not merged.

The README's 3.93-times claim is rejected. It records no input or binary hash,
raw trial ledger, peak RSS, output-equivalence digest, or complete command
provenance. The Criterion target times only the historical executable on a tiny
fixture, not the stated 500,000-site comparison. The README also claims a
single-byte indexed seek per record while the measured implementation loads
whole contigs. No performance seed survives beyond the idea of avoiding a
per-record reference allocation.

## Audited bcftools 1.24 divergences

Live probes and source inspection establish the following target corrections:

| Case | bcftools 1.24 | rsomics contract |
|---|---|---|
| reference contig absent | warns, exits zero, and drops every record on the contig | fail with record and contig context |
| no eligible denominator | prints `nan%` | emit a typed missing percentage in reports |
| triploid `0/1/1` REF/ALT exchange | emits `1/0/0`; leaves Number=A/R/G values unchanged | remap every representable allele-indexed value and reject or explicitly remove the rest |
| TOP ambiguity at the final reference base | prints a `FIXME` diagnostic and exits 255 | bounded-memory context scan ends as a typed unresolved result |
| ID source relocates records | warns when output becomes unsorted but exits zero | require explicit relocation and restore coordinate order transactionally |
| conflicting duplicate ID source rows | keeps the first row despite a source comment saying ambiguous IDs are skipped | fail before producing output |
| `ref-alt` or `swap` | changes allele labels without changing genotype indices | mode is not exposed |

The retained probe corpus is
`/Volumes/KIOXIA/Developments/tmp/rsomics-vcf-fixref-audit-20260819`.
It is dossier evidence, not a tracked release fixture.

## Command tree

The canonical interface is:

```text
rsomics-vcf fixref inspect [OPTIONS] [INPUT]
rsomics-vcf fixref repair --mode MODE [OPTIONS] [INPUT]
```

`inspect` never writes a variant stream. It produces a stable text report by
default and JSON with `--format json`. `repair` always requires an explicit
mode: `top`, `id`, `flip`, or `flip-all`. There is no repair default and no
option spelling that can silently select an allele-changing mode.

Every leaf uses `rsomics-help`. Family help explains the difference between
wrong reference build, representation mismatch, strand convention, and
allele exchange before listing commands. Repair help identifies which modes
can change GT, other allele-indexed values, position, and order. Examples show
inspection before repair and validation after repair.

## Shared input and output contract

Both leaves accept plain VCF, BGZF VCF, raw BCF, BGZF BCF, and standard input
by content. They share the product's typed header and record validation,
`--include` or `--exclude` selection, indexed regions and streaming targets,
and reference contig-alias policy. The reference must be indexed and random
access remains bounded through `rsomics-seqio::IndexedFasta`.

Repair accepts `-O v|z|b|u`, bounded BGZF workers, and optional TBI or CSI for
a compatible named output. Variant output and index commit together. Standard
output cannot request an index. If a two-pass mode receives standard input,
the validated stream is spooled under the configured external temporary
directory with a declared byte budget; it is never retained in memory.

Selected-out records are inspected only when `--report-all` is requested and
otherwise pass through repair unchanged. Selection cannot hide malformed
headers, records, or genotype values. Input order is checked before repair.
Broken pipes remain distinct from invalid input and reference failure.

Missing reference contigs, out-of-range coordinates, invalid bases, conflicting
header declarations, truncated input, and typed cardinality failures are fatal
by default. `--on-unresolved keep|drop` is an explicit repair policy for valid
but scientifically unresolved SNPs; the default is `error`. It never converts
I/O or malformed-data failures into keep or drop.

## Inspection contract

`inspect` classifies every selected record without mutating it:

- eligible biallelic A/C/G/T SNP or skipped record type;
- REF match, ALT match, reverse-complement REF match, reverse-complement ALT
  match, ambiguous pair, or unresolved;
- TOP-compatible, BOT-compatible, or convention-uninformative;
- action required: none, complement, exchange, complement plus exchange,
  contextual resolution, or external ID evidence;
- absent reference, invalid coordinate, field-cardinality, and source errors.

The report contains total and eligible counts, each substitution class,
reference matches and mismatches, candidate actions, ambiguous and unresolved
counts, skipped-type reasons, per-contig counts, selected-region provenance,
input and reference dictionary fingerprints, and tool/oracle versions. A zero
denominator is serialized as missing in text and `null` in JSON, never NaN or
infinity. Human percentages are derived presentation, not merge state.

Inspection is read-only and can run in one pass. It does not claim that a
majority convention proves the reference build. A report states that build
identity and sample provenance must be verified before repair.

## Repair modes

### `flip`

`flip` applies the four reference-comparison actions only to unambiguous allele
pairs. A/T and C/G records are unresolved because reference equality alone
cannot identify strand. Complement-only actions preserve allele indices;
exchange actions use the complete typed remapping contract below.

### `flip-all`

`flip-all` extends the same direct reference comparison to A/T and C/G pairs.
The explicit mode name is the risk acknowledgement: help and the completion
description state that reference matching alone cannot prove the biological
orientation of ambiguous SNPs. Every ambiguous action is counted separately
and recorded in the output provenance.

### `top`

`top` converts Illumina TOP convention to forward reference convention.
Unambiguous pairs follow the published complement table. For an ambiguous SNP,
the resolver walks equal distances upstream and downstream until the first
informative base pair, reading bounded reference windows rather than imposing
bcftools's arbitrary 100-base limit or materializing a contig. A contig edge,
masked reference, or no informative context yields unresolved and follows the
explicit unresolved policy.

Context lookup validates every fetched interval and terminates at reference
boundaries. Unit fixtures cover both contig edges, long homogeneous context,
masked bases, palindromic pairs, and the exact first-informative tie rule.

### `id`

`id` requires `--id-source FILE`, an indexed VCF or BCF whose contig dictionary
and declared reference identity match the repair reference. The authoritative
row supplies expected forward REF and, when requested, position. Missing IDs
are unresolved. Identical duplicate rows collapse; any duplicate that differs
in contig, position, REF, or ALT is a fatal ambiguous-source error.

Position changes require `--relocate`; otherwise a source/input position
mismatch is unresolved. ID lookup uses bounded spill-and-merge runs keyed by
contig and ID rather than a whole-chromosome hash table. Non-relocating output
is restored to input ordinal. Relocating output passes through the product's
bounded stable external sorter and is committed only after coordinate order
and optional index validation succeed. The summary records relocated records,
duplicate collapses, missing IDs, spill bytes, and sort runs.

## Typed allele transformation

The internal action model separates base complement from allele permutation.
Complement changes allele strings but leaves allele indices unchanged. A
REF/ALT exchange applies the old-to-new permutation to every representable
field:

- GT supports arbitrary checked ploidy, preserves allele order, phase and
  missing alleles, and maps every index;
- Number=R values exchange REF and ALT entries;
- Number=G FORMAT values use the sample's checked ploidy and standard VCF
  genotype ordering;
- known derivable Number=A fields such as AC and AF are recomputed from valid
  genotype state when requested by the field policy;
- output REF must equal the indexed reference after the action.

A general Number=A value for the old ALT cannot be converted into a value for
the new ALT, because that would require an absent old-REF value. INFO Number=G
without an unambiguous ploidy has the same problem. The default
`--allele-field-policy error` names every unrepresentable field before output.
`--allele-field-policy drop` removes those declared fields and reports them;
it never silently preserves stale values. Number=. fields are not guessed to
be allele-indexed. The output header records every dropped field and the
chosen policy.

All cardinalities, allele indices, integer ranges, float values, and genotype
combinations validate before mutation. An output record is built separately
and replaces the input only after every affected field succeeds. This private
`AllelePermutation` implementation is shared by `norm`, `fixref`, `split`, and
allele-aware annotation paths inside `rsomics-vcf`; it is not a public crate.

Each repaired record receives a checked INFO action tag, `FIXREF` by default,
with canonical values `none`, `complement`, `exchange`,
`complement-exchange`, `top-context`, `id`, `relocated`, or `unresolved`.
An existing tag must have the compatible String/variable declaration.
`--tag-name` changes the name after the same validation.

## Deliberate exclusions

The upstream `ref-alt` and `swap` modes are excluded because their contract
intentionally changes allele labels without remapping GT or other
allele-indexed fields. A safe complete repair operation does not reproduce a
known internally inconsistent record merely for flag parity.

There is no automatic reference-build guess, majority-vote repair, hidden
record discard, dynamic plugin ABI, warning-only unsorted output, arbitrary
TOP context cutoff, whole-reference cache, or in-place file mutation. Indels,
multiallelic records, symbolic alleles, breakends, gVCF blocks, and non-ACGT
alleles are inspected and counted but not repaired by this operation.

## Foundation decisions

No new public foundation or public item is added.

- `rsomics-seqio::IndexedFasta` supplies bounded checked reference access for
  the existing consensus and fixref consumers.
- `rsomics-common::AtomicFile::commit_all` supplies the variant-plus-index
  transaction. The existing external-sort review covers the ID relocation
  consumer after VCF sort and BAM sort demonstrate the same generic contract.
- `rsomics-help` supplies the nested family, safety notes, examples, exit
  semantics, and machine-readable completion metadata.

VCF headers, records, allele permutations, TOP convention, ID sources, action
tags, field policies, and report schemas remain private product policy. Layer A
does not learn VCF types.

## Implementation structure

The intended product-local structure is:

```text
src/fixref.rs
src/fixref/action.rs
src/fixref/alleles.rs
src/fixref/id.rs
src/fixref/inspect.rs
src/fixref/repair.rs
src/fixref/report.rs
src/fixref/top.rs
src/commands/fixref.rs
tests/fixref.rs
tests/compat/fixref.rs
```

The sequence is test-first:

1. extract and extend the private typed allele-permutation tests from current
   `norm` behavior;
2. implement inspection and stable text/JSON reports;
3. implement `flip` and `flip-all` over the checked transformation;
4. implement bounded TOP context resolution;
5. implement unique ID joining, optional relocation, and stable external sort;
6. add all encodings, selections, transactions, indexes, help, and completion;
7. run differential, adversarial, performance, and four-native release gates.

Historical modules are not copied wholesale. Useful cases are rewritten as
typed product tests before their corresponding implementation is admitted.

## Compatibility matrix

Differential tests pin bcftools/HTSlib 1.24 and compare typed records after
removing only tool provenance:

| Case | Compatibility target |
|---|---|
| inspect valid biallelic SNP substitutions | same sound counts and classes |
| `flip` and `flip-all` with GT-only diploid records | same REF, ALT, GT, and actions |
| TOP with informative context inside the upstream window | same repaired records |
| ID with unique same-position source rows | same repaired records |
| skipped valid record types under keep policy | same unchanged records and counts |
| all four VCF/BCF encodings | typed-equivalent output |

Deliberate divergence tests assert fail-loud or corrected output for missing
reference contigs, zero denominators, contig-edge TOP sites, conflicting IDs,
ID-induced order regression, arbitrary-ploidy GT, Number=A/R/G fields,
truncated streams, and upstream allele-only modes. The live probe expectations
are recorded alongside the tests, not normalized into fake parity.

Property tests generate biallelic alleles, arbitrary ploidies, phase patterns,
missing alleles, R/G vectors, complements, and exchange permutations. Applying
the inverse action must restore every losslessly representable field. Invalid
cardinality and allele indices must fail without a partially changed record.

## Performance evidence

The release benchmark uses at least two representative inputs on external
storage:

- millions of biallelic SNPs across many contigs in plain and BGZF VCF and
  BCF, including mixed direct actions and indexed reference access;
- TOP and ID workloads with ambiguous context, a large ID source, forced spill,
  optional relocation, and output sorting.

Each ledger records source and binary revisions, complete commands, machine,
reference and input hashes, encoding, compression and worker settings, warmup,
alternating raw rounds, timing distribution, peak RSS, output semantic digest,
spill bytes, and temporary storage. Inspection and direct flip paths compare
with bcftools 1.24. ID relocation also compares the explicit bcftools-plus-sort
workflow. The principal direct-repair hot path must show a strict throughput or
resource-use win; equal performance without another measured material benefit
does not pass.

## Release gate

Release 0.16 is eligible only when:

- every declared inspect and repair mode has typed unit, integration,
  adversarial, and differential coverage;
- the old fixture seeds have dispositions and no historical parser or whole-
  contig cache remains;
- allele permutation covers arbitrary GT ploidy plus representable R/G values,
  and unrepresentable A/G cases fail or drop only by explicit policy;
- missing reference, TOP boundary, duplicate ID, relocation order, malformed
  record, output, index, and transaction failures are tested;
- text and JSON inspection reports contain no NaN or infinity;
- formatting, strict Clippy, tests, package verification, and the performance
  decision pass on external storage;
- exact-head native CI passes Linux and macOS on x86_64 and aarch64;
- the public command, help, API, and production hot paths receive a fresh
  review before publication.

Primary upstream references are the
[bcftools fixref guide](https://samtools.github.io/bcftools/howtos/plugin.fixref.html),
the [bcftools 1.24 fixref source](https://github.com/samtools/bcftools/blob/1.24/plugins/fixref.c),
and the [VCF and BCF specifications](https://github.com/samtools/hts-specs).
