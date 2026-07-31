# Historical bed-* behavioral dedup evidence (#89)

Evidence: 3-way behavioral diff (bedtools 2.31.1 vs `bed-utils` subcmd vs standalone
`rsomics-bed-<op>`) on a representative fixture, 2026-05-30. **Nothing in the bed family
is published**, so retiring a duplicate is a clean local `rm`, no crates.io yank.

This document records which historical implementation supplied the stronger
operation-level behavior. Its former one-operation-one-crate boundary is
superseded by the coherent [`rsomics-bed`
product](../10-products/interval-annotation-index.md#rsomics-bed). Standalone
repositories and `bed-utils` are both source assets; selected code, fixtures,
and benchmarks move into modules of that product.

## A. Standalone supplied the stronger behavior
sort · merge · subtract · complement · flank · slop · cluster · map · makewindows ·
genomecov · fisher · multiinter · unionbedg · shift · window · intersect · groupby ·
**coverage** (standalone is the *only* correct one — bed-utils `coverage-hist` is a different
op) · **nuc** (standalone exact; bed-utils has last-digit float errors) · **spacing**
(standalone correct; bed-utils outputs gap-intervals = wrong semantics) · **overlap**
(standalone correct; bed-utils emits a JSON summary = wrong semantics) · **annotate**
(standalone = bedtools overlap-fraction; bed-utils does nearest-GFF = different op) ·
**summary** (standalone only; bed-utils lacks it) · jaccard (both truncate float vs bedtools;
standalone closer).

Extensions with no bedtools equivalent in the standalone source: count ·
unique · midpoint · len · total-bp · stats · to-gff.

## B. Standalone must be FIXED before bed-utils retires (bed-utils more correct here)
1. ~~**`rsomics-bed-reldist` — real binning BUG.**~~ ✅ **DONE 2026-05-30.** Verified the
   divergence (worse than reported: 0.00 bin 93 vs 8 from a start/midpoint-mixing +
   `.max(0)` clamp; boundary queries fabricated instead of skipped; 100 sub-bins collapsed
   by `%.2f`). Reimplemented as midpoint-straddle + skip-boundary + integer 0.01 bins →
   **byte-identical to bedtools 2.31.1**; real bedtools differential test replaces the
   invariant-only one that hid it; CI green.
2. ~~**`rsomics-bed-closest`**~~ ✅ **DONE 2026-05-30.** Found 3 divergences (extra distance
   column always on; tie ordering/completeness via a buggy early-exit scan; raw-gap distance
   instead of bedtools' gap+1 → book-ended spuriously tied overlaps). Rewrote: default 6-col,
   `-d` opts in distance, two-pass tie collection in B-order, overlap=0/book-ended=1.
   Byte-identical to bedtools 2.31.1 (default, -d, no-B edge); CI green.
3. ~~**`rsomics-bed-getfasta`**~~ ✅ **DONE 2026-05-30.** Dropped the 60bp line-wrap (bedtools
   emits one line per sequence); the compat test had rejoined wrapped lines, hiding it. Now
   byte-exact; CI green.

(Each fix: reproduce the divergence first — agent findings are external advice, sanity-check
vs bedtools directly before changing code; cf. the seqkit-GC false-positive lesson.)

## C. bed-utils-unique ops — preserve only what maps to a REAL surveyed upstream
Refined judgment 2026-05-30 (granularity rule: a crate = one operation of the *displaced*
C/Python/R toolchain, not a bed-utils invention; don't over-split tiny awk-class ops):

**Spun out (grounded in bedtools):**
- ✅ `rsomics-bed-maskfasta` ← `bedtools maskfasta`. **DONE 2026-05-30** — byte-identical to
  bedtools 2.31.1 (hard/soft/mc differentials), own repo, CI green.
- ~~`rsomics-bed-igv`~~ **SKIP.** bed-utils `to-igv` is NOT `bedtools igv`: bedtools igv emits
  an IGV *snapshot batch script* (`goto`/`snapshot`), bed-utils invented an IGV data-table.
  bedtools igv is niche GUI screenshot-automation, not a data op; nothing depends on it
  (unpublished). Rebuild as real snapshot-script semantics only if demand appears.

**Retire with bed-utils (bed-utils inventions, not in any surveyed upstream → YAGNI; rebuild
grounded in a spec if real demand ever appears):** promoters · resize · rename · union ·
to-wig · chroms · total-span — all tiny (27–68 LOC) bed-utils originals with no upstream to
match. Preserving each as its own crate is exactly the over-splitting the granularity rule
forbids; bundling them back into a "bed-utils-lite" reintroduces the multitool we're retiring.

**Already covered / fold (no action):** to-fasta (= getfasta), to-gff3 (= to-gff GFF3 mode),
sort-name/size (flags on bed-sort), merge-by-name/overlaps (flags on bed-merge), flank-bp
(flag on bed-flank), tail, chroms-sizes, coverage-hist (flag on bed-coverage), validate
(standalone exists).

## D. ✅ bed-utils retired (2026-05-30)
Verified first: no crate depends on bed-utils (leaf binary), it is unpublished, and every
real surveyed-upstream op has a correct standalone crate (A list + B fixes + maskfasta).
Then **archived** the GitHub repo (reversible; history preserved), removed the KIOXIA clone,
and dropped it from REGISTRY.md (added bed-maskfasta). The bed-utils inventions (chroms,
chroms-sizes, promoters, rename, resize, tail, to-wig, total-span, union) retired with it —
not grounded in any surveyed upstream; rebuild from a real spec if demand ever appears.

**#89 historical dedup pass complete.** It established the stronger source
asset for each compared operation and exposed three latent bugs: reldist
binning, closest ties/distance, and getfasta wrapping. Those fixes and
differential fixtures remain useful migration evidence; the per-operation
repositories are not revived as public products.
