# inferCNV ingestion source audit — future work, unaccepted

Read-only preparation after the current matched-performance task. This note
does not start implementation, expand the current plan, accept raw-input
compatibility, or revive the retired micro-crate. No local R/Cargo/build,
install, network fetch, deletion or Git mutation was performed.

## Sources and ownership

Pinned upstream: infercnv 1.28.0, commit
`b421d9405c97a309b081ef86d455e976df93eae4`, retained at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-shipped-2026-09-26/run-36221554790/artifact/infercnv-b421d9405c97a309b081ef86d455e976df93eae4/`.
All upstream line references below are to `R/inferCNV.R` unless stated otherwise.
Its previously verified SHA-256 is
`2b25f5fdf9665f382073255e44729ff38cca32f3862c158f91ebf58d43e475a2`.

Historical asset inspected:
`/Volumes/KIOXIA/Documents/omics-rust/rsomics-infercnv/`.
Dossier `docs/10-products/sc.md:225` records historical revision
`393882653ea27e65f38ec6997f8f26e91d04927b`; this turn did not inspect Git state.
Current inspected file hashes:

| Asset file | SHA-256 |
|---|---|
| src/lib.rs | 5b7dde156140ce4a5a8755ad1613b9631cea3f6a6990b1da1f155ff97e1cb091 |
| src/cli.rs | 983591a78e1243495802165f0c6302f276d4021138b4a0dbaf4d06e1eb63deeb |
| tests/compat.rs | 47ff9428026614467766f8b8aac18d928f9fce1ed2c949996d4a7be36b1006bd |
| benches/bench.rs | ba8546f963d3eb8d2f95df9b0d27875afe83ba19d0a3c627ddd5f12d9981a09e |

The dossier places expression-derived CNV inside rsomics-sc, not a separate
product (`sc.md:113`). It explicitly classifies this old asset as parser/fixture
material only, rejects the approximation, and prohibits another product
dependency or resurrected micro-crate (`sc.md:225,275–279,428–439`).

## Actual raw-input → stage-1 sequence

1. **Read counts, preserving supplied labels.** Character input chooses gzip
   by `.gz`, serialized R object by `.rds`, otherwise text. Text calls
   read.table(header=TRUE,row.names=1,check.names=FALSE,sep=delim), then as.matrix
   (143–157). Existing matrix/dgCMatrix is used directly; data.frame becomes a
   matrix (158–164). This is not an integer-UMI parser. Reader defaults such as
   quoting/comments/NA conversion are inherited from base R, not explicitly
   disabled here. Text dialect and in-memory-object behavior need distinct
   witnesses; one cannot infer all of them from the canonical test TSV reader.
2. **Read four-field genomic positions.** Filename input is headerless,
   tab-separated, first field used as row names; remaining columns are named
   chr/start/stop (168–178). Its delimiter is fixed tab even when counts and
   annotations use another delim. This is not GTF/GFF. Exclude exact chromosome
   labels from the supplied set, default chrX/chrY/chrM (179–180).
3. **Read annotations and check annotation→matrix membership.** Filename input
   is headerless, uses delim and two character columns with cell row names
   (183–193). A first row named literally V1 is removed (195–198), even if that
   were a real cell; do not silently generalize this to arbitrary headers.
   Every remaining annotated cell must occur in the expression columns or the
   function explicitly stops (200–210). The reverse condition is not required:
   unannotated matrix columns are removed later, not an immediate error.
4. **Reduce and order genes.** .order_reduce removes coordinate records whose
   start+stop equals zero (359–365), then intersects expression/position row
   names and uses match to pick their rows (377–385). This predicate is literally
   a sum, not a general coordinate validator. Chromosome rank is first occurrence
   in the remaining genomic-position table, including unmatched entries, not
   lexical or natural chromosome order (400–403). Sort by that factor, start,
   then stop, and apply the identical permutation to both inputs (405–413).
   Intersect/match is not a many-to-many join or gene-version normalization.
   Ties and duplicates require an oracle witness; do not invent a gene-ID
   tie-breaker absent from source.
5. **Filter columns by reduced-gene totals.** NULL count limits become (1,Inf),
   default is (100,Inf), and effective minimum is max(1,user_min)
   (133–140,236–242). colSums runs after chromosome/gene reduction; both minimum
   and maximum comparisons are inclusive (244–246). Thus counts on excluded or
   unmatched genes cannot rescue a low-count cell. Annotation rows are then
   restricted to retained matrix columns (256–258).
6. **Resolve reference availability; optionally sample.** Requested reference
   labels are intersected with surviving annotation labels while preserving
   request order (260–261). The changed-reference warning branch has a source
   hazard described below. A non-NULL max_cells_per_group splits annotations by
   group, randomly samples groups above the cap and recombines them (269–281).
   This is RNG-dependent, not a deterministic first-N cap.
7. **Restrict cells and rebuild indices.** Membership filtering retains matrix
   column order despite annotation/sampling order; annotation rows are reordered
   to it (285–288). Reference-map names follow requested surviving-reference
   order; which() yields ascending one-based column indices (291–300).
   Observation-map names are sort(setdiff(unique(groups),refs)), not matrix
   first-occurrence order; each vector also uses which() (302–312). Record R
   collation for arbitrary group names; the fixed shipped names alone do not
   establish locale-independent ordering for all inputs.
8. **Construct state.** expr.data=count.data=the prepared raw matrix, genomic
   rows aligned, both group maps present, tumor_subclusters=NULL, .hspike=NULL,
   creation arguments plus count digest recorded (320–332). The validator checks
   expression/gene-order row-name agreement, not a comprehensive numeric/schema
   contract (471–477). Stage 1 is saved before stage-2 mean/detection filtering
   (`R/inferCNV_ops.R:538–564`); do not move cutoff=1/min_cells=3 into creation.

## Duplicate/error behavior: do not infer friendly policy

- Text readers delegate duplicate row-name rejection and malformed-field
  handling to base read.table. Base-R source was not inspected here: exact
  diagnostics and header/quote/NA behavior remain real-oracle witness work.
  In-memory matrices can bypass those text-reader constraints. Intersect removes
  repeated labels and match selects first matches (377–385), so native strict
  duplicate rejection is not automatically identical to every R object path.
- Count columns use check.names=FALSE and creation has no explicit duplicate
  cell-name guard. Repeated cell labels can interact with membership filtering
  and annotation indexing; do not claim either clean rejection or safe support
  without execution. Duplicate reference names similarly are not checked before
  list assignment by name (293–299).
- Reference removal is not safely documented as warning-and-continue: line 262
  uses `if (! all.equal(ref_group_names, orig_ref_group_names))`. On disagreement
  all.equal can return a descriptive character vector, making unary ! fail.
  This source hazard needs a witness for absent refs and count-filtered refs.
  Do not repair it invisibly while calling the result exact upstream behavior.
- Singleton dimensions can collapse: subsetting at 256 and 285 omits
  drop=FALSE. Zero/one retained gene/cell cases need actual error/output evidence.
  An intended no-common-gene error appears at 229–233, but earlier operations
  (215–227) use empty reduction results and can fail first. Preserve actual
  failure evidence, not just the intended message.
- Creation has no explicit finite/nonnegative-expression check, nor explicit
  start<=stop check. colSums/which and later R operations decide malformed-data
  outcomes. The native PreparedCounts boundary is intentionally stricter:
  unique nonempty IDs/groups, u64 valid ordered coordinates, finite nonnegative
  values and positive finite column totals (`rsomics-sc/src/cnv/model.rs:61–137`).
  Future ingestion must state supported-domain/error policy before claiming
  compatibility; source quirks are not permission to weaken that boundary.

## Decimal conversion boundary

Fractional counts are real upstream input, not synthetic convenience. The
accepted shipped parser audit records five occurrences of raw literal 0.076439
where R 4.6.1 and Python conversion differ by one adjacent f64 value; canonical
17-digit text bridges those fixed bytes. See
`infercnv-shipped-parser-audit-2026-09-26.md` for exact literals, hashes and scope.
The accepted native core consumes R-canonical prepared inputs. This does not
accept arbitrary raw Rust decimal parsing as R-equivalent, a universal one-ULP
rule, or general read.table emulation. Future raw-input witnesses must include
that known literal, exponent notation, exact filter-boundary totals and output
round trips. Do not introduce gene-specific substitutions or widen downstream
tolerances to hide conversion differences.

## Historical parser assets: actual reuse assessment

| Code | Observed behavior and disposition |
|---|---|
| lib.rs:18–42, GenePos at 10–16 | GTF-like nine-column gene records, no stop field retained, malformed start→0, missing name→synthetic name, lexical chromosome/start sort. Not the upstream gene-order parser. At most attribute-tokenization/test material for a separately declared GTF import later; no direct merge into CreateInfercnvObject compatibility. |
| lib.rs:45–58 | Small attribute splitter accepting space/= syntax and quoted text; optional internal refactor asset only if a concrete GTF conversion is later in scope, not a reason to add it now. |
| lib.rs:63–93 | Buffered TSV line-reading skeleton with a first header field always discarded; row-major Vec<Vec<f64>>, malformed number→0, no width/unique-ID/finite checks, no gzip branch. No direct reuse as an accepted parser. Recover explicit tiny fixtures/error-plumbing patterns only; replace parsing policy and layout if reused. |
| cli.rs:52–67 | Single flat barcode list selects matching columns, silently ignores unknown names; has no annotation table or multiple named reference groups. Not reusable reference-membership semantics. |
| lib.rs:110–139 | BTreeMap overwrites duplicate position names, orders by old GTF indices, and fills absent/ragged expression cells with zero. Discard as ingestion semantics along with the approximation. |
| lib.rs:177–202 | Buffered output/error propagation is generic scaffolding; four-decimal formatting is not adequate for exact canonical counts or current goldens. |
| tests/golden/* | Five genes ×five cells, two chromosomes; useful clearly labeled hand fixture seed, not an upstream golden. GTF and normals.txt must not masquerade as upstream input dialects. |
| tests/compat.rs:11–52; smoke.rs | Self-relative signal/CLI checks, no actual R invocation; retain only relevant test ideas. README's black-box compatibility assertion is not supported by this inspected test. |
| benches/bench.rs:7–25 | Times the tiny old CLI, even supplies --normal-cells while cli.rs:32 exposes --ref-cells. Not accepted ingestion or performance evidence. No rerun attempted. |

The dossier's parser-asset classification is a ceiling on reuse, not an
endorsement of the permissive implementations. No production parser above is
a compatible drop-in.

## Focused real-oracle witnesses needed before a next-slice claim

Use the pinned installed package on hosted/remote infrastructure, preserving
raw input bytes, runtime/locale, explicit creation arguments, returned slots or
actual errors/warnings. These are future witnesses, not completed tests:

1. Non-square fractional matrix; permuted annotations; extra unannotated matrix
   cell (dropped) versus extra annotation cell (error); punctuation-bearing IDs.
2. Shuffled chromosomes chr2/chr10/chr1, mismatched gene sets, same-start different-
   stop records and complete coordinate ties. Verify exact gene order and values.
3. chrX/Y/M exclusion and chr_exclude=NULL; (0,0) coordinates; negative, reversed
   and sum-zero nonzero coordinates as explicit malformed-domain probes.
4. Column totals immediately below/equal/above both count limits, with counts
   removed by gene/chromosome filtering; default versus NULL versus min<1.
5. Multiple ordered reference groups and observations whose sorted-name order
   differs from matrix blocks; empty refs; absent requested ref; reference lost
   to count filtering; duplicate requested refs. Record actual branch behavior.
6. Duplicate genes/cells/annotations/positions through files and, separately,
   in-memory inputs; ragged/blank/malformed/NA/Inf/negative fields; zero/one-gene
   and zero/one-cell outcomes. Avoid assuming all paths reject identically.
7. Canonical gene-headed versus upstream row-name-header text, gzip/plain forms,
   custom delimiter, quoting/comment behavior and a real first cell named V1.
8. Non-NULL per-group cap with a fixed documented R seed and reordered input;
   capture selected identities. Leave RNG compatibility explicitly unaccepted
   if the future slice excludes sampling rather than pretending deterministic
   first-N is equivalent.
9. Known decimal parser-bridge literal and threshold-sensitive decimals; assess
   exact stage-1 values before any downstream algorithm comparison.

## Internal versus shared ownership

Keep count/coordinate/annotation dialect parsing, gene/cell joins, chromosome
ranking, group-map construction, filtering order, decimal policy and prepared-
state assembly inside rsomics-sc. They implement this product's contracts; a
second concrete product consumer has not been identified. Feed the accepted
PreparedCounts model rather than altering its numerical pipeline to compensate
for parsing. No new public crate or foundation API is justified by this audit.

Reuse existing rsomics-common errors/exit/reporting/resource/path-publication
facilities where they actually exist and match the contract, without claiming
its dossier responsibilities are all already implemented (`sc.md:315–317`).
rsomics-help owns future coherent product command/help presentation, not data
joins or R compatibility (`sc.md:318–320`). No CLI or help advertisement is
proposed for this future slice, and no dependency on retired rsomics-infercnv
should be introduced. All witnesses, policy choices, implementation and
acceptance remain future work.
