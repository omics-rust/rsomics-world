# Sparse BCF index construction, names and interoperability

Status: four-native expected reds verified; builder/statistics repair candidate
ready for full native diagnostics. External-index empty-tail behavior is a
separate observed compatibility gap, not fixed or publishable yet.

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

1. Run the full four-native repair diagnostics and verify retained artifacts.
2. Add and run empty-tail query/statistics regressions using externally shaped
   CSI, including `view` and concat, before changing that behavior. Preserve
   unknown-contig rejection and index corruption errors.
3. Add pinned sparse-ID interoperability tests with explicit expected
   differences for the observed upstream `--all` crash, not silent comparison
   exclusions. Continue concat's remaining naive/reheader/oracle/performance
   gates in its design and audit documents.
4. Before eventual owning-product exact-head CI, correct its inherited
   `bash -n benchmarks/*.sh` command: Bash treats later paths as arguments and
   checks only the first script. The diagnostic workflow already loops over
   every harness. Neither invocation is a benchmark.
5. Keep publication closed until all declared operations, performance,
   metadata, exact-head native CI and registry authorization gates are met.
