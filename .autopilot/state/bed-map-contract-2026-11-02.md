# BED map contract

This is an unreleased operation inside rsomics-bed, not a public micro-crate.
Source membership and actual outputs, not upstream help alone, define behavior.

## Evidence

Pinned BEDTools2.31.1 source archive SHA256
`fc7e660c2279b1e008b80aca0165a4a157daf4994d08a533ee925d73ce732b97`;
23 recovered original member identities independently verified by root in
`/Volumes/KIOXIA/Developments/tmp/bed-map-contract-source-20261102.53Ijah`.
Real oracle packet:
`/Volumes/KIOXIA/Developments/tmp/bed-map-contract-20261102.6g3jZT`.
SEALED.json SHA256
`fc09ee8a9e57ceff27d1d72925567274a4b62f8cab280ecb78709b4f3bf5f1ee`.
Root verifies all91 retained identities and reads complete initial14 outputs,
warnings, error statuses and corrected point supplement. Sixteen workflow
calls include one malformed-argv preparation call and its separately named
correction; version/help are two additional metadata calls. No source output
is replaced. Native correctness and performance are not yet established.

## Stable target behavior

One sorted BED A, one sorted BED B; output A with canonical coordinate text
and byte-preserved trailing fields, plus requested B-column
reductions. Reuse checked BED parsing and product common/help transactional
output/alias/JSON contracts. Use a private streaming candidate cache rather
than forcing the eager annotate/coverage index onto a sorted workflow.
Keep B encounter order, duplicates and A input order. Optional hit sorting
uses coordinates with a deterministic B-order tie guarantee; source std::sort
does not promise equal-key stability, so do not claim byte equivalence for
its unspecified completely equal-coordinate permutations.

Columns are1-based, default5; operations defaultsum. One column broadcasts
over operations, one operation over columns, otherwise lengths must match.
Requested columns must exist even for count. Record widths are consistent
within each input. Empty B and erroneous columns fail nonzero rather than
reproducing the source ERROR-with-exit0 defect.

All23 reducer strings are in scope:
sum, mean, stdev, sstdev, median, mode, antimode, min, max, absmin, absmax,
count, distinct, count_distinct, distinct_only, distinct_sort_num,
distinct_sort_num_desc, collapse, concat, freqasc, freqdesc, first, last.
Text reducers use byte values and lexical ties, not numeric interpretation.
Retain the actual distinct_only leading-separator quirk. Count/count_distinct
return0 without hits; other reducers return configured null, default period.
Numeric scalar output uses10 significant digits by default; numeric-sorted
distinct uses its separate6-digit source formatter. Delimiter defaults comma.
Mean/stdev count conversion follows the actual binary32 source divisor.
Population singleton deviation is0; sample singleton deviation is null.
Numeric selected malformed/nonfinite tokens fail nonzero under the product's
strict-input rule instead of source warning/partial output. Computed numeric
NaN/null or infinity from valid finite values is not blanket input sanitizing.

Selected valid BED6 strands, ordinary overlap fractions, ordinary BED12 split,
explicit genome order, original leading header output and sorted-input errors
belong to the full operation. Split reductions count eligible B records once,
not coverage's block pairs. Split plus explicit fractions remains held with
the existing source aggregate-fraction policy question; do not guess or reuse
coverage's full policy. All selected inputs and genome paths participate in
alias rejection; only A may be stdin. Unsupported format/flags error.

## Approved point-cache repair

The source accepts identical chr1:10–10 queries before and after ordinary A
rows with start10 but evicts B[0,10) in between: count4/sum17 becomes3/14.
Root independently reads those actual rows. On2026-11-03 the user explicitly
chooses repair, recorded as `BEDMAP-2311-POINT-CACHE-LOSS`, not source cache-loss
emulation. Identical point queries at the same raw A start must retain the
same eligible B records before and after ordinary intervals: the observed
case stays count4/sum17 instead of degrading to3/14. Cache eviction must retain
boundary candidates needed by a later point query at that same start; each
query still applies its actual overlap/strand/fraction filters. Do not expose
a bug-emulation switch or a placeholder point mode. Preserve both original
source outputs and corrected native expectations in compatibility evidence.
This resolves the point-cache policy only; split-plus-fraction remains held.

## Source supplement and root implementation rulings 2026-11-03

Eight tiny source workflow calls are retained at
`/Volumes/KIOXIA/Developments/tmp/bed-map-sweep-supplement-20261103.gECDtG`.
SEALED.json SHA256
73c73c48149334a76894aae9e6cc5399320aee77ae152ec26268a83c80fb5220;
root replays all43 identities and reads every argv/status/stdout/stderr.
No native implementation or performance result is inferred.

The supplement proves a second tied-order loss: B ordinary10–20 before B
point10–10 produces no hit for A0–10, while reversing those equal-start B
records yields point/count1/sum7. Root applies the user's approved point-loss
repair principle to this same-class defect, recorded separately as
`BEDMAP-2311-POINT-LOOKAHEAD-LOSS`. Gather the entire relevant raw-start group
before virtual-bound filtering; do not assume point-adjusted starts are
monotone or expose a bug-emulation switch. Original source outputs remain
unchanged evidence. Cost of this ruling is explicit divergence from the
upstream's order-dependent missing result, not a different overlap definition.

Source accepts A point0–0 against B0–10 and emits B/count1/sum3. Map therefore
uses private widened virtual coordinates that can represent -1; do not route
it through the existing u64 point-expansion helper that rejects zero. Raw BED
coordinates remain checked/nonnegative, and other operations are unchanged.
The positive header witness also proves canonical000/010→0/10 formatting;
retain initial headers, ignore later comments and preserve trailing fields.

Actual no-genome cases confirm aligned chr2→chr1 succeeds, conflicting common
chromosome orders error, lexical B-exclusive chromosomes succeed, and a
nonlexical B-exclusive chromosome errors even after partial source stdout.
Named native output must retain the common transactional rollback on that
late error. Preserve the conditional lexical-fallback source semantics, not a
blanket lexical restriction. Genome mode supplies chromosome order, not an
extra coordinate-size clipping policy.

The missing ContextIntersect.cpp validation body is now retained at
`/Volumes/KIOXIA/Developments/tmp/bed-map-context-source-20261103.2HnaYS`:
10694B SHA256aad77ed16c4f758a35cd347ab2f4db286e79a15a8318992b23ca27d007aace62,
extracted read-only from the same fc7e660c archive with unchanged archive
device/inode/size/mtime. Root reads the actual validation body. Missing fraction
thresholds are0; explicit finite f/F must be in(0,1]. Explicit F+r errors;
F+e without f errors; f+r copies A threshold to B. Bare r/e and r+e remain
accepted, with either retaining OR. Native strict finite parsing does not
copy atof's permissive/NaN hole. Split-plus-explicit-fraction remains held.

## Boundaries and delivery

Historical bed-map1fb4c6cc8d84 is reference/fixture only; do not move unsafe
lifetime extension, permissive parser, old12-reducer core or duplicated CLI.
No upstream implementation expression, new dependency, unsafe code, public
foundation or Layer B dependency is needed. Keep numeric formatting/reduction
private and source comments rare. An unfinished map command is not advertised.
Whole-feature fresh review, actual native4 packaging/install, representative
full-output wall/RSS comparison and release review remain required gates.
