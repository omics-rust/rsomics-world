# rsomics-fpkm-count

Compute per-gene FPKM from a BAM file and a BED12 reference gene model.
Rust port of RSeQC `FPKM_count.py`.

## Usage

```
rsomics-fpkm-count -i sample.bam -r genes.bed12 -o output_prefix
```

Writes `output_prefix.FPKM.xls` (tab-separated).

## Flags

| Flag | Description | Default |
|---|---|---|
| `-i` | Input BAM (sorted) | required |
| `-r` | Reference gene model in BED12 | required |
| `-o` | Output prefix | required |
| `-q` | MAPQ threshold for unique-mapped reads | 30 |
| `-u` | Skip reads below MAPQ threshold | off |
| `-s` | Strand specificity rule (e.g. `1++,1--,2+-,2-+`) | unstranded |
| `-t` | Worker threads | all cores |

## Output format

Tab-separated with a `#`-prefixed header:

```
#chrom  st  end  accession  mRNA_size  gene_strand  Frag_count  FPM  FPKM
```

- `mRNA_size`: sum of BED12 block sizes (spliced transcript length)
- `Frag_count`: reads overlapping any exon of this gene (pairs count once)
- `FPM`: `(Frag_count / total_fragments) × 10⁶`
- `FPKM`: `(Frag_count × 10⁹) / (mRNA_size × total_fragments)`

## Strand rules

Strand rules follow RSeQC's `-d` syntax. Common presets:

| Protocol | Rule |
|---|---|
| Unstranded | *(omit -s)* |
| fr-firststrand (dUTP) | `1+-,1-+,2++,2--` |
| fr-secondstrand | `1++,1--,2+-,2-+` |
| Single-end same-strand | `++,--` |

Use `rsomics-infer-experiment` or RSeQC's `infer_experiment.py` if unsure.

## Origin

This crate is an independent Rust reimplementation of RSeQC `FPKM_count.py` based on:
- Liguo Wang et al., "RSeQC: quality control of RNA-seq experiments", *Bioinformatics* 28:16 (2012), DOI [10.1093/bioinformatics/bts356](https://doi.org/10.1093/bioinformatics/bts356)
- The BED12 format specification
- Black-box behaviour testing against `FPKM_count.py` 5.0.4

No source code from the GPL-2 upstream was used as reference during implementation.
Test fixtures are independently generated synthetic data.

License: MIT OR Apache-2.0.
Upstream credit: RSeQC <https://rseqc.sourceforge.net/> (GPL-2.0).
