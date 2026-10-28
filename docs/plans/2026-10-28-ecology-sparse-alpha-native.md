# Ecology shared sparse model and complete alpha workflow

Adopted before native output, under the existing initial-diversity plan.
This is a private development slice of the one ecology product, not a new
alpha micro-crate or a stable/public release. Source gate40634594900 and
retained verification2d99ce956ac4dbc0bbee0775f650e5fe7f1ce32efabc8509c39e4b394b0894ae
qualify the actual NumPy2.5.4/SciPy1.18.1/skbio0.7.3/BIOM2.1.17 profile.
The original and source-rarefied count tables each have all15 alpha/PD series.

## Scope and unaffected contracts

Implement one real named-input/output `alpha` CLI with selected multi-metrics,
Faith PD and configured generalized PD, shared validated count model and
prepared phylogeny. No advertised beta/rarefy command, empty future module,
public compute API, new foundation, GitHub publication or placeholder.
The complete first release still requires the remaining workflows; do not
call this development slice release-complete.

Two user choices are pending and remain UNIMPLEMENTED: undefined beta-pair
public behavior, and empty/all-zero rarefaction at depth0 (actual source raises
IndexError despite the earlier intended successful-zero rule). They do not
change alpha contracts or count/phylogeny identity/layout. Do not silently
choose those policies, add a fake error subcommand or claim source parity.

## Data-informed layout

Raw whole-study qualification941282ce.../fdac6bda... confirms41328features,
1105samples,314497nonzeros,0.6887%density, no zero rows/columns, exact finite
nonnegative integers and matching CSR/CSC. Every sample joins exact metadata;
all features occur in unique tree tips. Extra metadata543/extra tips121003
stay explicit. Stored root length is absent; root degree2 is a structural
source profile, not biological validation. These facts are not timings.

Use sparse sample-major counts retaining ALL feature/sample IDs and order,
including zero features/samples in small source fixtures. Shared TSV parser
streams feature rows into sorted nonzero sample entries; no full dense copy.
Public input remains integer TSV only, not BIOM support. Each count and sample
total is <=2^53-1. Types/boundary constructors enforce shape/identity/range
once; kernels borrow them without repeating parser validation.

Avoid whole-tree-edge×sample workspace: its arithmetic size is2.859GB before
overheads, whereas original count CSR is3.94MB. One private prepared tree uses
existing phylo-tree0.2 topology/traversal/tip index, plus feature mapping and
per-worker reusable accumulation/touched-node storage or similarly bounded
sparse work. Do not invent another mutable topology or public tree wrapper.
Allocation and runtime remain to be measured; no claimed zero-cost/win yet.

## Actual metric and output semantics

Read qualified source bodies and selected team-owned alpha/Faith/phydiv
assets before porting. Reuse formulas/provenance, not old parsers/HelpSpec/
Tool/mutable trees/comment narration. One per-sample summary serves observed,
Shannon, Simpson/dominance/inverse/evenness, Pielou, corrected Chao1 and Good
coverage. Metric label mapping/source options are explicit. Named source root
rules, extra tips and stored-root effects remain exact; negative/missing/
nonfinite required lengths fail under the documented stricter input domain.

Represent source unavailable alpha values as typed unavailable results,
with null+reason in JSON and declared nan text in TSV; never output0 for an
undefined value or pass NaN through JSON serialization accidentally. Preserve
metric request/sample order, IDs and source zero/nonfinite classes.

Only common0.12.3/help0.4/phylo-tree0.2 registry foundations are needed, plus
normal external dependencies. Use actual shared ThreadArgs/errors/alias/
atomic named-output/runtime/help contracts; no duplicate presentation layer,
path/B-to-B dependencies or speculative stats API. Source comments are rare;
CLI/public docs describe user contracts, not implementation phases.

## Integrated acceptance and ownership

One or two complete tests cover shared parse→multi-alpha/phylogenetic output,
deterministic1/4 workers, source unavailable cases and a compact invalid/atomic
group. A private source comparator reads the existing complete witness,
does not refit/import scientific packages, and checks both full original and
source-rarefied tables across all15 source profiles under fixed abs1e-10+
rel1e-10, exact IDs/order/nonfinite classes. Extra source boundary cases qualify
only their actual defined domain; no fixture/band/reference tuning.

Fresh external/APFS/resolved-Cargo guards precede builds/downloads. Worker
owns only the new local ecology repository and an external concise report;
parent owns world/asset ledger/Git/index/hosted gates/release decisions. No
commit, push, publish or real-data benchmark in this task. Format, strict
Clippy, integrated release tests/build and one whole-feature review follow;
native4/performance/publication gates are still unearned. Keep other workflows
and source datasets intact. If a genuine mathematical/source contradiction
arises, preserve it and stop that affected operation rather than adjust bands.
