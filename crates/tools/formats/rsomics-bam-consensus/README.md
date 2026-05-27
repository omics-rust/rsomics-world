# rsomics-bam-consensus

FASTA consensus from a coordinate-sorted BAM — Rust port of `samtools consensus` (simple mode).

## Install

```
cargo install rsomics-bam-consensus
```

## Usage

```
rsomics-bam-consensus [OPTIONS] <input.bam>
```

Writes a FASTA record per reference sequence to stdout (or `-o FILE`).

## Flags

| Flag | Default | Description |
|---|---|---|
| `-o FILE` / `--output FILE` | stdout | Write FASTA to FILE |
| `-q` / `--use-qual` | off | Weight each base by its Phred quality score |
| `--min-BQ INT` | 0 | Minimum base quality to count a base |
| `-d INT` / `--min-depth INT` | 1 | Minimum depth to call a non-N base |
| `-c FLOAT` / `--call-fract FLOAT` | 0.75 | Min fraction of total score required for a call |
| `-H FLOAT` / `--het-fract FLOAT` | 0.50 | Min 2nd-to-1st score ratio for a het call (needs `-A`) |
| `-A` / `--ambig` | off | Enable IUPAC ambiguity codes for het sites |
| `--show-del` | off | Emit deletion bases (`*`) in the consensus |
| `-l INT` / `--line-len INT` | 70 | FASTA line wrap length |
| `--ff INT` | UNMAP,SECONDARY,QCFAIL,DUP | Exclude reads with any of these FLAG bits |
| `--rf INT` | 0 | Include only reads with any of these FLAG bits |
| `--min-MQ INT` | 0 | Minimum mapping quality |
| `-t INT` / `--threads INT` | 1 | Reader thread count |

## Not implemented in 0.1.0

- **Bayesian mode** (`-m bayesian` / default in samtools) — the full Gap5 Bayesian
  consensus algorithm (calculate_consensus_gap5m). Planned for 0.2.0.
- **FASTQ output** (`-f FASTQ`) — quality string emission. Deferred.
- **Pileup output** (`-f PILEUP`) — per-position tab-delimited pileup. Deferred.
- **Alleles flag** (`--alleles`) — emitting allele information. Deferred.
- **Modification (mod) calling** — 5mC / 6mA base-modification aware consensus. Deferred.
- **Region query** (`-r REG`) — BAI-indexed region subsetting. Deferred.
- **Reference-fill** (`-a`) — output bases for uncovered positions from a reference FASTA. Deferred.

## Origin

This crate is a Rust port of `samtools consensus` simple mode, informed by:

- `samtools/bam_consensus.c` — MIT license, tag 1.23.1
- `samtools/consensus_pileup.c` — MIT license, tag 1.23.1
- The `samtools` man page and black-box behaviour testing

The upstream source is MIT-licensed so clean-room restrictions do not apply.
No GPL source was used.

License: MIT OR Apache-2.0.  
Upstream: samtools <https://github.com/samtools/samtools> (MIT).
