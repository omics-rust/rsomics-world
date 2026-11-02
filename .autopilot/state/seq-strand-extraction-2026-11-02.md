# Sequence strand-extraction execution

Active plan: `docs/plans/2026-11-02-seq-strand-extraction.md`, committed as
97dcb2812526dd5ad52ffb0b86d6abb725166d83 with exact Control41422350129 SUCCESS.
One isolated worker owns source/test edits in rsomics-seq, starting clean at
d9734e51c4ed557f6d8790d97a686717ebc4769e; it may not stage/commit/push/publish.
No current implementation or release acceptance is recorded yet.

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
