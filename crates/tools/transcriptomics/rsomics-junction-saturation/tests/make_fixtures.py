#!/usr/bin/env python3
"""
Generate golden fixtures for rsomics-junction-saturation tests.

Creates a small BAM with splice junctions and a matching BED12 annotation.
Run from the crate root:
    python3 tests/make_fixtures.py
"""
import os
import re
import subprocess

OUT_DIR = os.path.join(os.path.dirname(__file__), "golden")
os.makedirs(OUT_DIR, exist_ok=True)


def write_sam_and_convert(reads, header_lines, out_bam):
    sam_path = out_bam.replace(".bam", ".sam")
    with open(sam_path, "w") as f:
        for h in header_lines:
            f.write(h + "\n")
        for r in reads:
            f.write("\t".join(str(x) for x in r) + "\n")
    subprocess.run(["samtools", "sort", "-o", out_bam, sam_path], check=True)
    subprocess.run(["samtools", "index", out_bam], check=True)
    os.unlink(sam_path)


def seq_len(cigar):
    return sum(
        int(m) for m, op in ((g[:-1], g[-1]) for g in re.findall(r"\d+[A-Z=]", cigar))
        if op in "MIS=X"
    )


# Reference: chr1 (2000 bp) only — all junctions on chr1 so that
# junction_saturation.py (which filters reads whose chrom is absent from the
# BED annotation) sees the same reads as our tool.
header = [
    "@HD\tVN:1.6\tSO:coordinate",
    "@SQ\tSN:chr1\tLN:2000",
    "@PG\tID:make_fixtures",
]

read_id = 0


def make_read(chrom, pos0, cigar, flag=0):
    global read_id
    read_id += 1
    slen = seq_len(cigar)
    return [
        f"read{read_id:04d}", flag, chrom, pos0 + 1, 60, cigar,
        "*", 0, 0, "A" * slen, "I" * slen, "NH:i:1",
    ]


reads = []

# BED12 annotation has two genes on chr1:
#   GeneA: exon1=[0,200),   intron=[200,700),  exon2=[700,900)  → junction (200,700)
#   GeneB: exon1=[1000,1100), intron=[1100,1500), exon2=[1500,1700) → junction (1100,1500)
# Annotated splice sites: {200, 700, 1100, 1500}

# Gene A known reads: pos=0, 200M500N200M → junction (200, 700)
for _ in range(30):
    reads.append(make_read("chr1", 0, "200M500N200M"))

# Gene B known reads: pos=1000, 100M400N200M → junction (1100, 1500)
for _ in range(25):
    reads.append(make_read("chr1", 1000, "100M400N200M"))

# Partial-novel reads: pos=100, 100M200N100M → junction (200, 400)
#   donor 200 IS annotated (gene A donor), acceptor 400 is NOT → partial_novel
for _ in range(5):
    reads.append(make_read("chr1", 100, "100M200N100M"))

# Complete-novel reads: pos=200, 50M100N50M → junction (250, 350)
#   donor 250 NOT annotated, acceptor 350 NOT annotated → complete_novel
for _ in range(10):
    reads.append(make_read("chr1", 200, "50M100N50M"))


write_sam_and_convert(reads, header, os.path.join(OUT_DIR, "small.bam"))

# BED12 annotation: GeneA and GeneB on chr1.
bed12_lines = [
    # chrom start end name score strand thickStart thickEnd rgb blockCount blockSizes blockStarts
    "chr1\t0\t900\tGeneA\t0\t+\t0\t900\t0\t2\t200,200,\t0,700,",
    "chr1\t1000\t1700\tGeneB\t0\t+\t1000\t1700\t0\t2\t100,200,\t0,500,",
]
with open(os.path.join(OUT_DIR, "small.bed12"), "w") as f:
    for line in bed12_lines:
        f.write(line + "\n")

print(f"Generated {len(reads)} reads in {OUT_DIR}/small.bam")
print(f"Annotated junctions:")
print(f"  GeneA: (chr1, 200, 700) — known")
print(f"  GeneB: (chr1, 1100, 1500) — known")
print(f"Read junctions:")
print(f"  (chr1, 200, 700) x 30 — known (gene A)")
print(f"  (chr1, 1100, 1500) x 25 — known (gene B)")
print(f"  (chr1, 200, 400) x 5 — partial_novel (donor 200 annotated, acceptor 400 not)")
print(f"  (chr1, 250, 350) x 10 — complete_novel (neither donor nor acceptor annotated)")
