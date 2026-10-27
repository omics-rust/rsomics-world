#!/usr/bin/env python3
"""Measure a real raw grouped-i6 Bayesian workflow against its accepted binary."""

import argparse
import json
import math
import os
from pathlib import Path
import platform
import statistics

from compare_infercnv_denoise_workflow import check
from compare_infercnv_hmm_workflow import native_command, read_reports, timed_native
from infercnv_trial_process import run_child
from measure_infercnv import TIME_FORMAT, _parse_time_file
from measure_infercnv_samples_cli import report_fingerprint
from validate_infercnv_shipped_oracle import file_sha256, owned_file


def distribution(values):
    check(bool(values) and all(math.isfinite(value) and value > 0 for value in values),
          "missing or invalid measurement")
    return dict(values=values, min=min(values), median=statistics.median(values), max=max(values))


def summarize(trials):
    measured = [trial for trial in trials if trial["phase"] == "measured"]
    source = [trial for trial in measured if trial["implementation"] == "R"]
    native = {worker: [trial for trial in measured
                       if trial["implementation"] == "native" and trial["threads"] == worker]
              for worker in (1, 4)}
    check(len(source) == 3 and all(len(rows) == 5 for rows in native.values()),
          "incomplete three-R/five-native paired campaign")
    r_summary = {field: distribution([trial[field] for trial in source])
                 for field in ("public_call_seconds", "wall_seconds", "maximum_rss_bytes")}
    workers = {str(worker): {field: distribution([trial[field] for trial in rows])
                            for field in ("wall_seconds", "maximum_rss_bytes")}
               for worker, rows in native.items()}
    serial, parallel = (workers[str(worker)]["wall_seconds"] for worker in (1, 4))
    actual = r_summary["public_call_seconds"]
    return dict(R=r_summary, native=workers,
                strict_serial_wall_advantage=serial["max"] < actual["min"],
                conservative_serial_median_ratio=actual["median"] / serial["median"],
                strict_parallel_wall_separation=parallel["max"] < serial["min"],
                native_worker_median_ratio=serial["median"] / parallel["median"],
                observed_serial_peak_rss_ratio=(r_summary["maximum_rss_bytes"]["median"]
                                                / workers["1"]["maximum_rss_bytes"]["median"]))


def adjusted_tables(report):
    result = dict(pred_cnv_regions=[], pred_cnv_genes=[])
    for group in report["bayesian"]["groups"]:
        for region in group["regions"]:
            result["pred_cnv_regions"].append((group["name"], region["name"], region["state"],
                                               region["chromosome"], region["start"], region["end"]))
            start, end = region["gene_range"]
            result["pred_cnv_genes"].extend((group["name"], region["name"], region["state"],
                                             gene["id"], gene["chromosome"], gene["start"], gene["end"])
                                            for gene in report["genes"][start:end])
    return result


def compare_adjusted(actual, expected, ambiguities):
    differences = set()
    for name, rows in expected.items():
        check(len(actual[name]) == len(rows), f"actual adjusted {name} row count differs")
        for left, right in zip(actual[name], rows):
            check(left[:2] == right[:2] and left[3:] == right[3:],
                  f"actual adjusted {name} identities, coordinates or order differ")
            if left[2] != right[2]:
                allowed = ambiguities.get(left[1], ())
                check(left[2] in allowed and right[2] in allowed,
                      f"stable adjusted state differs for {left[1]}")
                differences.add(left[1])
    return sorted(differences)


def validate_R(root, trial, baseline, original, ambiguities):
    check(trial["status"] == "complete" and trial["schema_version"] == 1 and trial["seed"] == 261027,
          "actual R public workflow did not complete")
    expression = trial["expression"]
    check((expression["genes"], expression["cells"]) == (8508, 184)
          and expression["gene_ids"] == [gene["id"] for gene in baseline["genes"]]
          and expression["cell_ids"] == [cell["id"] for cell in baseline["cells"]],
          "actual full-workflow expression identities differ")
    check(read_reports(root, trial["original_reports"]) == original,
          "actual raw HMM reports differ from the accepted workflow")
    shared = [name for name in trial["original_reports"]
              if name.endswith((".cell_groupings", ".genes_used.dat"))]
    actual = read_reports(root, shared + trial["adjusted_reports"])
    differences = compare_adjusted(actual, adjusted_tables(baseline), ambiguities)
    return dict(original_regions=len(original["pred_cnv_regions"]),
                retained_regions=len(actual["pred_cnv_regions"]),
                source_indeterminate_state_differences=differences,
                mcmc_sha256=file_sha256(owned_file(root, trial["mcmc_rds"])))


def run_measurement(shipped, hmm, native_evidence, output):
    shipped, hmm, native_evidence = [Path(path).resolve(strict=True)
                                     for path in (shipped, hmm, native_evidence)]
    output = Path(output).absolute()
    check(platform.system() == "Linux", "GNU time and native Linux execution are required")
    output.mkdir(parents=True, exist_ok=False)
    scripts = Path(__file__).resolve().parent
    driver = scripts / "infercnv_bayes_performance.R"
    binary = native_evidence / "rsomics-sc"
    accepted = json.loads((native_evidence / "summary.json").read_text())
    baseline = json.loads((native_evidence / "baseline.json").read_text())
    witness = json.loads((hmm / "witness.json").read_text())
    case = witness["cases"]["full"]
    check(accepted["status"] == "complete" and file_sha256(binary) == accepted["binary_sha256"]
          and report_fingerprint(baseline, 1) == accepted["results"][0]["report_fingerprint"]
          and file_sha256(shipped / "sha256-manifest.tsv") == witness["source_bundles"]["shipped"],
          "accepted binary, report or input identity differs")
    ambiguities = {}
    for comparison in accepted["results"][0]["comparisons"]:
        own = {row["region"]: set(row["admissible_states"])
               for row in comparison["ambiguous_reassignments"]}
        if not ambiguities:
            ambiguities = own
        else:
            ambiguities = {name: states & own.get(name, set()) for name, states in ambiguities.items()}
    original = read_reports(hmm, case["reports"])
    for key in ("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS", "VECLIB_MAXIMUM_THREADS"):
        os.environ[key] = "1"
    affinity = sorted(os.sched_getaffinity(0))
    selected = affinity[:4]
    prefix = ["taskset", "--cpu-list", ",".join(map(str, selected))]
    inputs = {name: file_sha256(owned_file(shipped, case["inputs"][name]))
              for name in ("counts", "positions", "annotations")}
    provenance = dict(scope="same-host raw Create+run19 grouped i6/Bayesian versus complete native CLI",
                      machine=platform.uname()._asdict(), available_affinity=affinity, selected_affinity=selected,
                      cpu_info=Path("/proc/cpuinfo").read_text(), memory_info=Path("/proc/meminfo").read_text(),
                      load_at_start=os.getloadavg(),
                      binary_sha256=file_sha256(binary), input_sha256=inputs,
                      source_hmm_witness_sha256=file_sha256(hmm / "witness.json"),
                      controller_sha256=file_sha256(Path(__file__)), R_driver_sha256=file_sha256(driver),
                      memory="GNU time process maximum RSS; not aggregate fork memory or a hard memory bound",
                      timing="R public call excludes startup; native includes entire process and JSON; warm input cache",
                      excluded=["large cohorts", "cold I/O", "default subclusters", "biological validity", "general convergence"])
    (output / "provenance.json").write_text(json.dumps(provenance, indent=2) + "\n")
    trials = []
    native_fingerprint = None
    for index in range(6):
        phase = "warmup" if index == 0 else "measured"
        if index <= 3:
            name = f"R-{index}"
            destination = output / name
            command = ["/usr/bin/time", "-f", TIME_FORMAT, "-o", str(output / f"{name}.time.txt"),
                       *prefix, "Rscript", "--vanilla", str(driver), str(shipped), str(hmm), str(destination)]
            print(f"Running actual raw R workflow: {phase} {index}", flush=True)
            child = run_child(command, output / name, timeout=1800)
            record = json.loads((destination / "trial.json").read_text())
            metrics = _parse_time_file((output / f"{name}.time.txt").read_text())
            validation = validate_R(destination, record, baseline, original, ambiguities)
            trials.append(dict(phase=phase, implementation="R", threads=1, index=index,
                               owner_wall_seconds=(child.end_ns - child.start_ns) / 1e9,
                               public_call_seconds=record["public_call_seconds"],
                               public_user_seconds=record["user_seconds"],
                               public_system_seconds=record["system_seconds"], **metrics, **validation))
        for worker in ((1, 4) if index % 2 == 0 else (4, 1)):
            name = f"native-{index}-threads-{worker}"
            report_path = output / f"{name}.json"
            command = native_command(binary, shipped, case, witness["seed"], worker, report_path)
            command += ["--bayes", "--bayes-max-p-normal", "0.5", "--bayes-seed", "0"]
            metrics = timed_native(prefix + command, output / name)
            report = json.loads(report_path.read_text())
            fingerprint = report_fingerprint(report, worker)
            check(native_fingerprint is None or native_fingerprint == fingerprint,
                  "native trials or worker counts change the report")
            native_fingerprint = fingerprint
            compare_adjusted(adjusted_tables(report), adjusted_tables(baseline), ambiguities)
            trials.append(dict(phase=phase, implementation="native", threads=worker, index=index,
                               report_fingerprint=fingerprint, **metrics))
        (output / "trials.json").write_text(json.dumps(trials, indent=2) + "\n")
    result = dict(status="complete", qualification=summarize(trials), trials=trials, provenance=provenance,
                  load_at_end=os.getloadavg())
    (output / "summary.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result["qualification"], indent=2), flush=True)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("shipped", "hmm", "native-evidence", "output"):
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    try:
        run_measurement(args.shipped, args.hmm, args.native_evidence, args.output)
    except (OSError, ValueError, RuntimeError, KeyError) as error:
        parser.exit(1, f"whole Bayesian measurement failed: {error}\n")


if __name__ == "__main__":
    main()
