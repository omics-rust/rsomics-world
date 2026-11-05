# BED Jaccard source contract and local candidate

The owning repository remains at `b4a32820adf2236bf86843973112e3cfb37dce9b`.
There is a coherent uncommitted core in `src/jaccard.rs`,
`src/cli/jaccard.rs`, and `tests/jaccard.rs`, with command dispatch and a
product-private extraction of map's existing pair-order validation. No
release, version change, new dependency, or foundation API was added.
Plan: [BED Jaccard](../../docs/plans/2026-11-05-bed-jaccard.md).
The exact seven changed source/test bodies are also retained as
`candidate-source.tar` in the packet below, SHA-256
`de060fd02dc57c1a2eb5057bf450674acdf1ea0858f4ee50041a2f7c308dfa84`.

## Source and reusable assets

- BEDTools 2.31.1 commit `705ccfdf2c9a77d71560c8adcece0663c2f5e18e`.
  Inspected the actual Jaccard, context, merge, sweep, block and record
  paths, plus bundled tests. Source archive SHA-256
  `1db4e57c1899aede13ef516c61c9b5819cf2790994a556dbb28972488f9946e6`.
  Native code uses the independently implemented two-pointer geometry,
  not upstream C++.
- Clean historical `rsomics-bed-jaccard`
  `ca81d15e39d042ed3568c03e702894cc94814f1a`: reuse the merged span
  sweep and compatible goldens, not full-file sorting/retention or f64
  ratio calculation. Its prior speed result is not this candidate's result.
- Actual source binary SHA-256
  `fc994e1fcfdce8ce8591630d71f21fe67e2275c2a8d967dc0e3dfb4cfa4677a3`,
  copied unchanged onto the external volume. Twenty-eight full command
  records, input identities, and stdout/stderr are retained under
  `/Volumes/KIOXIA/Developments/tmp/bed-jaccard-source-20261105.neNjvD`.
  Source `RESULTS.json` SHA-256
  `72846f71880eb05845156d37bd13afd319fc9a5e25bfb71a05a9082c0050633f`.

Touching or overlapping records within an input merge before intersection.
Default missing fraction thresholds are zero; the source only counts
positive overlaps in the ordinary default path. Directional `-s` uses
separate matching plus/minus lanes; `-S +|- ` selects a fixed strand
in both files, not opposite strands. Unknown strands are excluded in
directional mode. Two empty selected sets have an undefined `nan` ratio.
Raw coordinate sorting, actual shared contig order, optional genome order,
and an invalid B tail have executable evidence. A source failure can print
a partial summary for a missing nonlexical contig; the native candidate
validates both streams before writing and retains existing named output.

## Compatibility choices not yet adopted

1. **Tied points:** against B `[0,10)`, A `[10,10)` before
   `[10,20)` gives `1 20 0.05 1`; the reverse A order gives
   `0 20 0 0`. The source merge retains the first widened start and
   only extends the end. The local candidate explicitly rejects the
   backward-effective-start case, pending the user ruling. A repair must
   also merge with a preceding span newly touched by the widened start.
2. **Split BED12:** two A records with outer spans `[0,20)` and
   `[10,30)` merge before split processing. The merge constructs
   a replacement through base-class record assignment, losing BED12
   block metadata. B `[25,30)` reports `0 35 0 0` despite
   a real 5-bp block hit. The true A block union is 20 bp for this input.
   Standalone BED12 source behavior (3 bp intersection, 17 bp union) is
   separately witnessed. Split fractions are aggregate per query across
   candidate B blocks, not the ordinary per-merged-pair rule. No
   `--split` is advertised or qualified in the local draft.
3. **Book-end recycling:** under `-e -f 0.8` with implicit B fraction
   zero, A `[10,20)` versus B `[0,10)` prints
   `18446744073709551605 31 5.95056e+17 1`. Swapping A/B prints
   `0 20 0 0`. The source sweep stores the zero-width hit then
   recycles the B record before Jaccard reads it. This is not a valid
   similarity result or a portable compatibility target. The local
   candidate only admits positive width, yielding zero both ways; the
   public ruling (zero versus explicit error) is pending. Both orders
   and both executables are retained in the measurement packet.

Two async questions cover these behaviors. The earlier map repair is
specific to map. None of the unresolved options or behaviors was pushed
or released.

Additional split evidence: a 20-bp BED12 query and three candidate B
spans covering 25 bp yield 10 bp across two real block hits. The aggregate
`-f 0.45` retains both, while `-F 0.45` rejects both because the
intronic B candidate contributes to the aggregate B denominator. The
source also ignores `-e` in block filtering; the test with
`-e -f 0.9 -F 0.1` still rejects both. Do not infer a split OR
contract from the ordinary intersection code. This is captured in
`split-aggregate-evidence.log` SHA-256
`28ff74081db77e2a43517a5da84fb4cd1e150889a4fe6170af9bc3c1c37d3ca2`.

## Local verification and performance

The two consumer tests first failed on the absent `jaccard` subcommand,
then passed with the actual pinned BEDTools runtime. The existing map
suite also passed after the shared order extraction. Full debug and
release suites each passed 130 tests (none failed or ignored). Formatting,
strict Clippy and rustdoc passed under the physically verified external
Cargo/scratch guard. This is local evidence, not clean source-package or
four-native CI evidence.

One independent read-only review found no new Critical or Important
issue in the admitted merged-span core. It checked ordered lane geometry,
fraction arithmetic, both raw tails, output safety, map's unchanged
state transitions and the source hashes of the measured executable.
It did not approve any held behavior, run tests, or qualify a release.

The measured candidate on macOS arm64 used all 61,177 sorted RefSeq
transcript spans and 584,846 exon rows from the retained dataset. Sorting
is excluded from both commands. Every complete TSV matched in one warmup
and five alternating paired measurements per lane; all 24 retained outputs
were rechecked. No extrapolation to split, physical I/O, or parallel scaling.

| Workflow | Native / BEDTools wall median | Native / BEDTools peak RSS |
| --- | --- | --- |
| Default merged spans | 0.150 / 0.263 s | 4,308,992 / 141,754,368 bytes |
| Same strand, A/B fractions 0.25/0.1 | 0.198 / 0.314 s | 4,866,048 / 140,001,280 bytes |

Exact candidate binary SHA-256
`8c37fd3955438d290a30bace37e94edcf726546e5a338e34871f189c198a6ae1`.
Raw samples, complete outputs, source-file identities, and both book-end
orders are in `measurement.21yr2zwv/RESULTS.json` below the source
packet, SHA-256
`18ebbae4cdf2927782334590d480195fd2bda290a6b92e1055ceea6013b4ae65`.
Requalify changes on the final intended source before any product push.
