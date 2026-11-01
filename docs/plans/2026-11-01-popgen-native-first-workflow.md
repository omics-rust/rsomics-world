# Root adoption — 2026-11-01

Root accepts the complete local implementation wave below after fully reading PLAN,
SOURCE_ASSETS, REPORT and the relevant actual source bodies, and freshly verifying
all97 sealed design bodies. Original PLAN SHA2831ed43471368773661a88547c4f1846e77a96e0a5195faedbd1b0d3547a87a;
SOURCE_ASSETS a71f808cfbea2b4aac6893f77d7a91596c802495d78ce86106e90c53d34c74be;
completed505cecffba99fa1a8ff88ea16ec19ba55b8ba077f56bc603ce74caca51ea869c
at popgen-native-design-20261101.7FDlT2mI. Source oracle is already accepted in
.autopilot/oracles/popgen-source-2026-11-01.json.

Approve the connected `analyze` command and fixed canonical finite rule
`max(2^-44,2^-40*abs(source))`, exact discrete/nonfinite/legacy parsed decimal
contracts and exact native1/4 scientific payloads BEFORE any candidate output.
Freeze the full comparator and closed metadata map before the first execution.
No native-informed widening, epsilon patch, input/scope reduction or zero-fill.

The previously absent independent private owner may now be created at
/Volumes/KIOXIA/Documents/omics-rust/rsomics-popgen, publish=false. Real local
Rust implementation/build/tests/native-only full retained-cohort1/4 comparisons
are authorized through external guarded routes. Root retains final Git commits,
review, actual native-four/formal-performance/release decisions. No public GitHub
creation, publication, source runtime execution on Mac or new public foundation.
The draft's parent-approval prerequisites are satisfied by this adoption; its
provenance and unearned later gates remain unchanged.

# Popgen connected neutral-workflow implementation plan

> For agentic workers: implementation remains parent-review gated. After approval, use executing-plans or one bounded integrated implementation wave; do not turn this into a helper/test/reviewer ladder.

**Goal:** Build one functioning Rust product workflow covering the complete adopted neutral-variation and differentiation contract, using one checked input model and all original outputs.

**Architecture:** A bounded VCF parser reduces selected diploid calls directly to allele-copy and allele-heterozygote counts. A private compact count spool permits one original-VCF pass, discovers the true global allele width, and supports deterministic parallel statistics/export without retaining a human GT matrix. Canonical scikit-allel and explicitly named vcftools legacy estimators share counts, not policy or aggregation rules.

**Tech stack:** Rust2024/MSRV1.91; existing published `rsomics-common`/`rsomics-help` contracts, Clap, Rayon, flate2, serde/serde_json, SHA256 and private tempfile use. No FFI, Python runtime, Layer-B dependency, path dependency or new public foundation.

**Spec/authority:** World `docs/10-products/genotype-popgen.md` (Popgen section), `docs/plans/2026-11-01-popgen-source-preflight.md`, and the immutable original packet `/Volumes/KIOXIA/Developments/tmp/popgen-third-source-adoption-20261101.FIRTDpPx`. Its source-only adoption remains parent-owned. `READ_BINDINGS.json` here binds 41 fully read executable primary function bodies to authenticated originals; docstrings alone are omitted from reading excerpts. `SOURCE_ASSETS.md` records team-owned seeds and exclusions.

## Global constraints and scope

- No implementation, owner creation, build, scientific import/execution, Git mutation, dispatch or publication is authorized by this design packet.
- Later owner location is `/Volumes/KIOXIA/Documents/omics-rust/rsomics-popgen`; use only that independent repository after approval, never revive micro-crate repositories or change inherited locks/targets.
- All builds/downloads/cache/TMP remain verified external paths. Large human carriers stay private; no human GT/cohort data or Linux runtime is committed into a product.
- First wave covers the two-population complete connected canonical workflow and its five named legacy reports. No selection/admixture/PBS/LD, projection, scaled-SFS, haplotype or arbitrary-ploidy surface is advertised.
- Full original scope stays chr22:1–51,304,566, YRI108+CEU99, all 1,103,547 input rows. Actual canonical population is 723,783 rows, width9; 514 windows, all 176 empty/zero-accessible windows, 72 full blocks plus the exact 3,783-row tail. These counts are oracle checks, not production hard-coded cohort assumptions.
- Existing all-missing casebook and empty-window NaNs remain source behavior, not a repair option. Do not clamp negative FST, replace infinities/NaNs with zero/null, or invent invariant SFS counts.
- Rare comments explain a stable compatibility reason; no phase/history narration. Keep policy-local types/modules private rather than manufacturing public APIs.

## CLI and output contract

Recommended first complete candidate command:

```text
rsomics-popgen analyze --vcf input.vcf.gz --panel samples.tsv \
  --population YRI --population CEU --reference-fai reference.fai \
  --mask 22=chr22.mask.fa.gz --window-bp 100000 \
  --jackknife-variants 10000 --legacy-vcftools --out results \
  --threads 4 --memory-mib 512
```

`analyze` is a proposed connected-workflow command, not an extra product boundary. It implements the dossier's diversity/SFS/FST operations together. Do not add empty `diversity`, `sfs`, `fst`, selection or admixture aliases merely to reproduce a command diagram; separate views can later call the same engine if real users need them. The parent reviews this small CLI naming choice before implementation. Population labels are arguments, never YRI/CEU constants in kernels; this first profile requires exactly two disjoint, nonempty populations. Panel header uses `sample`/`pop`, tolerating additional original panel columns, with duplicate identities rejected and selected sample order inherited from VCF.

`--legacy-vcftools` explicitly requests the distinct five-table lane and is mandatory in acceptance/performance invocations. It does not invoke vcftools. Plain CLI success prints a concise output summary; `--json` emits the existing common result/error envelope, never row-level human data. Every scientific table goes to the requested directory. Use `rsomics_help::parse::<Cli>()`, `rsomics_common::{OutputArgs,ThreadArgs,run,ToolMeta,Result,Context}` and existing grouped `AtomicFile::commit_all`; do not duplicate color/error/exit handling or invent an execution-context API absent from common. Current inspected common/help manifests are0.12.3/0.4.0; parent checks published availability before later pinning.

Reject an existing output directory and input/output aliasing; stage named regular files in a fresh private directory and mark completion only after all outputs and manifest are committed. An error exits nonzero and never leaves a success manifest. Use the existing common transaction support, not another public transaction library.

Canonical scientific files retain the source logical schemas: each contig's `sites.tsv.gz`, `windows.tsv.gz`, `region.json`, `sfs.json`, `fst-jackknife.json`, `jackknife-membership.tsv.gz`, plus profile/filter/provenance/result/manifests. The legacy lane retains the five existing source table schemas/names. Runtime/producer metadata records the actual Rust executable, target, version, worker count and budget; it does not pretend to be Python metadata. A fixed metadata allowlist distinguishes these legitimate producer differences from scientific fields. Gzip headers, timestamps and compressed file hashes are not source numeric equality; each producer's own full manifest must nevertheless close exactly.

## Minimal internal interfaces and files

These are private shapes, not a proposed reusable public genotype API:

```rust
struct Selection {
    sample_ids: Vec<String>,
    original_columns: Vec<usize>,
    population_of_selected: Vec<PopulationId>,
    populations: [Population; 2],
}
enum DiploidCall { Missing, Called(AlleleIndex, AlleleIndex) }
struct PopulationCounts { copies: Vec<u32>, heterozygotes: Vec<u32> }
struct CountRow {
    ordinal: u64, contig: ContigId, position: Position1,
    alleles: Vec<String>, raw_aa: String,
    populations: [PopulationCounts; 2],
}
enum Ancestry { Matched(AlleleIndex), Missing, NonNucleotide, AlleleMismatch, Ambiguous }
struct CoordinateWindow { start: Position1, stop: Position1 }
struct VariantBlock { first_row: usize, stop_row_exclusive: usize }
```

`AlleleIndex` fits the source signed-int16 profile (maximum32767); missing is an enum, not an invalid numeric index. `copies` and `heterozygotes` share declared allele indices; a called heterozygote increments each of its two allele slots once, while a homozygote increments neither heterozygote slot. AN is the copy sum; called individuals are AN/2. Count products widen before multiplication; malformed allele indices are failures, never vector growth or missing conversion.

| File | Responsibility / narrow interface |
| --- | --- |
| `src/{lib,main,cli}.rs` | `run_cli()`, real command/help/common result handling, checked `RunConfig`; no statistic implementation |
| `src/input.rs` | Header/panel/FAI/strict-mask binding, original-order `Selection`, plain/multi-member gzip reader, streaming checked VCF records and full original stream hashes |
| `src/model.rs` | The small private types above, checked contig/coordinate/allele domains, profile and exclusion enums |
| `src/scan.rs` | `scan(config, selection, masks, spool) -> ScanSummary`; bounded ordered batches, direct copy/heterozygote counts, global ALT width and per-contig AN maxima/window row counts |
| `src/neutral/{mod,diversity,sfs,fst,windows}.rs` | `evaluate(row, contig_plan) -> SiteMetrics`; canonical kernels, spectra, inclusive windows and variant-count jackknife; shared prepared counts, no repeated readers |
| `src/legacy.rs` | Pooled complete-biallelic legacy site/window pi, legacy Tajima bins/constants and separate WC arithmetic/aggregation rules |
| `src/numeric.rs` | One product-private general-precision writer and deterministic source-shaped sum; all nonfinite/precision policy in one place |
| `src/export.rs` | Ordered full logical exports, provenance and grouped output transactions; `export(config, scan, spool) -> RunSummary` |
| `tests/neutral_workflow.rs` + `tests/fixtures/neutral/` | One connected actual casebook/CLI contract with source-bound synthetic goldens; no upstream scientific runtime required |

Do not split these further into one-file-per-number wrappers. Keep future-general types out until a second real operation requires them. Existing `rsomics-stats` gets no new API merely because a WC kernel exists; PLINK has not yet exercised an identical public FST contract.

## Exact parsing and canonical semantics

1. Match the admitted header/FORMAT cardinalities, selected GT profile and contig order. Preserve all original rows in the GT and record identity streams, including excluded rows. Only selected samples' GT values need decoding; skip irrelevant sample fields without allocating genotype vectors for all2,504 originals. Accept phased/unphased separators equally for this unphased workflow. Reject partial missing, non-diploid calls, duplicate GT/AA fields, out-of-range allele indices and malformed/unsorted coordinates. Do not import permissive old readers.
2. Apply exclusive filters in source order: not-PASS, not-literal-uppercase-SNV, inaccessible, retained. Keep multiallelic literal SNVs; preserve original ALT indices and original ordinal. Discover global ALT width over **all original records**, not only retained SNVs. Final AC/WC arrays are padded to that width, including the nontrivial source padding behavior on all-missing rows.
3. Represent project accessibility as a checked bitset with coarse rank counts and the complete original mask-code histogram, not a chromosome-length u32 prefix array. `P` alone is accessible. FAI lengths and explicitly allowed FASTA header aliases bind coordinate space. This is not personal callability.
4. Preserve raw AA. Empty/`.` is missing; uppercase only the token before the first`|`, require A/C/G/T, and match a unique REF/ALT index. Keep missing, non-nucleotide, mismatch and ambiguous reasons distinct; never default to REF. SFS unfolding requires matched index0/1 on its complete biallelic set.
5. Site MPD/Dxy use source `fill=NaN`. Pi/theta/Dxy density aggregation uses the source's explicit no-pair `fill=0` behavior, without overwriting the original site NaN. Fixed difference means **both populations called and no allele support shared**; the upstream multiallelic definition does not require each population to be monomorphic.
6. Theta and Tajima use `n=max(called AN)` on each complete **contig call**, not a window-specific maximum and not an across-contig maximum. Their harmonic/constants and operation order follow the read source. Tajima requires at least3 segregating sites; accessible denominators belong to pi/theta/Dxy/fixed density, not an extra normalization of D.
7. `CoordinateWindow` follows `position_windows` literally: starts are before stop, nonfinal stop is start+size−1, and the final stop becomes the declared contig stop when start+size≥stop. The casebook's `[21,31]` is intentionally11 bases. Generate one window array per contig and reuse it for every canonical statistic. Keep empty rows and exact record/accessibility counts.
8. Canonical WC uses source per-allele a/b/c expression ordering and `n=AN//2`, pooled heterozygote counts only over selected populations, and full allele width. Point/window components are independently `nansum(a)`, `nansum(b)`, `nansum(c)`; ignore only NaN, not infinities. Hudson independently nansums num and den. Never average site FST ratios.
9. Jackknife has its own `VariantBlock` plan. Point uses all rows. Full equal-size blocks alone enter resampling; expose the exact residual index range and all original tail ordinals. WC block denominator is `nansum(a+b+c)` **elementwise before reduction**, not the three separately summed point components. `vb` is each block ratio; `vj` recomputes the source-shaped masked leave-one-block sums, not a new total-minus-block algebra. SE uses ordinary mean/sum over vj, not nansum. One full block produces the actual NaN vj/SE; no-block inputs fail with an explicit profile error rather than fabricating uncertainty. Users can choose a smaller positive block size, as the casebook uses3.
10. Folded/unfolded1D and joint2D SFS use fixed2×population-size axes and complete-call **biallelic** rows for the relevant populations. Joint folding bins each marginal minor count independently, not a reflection of the joint unfolded grid. No projection/scaling or invented whole-genome bin0. Empty required SFS eligibility/nonexistent retained contig remains a fail-loud unsupported complete-workflow input, not a silent source repair.

## Legacy is a distinct estimator lane

Select exactly the source derivative population: canonical eligible, literal biallelic, both populations complete. Reuse each `CountRow`; do not write/reparse the700MB derivative VCF or build five additional readers. During phase1, stream-count/hash the **exact would-be derivative bytes** while the ordered original fields are available: original metadata lines normalized to LF, the source's rebuilt selected-sample header, then the original first9 fields and original selected sample fields separated by tabs/LF. Stream-hash each population-list serialization in original selected VCF order, not sorted order. Their source size/SHA/eligibility/input-GT bindings remain exact without writing another cohort carrier; native provenance must label these as `materialized=false` virtual identities, never claim those files exist. The accepted real derivative is721,170 rows, casebook10.

- Site pi pools **both populations** (414 chromosomes in the real input), not separate canonical YRI/CEU pi columns.
- Window pi uses doubled integer mismatches/comparisons, counts polymorphic derivative rows, and fills the remaining **full window size** with invariant fully called comparisons. It does not use canonical P-accessible denominators and can extend past the chromosome end. Preserve its printed `N_VARIANTS`/`N_MONOMORPHIC` definitions and omission of bins without differences.
- Legacy Tajima uses fixed pooled414 chromosomes, its original left-associated constants, `idx=(pos*(1/W)) as integer`, zero-based BIN_START, segregating p(1−p), and carries empty NaN bins after the first SNP-bearing bin. This differs from canonical inclusive coordinate windows and min_sites3.
- Legacy WC uses the original vcftools expression order, drops a whole allele contribution when any a/b/c is NaN, admits non-NaN site ratios for its weighted/mean accumulators, and emits its own nonempty/defined bins. Do not replace `!isnan` by `is_finite`, reuse canonical independent-NaN sums, or relabel canonical output as a compatibility lane.
- One private formatter handles canonical general17-significant-digit tables and legacy C++ default6-significant-digit values (5 for requested legacy log summaries). Preserve signs of infinities and defined zeros; raw NaN spelling/sign is not portable across ISAs. No PLINK formatter or other Layer-B dependency is introduced.

All five legacy reports are produced in the same complete invocation and compared against their own actual source tables. The observed source counts338/347/338 are expected consequences, not hard-coded row limits.

## Bounded memory and real parallel work

Recommended first implementation is **one original VCF pass + compact count spool + one statistics/export pass**, rather than a full cohort GT array or repeated original decompression.

- Header/panel/mask planning is small and shared. A batch is limited by both record count and byte budget; batch identity/order is independent of worker count. Start at up to8MiB raw lines/8,192 records per batch, with at most twice the selected worker count in flight; dynamically shrink from `--memory-mib`. One oversized valid record may run alone if it fits the admitted budget, otherwise fail with a resource error—never truncate it or silently reduce input. Track actual owned raw buffers/count batches/export buffers; do not claim a process RSS hard cap from an allocation budget.
- Phase1 workers parse selected GTs and compute AC and per-allele heterozygotes once per record. Ordered consumption hashes complete GT/record streams, accumulates input/filter/window populations and writes only retained variable-width counts/metadata into a mode0600 internal spool. There is no redundant full-human GT JSON/array. Source maximum ALT8 may come from excluded rows, so EOF fixes the width before export.
- Phase2 workers evaluate full-width WC/Hudson/MPD and format ordered per-site payloads. Global contig AN maxima and exact per-window row counts are already known; all canonical reductions use fixed source-shaped trees, independent of scheduling. A private online reducer can consume the NumPy128-element/eight-lane recursive sum topology with O(logN) state because sequence lengths are known from phase1; do not store full component matrices merely to reduce them. Keep only masks, bounded batches, output buffers, small window/block summaries and SFS grids in memory.
- Use the freshly read NumPy1.26.4 primary pairwise algorithm as the deterministic reduction authority; account for its eight-lane leaf and aligned recursive split, without unsafe SIMD or global fast-math/FMA flags. A faithful translated helper retains the required BSD attribution. Legacy linear accumulators consume the same ordered site results separately, preserving their C++ reduction order.
- Compression/export is part of the timed work. Parallel independent ordered gzip members at the source compression level9 are a permitted standard gzip representation; record actual output bytes and compare complete decompressed logical bodies. Do not benchmark uncompressed/native output against compressed/source output or time a cached aggregate-only path. First1/4 measurements decide whether compression, parsing or WC dominates; no performance claim follows from this design.
- The source population-ordered AC stream hash may need one additional **compact counts** pass after the first population has been hashed; include that cost rather than materialize another GT copy or claim separate SHA digests can be concatenated.

Default512MiB is a resource-planning target, not a cohort cap. The exact cohort and every output remain fixed; insufficient resources fail loudly. Scratch bytes/time, peak RSS, user/system CPU and output bytes are measured. A bottleneck that remains in serial decompression is reported rather than hiding it behind four named worker threads.

## Frozen comparison policy before candidate outcomes

Freeze one standard-library comparator revision **before the first candidate run** and retain its complete body/hash. It consumes only authenticated source/CAS regular files and native manifests; it does not restore or invoke the Linux runtime. The canonical and legacy field maps are explicit, not generic float-normalization across arbitrary JSON.

- **Exact:** file/schema/column populations, row identity/order, original ordinals, REF/ALT/raw AA/reasons, samples/population assignment, GT/record/AC stream hashes, all integers/booleans, filter/AN/AC/denominator counts, every SFS bin/dimension, window endpoints/counts, block membership/tail, and finite/NaN/+Inf/−Inf classes. +0 and−0 are numerically equal; NaN payload/sign is not a cross-ISA contract.
- **Canonical finite f64 values:** require `abs(native−source) <= max(2^-44, 2^-40 * abs(source))`, fixed once for every canonical site/window/region/component/vb/vj/SE field. This is a pre-outcome engineering f64 contract: the read source uses f64 arithmetic, at most the low hundreds of local WC/diversity/derived arithmetic operations, and logarithmic reductions (NumPy128/eight-lane leaves plus about23 levels for the largest current row×allele sequence). The absolute floor is256 binary64 epsilons; the relative allowance is4096 epsilons, leaving a fixed margin for two source-faithful reductions and platform rounding without accepting low-precision output. This is not a theorem for arbitrary ill-conditioned future profiles and must not be presented as one. Matching source expression/reduction order remains the implementation target; no per-row epsilon, value patch, native-informed tolerance or future table-specific relaxation is allowed. A failure is investigated as concrete components/order/conditioning, and remains red pending parent review.
- **Legacy finite values:** require equality of parsed source-format decimal values after the native general6/5 writer (plus exact schemas/identities/counts). Do not give a 6-digit lane an arbitrary canonical relative tolerance, nor widen a rounding-cell boundary after seeing a mismatch. Preserve raw original source text and accept only the already-declared NaN sign/spelling normalization across native ISAs.
- **Native1 versus native4:** exact decompressed scientific payloads and discrete/provenance identities except the declared worker/timing fields; fixed arithmetic/row order should make the numerical payload identical. Gzip framing bytes/timestamps and actual producer version are explicit non-scientific differences, not ignored scientific cells.

This fixed policy is deliberately proposed for parent approval now, before there is any native output. The descriptive roundoff deltas from the prior stdlib source audit are not used to set its constants and are not native acceptance evidence.

### Closed scientific/producer metadata map

Before comparator freezing, encode this map explicitly—no wildcard “ignore metadata” branch:

- Scientific member population is exactly the six named files per declared contig plus the five named legacy tables when requested. Both producer manifests must contain and truthfully hash their complete own outputs. Compare **every** scientific member, not gzip carrier SHA equality; compare `filter-counts.json` and all scientific result/admission fields as well.
- Exact result/admission payload includes selected sample IDs/order and population indices/counts, original/retained row counts, global ALT width, contig lengths/mask histograms, resolved region/window/block/AA/missing/filter policies, GT/record/AC digests, SFS eligibility, virtual derivative/list bytes+SHA and all canonical numerical fields. A native resolved-profile hash may differ from the source specification hash, but the complete resolved scientific profile must match field-for-field and each profile hash must bind its own actual canonical serialization.
- Allowed producer-specific scalar/path fields are request argv/cwd/input/output-root paths; native executable/version/target/build/dependency identities versus source Python/vcftools identities; effective worker count; measured elapsed/user/system/RSS/scratch/output-byte observations; actual original-file path/basename (not content hash/size); legacy derivative `root` versus the required native `materialized=false` marker; and each producer's actual output size/hash/manifest entries. These are retained and validated against the correct producer, not compared as source-equivalent strings.
- Source Python runtime lock/version/import diagnostics, Python warning filename/line/call logs and the five real external-command argv/stdout/stderr/elapsed/returncode records are **oracle-only execution evidence**. Retain them in the immutable source packet; the native product must not fabricate matching subprocesses, warnings or installed Python metadata. Its own runtime/request/result/manifest and native legacy-operation metadata are required and explicitly typed; extra unexpected native warnings/failures remain reviewable, not silently normalized. The comparator requires the native upstream-profile/version labels and exact virtual derivative identities even though native backend is Rust.
- Canonical numeric JSON/TSV schemas and the five legacy tables do not gain exceptions for producer, path, source labels, nulls or missing rows. Only NaN sign/spelling and numeric zero sign have the declared cross-ISA equivalence; infinity signs and all nonfinite classes remain exact. Completion and all scientific row populations are mandatory on both sides.

## One integrated implementation and verification wave

### Task1: functioning connected product, not scaffolding

- [ ] After parent approval, create the independent owner with the real `analyze` command, typed checked parser/count spool, all canonical estimators and all five optional named legacy reports, ordered full exports and source-bound provenance. Do not publish empty modules or document later operations.
- [ ] Consolidate only the selected seeds in `SOURCE_ASSETS.md`; remove duplicate readers/float writers in the new product, retain team revision/body provenance and required upstream notices. Keep old clone locks/targets untouched.
- [ ] Add **one** `neutral_workflow_complete` integration test using the actual adopted synthetic18-record/two-contig casebook and full retained expected scientific outputs. It invokes the real CLI at1/4, checks all source identities/schemas/bins/values/tails/nonfinite classes/legacy outputs, and verifies an out-of-range and a partial-missing GT mutation fail nonzero without a success manifest. Reuse representative old formula/fixture witnesses inside this test or an existing kernel test when they add a non-overlapping fact; do not copy a helper-level test suite for each port.

### Task2: whole-wave quality and full retained cohort comparison

- [ ] Run pinned formatting, strict all-target Clippy and the full release suite once after the integrated wave; perform one differentiated whole-product/source-hot-path review. No build/test executes as part of the current design task.
- [ ] Run native-only full original input at1/4 against the frozen source packet and comparator. Preserve every complete logical output, runtime/provenance, exit/stdout/stderr and full comparison population; never substitute a tiny fixture for the real cohort. All723,783 canonical rows and all five legacy reports must pass, not merely headline pi/FST or a sample of SFS bins. This is a correctness diagnostic, not the formal four-platform/performance gate.

### Task3: actual native/source resource and release qualification

- [ ] Only after correctness and review, root owns immutable native snapshot/controller preparation and actual native Linux/macOS x86_64/aarch64 tests. Cross-build alone is insufficient; preserve original source selectors/receipts and keep the accepted source-only packet immutable.
- [ ] For the representative same full profile, run full parse-to-identical logical outputs at1/4 CPU budgets, one warmup then five alternating native/source pairs. Shared prior input admission/hash scans are outside **both** timers; both timed paths still reread original VCF, compute all statistics and export/hash the same reports. Record exact source/native/fixture/controller hashes, flags, machine/affinity, output bytes, wall distributions, user/system CPU and honest peak RSS. Retain failed/incomplete outputs as failures, not a partial speed result.
- [ ] Require a strict useful throughput or resource advantage on the whole declared hot path before release; the design makes no threshold/speedup claim. Parent then reviews actual public API/metadata/hot paths and exact-head native CI before deciding commit/push/publication. This note itself authorizes none of those actions.

## Self-review

The plan covers every adopted canonical export and all five legacy reports without new public foundations or placeholder operations. Interfaces share one checked count model and distinct coordinate/block plans; missingness, multiallelic width, AA, NaN, separate WC denominator forms, real resource work and provenance are explicit. The remaining parent choices are approval of the connected `analyze` name, the fixed pre-outcome numeric contract, and authorization to create/implement the owner. No scientific scope, source evidence or user repair decision needs to change.
