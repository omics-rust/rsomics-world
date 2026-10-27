#!/usr/bin/env python3
"""Compare native raw-to-i6 workflows with retained installed-infercnv results."""

import argparse
import csv
from decimal import Decimal
import json
from pathlib import Path
import platform
import re

from compare_infercnv_denoise_workflow import check, close, identities, matrix_error
from infercnv_trial_process import run_child
from measure_infercnv import TIME_FORMAT, _parse_time_file
from measure_infercnv_samples_cli import report_fingerprint
from validate_infercnv_shipped_oracle import COMMIT, SOURCE_SHA256, file_sha256, owned_file, read_canonical


LEVELS = ("cnv:0.01", "cnv:0.5", "cnv:1", "cnv:1.5", "cnv:2", "cnv:3")
TABLES = {
    "cell_groupings": (["cell_group_name", "cell"], ()),
    "pred_cnv_regions": (["cell_group_name", "cnv_name", "state", "chr", "start", "end"], (2, 4, 5)),
    "pred_cnv_genes": (["cell_group_name", "gene_region_name", "state", "gene", "chr", "start", "end"], (2, 5, 6)),
    "genes_used": (["chr", "start", "stop"], (2, 3)),
}


def integer(value):
    value = Decimal(value)
    check(value.is_finite() and value == value.to_integral_value(), "noninteger report value")
    return int(value)


def read_reports(root, paths):
    result = {}
    for name, (header, numeric_columns) in TABLES.items():
        suffix = "." + name + ("" if name == "cell_groupings" else ".dat")
        matches = [path for path in paths if path.endswith(suffix)]
        check(len(matches) == 1, f"missing or duplicate actual {name} report")
        with owned_file(root, matches[0]).open(newline="") as source:
            rows = csv.reader(source, delimiter="\t")
            check(next(rows) == header, f"unexpected actual {name} header")
            width = len(header) + (name == "genes_used")
            records = []
            for row in rows:
                check(len(row) == width, f"incorrect actual {name} row width")
                records.append(tuple(integer(value) if index in numeric_columns else value
                                     for index, value in enumerate(row)))
            result[name] = records
    check(len(paths) == 4, "actual HMM reports are incomplete")
    return result


def calibration_error(actual, expected, spike_shape):
    check((actual["simulated_genes"], actual["simulated_cells"]) == spike_shape,
          "simulated calibration dimensions differ")
    check(len(actual["means"]) == len(actual["standard_deviations"]) == len(actual["log_sd_fits"]) == 6,
          "calibration must contain six states")
    maximum = 0.0
    for state, level in enumerate(LEVELS):
        mean_sd = expected["means"][level]
        coefficients = expected["coefficients"][level]
        fit = actual["log_sd_fits"][state]
        check(len(fit) == len(coefficients) == 2, "log-SD regression dimensions differ")
        for left, right in [(actual["means"][state], mean_sd["mean"]),
                            (actual["standard_deviations"][state], mean_sd["sd"]), *zip(fit, coefficients)]:
            maximum = max(maximum, close(left, right, 1e-9 + 1e-8 * abs(float(right)),
                                         f"calibration differs for {level}"))
    return maximum


def compare_report(report, case, expression, states, genes, cells, spike_shape, tables):
    check(report["schema"] == "rsomics-sc.cnv.samples.v1", "unexpected native report schema")
    check(report["genes"] == genes and report["cells"] == cells, "gene/cell identities or order differ")
    check(expression.genes == states.genes == tuple(gene["id"] for gene in genes)
          and expression.cells == states.cells == tuple(cell["id"] for cell in cells),
          "actual HMM expression/state identities differ")
    expression_error = matrix_error(report["expression"], expression, 14)
    hmm = report["hmm"]
    check((hmm["step"], hmm["model"], hmm["grouping"], hmm["bayesian_filter"]) == (17, "i6", "samples", False),
          "native HMM operation differs from the actual workflow")
    maximum = calibration_error(hmm["calibration"], case["calibration"], spike_shape)
    check(len(hmm["groups"]) == len(case["groups"]) == len(report["groups"]), "HMM group count differs")
    projected = {name: [] for name in TABLES}
    projected["genes_used"] = [(gene["id"], gene["chromosome"], gene["start"], gene["end"]) for gene in genes]
    members_seen = []
    state_values = 0
    for actual, expected, sample in zip(hmm["groups"], case["groups"], report["groups"]):
        members = actual["member_indices"]
        check(sample["name"] == expected["name"]
              and actual["name"] == sample["name"] + "." + sample["subcluster_name"]
              and members == sample["member_indices"]
              and sorted(members) == sorted(index - 1 for index in expected["indices"]),
              "HMM group identity or membership differs")
        members_seen.extend(members)
        path = actual["states"]
        check(len(path) == len(genes) and all(type(state) is int and 1 <= state <= 6 for state in path),
              "HMM state path dimensions or state values differ")
        for gene, row in enumerate(states.rows):
            check(all(row[cell] == path[gene] for cell in members),
                  f"HMM states differ for {actual['name']} at {genes[gene]['id']}")
            state_values += len(members)
        projected["cell_groupings"].extend((actual["name"], cells[index]["id"]) for index in members)
        for region in actual["regions"]:
            start, end = region["gene_range"]
            check(0 <= start < end <= len(genes) and region["state"] != 3,
                  "invalid native predicted region range or neutral state")
            selected = genes[start:end]
            check(all(gene["chromosome"] == region["chromosome"] for gene in selected)
                  and all(state == region["state"] for state in path[start:end])
                  and region["start"] == min(gene["start"] for gene in selected)
                  and region["end"] == max(gene["end"] for gene in selected),
                  "native region disagrees with its genes or states")
            projected["pred_cnv_regions"].append((actual["name"], region["name"], region["state"],
                                                  region["chromosome"], region["start"], region["end"]))
            projected["pred_cnv_genes"].extend((actual["name"], region["name"], region["state"],
                                                gene["id"], gene["chromosome"], gene["start"], gene["end"])
                                               for gene in selected)
    check(sorted(members_seen) == list(range(len(cells))), "HMM groups do not cover cells exactly once")
    for name in TABLES:
        check(projected[name] == tables[name], f"native projection differs from actual {name}")
    return dict(genes=len(genes), cells=len(cells), groups=len(hmm["groups"]), state_values=state_values,
                regions=len(projected["pred_cnv_regions"]), predicted_genes=len(projected["pred_cnv_genes"]),
                max_calibration_scaled_error=maximum, max_expression_scaled_error=expression_error)


def timed_native(command, prefix):
    time_path = prefix.with_suffix(".time.txt")
    system = platform.system()
    if system == "Linux":
        command = ["/usr/bin/time", "-f", TIME_FORMAT, "-o", str(time_path), *command]
    elif system == "Darwin":
        command = ["/usr/bin/time", "-l", "-o", str(time_path), *command]
    trial = run_child(command, prefix, timeout=300)
    metrics = dict(owner_wall_seconds=(trial.end_ns - trial.start_ns) / 1e9)
    if system == "Linux":
        metrics.update(_parse_time_file(time_path.read_text()))
        metrics["timing_source"] = "GNU time; owner wall includes controller process ownership checks"
    elif system == "Darwin":
        match = re.search(r"^\s*(\d+)\s+maximum resident set size\s*$", time_path.read_text(), re.MULTILINE)
        check(match is not None, "BSD time maximum RSS is missing")
        metrics["maximum_rss_bytes"] = int(match[1])
        metrics["timing_source"] = "BSD time RSS bytes; owner wall includes controller process ownership checks"
    else:
        metrics["timing_source"] = "controller process ownership wall only; RSS unavailable"
    return metrics


def native_command(binary, bundle, case, seed, threads, output):
    parameters = case["run_parameters"]
    command = [str(binary), "cnv", "samples", "--hmm", "--cluster-by-groups", "--hmm-seed", str(seed),
               "--hmm-transition-prob", str(parameters["HMM_transition_prob"]), "--include-expression",
               "--threads", str(threads), "--output", str(output)]
    for name in ("counts", "positions", "annotations"):
        command += ["--" + name, str(owned_file(bundle, case["inputs"][name]))]
    for reference in case["inputs"]["references"]:
        command += ["--reference", reference]
    for name in ("cutoff", "min_cells_per_gene", "window_length", "max_centered_threshold", "z_score_filter"):
        command += ["--" + name.replace("_", "-"), str(parameters[name])]
    if not parameters["ref_subtract_use_mean_bounds"] and case["inputs"]["references"]:
        command += ["--reference-mean"]
    if parameters["hspike_aggregate_normals"]:
        command += ["--hspike-aggregate-normals"]
    return command


def run_comparison(synthetic, shipped, witness, binary, output):
    synthetic, shipped, witness, binary = [Path(path).resolve(strict=True) for path in (synthetic, shipped, witness, binary)]
    output = Path(output).absolute()
    output.mkdir(parents=True, exist_ok=False)
    record = json.loads((witness / "witness.json").read_text())
    check(record["status"] == "complete" and record["schema_version"] == 1
          and record["successful_cases"] == ["full", "proxy", "aggregated"]
          and record["expected_error_cases"] == ["grouped"] and len(record["cases"]) == 4,
          "incomplete installed-package HMM evidence")
    check(record["package"] == dict(name="infercnv", version="1.28.0", source_commit=COMMIT,
                                    source_archive_sha256=SOURCE_SHA256)
          and record["seed"] == 261027
          and record["rng_kind"] == ["Mersenne-Twister", "Inversion", "Rejection"],
          "installed-package source or RNG profile differs")
    for name, digest in record["files"].items():
        check(file_sha256(owned_file(witness, name)) == digest, "installed-package HMM export changed")
    check("R version 4.6.1" in (witness / "metadata/session.txt").read_text(), "installed R version differs")
    bundles = dict(synthetic=synthetic, shipped=shipped)
    for name, bundle in bundles.items():
        check(file_sha256(bundle / "sha256-manifest.tsv") == record["source_bundles"][name], "source bundle differs")
    negative = record["cases"]["grouped"]
    check(negative["status"] == "expected_error"
          and negative["error"]["message"] == "smoothing parameter value too small"
          and negative["error"]["call"] == "smooth.spline(logv ~ logm)",
          "recorded upstream synthetic failure differs")
    binary_hash = file_sha256(binary)
    results = []
    for case_id in record["successful_cases"]:
        case = record["cases"][case_id]
        parameters = case["run_parameters"]
        check(case["status"] == "success" and parameters["HMM"] and parameters["HMM_type"] == "i6"
              and parameters["analysis_mode"] == "samples" and parameters["cluster_by_groups"]
              and parameters["HMM_report_by"] == "subcluster" and parameters["BayesMaxPNormal"] == 0
              and parameters["up_to_step"] == 17, "actual HMM operation differs from native scope")
        genes, cells = identities(witness, case["expression"])
        expression = read_canonical(owned_file(witness, case["expression"]["expression"]), len(cells))
        states = read_canonical(owned_file(witness, case["states"]["expression"]), len(cells))
        spike_genes, spike_cells = identities(witness, case["spike"])
        tables = read_reports(witness, case["reports"])
        baseline = None
        for threads in (1, 4):
            prefix = output / f"{case_id}-threads-{threads}"
            report_path = prefix.with_suffix(".report.json")
            command = native_command(binary, bundles[case["inputs"]["bundle"]], case, record["seed"], threads, report_path)
            metrics = timed_native(command, prefix)
            report = json.loads(report_path.read_text())
            result = compare_report(report, case, expression, states, genes, cells, (len(spike_genes), len(spike_cells)), tables)
            fingerprint = report_fingerprint(report, threads)
            check(baseline is None or fingerprint == baseline, "native HMM reports differ across workers")
            baseline = fingerprint
            results.append(dict(case=case_id, threads=threads, **result, **metrics,
                                report_fingerprint=fingerprint, report_sha256=file_sha256(report_path),
                                upstream_in_process_wall_seconds=case["raw_create_to_return_wall_seconds"]))
            print(f"matched actual i6 workflow {case_id}: workers={threads}, genes={len(genes)}, "
                  f"regions={result['regions']}, max_calibration_scaled_error={result['max_calibration_scaled_error']:.6g}", flush=True)
    check(file_sha256(binary) == binary_hash, "native binary changed")
    summary = dict(status="complete", cases=3, native_processes=len(results), binary_sha256=binary_hash,
                   witness_sha256=file_sha256(witness / "witness.json"), upstream_expected_error_cases=["grouped"],
                   results=results, calibration_tolerance=dict(absolute=1e-9, relative=1e-8),
                   scope="actual raw-to-grouped-i6 compatibility and four report projections; no Bayesian filtering",
                   timing="native process timing and retained upstream in-process timing are not paired performance")
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("synthetic", "shipped", "witness", "binary", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    try:
        run_comparison(args.synthetic, args.shipped, args.witness, args.binary, args.output)
    except (OSError, ValueError, RuntimeError) as error:
        parser.exit(1, f"HMM comparison failed: {error}\n")


if __name__ == "__main__":
    main()
