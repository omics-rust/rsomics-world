# rsomics-mismatch-profile

Per-base mismatch-rate profile from BAM MD tags.

Produces a position-by-mismatch-type count table (`.mismatch_profile.xls`) and an R visualisation script (`.mismatch_profile.r`) identical in format to RSeQC `mismatch_profile.py`.

## Usage

```
rsomics-mismatch-profile -i <in.bam> -l <read_len> -o <prefix> [options]

Options:
  -i, --input <FILE>              Input BAM (must carry MD tags)
  -l, --read-align-length <N>     Declared read alignment length (e.g. 100)
  -o, --out-prefix <PREFIX>       Output prefix
  -n, --read-num <N>              Max reads to process [default: 1000000]
      --mapq <N>                  Minimum mapping quality [default: 30]
  -t, --threads <N>               Decode threads [default: 1]
  -h, --help                      Print help
```

## Output

| File | Contents |
|------|----------|
| `<prefix>.mismatch_profile.xls` | TSV: `read_pos  sum  A2C  A2G  A2T  C2A  C2G  C2T  G2A  G2C  G2T  T2A  T2C  T2G` |
| `<prefix>.mismatch_profile.r`   | R script; run with `Rscript <prefix>.mismatch_profile.r` to produce PDF |

Unmapped, secondary, and supplementary alignments are skipped. Reads below `--mapq` are skipped. Reads without an MD tag are skipped.

## Origin

This crate is an independent Rust reimplementation of `mismatch_profile.py` from RSeQC based on:

- The RSeQC documentation: <https://rseqc.sourceforge.net/#mismatch-profile-py>
- The BAM/SAM format specification (MD tag semantics)
- Black-box behaviour testing against the upstream binary

No source code from the GPL-2 upstream (RSeQC) was used as reference during implementation. Test fixtures are independently generated synthetic BAM files.

License: MIT OR Apache-2.0  
Upstream credit: RSeQC <https://rseqc.sourceforge.net/> (GPL-2.0)
