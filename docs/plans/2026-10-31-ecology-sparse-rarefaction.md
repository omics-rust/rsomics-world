# Ecology sparse rarefaction extension

Extend the existing private owner at main01c9089. The alpha implementation,
its source comparisons and shared sparse count model remain the baseline.
The complete first-release product still needs beta, whole-product/native4
and representative performance gates; this extension does not narrow it.

## Behavior authority and user ruling

Use admitted source40634594900/verification2d99ce956ac4dbc0bbee0775f650e5fe7f1ce32efabc8509c39e4b394b0894ae:
scikit-bio0.7.3/BIOM2.1.17 and the actual installed NumPy2.5.4 profile.
Parent reads actual skbio wrapper, BIOM sparse sampler, Generator.choice,
SeedSequence/PCG64/bounded-integer bodies and complete team-owned legacy RNG.
The latter's cutoff20 is already correct; do not invent a cutoff correction.

Each sample initializes a fresh same-seed PCG64. Scalar seeds are nonnegative
u128, default SeedSequence pool4, no spawn key. Retain low-first32-bit entropy,
explicit little-endian paired64 seed words, step-before-output XSL-RR and the
cached high32 behavior across mixed32/64 draws. Inclusive bounded draws use
the actual source-width dispatch and Lemire rejection; zero bounds consume
no random value. This is one recorded profile, not arbitrary NumPy parity.

Without replacement and shuffle=false, population>10000 and depth>population/20
uses partial tail shuffle; other populations use Floyd. Preserve the exact
stream and resulting selected indices, then map sorted draws into nonzero
feature spans without materializing repeated reads or a dense count matrix.
Private team-owned rng.rs is a refactor seed, not a new public RNG crate.
Retain actual relevant NumPy/PCG/BIOM notices for adaptations.

The user explicitly chooses same-dimensional zeros at depth0 for empty or
all-zero input. Document this as fixing the captured upstream IndexError,
not an upstream-compatible status. All feature/sample IDs and order remain.
Depth greater than any sample total fails before output; do not inherit the
legacy helper's silently zeroed undersized columns. Exact-total sampling may
reuse original entries: each call owns its RNG and exposes no later state.

## One shared product workflow

Add a working rarefy subcommand with named --counts/--output, required depth
and recorded seed/profile, using current common/help/ThreadArgs/result envelope
and atomic output. A canonical feature-by-sample TSV can be fed directly to
alpha and later beta. Reuse Counts and its validated integer/identity domain;
do not introduce another parser, public matrix type, path dependency or B→B
dependency. Keep no-feature tables valid when sample headers are present.

Counts stores source-ordered sparse sample entries. Sampling transforms only
those entries and totals, leaving IDs/order intact. Independent columns use
an indexed Rayon execution; workers1/4 produce exact same table and provenance
apart from the recorded actual worker count. Dense TSV emission may use one
cursor per sample, not a second dense features×samples allocation. Allocation
failures propagate as errors rather than fabricated successful samples.
Any population-sized tail-shuffle workspace must be explicit in the report;
do not claim sparse-count storage also makes sampling independent of depth or
population. No scheduling or seed-search adjustment after a mismatch.

## Compact qualification and ownership

One integrated CLI lifecycle checks actual primary12×6 depth8/seed42 output,
small40×3 depth200/seed42 Floyd and big300×1 total59297 depth29648/seed123
tail shuffle against unchanged captured arrays/identities. Reuse resulting
primary output in the current complete multi-alpha/phylogeny source comparator.
Check both worker counts, exact conservation/bounds/IDs and ordinary replay.
One compact failure/atomic-output group includes zero depth/empty features,
all-zero samples, depth>total and input/output alias. Reuse existing tests;
do not create a separate per-helper test/review or new oracle campaign.

Use source's retained exact-integer-max draw to exercise the64-bit bound path
without an enormous population allocation. Existing counts require cells and
sample totals<=2^53-1; keep this limit, no new broader source-precision promise.
Formatting, strict Clippy, complete existing tests and one fresh whole-feature
review precede adoption. Keep raw failed outputs and command/binary/source
hashes; no local scientific imports/refit/RNG oracle rerun or benchmark tuning.

Worker owns only rsomics-ecology source/tests/docs and external evidence; parent
owns control plane, Git/index, adoption/native4/performance/publication. Keep
publish=false/no remote. Every build/test/extraction/download uses fresh owning
guard with actual Cargo target/external paths and APFS<80%. Scratch/cache/builds
stay external. No deletion, install, empty future module or speculative foundation.
The unresolved undefined beta-distance decision does not block this extension.
