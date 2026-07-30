#!/usr/bin/env python3
"""Linux benchmark gate for the rsomics-bed pilot.

The generated workload is deliberately unlike the legacy sparse/no-hit
benchmark: it spans ten chromosomes, merge has overlapping groups, intersect
has a hit for every A record plus repeated B intervals, subtract emits two
fragments per A record, and complement emits real gaps.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import statistics
import subprocess
from pathlib import Path


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def output(command: list[str]) -> str:
    return subprocess.run(
        command, check=True, capture_output=True, text=True
    ).stdout.strip()


def generate_fixtures(directory: Path, count: int) -> dict[str, Path]:
    if count < 100 or count % 10:
        raise ValueError("--records must be at least 100 and divisible by ten")
    directory.mkdir(parents=True, exist_ok=True)
    paths = {
        name: directory / filename
        for name, filename in {
            "unsorted": "unsorted.bed",
            "merge": "merge.bed",
            "a": "a.bed",
            "b": "b.bed",
            "genome": "genome.tsv",
        }.items()
    }
    per_chromosome = count // 10
    with (
        paths["unsorted"].open("wb") as unsorted,
        paths["merge"].open("wb") as merge,
        paths["a"].open("wb") as a_file,
        paths["b"].open("wb") as b_file,
        paths["genome"].open("wb") as genome,
    ):
        for chromosome_index in range(1, 11):
            chromosome = f"chr{chromosome_index:02d}"
            size = per_chromosome * 100 + 200
            genome.write(f"{chromosome}\t{size}\n".encode())
            for index in range(per_chromosome):
                reverse = per_chromosome - index - 1
                unsorted.write(
                    f"{chromosome}\t{reverse * 100 + 5}\t"
                    f"{reverse * 100 + 23}\tU{chromosome_index}-{reverse}\n".encode()
                )

                group = index // 5
                member = index % 5
                merge_start = group * 100 + member * 10 + 1
                merge.write(
                    f"{chromosome}\t{merge_start}\t{merge_start + 15}\t"
                    f"M{chromosome_index}-{index}\n".encode()
                )

                start = index * 100 + 5
                a_file.write(
                    f"{chromosome}\t{start}\t{start + 18}\t"
                    f"A{chromosome_index}-{index}\n".encode()
                )
                b_start = index * 100 + 10
                b_record = (
                    f"{chromosome}\t{b_start}\t{b_start + 6}\t"
                    f"B{chromosome_index}-{index}\n"
                ).encode()
                b_file.write(b_record)
                if index % 50 == 0:
                    b_file.write(b_record)
    return paths


def timed(command: list[str], cores: str) -> tuple[float, int]:
    result = subprocess.run(
        [
            "/usr/bin/time",
            "-f",
            "__RSOMICS_METRIC__%e\t%M",
            "taskset",
            "-c",
            cores,
            *command,
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if result.returncode:
        raise RuntimeError(
            f"command failed ({result.returncode}): {command}\n{result.stderr}"
        )
    metric = next(
        (
            line.removeprefix("__RSOMICS_METRIC__")
            for line in result.stderr.splitlines()
            if line.startswith("__RSOMICS_METRIC__")
        ),
        None,
    )
    if metric is None:
        raise RuntimeError(f"GNU time metric missing for {command}: {result.stderr}")
    elapsed, rss = metric.split("\t")
    return float(elapsed), int(rss)


def benchmark(command: list[str], cores: str, repetitions: int) -> dict[str, object]:
    timed(command, cores)
    samples = [timed(command, cores) for _ in range(repetitions)]
    elapsed = [sample[0] for sample in samples]
    rss = [sample[1] for sample in samples]
    return {
        "command": command,
        "elapsed_seconds": elapsed,
        "elapsed_mean": statistics.mean(elapsed),
        "elapsed_stdev": statistics.stdev(elapsed) if len(elapsed) > 1 else 0.0,
        "max_rss_kib": rss,
        "max_rss_median_kib": statistics.median(rss),
    }


def stream_digest(command: list[str]) -> dict[str, object]:
    digest = hashlib.sha256()
    byte_count = 0
    line_count = 0
    process = subprocess.Popen(
        command,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    assert process.stdout is not None
    for chunk in iter(lambda: process.stdout.read(1024 * 1024), b""):
        digest.update(chunk)
        byte_count += len(chunk)
        line_count += chunk.count(b"\n")
    _, stderr = process.communicate()
    if process.returncode:
        raise RuntimeError(
            f"correctness command failed ({process.returncode}): {command}\n"
            f"{stderr.decode(errors='replace')}"
        )
    return {
        "sha256": digest.hexdigest(),
        "bytes": byte_count,
        "lines": line_count,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rsomics-bed", type=Path, required=True)
    parser.add_argument("--bedtools", type=Path, required=True)
    parser.add_argument("--workdir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--records", type=int, default=100_000)
    parser.add_argument("--repetitions", type=int, default=10)
    parser.add_argument("--cores", default="48-51")
    args = parser.parse_args()

    if output([str(args.bedtools), "--version"]) != "bedtools v2.31.1":
        parser.error("--bedtools must identify itself exactly as bedtools v2.31.1")
    if args.repetitions < 2:
        parser.error("--repetitions must be at least two")

    fixtures = generate_fixtures(args.workdir / "fixtures", args.records)
    ours = str(args.rsomics_bed)
    oracle = str(args.bedtools)
    commands = {
        "sort": {
            "rsomics": [ours, "sort", str(fixtures["unsorted"])],
            "bedtools": [oracle, "sort", "-i", str(fixtures["unsorted"])],
        },
        "merge": {
            "rsomics": [ours, "merge", str(fixtures["merge"])],
            "bedtools": [oracle, "merge", "-i", str(fixtures["merge"])],
        },
        "intersect": {
            "rsomics": [
                ours,
                "intersect",
                "-a",
                str(fixtures["a"]),
                "-b",
                str(fixtures["b"]),
            ],
            "bedtools": [
                oracle,
                "intersect",
                "-a",
                str(fixtures["a"]),
                "-b",
                str(fixtures["b"]),
            ],
        },
        "subtract": {
            "rsomics": [
                ours,
                "subtract",
                "-a",
                str(fixtures["a"]),
                "-b",
                str(fixtures["b"]),
            ],
            "bedtools": [
                oracle,
                "subtract",
                "-a",
                str(fixtures["a"]),
                "-b",
                str(fixtures["b"]),
            ],
        },
        "complement": {
            "rsomics": [
                ours,
                "complement",
                str(fixtures["merge"]),
                "-g",
                str(fixtures["genome"]),
            ],
            "bedtools": [
                oracle,
                "complement",
                "-i",
                str(fixtures["merge"]),
                "-g",
                str(fixtures["genome"]),
            ],
        },
    }
    correctness = {
        operation: {
            implementation: stream_digest(command)
            for implementation, command in implementations.items()
        }
        for operation, implementations in commands.items()
    }
    for operation, results in correctness.items():
        if results["rsomics"] != results["bedtools"]:
            raise RuntimeError(
                f"{operation} output differs from bedtools 2.31.1: {results}"
            )
    measurements = {
        operation: {
            implementation: benchmark(command, args.cores, args.repetitions)
            for implementation, command in implementations.items()
        }
        for operation, implementations in commands.items()
    }
    report = {
        "schema": 1,
        "host": platform.node(),
        "platform": platform.platform(),
        "machine": platform.machine(),
        "cores": args.cores,
        "records": args.records,
        "repetitions": args.repetitions,
        "rsomics_bed_sha256": sha256(args.rsomics_bed),
        "bedtools_version": output([oracle, "--version"]),
        "bedtools_sha256": sha256(args.bedtools),
        "fixtures": {
            name: {
                "path": str(path),
                "sha256": sha256(path),
                "bytes": path.stat().st_size,
            }
            for name, path in fixtures.items()
        },
        "correctness": correctness,
        "measurements": measurements,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
