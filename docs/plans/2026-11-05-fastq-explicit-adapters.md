# FASTQ explicit adapter trimming

> Execute inline on owning main. Do not push or release the draft while
> the one-indel compatibility policy is held.

**Goal:** Deliver explicit single-end and independent-mate paired adapter
trimming within `run` and `trim`, for named sequences and a shared FASTA list.
**Architecture:** A checked immutable product-local adapter plan, a
non-allocating per-read match stage between poly-G and poly-X, and the
existing ordered SE/PE engine and output transactions. No new dependency
or shared foundation.
**Spec:** `AGENTS.md`, `docs/10-products/sequence-fastq.md`, and the
[pinned fastp source](https://github.com/OpenGene/fastp/blob/23d6211d4f05d61f561899f1b7702435a4b5d408/src/adaptertrimmer.cpp).

## Contracts

- Owner base `c20874ecb4c6561ff9ebcfc4c759bdacc88524be`; use Rust 1.91,
  rsomics-help, rsomics-common and rsomics-seqio. Build and store evidence only
  on physically verified external disks.
- Opt-in explicit R1/R2 sequences are uppercase ACGT of at least four bases;
  omitted R2 inherits R1. A FASTA list is shared and applied in bytewise
  header order after last-wins duplicate headers and sequence deduplication.
  Check FASTA and its minimum length at the input boundary.
- Retain the first bounded-Hamming hit, nominal overlap and negative-start
  rules. Apply fixed/quality, poly-G, explicit adapter, list, then poly-X.
  Count actual bases removed and one adapter match per read; drop dimer
  reads/pairs using the pre-poly-X remainder and adapter evidence. Named
  output keeps the existing no-clobber contract. The JSON report is native,
  not a fastp JSON clone.
- This manual profile does not enable auto-detection or infer adapters from
  paired overlap. For comparison, force upstream overlap length above both
  mate lengths. The separate fastp default PE witness trims both 47-base
  reads to 37 without a supplied-sequence match.
- Fastp's one-gap fallback computes length at each candidate offset but
  matches from read start. The pinned executable trims an inserted adapter
  at the head yet leaves the same adapter with a nonzero offset intact.
  The one-indel branch and integration gate await the user decision.

## Implementation and acceptance

- [x] Run five real fastp 1.3.6 witnesses: sequence/partial/dimer/gap, PE
  default versus forced independent matching, and two FASTA orders. Retain
  complete output and reports in `/Volumes/KIOXIA/Developments/tmp/fastq-02-exact-20261105.Z25dkh67/tmp/fastq-adapter-witness-20261105.9CYoQ4Xa`.
- [x] Add one connected RED test in `tests/fastp_compat.rs` for complete
  SE/PE FASTQ, FASTA ordering and physical-base/dimer metrics. Run it with
  `RSOMICS_REQUIRE_FASTP=1`; expect unsupported adapter CLI options.
- [x] Add `AdapterConfig` and ungapped matcher in `src/transform/adapter.rs`,
  FASTA loading via seqio, CLI flags, and adapter evidence/counters in the
  existing trim and pipeline paths. Test both `run` and `trim` without
  duplicating the old transform or mate handling. Keep this draft local.
- [ ] Apply the recorded one-indel decision and add the affected witness.
  One fresh whole-feature review checks short negative matches, offset
  precedence, R2 defaults, list order, and discarded-pair/report contracts.
- [ ] Measure full output on representative single- and paired-read
  workloads with an observed affected fraction and 1/4-worker identity.
  Use identical input, flags, complete output, one warmup and five
  alternating samples; retain wall/CPU/RSS and exact identities. A starting
  SE profile is the first 1.1 million verified SRR341550 R1 records with
  deterministic 12/20/33-base adapter suffixes on every third read.
- [ ] Run fmt, strict Clippy, full live-oracle suite, and exact-head four
  native CI before promoting the operation. Publication remains separate.

The retired `rsomics-fastq-trim` revision `30b07d34d028` supplies fixture
ideas; its suffix scoring is not the source matcher. Old fastp and BBDuk
variants do not supply this compatibility contract.

## Local draft and benchmark inputs

The [preflight receipt](../../.autopilot/oracles/fastq-explicit-adapter-preflight-2026-11-05.json)
binds the pinned oracle, live SE/PE/list witnesses, the seven uncommitted
owner paths and the four verification logs. RED was the missing CLI
option; GREEN compared complete outputs for named SE, shared FASTA, and
independent paired R2 inheritance/override. The full debug suite passes
57 tests with the pinned oracle, plus formatting and strict Clippy.
The new path counts actual adapter bases per attempted mate; upstream
reported 56 where 48 observed bases were removed in the full SE witness.
Neither those report semantics nor skipped-mate counts are advertised as
fastp JSON equivalence. Cargo metadata, lockfile, README and CI are unchanged.

Input packet: `/Volumes/Zane's HDD/rsomics-fixtures/fastq/fastq-explicit-adapter-20261105.xay9xz5t`.
The SHA-256-pinned manifest covers 1.1 million real SE reads (366,666 designated
suffix injections, 366,514 changed sequences) and one million preserved
synthetic pairs (500,000 changed per mate). Headers, lengths and qualities
are unchanged. The base inputs were rehashed before preparation; the
manifest SHA-256 is
`9e2377f5a6d56336c381e6775582569e67e75d751ed96bda9cfd80b9e0c640c7`.
These are injected counts, not observed adapter matches or performance.
The dirty draft is not a release candidate; resolve the one-indel policy
before measuring and qualifying the intended executable.
