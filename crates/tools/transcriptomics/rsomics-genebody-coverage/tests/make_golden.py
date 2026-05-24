#!/usr/bin/env python3
"""Generate golden fixtures for rsomics-genebody-coverage compat tests.

Produces:
  tests/golden/genes.bed12   — 3 transcripts with multi-exon structure, mRNA >= 100 nt
  tests/golden/reads.bam     — reads distributed along gene bodies (incl a 3'-biased case)
  tests/golden/reads.bam.bai — BAM index

Run from the crate root:
  python3 tests/make_golden.py
"""

import os
import random
import pysam

random.seed(42)

OUT = os.path.join(os.path.dirname(__file__), "golden")
os.makedirs(OUT, exist_ok=True)

# ─── BED12 gene model ─────────────────────────────────────────────────────────
# Three transcripts; each has mRNA (total exon) length well above 100 nt.
#
# BED12: chrom start end name score strand thickStart thickEnd rgb blockCount blockSizes blockStarts
#
# GENE_A (+, chr1): 2 exons, 150bp + 200bp = 350bp mRNA
#   tx: 1000-1600; exon1: 1000-1150 (150bp), intron: 1150-1400, exon2: 1400-1600 (200bp)
#   blockSizes: 150,200   blockStarts: 0,400
#
# GENE_B (+, chr1): 2 exons, 200bp + 300bp = 500bp mRNA
#   tx: 5000-5800; exon1: 5000-5200 (200bp), intron: 5200-5500, exon2: 5500-5800 (300bp)
#   blockSizes: 200,300   blockStarts: 0,500
#
# GENE_C (-, chr1): 2 exons, 200bp + 200bp = 400bp mRNA
#   tx: 9000-9600; exon1: 9000-9200 (200bp), intron: 9200-9400, exon2: 9400-9600 (200bp)
#   blockSizes: 200,200   blockStarts: 0,400

GENES = [
    "chr1\t1000\t1600\tGENE_A\t0\t+\t1000\t1600\t0\t2\t150,200,\t0,400,",
    "chr1\t5000\t5800\tGENE_B\t0\t+\t5000\t5800\t0\t2\t200,300,\t0,500,",
    "chr1\t9000\t9600\tGENE_C\t0\t-\t9000\t9600\t0\t2\t200,200,\t0,400,",
]

BED12_PATH = os.path.join(OUT, "genes.bed12")
with open(BED12_PATH, "w") as f:
    for g in GENES:
        f.write(g + "\n")
print(f"Wrote {BED12_PATH}")

# ─── BAM ──────────────────────────────────────────────────────────────────────
header = pysam.AlignmentHeader.from_dict({
    "HD": {"VN": "1.6", "SO": "coordinate"},
    "SQ": [{"SN": "chr1", "LN": 200_000}],
})

READ_LEN = 50

def make_read(name, start, flag=0):
    a = pysam.AlignedSegment(header)
    a.query_name = name
    a.reference_id = 0
    a.reference_start = start
    a.cigar = [(0, READ_LEN)]  # 50M
    a.mapping_quality = 60
    a.query_sequence = "A" * READ_LEN
    a.query_qualities = pysam.qualitystring_to_array("I" * READ_LEN)
    a.flag = flag
    return a

reads = []

# GENE_A: uniform coverage across both exons.
# Exon1: 1000-1150, Exon2: 1400-1600.
# Reads starting in exon1: 1000, 1020, 1040, 1060 (all 50M, land within exon1)
for i, start in enumerate([1000, 1015, 1030, 1045, 1060, 1075, 1090]):
    reads.append(make_read(f"gA_ex1_{i}", start))
# Reads in exon2: 1400, 1420, 1440, 1460, 1480, 1500, 1520, 1540
for i, start in enumerate([1400, 1415, 1430, 1445, 1460, 1475, 1490, 1505, 1520, 1540]):
    reads.append(make_read(f"gA_ex2_{i}", start))

# GENE_B: 3'-biased coverage (more reads toward 3' end = higher start positions).
# Exon1: 5000-5200, Exon2: 5500-5800.
# Exon1: just 2 reads near the 5' end
for i, start in enumerate([5000, 5020]):
    reads.append(make_read(f"gB_ex1_{i}", start))
# Exon2: many reads near 3' end
for i, start in enumerate([5500, 5520, 5540, 5560, 5580, 5600, 5620, 5640, 5660, 5680, 5700, 5720]):
    reads.append(make_read(f"gB_ex2_{i}", start))

# GENE_C: minus-strand, uniform coverage.
# Exon1: 9000-9200, Exon2: 9400-9600.
# For minus strand: genomic 5' end is at tx_end (9600); 3' end is at tx_start (9000).
# Reads in exon1 (genomic start of chr): 9000, 9020, ..., 9140
for i, start in enumerate([9000, 9020, 9040, 9060, 9080, 9100, 9120, 9140]):
    reads.append(make_read(f"gC_ex1_{i}", start))
# Reads in exon2: 9400, 9420, ..., 9540
for i, start in enumerate([9400, 9420, 9440, 9460, 9480, 9500, 9520, 9540]):
    reads.append(make_read(f"gC_ex2_{i}", start))

# Sort by position (already sorted but ensure)
reads.sort(key=lambda r: r.reference_start)

BAM_PATH = os.path.join(OUT, "reads.bam")
with pysam.AlignmentFile(BAM_PATH, "wb", header=header) as bam:
    for r in reads:
        bam.write(r)

pysam.sort("-o", BAM_PATH + ".sorted.bam", BAM_PATH)
os.rename(BAM_PATH + ".sorted.bam", BAM_PATH)
pysam.index(BAM_PATH)

print(f"Wrote {BAM_PATH} ({len(reads)} reads)")
print("Done.")
