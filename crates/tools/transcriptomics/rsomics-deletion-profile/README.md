# rsomics-deletion-profile

Per-base CIGAR-deletion rate along aligned reads.

For each aligned read whose query length matches `-l`, every CIGAR `D` operation contributes one count at the read position where the deletion starts (0-indexed, 5'→3'). Output is a tab-delimited `.deletion_profile.txt` and an R companion `.deletion_profile.r` suitable for plotting.

## Usage

```
rsomics-deletion-profile -i <BAM> -l <read_length> -o <prefix> [options]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-i` / `--input` | required | Input BAM file |
| `-l` / `--read-align-length` | required | Read alignment length; reads not matching are skipped |
| `-o` / `--out-prefix` | required | Output file prefix |
| `-n` / `--read-num` | 1 000 000 | Max reads with deletions to use |
| `-q` / `--mapq` | 30 | Minimum mapping quality |
| `-t` / `--threads` | all cores | BGZF decode workers |

## Origin

This crate is an independent Rust reimplementation of `deletion_profile.py` (RSeQC) based on:

- The RSeQC documentation at <https://rseqc.sourceforge.net/#deletion-profile-py>
- Black-box behaviour testing against the upstream binary (output format verified
  field-by-field against RSeQC 5.0.3)

No GPL source code from RSeQC was used as reference during implementation.
Test fixtures are independently generated synthetic BAM records.

License: MIT OR Apache-2.0.
Upstream credit: RSeQC <https://github.com/MonashBioinformatics/RSeQC> (GPL-2.0).
