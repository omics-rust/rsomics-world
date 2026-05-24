#!/usr/bin/env python3
"""Generate golden fixtures for rsomics-read-duplication tests.

Produces:
  tests/golden/dup.bam  -- synthetic BAM with controlled duplicates
  tests/golden/dup.bam.bai

Design (seeded for reproducibility):
  - chr1:1000  ACGTACGTACGTACGTACGTACGT  24M  MAPQ=60  -- appears 3 times (seq dup=3, pos dup=3)
  - chr1:2000  TTTTTTTTTTTTTTTTTTTTTTTT  24M  MAPQ=60  -- appears 2 times (seq dup=2, pos dup=2)
  - chr1:3000  GGGGGGGGGGGGGGGGGGGGGGGG  24M  MAPQ=60  -- appears 1 time  (seq dup=1, pos dup=1)
  - chr1:4000  CCCCCCCCCCCCCCCCCCCCCCCC  24M  MAPQ=60  -- appears 1 time  (seq dup=1, pos dup=1)
  - chr1:1000  TTTTTTTTTTTTTTTTTTTTTTTT  24M  MAPQ=60  -- seq_dup[TTTT]+=1 but pos is chr1:1000:
                                                           same pos as ACGT reads? No, different seq
                                                           pos key = chr1:1000:1000-1024: -- also 3 times
                                                           so pos_dup[chr1:1000:...] gets +1 here too

Actually let's be explicit and simple:

Read set:
  r1  chr1:1000  SEQ_A  (3 copies → pos key P1, seq key S_A)
  r2  chr1:1000  SEQ_A  (duplicate of r1)
  r3  chr1:1000  SEQ_A  (duplicate of r1)
  r4  chr1:2000  SEQ_B  (2 copies → pos key P2, seq key S_B)
  r5  chr1:2000  SEQ_B  (duplicate of r4)
  r6  chr1:3000  SEQ_C  (unique → pos key P3, seq key S_C)
  r7  chr1:4000  SEQ_D  (unique → pos key P4, seq key S_D)
  r8  chr1:5000  SEQ_A  (SEQ_A appears at NEW position — seq S_A count=4, pos key P5 count=1)

Wait, let's keep it simple for easy verification:

  3x chr1:1000 SEQ_A → seqDup[SEQ_A]=3, posDup[P1]=3
  2x chr1:2000 SEQ_B → seqDup[SEQ_B]=2, posDup[P2]=2
  1x chr1:3000 SEQ_C → seqDup[SEQ_C]=1, posDup[P3]=1
  1x chr1:4000 SEQ_D → seqDup[SEQ_D]=1, posDup[P4]=1

After inversion:
  seq_hist: {3:1, 2:1, 1:2}
  pos_hist: {3:1, 2:1, 1:2}

.seq.DupRate.xls:
  Occurrence\\tUniqReadNumber
  1\\t2
  2\\t1
  3\\t1

.pos.DupRate.xls: (identical in this design)
  Occurrence\\tUniqReadNumber
  1\\t2
  2\\t1
  3\\t1
"""
import os
import struct
import sys

PAIRED = 0x1
PROPER_PAIR = 0x2
UNMAPPED = 0x4
MATE_UNMAPPED = 0x8
REVERSE = 0x10
MATE_REVERSE = 0x20
READ1 = 0x40
READ2 = 0x80
SECONDARY = 0x100
QCFAIL = 0x200
DUPLICATE = 0x400

SEQ_A = "ACGTACGTACGTACGTACGTACGT"  # 24 bp
SEQ_B = "TTTTTTTTTTTTTTTTTTTTTTTT"  # 24 bp
SEQ_C = "GGGGGGGGGGGGGGGGGGGGGGGG"  # 24 bp
SEQ_D = "CCCCCCCCCCCCCCCCCCCCCCCC"  # 24 bp
QUAL = "I" * 24
MAPQ = 60
CIGAR = "24M"


def build_reads():
    """Return list of SAM records (tab-separated strings, no header)."""
    records = []

    # 3 copies of SEQ_A at chr1:1000
    for i in range(3):
        records.append(
            f"read_A_{i+1}\t0\tchr1\t1000\t{MAPQ}\t{CIGAR}\t*\t0\t0\t{SEQ_A}\t{QUAL}"
        )

    # 2 copies of SEQ_B at chr1:2000
    for i in range(2):
        records.append(
            f"read_B_{i+1}\t0\tchr1\t2000\t{MAPQ}\t{CIGAR}\t*\t0\t0\t{SEQ_B}\t{QUAL}"
        )

    # 1 copy of SEQ_C at chr1:3000
    records.append(
        f"read_C_1\t0\tchr1\t3000\t{MAPQ}\t{CIGAR}\t*\t0\t0\t{SEQ_C}\t{QUAL}"
    )

    # 1 copy of SEQ_D at chr1:4000
    records.append(
        f"read_D_1\t0\tchr1\t4000\t{MAPQ}\t{CIGAR}\t*\t0\t0\t{SEQ_D}\t{QUAL}"
    )

    # 1 read with MAPQ=10 (below default threshold of 30) — should be EXCLUDED
    records.append(
        f"read_lowq\t0\tchr1\t5000\t10\t{CIGAR}\t*\t0\t0\t{SEQ_A}\t{QUAL}"
    )

    # 1 QC-fail read — should be EXCLUDED
    records.append(
        f"read_qcfail\t{QCFAIL}\tchr1\t6000\t{MAPQ}\t{CIGAR}\t*\t0\t0\t{SEQ_A}\t{QUAL}"
    )

    return records


def write_sam(path, records):
    with open(path, "w") as f:
        f.write("@HD\tVN:1.6\tSO:coordinate\n")
        f.write("@SQ\tSN:chr1\tLN:1000000\n")
        for r in records:
            f.write(r + "\n")


if __name__ == "__main__":
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    golden = "golden"
    os.makedirs(golden, exist_ok=True)

    sam_path = os.path.join(golden, "dup.sam")
    bam_path = os.path.join(golden, "dup.bam")

    records = build_reads()
    write_sam(sam_path, records)
    print(f"Written {sam_path} ({len(records)} records)")

    ret = os.system(
        f"samtools sort -o {bam_path} {sam_path} && samtools index {bam_path}"
    )
    if ret != 0:
        print("ERROR: samtools not found or failed", file=sys.stderr)
        sys.exit(1)
    os.remove(sam_path)
    print(f"Written {bam_path} (sorted + indexed)")

    print()
    print("Expected .seq.DupRate.xls:")
    print("Occurrence\tUniqReadNumber")
    print("1\t2")
    print("2\t1")
    print("3\t1")
    print()
    print("Expected .pos.DupRate.xls:")
    print("Occurrence\tUniqReadNumber")
    print("1\t2")
    print("2\t1")
    print("3\t1")
