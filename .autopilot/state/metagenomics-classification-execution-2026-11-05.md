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

## Source-package reuse check

Mac ARM clean `be46071` also passes the shared delivery helper pinned by
FASTQ at World `f580be87`. Guarded offline Cargo packaging and exact Git
body/mode plus normalized metadata verification accept 48 originals and
50 packaged files. The 57,857-byte source crate has SHA-256
`22e45fdbd4e8d8ed3ec072101dac4b8252bbbcd5585e64e2fefab4428f1ac35f`.
The archive-checksum-backed native graph contains 60 dependency packages
and 124 original notice files.

Packet: `/Volumes/KIOXIA/Developments/tmp/metagenomics-source-delivery-20261105.wxPIox`.
This was the initial package/notice check, without carrier assembly. The
local install from that extraction subsequently passed the nonempty
classification workflow against retained goldens (`INSTALLED.json`, SHA-256
`04cbce27deb4db45762da62922a8449e47581250f62dfffe87ce998af5892adf`).
It did not re-execute the oracles or exercise amplicon operations.

## Installed 0.2 delivery candidate

Owner `335d1039d4a9422b0c79fc98e3093bd63b5b7750` adds the product notice,
installed CLI comparison and four-native delivery CI. No Rust source, Cargo
manifest/lock, version, benchmark or publication workflow changed. CI pins the
same World `f580be87` helper as FASTQ. Kraken's preserved MIT original
matches the pinned source archive byte-for-byte; neither oracle is shipped
as a runtime binary.

Packet:
`/Volumes/KIOXIA/Developments/tmp/metagenomics-installed-delivery-20261105.Ro996v`.
The installed gate first fails on the absent script, then passes against the
previous package-installed executable. The final clean-head offline package
contains 50 Git originals and 52 files. Installed from its verified
extraction, the executable passes eight native invocations and full
classification/report comparison with live Kraken2.17.1 plus connected
dereplicate, abundance selection and rereplication against live VSEARCH2.31.0.
The local fmt, strict Clippy, rustdoc and 23-test debug/release suites pass
with both oracles. No new benchmark or empty-report qualification is implied.

| Local exact-head evidence | SHA-256 |
|---|---|
| Source crate, 61,122 bytes | `588f74f4ec8cd817e0b8ad3daa4c9370803aaed065866554cdeb26235c3f2d04` |
| Mac ARM tarball, 936,950 bytes | `59af8623e6f19a074192c39d671f8ddcb30e9b783f6bc2945c8901d65ea76295` |
| Installed binary, 2,121,696 bytes | `20f28a084757037aecafc476eb33e205876d39b0e89618eef8a9e804215cd4e0` |
| `SOURCE.json` | `d59b42bb93aceea9e49c4ba2976159bacf8191f06f0518c7359805903a1db055` |
| `lifecycle/VERIFIED.json` | `fff8aa842deafd0072da677db136c3334c1a0651113532635e38c8f4b6c2b5f3` |

The tarball independently matches the tested executable, upstream/product
notices and all 124 indexed notice originals from 60 enabled registry
packages. The Mac ARM binary records only system libiconv and libSystem.
The fresh delivery review found no Critical/Important defect. Its minor
timeout diagnostic finding is deferred: a timeout fails before a success
receipt, but that command's partial output is not saved. Existing production
review and performance decisions remain separate.

## Hosted installed qualification

[Run 41978983755](https://github.com/omics-rust/rsomics-metagenomics/actions/runs/41978983755)
passes at exact main `335d1039d4a9422b0c79fc98e3093bd63b5b7750` on Linux and
macOS x86_64/aarch64. All four run the pinned source oracles, debug and
release suites, source package verification, installed CLI comparison,
carrier assembly and upload. The actual job, step and artifact identities,
byte counts, API digests and expiration are in
`.autopilot/oracles/metagenomics-installed-delivery-metadata-2026-11-05.json`.

The original ZIP probe for artifact13392559033 returns HTTP 401. This
verifies an access gate, not archive contents or a failed CI run. Original
carrier adoption, zero-denominator policy and public installation remain
open. No GitHub or crates.io release was attempted.

## Same-run original audit and staged candidate

Owner `9092568f06b309e0b4b5958e570514aa82a08153` changes only CI from
`335d103`. [Run 42009300914](https://github.com/omics-rust/rsomics-metagenomics/actions/runs/42009300914)
attempt 1 passed all four native tests, package installations and live
oracles. Job `127186273171` read the actual original ZIPs using the pinned
control-plane helper, verified source, carrier, license and installation
receipts, then staged four native carriers, one identical source crate and
SHA256SUMS. It uploaded AUDITED.json and the staged packet separately.

The [hosted metadata receipt](../oracles/metagenomics-hosted-stage-2026-11-05.json)
records the run, jobs, and all six ZIP IDs/sizes/digests. Stage artifact
`13406103388` is 3,988,198 bytes with ZIP SHA-256
`062bbaa83613762eaaa3606f84f875505646c7f9d389b4f58f9ac9ea710bb196`.
Anonymous ZIP download returned 401; no local adoption or public install
is claimed. The native standalone report still rejects zero input rows
where the retained Kraken witness returns eight zero-count rows with
`nan` percentages. No tag, GitHub release, registry dispatch or credential
change was made. The sparse registry index still lists only 0.1.0.
