#!/usr/bin/env python3
"""Deterministic synthetic fixtures for perfgate. Tier-3 (HDD, not git).

    mkfixture.py fasta   OUT n_records seq_len
    mkfixture.py fastq   OUT n_reads read_len     # 3' TruSeq adapter on ~60%
    mkfixture.py fastqgz OUT n_reads read_len     # same bytes, gzip (mtime=0)
    mkfixture.py bed     OUT n_intervals n_chroms  # sorted, overlapping
    mkfixture.py csv     OUT n_records cardinality # repeated categorical keys
    mkfixture.py bam-to-bed OUT n_pairs 100        # paired mixed-CIGAR BAM

Fixed seed → byte-identical across runs, so a fixture's sha256 is a
stable identity recorded by perfgate.
"""
import gzip
import random
import sys

KIND, OUT = sys.argv[1], sys.argv[2]
A, B = int(sys.argv[3]), int(sys.argv[4])
# Optional 5th arg = seed, so a two-input tool gets distinct a/b fixtures.
SEED = int(sys.argv[5]) if len(sys.argv) > 5 else 0x00C0FFEE
random.seed(SEED)
ACGT = b"ACGT"
ADAPTER = b"AGATCGGAAGAGCACACGTCTGAACTCCAGTCAC"

if KIND == "fasta":
    with open(OUT, "wb") as f:
        for i in range(A):
            f.write(f">contig_{i}\n".encode())
            f.write(bytes(ACGT[random.getrandbits(2)] for _ in range(B)) + b"\n")

elif KIND == "fastq":
    with open(OUT, "wb") as f:
        for i in range(A):
            insert = random.randint(B // 3, B)
            seq = bytes(ACGT[random.getrandbits(2)] for _ in range(insert))
            if random.random() < 0.6:
                seq = (seq + ADAPTER)[:B]
            seq = (seq + bytes(ACGT[random.getrandbits(2)]
                               for _ in range(B)))[:B]
            q = bytes(33 + min(40, 20 + random.randint(-8, 15))
                      for _ in range(B))
            f.write(b"@r%d\n%s\n+\n%s\n" % (i, seq, q))

elif KIND == "fastqgz":
    # Byte-identical uncompressed content to `fastq` for the same seed/params,
    # then single-member gzip. mtime=0 keeps the header fixed so the fixture
    # sha256 is a stable identity. level 6 = typical real .fastq.gz.
    buf = bytearray()
    for i in range(A):
        insert = random.randint(B // 3, B)
        seq = bytes(ACGT[random.getrandbits(2)] for _ in range(insert))
        if random.random() < 0.6:
            seq = (seq + ADAPTER)[:B]
        seq = (seq + bytes(ACGT[random.getrandbits(2)]
                           for _ in range(B)))[:B]
        q = bytes(33 + min(40, 20 + random.randint(-8, 15))
                  for _ in range(B))
        buf += b"@r%d\n%s\n+\n%s\n" % (i, seq, q)
    with open(OUT, "wb") as f:
        f.write(gzip.compress(bytes(buf), compresslevel=6, mtime=0))

elif KIND == "bed":
    rows = []
    for _ in range(A):
        c = random.randint(1, B)
        s = random.randint(0, 250_000_000)
        rows.append((f"chr{c}", s, s + random.randint(50, 5000)))
    rows.sort(key=lambda r: (r[0], r[1]))
    with open(OUT, "w") as f:
        f.writelines(f"{c}\t{s}\t{e}\n" for c, s, e in rows)

elif KIND == "csv":
    with open(OUT, "w") as f:
        f.write("id,group,value,sample\n")
        for i in range(A):
            group = i % B
            value = (i * 2_654_435_761) % 1_000_003
            sample = (i * 17) % B
            f.write(f"{i},group_{group},{value},sample_{sample}\n")

elif KIND == "genome":
    # A = n_chroms. Lexicographic chrom order matches the `bed` kind's
    # sort so `bedtools complement -g` accepts the pairing. Size exceeds
    # the bed kind's max coordinate (250M + 5k).
    chroms = sorted(f"chr{i}" for i in range(1, A + 1))
    with open(OUT, "w") as f:
        f.writelines(f"{c}\t300000000\n" for c in chroms)

elif KIND == "bam":
    # A = n_records, B = read_len. Optional SEED (argv[5]) and per-chrom
    # genome length (argv[6], default 300Mbp). A small genome length yields
    # heavy read overlap and bounded per-base output — the right shape for
    # depth/coverage benches; the default is sparse for read-throughput.
    # SAM is streamed straight into `samtools sort` stdin (no in-memory
    # accumulation, no temp file), so multi-million-record fixtures stay flat
    # in memory. Sequences use C-level random.choices; quality is a fixed Q40
    # string (content does not affect read/sort/depth/coverage throughput).
    import subprocess
    n_chroms = 2
    glen = int(sys.argv[6]) if len(sys.argv) > 6 else 300_000_000
    max_pos = max(1, glen - B)
    qual = "I" * B
    cigar = f"{B}M"
    sort = subprocess.Popen(
        ["samtools", "sort", "-O", "bam", "-o", OUT, "-"],
        stdin=subprocess.PIPE,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    w = sort.stdin
    w.write(b"@HD\tVN:1.6\tSO:coordinate\n")
    for c in range(1, n_chroms + 1):
        w.write(f"@SQ\tSN:chr{c}\tLN:{glen}\n".encode())
    buf = []
    for i in range(A):
        chrom = random.randint(1, n_chroms)
        pos = random.randint(1, max_pos)
        seq = "".join(random.choices("ACGT", k=B))
        flag = 99 if random.random() < 0.5 else 0
        buf.append(f"r{i}\t{flag}\tchr{chrom}\t{pos}\t60\t{cigar}\t*\t0\t0\t{seq}\t{qual}\n")
        if len(buf) >= 50000:
            w.write("".join(buf).encode())
            buf.clear()
    if buf:
        w.write("".join(buf).encode())
    w.close()
    if sort.wait() != 0:
        sys.exit("samtools sort failed")
    subprocess.run(["samtools", "index", OUT], check=True,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

elif KIND == "bam-to-bed":
    import subprocess
    if B != 100:
        sys.exit("bam-to-bed read length must be 100")
    view = subprocess.Popen(
        ["samtools", "view", "--no-PG", "-b", "-o", OUT, "-"],
        stdin=subprocess.PIPE,
        stdout=subprocess.DEVNULL,
    )
    w = view.stdin
    w.write(b"@HD\tVN:1.6\tSO:queryname\n")
    for chrom in range(1, 5):
        w.write(f"@SQ\tSN:chr{chrom}\tLN:10000000\n".encode())
    cigars = [
        "100M",
        "5S40M2I20=3X2D30M",
        "30M50N70M",
        "20M5D30M20N50M",
        "10H100M",
        "10S30M5N60M",
        "25=5X3I67M",
        "45M10D55M",
    ]
    seq1 = "ACGT" * 25
    seq2 = "TGCA" * 25
    qual = "I" * 100
    buf = []
    for i in range(A):
        name = f"pair{i:010d}"
        chrom = f"chr{1 + i % 4}"
        pos = (i * 211) % 9_000_000 + 1
        mate_pos = pos + 200
        nm1 = i % 9
        nm2 = (i * 3) % 11
        xi1 = i % 101
        xi2 = (i * 7) % 101
        cigar1 = cigars[i % len(cigars)]
        cigar2 = cigars[(i + 3) % len(cigars)]
        if i % 16 == 0:
            first = f"{name}\t77\t*\t0\t0\t*\t*\t0\t0\t{seq1}\t{qual}\n"
            second = f"{name}\t141\t*\t0\t0\t*\t*\t0\t0\t{seq2}\t{qual}\n"
        elif i % 16 == 1:
            first = (
                f"{name}\t73\t{chrom}\t{pos}\t55\t{cigar1}\t*\t0\t0\t"
                f"{seq1}\t{qual}\tNM:i:{nm1}\tXI:i:{xi1}\n"
            )
            second = f"{name}\t133\t*\t0\t0\t*\t{chrom}\t{pos}\t0\t{seq2}\t{qual}\n"
        else:
            first = (
                f"{name}\t99\t{chrom}\t{pos}\t55\t{cigar1}\t=\t{mate_pos}\t300\t"
                f"{seq1}\t{qual}\tNM:i:{nm1}\tXI:i:{xi1}\n"
            )
            second = (
                f"{name}\t147\t{chrom}\t{mate_pos}\t42\t{cigar2}\t=\t{pos}\t-300\t"
                f"{seq2}\t{qual}\tNM:i:{nm2}\tXI:i:{xi2}\n"
            )
        buf.extend((first, second))
        if len(buf) >= 50_000:
            w.write("".join(buf).encode())
            buf.clear()
    if buf:
        w.write("".join(buf).encode())
    w.close()
    if view.wait() != 0:
        sys.exit("samtools view failed")

elif KIND == "vcf":
    # A = n_variants, B = n_samples. Generates a minimal VCF with random SNPs.
    chroms = [f"chr{c}" for c in range(1, 3)]
    with open(OUT, "w") as f:
        f.write("##fileformat=VCFv4.3\n")
        for c in chroms:
            f.write(f"##contig=<ID={c},length=300000000>\n")
        samples = [f"SAMPLE{s}" for s in range(1, B + 1)]
        f.write("#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT")
        for s in samples:
            f.write(f"\t{s}")
        f.write("\n")
        for i in range(A):
            chrom = chroms[random.randint(0, len(chroms) - 1)]
            pos = random.randint(1, 299_000_000)
            ref = chr(ACGT[random.getrandbits(2)])
            alt = chr(ACGT[random.getrandbits(2)])
            while alt == ref:
                alt = chr(ACGT[random.getrandbits(2)])
            qual = random.randint(1, 99)
            filt = "PASS" if qual > 30 else "LowQual"
            gts = "\t".join(
                f"{random.randint(0,1)}/{random.randint(0,1)}:{random.randint(5,50)}"
                for _ in samples
            )
            f.write(f"{chrom}\t{pos}\t.\t{ref}\t{alt}\t{qual}\t{filt}\t.\tGT:DP\t{gts}\n")

else:
    sys.exit(f"unknown kind {KIND}")
print(f"wrote {OUT}")
