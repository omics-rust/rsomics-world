#!/usr/bin/env python3
import hashlib
import json
import os
import platform
import shlex
import statistics
import subprocess
import tempfile
import time
from pathlib import Path


def make_fixture(path: Path) -> None:
    with path.open("w") as stream:
        stream.write("id,group,value,sample\n")
        for i in range(1_000_000):
            group = i % 100_000
            value = (i * 2_654_435_761) % 1_000_003
            sample = (i * 17) % 100_000
            stream.write(f"{i},group_{group},{value},sample_{sample}\n")


def run_timed(command: list[str]) -> float:
    started = time.perf_counter()
    subprocess.run(command, stdout=subprocess.DEVNULL, check=True)
    return time.perf_counter() - started


def measure(
    ours_command: list[str], upstream_command: list[str], runs: int
) -> tuple[list[float], list[float]]:
    for _ in range(3):
        subprocess.run(ours_command, stdout=subprocess.DEVNULL, check=True)
        subprocess.run(upstream_command, stdout=subprocess.DEVNULL, check=True)

    ours = []
    upstream = []
    for sample in range(runs):
        if sample % 2 == 0:
            ours.append(run_timed(ours_command))
            upstream.append(run_timed(upstream_command))
        else:
            upstream.append(run_timed(upstream_command))
            ours.append(run_timed(ours_command))
    return ours, upstream


def main() -> None:
    binary = os.environ["PERF_BINARY"]
    ours_args = shlex.split(os.environ["PERF_OURS_ARGS"])
    csvtk_operation = os.environ["PERF_CSVTK_OPERATION"]
    csvtk_args = shlex.split(os.environ["PERF_CSVTK_ARGS"])
    baseline_ratio = float(os.environ["PERF_BASELINE_RATIO"])
    runs = int(os.environ["PERF_RUNS"])
    minimum_ratio = baseline_ratio * 0.9

    with tempfile.TemporaryDirectory(prefix="rsomics-perfgate-") as work:
        fixture = Path(work) / "csv-1000000-100000.csv"
        make_fixture(fixture)
        ours_command = [binary, *ours_args, str(fixture)]
        upstream_command = [
            "csvtk",
            csvtk_operation,
            *csvtk_args,
            str(fixture),
        ]
        ours, upstream = measure(ours_command, upstream_command, runs)
        fixture_size = fixture.stat().st_size
        fixture_sha256 = hashlib.sha256(fixture.read_bytes()).hexdigest()

    result = {
        "revision": os.environ.get("GITHUB_SHA", "local"),
        "machine": platform.platform(),
        "logical_cores": os.cpu_count(),
        "fixture_bytes": fixture_size,
        "fixture_sha256": fixture_sha256,
        "ours_command": [binary, *ours_args, "<fixture>"],
        "upstream_command": [
            "csvtk",
            csvtk_operation,
            *csvtk_args,
            "<fixture>",
        ],
        "upstream_version": subprocess.check_output(
            ["csvtk", "version"], text=True
        ).splitlines()[0],
        "ours_seconds": ours,
        "upstream_seconds": upstream,
        "ours_mean": statistics.fmean(ours),
        "upstream_mean": statistics.fmean(upstream),
        "baseline_ratio": baseline_ratio,
        "minimum_ratio": minimum_ratio,
        "runs": runs,
    }
    result["observed_ratio"] = result["upstream_mean"] / result["ours_mean"]
    print(json.dumps(result, indent=2))

    if result["observed_ratio"] <= 1.0:
        raise SystemExit("rsomics no longer outperforms csvtk")
    if result["observed_ratio"] < minimum_ratio:
        raise SystemExit(
            f"performance ratio {result['observed_ratio']:.4f} is below "
            f"the 10% regression floor {minimum_ratio:.4f}"
        )


if __name__ == "__main__":
    main()
