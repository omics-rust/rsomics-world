# SDD ledger — plan: docs/plans/2026-10-15-infercnv-python-identity-capture.md

Spec: docs/plans/2026-09-30-infercnv-raw-ingestion-measurement-design.md.
Status: implementation b62a8fbbf8e0c9bfad908c70aa96c9ceb82c3c32 reviewed
clean; exact-head controller Linux CI pending. Metadata predecessor CI38706864188
passed at1912eb9c64e63e60dcdcb7dcd1567c9c7f661626.

| Task/interface | Producer/consumer | Preflight finding |
| --- | --- | --- |
| Task1 self | two stdlib files, argumentless capture/pure parser tests | no fake success mode or project imports under -I |
| Task1/current metadata | exact7-field full Python profile | metadata still checks projection/shape only, owner proves equality |
| Task1/future owner | same script run actual shell/R processes | source/executable hashes authenticated before/after by owner |
| Task1/old workflow | existing real initial mismatch test stays unchanged | later integration can reuse helper without changing accepted R source |
| Task1/platform | actual Linux procfs, pure portable unit cases | local Mac has no procfs and cannot masquerade as live Linux evidence |

Ruling: one small real capture script, reusable inside/outside R, rather than
copying the inline probe or another R runtime producer into the paired owner.
Cost if wrong: named-libpython filtering may need adjustment for another
supported environment; renamed/deleted/malformed runtime state fails or stays
explicitly outside supply-chain scope. No public API is locked in.

The existing initial workflow probe precedes dependencies. This capture is a
prerequisite, not proof that the later owner has already implemented after-
dependency or campaign-wide probes. No new factory/performance/release claim.

Implementation: only the two assigned files. Genuine missing-module RED,
then16 focused tests and334 full controller tests with one macOS skip of the
actual Linux success case; no failures. Full report and review diff are in
`.superpowers/sdd/2026-10-15-infercnv-python-identity-capture/`. Fresh independent
spec/code review found no Critical, Important or Minor findings. Root read both
complete files and the report. No local build/R/install/download occurred;
APFS remained above80%, all test scratch/bytecode policy stayed external.

Root also copied the exact two files to the unique retained remote directory
`4090:/dev/shm/rsomics-python-identity-20261015-dCA2R3no/`. Its native Linux
Python3.10.12 focused suite passed16 tests with one expected skip of the
absent-procfs failure case; actual isolated capture ran successfully. An
independent actual `/usr/bin/Rscript`4.6.1 base-R invocation of the same
argumentless `-I -B` script matched the shell's complete seven-field JSON
byte-for-byte with empty stderr. Original output is retained at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-ingestion-measurement-2026-09-30/python-identity-b62a8fbb/4090-base-r-identity.json`.
This establishes only the real base-R launch plumbing on4090/Python3.10, not
the campaign's required Python3.12, installed infercnv namespace context or
after-dependency/campaign-wide identity gates. No remote packages installed,
no originals removed; remote scratch is retained and ephemeral across reboot.
