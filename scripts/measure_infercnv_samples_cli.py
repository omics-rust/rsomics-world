#!/usr/bin/env python3
"""Measure raw-to-samples CLI lifetime and worker scaling on Linux fixtures."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess

from infercnv_trial_process import run_child
from measure_infercnv import TIME_FORMAT, _parse_time_file
from validate_infercnv_shipped_oracle import file_sha256


REFERENCES = ("Microglia/Macrophage", "Oligodendrocytes (non-malignant)")
METRICS = ("wall_seconds", "user_seconds", "system_seconds",
           "maximum_rss_bytes", "owner_wall_seconds")


def report_fingerprint(report, threads):
    settings = dict(report["settings"])
    if settings.pop("threads") != threads:
        raise ValueError("report worker count differs from command")
    normalized = dict(report, settings=settings)
    data = json.dumps(normalized, sort_keys=True, separators=(",", ":"), allow_nan=False)
    return hashlib.sha256(data.encode()).hexdigest()


def validate_report(report, case="canonical"):
    if report["schema"] != "rsomics-sc.cnv.samples.v1":
        raise ValueError("unexpected samples report schema")
    if case == "canonical":
        if (len(report["genes"]) != 8508 or len(report["cells"]) != 184
                or len(report["selected_gene_indices"]) != 6187
                or len(report["outlier_gene_indices"]) != 2321):
            raise ValueError("report differs from the canonical input/selection contract")
        expected = [("all_observations", 142), (REFERENCES[0], 19), (REFERENCES[1], 23)]
        cells = 184
    elif case == "gbm-4103":
        genes, cells = len(report["genes"]), 4103
        if (genes == 0 or len(report["cells"]) != cells
                or report["outlier_gene_indices"] is not None
                or report["selected_gene_indices"] != list(range(genes))):
            raise ValueError("report differs from the unclassified GBM scaling contract")
        expected = [("all_observations", cells)]
    else:
        raise ValueError("unknown fixture case")
    groups = report["groups"]
    if [(g["name"], len(g["member_indices"])) for g in groups] != expected:
        raise ValueError("report group names/cardinalities differ")
    if sorted(i for g in groups for i in g["member_indices"]) != list(range(cells)):
        raise ValueError("report groups do not cover cells exactly once")
    for group, (_, count) in zip(groups, expected):
        if (len(group["tree"]["merge"]) != count - 1
                or sorted(group["tree"]["order"]) != list(range(count))):
            raise ValueError("report tree dimensions/order differ")


def summarize(trials):
    measured = [t for t in trials if t["phase"] == "measured"]
    by_worker = {n: [t for t in measured if t["threads"] == n] for n in (1, 4)}
    if not by_worker[1] or len(by_worker[1]) != len(by_worker[4]):
        raise ValueError("incomplete serial/parallel comparison")
    workers = {}
    for n, own in by_worker.items():
        workers[str(n)] = {}
        for key in METRICS:
            values = [t[key] for t in own]
            workers[str(n)][key] = dict(values=values, median=statistics.median(values),
                                        min=min(values), max=max(values))
    serial = workers["1"]["wall_seconds"]
    parallel = workers["4"]["wall_seconds"]
    return dict(workers=workers,
                serial_to_parallel_median_wall_ratio=serial["median"] / parallel["median"],
                strict_parallel_wall_separation=parallel["max"] < serial["min"])


def read_optional(path):
    path = Path(path)
    return path.read_text().strip() if path.is_file() else None


def run_measurement(bundle, binary, output, repeats=5, case="canonical"):
    bundle, binary = Path(bundle).resolve(strict=True), Path(binary).resolve(strict=True)
    output = Path(output).absolute()
    if platform.system() != "Linux" or not os.access(binary, os.X_OK):
        raise ValueError("an executable native Linux binary is required")
    if repeats < 1:
        raise ValueError("repeats must be positive")
    if case == "canonical":
        inputs = [bundle / "inputs" / name for name in (
            "canonical-full.tsv", "gencode_downsampled.EXAMPLE_ONLY_DONT_REUSE.txt",
            "oligodendroglioma_annotations_downsampled.txt")]
        reference_args = [flag for name in REFERENCES for flag in ("--reference", name)]
        extra_args, cutoff, timeout = [], "1", 300
    elif case == "gbm-4103":
        inputs = [bundle / name for name in ("counts.tsv", "positions.tsv", "annotations.tsv")]
        reference_args, cutoff, timeout = [], "0.1", 1800
        extra_args = ["--max-numeric-bytes", str(4 * 1024**3)]
    else:
        raise ValueError("unknown fixture case")
    input_hashes = {str(path): file_sha256(path) for path in inputs}
    binary_hash = file_sha256(binary)
    output.mkdir(parents=True, exist_ok=False)
    output = output.resolve(strict=True)
    controller = Path(__file__).resolve()
    provenance = dict(
        scope="whole fresh CLI process: plain raw input through default samples JSON report",
        case=case,
        excludes=["compilation", "report validation", "cold I/O", "R comparison",
                  "HMM", "denoise", "plots", "biological CNV accuracy"]
                 + (["larger cohort"] if case == "canonical" else []),
        warmup="one fresh process per worker count; warm OS input/page cache",
        timing="GNU time process lifetime; owner_wall_seconds also includes observer overhead",
        binary=str(binary), binary_sha256=binary_hash, input_sha256=input_hashes,
        controller_sha256=file_sha256(controller), python=platform.python_version(),
        machine=platform.uname()._asdict(), cpu_count=os.cpu_count(),
        cpu_affinity=sorted(os.sched_getaffinity(0)),
        cpu_quota=read_optional("/sys/fs/cgroup/cpu.max"),
        load_before=read_optional("/proc/loadavg"), repeats=repeats,
        version=subprocess.check_output([str(binary), "--version"], text=True).strip(),
        gnu_time=subprocess.check_output(["/usr/bin/time", "--version"], text=True).splitlines()[0],
    )
    (output / "provenance.json").write_text(json.dumps(provenance, indent=2) + "\n")
    order = [("warmup", 0, n) for n in (1, 4)]
    for pair in range(1, repeats + 1):
        order.extend(("measured", pair, n) for n in ((1, 4) if pair % 2 else (4, 1)))
    baseline = None
    trials = []
    with (output / "trials.jsonl").open("x") as journal:
        for phase, pair, threads in order:
            prefix = output / f"{phase}-{pair:02d}-threads-{threads}"
            report_path = prefix.with_suffix(".report.json")
            time_path = prefix.with_suffix(".time.tsv")
            command = [str(binary), "cnv", "samples", "--counts", str(inputs[0]),
                       "--positions", str(inputs[1]), "--annotations", str(inputs[2]),
                       *reference_args, "--cutoff", cutoff, "--threads", str(threads),
                       "--output", str(report_path), *extra_args]
            completed = run_child(["/usr/bin/time", "-f", TIME_FORMAT, "-o", str(time_path),
                                   *command], prefix, timeout=timeout)
            metrics = _parse_time_file(time_path.read_text())
            if metrics["wall_seconds"] <= 0:
                raise ValueError("GNU time wall duration must be positive")
            report = json.loads(report_path.read_text())
            validate_report(report, case)
            fingerprint = report_fingerprint(report, threads)
            if baseline is not None and fingerprint != baseline:
                raise ValueError("CLI output changed between worker counts or repetitions")
            baseline = fingerprint
            trial = dict(phase=phase, pair=pair, threads=threads, argv=command,
                         **metrics, owner_wall_seconds=(completed.end_ns - completed.start_ns) / 1e9,
                         report_sha256=file_sha256(report_path), report_fingerprint=fingerprint)
            trials.append(trial)
            journal.write(json.dumps(trial) + "\n")
            journal.flush()
            print(f"{phase} pair={pair} workers={threads} wall={metrics['wall_seconds']:.3f}s "
                  f"maxRSS={metrics['maximum_rss_bytes']} bytes", flush=True)
    if file_sha256(binary) != binary_hash or any(file_sha256(Path(p)) != h for p, h in input_hashes.items()):
        raise ValueError("binary or input changed during measurement")
    result = dict(schema_version=1, status="complete", **summarize(trials),
                  report_fingerprint=baseline, measured_trials=2 * repeats,
                  load_after=read_optional("/proc/loadavg"),
                  claim=f"{case} warm-input CLI scaling only; no R replacement-speed or CNV-accuracy claim")
    (output / "summary.json").write_text(json.dumps(result, indent=2) + "\n")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--repeats", type=int, default=5)
    parser.add_argument("--case", choices=("canonical", "gbm-4103"), default="canonical")
    args = parser.parse_args()
    result = run_measurement(args.bundle, args.binary, args.output, args.repeats, args.case)
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
