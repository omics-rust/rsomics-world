# Metagenomics compact-classification candidate

Owner: `/Volumes/KIOXIA/Documents/omics-rust/rsomics-metagenomics`.

- `9bfb17949a9cb7e9bf128ad0694b5475b3712ef6`: complete nucleotide single-end
  database build/inspect, classify and report workflow.
- `c5f5a5fb53c0eae1de97bca6c860cef04bb81295`: mandatory pinned Kraken oracle
  on four native targets, alongside VSEARCH.
- `be46071cdc2ff639a166c0315b7459ef13be8736`: 0.2.0 candidate scope, version
  and measured performance record. This is the pushed main head, not a tag.

The same product retains its existing amplicon workflow. The new CLI uses
common/help/seqio and an ordered local Rayon worker pool. No new public
foundation, Layer B dependency or micro-crate was introduced. The versioned,
checksummed native database retains compact fingerprint, probe and collision
LCA semantics; raw Kraken database interchange is not supported.

## Local verification and review

Packet: `/Volumes/KIOXIA/Developments/tmp/metagenomics-native-20261105.TW2Bmae0`.
`red.log` shows the connected CLI workflow fail before implementation because
the database command is absent. Final formatting, strict Clippy, rustdoc,
debug and release all-target/all-feature suites pass under guarded Rust1.91.
Debug and release each pass 23 tests with no skips, including complete
five/six-column oracle outputs, every compact cell for the fixed collision
profiles, one/four-worker identity and the existing VSEARCH2.31.0 suite.
The exact 0.2 metadata candidate repeats the full release pass
(`release-candidate-02.log`). Invalid taxonomy/database/query regression
preserves the affected destination; the small malformed-query fixture does
not prove rollback after an already-written batch.

One whole-feature review of `34e60b5..c5f5a5f` found no critical or important
issue requiring a fix pass. It noted that the report validates hit-list tokens
without reconciling their run lengths to the sequence length and k. That
does not change counting of the assigned taxon column. No broad malformed-
input or biological-accuracy claim is made.

## Representative performance

Packet: `/Volumes/KIOXIA/Developments/tmp/metagenomics-classify-perf-20261105.HaqAWrwA`.
Apple M2, real b37 chr22 plus NCBI K-12 MG1655 references totaling 55,946,218
bases and a 42-node lineage subset. One million deterministic 150-base
queries include 900,000 verified reference windows, 80,000 random controls
and 20,000 all-N reads. Both tools use the pinned Kraken2.17.1 nucleotide
profile, identical k35/l31/spaces7, sampled capacity and explicit 16384
block/subblock. This is not a full NCBI taxonomy or biological-truth benchmark.

One warmup and five alternating measured pairs per lane, 24 pairs total.
Complete classifications, reports, decoded taxonomy and all 16,653,531
compact cells match across tools and worker counts. Native/source median
build wall is 4.373/7.869 s at one worker and 3.757/8.039 s at four; ranges
are disjoint. For classify plus report, one worker improves wall and process
RSS. Four workers use 75.5/309.4 MB maximum-process RSS, with disjoint ranges,
but 5.388/4.896 s median wall, about 10% slower. All four lanes qualify on
at least one axis. No general speed or aggregate-memory claim follows.

The measured binary predates the 0.2 metadata bump; its production bodies
match `9bfb179`. The owner's `benchmarks/2026-11-05-classification.md` links
the raw samples, executable, inputs and complete-output hashes. The original
completion receipt is tracked byte-for-byte as
`.autopilot/oracles/metagenomics-classification-perf-2026-11-05.json`,
SHA-256 `570bd9b147e84bb351fe5c7ba838626ee7a2dab9ab20dab88fa84504cd40d0ee`.

## Native CI and remaining delivery gates

[Run 41972846684](https://github.com/omics-rust/rsomics-metagenomics/actions/runs/41972846684)
is a successful main push run at exact `be46071cdc2ff639a166c0315b7459ef13be8736`.
Job IDs: Linux x86_64 127064819336, Linux aarch64 127064819660, macOS aarch64
127064819694, macOS x86_64 127064819779. The public job steps show native
host checks, pinned Kraken2.17.1/VSEARCH2.31.0 setup, debug and release tests
on all four; Linux x86_64 also passes fmt, strict Clippy, docs and source
packaging. This is not proof of installed distribution binaries.

The real upstream empty-query `--report-zero-counts` witness in the local
packet's `empty-report-oracle/` returns zero counts with `nan` percentages.
The candidate standalone report rejects a zero denominator. Hold that
boundary and the release until policy is resolved; the accepted BED repair
does not decide this unrelated policy. Clean-source installed four-native
lifecycle, license/artifact adoption and public installation are still
required. No 0.2.0 GitHub release or registry publication was attempted.
Do not retry the unchanged known-invalid registry secret.
