#!/usr/bin/env python3
"""Generate small synthetic BAM + FASTA golden fixtures for compat tests.

Requires: samtools in PATH.
Run from the crate root:
    python3 tests/make_fixtures.py
"""

import os
import subprocess
import sys
import struct
import zlib
import hashlib
import random

GOLDEN = os.path.join(os.path.dirname(__file__), "golden")
os.makedirs(GOLDEN, exist_ok=True)

REF_NAME = "chr1"
REF_LEN = 100
SEED = 42
random.seed(SEED)

BASES = b"ACGT"


def rand_base():
    return BASES[random.randint(0, 3)]


def make_fasta():
    """Write a 100-bp reference to small.fa and index it."""
    seq = bytearray(REF_LEN)
    for i in range(REF_LEN):
        seq[i] = rand_base()
    seq = bytes(seq)

    fa_path = os.path.join(GOLDEN, "small.fa")
    with open(fa_path, "wb") as f:
        f.write(b">" + REF_NAME.encode() + b"\n")
        # 60 chars per line
        for i in range(0, len(seq), 60):
            f.write(seq[i : i + 60] + b"\n")

    subprocess.run(["samtools", "faidx", fa_path], check=True)
    return seq


def nt16(b):
    return {ord("A"): 1, ord("C"): 2, ord("G"): 4, ord("T"): 8}.get(b, 15)


def make_sam(ref_seq):
    """Build a SAM with 30 reads covering positions 0-49 (0-based)."""
    header_lines = [
        f"@HD\tVN:1.6\tSO:coordinate",
        f"@SQ\tSN:{REF_NAME}\tLN:{REF_LEN}",
    ]
    reads = []
    read_len = 50
    n_reads = 30
    for i in range(n_reads):
        pos0 = random.randint(0, 49)
        end0 = min(pos0 + read_len, REF_LEN)
        actual_len = end0 - pos0
        seq = bytearray(actual_len)
        qual = bytearray(actual_len)
        # Mostly match the reference, introduce 1-2 SNPs
        for j in range(actual_len):
            ref_b = ref_seq[pos0 + j]
            if random.random() < 0.05:
                # Random SNP
                alt = BASES[random.randint(0, 3)]
                seq[j] = alt
            else:
                seq[j] = ref_b
            qual[j] = 30 + random.randint(0, 10)  # BQ 30-40

        reads.append(
            f"read{i}\t0\t{REF_NAME}\t{pos0+1}\t60\t{actual_len}M\t*\t0\t0\t"
            + seq.decode()
            + "\t"
            + "".join(chr(q + 33) for q in qual)
        )

    sam_path = os.path.join(GOLDEN, "small.sam")
    with open(sam_path, "w") as f:
        for h in header_lines:
            f.write(h + "\n")
        for r in reads:
            f.write(r + "\n")
    return sam_path


def main():
    print("Generating reference FASTA...")
    ref_seq = make_fasta()

    print("Generating SAM reads...")
    sam_path = make_sam(ref_seq)

    bam_path = os.path.join(GOLDEN, "small.bam")
    print(f"Converting {sam_path} -> {bam_path}...")
    # Sort + convert to BAM + index
    tmp_bam = bam_path + ".tmp.bam"
    subprocess.run(
        ["samtools", "view", "-bS", "-o", tmp_bam, sam_path], check=True
    )
    subprocess.run(
        ["samtools", "sort", "-o", bam_path, tmp_bam], check=True
    )
    subprocess.run(["samtools", "index", bam_path], check=True)
    os.remove(tmp_bam)
    os.remove(sam_path)

    print(f"Done. Files in {GOLDEN}:")
    for f in sorted(os.listdir(GOLDEN)):
        p = os.path.join(GOLDEN, f)
        print(f"  {f}  ({os.path.getsize(p)} bytes)")


if __name__ == "__main__":
    main()
