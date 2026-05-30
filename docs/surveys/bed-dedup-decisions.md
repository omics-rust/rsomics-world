# bed-* dedup: per-op canonical decisions (#89)

Evidence: 3-way behavioral diff (bedtools 2.31.1 vs `bed-utils` subcmd vs standalone
`rsomics-bed-<op>`) on a representative fixture, 2026-05-30. **Nothing in the bed family
is published**, so retiring a duplicate is a clean local `rm`, no crates.io yank.

**Canonical form = the standalone per-op crate** (project principle: one op = one binary,
not a multitool). `bed-utils` is the bedtools-clone anti-pattern to retire. BUT the evidence
shows bed-utils is *more correct* than the standalone for 3 ops — those standalones must be
**fixed first** so retiring bed-utils loses no correct code. Both directions of divergence
confirm the user's rule: never yank by op-name alone.

## A. Standalone already canonical — retire bed-utils' copy (no fix needed)
sort · merge · subtract · complement · flank · slop · cluster · map · makewindows ·
genomecov · fisher · multiinter · unionbedg · shift · window · intersect · groupby ·
**coverage** (standalone is the *only* correct one — bed-utils `coverage-hist` is a different
op) · **nuc** (standalone exact; bed-utils has last-digit float errors) · **spacing**
(standalone correct; bed-utils outputs gap-intervals = wrong semantics) · **overlap**
(standalone correct; bed-utils emits a JSON summary = wrong semantics) · **annotate**
(standalone = bedtools overlap-fraction; bed-utils does nearest-GFF = different op) ·
**summary** (standalone only; bed-utils lacks it) · jaccard (both truncate float vs bedtools;
standalone closer).

Our extensions with no bedtools equivalent (standalone canonical): count · unique · midpoint ·
len · total-bp · stats · to-gff.

## B. Standalone must be FIXED before bed-utils retires (bed-utils more correct here)
1. ~~**`rsomics-bed-reldist` — real binning BUG.**~~ ✅ **DONE 2026-05-30.** Verified the
   divergence (worse than reported: 0.00 bin 93 vs 8 from a start/midpoint-mixing +
   `.max(0)` clamp; boundary queries fabricated instead of skipped; 100 sub-bins collapsed
   by `%.2f`). Reimplemented as midpoint-straddle + skip-boundary + integer 0.01 bins →
   **byte-identical to bedtools 2.31.1**; real bedtools differential test replaces the
   invariant-only one that hid it; CI green.
2. **`rsomics-bed-closest`** — appends a 7th distance column bedtools' default doesn't emit;
   bed-utils matches bedtools. → drop/gate the extra column to bedtools default.
3. **`rsomics-bed-getfasta`** — wraps sequence at 60 chars/line; bedtools (and bed-utils)
   emit one line per sequence. → default to one-line-per-seq.

(Each fix: reproduce the divergence first — agent findings are external advice, sanity-check
vs bedtools directly before changing code; cf. the seqkit-GC false-positive lesson.)

## C. bed-utils-unique ops → spin out a standalone before deleting bed-utils
Genuinely distinct, no standalone today → create `rsomics-bed-<op>`:
**maskfasta** (bedtools maskfasta) · promoters · resize · rename · chroms · total-span ·
to-igv · to-wig · union.
Skip / fold (trivial variants, not distinct ops): to-fasta (= getfasta), to-gff3 (= to-gff
GFF3 mode), sort-name/sort-size (flags on bed-sort), merge-by-name/merge-overlaps (flags on
bed-merge), flank-bp (flag on bed-flank), tail (trivial), chroms-sizes (flag on chroms),
coverage-hist (flag on bed-coverage). validate already has a standalone.

## D. Execution order
1. Fix B1 (reldist bug — correctness), B2 (closest), B3 (getfasta); compat-verify each vs
   bedtools 2.31.1; commit per crate.
2. Spin out the C ops as standalone crates (own repo + CI), each compat + perfgate.
3. Delete `rsomics-bed-utils` (local dir + its GitHub repo) once A+B+C confirm every operation
   has a correct standalone home. Record in REGISTRY.md.

Granularity verdict stands: the per-op crates are the right shape; bed-utils was the lone
structural dup. The work is correctness-preserving consolidation, not a rename.
