# Native delivery original-artifact audit

> Execute inline on the authorized main branches. Review the whole gate
> before push. This is a verification job, not a publishing workflow.

**Goal:** Close the evidence gap between a green four-native install matrix
and verified uploaded bodies, starting with FASTQ 0.2.

**Architecture:** The existing pinned `native_delivery.py` produces the
source, install and carrier chain. A post-matrix job uses same-repository
`actions: read` to fetch the four original ZIPs and a private control-plane
script to recheck their complete SHA-256, exact Git source, binaries, and
notices. Store a receipt only after all four pass.

**Tech stack:** Python 3.12 standard library, curl over HTTPS, Cargo 1.91
offline metadata, and the existing GitHub Actions delivery contract. No new crate, token,
external proxy, artifact visibility change, or publication permission.

**Spec:** `AGENTS.md`, `.autopilot/state/fastq-native-delivery-2026-11-05.md`,
and the actual `SOURCE.json`, `BUILD.json`, and lifecycle contracts.
The retained four original BED 0.6 ZIPs independently match their official
API digests; they are transport evidence, not FASTQ qualification.

## Task 1: Real archive verifier

- [x] Write RED tests in `scripts/test_audit_native_delivery.py`: require
  one artifact per native target from the same run/attempt; admit a checked
  real ZIP/tar/source fixture, and reject mismatching ZIP, binary or notice
  bodies without producing an audit receipt.
- [x] Add `scripts/audit_native_delivery.py`. Verify ZIP size and SHA before
  parsing; read regular members without extracting binary code. Reuse the
  producer's exact-Git package verifier and bind all carriers to the
  same clean source, actual installed executable and locked notices.
  Use the exact run ID, commit and attempt in names and receipts.
- [x] Close the independent review findings with real changed archives:
  nested crate expansion is bounded, and full normalized metadata is
  compared independently with the Git checkout. Both issues are RED then GREEN.
- [x] Run GREEN and the control-plane suite on physically verified external
  scratch. Replay the exact local FASTQ package as a local sample, never as
  a hosted original. Obtain one independent review and verify its fixes.
- [ ] Commit and push the gate; verify exact control-plane CI.

## Task 2: Exercise in the owning workflow

- [ ] In `rsomics-fastq-preprocess/.github/workflows/ci.yml`, pin the checked
  control revision. Add the attempt to four artifact names and an audit
  job requiring the full native matrix, on trusted main pushes only.
  Give only that job `actions: read`; pass the repository token only to
  the verifier step. No release/tag/registry side effect.
- [ ] Push one owner CI commit. Observe the exact head, attempt, all four
  native jobs and the original-body audit. Record actual artifact IDs and
  audit receipt metadata. Do not claim local download or public install.
  Metagenomics is the second concrete user of this existing delivery
  format; its distinct zero-read policy and release gates remain separate.
