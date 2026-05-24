#!/usr/bin/env python3
"""Generate golden fixtures for rsomics-bam-junctions tests.

Produces:
  tests/golden/spliced.bam   -- reads with N-ops in CIGAR (spliced reads)
  tests/golden/genes.bed12   -- BED12 gene model with known introns

Gene model layout (chr1, 0-based):
  gene_A: chr1 1000-4000 (+ strand)
    exon1: 1000-1200  exon2: 2000-2200  exon3: 3000-4000
    known introns: [1200,2000) and [2200,3000)
    intron_starts (exon ends): 1200, 2200
    intron_ends   (exon starts): 2000, 3000

  gene_B: chr1 5000-8000 (- strand)
    exon1: 5000-5300  exon2: 6000-6300  exon3: 7000-8000
    known introns: [5300,6000) and [6300,7000)
    intron_starts: 5300, 6300
    intron_ends:   6000, 7000

Reads:
  r1:  chr1 1000, CIGAR 200M800N200M  → intron [1200,2000) KNOWN in gene_A
  r2:  chr1 2000, CIGAR 200M800N200M  → intron [2200,3000) KNOWN in gene_A
  r3:  chr1 1000, CIGAR 200M800N200M  → intron [1200,2000) KNOWN (same as r1 — 2 events, 1 junction)
  r4:  chr1 1000, CIGAR 200M1200N200M → intron [1200,2400) PARTIAL_NOVEL (start=1200 known, end=2400 not)
  r5:  chr1 5000, CIGAR 300M700N300M  → intron [5300,6000) KNOWN in gene_B
  r6:  chr1 1000, CIGAR 200M500N200M  → intron [1200,1700) NOVEL (start 1200 known but end 1700 not known → PARTIAL)
       Wait: intron_starts has 1200 (known), end 1700 not in intron_ends → PARTIAL_NOVEL
  r7:  chr1 100,  CIGAR 200M900N200M  → intron [300,1200) NOVEL (neither start=300 nor end=1200 in any set)
       end=1200? No: intron_ends = {2000, 3000, 6000, 7000} — 1200 is NOT there. start=300 not in intron_starts.
       → COMPLETE_NOVEL
  r8:  chr1 1000, CIGAR 200M30N200M   → intron [1200,1230) length=30 < min_intron=50 → FILTERED

So expected:
  total_events = 8 (all N-ops seen before filtering)
  filtered_events = 1 (r8)
  Passing events = 7:
    known: r1(1200-2000) + r2(2200-3000) + r3(1200-2000) + r5(5300-6000) = 4
    partial_novel: r4(1200-2400) + r6(1200-1700) = 2
    novel: r7(300-1200) = 1

  Distinct junctions from splicing_events dict:
    (1200,2000): known → known_junc
    (2200,3000): known → known_junc
    (5300,6000): known → known_junc
    (1200,2400): partial_novel → partial_junc
    (1200,1700): partial_novel → partial_junc
    (300,1200): novel → novel_junc
  Total junctions = 6
  known_junctions = 3
  partial_novel_junctions = 2
  novel_junctions = 1

  total = 7 (stdout: passing events)
"""
import os
import sys

PAIRED = 0x1
PROPER_PAIR = 0x2
MATE_REVERSE = 0x20
READ1 = 0x40
REVERSE = 0x10

SEQ = "A" * 200
QUAL = "I" * 200
MAPQ = 60


def make_bed12():
    lines = [
        # gene_A: 3 exons at 1000-1200, 2000-2200, 3000-4000 (+ strand)
        "chr1\t1000\t4000\tgene_A\t0\t+\t1000\t4000\t0\t3\t200,200,1000,\t0,1000,2000,",
        # gene_B: 3 exons at 5000-5300, 6000-6300, 7000-8000 (- strand)
        "chr1\t5000\t8000\tgene_B\t0\t-\t5000\t8000\t0\t3\t300,300,1000,\t0,1000,2000,",
    ]
    return "\n".join(lines) + "\n"


def make_read(name, flag, chrom, pos, cigar, seq, qual, mapq=60):
    return f"{name}\t{flag}\t{chrom}\t{pos}\t{mapq}\t{cigar}\t*\t0\t0\t{seq}\t{qual}\tNH:i:1"


def build_sam_reads():
    records = []
    # r1: known intron [1200,2000)
    records.append(make_read("r1", 0, "chr1", 1001, "200M800N200M", SEQ + SEQ, QUAL + QUAL))
    # r2: known intron [2200,3000)
    records.append(make_read("r2", 0, "chr1", 2001, "200M800N200M", SEQ + SEQ, QUAL + QUAL))
    # r3: same known intron [1200,2000) as r1 (2 events, 1 junction)
    records.append(make_read("r3", 0, "chr1", 1001, "200M800N200M", SEQ + SEQ, QUAL + QUAL))
    # r4: partial_novel intron [1200,2400) — start=1200 known, end=2400 not
    records.append(make_read("r4", 0, "chr1", 1001, "200M1200N200M", SEQ + SEQ, QUAL + QUAL))
    # r5: known intron [5300,6000) in gene_B
    records.append(make_read("r5", 0, "chr1", 5001, "300M700N300M", "A" * 300 + "A" * 300, "I" * 300 + "I" * 300))
    # r6: partial_novel intron [1200,1700) — start=1200 known, end=1700 not
    records.append(make_read("r6", 0, "chr1", 1001, "200M500N200M", SEQ + SEQ, QUAL + QUAL))
    # r7: complete_novel intron [300,1200) — neither known
    records.append(make_read("r7", 0, "chr1", 101, "200M900N200M", SEQ + SEQ, QUAL + QUAL))
    # r8: filtered intron [1200,1230) length=30 < 50
    records.append(make_read("r8", 0, "chr1", 1001, "200M30N200M", SEQ + SEQ, QUAL + QUAL))
    return records


def write_sam(path, records):
    with open(path, "w") as f:
        f.write("@HD\tVN:1.6\tSO:coordinate\n")
        f.write("@SQ\tSN:chr1\tLN:100000\n")
        for r in records:
            f.write(r + "\n")


if __name__ == "__main__":
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    golden = "golden"
    os.makedirs(golden, exist_ok=True)

    bed_path = os.path.join(golden, "genes.bed12")
    with open(bed_path, "w") as f:
        f.write(make_bed12())
    print(f"Written {bed_path}")

    sam_path = os.path.join(golden, "spliced.sam")
    bam_path = os.path.join(golden, "spliced.bam")
    records = build_sam_reads()
    write_sam(sam_path, records)
    print(f"Written {sam_path} ({len(records)} records)")

    ret = os.system(f"samtools sort -o {bam_path} {sam_path} && samtools index {bam_path}")
    if ret != 0:
        print("ERROR: samtools not found or failed", file=sys.stderr)
        sys.exit(1)
    os.remove(sam_path)
    print(f"Written {bam_path} (sorted + indexed)")
    print()
    print("Expected counts (with -m 50 -q 30):")
    print("  total_events = 7 (8 N-ops total, 1 filtered)")
    print("  filtered_events = 1 (r8 intron length 30)")
    print("  known_events = 4  (r1, r2, r3, r5)")
    print("  partial_novel_events = 2  (r4, r6)")
    print("  novel_events = 1  (r7)")
    print("  known_junctions = 3  ([1200,2000), [2200,3000), [5300,6000))")
    print("  partial_novel_junctions = 2  ([1200,2400), [1200,1700))")
    print("  novel_junctions = 1  ([300,1200))")
