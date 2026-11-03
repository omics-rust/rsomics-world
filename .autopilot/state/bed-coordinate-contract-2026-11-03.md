# BED coordinate-generation contract

Coherent flank/slop/shift wave inside rsomics-bed, not new crates. Owner is
clean49cd59e3fb815af81cbf580185a3c88f4ead6610. Existing Genome, BedReader,
coordinate-preserving record output and common/help CLI plumbing are the
intended reuse points; no public foundation or interval index is needed.

Root reads official tool documentation and all three actual operation bodies
from BEDTools2.31.1 commit705ccfdf2c9a77d71560c8adcece0663c2f5e18e.
Complete retained archive SHA-256
1db4e57c1899aede13ef516c61c9b5819cf2790994a556dbb28972488f9946e6.
Sources are behavioral references, not Rust implementation text to translate.
Official overview documentation:
[slop](https://bedtools.readthedocs.io/en/latest/content/tools/slop.html),
[flank](https://bedtools.readthedocs.io/en/latest/content/tools/flank.html),
[shift](https://bedtools.readthedocs.io/en/latest/content/tools/shift.html).
These pages display2.31.0; pinned2.31.1 source and actual binary observations
own the precise compatibility contract. Root independently rehashes the full
archive, three operation bodies, actual tool and helper after capture.

Historical flank30824143, slop5fc4c997 and shift06ae5853 are fixture/reference
assets, not production merges. Their parsers/CLI, missing options, fractional
rules and clipping behavior are not the target contract. The broad routing
category2 means rewrite the execution core and reuse useful assets, not revive
micro-crates. Existing optional/skipping/sorted-oracle harnesses are discarded.

## Once-only integrated witness admission

Root writes and reads the complete helper/literal BED7/genome fixture in
external bed-coordinate-witness-20261103.Yme2eIoj. Admit exactly eight source
calls from one shared nine-record input: three flank, three slop, two shift.
They combine strand/unknown strand, fractions, negative shrink/crossing,
positive/origin points, clipping, input order and opaque fields/headers.
Do not prefill expected output, sort output or omit failures.

Existing read-only Homebrew BEDTools2.31.1 is exact1691808B SHA-256
fc994e1fcfdce8ce8591630d71f21fe67e2275c2a8d967dc0e3dfb4cfa4677a3.
Reuse its already retained version evidence, not another metadata call.
Each command has10s timeout, zero core and32KiB OS file limits per stdout/
stderr; exclusive raw captures and final ORACLE retain statuses/partials.
Packet16MiB plus independent8GiB free headroom, fresh USB external disk12s1
and28GiB free. Use owner guarded-cargo.sh --exec with external resolved Cargo/
Rustup/target/tmp/cache routes, offline metadata and Python bytecode disabled.
No dependency download/build, native implementation, timing claim, retry,
publication or expansion to a malformed-input/overflow matrix is admitted.

## Actual integrated source observations

The first actual guarded capture exits0 after all eight calls; every source
returncode is0, stderr empty, no timeout/output limit/retry. Root reads every
complete stdout and independently rehashes both inputs and all16 captured
streams. ORACLE.json SHA-256
7a149f16fbf83d87d633637fa7ccfd212382d4844d54fffc5434255baf9a306e;
helper8cd46c816e7de32c2c0334e96e8a809e4eb50e62d7073882c42eb5fd950515b6;
inputb02c9fa9ea1d0fdbdd7537de1fce3e63160a54e25b672c4db8db119d3f146c9d;
genomedf776af089d17b5d375fa1976a46154e7b990dc8d19b3148f9d5bcd77642535c.
These are source observations, not native correctness or performance gates.

Normal rows confirm binary32 fractional/truncation rules, strand-relative
asymmetric widths, negative shrink with crossed-coordinate swapping, clipped
one-base shift outputs, genomic left/right flank order, headers and unchanged
opaque suffixes. Slop treats unknown strand as forward; shift uses plus shift;
strand-relative flank omits unknown strands. These are different source
selection contracts, not a reason to blindly reuse strict required_strand.

Point transformations use the source's expanded point span and later report
adjustments. Actual unstranded flank of origin0..0 emits0..-1; negative-strand
fractional flank of15..15 emits13..13 and17..16. Root preserves all original
bytes and asks asynchronously whether native should repair these into bounded
valid flanks (empty clipped flanks omitted), or explicitly reject the affected
point cases. No reply or broader bug-policy authority is presumed. Stop that
affected flank policy/interface decision; ordinary transforms and slop/shift
contract work may continue. Do not advertise unfinished behavior or expose a
bug-emulation placeholder.

## Actual slop/shift implementation and local qualification

Plan: `docs/plans/2026-11-03-bed-coordinate-generation.md`. Root implements
the shared streaming/signed-coordinate kernel and private optional-field
accessor; one isolated agent implements only unified CLI plumbing. All required
column callers keep their error contract; missing/unknown strand semantics
use a typed Option, not swallowed required-field errors. No new dependency,
public foundation, version, unrelated comment cleanup or flank placeholder.

One integrated CLI consumer first fails because slop is unavailable, then
passes five complete byte-exact source goldens, JSON named output, command
composition and late-error output preservation. Root verifies all seven
fixture/golden bodies against the original source-witness inputs/outputs.
Actual local fmt, strict all-target/all-feature Clippy and complete debug/
release each pass127 executions, zero failed/ignored. Each includes the same
single coordinate consumer; these are not254 distinct cases.

ONE fresh whole-feature review finds one Important output-bound omission,
no Critical/other Important/Minor findings. At size9007199254740995 binary64
rounding can otherwise write end9007199254740996. Root adds the reproduction
to the existing consumer, observes its actual failure, then adds one final
integer end<=size postcondition before writing. Overflow fails loudly without
altering source-prescribed float arithmetic or selecting a flank policy.
The same consumer and entire final suite pass afterward. No second review or
helper matrix. Root reads/adjudicates the complete review, including its
declined native/install/performance/publication gates.

External logs: bed-coordinate-implementation-20261103.GVbJQvyI.
Review7586f16d399afdba846527efaa53fa6b419d4e1025d2ee976c3f5591a54f1c8f;
initial RED973e931a72a98deb98c7e7b74d11de0cec1d3fa75343109b2d024ab34f6359f1;
rounding REDb08ae6a0921d48386a7c03e8d1a8b854bc0c53bfcab9e2629cb3e76ffbefc7da;
rounding GREENe6e006def26e474c754298017f027e3d4e844ac4927f3780be8a57f1fb37828a;
final form0164de2f6e233c33fc99adad52c1fb3b4dec5b5493339bd88caa03779c069f0b;
Clippye9e91e8eee150651f9fd1a9c2bc3b3fb060448c3797b1df2693db219ca39d460;
debuga9d85f35cee7f944f65f1f13e1e6d027706f499759d8811bd98ee15d6f844cda;
release770cd55b80ff4b82dec3c9a74fff1acfacdd17960fbdcdee19e6906708c78c3a.
All actual guards verify external disk12s1/Cargo/Rustup/resolved-target/tmp/
cache routes, offline execution and28GiB free space. No boot project output.

Private feature446b144ed5e0b6c35be1057a8ee297022c5ff09d commits13 paths,
tree d143f3c23347acc765933a2f6d142dff04de35ea. Separate CI concern
161961c688a5c2ecedfe28e3dc152abd8a56a011 only extends existing installed
lifecycle to14 operations/17 calls, including both new actual-binary goldens.
Root reads the full surrounding source/package/smoke logic and exact diff;
AST syntax passes. No hard-coded126 test gate existed; no workflow, source
membership, dependency inventory or historical test counts are weakened.

Fresh public main still49cd59 before the intended fast-forward push. Exact-head
native-four/package/install results remain pending; no publication or time/RSS
advantage is inferred. The released v0.4.0 assets stay immutable. Flank's point
decision and the existing registry authorization hold remain separate.

The fast-forward push succeeds. Actual native-four CI41569005243/1 binds
161961c688a5c2ecedfe28e3dc152abd8a56a011. Read-only full-original collection
uses external bed-coordinate-native-161961c-20261103.vEzROm,256MiB cap plus
8GiB headroom; no retry or scientific execution by the collector. Preserve
actual new-head source/package/install evidence before qualification claims.

Performance preparation is independent and unexecuted at
bed-coordinate-performance-prep-20261103.LmcGpPFl. It reuses all584846 RefSeq
exon rows/28597646B from the accepted full input, not a toy subset; the full
93-contig genome and existing2.31.1 source binary are unchanged. Data-only
inspection finds zero points and297458+/287388- records. Candidate binary
pins remain unresolved pending adopted actual Mac ARM originals. Proposed
24 whole slop/shift runs use one warmup/tool/operation then five alternating
pairs each, complete output comparisons and retained wall/RSS distributions.
No target execution, invented native pin or performance claim is authorized
by preparation; root must read/admit the final runner separately. The old
genomecov dense-source Mac memory obstacle does not describe these streaming
operations, and its original Linux measurements are not reassigned.

## Exact-head native originals adopted and performance admission

All four jobs in41569005243/1 succeed. Root reads the full175-line adapted
verifier and report, independently rehashes all748 explicit packet pins,
checks actual artifact run IDs, replays the complete data verifier in memory
before its receipt write, and compares its whole core result with the existing
VERIFIED without overwriting it. All276 Git source bodies/modes,278 package
members, normalized clean VCS, three127-test profiles per host,14 installed
operations/17 calls, exact new slop/shift installed golden output/argv and
dependency/license/binary correspondences pass. No scientific/recovered code
runs in data adoption. New canonical receipt is
`.autopilot/oracles/bed-coordinate-native-2026-11-03.json`.

Package139021B SHA-256
a7d300c461396da0cec52f4bf4fbe78d3f8fc33a881bd7dd3b763ad430b05bbc;
MacARM1915856B SHA-256
7f77660e24edaeeb1c2d9cf1cda05bb9966a889f474f8982bd4687422cec5c3a.
This source still reports0.4.0 metadata; it is not a new formal release or a
replacement of the immutable0.4.0 assets.

Root fully reads the prepared24-call benchmark and the final two-change delta:
disable core dumps and rehash input/genome/source/native after the campaign.
Final packet bed-coordinate-performance-final-20261103.OtOc3Uyz;
runner10253B SHA-256
7802931896e694a6a9f62c61afa88578c45efed6f2b99c1a564c28ba2d68a144.
Root rehashes all input/genome/source/native bodies and the runner, confirms
the staged0500 native is byte-identical to the adopted artifact, and reads
its Mach-O metadata: arm64, minimum macOS11.0, only libSystem/libiconv; actual
host macOS27.2. These checks authorize this run, not general signing/minimumOS
compatibility promises. No target executes during preparation/metadata reads.

Admit exactly24 full-input slop/shift calls using the recorded fixed flags,
one warmup/tool/operation then five alternating pairs, complete stdout byte
comparisons and raw BSD time wall/RSS. Child300s, core0,40294566B stdout hard
limit,2GiB sampled RSS/packet caps and independent8GiB external headroom.
Actual physical external/caches and source/inputs are rechecked by the guard;
source and native pins are rechecked at completion. No extra probe, local
rebuild, favorable subset, automatic retry or publication. All partial output
and negative results remain. Root AUTHORIZATION contains the actual adopted
pins, not unresolved placeholders. Earlier accepted campaigns are unchanged.

## Actual complete-input throughput accepted

The once-issued benchmark handle14042 exits0 after all24 calls; COMPLETE is
present and FAILED absent. Root independently rehashes/byte-compares every
full output, checks584846 rows each, all exact argv, raw BSD time wall/RSS,
absence of unexpected scientific stderr and all before/after input/genome/
source/native pins, then recomputes all measured medians. No target reruns.
COMPLETE SHA-256
237498a92fcfbacbc8726d7dae2451b6aaaaf67d8b382ef05116428fa96fad67.

On actual AppleM2/macOS27.2, source/native slop wall medians0.48/0.16s (3.0x)
and shift0.48/0.15s (3.2x). Every measured native call is faster than its paired
source. Source RSS median2605056B; native slop3260416B and shift3244032B:
memory is higher, not an RSS benefit. Shared host, warm-cache possibility and
0.01s time resolution remain explicit; no universal speed/thread-scaling or
causal claim. The native body is the actual adopted CI artifact, not a rebuild.

Canonical full distributions/provenance:
`.autopilot/oracles/bed-coordinate-performance-2026-11-03.json`.
This earns the representative throughput gate for slop/shift only. Formal
version/release metadata, final-head delivery and fresh public installation
remain unearned; registry authorization and flank point policy remain held.
The overall three-operation wave and30-product goal are not marked complete.

## 0.5.0 metadata candidate

Root reviews the complete six-file metadata/documentation diff and pushes
`91f2bf9498341b1d1fd4cf27fd244c2820f73ad4` to product main. Only Cargo package
self-version/description, lock self-version, provenance lock digest and
README/PERFORMANCE/MIGRATION change. Production source, tests and CI remain
byte-identical to qualified161961c. Final-head native-four run41573159869 is
queued; no success, release or fresh public installation is asserted yet.
World evidence head a3be99c21f8f41a49da856dd506992c1c4839150 passes exact-head
control CI41572298719. Flank and registry holds remain unchanged.

## Final-head native delivery adopted

Run41573159869/1 at exact91f2bf9 is terminal success on all four native hosts.
Root reads the complete adapted data verifier, independently rehashes747
explicit pinned files, checks artifact run IDs and replays the entire verifier
in memory against existing VERIFIED.json without rewriting it. All276 original
Git bodies/modes and278 source members match; all four source crates agree.
Each host passes127 debug/release/packaged tests and17 installed calls across
14 operations (16 successes plus the expected transactional error). Version,
new slop/shift complete outputs, dependencies, notices and binary correspondence
are checked. No delivery binary or scientific oracle is rerun by adoption.
Canonical receipt: `../oracles/bed-native-delivery-2026-11-03-v05.json`.

Root rereads the current coordinate API/hot paths and confirms the complete
six-file final delta changes no source/tests/CI/dependency. Existing review's
one important correction remains integrated. Actual161961c performance stays
bound to that measured binary; final0.5 binaries are not falsely remeasured.
GitHub draft/public installation remain next gates, not achieved claims.
Fresh organization-secret metadata still reports updated2026-09-08T20:26:08Z;
no unchanged failed registry authentication is retried. Worlddb7ef82f passes
exact-head control CI41574034893. Full30-product goal remains active.

## Public0.5.0 delivery accepted

Root compares all five prepared archive bodies with adopted originals and all
four source crates, reads final notes, freshly confirms release404/tag absence,
and creates one six-asset draft432257319. Exact draft target/notes/names/sizes/
SHA digests match before publication. Publishing that ID succeeds at
2026-11-03T03:50:43Z without changing assets or old tags.

The once-issued anonymous public installer exits0. Root independently replays
public/tag/three-download HTTP200 statuses, all six asset identities, all276
source Git bodies/modes, exact fresh binary,17 command statuses/argv, complete
new/old goldens, JSON/error and transactional output records. All189 packet
files are rehashed; no mutable .remember data is included. Fourteen operations
are delivered; source correspondence is not a fresh public source compile and
no new benchmark/oracle run is claimed. Canonical receipt:
`../oracles/bed-public-delivery-2026-11-03-v05.json`.
Registry and flank holds are unchanged; makewindows has its own origin policy
question, not a reason to withdraw this completed slop/shift delivery.
