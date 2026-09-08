# VCF indexed ingestion and writer repair

Status: observed diagnostic failures are retained; the per-contig chunk-union
candidate is ready for full native source-snapshot validation, not publication.
The exact-index/native-path gate in `vcf-index-selection-2026-09-09.md` remains
the preceding completed slice. The broader concat plan is still incomplete.

## Verified diagnostics

Every run below used four native GitHub-hosted targets: Linux and macOS, each
on x86_64 and aarch64. Snapshot manifests and raw logs identify the actual
dirty source independently of owning product HEAD. No local Rust ran. The
Mac mini boot APFS container was most recently 93.40% occupied; KIOXIA had
59 GiB available. Cargo and scratch remain on external storage locally and
isolated runner-temporary paths remotely.

| Run | Control-plane exact head | Observed result on every native target |
|---|---|---|
| `34282382683` | `eabfe1eb669114601e745e0dffaff3d50a01588f` | Stdout preflight red: 218 bytes emitted before corrupt input 0 failed; concat 30 pass / 1 fail |
| `34283224823` | `8589c1e700b03c92c9f274e611fb8b646321787e` | Initial resource probe: 1 pass / 1 fail; stopped at one input; not scaling evidence |
| `34284272693` | `496a7db14ff2c6b9593757772ffdddbf54db299a` | Corrected resource red: 1 pass / 2 fail; writer red: 10 pass / 1 fail; stdout still fails |
| `34286117195` | `6f5c86b304eea7de9bdbc49703d2897b42ea07cd` | Rejected region-at-a-time merge: concat 33 pass / 2 fail, six collected ordering/tie/dedup mismatches; resources 3 pass and writer 11 pass |

Corrected original input counts 1/2/64/256 yield 2/3/65/257 child threads on
all four targets. Descriptors are 6/7/69/261 on Linux and 8/9/71/263 on macOS.
The resource probe uses backpressured synthetic debug output. RSS is sampled,
not peak RSS; this is not throughput or release performance evidence.

The first resource harness incorrectly stopped at a broken-pipe diagnostic
assertion before measuring higher counts. Its macOS parser also missed
PID-first continuation rows and falsely reported one thread. A corrected
first-record watchdog bounds the source read; resource and I/O assertions are
separate. Only the corrected run supplies scaling evidence.

The initial stdout loop stops at its first failure (plain VCF, input 0);
the other seven combinations were not observed red. Writer fault injection
similarly first exposes BrokenPipe at byte zero, misclassified as InvalidInput.
After repair, all byte boundaries and BrokenPipe/PermissionDenied/WriteZero
cases pass in the unchanged writer test. Innermost I/O kind, leaf diagnostic
and record context are preserved, not the structured source chain.

All 16 artifacts above were independently checked against API SHA/size, ZIP
CRC, every extracted byte, manifests and before/after source checks, native
Rust 1.91 identities, locked dependencies, world HEAD and raw test results.
Permanent copies match external scratch recursively under
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/vcf-index-selection-2026-09-09/`:
`stdout-red-34282382683/`, `resource-probe-34283224823/`,
`io-red-34284272693/`, `order-red-34286117195/`.
Each corresponding control-plane exact-head CI passed. These are diagnostic
gates, not owning-product exact-head CI or publication gates.

## Current candidate

Snapshot `.autopilot/snapshots/vcf-indexed-chunk-union-sparse-2026-09-09` contains 172
files and its README records all hashes. Production changes defer writer
construction until header/index preflight, retain indexed readers, propagate
nested writer I/O errors, and union index chunks per selected contig before
caller-thread k-way merging. No foundation API or new dependency is needed.

The earlier region-at-a-time plan is explicitly superseded: variant-overlap
selection may make an early-POS record visible only in a later query region.
Completing each region separately reordered records and split duplicate
groups. Both VCF/BCF encodings and dedup-off/on failures were collected before
asserting and observed on every native target before the chunk-union repair.

Indexed concat now counts each coarse-span candidate once per input, even if
later rejected by position/variant overlap policy. Off-region chunk records
do not receive full typed decoding or increment the count. The unpublished
three-input fixture changes from 18 query occurrences to 15 candidates;
within-input repeated records remain distinct. Existing `view` counts and
behavior are untouched. Tests additionally cover all overlap policies within
one input and reversed sparse BCF dictionaries. Sparse fixture CSI is built
independently with Noodles after checking raw RIDs and BGZF record offsets.
The initial undispatched chunk-union draft relied on the product index builder,
whose contig-count allocation rejects valid sparse RIDs. That existing defect
is isolated from concat and must receive a separate test-first repair.
Source review is not runtime
verification: the full diagnostic run is still required.

## Open gates and resumption

1. Dispatch and verify the current snapshot's full four-native debug/release
   tests, index/concat/resource/writer regressions, and Linux x86_64 pinned
   bcftools 1.24 oracles, formatting, strict Clippy, harness syntax and package.
2. Add an observed sparse-BCF index-builder red and repair its raw-ID slot
   sizing without weakening unknown-ID rejection. Continue remaining
   query/error and oracle-matrix coverage, two-pass fully
   validated naive concat, many-sample ligation/per-mode performance, and the
   reheader BGZF extra-field detection regression. No performance win is yet
   established for the new indexed traversal.
3. Keep public BGZF extraction behind the completed VCF/BAM consumers and
   performance gate. Do not publish frozen `682942c` unchanged: it retains
   known native-byte index-path and nested writer-error defects.
4. Owning VCF HEAD and Git index remain unchanged, with inherited dirty work
   preserved. Final product commits and exact-head release gates still remain.
   Registry authorization is separately unresolved; no publish retry occurred.
