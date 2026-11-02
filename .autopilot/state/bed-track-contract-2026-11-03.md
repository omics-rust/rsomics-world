# BED coverage-track source contract

Next coherent existing-dossier slice: BED-input `genomecov` and multi-track
`unionbedg`, inside rsomics-bed. This is source/oracle preparation, not an
implemented or advertised command. Owner starts at clean26ab4898; accepted
v0.3.0 and its eleven-operation evidence remain unchanged.

## Sources and historical assets

- BEDTools2.31.1 `src/genomeCoverageBed/{genomeCoverageBed.cpp,genomeCoverageMain.cpp}`;
  https://github.com/arq5x/bedtools2/tree/v2.31.1/src/genomeCoverageBed
- https://bedtools.readthedocs.io/en/latest/content/tools/genomecov.html
- https://bedtools.readthedocs.io/en/latest/content/tools/unionbedg.html
- Historical `rsomics-bed-genomecov@0087cf10fc15` and
  `rsomics-bed-unionbedg@17190fd1bff7` remain implementation/fixture assets,
  never revived micro-crates. The former loads every chromosome before
  reduction, silently skips missing strand fields, iterates genome order and
  uses binary64 fractions. Do not directly inherit these policies.

Read allowed source to establish behavior; do not copy GPL implementation.
Reuse team-owned historical fixtures and selected algorithms only after
comparing their actual contracts with the pinned upstream.

## Genomecov source and actual witness

Packet `/Volumes/KIOXIA/Developments/tmp/bed-genomecov-contract-20261103.pgnXoz`.
ORACLE.json SHA-256
`240d341093a27b0b573c520347d8c94273b8f288820a5b2d599805fe3ee5600d`.
The actual installed BEDTools2.31.1 binary identity,13 argv/exit/stdout/stderr
records and three input identities are retained. All13 commands exit0 with
empty stderr. Physical external-device/cache/tmp preflight precedes fixture
creation and execution. No compilation, dependency download or product edit.

Observed modes are histogram, bedGraph with/without zeros, one-based all-base
depth, zero-based nonzero depth, strand selection,5'/3' ends, scale, histogram
and bedGraph max, trackline/options and BED12 split. A deliberately non-genome-
ordered fixture also has unsorted starts within a chromosome and an interior
point. Source emits encountered chromosomes in input order, followed by absent
chromosomes in genome order. It accepts unsorted starts within a single
chromosome group; do not impose the map operation's sorted-start requirement.
The interior point10 expands to9..11. Split2..16 with blocks3,4 at0,10 emits
2..5 and12..16. Endpoint selection follows strand and expands no whole span.

Source reads scale as binary32 and uses binary32 histogram fractions with
six-significant-digit C++ formatting. Explicit scale is invalid with histogram;
trackline/options require bedGraph; split excludes end selection. Preserve raw
depth change boundaries before max/scale: `-bg -max1` keeps adjacent emitted
segments even when their reported capped depths are all1. These are ordinary
contracts, not upstream defects to repair.

The pinned source allocates chromosome-length depth arrays and iterates bases
even for histogram/bedGraph. Candidate design should reuse the product-private
DepthBuffer sparse events/runs, retain at most one chromosome group, coalesce
equal raw depths before reporting and aggregate histogram bins without dense
genome-length storage. This is a design hypothesis, not measured performance.
No new foundation: named existing consumers coverage/annotate/genomecov share
only private depth reduction, not their distinct point/format/output policies.

## Unionbedg inventory and affected policy hold

Packet `/Volumes/KIOXIA/Developments/tmp/bed-unionbedg-contract-20261103.n7WBnQ`.
CAPTURE.json SHA-256
`98195c3e2fa3255e7881c6d4701b243e160ffcd3c6bb5b263f26116284ac1238`;
REPORT.md SHA-256
`89a584a8f9e69aec45bfc171913d63b66365677a8ef302d4b3f0002325a469ef`.
Root reads the bounded report and verifies every PINS.sha256 entry. Ten pinned
source members and16 tiny workflow calls plus version are retained externally;
source/parser/CLI members also match fresh exact-revision public raw bodies.

Presence is separate from a textual value or zero. Preserve every interval
boundary, including adjacent equal vectors. Header without names has only
three coordinate column names in the actual2.31.1 oracle, unlike the rendered
documentation example. Names alone do not turn on the header. Empty-region
output covers encountered chromosomes only, not every genome entry; genome
order does not control output. A single empty input retains its filler column.

Do not inherit the legacy whole-event sort/global first-seen chromosome rank,
signed coordinate permissiveness or missing-fourth-field default. Keep its
fixtures/reference as class3 assets; source preconditions require grouped,
sorted, nonoverlapping per-file tracks. A target common-chromosome ordering
rule still needs to be frozen before implementation; arbitrary source reentry
from conflicting inputs is not a valid ordered-track contract.

Actual source drops the last valid record without a final newline and aborts
when all inputs are empty. Root asks the user asynchronously whether to repair
these as known incompatibilities (preserve the record, return empty output).
These two behaviors remain held pending reply. Malformed-width silent
truncation and overlapping-input data loss are not accepted product policies:
the existing fail-fast parser boundary applies. Other source/design work is
unblocked; no implementation or automatic compatibility waiver follows.

## Remaining before implementation

Finish unionbedg source/oracle inventory and freeze narrow typed interfaces,
CLI mapping and existing strict parser/output integration. Explicitly record
unknown-contig/bounds, repeated chromosome, zero-origin point and endpoint
strand contracts rather than importing unrelated operation behavior. Do not
silently repair any newly contradictory valid-input behavior.

Exclude BAM/CRAM, fragment/proper-pair/dUTP/CIGAR options, GFF/VCF input,
bigWig conversion and unrelated coverage-report extensions. These have owning
products or later dossier slices, not placeholders in this work.

Then implement the complete declared workflow, preserve strongest historical
goldens, obtain real upstream complete-output comparisons, one whole-feature
review, representative whole-track throughput/RSS evidence and exact-head
four-native CI. Do not reopen unchanged accepted eleven-command campaigns or
advertise either operation before its own complete acceptance.
