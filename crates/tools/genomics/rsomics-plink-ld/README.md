# rsomics-plink-ld

Pairwise LD (r²) computation from PLINK1 binary filesets.

## Usage

```
rsomics-plink-ld --plink data --out data.ld
rsomics-plink-ld --plink data --window 100 --min-r2 0.2 --out data.ld
```

## Flags

| Flag | Default | Description |
|---|---|---|
| `-p, --plink` | (required) | PLINK1 binary prefix (reads `.bed`/`.bim`/`.fam`) |
| `-o, --out` | stdout | Output file path |
| `-w, --window` | 50 | Sliding window size in variants (0 = all pairs per chromosome) |
| `--min-r2` | 0.0 | Minimum r² to report (0 = all pairs) |
| `-t, --threads` | 1 | Number of threads |

## Output format

Tab-separated, one pair per line, matching PLINK1 `--r2` `.ld` format:

```
CHR_A   BP_A    SNP_A   CHR_B   BP_B    SNP_B   R2
1       100     rs1     1       200     rs2     0.123456
```

## Origin

This crate is an independent Rust implementation of the pairwise LD (r²)
computation described in:

- Purcell et al. (2007) PLINK: A Tool Set for Whole-Genome Association and
  Population-Based Linkage Analyses. *Am J Hum Genet* 81(3):559-575.
  doi:10.1086/519795

The algorithm (additive dosage Pearson r², windowed pairwise) is derived from
the published method and format specification. PLINK1 binary format specification:
<https://www.cog-genomics.org/plink/1.9/formats#bed>.

License: MIT OR Apache-2.0.
Upstream credit: PLINK 1.9 <https://www.cog-genomics.org/plink/1.9/> (GPL-3.0).
