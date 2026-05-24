#!/usr/bin/env python3
"""Generate golden fixtures for rsomics-bam-strandedness tests.

Produces:
  tests/golden/fwd_pe.bam   -- paired-end forward-stranded reads
  tests/golden/genes.bed12  -- two + strand genes and one - strand gene

Forward-stranded (1++,1--,2+-,2-+) protocol means:
  read1 on + strand → gene on + strand
  read1 on - strand → gene on - strand
  read2 on + strand → gene on - strand
  read2 on - strand → gene on + strand

We generate 10 paired-end reads, all from gene+ and gene-minus with
matching strand assignments so the expected fractions are deterministic.

Design:
  - gene_A: chr1:1000-2000 strand=+ (forward gene)
  - gene_B: chr1:3000-4000 strand=- (reverse gene)
  - gene_C: chr1:5000-6000 strand=+ (forward gene)

  - 6 read pairs from gene_A / gene_C (+ strand genes), forward protocol:
      read1 maps + strand, read2 maps - strand
      → p_strandness["1++"] += 6, p_strandness["2+-"] += 6 (wait, check)

Actually for forward-stranded, read1 + strand → gene +, read2 - strand → gene +.
Keys: read1 maps +, gene +  → "1++"
      read2 maps -, gene +  → "2-+"
These are BOTH in spec1 (1++,1--,2+-,2-+).

  - 4 read pairs from gene_B (- strand gene), forward protocol:
      read1 maps - strand → gene -  → "1--"  (in spec1)
      read2 maps + strand → gene -  → "2+-"  (in spec1)

So all 20 reads (10 pairs × 2 reads) fall into spec1.
spec1 = 1.0, spec2 = 0.0, other = 0.0.

This is easily verifiable.
"""
import struct
import sys
import os

# --- SAM/BAM constants ---
# Flag bits
PAIRED = 0x1
PROPER_PAIR = 0x2
MATE_REVERSE = 0x20
READ1 = 0x40
READ2 = 0x80
REVERSE = 0x10

def make_bed12():
    lines = [
        "chr1\t1000\t2000\tgene_A\t0\t+\t1000\t2000\t0\t1\t1000,\t0,",
        "chr1\t3000\t4000\tgene_B\t0\t-\t3000\t4000\t0\t1\t1000,\t0,",
        "chr1\t5000\t6000\tgene_C\t0\t+\t5000\t6000\t0\t1\t1000,\t0,",
    ]
    return "\n".join(lines) + "\n"


def build_sam_reads():
    """Return list of SAM records as strings (no header)."""
    records = []

    SEQ = "ACGTACGTACGTACGTACGTACGT"  # 24 bp
    QUAL = "I" * len(SEQ)
    MAPQ = "60"
    CIGAR = f"{len(SEQ)}M"

    # 6 pairs from + strand genes (gene_A or gene_C), forward protocol:
    # read1: + strand → gene_A/C (+)  → flag PAIRED|READ1|PROPER_PAIR|MATE_REVERSE
    # read2: - strand → gene_A/C (+)  → flag PAIRED|READ2|PROPER_PAIR|REVERSE
    for i in range(6):
        gene = "gene_A" if i < 3 else "gene_C"
        pos = 1001 + i * 10 if i < 3 else 5001 + (i - 3) * 10
        mate_pos = pos + 50

        # read1, + strand
        f1 = PAIRED | PROPER_PAIR | MATE_REVERSE | READ1
        records.append(
            f"read{i+1}_1\t{f1}\tchr1\t{pos}\t60\t{CIGAR}\t=\t{mate_pos}\t100\t{SEQ}\t{QUAL}\tNH:i:1"
        )
        # read2, - strand
        f2 = PAIRED | PROPER_PAIR | REVERSE | READ2
        records.append(
            f"read{i+1}_2\t{f2}\tchr1\t{mate_pos}\t60\t{CIGAR}\t=\t{pos}\t-100\t{SEQ}\t{QUAL}\tNH:i:1"
        )

    # 4 pairs from - strand gene (gene_B), forward protocol:
    # read1: - strand → gene_B (-)  → flag PAIRED|READ1|PROPER_PAIR|REVERSE|MATE_UNSET
    # read2: + strand → gene_B (-)  → flag PAIRED|READ2|PROPER_PAIR|MATE_REVERSE
    for i in range(4):
        pos = 3001 + i * 10
        mate_pos = pos + 50
        # read1, - strand
        f1 = PAIRED | PROPER_PAIR | REVERSE | READ1
        records.append(
            f"read{i+7}_1\t{f1}\tchr1\t{pos}\t60\t{CIGAR}\t=\t{mate_pos}\t100\t{SEQ}\t{QUAL}\tNH:i:1"
        )
        # read2, + strand
        f2 = PAIRED | PROPER_PAIR | MATE_REVERSE | READ2
        records.append(
            f"read{i+7}_2\t{f2}\tchr1\t{mate_pos}\t60\t{CIGAR}\t=\t{pos}\t-100\t{SEQ}\t{QUAL}\tNH:i:1"
        )

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

    sam_path = os.path.join(golden, "fwd_pe.sam")
    bam_path = os.path.join(golden, "fwd_pe.bam")
    records = build_sam_reads()
    write_sam(sam_path, records)
    print(f"Written {sam_path} ({len(records)} records)")

    # Convert SAM → sorted BAM
    ret = os.system(f"samtools sort -o {bam_path} {sam_path} && samtools index {bam_path}")
    if ret != 0:
        print("ERROR: samtools not found or failed", file=sys.stderr)
        sys.exit(1)
    os.remove(sam_path)
    print(f"Written {bam_path} (sorted + indexed)")
    print()
    print("Expected output for fwd_pe.bam:")
    print("This is PairEnd Data")
    print("Fraction of reads failed to determine: 0.0000")
    print('Fraction of reads explained by "1++,1--,2+-,2-+": 1.0000')
    print('Fraction of reads explained by "1+-,1-+,2++,2--": 0.0000')
