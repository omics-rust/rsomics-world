# Raw creation metrics and closed-campaign checker

Status: owning test-first RED/GREEN and fresh independent source review pass.
Main delivery/exact-head controller CI pending. This is not an actual campaign.

Plan: `../../docs/plans/2026-09-30-infercnv-ingestion-campaign-checker-plan.md`.
Spec: `../../docs/plans/2026-09-30-infercnv-raw-ingestion-measurement-design.md`.

Root read complete proposal/source/tests. Owning test-only run first fails
specifically ModuleNotFoundError naming infercnv_ingestion_campaign, then the
minimal reviewed helper passes all32 focused tests. Full controller274 passes
with external TMPDIR and bytecode disabled; architecture/whitespace checks pass.
Original root RED/GREEN/full logs are retained under external scratch
`rsomics-ingestion-controller-proposal-20260930/root-*.log`.

Fresh independent owning review finds no critical/important defect, independently
passes32/274 tests and architecture/whitespace checks, verifies complete plan
code matches owning files. Test AST changes from reviewed proposal only remove
an unnecessary proposal-history docstring and rename the private import;
all assertion bodies are unchanged. No workflow/selector/launcher change.

| File | SHA256 |
| --- | --- |
| scripts/infercnv_ingestion_campaign.py | 212bf00ad68c59e652f220eacdf76350ca4e56d48752c3b1592fd46dba5c4dea |
| scripts/test_infercnv_ingestion_campaign.py | c0e387d5b9a11ce3ffdc5ae7a3780314e5266ca3c21d2c0fbba5409d5fbee6f0 |
| unchanged exact creation checker | 0fb1c968e46ff4dd041a42907ab70efaee9fae8beadf1254c81b1edf3c19cb36 |
| unchanged project import | 682d2a41bbe901a02b6687756e64cc1359fa1e3881d49af40eaa5fc725211222 |

The helper checks literal12-key metrics, finite/nonnegative counters, strict
child CPU zero, nonzero underflow rejection and positive u64 RSS. Schedule
requires all32 typed ordered slots, successful exact-int statuses, distinct
canonical nonsymlink trial directories and64 actual warm/measured validations
through the unchanged exact checker, including pilots. Mutations cover missing/
duplicate/reordered/failed samples, bool/int aliases, mixed metrics, numeric
underflow/overflow, path aliases, one-ULP/signed-zero and ordered-map differences.

Only after all exact states pass, four declared pilot processes are excluded
from descriptive distributions. Both plain/gzip independently require
max7Rust<min7R; overall AND, no epsilon or resource fallback. An intact campaign
failing wall retains all distributions and false gate rather than hiding data.

This validates declarations and exports, not launch truth or authenticity.
Actual sequential fresh processes/non-overlap, original artifact API/digest/
CRC/inputs/decoded gzip, prelaunch and post source/import/executable/runtime
pins, whole-process GNUtime/I/O, complete root inventory and representative
performance remain the separate real driver gate. R package tests may use a
weaker as.numeric screen; the real controller must apply this literal parser
to actual R metric bytes, not normalize stale ten-key proposal outputs.
