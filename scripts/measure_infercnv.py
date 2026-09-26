#!/usr/bin/env python3
"""Private matched prepared-input inferCNV measurement driver."""

import argparse
import csv
import hashlib
import json
import math
import os
from pathlib import Path
import signal
import statistics
import subprocess
import sys

from validate_infercnv_shipped_oracle import file_sha256, owned_file, validate_bundle


TRUSTED = {
    "schema_version": 1,
    "accepted_oracle_receipt_sha256": "be0342e7c3f701e98b2f4db5199230de542a6f9bbfb6ea3379c1958068094476",
    "source_commit": "b421d9405c97a309b081ef86d455e976df93eae4",
    "case": "full", "incoming_genes": 9939, "retained_genes": 8508, "cells": 184,
    "config": {"cutoff": 1, "min_cells_per_gene": 3, "window_length": 101,
               "max_centered_threshold": 3,
               "reference_groups": ["Microglia/Macrophage", "Oligodendrocytes (non-malignant)"],
               "num_threads": 1},
    "files": {
        "full/01.tsv": "e8c488e7cbc21307ac8b39ba26dab43e2a04d7f2fdf3f158e244a65afa4a654b",
        "full/01.genes.tsv": "8aad9e524e1be3f2538b01b11a0dc81dfac34e8b27690d64b2232c3944e7b2da",
        "full/01.cells.tsv": "36ffde7c1e10dd025a6e2bedab785c9d1bc32f2c02dfd0570db4e7a8d1300e03",
        "full/14.tsv": "9c1c54dc19febd6997e3b31b1352686315ba76ab3f09e194c622f2f116140320",
        "full/14.genes.tsv": "06bccb684392d48d81068f2fb2b157d46c497b079b42774497fef2595a06d529",
        "full/14.cells.tsv": "36ffde7c1e10dd025a6e2bedab785c9d1bc32f2c02dfd0570db4e7a8d1300e03",
    },
}
METRIC_KEYS = {"schema_version", "implementation", "preparation_wall_seconds",
               "region_wall_seconds", "region_user_seconds", "region_system_seconds",
               "region_child_user_seconds", "region_child_system_seconds", "baseline_rss_bytes"}
TIME_KEYS = {"wall_seconds", "user_seconds", "system_seconds", "maximum_rss_kib", "exit_status"}
TIME_FORMAT = "wall_seconds\t%e\nuser_seconds\t%U\nsystem_seconds\t%S\nmaximum_rss_kib\t%M\nexit_status\t%x"
THREAD_LIMITS = ("OPENBLAS_NUM_THREADS", "OMP_NUM_THREADS", "MKL_NUM_THREADS",
                 "BLIS_NUM_THREADS", "VECLIB_MAXIMUM_THREADS")


def _exact_keys(data, keys, label):
    if not isinstance(data, dict) or set(data) != set(keys):
        raise ValueError(f"{label} keys differ")


def _table(path, header=None):
    try:
        with path.open(encoding="utf-8", newline="") as handle:
            rows = list(csv.reader(handle, delimiter="\t"))
    except OSError as exc:
        raise ValueError(f"missing table: {path}") from exc
    if not rows or (header is not None and rows[0] != header) or any(len(row) != len(rows[0]) for row in rows):
        raise ValueError(f"malformed table: {path}")
    return rows


def _unique_table(text, keys, label):
    rows = list(csv.reader(text.splitlines(), delimiter="\t"))
    if not rows or rows[0] != ["metric", "value"] or any(len(row) != 2 for row in rows[1:]):
        raise ValueError(f"malformed {label}")
    values = {}
    for key, value in rows[1:]:
        if key in values:
            raise ValueError(f"duplicate {label} key: {key}")
        values[key] = value
    _exact_keys(values, keys, label)
    return values


def _duration(value, label, positive=False):
    try:
        parsed = float(value)
    except (TypeError, ValueError) as exc:
        raise ValueError(f"invalid {label}") from exc
    if not math.isfinite(parsed) or parsed < 0 or (positive and parsed == 0):
        raise ValueError(f"invalid {label}")
    return parsed


def _positive_integer(value, label, allow_zero=False):
    if not isinstance(value, str) or not value.isdecimal():
        raise ValueError(f"invalid {label}")
    result = int(value)
    if result < (0 if allow_zero else 1) or result > (2**64 - 1):
        raise ValueError(f"invalid {label}")
    return result


def _finite_value(value, label):
    try:
        parsed = float(value)
    except (TypeError, ValueError) as exc:
        raise ValueError(f"invalid {label}") from exc
    if not math.isfinite(parsed):
        raise ValueError(f"invalid {label}")
    return parsed


def validate_pins(bundle: Path, receipt: dict) -> dict:
    if receipt != TRUSTED or type(receipt.get("schema_version")) is not int:
        raise ValueError("measurement input receipt differs from trusted accepted pins")
    bundle = Path(bundle)
    try:
        oracle = json.loads((bundle / "oracle.json").read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ValueError("missing or malformed fresh oracle.json") from exc
    if oracle.get("package", {}).get("source_commit") != TRUSTED["source_commit"]:
        raise ValueError("fresh source commit mismatch")
    stages = oracle.get("cases", {}).get("full", {}).get("stages", {})
    hashes = oracle.get("sha256", {})
    for step, count in (("1", 9939), ("14", 8508)):
        stage = stages.get(step, {})
        if stage.get("genes") != count or stage.get("cells") != 184:
            raise ValueError(f"fresh stage {step} dimensions mismatch")
        for key, suffix in (("expression", ""), ("gene_order", ".genes"), ("cell_groups", ".cells")):
            rel = f"full/{int(step):02d}{suffix}.tsv"
            if stage.get(key) != rel or hashes.get(rel) != TRUSTED["files"][rel]:
                raise ValueError(f"fresh {rel} pin mismatch")
            if file_sha256(owned_file(bundle, rel)) != TRUSTED["files"][rel]:
                raise ValueError(f"fresh {rel} bytes mismatch")
    checkpoint = stages["1"].get("checkpoint")
    if checkpoint != "full/upstream/01_incoming_data.infercnv_obj" or checkpoint not in hashes:
        raise ValueError("fresh stage-1 checkpoint path missing or wrong")
    if file_sha256(owned_file(bundle, checkpoint)) != hashes[checkpoint]:
        raise ValueError("fresh stage-1 checkpoint hash mismatch")
    return {"checkpoint": checkpoint, "checkpoint_sha256": hashes[checkpoint],
            "files": dict(TRUSTED["files"]), "source_commit": TRUSTED["source_commit"]}


def _read_metrics(path, implementation):
    try:
        metrics = _unique_table(path.read_text(encoding="utf-8"), METRIC_KEYS, "metrics")
    except OSError as exc:
        raise ValueError("missing metrics.tsv") from exc
    if metrics["schema_version"] != "1" or metrics["implementation"] != implementation:
        raise ValueError("metrics schema or implementation mismatch")
    result = {key: _duration(value, key, key == "region_wall_seconds") for key, value in metrics.items()
              if key.endswith("_seconds")}
    for key in ("region_child_user_seconds", "region_child_system_seconds"):
        if result[key] != 0:
            raise ValueError(f"unexpected child CPU: {key}")
    result["baseline_rss_bytes"] = _positive_integer(metrics["baseline_rss_bytes"], "baseline RSS")
    return result


def validate_trial(result: Path, expected: Path, implementation: str) -> dict:
    if implementation not in ("rust", "infercnv"):
        raise ValueError("unknown implementation")
    result, expected = Path(result), Path(expected)
    metrics = _read_metrics(result / "metrics.tsv", implementation)
    hashes = {}
    for suffix, header in ((".genes.tsv", ["gene", "chr", "start", "stop"]),
                           (".cells.tsv", ["cell", "group", "role"])):
        name = "14" + suffix
        a, b = _table(result / name, header), _table(expected / name, header)
        if a != b or len(a) < 2:
            raise ValueError(f"final identity mismatch: {name}")
        hashes[name] = file_sha256(result / name)
    actual = _table(result / "14.tsv")
    reference = _table(expected / "14.tsv")
    if len(actual) != len(reference) or actual[0] != reference[0] or actual[0][0] != "gene" or len(actual) < 2:
        raise ValueError("final matrix shape or header mismatch")
    if [row[0] for row in actual[1:]] != [row[0] for row in reference[1:]] or \
            [row[0] for row in actual[1:]] != [row[0] for row in _table(result / "14.genes.tsv")[1:]] or \
            actual[0][1:] != [row[0] for row in _table(result / "14.cells.tsv")[1:]]:
        raise ValueError("final matrix identities mismatch")
    absolute = scaled = 0.0
    for observed, wanted in zip(actual[1:], reference[1:]):
        for raw_a, raw_b in zip(observed[1:], wanted[1:]):
            a, b = _finite_value(raw_a, "final value"), _finite_value(raw_b, "expected final value")
            error = abs(a - b)
            bound = 1e-12 + 1e-12 * abs(b)
            absolute = max(absolute, error)
            scaled = max(scaled, error / bound)
            if error > bound:
                raise ValueError("final value exceeds tolerance")
    hashes["14.tsv"] = file_sha256(result / "14.tsv")
    return {**metrics, "max_absolute_error": absolute, "max_scaled_error": scaled,
            "output_sha256": hashes}


def _parse_time_file(text):
    values = _unique_table("metric\tvalue\n" + text, TIME_KEYS, "GNU time")
    result = {key: _duration(values[key], key) for key in ("wall_seconds", "user_seconds", "system_seconds")}
    rss = _positive_integer(values["maximum_rss_kib"], "maximum RSS KiB")
    if rss > (2**64 - 1) // 1024 or _positive_integer(values["exit_status"], "exit status", True) != 0:
        raise ValueError("GNU time overflow or child failed")
    result["maximum_rss_bytes"] = rss * 1024
    return result


def _run_child(command, prefix, timeout=1200, env=None):
    prefix = Path(prefix)
    with (Path(str(prefix) + ".stdout")).open("wb") as stdout, \
            (Path(str(prefix) + ".stderr")).open("wb") as stderr:
        process = subprocess.Popen(command, stdout=stdout, stderr=stderr, env=env,
                                   start_new_session=True)
        try:
            status = process.wait(timeout=timeout)
        except BaseException as exc:
            try:
                os.killpg(process.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                pass
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait(timeout=5)
            if isinstance(exc, subprocess.TimeoutExpired):
                raise RuntimeError(f"trial timed out; raw output preserved at {prefix}: {exc}") from exc
            raise
        if status != 0:
            raise RuntimeError(f"trial child exited {status}; raw output preserved at {prefix}")


def _trial_order():
    order = [("smoke", 0, ("rust", "infercnv"))]
    order += [("measured", pair, ("rust", "infercnv") if pair % 2 else ("infercnv", "rust"))
              for pair in range(1, 8)]
    return order


def _trial_argv(implementation, bundle, binary, r_script, result_dir):
    if implementation == "rust":
        return [str(binary.resolve()), str((bundle / "full").resolve()), str(result_dir)]
    if implementation == "infercnv":
        return ["Rscript", "--vanilla", str(r_script.resolve()),
                str(bundle.resolve()), str(result_dir)]
    raise ValueError("unknown implementation")


def summarize_trials(trials: list[dict]) -> dict:
    expected = [(phase, pair, implementation) for phase, pair, implementations in _trial_order()
                if phase == "measured" for implementation in implementations]
    if [(trial.get("phase"), trial.get("pair"), trial.get("implementation")) for trial in trials] != expected:
        raise ValueError("missing, extra, duplicate or out-of-order measured trials")
    groups = {}
    numeric_fields = ("preparation_wall_seconds", "region_wall_seconds", "region_user_seconds",
                      "region_system_seconds", "baseline_rss_bytes")
    lifetime_fields = ("wall_seconds", "user_seconds", "system_seconds", "maximum_rss_bytes")
    for implementation in ("rust", "infercnv"):
        own = [t for t in trials if t["implementation"] == implementation]
        group = {}
        for key in numeric_fields + tuple("lifetime_" + key for key in lifetime_fields):
            try:
                raw = [t["lifetime"][key.removeprefix("lifetime_")] if key.startswith("lifetime_")
                       else t[key] for t in own]
            except (KeyError, TypeError) as exc:
                raise ValueError(f"missing measured metric: {key}") from exc
            values = [_duration(value, key, key == "region_wall_seconds") for value in raw]
            if key.endswith("rss_bytes") and any(value != int(value) or value <= 0 for value in values):
                raise ValueError(f"invalid measured RSS: {key}")
            group.update({f"{key}_median": statistics.median(values), f"{key}_min": min(values),
                          f"{key}_max": max(values)})
        groups[implementation] = group
    return {"implementations": groups,
            "infercnv_to_rust_median_wall_ratio": groups["infercnv"]["region_wall_seconds_median"] /
                                                  groups["rust"]["region_wall_seconds_median"],
            "clear_rust_wall_advantage": groups["rust"]["region_wall_seconds_max"] <
                                         groups["infercnv"]["region_wall_seconds_min"]}


def run_measurement(bundle: Path, binary: Path, output: Path, input_receipt: Path, r_script: Path) -> dict:
    bundle, binary, output, input_receipt, r_script = map(Path, (bundle, binary, output, input_receipt, r_script))
    receipt = json.loads(input_receipt.read_text(encoding="utf-8"))
    pin_record = validate_pins(bundle, receipt)
    validate_bundle(bundle)
    if not binary.is_file() or not os.access(binary, os.X_OK) or not r_script.is_file():
        raise ValueError("missing executable or R script")
    if output.exists():
        raise ValueError("measurement output must be new")
    output.mkdir(parents=True)
    (output / "input-pins.json").write_text(json.dumps(pin_record, indent=2) + "\n")
    (output / "binary.sha256").write_text(file_sha256(binary) + "\n")
    env = os.environ.copy()
    env.update({key: "1" for key in THREAD_LIMITS})
    env["INFER_CNV_MEASUREMENT_INPUT_RECEIPT"] = str(input_receipt.resolve())
    trials = []
    expected = bundle / "full"
    for phase, pair, implementations in _trial_order():
        for implementation in implementations:
            name = f"{phase}-{pair:02d}-{implementation}"
            trial_dir = output / name
            trial_dir.mkdir()
            result_dir = trial_dir / "result"
            executable = _trial_argv(implementation, bundle, binary, r_script, result_dir)
            time_path = trial_dir / "gnu-time.tsv"
            command = ["/usr/bin/time", "-f", TIME_FORMAT, "-o", str(time_path), *executable]
            _run_child(command, trial_dir / "child", 1200, env)
            validated = validate_trial(result_dir, expected, implementation)
            lifetime = _parse_time_file(time_path.read_text(encoding="utf-8"))
            trial = {"phase": phase, "pair": pair, "implementation": implementation,
                     "execution_order": len(trials), **validated, "lifetime": lifetime}
            (trial_dir / "validated.json").write_text(json.dumps(trial, indent=2) + "\n")
            trials.append(trial)
    summary = summarize_trials([t for t in trials if t["phase"] == "measured"])
    report = {"schema_version": 1, "input": pin_record, "thread_limits": {key: "1" for key in THREAD_LIMITS},
              "trials": trials, "summary": summary}
    (output / "summary.json").write_text(json.dumps(report, indent=2) + "\n")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("bundle", "binary", "output", "input-receipt", "r-script"):
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    try:
        run_measurement(args.bundle, args.binary, args.output, args.input_receipt, args.r_script)
    except (OSError, ValueError, RuntimeError) as exc:
        parser.exit(1, f"measurement failed: {exc}\n")


if __name__ == "__main__":
    main()
