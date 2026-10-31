# Peak native first-release implementation

Adopt the integrated proposal at
`/Volumes/KIOXIA/Developments/tmp/peak-native-design-20261031.MCyvMC/PLAN.md`
(SHA256 f9f8429a1f7fad9e5aa8d9fce7cd17c60676c4a8103b7f42f492d3a4614aba1c)
with the rulings below. Parent has read the complete473-line proposal and its
107-line report, including current public foundation contracts and historical
asset dispositions. This is implementation authority, not native acceptance.

## Scope and rulings

One private owning repository `/Volumes/KIOXIA/Documents/omics-rust/rsomics-peak`
may now be created. Keep publish=false and no public remote/release until the
whole declared slice earns its gates. Only a functioning operation may enter
the CLI; do not add placeholder subcommands or accept ignored options.

The eventual slice is narrow BAM/BAMPE calling, sparse positive-signal calling,
transcript-level GTF/GFF3 annotation, and multi-BAM overlap counts. Broader
callers, broad peaks, FRAG/barcodes, plots, gene aggregation, fragment counts
and extra input-format claims remain excluded. Original source authority is
run41209740891, not historical approximate goldens.

Ruling: first-release count units are alignments, following the declared
multicov replacement. Proper-pair filtering does not collapse mates. No human
decision is needed to retain that existing scope. Expose default/MAPQ/proper/
duplicate/QC/split selectors; defer overlap-fraction/reciprocal/strand options
unless a concrete supported workflow requires them. Keep duplicate peak rows
and both secondary/supplementary source populations.

Ruling: annotation initially exposes the default category priority, not a
custom-policy framework. Preserve nearest identity/distance independently of
category; reject unmatched annotation contigs before publishing output.
Source addon41220226878 at35af65ef is an actual separate restoration/capture
gate; its proposed categories/counts do not become source truth before success.
Original38-case inputs and answers never change to match native results.

SEACR default repair versus source profile remains the already-asked human
choice. Code may use a mandatory private compatibility enum, with no inferred
default. Do not expose an unresolved public default or call corrected answers
source-compatible. This hold does not stop call/annotation/count construction.
Exact base-R4.6.1 density/bandwidth/FFT/interpolation and current MACS3.1 used
probability/reader bodies must be read before implementing those algorithms.
Team-owned assets may be reused; inspecting GPL upstream source is not authority
to copy GPL code into a differently licensed implementation.

## Coordination and delivery

One coordinator owns Cargo/lock, CLI, product-private models, execution/output
contracts, guard and integration. First freeze concrete narrow module interfaces;
then isolated agents may own operation directories without changing shared
contracts or foundations. Existing common/help/bamio/intervals are concrete
dependencies; no new public crate or API is justified. Public library surface
is the CLI entry point only. Use rsomics-help, common errors/output/threads and
transactional grouped output. Keep source comments rare and stable.

First deliver the shared contracts and genuinely functioning count operation;
then compose the other actual operations under the same owner. Private module
delivery is not a separate product or release. Prefer one focused connected
RED/GREEN workflow and ordinary module invariants over per-helper campaigns.
No local upstream R/Python science, source installs or containers. Native Rust
builds/tests use an external-only guard; all Cargo/rustup/target/TMP/tool caches
must physically resolve to the allowed external volumes.

The complete native graph must route native call tracks into native sparse
calling, then native peak rows into native annotation/count, with1/4-worker
semantic equality and truthful worker provenance. Imported original answers
are additional comparisons, not connected native intermediates. Preserve
source precision stages, global q adjustment, all pooled inputs, actual model
fits and no implicit fallback. Do not invent finite numeric defaults.

After integration: one fresh whole-product review, one concentrated material
correction wave, full fmt/strict Clippy/debug+release suite, frozen actual owner
snapshot, exact four-native gate and one pinned source-on-native lane. Subsequent
real failures get narrow evidence-based fixes, not a new review ladder. Real
representative correctness-bound performance and public metadata/API/hot-path
review remain release gates. Parent alone stages, commits, pushes, dispatches
or publishes; direct main, one concern per commit, no PR/Co-Authored-By.
