# Accepted shipped-example upstream oracle — 2026-09-26

Accepted for native pre-clustering conformance, not as biological ground truth,
performance evidence or completion of the inferCNV workflow.

## Identity and preserved bytes

Run `36221554790`, attempt 1, exact world head
`4afd7558a2407a5c43582ab473137d3271a9ad9f`, completed successfully. The single
oracle job and every step succeeded, including actual installed infercnv
execution and independent validation of both synthetic and shipped bundles.
The source head passed Control plane run `36221478265` before dispatch.

Artifact `10898668548`, `infercnv-oracle-36221554790-1`, 270,005,932 bytes:

- Original ZIP and GitHub API digest:
  `334508a7ac969165f9d5cbe4d781d81a1fd5e066f4fdb395a4efe4a81d9af25c`.
- Original logs ZIP:
  `926238c72abb08677f957ad7b89e7936e3d9ca1368a5a808eb6a6a404213779e`.
- Shipped sorted manifest:
  `524bf2544af1098968311dcd3d358a7a8439a2f1cde56fb378cb1d8114f3cd97`.
- Rerun synthetic manifest:
  `7906ae8dd9139df988a19062e3950c12bca71295fab9fb9f9359f7485b48c254`.

Original API run/jobs/artifacts records, single-artifact record, ZIPs,
extracted contents and readonly revalidation logs are retained at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-shipped-2026-09-26/run-36221554790/`.
The machine receipt is `../oracles/infercnv-shipped-2026-09-26.json`.

The controller checked exact repository/head/run/attempt/status/workflow/event,
all job steps, artifact association/name/size/nonexpiry and API digest. Both
ZIPs passed CRC, path, collision and member-type checks; all 391 extracted
artifact members match their original ZIP bytes. The shipped and synthetic
manifest entries match the complete respective file sets and per-file hashes
(90 and 167 entries, excluding their manifests). Twenty original shipped RDS
checkpoints are present and nonempty; the Python checker does not parse RDS.

## Runtime and actual results

Pinned infercnv 1.28.0 source commit
`b421d9405c97a309b081ef86d455e976df93eae4`, source archive and three original
input hashes match the binding design. Runtime: R 4.6.1, Ubuntu 24.04.5,
x86_64, OpenBLAS 0.3.26; full session and installed-package metadata are in
the bundle. Original counts are fractional and the shipped coordinates are
explicitly abridged example coordinates.

| Case | Incoming genes | Retained genes | Cells | Retained chr21 genes |
|---|---:|---:|---:|---:|
| chr1/chr19/chr21 subset | 1,775 | 1,487 | 184 | 90 |
| Full, excluding chrX/chrY/chrM | 9,939 | 8,508 | 184 | 90 |

Every stage `1,2,3,4,8,9,10,11,12,14` is present. The four independent
stage-1/2 ordered-gene hashes exactly match `infercnv-shipped-input-review-2026-09-26.md`.
Readonly execution of both committed validators passed from preserved bytes,
without creating or replacing either manifest. This verifies exact canonical
stage-1 values and unchanged retained stage-2 values, not just dimensions.
The reviewed five raw-decimal one-ULP conversions remain isolated at the pinned
R-canonical bridge; no downstream tolerance was relaxed.

All 60 shipped expression/gene/cell TSV files are byte-identical to the earlier
completed R computations in failed run `36220439929`. That run remains failed
and unaccepted; the newly successful run supplies independent valid provenance.
All 120 synthetic expression/gene/cell TSV files are byte-identical to accepted
run `36216764707`, verifying that extraction of shared exporters did not alter
the old numerical oracle.

## Independent numerical diagnostic

The independent reviewer derived reference-bounds subtraction, clipping and
chromosome-local triangular smoothing directly from pinned source, without
importing the checker or Rust implementation. Its diagnostic was executed on
the earlier R output. All consumed expression/identity/annotation files match
this fresh bundle byte-for-byte, permitting reuse of that bounded diagnostic.

- Exhaustive stage-8 bounds subtraction: maximum absolute difference
  `1.7763568394002505e-15` in each case.
- Exhaustive stage-12 bounds subtraction: `1.1102230246251565e-16` in each case.
- Exhaustive clipping: exact; 22,227 subset and 138,239 full entries changed.
- Sampled smoothing: 21,344 subset and 66,792 full entries, including every
  chr21 entry and other chromosome edges/transitions; maxima
  `6.661338147750939e-16` and `1.1102230246251565e-15`.

The diagnostic's `1e-9` tripwire is not the native acceptance criterion.
Its source/report and per-input hashes are preserved in external
`evidence/infercnv-shipped-2026-09-26/independent-numeric-review/`:
script SHA-256 `4e02a34afe7ace0622d4bb8129851f2117096a28c1d4a280ec322ad7d0663d3b`,
report SHA-256 `de5eb7b5b932c30d981d06a7c02bca7b80fbfa0220c4bda0c3a814e1198773e7`.
It does not independently recompute every smoothing interior or every pipeline
stage and does not establish biological validity.

## Remaining gate

No Rust shipped-data conformance result exists yet. The external-reader and
artifact-acquisition changes need final integration, independent review and
four-native debug/release execution at the unchanged
`1e-12 + 1e-12 * abs(expected)` criterion. Product remains unpublished,
library-only, with no CLI, HMM, downstream clustering or performance claim.
