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
