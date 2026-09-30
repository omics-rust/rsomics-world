# inferCNV canonical ingestion witness — accepted

The user explicitly resumed inferCNV work on 2026-09-30 and repeated that
instruction after an unexpected Mac restart. This revokes the earlier pause;
the historical step-14 receipt remains unchanged. Full inferCNV remains
unfinished, and the native product is still unpublished at `3715e55b`.

## Next deliverable

Creation-only witnesses from installed infercnv 1.28.0, using the same pinned
source and existing synthetic/shipped validators. The
[design](../../docs/plans/2026-09-30-infercnv-ingestion-witness-design.md) and
[plan](../../docs/plans/2026-09-30-infercnv-ingestion-witness-plan.md) require
canonical plain/gzip equivalence, existing stage-1 comparison, explicit ordered
group maps, and a small fractional/order/annotation witness. Product raw-reader
implementation follows acceptance of those witnesses. Original shipped gzip
format behavior is still a separate unexecuted probe.

## Implementation and review

The R exporter, independent Python checker and workflow integration have been
implemented as a candidate. Local standard-library control-plane tests passed
119/119 with `TMPDIR=/Volumes/KIOXIA/Developments/tmp`; the new checker accounts
for 11 tests. R syntax parsing passed on 4090 using `TMPDIR=/dev/shm`, R 4.6.1.
This is syntax and checker evidence, not installed-package execution.

An independent read-only review found three substantive checker gaps: the
small matrix compared against itself, stage-1 gene/cell exports were not bound,
and a map test failed at the wrong earlier check. All three were corrected with
regression cases for adjacent decimal values, nonfinite values, changed gene
coordinates, and map-index identity. Creation arguments and checkpoint paths
were added to provenance. The candidate still needs the real-package run and
artifact inspection before acceptance.

Ruling: ship the exporter, checker and workflow together as one witness feature;
the installed-package execution is their integration gate. Routine design,
commit and push decisions are covered by the user's existing authorization.
No raw-input numerical tolerance was widened, and no generic R-decimal parser
or public foundation was introduced.

## Storage and restart recovery

After reboot the APFS container was 245,107,195,904 bytes with
26,971,713,536 bytes free (about 89% occupied). KIOXIA had 43,468,980 KiB free.
Local Cargo/R builds and dependency downloads remain stopped. Native compilation
and the installed-package witness use GitHub runner storage; controller source,
scratch and artifacts remain on external disks.

World inherited files remain untouched: the VCF preflight edit, `.csv`,
`crlf.txt`, `floatreads.txt`, and `ws.txt`. No witness commit, push or dispatch
had happened before that restart. The last completed control-plane run then was
`36231646009` at world `c5f669b87a647d276acb50f42604b82fd9514abc`.

## Actual package execution and offline acceptance

Exporter/checker/workflow commit `5b72d71a29f434e7064302c59124921b46bde74d`
passed exact-head Control plane run `36685364977`. Actual installed-package
run [36685366497](https://github.com/omics-rust/rsomics-world/actions/runs/36685366497),
attempt 1, also completed successfully on Linux x86_64 with R 4.6.1,
infercnv 1.28.0 and Bioconductor 3.23. Its source archive SHA-256 remains
`b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207`.

Original run/jobs/artifact JSON, artifact ZIP and full Actions logs are preserved
under `/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-ingestion-2026-09-30/run-36685366497/`.
The [machine receipt](../oracles/infercnv-ingestion-witness-2026-09-30.json)
pins artifact `11083089026`. Offline verification checked its API size/digest,
original ZIP SHA-256 and CRCs, safe unique member paths, and all source-bundle
manifest entries before extraction. The independent witness checker then
passed on original extracted bytes; a separate read-only reviewer agreed.

| Evidence | SHA-256 |
|---|---|
| Original artifact ZIP | `c6494344ba18fa10af8c6f55cff0d5b92913c94167669057c74af4dd74a9e045` |
| Original Actions logs ZIP | `7013158ca4d4b73ae3694eebc74ecd10bb5e7a2e9482e3ae84c1b7538b192e9b` |
| Shipped sorted manifest | `524bf2544af1098968311dcd3d358a7a8439a2f1cde56fb378cb1d8114f3cd97` |
| Witness JSON | `de3f942d9e19c7e7142652f33f1ad35dbe4d25239babce66b2acda66f8063258` |
| R sessionInfo | `d80333b2ae1e5e8e6bd39376ae5b4596b0f90c437c7b41f971c9043fced9d1e8` |

Accepted dimensions are `448 × 13` for each synthetic profile, `9939 × 184`
for shipped full, and `4 × 4` for the hand-checked small fixture. Each gzip
decompresses to the exact canonical count bytes. Plain/gzip exports agree
exactly for expression, gene coordinates, cell identity/role and ordered
one-based group maps. Both calls' five R object slots were `identical` to the
stage-1 RDS; the checker separately binds checkpoint hashes and source
expression/genes/cells exports. The small fixture confirms chromosome rank,
annotation restriction, cell preservation and group order.

Only upstream `scipen` advisories were observed during creation. They concern
future subclusters/hclust formatting; they are retained, not interpreted as
creation errors. Original script hashes match the executed commit. A follow-up
checker hardening adds independent creation-argument and checkpoint-path
checks, with four regression tests (15 total), and passes this same immutable
artifact. Those provenance fields were also manually checked by the reviewer.

The literal and exponent forms of `0.076439` both produce
`0x1.391819d2391d6p-4` in this R runtime, one ULP above direct Rust binary64
parsing. This is an observed lexical conversion boundary, not permission to
relax numerical-core tolerances. No arbitrary fractional/raw lexical parity
or four-platform R-decimal result is accepted.

## Continuing work

The [product-local raw reader design](../../docs/plans/2026-09-30-infercnv-raw-reader-design.md)
is now the next slice. The reader's source/tests and further creation edge
probes proceed independently. Original shipped gzip headers, min/max filtering,
missing references, malformed inputs, ingestion performance and native raw
ingestion are still unaccepted. This receipt does not complete inferCNV or
authorize publishing an unfinished product.
