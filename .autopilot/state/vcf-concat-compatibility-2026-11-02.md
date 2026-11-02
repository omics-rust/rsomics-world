# VCF concat normal compatibility and region-file consumer

Status: shared public-parser consumer and maintained normal-oracle increment
root-adopted after one whole-feature review and one P2 test-only correction.
Capture evidence remains diagnosis; the corrected local test bundle is now
accepted for this bounded wave, not full concat qualification. Consumer plan:
`docs/plans/2026-11-02-vcf-region-files-and-normal-oracles.md`.

## Captured normal workflows

One uIrZ88 bundle completed332 commands/74 paired cases/0 timeouts. All16
encoding combinations, four BCF-G outputs and six full two-sample phase cases
match normalized complete headers/bodies. Source/tool174-body/mode identities
remain unchanged; root reads the full226-line runner/report and freshly checks
all894 final manifest entries. ManifestSHA324b2121aad8f4c1d880f113975360ff85b3db2c90c3629366a4381d337bd5a5;
root checksum logSHA429d6e533c598bf31fff27e7865ed486d71a32b20c55002a2bf18247facb7251,
stderr0 at vcf-concat-probe-root-20261102.aMJf7i.

All36 indexed inline-region cases have equal record multisets/dedup winners;
twenty differ only in same-coordinate order. Native retains A-del,B10,C10,
source gives B10,C10,A-del; exact dedup keeps the same B10 in both. One new
asynchronous policy question asks whether to preserve documented input order
(recommended) or align source order. Neither is labeled an upstream defect,
and no body-order normalization is accepted as a compatibility pass. Missing-
LF policy remains a separate pending question.

The two TSV `-R` cases fail natively before output because a whole tabbed line
is interpreted as the reference name. Root reads actual shared entry points
and primary installed manual; this is not an invalid fixture or CLI setup.
Other observed safety contrasts retain exact outputs. Specifically, upstream
concat exits0 and retains raw QUAL=bad; a subsequent view decode exits0 and
coerces it to0. These are distinct stages, not concat coercion. Native fail-loud
behavior and destination preservation remain unchanged.

Report uIrZ88/REPORT.md SHA64a0601d348def4eb5381dd7c5ec2ae23331dedde6d427a71c7febe23db44d47;
runnerSHA36feb2c7ff0d626e812c10acdbad447643fcbc25ff08e43a10628e3be562d4fb.
The capture.log hash in the in-process inventory predates its final stdout;
the final894-file manifest is authoritative, not that one stale row.

## Focused file-format contract

A separate FI6p2q bundle, justified by the confirmed bug, executes33 commands/
28 outcomes over six normal formats plus one legacy control, with0 timeouts.
View and concat agree: source accepts TSV point/interval, BED, uppercase gzip
BED, ordinary-gzip TSV and VCF POS-only files. Native plain formats exit1 as
whole-line-reference errors; gzip exits4 as UTF-8 failure. Legacy colon-file
selection works natively and is rejected upstream. No parser fix or repeated
old bundle is hidden in this capture.

Report FI6p2q/REPORT.md SHA58c92d160fde1625563c8a3538eedc935a5f5b2c32d06a32dba7af3c66e3d4a6;
runnerSHA97278217b99b7bf7ec3094d1b324792bd5e7ccc8673bbf0dd71bf57a7f1bd2b4.
These observations establish the normal coordinate conversion, not INFO/END,
symbolic/gVCF or tabix-streaming region-file guarantees.

## Existing Layer A component, not another parser

Root reads complete intervals0.4.0 parser and real Call/CNV consumer adapters.
Clean sourcecda29c88b67882b63e1710a22aa480e905d4423f matches cached published
parser bytes. Reuse its gzip/BED/VCF/tabular normalization; VCF handles only
typed Noodles conversion, its legacy selector grammar and selection policy.
No new foundation/API is needed. A source-only non-UTF-8 suffix-inference
finding is retained for later foundation follow-up before public delivery;
do not claim it runtime-qualified or duplicate its parser to avoid it.

The planned wave is unambiguous shared-consumer repair plus observed normal
workflow oracles, not a request to implement every hypothetical corner case.
Both public-policy holds, performance, broader resource/integration and owning
native-four/delivery gates remain separate. No owning stage/commit/publication
has occurred. The now-accepted production/test increment is recorded below.

## Completed bounded consumer and oracle increment

Packet:
`/Volumes/KIOXIA/Developments/tmp/vcf-region-consumer-wave-20261102.pAZRxI`.
Final REPORTSHAcb7ca2d9c02c69cd9cf114c4b9a274887af6189ae263a48e3441d0a4305154c4;
72-payload manifestSHAa44c5e9592f8fadc575ba5e9f6adb30b744eae13a11b38407e678185182f3ddd.
Root reads the complete report/command record, one review and sole P2 delta,
checks all72 final payloads and freshly checks every176 live source body.
Raw result lines total30 groups/452pass/0fail/64ignored per full profile and
7concat+3filter+4view explicit passes per profile. No scientific rerun is needed
for adoption. The initial failed logs and wrong-stage expectation remain.

Exactly seven paths form this increment: Cargo.toml/lock, shared variant entry,
RegionSet, new private region-file adapter, new connected CLI regression and
concat compatibility tests. Published intervals0.4.0 is the only lock addition,
checksumd55e00dc3e6c0423430ed2ac84dfb21fe05acf522e83db1b75b09c995fa5b593;
other locked versions, generic sample-list body and three protected BGZF
repair bodies remain exact. No new public item, foundation edit, flag/version,
overlap or indexed-traversal policy is introduced.

The connected real CLI regression moves12 normal view/concat format commands
from observed failure to exact full-body success, retains both legacy-colon
controls, and checks caret target exclusion plus malformed-file rollback/path
context. Maintained normal oracles now cover all16 encodings, relative file
lists, four BCF-G outputs, six complete two-sample PS handoffs, complete safety
contrasts and actual stdin. No body-row sorting hides source differences.

Initial format/Clippy/full suites pass, but both explicit concat runs fail6/1
because the new test expected post-view QUAL0 directly from raw concatbad.
The one fresh whole-feature review finds no material new production defect;
root authorizes only the P2 test-stage correction. It now asserts successful
raw concatbad, explicitly successful view0, unchanged native rejection and
marker preservation. The previously blocked stdin block now completes.
Fresh coherent format/strict Clippy/full debug+release/explicit selection bundle
exits0. Explicit14 were default-ignored, not ordinary passes; other50 ignored
tests remain unexecuted. Existing3 resource CLI tests pass without a new
representative resource/performance claim.

Review: vcf-region-consumer-whole-review-20261102.bCIHPx/REVIEW.md,
SHA0f530b8acaac38fb0dfc480d3e4dc38caaf0f12e2fbde8d740fcb2cc178ad5ee.
No second reviewer or helper campaign. Final concat-test SHA
d054d8c3b45d180f4af1e48bdaadcb9a35eedae2d678a9dd8faf7d6c18f0ed7f;
six other reviewed bodies remain exact. Final source174→176:5changed,
169untouched,2new0644, all prior modes preserved and index empty. Owner HEAD
still682942cfa69768dc3a127a8544f2f07213b704ea; unrelated work is not staged.
Final source manifestSHA2ceca46765c96f7a456010b398fc905424ab6b02b2680e47eae141c93e36fe53.
Exact seven final patches/manifests are retained under
`.autopilot/snapshots/vcf-region-consumer-2026-11-02/`, only over its identified
174-body predecessor, not owning HEAD alone.

## Review rulings and independent remaining gates

- P2 QUAL stage is corrected as above; no scientific/production relaxation.
- P3 empty/comment-only selection-file path-context loss is explicitly deferred
  as minor; no helper test matrix or polish patch is added.
- Missing-final-LF splice remains P1/human-policy held; no error/insert choice
  is inferred and no corruption is reclassified as compatibility.
- Same-coordinate indexed ordering remains human-pending; equal membership/
  dedup winners do not prove body-order equality.
- Foundation non-UTF8 suffix inference remains a source-established follow-up
  before public delivery; no new runtime pass or duplicate parser workaround.
- Targets mixed-column/whole-token permissiveness is retained product grammar,
  not new upstream-strictness parity or a shared API change.
- Indexed/streamed file processing, chromosome ordering and INFO/END/symbolic/
  gVCF semantics beyond observed VCF POS selection remain outside this repair.
- Generic sample files and unusual caret/non-UTF8 combinations retain existing
  behavior, not new qualification or a sample-selection redesign.
- Unrelated inherited concat/reader/index code was not re-reviewed/reclassified
  by this increment. Its earlier repairs retain their own bounded evidence.
- Representative throughput/RSS/I/O, many-input resource evidence, owning
  exact-head native-four, package/API/delivery and registry gates remain open.
  This bounded local acceptance does not authorize public frozen0.6 unchanged.
