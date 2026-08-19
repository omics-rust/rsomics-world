# rsomics-vcf consensus design

Status: product boundary, bcftools 1.24 behavior, historical assets, streaming
reference model, allele and mask semantics, chain output, transaction policy,
compatibility oracle, and release gates are defined. The target release is
0.12.0 after the complete `concat`, `merge`, `isec`, `sort`, and `split`
slices.

## Product boundary

`consensus` applies selected typed VCF or BCF alleles to one FASTA reference
and emits the resulting FASTA. It owns sample and haplotype selection,
reference matching, masks, absent-site replacement, variant marking, overlap
policy, and optional coordinate-chain output.

This is a subcommand of `rsomics-vcf` because the difficult policy is the
interpretation of typed VCF alleles and genotypes. It is not a second sequence
product. `rsomics-seq consensus` retains alignment- or sequence-derived
consensus, and alignment-read consensus remains in the BAM product. No Layer B
product depends on another Layer B product.

The command does not call variants, phase genotypes, normalize records, build
reference indexes, or infer a reference assembly. Users perform those distinct
operations explicitly. It does not promise arbitrary symbolic-allele
interpretation without enough VCF data to construct a sequence.

## Upstream and format authority

The compatibility oracle is bcftools 1.24 `consensus`. VCF 4.5 and BCF2 define
variant representation, HTSlib 1.24 supplies the indexed VCF and BCF behavior,
FASTA and FAI conventions define reference access, and the UCSC chain format
defines coordinate mapping.

The audited bcftools tag is revision
`fb9f0f783e0f67d734f6fa7fe4df9d230522f196`. Its MIT-licensed
`consensus.c` has SHA-256
`e856d6228d8445833a5dd155f66a8d8393993266802eca7b44854eadcaacf751`.
Implementation may reproduce the documented behavior and black-box results,
but bcftools or HTSlib source is not copied.

The declared upstream surface includes:

- reference FASTA input and VCF or BCF variant input;
- include or exclude expressions and partial-reference overlap selection;
- one or more selected samples and one-sample haplotype selection;
- genotype-position, REF, ALT, indel-length, and IUPAC allele choices;
- missing-genotype and absent-variant replacement;
- repeated masks with replacement by a character, lower case, or upper case;
- independent insertion, deletion, and SNV marking;
- FASTA header prefixing and UCSC chain output.

The target retains this complete useful surface. Upstream compression-level
spelling, verbosity values, and implicit acceptance of extra positional
arguments are excluded under the product-wide CLI policy.

## Historical asset

The retired `rsomics-vcf-consensus` repository is retained at revision
`bb016cf71d28ffa12562e875e0c5db7a431d148c`, version 0.1.1. Its production
path reads the complete FASTA and VCF into memory, accepts only VCF text or
gzip, chooses only the first ALT, and ignores sample genotypes, FILTER state,
expressions, masks, absent sites, marks, regions, symbolic alleles, and chain
output.

It does not validate REF against the reference, represent typed alleles, bound
contig memory, support BCF or indexes, or make named output transactional.
Overlapping or out-of-range records are silently skipped. Its applied counter
includes records assigned to a contig even when no allele was applied. JSON
mode discards the FASTA stream. The compatibility harness can skip its oracle,
labels its committed golden as bcftools 1.13, and the benchmark measures a tiny
process launch.

The source classification is:

1. direct merge: none;
2. refactor then merge: none;
3. test, fixture, or benchmark asset only: the compact reference, variant, and
   expected-FASTA fixture plus the golden as a narrow regression seed;
4. discard: the production parser and mutation engine, whole-file ownership,
   standalone CLI and help, direct output path, counters, skip-capable harness,
   benchmark, and narrated source.

The audit reran the exact fixture with bcftools and HTSlib 1.24. Its FASTA is
byte-identical to the committed golden at SHA-256
`da7c8e877b9a753af2ce3072bce3b56a828b5c9b98f95e757dee397c3cd9a4ce`.
That confirms the three-record regression, not the missing command surface.

## Command surface

```text
rsomics-vcf consensus [OPTIONS] [INPUT]
```

`INPUT` is the VCF or BCF stream and defaults to standard input.
`-f, --fasta-ref FASTA` is required. At most one of the variant stream and
reference may be standard input. The command writes FASTA to standard output
unless `-o, --output FILE` is supplied. It is rendered through `rsomics-help`
and uses the product-wide expression, thread, JSON, quiet, diagnostic, and exit
conventions.

The stable options include:

- `--include EXPR` or `--exclude EXPR`, never both;
- `--samples LIST` or `--samples-file FILE`, never both;
- `--haplotype MODE` for one selected sample;
- `--iupac-codes` for allele-set IUPAC output;
- `--missing CHAR` for missing selected genotypes;
- `--absent CHAR` for reference positions without an applied record;
- repeated `--mask FILE`, each optionally followed by
  `--mask-with CHAR|lower|upper`;
- `--mark-snv CHAR|lower|upper`, `--mark-ins CHAR|lower|upper`, and
  `--mark-del CHAR`;
- `--prefix STRING` for complete FASTA record headers;
- `--chain FILE` for a UCSC chain matching the emitted sequence;
- `--regions-overlap position|record|variant` for partial-reference bounds;
- `--overlap error|first` for overlapping selected variants.

`--mask-with` binds the most recent mask and defaults to `N` for each new
mask. Each mask may therefore have its own replacement policy.

Replacement characters are exactly one Unicode-independent ASCII byte allowed
by the sequence policy. `lower` and `upper` are named modes, not multibyte
characters. Empty, multibyte, and multi-character values fail during argument
validation.

Numeric compression levels, automatic reference indexing, automatic output
compression inferred from a suffix, arbitrary HTSlib options, provenance
header stamping, and additional positional inputs are excluded. The output is
FASTA text; gzip output can be introduced only with an explicit product-wide
sequence-output contract and performance evidence.

## Sample and allele contract

Sample selection and allele choice are explicit because the opening bcftools
1.24 help text and its live multi-sample default disagree.

With no sample option, rsomics ignores FORMAT/GT and applies the first ALT.
`--samples -` spells the same policy explicitly. This follows the documented
default and makes output independent of the number or order of samples in a
file. It deliberately does not copy the live 1.24 behavior that emits an IUPAC
union across every sample in a multi-sample VCF.

With selected samples and no haplotype mode, the command emits the IUPAC union
of the alleles present in the selected genotypes. Missing alleles are ignored
when another selected allele is known and use `--missing` when no selected
allele is known. `--iupac-codes` without selected samples emits the IUPAC union
of REF and every concrete ALT.

`--haplotype` requires exactly one selected sample, or an input containing
exactly one sample when no sample option is given. Its stable modes are:

- a positive genotype allele position, with an optional phase-sensitive
  position;
- `ref` or `alt`;
- `longer-ref`, `longer-alt`, `shorter-ref`, or `shorter-alt`;
- an IUPAC genotype-position mode where the chosen genotype allele is
  ambiguous.

The implementation preserves mixed ploidy, phasing, missing alleles, allele
indices, and the distinction between a missing genotype and a reference
genotype. A chosen allele index outside the record ALT vector fails.

Concrete SNVs, MNVs, insertions, deletions, and replacements are supported.
`<DEL>` is supported only when a valid END or equivalent declared deletion
span identifies the removed reference sequence. `<*>` and `<NON_REF>` are
nonsequence alleles and do not replace reference bases. Other symbolic alleles
and breakends fail when selected unless an earlier expression or region policy
excludes them. They are never rendered literally into FASTA.

INFO/AD or other annotations do not choose alleles. The genotype contract is
FORMAT/GT only. Selection by quality or annotation belongs in the existing
typed expression engine and is evaluated before allele application.

## Reference and variant streaming model

Plain, gzip, or BGZF FASTA is accepted. The complete FASTA header is preserved
for output, while the first ASCII-whitespace-delimited token identifies the
reference sequence. Duplicate identifiers fail. Sequence whitespace is
removed by the FASTA decoder; invalid sequence bytes retain input context in
the error.

A reference header may name a slice as `name:start-end`, using one-based
inclusive coordinates. The decoded sequence length must equal the declared
span. Variants are translated into that slice and records outside it are
reported as out of scope rather than treated as reference mismatches. Full
reference records start at coordinate one.

The primary path streams one reference record and one compatible ordered
variant cursor at a time. It buffers only the current FASTA chunk, pending
record, mask intervals overlapping the current position, and a bounded output
line. It does not allocate a whole chromosome or whole VCF. Output bases before
a selected variant are copied as chunks; replacement alleles are emitted only
after the complete REF span has been checked.

The variant input accepts plain VCF, BGZF VCF, raw BCF 2.2, and BGZF BCF 2.2.
An indexed input may be queried in FASTA record order. An unindexed input or
standard input uses a checked streaming path and must be sorted by header
contig rank and coordinate. Contig blocks cannot recur. When the reference is
standard input, variants must be indexed because the reference order is not
known before it is consumed. At most one input can require standard input.

Records on reference contigs not supplied by a deliberately partial FASTA are
out of scope. A selected record starting beyond the end of a supplied full
reference record fails. A selected REF span extending past the record or slice
fails. Every applied REF must match the reference byte-for-byte under the VCF
case rule; mismatch is a nonzero error.

Overlapping selected records fail by default. `--overlap first` retains the
first applicable record in input order, emits one contextual warning for each
skipped overlap, and accounts for it separately. Sorting does not resolve
overlap semantics and is never performed implicitly.

## Masks, absent sites, and marks

Mask inputs are coordinate files using the product's explicit BED or generic
tabular interval parser. Coordinate convention is selected or inferred only
from a declared input profile, never from numeric values. Repeated masks are
kept as ordered streaming cursors, so a later mask can transform a base already
handled by an earlier mask without loading either file in memory. Unknown
contigs, reversed intervals, overflow, malformed rows, and coordinate
regression fail.

Processing order is stable and matches the useful bcftools model:

1. masks replace or case-convert reference spans; character replacement
   suppresses overlapping variants, while case-only conversion does not;
2. `--absent` replaces unmasked reference positions that receive no applied
   variant;
3. SNV, insertion, and deletion marks transform the already selected allele.

A masked variant is counted as masked rather than absent or overlap-skipped.
Insertion marks affect inserted bases, SNV marks affect the emitted alternate
bases, and deletion marks replace deleted reference bases and therefore remove
the coordinate-length change. Case conversion preserves nonalphabetic bytes.

Mask parsing and interval policy remain private to `rsomics-vcf` for this
slice. Existing VCF region code and annotation intervals do not by themselves
justify a new `rsomics-intervals` API. Promotion requires a second product
consumer and consumer-side tests under the foundation rule.

## FASTA and chain output

Output FASTA wraps sequence at exactly 60 bases per line and ends every record
with a newline. Input sequence case is preserved except where a mask or mark
explicitly changes it. `--prefix` prepends text to the complete original
header, not only its identifier. Empty reference records remain represented.

The result summary distinguishes reference records, reference bases, decoded
variant records, expression-selected records, applied alleles, overlap-skipped
records, masked records, out-of-scope records, absent bases, and output bases.
Counters are derived from completed actions and never infer application from a
record merely sharing a contig.

The optional UCSC chain describes the actual emitted sequence relative to the
supplied reference. It contains every FASTA record in output order, including
identity-only records, and represents insertions and deletions as chain gaps.
Partial reference headers retain their source assembly coordinates. A deletion
mark that preserves output length produces no deletion gap. Chain sizes,
starts, ends, strands, block sums, gap sums, and unique numeric IDs are
validated before commit.

The chain is an output of this operation, not a public liftover API. Its parser
and writer stay private until another product consumer proves a shared
contract.

## Output transactions

A named FASTA uses the existing `rsomics-common::AtomicFile`. A named FASTA
plus named chain stages both files and commits them through the existing
`AtomicFile::commit_all` or `write_atomic_pair` contract. Parse, reference,
expression, mask, write, flush, sync, chain-validation, or commit failures
leave both previous destinations unchanged.

When FASTA is written to standard output and chain output is named, the chain
is staged and committed only after all FASTA bytes have been written and
flushed. Standard output is inherently not rollbackable; a broken pipe fails
and removes the staged chain. JSON output requires named FASTA output so the
structured summary cannot replace or interleave with biological data.

Existing destination files may be atomically replaced under the product-wide
owned-output policy. FASTA and chain paths must be distinct and cannot alias
the input, reference, masks, or one another after path resolution.

## Foundation evolution

The existing `rsomics-seqio::IndexedFasta` contract already supplies
zero-based half-open access to indexed plain or BGZF references. It is used by
`rsomics-vcf norm` and `rsomics-call`; no second indexed-reference abstraction
is added for consensus.

Consensus creates a concrete need for a public chunked FASTA reader that does
not own an entire record. The candidate `rsomics-seqio::FastaChunkReader`
contract exposes checked record boundaries, complete headers, identifiers, and
borrowed or bounded sequence chunks over plain, gzip, and BGZF input. It owns
FASTA decoding and byte offsets, not variant application, masks, coordinates,
line wrapping, or CLI policy.

Its two named Layer B consumers and call sites are:

1. `rsomics-vcf consensus`, which copies and edits chromosome-scale reference
   streams without whole-contig allocation;
2. `rsomics-seq stats` and `rsomics-seq validate`, which currently use the
   whole-record `rsomics-seqio::Reader` even though their FASTA paths need only
   sequential chunks.

The exact Rust surface is fixed through consumer-first tests. The existing
record reader remains source-compatible and may reuse the event parser
internally. Promotion requires chunk-boundary, malformed-input, gzip/BGZF,
error-context, allocation, and both consumer tests plus an API review. The
change must not regress `rsomics-seq` or indexed-reference performance.

No new public crate is created. Consensus uses the existing
`rsomics-common` transaction API, the existing `rsomics-seqio` indexed API,
and the private VCF typed format and expression layers.

## Product structure

```text
src/
├── consensus.rs
├── consensus/
│   ├── allele.rs
│   ├── chain.rs
│   ├── mask.rs
│   ├── reference.rs
│   ├── stream.rs
│   └── write.rs
└── commands/
    └── consensus.rs
```

`consensus.rs` owns typed options, invariants, dispatch, and counters.
`allele.rs` binds sample and haplotype policy to typed records. `reference.rs`
binds full and sliced FASTA coordinates. `stream.rs` synchronizes the ordered
reference, variant, and mask cursors. `mask.rs` owns interval parsing and
priority. `write.rs` owns FASTA wrapping and grouped output. `chain.rs` builds
and validates the optional mapping.

The command adapter converts Clap values, binds expressions, uses
`rsomics-help`, selects input paths, and renders the summary. It contains no
sequence mutation engine or duplicate FASTA parser.

## Compatibility contract

The stable differential matrix covers:

- plain, gzip, and BGZF FASTA, complete and region-labelled records, and
  standard input;
- plain VCF, BGZF VCF, raw BCF, compressed BCF, indexed and checked streaming
  variants;
- SNVs, MNVs, insertions, deletions, replacements, `<DEL>`, `<*>`, and
  `<NON_REF>`;
- first-ALT, selected-sample IUPAC, explicit IUPAC, every haplotype family,
  mixed ploidy, phase, and missing genotypes;
- expressions, full and partial references, multiple and reordered contigs,
  every partial-reference overlap mode, and out-of-scope variants;
- repeated masks, every mask replacement, absent replacement, every mark, and
  their combined priority;
- overlaps, REF mismatch, invalid symbolic alleles, duplicate FASTA records,
  invalid region headers, truncated inputs, broken pipes, path aliases, and
  transaction rollback;
- byte-stable FASTA wrapping, exact counters, and chain coordinate invariants.

Semantic output is compared with bcftools 1.24 for the retained valid contract.
Where header spelling or deliberate safety policy differs, decoded sequence,
selected allele, chain mapping, diagnostic category, and exit decision are
compared explicitly rather than hiding differences through normalization.

## Deliberate fail-loud differences

Live bcftools 1.24 probes recorded behaviors that are not copied:

| Probe | bcftools 1.24 | rsomics contract |
|---|---|---|
| multi-sample input without sample options | applies an all-sample FORMAT/GT IUPAC union despite help describing first ALT | ignore genotypes and apply first ALT; sample-derived IUPAC requires selection |
| extra positional variant input | silently ignores the extra path | reject extra positional input |
| `--mark-del XX` | uses the first byte and exits 0 | require one valid replacement byte |
| overlapping selected variants | warns, skips the later record, and exits 0 | fail by default; allow explicit `--overlap first` |
| REF mismatch with named FASTA and chain outputs | exits nonzero after leaving partial artifacts | nonzero exit and grouped rollback |
| existing FASTA or chain output | truncates and replaces it during processing | stage complete outputs and atomically replace together |
| duplicate FASTA identifier | applies the same contig variants repeatedly | reject the duplicate identifier |
| `name:start-end` sequence shorter than its declared span | exits 0 | reject the inconsistent reference record |
| selected record beyond a supplied full contig | exits 0 and reports zero applied | fail with contig, position, and reference length |
| VCF contains contigs absent from a deliberately partial FASTA | exits 0 and leaves supplied records unchanged | report those records as out of scope; do not treat a partial reference as complete |

Bcftools's default overlap warning remains available only through the explicit
policy. Partial-reference support remains useful, but same-contig bounds and
declared slice lengths are invariants rather than silent omissions.

## Tests

Unit tests cover:

- sample resolution, one-sample haplotype requirements, ploidy, phase, missing
  alleles, IUPAC sets, and invalid allele indices;
- concrete and symbolic allele classification, REF spans, replacement bases,
  overlap state, and precise counters;
- FASTA header identifiers, region headers, duplicate names, whitespace,
  invalid bytes, chunk boundaries, line wrapping, and empty records;
- BED and tabular mask coordinates, ordered overlap, streaming order, unknown
  contigs, replacement-versus-case suppression, absent positions, and mark
  composition;
- chain blocks, insertions, deletions, partial coordinates, marked deletions,
  identity records, IDs, and validation;
- path alias detection, standard-input combinations, grouped transactions,
  JSON constraints, and broken pipes.

Golden and differential tests cover every input encoding, allele mode, mask
and mark combination, expression and partial-reference overlap path, complete
and partial FASTA, multi-contig order, chain output, and deliberate divergence.
Malformed tests
cover truncated compression, invalid typed records, unsorted coordinates,
recurring contigs, REF mismatch, out-of-range records, unsupported selected
alleles, malformed masks, and output failures.

Fault injection covers variant and reference decode, expression evaluation,
mask read, output write, chain write, flush, sync, grouped rename, and parent
sync. No failed named run may expose one new artifact without the other.

`rsomics-seqio` consumer tests force headers and sequences across internal
buffer boundaries, stream chromosome-scale generated records under an
allocation ceiling, and compare the record and chunk readers on identical
plain, gzip, and BGZF fixtures. `rsomics-seq stats/validate` and
`rsomics-vcf consensus` must exercise the public contract directly.

## Performance gates

Formal comparison uses pinned bcftools and HTSlib 1.24 with binary and source
hashes, machine, filesystem, input hashes, flags, warmups, alternating runs,
timing distribution, peak RSS, bytes read and written, output hashes, and
chain validation recorded.

Representative workloads include:

- a chromosome-scale plain FASTA with 5 million sorted biallelic SNVs and
  indels in indexed BGZF VCF;
- the same reference with compressed BCF and one selected diploid sample;
- multiple large contigs with repeated masks, absent replacement, and chain
  output;
- an unindexed streaming VCF path and a reference-standard-input path backed
  by indexed variants;
- dense nearby variants that stress REF checks, output wrapping, and overlap
  state without becoming an artificial all-conflict input.

The release requires a strict throughput or resource-use advantage on a
representative hot path. The primary hypotheses are bounded reference memory
and less copying through chunked FASTA streaming; neither is a claim until
measured. If indexed per-contig work is parallelized, ordering, memory,
temporary output, and thread scaling must be bounded and the serial path must
remain available. Every timed result must reproduce the semantic FASTA hash
and a valid chain.

## Release gate

Release 0.12.0 is complete only when:

- every declared sample, allele, expression, reference-overlap, mask, mark,
  variant-overlap, reference, and chain behavior is implemented without
  placeholders;
- `FastaChunkReader` has both product consumers, consumer-side tests, public
  API review, and measured allocation behavior;
- all four VCF/BCF encodings and all declared FASTA encodings pass the ordinary
  and malformed compatibility matrices;
- grouped FASTA and chain transactions pass fault injection;
- formatting, strict Clippy, unit, integration, differential, malformed-input,
  transaction, and benchmark smoke suites pass;
- the formal performance gate records a strict useful advantage;
- package contents, repository metadata, README, unified help, licenses, and
  attribution are reviewed from a clean exact head;
- native Linux and macOS CI pass on `x86_64` and `aarch64` at that exact head;
- the crate is published only after all earlier declared release slices and
  this complete consensus slice are present.

The audit fixtures and live outputs are retained outside the repository at:

```text
/Volumes/KIOXIA/Developments/tmp/rsomics-vcf-consensus-audit-20260819
```

Primary references are the
[bcftools consensus guide](https://samtools.github.io/bcftools/howtos/consensus-sequence.html),
the [bcftools 1.24 manual](https://samtools.github.io/bcftools/bcftools.html#consensus),
the [bcftools 1.24 consensus source](https://github.com/samtools/bcftools/blob/1.24/consensus.c),
the [UCSC chain format](https://genome.ucsc.edu/goldenPath/help/chain.html),
and the [VCF and BCF specifications](https://github.com/samtools/hts-specs).
