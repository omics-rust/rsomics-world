# inferCNV ingestion witness — resumed, acquisition pending

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
had happened before the restart. The last completed control-plane run was
`36231646009` at world `c5f669b87a647d276acb50f42604b82fd9514abc`.
