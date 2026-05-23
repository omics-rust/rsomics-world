# rsomics-bam-mpileup

Per-position text pileup of a coordinate-sorted BAM — a Rust port of
`samtools mpileup` (the modern non-VCF text pileup). For each covered reference
position it prints the chromosome, 1-based position, reference base, depth, the
read bases over the covering reads, and their base qualities.

```
rsomics-bam-mpileup sorted.bam                 # no-reference, ref column = N
rsomics-bam-mpileup -f ref.fa -B sorted.bam    # reference-aware (.,/= encoding)
```

The base encoding matches `samtools` exactly: `.`/`,` for a base equal to the
reference (forward / reverse strand), `ACGTN`/`acgtn` for a mismatch, `^<q>` /
`$` read start / end markers, `+N<seq>` / `-N<refseq>` indel notation, `*` for a
deleted position, `<`/`>` for a reference skip. Defaults follow samtools:
`min-BQ 13`, `min-MQ 0`, skip `UNMAP|SECONDARY|QCFAIL|DUP`, orphan filtering and
overlapping-mate quality removal both ON.

## Origin

This crate is a Rust port of `samtools mpileup`. samtools and htslib are MIT
(Expat) licensed, so their source was read and the behaviour reproduced
directly:

- Column construction, the per-read `qpos` / `is_del` / `indel` / `is_head` /
  `is_tail` state, and overlapping-mate quality removal come from htslib's
  pileup engine (`bam_plp`), ported in the [`rsomics-pileup`](../../../foundation/rsomics-pileup)
  Layer A crate.
- The text encoding (`pileup_seq` and the column output loop), the
  `.,ACGTN`/`=` alphabet, indel / `*` / `^` / `$` notation, the per-base
  `min_baseQ` filter and the `-a`/`-aa` empty-position handling come from
  `samtools` `bam_plcmd.c`.

Base Alignment Quality (BAQ, htslib `sam_prob_realn`) — the one default that
mutates qualities when `-f` is given — is **not** implemented. Pass `-B` (no
BAQ) for byte-exact reference-aware output (matching `samtools mpileup -f … -B`);
reference-aware mpileup without `-B` is refused rather than silently emitting
non-samtools output.

License: MIT OR Apache-2.0.
Upstream credit: [samtools](https://github.com/samtools/samtools) (MIT/Expat).
