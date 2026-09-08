# Canonical short-window repair checkpoint

The unpublished kmer 0.2.3 candidate is approved for the narrow correctness
release. Exact-head package verification passed on Linux x86_64; native tests
and both real consumers pass on all four platforms. All eight consumer performance
bundles are verified and permanently retained. Release acceptance is based on
the measured resource-use advantage, not universal throughput or a causal
guard optimization. One publication attempt failed registry authentication;
kmer alone was added to the selected secret-access list. Registry remains 0.2.2.

The authoritative detailed evidence, rejected experiments and release decision
are in `docs/01-foundations/kmer-consumer-review-2026-09-09.md`.

## Owning repositories

- Kmer: clean `c79111f31651bad581920b7b5c2ddde9a57534e1`, candidate 0.2.3.
  The sole production delta from registry baseline `d89e2df` is
  `len < k || start > len - k`. No public API, iterator state, hash or
  allocation-policy change; no inlining hint remains.
- Sketch: clean `f430522bb3d3c6fd38e08af758dca11e5262f47b`, still using
  registry kmer 0.2.2. Its normal CI deliberately expects the old short-input
  panic. Scale recovery and collection comparison are repaired in source;
  see `docs/10-products/sketch-contract-review-2026-09-09.md`.
- Seq: unchanged `d9734e51c4ed557f6d8790d97a686717ebc4769e`.
- VCF: inherited dirty concat work atop `682942c` is untouched.

## Candidate and consumer evidence

| Gate | Exact run | Verified outcome |
|---|---|---|
| Candidate ordinary CI | `34271886882` | Kmer `c79111f`; four native test jobs plus strict lint, package verification and benchmark smoke |
| Candidate consumers | `34272105400` | Kmer 0.2.3 `c79111f`; seq `d9734e5` and sketch `f430522`, all eight native jobs |
| Prior repaired-product consumers | `34269918433` | Workflow head `1a0d9aa`; same product pins, all eight jobs |
| Sketch scale red/green | `34266897367` / `34267432784` | Four-native expected failure then normal repair pass |
| Sketch collection red/green | `34267814268` / `34268416605` | Four-native expected failure then normal repair and mixed-scale oracle pass |
| Sketch measurement-head ordinary CI | `34261359503` | Product `3802b1a`, four native tests plus release checks |
| Complete construction measurements | `34261423289` | All eight genome/FASTQ native jobs; final Intel FASTQ job `102180058887` succeeded |

Candidate consumer archives match all GitHub digests/sizes, pass CRC and match
extracted bytes. Metadata, lockfiles and Cargo configuration select one exact
kmer 0.2.3 Git identity. Actual checkout and native host triples were checked.
Sketch's full four-test oracle executes in debug and release on every platform;
seq's independent kmer oracle and six required live SeqKit contracts do too.
All 13 ordinary/consumer raw logs were independently reviewed.

Permanent root:
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/kmer-short-window-2026-09-09/`

- `consumers-34272105400/`: final candidate archives, metadata and 13 raw logs;
  scratch `/Volumes/KIOXIA/Developments/tmp/kmer-consumers-34272105400-pix9ml/`.
- `consumers-34269918433/`: earlier source-head consumer evidence;
  scratch `/Volumes/KIOXIA/Developments/tmp/kmer-consumers-34269918433-Yvr8UM/`.
- `sketch-construction-34261423289/`: all eight measurement archives,
  extracted binaries, lockfiles, metadata, actual compiled features, complete
  output bytes, raw resource values and logs;
  scratch `/Volumes/KIOXIA/Developments/tmp/sketch-construction-34261423289-mjkyFe/`.

All permanent copies were recursively compared with scratch. Do not overwrite
them. Earlier negative measurements remain in `first-guard`,
`invariant-boundary`, `inlining`, `equal-source`, `interleaved-control`
and `shared-scan-control`. Their exact handles and limitations remain in the
dossier; no failed run has been relabelled or removed.

## Performance decision and scope

Measured product `3802b1a` uses three pinned kmer selections: published 0.2.2,
corrected reference `23e42f3`, and candidate `61de048`; sourmash is 4.9.4.
All product variants were archived immediately after matched ordinary release
builds, before test-build feature unification. The full oracle executes the
archived candidate and rechecks its checksum. Actual compiled external
dependency features and normalized package identities match.

Inputs are complete E. coli FASTA (4,699,745 bytes, SHA-256
`53bb6a51b6e92139ced1e38f74b7938781027c52200922ff03718c2237d23bb4`)
and SRR341550 read-1 FASTQ gzip (87,439,836 bytes, SHA-256
`d7a15c1762d64a5434ced0cc665d7f5d167ca81a71e239f8237b9cd490dd7683`).
FASTQ contains 6,282,141 complete 101-base records; separate oracle cases cover
mixed/all-short input. DNA k31, scaled 1000, seed 42, abundance only for FASTQ.
Genome has 16 measured four-tool rounds after four warmups; FASTQ has eight
after one warmup. Linux uses one allowed CPU; worker environments are one.

All eight ZIP/API digests and CRCs, 464 trial rows (384 measurements), 24
product binary/lock checksum sets, full outputs, resource values and compiled
feature records passed primary and independent review. These are paired
median candidate/sourmash time and peak-RSS ratios:

| Native runner | Genome time / RSS | FASTQ time / RSS |
|---|---|---|
| Linux x86_64 | 1.110640 / 0.063158 | 1.864916 / 0.090191 |
| Linux aarch64 | 0.515002 / 0.058569 | 0.885291 / 0.082669 |
| macOS x86_64 | 1.095327 / 0.058394 | 1.815349 / 0.096582 |
| macOS aarch64 | 0.704724 / 0.062135 | 1.360956 / 0.187212 |

Every measured candidate/sourmash pair has lower peak RSS. Accept this strict
resource-use advantage for the narrow foundation correctness repair. Explicitly
retain Linux ARM FASTQ's 3.39% paired-median slowdown against the registry-kmer
baseline, observed in all eight pairs, as a correctness tradeoff. Other slower
paths stay documented. Equal-source timing effects remain, notably Linux x86
candidate/reference 0.966135 for FASTQ; no guard-specific attribution or
no-regression claim is justified. These are single warm-cache runner jobs,
not machine replications or a universal workload claim.

Candidate `c79111f` changes only version metadata and consumer workflow versus
measured `61de048`; source, tests and benchmarks remain identical. Its source
tree is `4ea404fb294e376032ce49bcbedaac72b1b4f79e`. Distinct build identities
are not relabelled: the 0.2.3 registry artifact itself was not timed. Candidate
package, API and consumer verification supply the separate exact-head gates.
The later sketch loading/comparison changes require new product-head evidence.

## Next bounded actions

1. Continue independent control-plane and product work. Do not rerun publication
   unchanged or claim sketch has adopted a nonexistent registry version.
2. Wait for an externally updated valid organization publication credential.
   Do not read/request its value. After a confirmed change, recheck candidate
   main, current registry and quality evidence; then dispatch once and verify
   registry checksum, archive source/VCS identity and exact publish head.
3. Once publication is verified, update sketch minimum registry dependency and
   lockfile, remove its expected-failure diagnostic, and run all native product
   tests/oracles and representative final-head measurements. Publish sketch
   only after its own complete gate; do not call it delivered beforehand.

World decision `3be29dd4` passed exact-head CI `34274695274`. Remote kmer main
was verified as `c79111f`. Only kmer repository ID `1245819199` was added to
the organization secret: before/after API records prove all 17 prior entries
are preserved, total 18, visibility still selected. No value was read/changed;
sketch remains excluded.

Publish run `34274791181` at `c79111f` constructed and verified the package,
then failed uploading with HTTP 403 `authentication failed`, exit 101. The
credential was present (masked) in the environment. Its exact invalidity cause
is not established; selection alone did not resolve authentication. No retry.

Raw job/CI logs, before/after selected-access metadata and registry snapshots
are permanently retained in `publication-34274791181/` under the evidence root,
recursively matched with scratch
`/Volumes/KIOXIA/Developments/tmp/kmer-publication-20260909-2xSzvA/`.

Post-failure registry check still has kmer 0.2.2, non-yanked, checksum
`e1254977d1eaf89b29e727b7ea552ec8bd4bd0740b45fa40ac943e93ffaf9ed4`.
Secret metadata changed from timestamp 2026-08-20T01:35:00Z to
2026-09-08T20:26:08Z after the access-only request. This timestamp does not
prove a credential-value change; the value's age is unknown. No 0.2.3 is published.

## Storage and operating limits

Physical boot APFS occupancy is 94.51%; local product builds/tests/benchmarks
remain stopped. `df /` is not a sufficient physical-capacity check. Cargo,
scratch, fixtures and downloads stay on the allowed external disks. KIOXIA has
about 59 GiB available; the HDD about 227 GiB. No 4090 build, deletion,
automation or user-file staging occurred. Stop guard/microbenchmark permutations
and continue from the verified release gate above.
