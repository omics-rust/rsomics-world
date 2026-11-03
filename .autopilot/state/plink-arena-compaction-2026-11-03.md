# PLINK completed arena compaction

Base is clean private7cee8e4366ef957981c1741055c3ff716dd5b53b. Green14 native/source
and the separate qualified-CI-binary metadata comparison are accepted; the
original upstream-negative whole-performance gate remains. Plan:
`docs/plans/2026-11-03-plink-arena-compaction.md`.

## Next-path decision

Existing complete single-command attribution contains0.237102s of output
hashing and0.275726s of manifest/durable commit; these are coarse observations,
not isolated hash or fsync costs. Preserve final-file SHA-256 and every file/
directory sync. PGEN header backseeks make chronological stream hashing invalid.

The actual locked sha2-0.10.9 source dispatch falls back to software without
SHA-NI. Do not infer a speedup from changing a feature or version. The upstream
[assembly repository](https://github.com/RustCrypto/asm-hashes) is archived;
the [sha2-0.11 documentation](https://docs.rs/sha2/0.11.0/sha2/) documents x86
SHA-256 SHA-NI and software fallback, with AVX2 listed for SHA-512, not SHA-256.
No suitable measured hash-backend gain has been established. No dependency,
backend or cryptographic algorithm is changed by this decision.

Current RowsBuilder retains geometric String and field-range Vec capacity in
the shared immutable arena. Compact both once before Arc publication. This is
a narrow standard-library resource change, not a new parser, format policy,
foundation or speculative unsafe optimization. Allocation/copy cost is a risk
for later measured timing; neither RSS nor speed is presumed from source.

## Bounded implementation admission

Delegate only src/dataset/metadata/text.rs production compaction and the
existing10000-variant resource consumer in src/dataset/metadata.rs. Fix the
new2611523-byte ceiling before production edits, after observing the existing
2911523-byte baseline; require real RED/GREEN. Run existing guarded fmt,
strict Clippy and complete debug/release suites, no new scientific campaign
or scalar-helper test matrix. Preserve outputs/logs and inherited files.

All work/cache/tmp uses physically checked KIOXIA paths; preparation packet
`/Volumes/KIOXIA/Developments/tmp/plink-arena-compaction-20261103.lVXyaBFp`,
local128MiB envelope plus independent8GiB free headroom. No dependency download,
SSH, publication, source staging or commit by implementer. Root performs one
fresh whole-feature review after the frozen delta, then separately adjudicates
integration and any future native/source/performance execution.

## Caller-driven placement correction

Root reads io/vcf.rs::Reader::read: each record constructs a transient
single-row RowsBuilder and calls finish. Compaction in universal into_rows
would introduce two needless allocator requests per VCF record. Before final
review/integration, narrow the implementation to a private shrink_to_fit method
called once by metadata::table after width validation. Keep into_rows/finish
and existing BED/VCF callers unchanged. No row-count heuristic, parser rewrite
or speculative performance claim. The original resource ceiling and fixture
stay fixed; already-live interim verification may finish, but final corrected
source requires its actual fmt/Clippy/debug/release results.

## Private integration accepted

Owner270caf52d2aa87375cfd1ab5c6ef8d0ca2acf29c commits exactly the corrected
two-file6-insertion/1-deletion feature. Index/worktree are clean afterward;
no public remote or publication change. Frozen final-delta.patch SHA-256
998d0783a79b0ba145f27890198e7df58e13e2fae8b99d204646b69587d450b0.
Final metadata.rs SHA-256
efa95dd409a8722510e83e80b211c453332ecb9e1998a11ea21a524c3f7d6f22;
text.rs12c671003c419372a6d0b0b159552a1c23a1fe889f20a968beb179461b8b5545.

Root reads the complete final source delta/report, baseline and actual resource
RED, successful final fmt/strict Clippy/debug/release logs, rehashes all15
SHA256SUMS entries and matches current source bodies. Final debug/release
each17-library+5-workflow executions, zero failed/ignored; each workflow checks
74878 raw cells. Four metadata cases occur in both harnesses, not22 distinct
scientific scenarios. All final memory observations2578544B:332979 fewer than
the unchanged2911523B baseline, within the fixed2611523B ceiling. No tests,
data or accounting were weakened. Interim universal-compaction results remain
separate, not the integrated source authority.

One fresh whole-feature review reads actual callers, ownership/accounting and
logs; no critical, important or minor findings and no fix pass. Root reads
the whole report, SHA-256
0a2d45c57025dbb7deda1e0c9d10e2a6ead6cc41b7d2892fe36c455ef769f07e.
Its declined four-native/source/RSS/whole-performance/publication judgments
are retained as separate pending gates, not implicitly accepted. The source
method attribution is corrected to Reader::read, reached by default read_into.
Report SHA-256f0d721126ccc2f763f0e3bbc30c66c10e6283e2370404c2a878ae91d34df422f;
SHA256SUMS9880093e1bdbc38dce6d9e38ed4a45e070c1cbdd44442c65311bee3d3b83dc89.
The128MiB bound applies to the retained preparation packet, not every shared
Cargo build output or existing test fixture directory; those use the existing
physical external guard and free-space checks. No kernel quota is claimed.

Next prepare an immutable green15 snapshot/selector using the existing route,
unchanged307 files/17+5 populations/scientific consumer/runtime. Preparation
alone authorizes no native/source dispatch, benchmark, download or release.
Previous green13/14 observations and negative gates remain immutable.

## Green15 preparation accepted

Preparation packet plink-green15-preparation-20261103.wPIzPTpR records the
unchanged existing three preparation tests passing, exit0, and preserves the
expected missing-selector RED. Root reads the complete selector/test-only
registration delta, snapshot README, admission and report, rehashes all frozen
report inputs, and independently compares every307 archived body and committed
Git mode against owner270caf. Exactly305 members remain unchanged from7cee;
only the two intended metadata files differ. Regular sorted USTAR metadata and
gzipmtime0 are verified. Owner/index remain clean; no scientific run occurs.

An initial root checker compares archive permissions from git archive, whose
default tar mode is0664, rather than committed tree100644. Actual Cargo.lock
bytes match. Correcting the checker to use git ls-tree modes passes all307
members without changing snapshot, source or thresholds. This is a verifier
authority correction, not a product/artifact discrepancy or permission waiver.

Owner tree0e3392246fda32bc7a569906118832c077221202. Source559710B SHA-256
0db2a250274cc02491bbc2a5b68c536191b082f65cab75e29e455c2c2502c52e;
files manifest5bd84cd59118937f2b1bf8dc8249f949f2b0e4b4a954eaa204c8e737ba95c60f;
reportf4e904812f696cc84ac99c98eb5ef3de73340fd42128bfdcfef921ec551e5966.
Native/source workflow, runtime,307/17+5/123-report/66-worker populations and
scientific tolerances remain unchanged. Commit/control CI precede any separate
native/source dispatch; no performance command or publication is admitted.

## Once-only green15 native/source admission

Controllerc3f407eba45a21bbbaa4333cec5c93f805224e28 is pushed; actual exact-head
control CI41550165726 succeeds. Root freshly reads remote main at that head
and observes no same-head plink-candidate run. Admit one existing workflow
dispatch with snapshot=green-15, all four native targets and unchanged same-
attempt Linux-artifact source consumer. Bind the actual returned run/head;
preserve failure/partial originals, no automatic retry or source/policy change.
This authorizes compatibility qualification, not timing, publication or reuse
of earlier binary measurements for the new source.

Actual dispatch exits0 and returns41550834496; fresh listing binds attempt1
workflow_dispatch to exact controllerc3f407eb. Read-only observation/retention
uses external plink-green15-originals-20261103.AefSWuqe,2GiB cap plus8GiB
independent headroom. First attempt-specific jobs API has both Linux jobs
successful, both Macs in progress, no failure. This is not native-four/source
acceptance. Preserve this exact run; no duplicate dispatch or automatic retry.

## Green15 native/source adoption

Attempt1 completes SUCCESS on all four actual native jobs and the dependent
same-attempt source consumer. No retry occurs. Root reads the complete frozen
report/current wrappers/explicit verifier adaptations, independently rehashes
all11079 packet files and recipe inputs, then replays the entire adapted data
verifier against immutable270caf Git bodies and committed modes. All four
native debug/release profiles retain17+5 executions,36 generations,74878 raw
cells,246 reports and66 workers, zero failed/ignored. Source224 report plus36
raw bridge calls succeed and bind this attempt's actual Linux TAR/manifest.
The replay compares three existing receipts without overwriting them; no
scientific code is executed locally. Source/native compatibility is adopted
in `.autopilot/oracles/plink-native-2026-11-03-green-15.json`.

Linux release3337200B SHA-256
46af1f4220bc9164732fed7a76a71254d85c0030df76f4970fdafd8b649bf755,
artifact13191022217. The sealed original proposal stays root_adopted=false;
the new canonical receipt records root's independent adoption. Official runtime/
source correspondence, target-host provider support, RSS/time, whole-performance
and publication remain separate gates. Old7cee measurements are not reassigned
to this binary; original green13 negative evidence remains unchanged.

Data-only preparation of a matched7cee→270caf full-I-w1 comparison is delegated
using the established uninstrumented12-call runner and unchanged input/output
pins. Prospective new arena-ci-comparison child is512MiB within existing8GiB
root/joined618GiB envelope; no new root, hidden worker, dependency, instrumentation,
SSH stage, provider check or runtime is authorized by preparation alone.
