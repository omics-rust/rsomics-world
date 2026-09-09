# Indexed BCF logical EOF audit

Status: test-only snapshot `vcf-indexed-bcf-framing-red-2026-09-09` is running
as `34297832960`, world `d449c86b369e2ae3928060f17c7d30c34fdfcd96`; exact-head
control CI `34297754328` passed before dispatch. Linux ARM raw assertions were
read before indexed production edits. All 16 malformed commands incorrectly
succeed and replace synthetic prior destinations; all eight positive controls
succeed. Selection has 11 pass/one fail; concat 38, reheader 21, index 13,
resources three and writer 11 pass. First raw log is external
`/Volumes/KIOXIA/Developments/tmp/vcf-indexed-bcf-first-red-qDU0aX/linux-aarch64.log`.
All-target artifact verification remains separate. The reviewed one-file
indexed repair is now being implemented; no repaired execution is claimed.

This follows the fully verified nonindexed/reheader repair in `34296650518`.
That run excludes this new test. Current pinned Noodles BCF
0.88 uses zero for both actual record EOF and encoded `l_shared=0`.
`regions::QueryReader::Bcf` directly trusts that return; indexed view delegates
to Noodles util 0.82 / BCF query, whose record loop also trusts it. These are
separate paths from the already changed `format::Reader` and reheader loop.
The custom index builder reads its own record framing and is not presumed
affected by the same bug.

The appended CLI group runs 16 malformed and eight positive commands. It uses
an independently constructed, serialized and reread CSI; a raw CSI query must
return every malformed byte. Actual header and exhausted-frame virtual offsets
are asserted. Cases cover a four-byte zero word before or after a valid record,
indexed view/concat, and v/z/b/u named outputs. Existing synthetic output must
survive failure; no streaming stdout rollback is asserted. Positive records
are independently decoded and compared to a complete literal VCF body.
Coverage is one contig/chunk/frame. Cross-frame words, multiple chunks,
excluded corruption, ligation and precise error classification are outside
this slice. See the immutable snapshot README for hashes and limitations.

After observing actual expected-red failures, reuse the private checked BCF
reader in indexed concat and replace only the unchecked BCF query route for
view. Preserve candidate filtering, record counters, region overlap/dedup,
sparse RID handling, chosen-index precedence and I/O error propagation. In
particular, Noodles' unbounded BCF query short-circuits its interval test;
do not silently change this while fixing record framing. No new foundation,
public item, dependency or product boundary is required.

Keep tests unchanged for the repair, review source independently, then run all
four native debug/release suites, focused tests, pinned Linux oracles, strict
format/Clippy and packaging. Benchmark gates remain separate and open.

Product HEAD is still `682942cfa69768dc3a127a8544f2f07213b704ea`; index empty.
No local Rust/product execution is permitted while boot occupancy exceeds
80%. All capture and evidence work uses the specified external disks.
