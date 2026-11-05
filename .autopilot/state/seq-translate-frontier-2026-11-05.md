# Sequence translation source preflight

Owner `rsomics-seq` is clean at `b3d045affba732f2075cac4fc164bf08e204d220`
with seven stable operations and no `translate`. Retired team-owned donor
`rsomics-fasta-translate` at `095837669bcf0520a68df496cfc589cdffcee051` retains
codon/frame logic and six useful fixture pairs. Its table parameter is ignored,
its compatibility projection drops headers and wrapping, and it writes named
outputs directly. Reuse proven pieces and fixtures, not that CLI or its
limited compatibility verdict. The owning product already has sequence I/O,
shared help and named-output transactions.

## Actual source behavior

Packet: `/Volumes/KIOXIA/Developments/tmp/seq-translate-source-20261105.fj3KEN`.
Official SeqKit v2.13.0 Mac ARM asset matches GitHub's 8,188,794-byte SHA-256
`c36d68cfe4d8796c017a136a625b4706b4f7dc2664f1a555e4b4ced4ee394a28`.
Extracted binary SHA-256
`dac78516e7c4c362cfbcfcc265af82f26a42ca4425289e2c602e0b8e6ef0b4b5`.
All 14 one-worker calls retain complete argv/status/stdout/stderr identities;
`witness/RESULTS.json` SHA-256
`6d3b53afcad7247a28c9b456fddc766e2835879ce2bd806ba06deeb1171bd883`.
The six complete donor goldens replay byte-for-byte, including RNA, degenerate
bases, gaps and negative frame. Invalid table999 fails with exit255 and empty
stdout; this is not a requirement to reproduce the numeric exit code.

Default FASTA and FASTQ produce protein FASTA with the complete description.
Table1 yields `MR***V` for the first control, table2 `M*W**V`. Table11 with
initiator conversion yields `MK*MN` for the second control, converting a
valid initiator after an internal stop too. The source lists 24 supported
tables. Signed six-frame order is 1,2,3,-1,-2,-3 per record; append-frame
inserts the frame in the ID and the source emits a separating space even
without a description. The witness retains exact width4 wrapping.

`--trim` needs a compatibility decision. Help promises suffix trimming,
but both forward and reverse loops in
[bio v0.13.9](https://github.com/shenwei356/bio/blob/v0.13.9/seq/codon_tables.go)
break on the first X or stop. Actual `--trim` truncates `MR***V` to `MR`,
and adding `--clean` does not change it; `--clean` alone gives `MRXXXV`.
Both single-flag witnesses are retained in `trim-order/`. The user was
asked whether to implement suffix trimming as documented or emulate the
source's internal truncation. Hold this flag's behavior, not the other
translation work. No production code or public command was added here.

## Representative workload candidate

`/Volumes/KIOXIA/Developments/rsomics-fixtures/bench_500k.fa`: 257,888,890
bytes, SHA-256 freshly verified as
`cd0e4b140a40c2b36f298b084c69d68151f98fa2379c0e81c1607f4d2fa57ff9`.
Its index lists 500,000 records and 250 million bases. It can support
matched full-output one-worker timing under external headroom. No new
performance result or table-1-only boundary is inferred from this preflight.

## Local native candidate

Owner `rsomics-seq` is still at `b3d045affba732f2075cac4fc164bf08e204d220`.
Keep the uncommitted translation source, shared header helper, CLI and
installed-lifecycle changes. No dependency, public foundation, release
version or published artifact was changed.

Use `--min-protein-length` for ordinary protein filtering. The pinned
[SeqKit command](https://github.com/shenwei356/seqkit/blob/v2.13.0/seqkit/cmd/translate.go)
only uses its `--min-len` inside stop-delimited extraction. Named protein
BGZF is checked with a full output record and a late codon error that must
leave the previous file untouched.

The fresh whole-feature review found no critical issue and two source
differences. Literal `*` as an unknown nucleotide now yields X without
opt-in, including `AC*`, `--*` and `***`; only `---` yields a full gap.
The new differential failed on `AC*` before that correction and passes
afterward. With the actual 2.13.0 executable, debug and release each pass
59 tests including all 3,072 concrete-codon comparisons; strict Clippy,
rustdoc, formatting and release-binary lifecycle also pass. This lifecycle
replay is not an install from a clean source package.

The other finding is an unresolved compatibility choice. The same reverse
complement issue that affects uracil also affects asymmetric IUPAC DNA
bases: both executables accept `YAR` in frame −1, but the native
complement `YTR` produces L while the source produces X. Pinned
`-M` also misses RNA initiators. A combined user decision is pending;
do not treat the native draft as qualified for these cases. The independent
`--trim` behavior remains held.

Evidence: `/Volumes/KIOXIA/Developments/tmp/seq-translate-impl-20261105.H6w0AF`
and `/Volumes/KIOXIA/Developments/tmp/seq-translate-lifecycle-20261105.CxqhVs`.
Local tests, compatibility, performance, exact-head native CI, and
publication remain distinct gates.

## Representative candidate measurements

Mac mini `Mac14,3`, macOS arm64, Rust 1.91.0 release build, native serial
and SeqKit 2.13.0 with `-j 1`. Both used the pinned 500,000-record FASTA.
Full outputs match byte-for-byte in each lane: 91,888,890 bytes for
default frame 1 and 579,833,340 bytes for `-f 6 -F -T 2`. Root replayed
both comparisons after the run. The paired timing uses one warmup per tool
and five alternating measured pairs to `/dev/null`; stdout generation
is included, durable output I/O is not.

| Workload | Native / source wall median | Native / source peak RSS median |
| --- | --- | --- |
| Frame 1 | 0.267 / 2.256 s | 4,440,064 / 66,813,952 bytes |
| Six frames, code 2 | 1.151 / 15.104 s | 4,440,064 / 66,486,272 bytes |

Both axes show a material advantage on these two workloads: 8.46× and
13.13× elapsed speedups, about 93% lower peak process RSS. No parallel,
compressed-output or universal performance claim. Native executable SHA-256
`cd87c670dbcfd8b64ce1a15979d56137787cd6e273e5b44d8180abf82afbcaed`.
The raw commands, resource stderr, samples, output hashes, and exact Rust
source-file identities are in `benchmark.r_9ze1fc/RESULTS.json` under the
implementation packet, SHA-256
`898317ba6d2da9c82ab0a7c389e0cbfd010e57419c7fed2ab57198a85b4a7ce8`.
Do not transfer these measurements to an edited candidate or a release
binary. Resolve the held behaviors before owner integration and clean
exact-head four-native CI; no release was dispatched.
