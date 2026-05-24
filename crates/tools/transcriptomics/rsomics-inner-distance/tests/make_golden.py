#!/usr/bin/env python3
"""Generate golden fixtures for rsomics-inner-distance tests.

Gene model (chr1, 0-based):
  gene_A1/A2 (duplicated to work around RSeQC transcript_ranges bug where first
  transcript per chromosome is never added to the lookup table): chr1 1000-4000 (+)
    exons: [1000,1300), [2000,2300), [3000,4000)
    introns: [1300,2000)=700bp, [2300,3000)=700bp

  gene_B1/B2: chr1 5000-8000 (-)
    exons: [5000,5400), [6000,6400), [7000,8000)
    introns: [5400,6000)=600bp, [6400,7000)=600bp

  gene_C1/C2 (intergenic anchor — no use, only to keep C records in same-transcript):
    chr1 10000-11000 (+), single exon

Paired-end read pairs (coordinate-sorted, all read1 ≤ read2):

Pair 1: read1 pos=1001 (SAM 1-based) = 1000 (0-based), CIGAR 100M
         qlen=100, intron=0, read1_end=1100
         mate pos=1151 (0-based: 1150)
         read2_start=1150 >= read1_end=1100 → positive gap
         inner_distance = 1150 - 1100 = 50
         read1_end-1=1099 in gene_A2 [1000,4000), read2_start=1150 in gene_A2 → same transcript
         exon_bases in [1100,1150): fully in exon [1000,1300) → 50 bases
         50 == inner_distance → sameTranscript=Yes,sameExon=Yes,dist=mRNA, written distance=50

Pair 2: read1 pos=1001 CIGAR 100M700N100M (spliced across [1300,2000) intron)
         qlen=200, intron=700, read1_end = 1000 + 200 + 700 = 1900
         mate pos=2051 (0-based: 2050)
         inner_distance = 2050 - 1900 = 150
         read1_end-1=1899 in gene_A2 [1000,4000) ✓
         read2_start=2050 in gene_A2 [1000,4000) ✓ → same transcript
         exon_bases in [1900,2050): overlap with exon [2000,2300): [2000,2050) = 50 bases
         50 > 0 and 50 < 150 → sameTranscript=Yes,sameExon=No,dist=mRNA, distance=50

Pair 3: read1 pos=2001 CIGAR 200M (in gene_A2 exon 2)
         qlen=200, intron=0, read1_end=2200
         mate pos=2101 (0-based: 2100), read2_start=2100 < read1_end=2200 → overlap
         exon_bases in (2100,2200]: 0-based (2100,2200]: [2000,2300)∩(2100,2200] = 100 bases
         inner_distance = -100
         read1_end-1=2199 in gene_A2, read2_start=2100 in gene_A2 → same transcript
         → readPairOverlap, written distance=-100

Pair 4: read1 pos=10001 CIGAR 100M (in intergenic region not in gene_A/B)
         qlen=100, read1_end=10100
         mate pos=10201 (0-based: 10200)
         inner_distance = 10200 - 10100 = 100
         read1_end-1=10099: in no transcript → sameTranscript=No,dist=genomic, distance=100

Expected histogram (default -l -250 -u 250 -s 5):
  d=50  (pair1 mRNA written as 50):  bin [45,50)  (50>45 and 50<=50) → count 1
  d=50  (pair2 mRNA written as 50):  bin [45,50)  → count 1  → total [45,50) = 2
  d=-100 (pair3 overlap):            bin [-105,-100) (-100>-105 and -100<=-100) → count 1
  d=100  (pair4 intergenic):         bin [95,100)  (100>95 and 100<=100) → count 1
"""
import os
import sys

PAIRED = 0x1
PROPER_PAIR = 0x2
MATE_REVERSE = 0x20
READ1 = 0x40
READ2 = 0x80

MAPQ = 60
SEQ_100 = "A" * 100
QUAL_100 = "I" * 100
SEQ_200 = "A" * 200
QUAL_200 = "I" * 200


def make_bed12():
    """Two transcripts per chromosome so RSeQC's if/else bug adds the second one."""
    lines = [
        # gene_A1: first on chr1 — added to Intersecter (created but NOT inserted due to bug)
        "chr1\t1000\t4000\tgene_A1\t0\t+\t1000\t4000\t0\t3\t300,300,1000,\t0,1000,2000,",
        # gene_A2: second on chr1 — correctly added by the else branch
        "chr1\t1000\t4000\tgene_A2\t0\t+\t1000\t4000\t0\t3\t300,300,1000,\t0,1000,2000,",
        # gene_B1: first - strand entry (not used by pairs, padding)
        "chr1\t5000\t8000\tgene_B1\t0\t-\t5000\t8000\t0\t3\t400,400,1000,\t0,1000,2000,",
        # gene_B2: second - strand entry
        "chr1\t5000\t8000\tgene_B2\t0\t-\t5000\t8000\t0\t3\t400,400,1000,\t0,1000,2000,",
    ]
    return "\n".join(lines) + "\n"


def make_read(name, flag, chrom, pos, cigar, seq, qual, mpos, mapq=MAPQ):
    """Create a SAM record (pos/mpos are 1-based SAM positions)."""
    return f"{name}\t{flag}\t{chrom}\t{pos}\t{mapq}\t{cigar}\t=\t{mpos}\t0\t{seq}\t{qual}"


def build_sam_records():
    records = []

    # Pair 1: read1 at 1001, CIGAR 100M, mate at 1151
    # read1_start=1000, read1_end=1100, mate_start=1150 → gap=50
    # same transcript (gene_A2) → exon_bases([1100,1150))=50=gap → sameExon=Yes, mRNA dist=50
    f1 = PAIRED | PROPER_PAIR | READ1
    f1m = PAIRED | PROPER_PAIR | MATE_REVERSE | READ2
    records.append(make_read("pair1", f1, "chr1", 1001, "100M", SEQ_100, QUAL_100, 1151))
    records.append(make_read("pair1", f1m, "chr1", 1151, "100M", SEQ_100, QUAL_100, 1001))

    # Pair 2: read1 spliced at 1001, CIGAR 100M700N100M, mate at 2051
    # qlen=200, intron=700, read1_end=1900, mate_start=2050 → gap=150
    # exon_bases([1900,2050))=50 (from exon [2000,2300)) → sameExon=No, mRNA dist=50
    f2 = PAIRED | PROPER_PAIR | READ1
    f2m = PAIRED | PROPER_PAIR | MATE_REVERSE | READ2
    records.append(make_read("pair2", f2, "chr1", 1001, "100M700N100M", SEQ_200, QUAL_200, 2051))
    records.append(make_read("pair2", f2m, "chr1", 2051, "100M", SEQ_100, QUAL_100, 1001))

    # Pair 3: overlap — read1 at 2001, CIGAR 200M, mate at 2101
    # read1_start=2000, qlen=200, read1_end=2200, mate_start=2100 < read1_end → overlap
    # exon_bases in (2100,2200]: 100 → inner_distance=-100
    # same transcript (gene_A2) → readPairOverlap
    f3 = PAIRED | PROPER_PAIR | READ1
    f3m = PAIRED | PROPER_PAIR | MATE_REVERSE | READ2
    records.append(make_read("pair3", f3, "chr1", 2001, "200M", SEQ_200, QUAL_200, 2101))
    records.append(make_read("pair3", f3m, "chr1", 2101, "200M", SEQ_200, QUAL_200, 2001))

    # Pair 4: intergenic — read1 at 10001, CIGAR 100M, mate at 10201
    # read1_start=10000, read1_end=10100, mate_start=10200 → gap=100
    # no transcript → sameTranscript=No,dist=genomic, distance=100
    f4 = PAIRED | PROPER_PAIR | READ1
    f4m = PAIRED | PROPER_PAIR | MATE_REVERSE | READ2
    records.append(make_read("pair4", f4, "chr1", 10001, "100M", SEQ_100, QUAL_100, 10201))
    records.append(make_read("pair4", f4m, "chr1", 10201, "100M", SEQ_100, QUAL_100, 10001))

    return records


def write_sam(path, header, records):
    with open(path, "w") as f:
        f.write(header)
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

    header = "@HD\tVN:1.6\tSO:coordinate\n@SQ\tSN:chr1\tLN:200000\n"
    sam_path = os.path.join(golden, "pairs.sam")
    bam_path = os.path.join(golden, "pairs.bam")
    records = build_sam_records()
    write_sam(sam_path, header, records)
    print(f"Written {sam_path} ({len(records)} SAM records = {len(records)//2} pairs)")

    ret = os.system(
        f"samtools sort -o {bam_path} {sam_path} && samtools index {bam_path}"
    )
    if ret != 0:
        print("ERROR: samtools not found or failed", file=sys.stderr)
        sys.exit(1)
    os.remove(sam_path)
    print(f"Written {bam_path} (sorted + indexed)")

    print()
    print("Expected output (inner_distance.txt):")
    print("  pair1  50  sameTranscript=Yes,sameExon=Yes,dist=mRNA")
    print("  pair2  50  sameTranscript=Yes,sameExon=No,dist=mRNA")
    print("  pair3  -100  readPairOverlap")
    print("  pair4  100  sameTranscript=No,dist=genomic")
    print()
    print("Expected histogram (non-zero bins):")
    print("  -105  -100  1  (pair3 d=-100)")
    print("   45    50   2  (pair1+pair2 d=50)")
    print("   95   100   1  (pair4 d=100)")
