# SDD ledger — plan: docs/plans/2026-10-15-infercnv-python-identity-capture.md

Spec: docs/plans/2026-09-30-infercnv-raw-ingestion-measurement-design.md.
Status: plan prepared after metadata exact-head CI38706864188 success at
1912eb9c64e63e60dcdcb7dcd1567c9c7f661626. No implementation yet.

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
