# Sparse BCF index construction, names and interoperability

Status: sparse builder/statistics repair full four-native diagnostics verified
green. Subsequent empty-tail query/statistics expected reds verified on all
four native platforms. Empty-tail repair run `34291571671` is fully verified
green, including actual HTSlib-generated CSI queries. Subsequent oracle
expansion `34292257895` exposes a wrong test expectation: the completely
empty bcftools index has three slots, not zero. The corrected isolated test
snapshot is `vcf-sparse-oracle-empty-layout-2026-09-09`; only that expectation
changes from the previous 174-file snapshot. Complete empty-file bidirectional
validation is still pending. No publication gate is closed. Boot occupancy
exceeds 95%; local compilation remains prohibited.

## Latest verified results

Empty-tail fix `34291571671`, world
`ac6ccc9e8cecec466844b627f3b2f5cd41d46037`, passes four native jobs and
control CI `34291509323`. Every ordinary profile passes 431 Linux or 429 Mac
tests, with 61 ignored, and Linux x86 separately passes all 61 oracle groups
per profile plus fmt/Clippy/harness syntax/package. Focused index construction
passes 13, including all 32 intended query cases and rejection guards;
concat/resources/writer remain 37/3/11. Four ZIP API identities/CRCs, 97
extracted files, 172-source before/after lists, heads, native Rust and lock
identities are independently verified. The live HTSlib CSI has six slots;
both tools now return success/empty for RID 9, and rsomics `--all` correctly
includes `empty\t300\t.`. The upstream `--all` crash remains explicit.
Permanent evidence matches KIOXIA `vcf-empty-tail-fix-34291571671-sKk0O5` at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/vcf-index-selection-2026-09-09/empty-tail-fix-34291571671/`.

The subsequent test-only run `34292257895`, world
`d4383d4e8e672cbb881d1eee383039cac93bb1eb`, fails only the Linux oracle step;
its new index oracle fails at the empty-file span assertion in both profiles
after completing the nonempty case. Actual span is three, expected zero.
Raw log evidence is at KIOXIA `vcf-sparse-oracle-first-failure-cZSubA`.
HTSlib initializes from the declared contig count before raw-ID expansion.
Only this test expectation is corrected; the independent zero-slot fixture
and all production code remain unchanged. The full run artifacts are being
retained, and the later empty-file checks must still execute. Control CI
`34292169465` passed. A prepared reheader expected-red test draft is excluded
from the isolated corrected-oracle snapshot; see its README for staging.

Builder/statistics fix run `34290416281` at world
`47de686b0d4606992010cf07887745129804ee3d` passed all native jobs; control CI
`34289977881` passed. Four ZIP API digests/sizes, CRCs, 97 extracted files,
172 source checks before/after, Rust/native host and dependency identities
were independently verified. Every ordinary profile passes 428 Linux or
426 macOS tests, with 61 ignored oracle tests. Linux x86 separately passes
all 61 pinned oracle tests per profile, plus fmt, strict Clippy, per-harness
syntax and package. Focused index construction/concat/resources/writer pass
10/37/3/11. The probe confirms default BCF statistics fixed and owned CSI
creation succeeds with 10 slots; the external six-slot CSI still exposes
the empty-tail query and `--all` omissions. These are separate gates.

Permanent evidence, recursively verified against KIOXIA capture
`vcf-sparse-fix-34290416281-cTcG84`, is
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/vcf-index-selection-2026-09-09/sparse-fix-34290416281/`.

Focused empty-tail red run `34290914478` at world
`d7fd393970ea1605ae9cb512e010c6528e21f36a` fails exactly the index step on
all four native targets (11 pass/two fail); control CI `34290856966` passes.
Each platform records all 32 intended query failures and both statistics
omissions. The 16 unknown-region/corrupt-index output-preservation cases pass.
All four ZIPs, 56 extracted files and 172-source before/after identities are
verified. This focused run intentionally does not execute ordinary full
debug/release or oracle suites. Evidence is at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/vcf-index-selection-2026-09-09/empty-tail-red-34290914478/`,
identical to KIOXIA capture `vcf-empty-tail-red-34290914478-5zt4La`.

The empty-tail fix changes only the private region reader and statistics
implementation. The expected-red tests are byte-identical. It caches the BCF
index span once per input, skips valid out-of-span IDs without suppressing
header/index errors, and appends declared tail contigs in raw-ID order for
`--all`. Default totals, VCF behavior and existing read-counter semantics
remain unchanged. Source review approves remote execution, not correctness.

## Evidence

Expected-red snapshot `vcf-sparse-index-red-2026-09-09` ran as `34288833017`
at world `75c2cf858cde5dba0c17692b67db8b61ef8fb95e`. Exact-head control-plane
CI `34288777659` passed. All four native jobs reach the same two intended
failures, each with seven passing index groups, in focused/debug/release:

- Nonempty sparse BCF rejects valid RID 5 under zero/two decompression workers.
- Empty sparse BCF has three index slots rather than ten under both modes.
- Statistics use header ordinal positions, mislabeling the two RID-2 records
  as contig `empty` and the RID-5 record as `n/a`; both default/all outputs fail.
- The independent CSI fixture proves statistics fail separately from building.
- Interior-hole, outside and i32::MAX raw-record IDs still fail without
  replacing an existing destination, under both worker modes.

All four API ZIP digests/sizes, CRCs, extracted bytes, 172-file before/after
checks and exact failing groups were verified. Permanent evidence matches
KIOXIA capture `vcf-sparse-red-34288833017-Q8VMMB` recursively at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/vcf-index-selection-2026-09-09/sparse-red-34288833017/`.
The earlier first-observed Linux ARM raw log is
`/Volumes/KIOXIA/Developments/tmp/vcf-sparse-red-linux-arm-opjfmJ`.

The full ordinary test commands stop at the index-suite failure. Do not sum
their earlier passes into a complete product count. The separately focused
concat/resource/writer suites pass 37/3/11. Source identities are frozen, not
the later live repair. Boot occupancy remains over 93%; no local Rust ran.

## Actual pinned-oracle behavior

The Linux x86_64 diagnostic creates and checks raw BCF RIDs 2/2/5 with header
IDs 5/2/9, then records both implementations. bcftools/HTSlib 1.24 reads and
indexes the input, returns correct default stats and total three, but exits
by signal 11 on `index --stats --all`, with no output. Core dumps were disabled
and all commands returned before the ten-second deadlines. This narrowly
observed difference is not generalized to other upstream versions/platforms.

Both tools agree on the two populated-region queries. For declared empty
RID 9, bcftools returns success and empty output, while rsomics exits 4 with
`invalid reference sequence ID: 9`. The HTSlib-generated CSI ends after the
last populated RID rather than the final declared contig. Current `view` and
the private concat chunk resolver need follow-up coverage for that valid form.
Likewise check `--all` output for declared contigs beyond the CSI slot span.
The probe is diagnostic evidence, not a completed compatibility matrix.

## Current bounded repair

Snapshot `vcf-sparse-index-fix-2026-09-09` changes only builder, statistics,
and index tests. Its README records all identities. Actual dictionary IDs
determine CSI/linear slot span; build-summary contig count remains three.
Raw-record unknown-ID validation is unchanged. BCF statistics resolve raw
IDs before auxiliary index names, omit empty unnamed holes, and reject
nonempty unnamed slots before emitting output. VCF/index-only naming stays
unchanged. The original sparse-red file content is retained byte-for-byte;
an added corrupt-CSI/auxiliary-name guard has not been observed red separately.
Independent source review approves remote validation, not runtime correctness.

No public item, dependency or foundation extraction was added. Huge-header
allocation safety is not solved by this change: upstream Noodles header and
index storage already grow densely before/alongside this code.

## Next gates

1. Run the corrected full native oracle candidate, finish empty-file
   bidirectional checks, and retain/verify the erroneous-expectation run.
2. Keep the actual upstream `--all` crash distinct from product contracts and
   preserve zero-slot native regressions alongside HTSlib's three-slot case.
3. Continue reheader extra-field expected-red verification and concat's
   remaining naive/oracle/performance gates in its design and audit documents.
4. Before eventual owning-product exact-head CI, correct its inherited
   `bash -n benchmarks/*.sh` command: Bash treats later paths as arguments and
   checks only the first script. The diagnostic workflow already loops over
   every harness. Neither invocation is a benchmark.
5. Keep publication closed until all declared operations, performance,
   metadata, exact-head native CI and registry authorization gates are met.
