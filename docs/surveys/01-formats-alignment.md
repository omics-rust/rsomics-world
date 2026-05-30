# Survey: formats & alignment-IO domain

Ground truth captured from installed binaries 2026-05-30:
samtools **1.23.1**, bcftools **1.23.1**, bedtools **2.31.1**, deeptools (7 tools),
seqkit **2.13.0**, fastp, MethylDackel. Verification status per tool noted below.

## samtools 1.23.1 (40 subcommands) → `rsomics-bam-*`

| samtools op | our crate | status |
|---|---|---|
| addreplacerg ampliconclip ampliconstats bedcov calmd cat checksum collate consensus coverage depad depth dict fasta fastq(→to-fastq) fixmate flagstat head idxstats import index markdup merge mpileup phase quickcheck reheader reset samples sort split stats targetcut view | bam-<same> | ✓ canonical 1:1 |
| faidx / fqidx | fasta-index, (fastq idx gap) | ✓ / gap |
| flags (flag explainer) | — | gap (niche) |
| reference (CRAM ref) · tview (TUI) · cram-size | — | skip (CRAM/TUI out of scope) |

**Extra `rsomics-bam-*` crates from OTHER upstreams (legitimately distinct, not samtools):**
compare·fingerprint·signal = deeptools (bamCompare/plotFingerprint/bamCoverage);
junctions·read-dist·strandedness = RSeQC/regtools; subsample = `view -s`;
to-bed = bedtools bamtobed; region = convenience. → keep; they are real ops of
other tools. **Cross-tool note:** bam-coverage(ours, samtools coverage) vs
bam-signal(deeptools bamCoverage) vs bed-genomecov(bedtools) all touch "depth/
coverage" — different outputs (summary table vs bigWig vs per-base bedGraph);
NOT duplicates. Verified by output shape.

## bedtools 2.31.1 (43 subcommands) → `rsomics-bed-*`

| bedtools op | our per-op crate | also in bed-utils multitool? |
|---|---|---|
| annotate closest cluster complement coverage expand fisher flank genomecov getfasta groupby intersect jaccard makewindows map maskfasta(→utils) merge multicov multiinter nuc overlap random reldist sample shift shuffle slop sort spacing split subtract summary unionbedg window | bed-<same> | **YES — duplicated** |
| bamtobed / bamtofastq | bam-to-bed / bam-to-fastq | (bam side) |
| bedtobam / bedpetobam / pairtobed / pairtopair / tag / igv / links | — | gap / in utils (toigv) |

**⚠ THE duplication:** every per-op `bed-*` crate is ALSO a subcommand of the
`bed-utils` 54-op multitool — and they are NOT the same code (3-way vs bedtools
2.31.1: `bed-sort` byte-matches bedtools, `bed-utils sort` returns empty). The
per-op crate is canonical (project principle = one op one binary); bed-utils is
the bedtools-clone anti-pattern to retire (#89), after spinning out its
genuinely-unique ops (maskfasta, promoters, resize, rename, chroms, tofasta,
toigv, towig, totalspan) that have no standalone crate.

**Our `bed-*` not in bedtools:** count·len·midpoint·stats·total-bp·unique·to-gff
— small ops (seqkit/awk-class); keep as per-op, candidates for a future
`bed-utils`-as-thin-aggregator decision but NOT urgent.

## bcftools 1.23.1 (23) → `rsomics-vcf-*`

| bcftools op | our crate | status |
|---|---|---|
| annotate call cnv concat consensus convert csq filter gtcheck head index isec merge mpileup norm polysomy query reheader roh sort stats view | vcf-<same> | ✓ canonical 1:1 |
| plugin (+fill-tags/+setGT/+fixref) | vcf-fill-tags, vcf-setgt, vcf-fixref | ✓ (plugins as per-op crates) |

**Extra vcf crates:** expr = Layer-A filter engine (shared by filter/view/setgt —
correct dedup already); popgen = vcftools popgen stats; to-bed, validate, split,
sample = convenience/vcftools. No duplication here.

## Verification status
- samtools/bcftools/bedtools op lists: **binary-verified** (exact `--help` of the
  pinned versions). ✅
- semantics/flags per op: per-crate `tests/compat.rs` already pins byte/field
  equality vs these versions (the real verification layer).
- TODO: confirm samtools `flags`/`fqidx` and the bedtools bam/pair-conversion ops
  against source for the gap list; survey seqkit's ~40 subcommands (help format
  differs — pull from source) for the fasta/fastq/fastx crates.
