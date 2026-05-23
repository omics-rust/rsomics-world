# rsomics-pileup

Coordinate-sorted BAM pileup engine. Layer A primitive: given a stream of
coordinate-sorted records it yields one column per covered reference position,
each carrying the reads overlapping that position with their CIGAR-resolved
per-read state (`qpos`, `is_del`, `is_refskip`, `indel`, `is_head`, `is_tail`)
and overlapping-mate quality removal applied. This is the exact state
`samtools mpileup` and `samtools consensus` consume, so both tools build on this
one engine rather than each re-deriving the pileup (B never depends on B).

## Origin

This crate is a port of htslib's pileup engine. htslib is MIT-licensed, so its
source was read and the algorithm reproduced directly:

- `bam_plp_push` / `bam_plp64_next` — the active-read buffer and emit cursor.
- `resolve_cigar2` — per-read `qpos` / `is_del` / `indel` / `is_head` / `is_tail`
  with the deletion/insertion/pad look-ahead.
- `overlap_push` / `tweak_overlap_quality` / `cigar_iref2iseq_*` — overlapping
  proper-pair quality removal, including the `__ac_X31_hash_string` /
  `__ac_Wang_hash` name-hash keeper selection, reproduced bit-for-bit.

Records are read via `rsomics-bamio`'s `RawRecord` (no decode of seq/cigar/qual
into noodles types); each buffered read caches its decoded CIGAR once so the
per-position walk is O(1) amortised.

License: MIT OR Apache-2.0.
Upstream credit: [htslib](https://github.com/samtools/htslib) (MIT/Expat).
