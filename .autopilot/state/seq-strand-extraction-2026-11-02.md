# Sequence strand-extraction execution

Current: reviewed source integrated as3ba1818e85477573995f6a391522f2ef35054c79;
exact-head four-native CI41424424381 passes and raw logs are adopted. Clean packaging passes;
one representative campaign is running; performance acceptance, delivery
and publication remain unearned.

Active plan: `docs/plans/2026-11-02-seq-strand-extraction.md`, committed as
97dcb2812526dd5ad52ffb0b86d6abb725166d83 with exact Control41422350129 SUCCESS.
One isolated worker owns source/test edits in rsomics-seq, starting clean at
d9734e51c4ed557f6d8790d97a686717ebc4769e; it may not stage/commit/push/publish.
The initial handoff below predates source acceptance; the final section
records integration evidence. No release acceptance is recorded.

Scratch and worker ledger:
`/Volumes/KIOXIA/Developments/tmp/seq-strand-20261102.GMB2B7`.
Root independently compares its guard against the fully read intervals guard:
only exact owner, Cargo target and tool-cache suffix substitutions differ.
All project outputs remain on verified external devices. Inherited OpenJDK
CPPFLAGS is removed explicitly, not converted into a compiler assumption.

Installed actual oracle is SeqKit2.14.0, not the historical2.13.0. The worker
retains full-output witnesses and reports source-route equivalence for the
two selected operations across those versions, including the irregular -R
header parser behavior. Root must inspect the final source/evidence before
adopting that conclusion. Historical2.13 goldens remain separate evidence.

Contract clarification: preserve every header byte accepted by existing
seqio, including ASCII whitespace. Current seqio rejects non-ASCII header
bytes; do not widen that foundation contract to satisfy a newly authored
test. Explicit -R rewriting follows its observed oracle contract; default
headers remain unchanged. Existing grep policy stays unchanged.

Connected new CLI test genuinely fails on absent subseq before implementation.
Worker implementation/checks are in progress; no pass count is adopted here.
One fresh whole-feature review follows the completed wave, not each helper.

## Public performance input recovery

Old documented local SRR341550 path is absent. Bounded filename searches in
external fixture/retired/tmp roots find no matching R1 FASTQ; the old explicit
4090 benchmark path also lacks it. This does not prove global deletion.

The initial ENA HTTPS download was observed live as curl PID85012 and exec
session69501. It subsequently terminates with actual curl18 after1711748B.
A prefix backup is copied and cmp-verified before one bounded range-resume
in session4729; that also terminates curl18, leaving3167435B. Both failed
partials remain preserved, not accepted fixtures. No ENA download remains live.

- URL: https://ftp.sra.ebi.ac.uk/vol1/fastq/SRR341/SRR341550/SRR341550_1.fastq.gz
- Pending: `/Volumes/Zane's HDD/rsomics-fixtures/fastq/seq-strand-20261102/SRR341550_1.fastq.gz.part`
- Final: same path without `.part`, only after expected SHA256 matches
  d7a15c1762d64a5434ced0cc665d7f5d167ca81a71e239f8237b9cd490dd7683.
- Expected original size87439836B,6282141 reads; not freshly recounted yet.

The invoked guard verifies KIOXIA Cargo/Rustup/resolved target/tmp/caches.
The download additionally verifies HDD physical external mount/devicedir,
resolved destination and8GiB headroom. It refuses existing final/partial
targets, writes only the named external partial, hashes then renames without
overwrite. Partial data is not an accepted fixture; no benchmark has run.
Old source performance is not reassigned to this candidate.

## Verified input recovered from existing remote copy

A subsequent bounded filename inventory of the old benchmark root locates
`/data1/liangjy/rsomics-linux-x86_64-20260730/fixtures/SRR341550_1.fastq.gz`
on4090. This corrects the earlier checked-but-absent root-level path; the
fixture was not globally absent. Actual remote stat reports87439836B and
fresh SHA256 equals the expected d7a15c1762d64a5434ced0cc665d7f5d167ca81a71e239f8237b9cd490dd7683.

SCP under the same external preflight copies it once into a distinct
`.ssh.part` destination; fresh local full SHA256 agrees, then non-overwriting
rename creates the final path above. Actual session10638 exits0. This proves
byte identity, not a freshly recounted record population or performance.
The original remote copy and both failed ENA partials remain untouched.

## Reviewed source integrated; native qualification running

The prior in-progress description above is historical. Source integration
is now owning3ba1818e85477573995f6a391522f2ef35054c79, pushed to live main.
Root fully reads production/new tests/docs, verifies every one of the23
frozen file hashes and independently cmp-checks all13 historical fixture
copies. Manifest SHA256 is1dba8c57d2dd960b1487e32c2f71b378fd7be90008347d7396a7f0820d60bd47.
Full debug/release each have56 passes, zero failed/ignored, including eight
live suites; strict Clippy/fmt/rustdoc pass. Logs remain in the scratch above.

One fresh integrated reviewer inspects the complete frozen delta and surrounding
traversal/output/writer/classifier contracts; no P0/P1/P2 material finding.
P3 suffix allocation even without -R is deferred: one tiny allocation per
invocation, not a hot-path defect. No cosmetic correction or second reviewer.
Reviewed exclusions remain later BED/GTF/flank/wrapping/indexed operations,
existing strict header domain, incidental source warnings/index side effects
and unproven exhaustive input agreement. None qualifies release/performance.

Root additionally extends existing owning CI with actual uname/Rust-host
assertions and full release-profile tests on the same four existing runners;
no oracle revision, compiler or comparison changes. The23-file review manifest
does not include this root-owned YAML extension; production remains unchanged.

The first staged diff check identifies four intentional trailing spaces in
two copied coordinate-header golden files. They are exact source output and
were cmp-verified, not code whitespace defects. A narrowly scoped check
excluding only those two fixture files passes; preserve the data bytes.
No global whitespace configuration or source-format waiver is introduced.

After the reviewed source commit, clean cargo package --locked --offline
passes (50files157.9KiB/41.5KiBcompressed), including actual package build;
no --allow-dirty. Log: `clean-package.log` in the same scratch. Version0.2.0
remains unchanged and no registry archive is overwritten or published.

Actual exact-head owning CI41424424381 is created2026-11-02T09:54:17Z at
3ba1818e85477573995f6a391522f2ef35054c79 and initially queued. Wait for its
four actual native hosts, both profiles and pinned SeqKit2.13 witnesses;
local2.14 tests alone are not that acceptance. Representative performance
and any version bump/delivery/publication remain subsequent gates.

## Actual four-native correctness accepted

Actual41424424381/1 completes SUCCESS at exact3ba1818e85477573995f6a391522f2ef35054c79.
Root retains raw run/jobs API bodies, complete attempt log and exact Git source
TAR under `/Volumes/KIOXIA/Developments/tmp/seq-strand-native-20261102.Sj6Rdx`.
Four-file checksums manifest SHA256:
13941d36d6b32ec532ac57381028b412529e3ba8078c900f4d756266415c0370.
Root freshly verifies every digest and actual job/required-step metadata;
all four native-host checks, pinned source build and debug/release suites pass.
Raw Rusthosts are x86_64/aarch64 Linux GNU and x86_64/aarch64 Apple Darwin.
All eight profiles contain7+5+35+1+8=56 passes and zero failed/ignored.
The source revision d13b5fa388cc869de05abe1bdb07980eef5efb4e is built and
asserts SeqKit2.13.0 on every host; new live whole-record suites pass there.
Linux x86_64 also passes format, strict Clippy, rustdoc, clean package and
existing benchmark smoke. This closes native correctness, not performance.

No final native executable artifacts are uploaded by this CI; do not call
these logs binary-delivery/architecture-inspection/public-install evidence.
Representative gate is now fixed in
`docs/plans/2026-11-02-seq-strand-performance.md`. One worker prepares a
simple full-input runner/admission, but no timed campaign is yet authorized
until root inspects its paths, command/resource recording and capacity check.

## Single representative campaign started

Fresh guarded locked/offline release build uses clean3ba1818e and native SHA
12c188736686a8ba44714ad6e3500ed24585731e2099aeca538d60d827d69df9.
Actual SeqKit2.14.0 SHA is
2f7d024ddce72519d0b6dd913eb7ff6c22229487c6ede2467e237ab615b7f1c4.
Complete input admission finds6282141 FASTQ records, all101nt with equal
quality lengths,634496241 bases and1474080927 uncompressed bytes.
Conservative36-output capacity plus8GiB is61656847964B; admitted HDD free
is about77712392192B. Nothing is deleted to create that capacity.

Root reads the entire runner before authorizing execution. One privacy issue
is corrected first: blanket environment/SPHardwareDataType capture is removed
from the task-owned admission metadata without printing its values or keeping
duplicate potentially sensitive contents. No campaign output or public upload
had occurred. Root verifies the broad fields absent and one-shot marker absent;
safe CPU/RAM/count and explicit reproducibility fields replace that capture,
with hostname omitted. Existing project/scientific originals are untouched.

The authorized runner remains private scratch:
`/Volumes/KIOXIA/Developments/tmp/seq-strand-campaign-20261102.JNGmJL/campaign.py`,
SHA912d68ff339aafa6c2500e0f09bc8d5ab976f23f54be3b60dbe16a0b0815eff0.
Root checks macOS time supports the selected-l/-o resource-output options;
pipeline stages have separately observed exits, not a last-command-only pass.

Actual campaign session20441, Python PID16182 is freshly observed running.
Output root:
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/seq-strand-20261102-i6yc6i6d`.
Log: `campaign.log` in the scratch above. The trials ledger has begun; no
performance conclusion is yet accepted. Re-poll this exact handle/PID before
any restart decision. Retain all36 full outputs and48 per-stage original
resource/exit records, including failure; do not rerun unchanged on timeout
or an expired observation. No root/source edits while this campaign runs.
