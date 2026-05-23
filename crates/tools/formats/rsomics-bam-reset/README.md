# rsomics-bam-reset

Revert aligner-applied changes on BAM reads — a Rust port of `samtools reset`.

Each primary alignment is reverted to its pre-alignment state:

- FLAG is reduced to read/mate pairing plus unmapped bits: `PROPER_PAIR`,
  `MREVERSE`, `REVERSE` and (unless `--dupflag`) `DUP` are cleared; `UNMAP` is
  set, and `MUNMAP` is set when the read is paired.
- RNAME, POS, MAPQ, CIGAR, RNEXT, PNEXT and TLEN go to their unmapped defaults
  (`*` / `0` / no CIGAR / `-1`).
- A record stored reverse-complemented (the `0x10` reverse bit) has its SEQ and
  QUAL restored to original read orientation.
- Aligner aux tags are removed (default set: `AS CC CG CP H1 H2 HI H0 IH MC MD
  MQ NM SA TS`); `--remove-tag`/`-x` adds to that set, `--keep-tag` switches to
  keep-only mode.
- Secondary (`0x100`) and supplementary (`0x800`) records are dropped.

The output header is rebuilt from scratch: `@HD VN:1.6`, the input `@RG` lines
(unless `--no-RG`) and `@PG` chain (truncated by `--reject-PG`), plus a
provenance `@PG` for this command (unless `--no-PG`). All `@SQ` lines are
dropped — the reads are unmapped, so the output references nothing.

## Usage

```
rsomics-bam-reset <input.bam> [-o out.bam]
rsomics-bam-reset <input.bam> --keep-tag RG,BC -o out.bam
rsomics-bam-reset <input.bam> -x XS,YT --no-RG -o out.bam
```

## Scoped flags

`-O`/`--output-fmt` (CRAM/SAM output selection) is scoped out — this crate emits
BAM, matching the one-operation rule. The provenance `@PG` line records
`PN:rsomics-bam-reset` and our `CL:`, which differs by design from samtools'
`PN:samtools` line.

## Origin

This crate is an independent Rust reimplementation of `samtools reset` based on
the samtools source (`reset.c`, MIT licence, commit 1.23). Reading and citing
the source is permitted because samtools is MIT-licensed. The implementation
mirrors:

- the FLAG reduction (`reset.c:354-362`),
- the secondary/supplementary drop (`reset.c:350-352`),
- the reverse-complement restore using the `=TGKCYSBAWRDMHVN` table
  (`reset.c:377-391`),
- the default aux-tag removal set and `--keep-tag`/`--remove-tag` interaction
  (`reset.c:80-159`),
- the rebuilt-header `@HD`/`@RG`/`@PG` logic (`reset.c:307-323`).

Test fixtures are independently generated.

License: MIT OR Apache-2.0.
Upstream credit: samtools <https://github.com/samtools/samtools> (MIT).
