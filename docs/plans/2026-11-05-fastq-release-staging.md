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
- [ ] Wire the FASTQ owner to stage original bodies on an exact-head `main`
  run with read-only permissions. A separate version-tag gate may publish
  the same six assets after verifying draft identity, size and digest. Do
  not use a stale registry credential, move tags, or modify older releases.

The staging tests fail before the helper exists, then pass against four
fixture ZIPs. The full control suite runs 548 tests with two resource skips.
Read-only review found no Critical or Important issue. Additional receipt
boundary cases can reuse the upstream auditor's tests; staging adds no
independent audit or publication authority. Invoke it in the audit job while
the original ZIPs are available, not from the receipt-only artifact.
