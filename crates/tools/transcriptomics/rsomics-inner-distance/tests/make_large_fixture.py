#!/usr/bin/env python3
"""Generate a large synthetic fixture for perf benchmarking.

Creates a BAM with ~100k pairs spanning a multi-gene BED12.
Gene model: 50 genes on chr1, each ~20kb with 5 exons.
Pairs: mix of same-exon, same-transcript, and intergenic.
"""
import os
import random
import subprocess
import sys
import tempfile

random.seed(42)

PAIRED = 0x1
PROPER_PAIR = 0x2
MATE_REVERSE = 0x20
READ1 = 0x40
READ2 = 0x80
MAPQ = 60

N_GENES = 50
GENE_SPACING = 30000
EXONS_PER_GENE = 5
EXON_LEN = 500
INTRON_LEN = 3000
READ_LEN = 100
N_PAIRS = 100_000
CHROM_LEN = N_GENES * (EXONS_PER_GENE * (EXON_LEN + INTRON_LEN)) + GENE_SPACING * 2


def build_genes():
    """Return list of (tx_start, tx_end, [(exon_start, exon_end), ...])."""
    genes = []
    pos = GENE_SPACING
    for _ in range(N_GENES):
        exons = []
        for e in range(EXONS_PER_GENE):
            es = pos + e * (EXON_LEN + INTRON_LEN)
            ee = es + EXON_LEN
            exons.append((es, ee))
        tx_start = exons[0][0]
        tx_end = exons[-1][1]
        genes.append((tx_start, tx_end, exons))
        pos = tx_end + INTRON_LEN + GENE_SPACING
    return genes


def make_bed12(genes):
    lines = []
    for i, (tx_start, tx_end, exons) in enumerate(genes):
        n_exons = len(exons)
        block_sizes = ",".join(str(e - s) for s, e in exons) + ","
        block_starts = ",".join(str(s - tx_start) for s, e in exons) + ","
        # Two transcripts per gene so RSeQC adds the second one (if/else bug)
        for suffix in ("A", "B"):
            name = f"gene_{i}_{suffix}"
            lines.append(
                f"chr1\t{tx_start}\t{tx_end}\t{name}\t0\t+\t{tx_start}\t{tx_end}\t0"
                f"\t{n_exons}\t{block_sizes}\t{block_starts}"
            )
    return "\n".join(lines) + "\n"


def pick_exon_position(exons):
    """Pick a random position within one of the exons such that read fits."""
    ex = random.choice(exons)
    max_start = ex[1] - READ_LEN
    if max_start < ex[0]:
        return ex[0]
    return random.randint(ex[0], max_start)


def build_sam_records(genes):
    records = []
    for pair_i in range(N_PAIRS):
        name = f"r{pair_i}"
        r = random.random()
        if r < 0.4:
            # same-exon pair
            gene = random.choice(genes)
            exons = gene[2]
            ex = random.choice(exons)
            r1_start = pick_exon_position([ex])
            r1_end = r1_start + READ_LEN
            max_gap = ex[1] - r1_end - READ_LEN
            if max_gap >= 10:
                gap = random.randint(10, min(100, max_gap))
            else:
                gap = 0
            r2_start = r1_end + gap
            if r2_start + READ_LEN > ex[1]:
                r2_start = r1_end  # overlap fallback
        elif r < 0.7:
            # different-exon same-transcript
            gene = random.choice(genes)
            exons = gene[2]
            if len(exons) < 2:
                continue
            ex1, ex2 = exons[0], exons[1]
            r1_start = pick_exon_position([ex1])
            r2_start = pick_exon_position([ex2])
            if r2_start < r1_start + READ_LEN:
                r2_start = ex2[0]
        elif r < 0.85:
            # overlap
            gene = random.choice(genes)
            exons = gene[2]
            ex = random.choice(exons)
            r1_start = pick_exon_position([ex])
            r2_start = r1_start + random.randint(10, READ_LEN - 1)
        else:
            # intergenic (after last gene)
            base = genes[-1][1] + GENE_SPACING
            r1_start = base + pair_i * 300
            r2_start = r1_start + READ_LEN + 100

        # SAM is 1-based
        p1 = r1_start + 1
        p2 = r2_start + 1
        seq = "A" * READ_LEN
        qual = "I" * READ_LEN
        f1 = PAIRED | PROPER_PAIR | READ1
        f2 = PAIRED | PROPER_PAIR | MATE_REVERSE | READ2
        records.append(f"{name}\t{f1}\tchr1\t{p1}\t{MAPQ}\t{READ_LEN}M\t=\t{p2}\t0\t{seq}\t{qual}")
        records.append(f"{name}\t{f2}\tchr1\t{p2}\t{MAPQ}\t{READ_LEN}M\t=\t{p1}\t0\t{seq}\t{qual}")

    return records


if __name__ == "__main__":
    out_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "large_fixture")
    os.makedirs(out_dir, exist_ok=True)

    genes = build_genes()

    bed_path = os.path.join(out_dir, "genes_large.bed12")
    with open(bed_path, "w") as f:
        f.write(make_bed12(genes))
    print(f"Written {bed_path} ({N_GENES * 2} transcripts)")

    header = f"@HD\tVN:1.6\tSO:coordinate\n@SQ\tSN:chr1\tLN:{CHROM_LEN}\n"
    records = build_sam_records(genes)
    # Sort by coordinate
    records.sort(key=lambda r: int(r.split("\t")[3]))

    with tempfile.NamedTemporaryFile(mode="w", suffix=".sam", delete=False) as tmp:
        tmp.write(header)
        for r in records:
            tmp.write(r + "\n")
        sam_path = tmp.name

    bam_path = os.path.join(out_dir, "pairs_large.bam")
    ret = os.system(
        f"samtools sort -o {bam_path} {sam_path} && samtools index {bam_path}"
    )
    os.unlink(sam_path)
    if ret != 0:
        print("ERROR: samtools failed", file=sys.stderr)
        sys.exit(1)

    size = os.path.getsize(bam_path)
    print(f"Written {bam_path} ({len(records)//2} pairs, {size//1024}KB)")
