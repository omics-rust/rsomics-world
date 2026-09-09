# Indexed BCF logical EOF audit

Status: test-only snapshot `vcf-indexed-bcf-framing-red-2026-09-09` failed
as `34297832960`, world `d449c86b369e2ae3928060f17c7d30c34fdfcd96`; exact-head
control CI `34297754328` passed before dispatch. Linux ARM raw assertions were
read before indexed production edits. All 16 malformed commands incorrectly
succeed and replace synthetic prior destinations; all eight positive controls
succeed. Selection has 11 pass/one fail; concat 38, reheader 21, index 13,
resources three and writer 11 pass. First raw log is external
`/Volumes/KIOXIA/Developments/tmp/vcf-indexed-bcf-first-red-qDU0aX/linux-aarch64.log`.
All four targets fail only index selection with the same 16 malformed
successes/destination replacements and eight controls; macOS has nine pass/
one fail. The reviewed one-file repair `vcf-indexed-bcf-framing-fix-2026-09-09`
was submitted for full verification as `34298954629`, world
`9868be30ec97bde5343ea1ddf6be04fc92dd3a0f`; control CI `34298801698` passed.
Focused indexed regressions pass, but both supplemental unit groups fail in
debug/release. Fixture-only correction
`vcf-indexed-bcf-fixture-fix-2026-09-09` is now running as `34299936099`, world
`5e300ee1e0309beaf1cf783fc5415386f6183cb5`; control CI `34299771171` passed.
This is not a completed indexed correctness gate.

The first actual Linux ARM failures were read at
`/Volumes/KIOXIA/Developments/tmp/vcf-indexed-bcf-unit-failure-MNjEp4/linux-aarch64.log`.
Both ordinary invocations stop in the library with 280 pass/two fail; do not
report the intended full 448/446 totals. The fixture's Noodles BCF writer
assigns raw RID 2 from explicit IDX 2/5, but serializes its header through a
VCF writer which omits those IDX fields. Re-read header IDs become 0/1, so
the production predicate correctly rejects a missing name. Source inspection
independently confirms both paths. The correction rebuilds the raw BCF header
from literal sparse text, then independently checks both mapping directions,
raw RID and typed decoding. Production and frozen CLI bytes are unchanged.
The failed full run is independently verified on all four targets: each
debug/release invocation reaches only the library group, with 280 pass/two
fail/zero ignored. All focused groups pass, including 16 indexed malformed
cases rejected with preserved destinations and eight controls. Linux x86's
separate 62 oracles per profile, format, strict Clippy, harness syntax and
package checks pass. Sparse-query results and the upstream `--all` SIGSEGV
divergence are unchanged. All four ZIP API hashes/sizes/CRCs, 101 extracted
files, 174-source before/after identities and native Rust/lock metadata are
verified. Evidence is
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/vcf-index-selection-2026-09-09/indexed-bcf-framing-fix-34298954629/`:
112 retained files (4,076,056 bytes), recursively identical to scratch,
inventory SHA-256 `6dc63141760b6a35a1f5ccddb9059295d981188b82df74ec816d5c8d1036e54b`.

The corrected snapshot excludes the separately appended naive CLI draft.
Its README also records a capture-format issue: local `diff.external=difft`
renders `git diff` rather than producing an applicable patch. Previous source
archives/hashes remain verified and immutable. The corrected snapshot uses
explicit `--no-ext-diff --binary` and an external temporary Git index; applying
that patch to a fresh HEAD archive reproduces every tracked source. Use this
explicit capture/replay check for future snapshots; the product index remains
empty throughout.

Both independent audits verify four ZIP API hashes/sizes/CRCs, all 60 extracted
files, exact 174-source before/after lists, native Rust 1.91, shared lock and
dependency identities, raw unique case evidence and intended skipped steps.
Permanent evidence is
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/vcf-index-selection-2026-09-09/indexed-bcf-framing-red-34297832960/`.
All 71 retained files (3,367,370 bytes) match external scratch recursively;
inventory SHA-256 is `02d8bf6f976b5a16c54e31bb9092dff49591245ff0b090899c61f01344c236bc`.

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

After observing actual expected-red failures, the repair reuses the private
checked BCF reader in indexed concat and replaces the unchecked BCF query
route for view. It preserves candidate filtering, record counters, region overlap/dedup,
sparse RID handling, chosen-index precedence and I/O error propagation. In
particular, Noodles' unbounded BCF query short-circuits its interval test;
the private raw filter retains this order. The VCF branch uses the same inner
query/records mapping as pinned noodles-util 0.82 without another iterator
box. Concat gains no additional buffer. Two supplemental unit groups cover
sparse RID 2, record-span endpoints, unbounded POS 0 and missing RID names.
Independent review approves the exact repaired source
`7188ed9ac337c5e4bd3d1b67517e09643e3d30062ffe726af38cf3291eeca2f3`; source
inspection is not runtime or performance evidence. No new foundation,
public item, dependency or product boundary is introduced.

Keep tests unchanged for the repair, review source independently, then run all
four native debug/release suites, focused tests, pinned Linux oracles, strict
format/Clippy and packaging. Benchmark gates remain separate and open.

Product HEAD is still `682942cfa69768dc3a127a8544f2f07213b704ea`; index empty.
No local Rust/product execution is permitted while boot occupancy exceeds
80%. All capture and evidence work uses the specified external disks.
