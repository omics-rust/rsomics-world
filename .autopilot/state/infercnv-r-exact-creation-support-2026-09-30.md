# R exact creation helper gate

Status: root inspected actual remote RED, parse and package-free GREEN.
Independent complete-code review approved integration only. Main push and
exact-head controller CI remain pending. Actual package/factory/trials are
not implemented or accepted by this helper gate.

Plan: `../../docs/plans/2026-09-30-infercnv-r-exact-creation-support-plan.md`.
Parent spec: `../../docs/plans/2026-09-30-infercnv-raw-ingestion-measurement-design.md`.

## Exact source and real test evidence

- `scripts/infercnv_ingestion_matched_support.R`, 280 lines:
  `d583b156e9a08f3422969697b4524469b8eb05bb92d6c65d41e2aa1df7990faa`.
- `scripts/test_infercnv_ingestion_matched_pure.R`, 133 lines:
  `2b4468e94c993ea198cc3830fb3f1936b917bb817d53dfc4fa4f06162e2e7fdd`.
- All reused R/Python bytes remain identical to accepted controller/source.
  Root compared all local/remote before/after SHA256 rows; new helper and
  test also match their frozen external proposals exactly.
- Actual ssh 4090 native Linux R 4.6.1 test first exited 1 because the new
  helper was physically absent. Root read the original Chinese-language R
  `source -> file` diagnostic naming that exact missing file.
- After adding the helper, both complete files parsed with exit 0 and empty
  stderr. The unchanged test reached Python export validation, then failed
  because the default Python 3.10.12 lacks hashlib.file_digest. That run is
  preserved and is not GREEN.
- Selecting existing Python 3.12.11 through an exclusive remote PATH alias
  produced actual exit 0, empty stderr and the complete package-free success
  marker. No R package or other software was installed; no assertion,
  export, comparison or script byte was changed to obtain success.
- Pure tests exercise exact expr/count bits, one-ULP/signed-zero mutants,
  integer/double storage, literal TSV, roles/ordered maps, metadata without
  live state, opening/preservation errors and checked RSS/self/child clocks.
  A test-only S4 class is not an infercnv factory; no fake factory is called.

Evidence is preserved externally at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-ingestion-measurement-2026-09-30/r-helper-4090/`.
Original missing-helper stderr SHA256:
`f308e7a819c2fa4b7fc99d3e6fdd2a6c51ec63e050ac6cfe64d7071de02c9938`.
Complete parse stdout SHA256:
`c174a520a4e38b2e43b7388df06866244b8cbd452aecbe588d6b5a74b029f64f`.
Actual final GREEN stdout SHA256:
`c19be6bef9fae05f1b8c1989f3964a0be105afd5a4cb23ef308b4b269eee0644`.
The failed fixture (including nested Python traceback and actual exports) is
kept as `failed-pure-fixture.tar.gz`, SHA256
`452abe7bc33a57e5eaa62e45fde36e5bf901d1f8b4ddcc5172cea4feee710bac`.
Remote source hashes, machine/R/Python identity and actual proc.time help are
preserved in `remote-after-runtime.txt`.

Ruling: select existing Python >=3.11 rather than weaken the accepted
file_digest-based checker — the checker requires this runtime and all bytes
remain frozen — cost if wrong is claiming a run with different executed
code, prevented by exact before/after source checks and preserved runtime.

## Review boundaries and next component

Independent review found no factory-substitution or lifecycle defect in the
external complete trial proposal. It identified an Important executed-code
identity gate: hashes recorded only at the end do not prove executed scripts
or checker import-chain identity. It also required actual C locale and clock
resolution provenance. Those findings constrain the subsequent complete
installed-package/driver plan; do not interpret helper success as fixing or
accepting them. The real trial and package tests remain frozen scratch only.

Installed R 4.6.1 help was read in full: Unix-like proc.time values are rounded
down to milliseconds; accuracy remains host-specific. Record this, actual
package paths/versions/options and all original warnings in later trials.
I/O, full trial inventory, original artifact authentication, decoded gzip
equivalence, 32 fresh processes/64 results and strict plain/gzip wall criteria
remain separate gates. No raw-ingestion performance or whole inferCNV claim.
