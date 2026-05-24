#!/usr/bin/env python3
"""Generate golden fixtures for rsomics-bam-read-dist compat tests.

Produces:
  tests/golden/genes.bed12   — 3 genes with multi-exon structure, UTRs, and introns
  tests/golden/reads.bam     — reads landing in CDS, UTR, intron, TSS-up, and intergenic

Run from the crate root:
  python3 tests/make_golden.py
"""

import os
import random
import struct
import pysam

random.seed(42)

OUT = os.path.join(os.path.dirname(__file__), "golden")
os.makedirs(OUT, exist_ok=True)

# ─── BED12 gene model ────────────────────────────────────────────────────────
# Three genes on chr1 (+ strand), one on chr1 (- strand).
# BED12 cols: chrom,start,end,name,score,strand,thickStart,thickEnd,rgb,blockCount,blockSizes,blockStarts
GENES = [
    # Gene A (+): tx 1000-5000, CDS 1200-4800, 3 exons
    # exon1: 1000-1500 (500bp), intron: 1500-2000, exon2: 2000-3500 (1500bp), intron: 3500-4000, exon3: 4000-5000 (1000bp)
    # 5'UTR: 1000-1200 (in exon1), 3'UTR: 4800-5000 (in exon3)
    "chr1\t1000\t5000\tGENE_A\t0\t+\t1200\t4800\t0\t3\t500,1500,1000,\t0,1000,3000,",
    # Gene B (+): tx 10000-15000, CDS 10500-14500, 2 exons
    # exon1: 10000-11000 (1000bp), intron: 11000-13000, exon2: 13000-15000 (2000bp)
    # 5'UTR: 10000-10500, 3'UTR: 14500-15000
    "chr1\t10000\t15000\tGENE_B\t0\t+\t10500\t14500\t0\t2\t1000,2000,\t0,3000,",
    # Gene C (-): tx 20000-25000, CDS 20500-24500, 2 exons
    # exon1: 20000-22000 (2000bp), intron: 22000-23000, exon2: 23000-25000 (2000bp)
    # minus strand: 5'UTR = end>cdsEnd, 3'UTR = start<cdsStart
    # 5'UTR: 24500-25000 (in exon2), 3'UTR: 20000-20500 (in exon1)
    "chr1\t20000\t25000\tGENE_C\t0\t-\t20500\t24500\t0\t2\t2000,2000,\t0,3000,",
]

BED12_PATH = os.path.join(OUT, "genes.bed12")
with open(BED12_PATH, "w") as f:
    for g in GENES:
        f.write(g + "\n")

print(f"Wrote {BED12_PATH}")

# ─── BAM ─────────────────────────────────────────────────────────────────────
# Chromosome lengths
CHROM_LENGTHS = {"chr1": 100_000}
READ_LEN = 100

header = pysam.AlignmentHeader.from_dict({
    "HD": {"VN": "1.6", "SO": "coordinate"},
    "SQ": [{"SN": "chr1", "LN": 100_000}],
})

reads = []

def make_read(name, chrom, start, cigar, is_reverse=False):
    a = pysam.AlignedSegment(header)
    a.query_name = name
    a.reference_id = 0
    a.reference_start = start
    a.cigar = cigar
    a.mapping_quality = 60
    a.query_sequence = "A" * READ_LEN
    a.query_qualities = pysam.qualitystring_to_array("I" * READ_LEN)
    a.flag = 0x10 if is_reverse else 0
    return a

# CDS reads: mid-point lands in CDS exons of GENE_A (exon1 CDS: 1200-1500, exon2: 2000-3500, exon3: 4000-4800)
# Read in CDS exon2 of GENE_A: start=2200, cigar=100M → mid=2250, CDS
for i in range(20):
    reads.append(make_read(f"cds_a_{i}", "chr1", 2200 + i * 2, [(0, READ_LEN)]))

# CDS reads in GENE_B exon1 CDS (10500-11000): start=10550, mid=10600
for i in range(10):
    reads.append(make_read(f"cds_b_{i}", "chr1", 10550 + i * 2, [(0, READ_LEN)]))

# 5'UTR reads in GENE_A (1000-1200): start=1050, mid=1100
for i in range(8):
    reads.append(make_read(f"utr5_a_{i}", "chr1", 1050 + i, [(0, READ_LEN)]))

# 3'UTR reads in GENE_A (4800-5000): start=4830, mid=4880
for i in range(6):
    reads.append(make_read(f"utr3_a_{i}", "chr1", 4830 + i, [(0, READ_LEN)]))

# Intron reads in GENE_A intron1 (1500-2000): start=1600, mid=1650
for i in range(10):
    reads.append(make_read(f"intron_a_{i}", "chr1", 1600 + i * 2, [(0, READ_LEN)]))

# TSS upstream reads for GENE_A (TSS=1000, up_1kb=[0,1000)): start=700, mid=750
for i in range(5):
    reads.append(make_read(f"tss_up_{i}", "chr1", 700 + i * 2, [(0, READ_LEN)]))

# TES downstream reads for GENE_A (TES=5000, down_1kb=[5000,6000)): start=5200, mid=5250
for i in range(5):
    reads.append(make_read(f"tes_down_{i}", "chr1", 5200 + i * 2, [(0, READ_LEN)]))

# Intergenic reads (far from any gene): start=50000, mid=50050
for i in range(5):
    reads.append(make_read(f"intergenic_{i}", "chr1", 50000 + i * 2, [(0, READ_LEN)]))

# Splice-junction read spanning GENE_A exon1-exon2 (split across intron):
# exon1 part: 1400-1450 (50M), intron skip: 550N, exon2 part: 2000-2050 (50M)
for i in range(4):
    reads.append(make_read(f"splice_{i}", "chr1", 1400 + i, [(0, 50), (3, 550), (0, 50)]))

# Sort by position
reads.sort(key=lambda r: r.reference_start)

BAM_PATH = os.path.join(OUT, "reads.bam")
with pysam.AlignmentFile(BAM_PATH, "wb", header=header) as bam:
    for r in reads:
        bam.write(r)

pysam.sort("-o", BAM_PATH + ".sorted.bam", BAM_PATH)
os.rename(BAM_PATH + ".sorted.bam", BAM_PATH)
pysam.index(BAM_PATH)

print(f"Wrote {BAM_PATH} ({len(reads)} reads)")
print("Done — run read_distribution.py to generate expected output")
