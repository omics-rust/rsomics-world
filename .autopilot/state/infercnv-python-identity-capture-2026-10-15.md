# SDD ledger — plan: docs/plans/2026-10-15-infercnv-python-identity-capture.md

Spec: docs/plans/2026-09-30-infercnv-raw-ingestion-measurement-design.md.
Status: implementation b62a8fbbf8e0c9bfad908c70aa96c9ceb82c3c32 reviewed
clean; exact-head controller Linux CI38708622261 passed at
e1f6fdb90c99891a7e4cd9bd6c8798f860a7a41e. Metadata predecessor CI38706864188
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

Root independently checked the original run/jobs JSON and log ZIP for
CI38708622261: exact pushed head, push event, workflow path/repository,
successful unique controller/architecture steps, ZIP CRC,334 tests in22.975s,
OK(skipped=1), no FAILED/ResourceWarning. On Linux the sole skip is the
absent-procfs failure test; actual-success capture cannot skip merely because
procfs is absent. Original API/log files are retained under
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-ingestion-measurement-2026-09-30/run-38708622261-python-identity-ci/`.
Log ZIP SHA256:
`cdc0a2622fe126d1a4f529988498996c2f5e5c38ef0e9e103fffde23435701fe`.
Implementation source SHA256 remains
`654cd3aede60d5aff366b96ad85cd5ecc0a8e6130a35905c752e973f99eb07ab`;
test SHA256 remains
`7e71f47c2c1f82679068908b4563be63993d2285044da3f1c5f654b3dd47fd74`.
This component is accepted within its bounded scope, not paired performance
acceptance. The real owner is now delegated under its separate tracked plan.
