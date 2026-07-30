#!/usr/bin/env python3
"""Seeded differential checks for the rsomics-bed pilot.

The script intentionally uses only the product's supported default behavior.
It compares complete stdout and exit status with a pinned bedtools oracle while
varying ordering, overlap density, duplicate intervals, zero-length intervals,
chromosomes, and trailing BED columns.
"""

from __future__ import annotations

import argparse
import collections
import random
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Record:
    chrom: str
    start: int
    end: int
    name: str

    def encode(self) -> bytes:
        return f"{self.chrom}\t{self.start}\t{self.end}\t{self.name}\n".encode()


def run(
    command: list[str], stdin: bytes | None = None
) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(command, input=stdin, capture_output=True, check=False)


def records_bytes(records: list[Record]) -> bytes:
    return b"".join(record.encode() for record in records)


def compare(
    operation: str,
    ours: list[str],
    oracle: list[str],
    *,
    stdin: bytes | None = None,
    context: str,
) -> None:
    ours_result = run(ours, stdin)
    oracle_result = run(oracle, stdin)
    if (
        ours_result.returncode != oracle_result.returncode
        or ours_result.stdout != oracle_result.stdout
    ):
        raise RuntimeError(
            f"{operation} mismatch ({context})\n"
            f"ours status={ours_result.returncode}\n"
            f"oracle status={oracle_result.returncode}\n"
            f"ours stderr={ours_result.stderr.decode(errors='replace')}\n"
            f"oracle stderr={oracle_result.stderr.decode(errors='replace')}\n"
            f"ours stdout={ours_result.stdout!r}\n"
            f"oracle stdout={oracle_result.stdout!r}"
        )


def compare_sort(
    ours: list[str],
    oracle: list[str],
    *,
    stdin: bytes,
    context: str,
) -> None:
    ours_result = run(ours, stdin)
    oracle_result = run(oracle, stdin)
    ours_lines = ours_result.stdout.splitlines()
    oracle_lines = oracle_result.stdout.splitlines()

    def coordinates(lines: list[bytes]) -> list[tuple[bytes, int]]:
        return [
            (fields[0], int(fields[1]))
            for line in lines
            if (fields := line.split(b"\t"))
        ]

    if (
        ours_result.returncode != oracle_result.returncode
        or coordinates(ours_lines) != coordinates(oracle_lines)
        or collections.Counter(ours_lines) != collections.Counter(oracle_lines)
    ):
        raise RuntimeError(
            f"sort mismatch ({context})\n"
            f"ours status={ours_result.returncode}\n"
            f"oracle status={oracle_result.returncode}\n"
            f"ours stderr={ours_result.stderr.decode(errors='replace')}\n"
            f"oracle stderr={oracle_result.stderr.decode(errors='replace')}\n"
            f"ours stdout={ours_result.stdout!r}\n"
            f"oracle stdout={oracle_result.stdout!r}"
        )


def random_records(
    rng: random.Random,
    count: int,
    prefix: str,
    *,
    allow_zero: bool,
) -> list[Record]:
    chromosomes = ("chr1", "chr2", "chr10")
    records: list[Record] = []
    for index in range(count):
        chrom = rng.choice(chromosomes)
        start = rng.randrange(1, 181)
        if allow_zero and rng.randrange(8) == 0:
            end = start
        else:
            end = min(200, start + rng.randrange(1, 31))
        records.append(Record(chrom, start, end, f"{prefix}{index}"))
        if rng.randrange(12) == 0:
            records.append(Record(chrom, start, end, f"{prefix}{index}-duplicate"))
    return records


def check_sort_and_merge(
    rng: random.Random,
    ours: str,
    bedtools: str,
    trial: int,
) -> None:
    records = random_records(rng, rng.randrange(1, 70), "R", allow_zero=True)
    rng.shuffle(records)
    payload = records_bytes(records)
    compare_sort(
        [ours, "sort", "-"],
        [bedtools, "sort", "-i", "-"],
        stdin=payload,
        context=f"trial={trial}",
    )

    sorted_records = sorted(records, key=lambda record: (record.chrom, record.start))
    payload = records_bytes(sorted_records)
    compare(
        "merge",
        [ours, "merge", "-"],
        [bedtools, "merge", "-i", "-"],
        stdin=payload,
        context=f"trial={trial}",
    )


def check_binary_operations(
    rng: random.Random,
    ours: str,
    bedtools: str,
    directory: Path,
    trial: int,
) -> None:
    a = random_records(rng, rng.randrange(1, 50), "A", allow_zero=True)
    b = random_records(rng, rng.randrange(1, 50), "B", allow_zero=True)
    rng.shuffle(a)
    rng.shuffle(b)
    a_path = directory / "a.bed"
    b_path = directory / "b.bed"
    a_path.write_bytes(records_bytes(a))
    b_path.write_bytes(records_bytes(b))

    compare(
        "intersect",
        [ours, "intersect", "-a", str(a_path), "-b", str(b_path)],
        [bedtools, "intersect", "-a", str(a_path), "-b", str(b_path)],
        context=f"trial={trial}",
    )
    compare(
        "subtract",
        [ours, "subtract", "-a", str(a_path), "-b", str(b_path)],
        [bedtools, "subtract", "-a", str(a_path), "-b", str(b_path)],
        context=f"trial={trial}",
    )


def check_complement(
    rng: random.Random,
    ours: str,
    bedtools: str,
    directory: Path,
    trial: int,
) -> None:
    ranks = {"chr1": 0, "chr2": 1, "chr10": 2}
    records = random_records(rng, rng.randrange(1, 70), "C", allow_zero=True)
    records.sort(key=lambda record: (ranks[record.chrom], record.start))
    input_path = directory / "input.bed"
    genome_path = directory / "genome.tsv"
    input_path.write_bytes(records_bytes(records))
    genome_path.write_bytes(b"chr1\t220\nchr2\t220\nchr10\t220\n")
    compare(
        "complement",
        [ours, "complement", str(input_path), "-g", str(genome_path)],
        [bedtools, "complement", "-i", str(input_path), "-g", str(genome_path)],
        context=f"trial={trial}",
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rsomics-bed", required=True)
    parser.add_argument("--bedtools", required=True)
    parser.add_argument("--trials", type=int, default=200)
    parser.add_argument("--seed", type=int, default=0xBED2311)
    parser.add_argument("--scratch", type=Path, required=True)
    args = parser.parse_args()

    if args.trials < 1:
        parser.error("--trials must be positive")
    args.scratch.mkdir(parents=True, exist_ok=True)

    oracle_version = run([args.bedtools, "--version"])
    if (
        oracle_version.returncode != 0
        or oracle_version.stdout.strip() != b"bedtools v2.31.1"
    ):
        parser.error("--bedtools must identify itself exactly as bedtools v2.31.1")

    rng = random.Random(args.seed)
    with tempfile.TemporaryDirectory(dir=args.scratch) as temporary:
        directory = Path(temporary)
        for trial in range(args.trials):
            check_sort_and_merge(rng, args.rsomics_bed, args.bedtools, trial)
            check_binary_operations(
                rng, args.rsomics_bed, args.bedtools, directory, trial
            )
            check_complement(rng, args.rsomics_bed, args.bedtools, directory, trial)

    print(
        f"PASS trials={args.trials} seed={args.seed} "
        "operations=sort,merge,intersect,subtract,complement"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
