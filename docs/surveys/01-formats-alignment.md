# Survey: formats & alignment-IO domain

Ground truth was refreshed from installed binaries on 2026-07-31:
samtools **1.24**, bcftools **1.24**, and bedtools **2.31.1**. Earlier
captures also covered deepTools, seqkit 2.13.0, fastp, and MethylDackel.
Product dossiers, rather than historical repository names, are the current
implementation contracts.

## samtools 1.24 (40 subcommands) → `rsomics-bam`

| Upstream group | Operations | Product routing |
|---|---|---|
| Indexing | `dict`, `faidx`, `fqidx`, `index` | `index` stays in `rsomics-bam`; the three reference-sequence operations route to `rsomics-index` |
| Editing | `calmd`, `fixmate`, `reheader`, `targetcut`, `addreplacerg`, `markdup`, `ampliconclip` | `rsomics-bam` subcommands |
| File operations | `collate`, `cat`, `consensus`, `merge`, `mpileup`, `sort`, `split`, `quickcheck`, `fastq`, `fasta`, `import`, `reference`, `reset` | `rsomics-bam` subcommands |
| Statistics | `bedcov`, `coverage`, `depth`, `flagstat`, `idxstats`, `cram-size`, `phase`, `stats`, `ampliconstats`, `checksum` | `rsomics-bam` subcommands |
| Viewing | `flags`, `head`, `tview`, `view`, `depad`, `samples` | `rsomics-bam` subcommands |

The historical operation crates are source assets. `region`, `subsample`, and
`sam-to-bam` collapse into `view`; `to-fastq` becomes `fastq`; `to-bed`
becomes a conversion subcommand. deepTools signal operations route to
`rsomics-signal`, while RSeQC/regtools/Picard RNA-seq QC routes to
`rsomics-rnaseq-qc`. See the [BAM dossier](../10-products/bam.md).

## bedtools 2.31.1 (43 subcommands) → `rsomics-bed-*`

| bedtools op | Historical assets | Target |
|---|---|---|
| annotate closest cluster complement coverage fisher flank genomecov getfasta intersect jaccard makewindows map maskfasta merge multicov multiinter nuc overlap random reldist sample shift shuffle slop sort split subtract summary unionbedg window | Per-operation crates plus overlapping `bed-utils` modules | `rsomics-bed` subcommands |
| expand / groupby | Per-operation crates | `rsomics-table` |
| bamtobed / bamtofastq | BAM conversion assets | `rsomics-bam` |
| bedtobam / bedpetobam / pairtobed / pairtopair / tag / igv / links | No accepted complete asset | Later only after product-fit and compatibility review |

The old per-operation and `bed-utils` implementations duplicate behavior and
sometimes disagree. Neither repository shape is canonical. Selected code,
fixtures, and benchmarks move into modules of one product; small inspection
operations consolidate rather than remain installable crates. See the
[interval dossier](../10-products/interval-annotation-index.md#rsomics-bed).

## bcftools 1.24 (23 operations) → workflow products

| bcftools surface | Current decision |
|---|---|
| `annotate`, `concat`, `consensus`, `convert`, `filter`, `head`, `index`, `isec`, `merge`, `norm`, `query`, `reheader`, `sort`, `stats`, `view` | `rsomics-vcf` format subcommands |
| `mpileup`, `call` | `rsomics-call` calling workflow |
| `cnv`, `polysomy` | `rsomics-cnv` copy-number workflow |
| `csq` and consequence plugins | `rsomics-annotation` |
| `gtcheck`, `roh`, and genotype-QC/LD/association/family plugins | `rsomics-plink` |
| Remaining plugins | Retain only named behavior that fits a reviewed product; no bcftools C plugin ABI |

`rsomics-vcf-expr` does not qualify as a public foundation from historical
micro-crate dependents alone. Keep expression parsing inside `rsomics-vcf`
until a second target product demonstrates the same policy-free API. See the
[joint variant dossier](../10-products/variant.md).

## Verification status
- The samtools and bcftools operation lists are binary-verified against 1.24;
  bedtools remains binary-verified against 2.31.1.
- Historical `tests/compat.rs` presence is not proof of compatibility. Many
  suites skip without the oracle, pin older versions, or compare only partial
  fields.
- Product release evidence must run the pinned oracle, validate complete
  observable behavior, and pass exact-head CI on all four native platform
  classes.
