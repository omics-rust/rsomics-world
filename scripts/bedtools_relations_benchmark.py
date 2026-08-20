#!/usr/bin/env python3
"""Representative Linux benchmark gate for rsomics-bed relation operations."""

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


def generate_fixtures(
    directory: Path, records: int, dense_records: int
) -> dict[str, Path]:
    if records < 100 or records % 10:
        raise ValueError("--records must be at least 100 and divisible by ten")
    if dense_records < 100:
        raise ValueError("--dense-records must be at least 100")

    directory.mkdir(parents=True, exist_ok=True)
    fixtures = {
        "cluster": directory / "cluster.bed",
        "a": directory / "relation-a.bed",
        "b": directory / "relation-b.bed",
        "window_b": directory / "window-b.bed",
        "dense_a": directory / "dense-a.bed",
        "dense_b": directory / "dense-b.bed",
    }
    per_chromosome = records // 10
    with (
        fixtures["cluster"].open("w") as cluster,
        fixtures["a"].open("w") as a_file,
        fixtures["b"].open("w") as b_file,
        fixtures["window_b"].open("w") as window_b_file,
    ):
        chromosomes = [f"chr{letter}" for letter in "ABCDEFGHIJ"]
        for chromosome_index, chromosome in enumerate(chromosomes, start=1):
            for index in range(per_chromosome):
                group = index // 5
                member = index % 5
                cluster_start = group * 200 + member * 20 + 1
                strand = "+" if index % 2 == 0 else "-"
                cluster.write(
                    f"{chromosome}\t{cluster_start}\t{cluster_start + 30}\t"
                    f"C{chromosome_index}-{index}\t0\t{strand}\n"
                )

                base = index * 200
                a_file.write(
                    f"{chromosome}\t{base + 40}\t{base + 50}\t"
                    f"A{chromosome_index}-{index}\t0\t{strand}\n"
                )
                b_file.write(
                    f"{chromosome}\t{base + 10}\t{base + 20}\t"
                    f"BL{chromosome_index}-{index}\t0\t+\n"
                )
                window_record = (
                    f"{chromosome}\t{base + 10}\t{base + 20}\t"
                    f"W{chromosome_index}-{index}\t0\t+\n"
                )
                window_b_file.write(window_record)
                if index % 50 == 0:
                    window_b_file.write(window_record)
                if index % 50 == 0:
                    b_file.write(
                        f"{chromosome}\t{base + 44}\t{base + 46}\t"
                        f"BO{chromosome_index}-{index}\t0\t{strand}\n"
                    )
                b_file.write(
                    f"{chromosome}\t{base + 70}\t{base + 80}\t"
                    f"BR{chromosome_index}-{index}\t0\t-\n"
                )

    with (
        fixtures["dense_a"].open("w") as a_file,
        fixtures["dense_b"].open("w") as b_file,
    ):
        for index in range(dense_records):
            a_file.write(f"chr1\t10000\t10001\tA{index}\n")
            b_file.write(f"chr1\t{index}\t{20000 + index}\tB{index}\n")
    return fixtures


def stream_digest(command: list[str]) -> dict[str, object]:
    digest = hashlib.sha256()
    byte_count = 0
    line_count = 0
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    assert process.stdout is not None
    for chunk in iter(lambda: process.stdout.read(1024 * 1024), b""):
        digest.update(chunk)
        byte_count += len(chunk)
        line_count += chunk.count(b"\n")
    _, stderr = process.communicate()
    if process.returncode:
        raise RuntimeError(
            f"command failed ({process.returncode}): {command}\n"
            f"{stderr.decode(errors='replace')}"
        )
    return {"sha256": digest.hexdigest(), "bytes": byte_count, "lines": line_count}


def timed(command: list[str], cores: str) -> dict[str, float | int]:
    result = subprocess.run(
        [
            "/usr/bin/time",
            "-f",
            "__RSOMICS_METRIC__%e\t%U\t%S\t%M",
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
    elapsed, user, system, rss = metric.split("\t")
    return {
        "elapsed_seconds": float(elapsed),
        "user_seconds": float(user),
        "system_seconds": float(system),
        "cpu_seconds": float(user) + float(system),
        "max_rss_kib": int(rss),
    }


def summarize(command: list[str], samples: list[dict[str, float | int]]) -> dict[str, object]:
    elapsed = [float(sample["elapsed_seconds"]) for sample in samples]
    cpu = [float(sample["cpu_seconds"]) for sample in samples]
    rss = [int(sample["max_rss_kib"]) for sample in samples]
    return {
        "command": command,
        "samples": samples,
        "elapsed_mean": statistics.mean(elapsed),
        "elapsed_stdev": statistics.stdev(elapsed),
        "cpu_mean": statistics.mean(cpu),
        "cpu_stdev": statistics.stdev(cpu),
        "max_rss_median_kib": statistics.median(rss),
    }


def paired_benchmark(
    commands: dict[str, list[str]], cores: str, repetitions: int
) -> dict[str, object]:
    timed(commands["rsomics"], cores)
    timed(commands["bedtools"], cores)
    samples: dict[str, list[dict[str, float | int]]] = {
        "rsomics": [],
        "bedtools": [],
    }
    for repetition in range(repetitions):
        order = ("rsomics", "bedtools") if repetition % 2 == 0 else ("bedtools", "rsomics")
        for implementation in order:
            samples[implementation].append(timed(commands[implementation], cores))
    return {
        implementation: summarize(commands[implementation], samples[implementation])
        for implementation in ("rsomics", "bedtools")
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rsomics-bed", type=Path, required=True)
    parser.add_argument("--bedtools", type=Path, required=True)
    parser.add_argument("--workdir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--source-revision", required=True)
    parser.add_argument("--records", type=int, default=1_000_000)
    parser.add_argument("--dense-records", type=int, default=5_000)
    parser.add_argument("--repetitions", type=int, default=10)
    parser.add_argument("--cores", default="48-51")
    args = parser.parse_args()

    if output([str(args.bedtools), "--version"]) != "bedtools v2.31.1":
        parser.error("--bedtools must identify itself exactly as bedtools v2.31.1")
    if args.repetitions < 2:
        parser.error("--repetitions must be at least two")

    fixtures = generate_fixtures(args.workdir / "fixtures", args.records, args.dense_records)
    ours = str(args.rsomics_bed)
    oracle = str(args.bedtools)
    commands = {
        "cluster": {
            "rsomics": [ours, "cluster", str(fixtures["cluster"])],
            "bedtools": [oracle, "cluster", "-i", str(fixtures["cluster"])],
        },
        "cluster_same_strand": {
            "rsomics": [ours, "cluster", "--strand", "same", str(fixtures["cluster"])],
            "bedtools": [oracle, "cluster", "-s", "-i", str(fixtures["cluster"])],
        },
        "window_pairs": {
            "rsomics": [
                ours,
                "window",
                "--window",
                "25",
                "-a",
                str(fixtures["a"]),
                "-b",
                str(fixtures["window_b"]),
            ],
            "bedtools": [
                oracle,
                "window",
                "-w",
                "25",
                "-a",
                str(fixtures["a"]),
                "-b",
                str(fixtures["window_b"]),
            ],
        },
        "closest": {
            "rsomics": [ours, "closest", "-a", str(fixtures["a"]), "-b", str(fixtures["b"])],
            "bedtools": [oracle, "closest", "-a", str(fixtures["a"]), "-b", str(fixtures["b"])],
        },
        "closest_distance": {
            "rsomics": [
                ours,
                "closest",
                "--distance",
                "unsigned",
                "-a",
                str(fixtures["a"]),
                "-b",
                str(fixtures["b"]),
            ],
            "bedtools": [
                oracle,
                "closest",
                "-d",
                "-a",
                str(fixtures["a"]),
                "-b",
                str(fixtures["b"]),
            ],
        },
        "window_dense_count": {
            "rsomics": [
                ours,
                "window",
                "--window",
                "0",
                "--report",
                "count",
                "-a",
                str(fixtures["dense_a"]),
                "-b",
                str(fixtures["dense_b"]),
            ],
            "bedtools": [
                oracle,
                "window",
                "-w",
                "0",
                "-c",
                "-a",
                str(fixtures["dense_a"]),
                "-b",
                str(fixtures["dense_b"]),
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
            raise RuntimeError(f"{operation} output differs from bedtools 2.31.1: {results}")

    measurements = {
        operation: paired_benchmark(implementations, args.cores, args.repetitions)
        for operation, implementations in commands.items()
    }
    report = {
        "schema": 1,
        "host": platform.node(),
        "platform": platform.platform(),
        "machine": platform.machine(),
        "cores": args.cores,
        "records": args.records,
        "dense_records": args.dense_records,
        "repetitions": args.repetitions,
        "source_revision": args.source_revision,
        "rsomics_bed_sha256": sha256(args.rsomics_bed),
        "bedtools_version": output([oracle, "--version"]),
        "bedtools_sha256": sha256(args.bedtools),
        "fixtures": {
            name: {"path": str(path), "sha256": sha256(path), "bytes": path.stat().st_size}
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
