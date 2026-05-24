# rsomics-bam-read-dist

Classify mapped reads into genomic regions (CDS exons, 5'/3' UTR, introns,
TSS upstream, TES downstream) from a BAM file and a BED12 gene model.

Rust port of `RSeQC` `read_distribution.py`.

## Usage

```
rsomics-bam-read-dist -i aligned.bam -r genes.bed12
```

Output matches the exact text format of `RSeQC` `read_distribution.py`:

```
Total Reads                   1234567
Total Tags                    1289012
Total Assigned Tags           1198340
=====================================================================
Group               Total_bases         Tag_count           Tags/Kb
CDS_Exons           ...
5'UTR_Exons         ...
3'UTR_Exons         ...
Introns             ...
TSS_up_1kb          ...
TSS_up_5kb          ...
TSS_up_10kb         ...
TES_down_1kb        ...
TES_down_5kb        ...
TES_down_10kb       ...
=====================================================================
```

## Options

| Flag | Default | Description |
|---|---|---|
| `-i` / `--input` | required | Input BAM (must be indexed) |
| `-r` / `--refgene` | required | BED12 gene model |
| `-t` / `--threads` | all CPUs | BGZF decode threads |
| `--json` | off | Emit machine-readable JSON |

## Performance

Measured on macOS (Apple M2), 64 k reads, 200-gene BED12, single thread.

| Tool | Time (mean) | Ratio |
|---|---|---|
| RSeQC 5.0.4 `read_distribution.py` | 303.6 ms | 1.00× |
| rsomics-bam-read-dist 0.1.0 `-t1` | 13.1 ms | **23.2×** |

## Origin

This crate is an independent Rust reimplementation of `RSeQC`
`read_distribution.py` based on:

- Wang L, Wang S, Li W. RSeQC: quality control of RNA-seq experiments.
  *Bioinformatics*. 2012;28(16):2184-2185.
  <https://doi.org/10.1093/bioinformatics/bts356>
- The SAM/BAM format specification
- BED12 format specification
- Black-box behaviour testing against `RSeQC` 5.0.4

No source code from the LGPL upstream was used as reference during
implementation. Test fixtures are independently generated with `pysam`.

License: MIT OR Apache-2.0.
Upstream credit: `RSeQC` <https://rseqc.sourceforge.net/> (LGPL-2.1+).
