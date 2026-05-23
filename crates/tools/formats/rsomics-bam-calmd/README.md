# rsomics-bam-calmd

Recompute the `MD` and `NM` aux tags of every alignment record against a
reference FASTA, then re-emit the BAM. Rust port of `samtools calmd`.

## Install

```
cargo install rsomics-bam-calmd
```

## Usage

```
# Recompute MD + NM against an indexed reference (writes BGZF BAM)
rsomics-bam-calmd aln.bam ref.fa -o calmd.bam

# Convert reference-matching read bases to '=' (compact storage)
rsomics-bam-calmd aln.bam ref.fa -e -o calmd.bam

# Multi-threaded BGZF inflate + deflate
rsomics-bam-calmd aln.bam ref.fa -t 4 -o calmd.bam

# To stdout (pipe into another BAM consumer)
rsomics-bam-calmd aln.bam ref.fa > calmd.bam
```

The reference must be indexed (`ref.fa.fai` alongside it — `samtools faidx
ref.fa`). Each contig is fetched once on first use, so coordinate-sorted input
reads every contig exactly once.

## MD / NM semantics

Sourced from `samtools` `bam_md.c` (`bam_fillmd1_core`) — samtools is
MIT-licensed, so reading and citing the source is the established practice in
this project.

The CIGAR is walked with a read cursor (`qpos`) and a 0-based reference cursor
(`rpos`, starting at the record's `POS`). Bases are compared at the **4-bit
nucleotide-code level** (htslib `seq_nt16_table` / `bam_seqi`), not as ASCII:

| CIGAR op | Consumes | Behaviour |
|---|---|---|
| `M` `=` `X` | read + ref | per base: a **match** needs equal non-N codes, or a read code of `0` (`=`); a **mismatch** flushes the current match run-length into MD, appends the uppercased ref base, and bumps NM |
| `D` | ref | flush run-length, emit `^` then the uppercased deleted ref bases; NM += deleted bases |
| `I` | read | NM += inserted bases (MD untouched) |
| `S` | read | advances the read only |
| `N` | ref | advances the reference only |
| `H` `P` | neither | no effect |

`MD` is the resulting run-length string (matches as decimal counts, mismatches
as the ref base, deletions as `^<bases>`), terminated by the trailing match
count. `NM` = mismatches + inserted + deleted bases. An N in the **read** or the
**ref** (code 15) is treated as a mismatch, so a read over an `N` reference
stretch yields e.g. `MD:Z:4N0N0N0N2`.

Tag updates follow samtools exactly: a tag is **appended** when absent,
**left in place** (preserving its position in the aux block and its on-wire
integer subtype) when its value already matches, and **deleted then re-appended
at the end** when the value differs. Unmapped records, records with no stored
sequence, and contigs missing from the reference are passed through untouched.

### Flags

| Flag | Status |
|---|---|
| (default) recompute `MD` + `NM` | implemented |
| `-e` convert matching bases to `=` | implemented |
| `-o FILE` output path (default stdout) | implemented |
| `-t N` BGZF worker threads | implemented |
| `-A`, `-r`, `-E`, `-C` (BAQ realignment / base-quality capping) | **deferred** — these invoke htslib `sam_prob_realn` / `sam_cap_mapq`, a large separate algorithm out of scope for the MD/NM recompute |
| `-b`, `-u`, `-S` (output-format selectors) | **deferred** — output is always BGZF BAM; SAM/uncompressed selectors are not exposed |
| `-q` (bin base qualities), `-d` (drop non-RG tags), `-N` (no MD/NM) | **deferred** — niche flags not part of the default recompute |

## Performance

The fair comparison is decode + reference walk + BGZF write on both sides
(samtools calmd also decodes SEQ and walks the reference). The levers are the
shared libdeflate BGZF reader/writer (`rsomics-bamio`), a branch-lean MD walk,
and an overlapping parallel deflate thread on output. See
`.autopilot/state/perf-*.md` for recorded ratios.

## Origin

Independent Rust reimplementation of `samtools calmd`, informed by:

- Reading the upstream source: `samtools` `bam_md.c` (`bam_fillmd1_core`,
  `bam_fillmd`) and htslib `hts.c` (`seq_nt16_table`) — samtools/htslib are
  MIT-licensed, so source reading is allowed and is the established practice for
  matching upstream semantics in this project.
- Black-box behaviour comparison via byte-equal output against
  `samtools calmd` (see `tests/compat.rs`).

License: MIT OR Apache-2.0.
Upstream credit: [samtools](https://github.com/samtools/samtools) (MIT).

### External-dep quadrant classification

- `rsomics-bamio` — the workspace BAM reader/writer: libdeflate BGZF, plain
  single-threaded reader at one worker, parallel inflate/deflate at `>= 2`
  (Quadrant ② for the bundled-C libdeflate inner loop).
- `noodles` (`sam`, `bam`, `fasta`, `core`) — pure-Rust format codecs and the
  indexed-FASTA reader (Quadrant ①).
- `rsomics-common`, `rsomics-help`, `clap`, `serde`, `serde_json` — Quadrant ④.
