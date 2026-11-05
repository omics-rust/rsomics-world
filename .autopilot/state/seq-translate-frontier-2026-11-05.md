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
