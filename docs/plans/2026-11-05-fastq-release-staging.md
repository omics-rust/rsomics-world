# FASTQ GitHub release staging

> Execute on the authorized main checkout; verify before publication.

**Goal:** Assemble the six public FASTQ 0.2 files from the original, audited
native carriers. **Architecture:** `stage_native_release.py` consumes
`AUDITED.json` and the four original ZIPs retained by the pinned auditor.
It writes a new packet with a separate staging receipt, without running an
executable from a carrier or calling GitHub's release API. **Tech stack:**
Python 3.12 and the existing audit helper. **Spec:** `AGENTS.md` and
the [native audit contract](2026-11-05-native-original-audit.md).

- [x] Prove that exact source and carrier bytes and their checksums are staged
  and that a changed ZIP or carrier yields no complete packet.
- [x] Implement and verify the staging command with the existing audit
  readers. It must bind all four targets, run and attempt, deduplicate one
  identical source crate, and create a receipt only after all bytes agree.
- [x] Verify the FASTQ owner's exact-head `main` run stages the original
  bodies with read-only permissions and retains the six assets plus receipt.
- [ ] Add and test a separate version-tag release job, then verify the full
  hosted gate and anonymously install the public macOS arm64 carrier. Do
  not use a stale registry credential, move tags, or modify older releases.

The staging tests fail before the helper exists, then pass against four
fixture ZIPs. The full control suite runs 548 tests with two resource skips.
Read-only review found no Critical or Important issue. Additional receipt
boundary cases can reuse the upstream auditor's tests; staging adds no
independent audit or publication authority. Invoke it in the audit job while
the original ZIPs are available, not from the receipt-only artifact.

Hosted control-plane run [41999655845](https://github.com/omics-rust/rsomics-world/actions/runs/41999655845)
at `825414b5` failed in the script suite, before ledger validation. The
public annotation only gives exit 1 and the anonymous job log returns 403.
The local 548-test pass does not explain that failure. Hold owner wiring
until a qualified helper pin is available; expose the suite failure and
runner identity without changing the test selection.

The diagnostics-only `46f68f15` passed exact-head control CI
[42000434013](https://github.com/omics-rust/rsomics-world/actions/runs/42000434013).
The first failure's cause remains unknown; no production or test-selection
change was made. FASTQ uses that pin at `9b9c793`, exact run
[42000874546](https://github.com/omics-rust/rsomics-fastq-preprocess/actions/runs/42000874546),
with no new permissions and no release or registry step.

Run 42000874546 attempt 1 passed all four native installs. In
[job 127158522546](https://github.com/omics-rust/rsomics-fastq-preprocess/actions/runs/42000874546/job/127158522546),
original ZIP verification, release staging, SHA256SUMS checking, and both
uploads succeeded. The attempt-bound staged artifact is `13402423264`,
3,361,245 bytes, ZIP SHA-256
`29e28e2785044d7c14167112fb899832c1baea1aaeced638d4d74006789acb26`.
This digest identifies the GitHub Actions ZIP, not its internal `STAGED.json`.
The Mac has not downloaded the ZIP. `9b9c793` only changes CI from the
measured production fix; no hosted binary is relabeled as benchmarked.
No tag was pushed, and the registry workflow remains untouched.

## Tag publication contract

Only a deliberate new `v0.2.0` tag on the qualified owner main commit may
enable the GitHub write job. Its own four-native and original-audit results
must succeed. Bind the downloaded staging receipt to the audit job's
reported SHA-256, exact repository/head/run/attempt, and all six file bytes.
The staging receipt is internal evidence, not a public asset or authority.

Give only the tag release job the repository's short-lived `contents: write`
permission. Refuse any pre-existing release or draft for that tag. Bind
creation to the verified remote tag and draft ID. Verify uploaded names,
sizes and SHA-256 digests at that ID before making it public. A failed upload
leaves the draft for inspection rather than deleting or replacing assets.
After publication verify the public tag, all six asset bodies and an
anonymous fresh install. Keep crates.io dispatch separate and unchanged.

Use the [paginated release list](https://docs.github.com/en/rest/releases/releases#list-releases)
to discover drafts and the [release-ID asset list](https://docs.github.com/en/rest/releases/assets#list-release-assets)
to verify the draft. Use the returned, validated GitHub upload endpoint and
raw file inputs; do not perform tag-based overwrite or reuse an old draft.
