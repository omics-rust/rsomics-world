# Indexed concat: per-contig chunk union candidate

Full source-snapshot diagnostic candidate; not a product release. Owning HEAD
remains unpublished `682942cfa69768dc3a127a8544f2f07213b704ea` with inherited
concat changes unstaged. No local Rust execution or dependency download ran.
The boot APFS container is over 93% occupied; all local assets are external.

The rejected region-at-a-time snapshot ran as `34286117195` at control-plane
head `6f5c86b304eea7de9bdbc49703d2897b42ea07cd`. All four native jobs reproduced
the six collected VCF/BCF variant-overlap order/tie/dedup mismatches: 33 concat
groups passed and two failed. All four resource suites passed 3/3 with one
child thread at 1/2/64/256 inputs and correct broken-pipe I/O classification.
All four writer suites passed 11/11, including unchanged every-byte injected
fault tests. Index suites passed 11 on Linux and nine on macOS. Artifact
digests, sizes, ZIP bytes, source checks, run identities and exact failure
groups were independently verified and retained under
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/vcf-index-selection-2026-09-09/order-red-34286117195/`.

This candidate replaces only the indexed cursor and merge traversal with
per-contig unioned CSI/TBI virtual chunks. It uses each input's own index/header
dictionary, filters raw off-region records before full typed decoding, and
merges selected records once per output-header contig. No input threads,
envelope scan, dependency, public API or Layer A item is added. Existing `view`
visit semantics are unchanged.

Indexed concat now counts each physical coarse-span candidate once per input,
including candidates later rejected by position/variant overlap policy. It
does not count incidental off-region chunk records. The three-input regression
therefore expects 15 candidates, not 18 query occurrences, and now includes an
outside record. Two additional groups cover a single input with long/short
variants under every overlap policy and inputs with reversed sparse contig
dictionaries. Both previously observed ordering/dedup red groups are unchanged.
Writer and resource regression sources are also unchanged. Deferred writer
creation and the nested I/O classification repair are retained.

Pre-dispatch review found that the existing product index builder allocates
BCF slots by contig count instead of raw dictionary IDs. The sparse BCF fixture
therefore constructs CSI independently with Noodles from verified raw RIDs
2/5 and exact before/after BGZF offsets for its two POS 10 records. It verifies
header IDs and EOF. This tests concat against sparse external indexes without
repairing or bypassing the product bug in production. That index-builder bug
needs separate test-first verification and repair before a product release.
The earlier `vcf-indexed-chunk-union-2026-09-09` draft was never dispatched or
staged because it depended on the defective fixture creation path.

The merge heap has one pending record per input; total memory additionally
includes indexes, headers and the complete same-coordinate matching batch.
Resource sampling is a synthetic backpressure diagnostic, not a peak-memory
or performance result. Remaining concat compatibility/per-mode performance,
naive two-pass repair, reheader extra-field regression and exact-product-head
release gates remain open.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (328021 bytes) | `f2628a56ca2e9c7b9d8fef8e7d0fea6faab2f49d6e792df91ce94342a21ef322` |
| `files.sha256` | `fdaa8097ff5673fabdd72bf8e93059af942f55caef1d7bcf738ab3a4e2a4dbe6` |
| `tracked.patch` | `ebae304555edc500939fe26095a32ff93b3a119bfdabe8847cca0b341c0f403f` |
| `src/regions.rs` | `1fb21bdd2638da9ab20c4dd2e93e6ccbf02bf27d404ec8f4d308df693acefa0a` |
| `src/concat/stream.rs` | `5e28b7b08c0448470eb5ca362b7f1c0fd7c0fdd781c11c060eed1f74123d4135` |
| `tests/concat_cli.rs` | `9eb5de2ec6429f70466e05445ad2373296049c83220c9620ade893400c8cb6e6` |

The 172-file inventory is unchanged. Exactly those three source hashes differ
from the preceding ordering-red snapshot. Local rustfmt and diff checks pass;
independent source review approves native diagnostic validation, not runtime
correctness. Dispatch with `regressions_only=false` to run four native debug
and release suites plus the pinned Linux x86_64 oracle/static/package gates.
