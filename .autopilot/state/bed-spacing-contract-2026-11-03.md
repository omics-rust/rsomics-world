# BED spacing contract

One BED-only streaming operation inside rsomics-bed, not a new crate.
Historical `rsomics-bed-spacing@fb4ede5c9a18bfdffcaafd1e45e369d7b6402e95`
is category3: fixtures/reference only. Root reads its whole library; points,
start-order checks, signed arithmetic and overlap semantics cannot merge as-is.

## Source identity and behavior

The complete BEDTools2.31.1 archive at
`/Volumes/KIOXIA/Developments/tmp/peak-input-runtime-20261031.LSP4R2/metadata/bedtools-2.31.1-complete.tar.gz`
has independently rechecked SHA256
1db4e57c1899aede13ef516c61c9b5819cf2790994a556dbb28972488f9946e6.
Root reads whole spacingFile.cpp/ContextSpacing.cpp/bundled tests and actual
Record overlap/point, FileRecordMgr sorting and RecordOutputMgr output paths.
Core88373e277322fcda3c281986878f53f7b88747262633311fe6770b2aa2bf19b7;
context54cffb60527848af52c6d0278000bc03416d0e02cc0820c4c91a89add6df867c;
Record57e7c47ace99b9506a7657eb8b466244de577c408525e0e5d9fde0a0f8437ac7;
sort managerc7fe70b21a5fb4afcd742606d49d12f0ab76dd27d72005c09934094d4b315244;
output4f9af1d82ac1f612215067d483c3bb04719eaf8b5c3be00a7674bfa8a9e1ad67.

Append one spacing value per BED record. First per chromosome is period;
eligible overlap is-1; otherwise signed current virtual start minus previous
virtual end. Compare immediately previous record, not maximum prior reach.
Points use signed widened[p-1,p+1] for both operands, restoring raw valid BED
coordinates for output. Overlap uses binary32 intersection/current-length and
intersection/previous-length, both at least1e-9, not merely positive overlap.
Starts sort by raw coordinate, ties allowed. Chromosome blocks cannot revisit;
without genome order is encounter order, not lexical. Optional genome enforces
its listed order/known chromosome, not coordinate clipping. Strand and blocks
have no selection effect. Output canonicalizes coordinates, retains suffix.
Initial headers are optional, including header-only input. Empty input is empty.
Other formats and generic no-op/source transport flags are not new BED modes.
Reuse product checked parser/genome/common/help/transaction/alias plumbing.
No new foundation or dependency, no change to other operation sorting policies.

## Six real source workflows

Packet `/Volumes/KIOXIA/Developments/tmp/bed-spacing-witness-20261103.vqHptDAX`.
Root reads complete helper/six input bodies, independently hashes them and
executes it once through physical external-disk/caches/Cargo/Rustup/target/tmp
guard, with28GiB free. Helper SHA256
c5d92edcaaa709a4c32ea08347ed6c88fae30531c3310fe09807ff6edcbe3336.
Existing read-only executable1691808B SHA256
fc994e1fcfdce8ce8591630d71f21fe67e2275c2a8d967dc0e3dfb4cfa4677a3.
10s/32KiB-per-stream/core0/8MiB packet/free8GiB; no retries or native build.
All actual statuses match[0,0,0,1,1,1]: integrated header/points/nesting/giant
fractions, same with genome, header-only, descending starts, bad genome order,
chromosome revisit. Root reads every complete stdout/stderr. Header-only does
print headers. The2bp overlap between4-billion-base spans emits signed-2;
nested30–1000 then40–50 then60–70 reports10 against previous50, not reach1000.
These are witnessed source semantics, not bugs to silently normalize.
ORACLE.json SHA256
913af17f1479b82127546814f93c8ba65837e5619d5f9195168abe21bc2edebd.

Implementation, native verification, performance and publication are unearned.

## Actual implementation frontier

The integrated consumer test first fails with actual unknown-spacing command
(guarded handle46960 exit101), then passes after implementation (32907 exit0).
One integrated flow covers three source successful outputs, the three sorted/
genome errors with prior-destination preservation and stdin/JSON output. Root
implements the stream/CLI inline with no new dependency or version change.

Ruling: centralize identical signed point expansion privately for actual map,
coordinate and spacing consumers, and share the existing cluster raw-order
validator with spacing's optional genome ranking. This avoids a third point
implementation and copied sorting state while keeping public foundations and
other operation-specific order policies untouched. Existing consumers retain
their tests. Full suite/strict Clippy/whole-feature review remain pending; no
performance/native-four/release claim is made for this dirty candidate.

## Local implementation qualification

Guarded68591 exits0: format, strict all-target/all-feature Clippy, full debug
and release each128 tests,0failed/ignored. Existing map/coordinate/cluster
consumers remain green. Root replays all six source statuses/argv/helper/input/
tool/output hashes and compares every one of the seven checked-in fixture
bodies with original source capture. No expected output is derived by native
code. RED stdout8715d6841ca3bdb5e8871bbc3f4da214057f2a5431d102a52ec4ca0da7e21f22;
GREEN stdoutfcc405704f764b4cfa3853a358a2e2fb58d02451691902ea29c9d207e04dae5b;
Clippy stderr7a2e61d212ac843d1a1f09f87348cf262efb0cbcf7dd6d35ca4ad14f9c27e13d;
debug stdouteaf0b858d829827a68b153c249bb98eface69867bdfb70a3f24167b08d7939bd;
release stdout92da06f7d7558ff88f49c27e6a154dc70747316f8ffb6b55cd7c91b74adb659e.

One fresh astra/high whole-feature read-only review reads complete spec/plan,
production/test delta and actual upstream implementation. Verdict: no Critical,
Important or actionable Minor findings; no fix pass or second review. Root
adopts the review's geometry/order/IO/private-reuse findings. Its declined
items (installed lifecycle/native4/performance/publication/platform execution)
remain explicit future gates, not implicitly passed.

Root extends the existing installed lifecycle by one full spacing golden call:
15 operations/18 calls,17 successes plus existing expected error. Complete
one-file diff is read and AST parses; no new test or guard framework. Prepare
0.6.0 metadata before the first push so final native artifacts can be measured
directly without a second version-only CI/build wave. This is an unpublished
candidate, not release or throughput qualification. Local logs above precede
the metadata-only version bump; final-head native4 must exercise0.6.0.

## Exact-head0.6 candidate pushed

Feature6158b28 and metadata35d656f00f0db19877b16aec23453db14520b8aa are
separate scoped commits, pushed together once. Owner is clean; actual Git
tree has287 bodies. Metadata changes only package self-version/lock digest
and README/MIGRATION/PERFORMANCE; all dependencies and license texts stay.
Root reads the complete six-file metadata and installed-lifecycle diffs.
Actual native-four run41577594195/1 is live at exact35d656f; collect those
originals without rerun. No old0.5 asset, performance pin or release is replaced.

Twelve-call representative spacing campaign is preparation only at
`/Volumes/KIOXIA/Developments/tmp/bed-spacing-performance-prep-20261103.t4PrVey3`:
unchanged584846-row full BED, actual future adopted MacARM CI binary versus
pinned source, one warmup/tool then five alternating pairs and full outputs.
No native artifact, authorization or measured benefit is inferred from a draft.
Final native adoption and root full helper/pin review must precede execution.
