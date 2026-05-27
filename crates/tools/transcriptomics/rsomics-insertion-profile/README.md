# rsomics-insertion-profile

Per-position CIGAR-insertion rate along the read — Rust port of RSeQC `insertion_profile.py`.

For each read cycle position (0-based, 0..read_length-1), counts how many reads carry a CIGAR
`I` (insertion) operation at that query position and reports the inserted / non-inserted counts.

## Install

```
cargo install rsomics-insertion-profile
```

## Usage

```
rsomics-insertion-profile -i input.bam -o prefix [-s SE|PE] [--mapq N] [-t threads]
```

### Flag table

| Flag | Long | Default | Description |
|------|------|---------|-------------|
| `-i` | `--input-file` | required | Input BAM or SAM file |
| `-o` | `--out-prefix` | required | Prefix for output files |
| `-s` | `--sequencing` | `SE` | Sequencing layout: `SE` or `PE` |
| | `--mapq` | `30` | Minimum mapping quality |
| `-t` | `--threads` | auto | BAM decode threads |
| `-q` | `--quiet` | false | Suppress progress messages |
| | `--json` | false | Emit JSON summary to stdout |
| `-h` | `--help` | | Show help |

### Output files

**SE layout:**
- `<prefix>.insertion_profile.xls` — tab-separated table: `Position`, `Insert_nt`, `Non_insert_nt`
- `<prefix>.insertion_profile.r` — R script for plotting

**PE layout:**
- `<prefix>.insertion_profile.xls` — same header; `Read-1:` and `Read-2:` section labels separate mate tables
- `<prefix>.insertion_profile.r` — R script with two `pdf()` blocks (`.R1.pdf` / `.R2.pdf`)

## Origin

This crate is an independent Rust reimplementation of `RSeQC` `insertion_profile.py` based on:
- The published method: Wang et al. 2012 <https://doi.org/10.1093/bioinformatics/bts356>
- The public SAM/BAM format specification (CIGAR `I` operation)
- Black-box behaviour testing against `RSeQC` 5.0.4
  (`insertion_profile.py` — GPL-v2+; source not read; clean-room implementation)

No source code from the GPL upstream was used as reference during implementation.
Test fixtures are independently generated.

License: MIT OR Apache-2.0.  
Upstream credit: `RSeQC` <https://rseqc.sourceforge.net/> (GPL-v2+).
