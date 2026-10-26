#!/usr/bin/env python3
"""Compare real native raw workflows with installed infercnv samples/denoise exports."""

import argparse
import csv
from decimal import Decimal
import json
import math
from pathlib import Path

from infercnv_trial_process import run_child
from measure_infercnv import TIME_FORMAT, _parse_time_file
from measure_infercnv_samples_cli import report_fingerprint
from validate_infercnv_shipped_oracle import file_sha256, owned_file, read_canonical


def check(condition, message):
    if not condition:
        raise ValueError(message)


def close(actual, expected, scale, context):
    actual, expected = float(actual), float(expected)
    error = abs(actual - expected) / scale
    check(math.isfinite(actual) and math.isfinite(expected) and error <= 1, context)
    return error


def matrix_error(actual, expected, step):
    genes, cells = len(expected.genes), len(expected.cells)
    check(actual["step"] == step and actual["layout"] == "column-major"
          and actual["shape"] == [genes, cells] and len(actual["values"]) == genes * cells,
          f"incorrect step-{step} matrix contract")
    maximum = 0.0
    for gene, row in enumerate(expected.rows):
        for cell, value in enumerate(row):
            maximum = max(maximum, close(actual["values"][cell * genes + gene], value,
                                         1e-12 + 1e-12 * abs(value),
                                         f"step-{step} expression differs at {expected.genes[gene]}/{expected.cells[cell]}"))
    return maximum


def compare_report(report, case, before, after, genes, cells):
    check(report["schema"] == "rsomics-sc.cnv.samples.v1", "unexpected native schema")
    check(report["genes"] == genes and report["cells"] == cells, "gene/cell identity or order differs")
    check(tuple(g["id"] for g in genes) == before.genes == after.genes
          and tuple(c["id"] for c in cells) == before.cells == after.cells, "exported matrix identities differ")
    check(report["selected_gene_indices"] == [i - 1 for i in case["diagnostic"]["selected_gene_indices"]],
          "selected genes differ")
    outliers = case["diagnostic"]["outliers"]
    check(report["outlier_gene_indices"] == (None if outliers is None else [i - 1 for i in outliers]),
          "reference outliers differ")
    check(len(report["groups"]) == len(case["groups"]), "group count differs")
    trees = 0
    for actual, expected in zip(report["groups"], case["groups"]):
        check(actual["name"] == expected["name"] and len(expected["subclusters"]) == 1,
              "group identity or samples partition differs")
        subgroup = expected["subclusters"][0]
        check(actual["subcluster_name"] == subgroup["name"]
              and actual["member_indices"] == [i - 1 for i in subgroup["indices"]],
              "group memberships/order differ")
        tree = expected["tree"]
        if tree is None:
            check(actual["tree"] is None, "invented tree for a small group")
        else:
            trees += 1
            check(actual["tree"] is not None and actual["tree"]["merge"] == tree["merge"]
                  and actual["tree"]["order"] == [i - 1 for i in tree["order"]], "tree merges/order differ")
            check(len(actual["tree"]["height"]) == len(tree["height"]), "tree heights shape differs")
            for left, right in zip(actual["tree"]["height"], tree["height"]):
                close(left, right, 1e-12 * max(1, abs(float(right))), "tree height differs")
    before_error = matrix_error(report["expression"], before, 14)
    if case["mode"]["enabled"]:
        check("denoised" in report, "requested denoise output is missing")
        bounds = report["denoised"]["bounds"]
        if case["noise"] is None:
            check(bounds is None, "unexpected noise bounds")
        else:
            check(bounds is not None, "observed noise bounds are missing")
            for key in ("center", "threshold"):
                value = case["noise"][key]
                close(bounds[key], value, 1e-12 + 1e-12 * abs(value), "noise bounds differ")
        after_error = matrix_error(report["denoised"]["expression"], after, 22)
    else:
        check("denoised" not in report, "disabled denoise changed the old report contract")
        after_error = matrix_error(report["expression"], after, 14)
    return dict(genes=len(genes), cells=len(cells), trees=trees,
                max_before_scaled_error=before_error, max_after_scaled_error=after_error)


def identities(root, paths):
    def rows(name):
        with owned_file(root, paths[name]).open() as source:
            return list(csv.DictReader(source, delimiter="\t"))
    def coordinate(value):
        decimal = Decimal(value)
        check(decimal.is_finite() and decimal == decimal.to_integral_value(), "noninteger coordinate")
        return int(decimal)
    genes = [dict(id=r["gene"], chromosome=r["chr"], start=coordinate(r["start"]),
                  end=coordinate(r["stop"])) for r in rows("genes")]
    cells = [dict(id=r["cell"], group=r["group"]) for r in rows("cells")]
    return genes, cells


def run_comparison(synthetic, shipped, witness, binary, output):
    synthetic, shipped, witness, binary = [Path(p).resolve(strict=True) for p in (synthetic, shipped, witness, binary)]
    output = Path(output).absolute()
    output.mkdir(parents=True, exist_ok=False)
    record = json.loads((witness / "witness.json").read_text())
    check(record["status"] == "complete" and record["schema_version"] == 1
          and len(record["cases"]) == 15, "incomplete installed-package evidence")
    for name, digest in record["files"].items():
        check(file_sha256(owned_file(witness, name)) == digest, "installed-package export changed")
    binary_hash = file_sha256(binary)
    results = []
    for case_id, case in record["cases"].items():
        check(case["status"] == "success", "failed installed-package case")
        genes, cells = identities(witness, case["before"])
        before = read_canonical(owned_file(witness, case["before"]["expression"]), len(cells))
        after = read_canonical(owned_file(witness, case["after"]["expression"]), len(cells))
        bundle = {"synthetic": synthetic, "shipped": shipped}[case["inputs"]["bundle"]]
        inputs = [owned_file(bundle, case["inputs"][name]) for name in ("counts", "positions", "annotations")]
        parameters = case["native"]
        baseline = None
        for threads in (1, 4):
            prefix = output / f"{case_id}-threads-{threads}"
            report_path = prefix.with_suffix(".report.json")
            command = [str(binary), "cnv", "samples", "--counts", str(inputs[0]),
                       "--positions", str(inputs[1]), "--annotations", str(inputs[2]),
                       "--output", str(report_path), "--include-expression", "--threads", str(threads)]
            for reference in case["inputs"]["references"]:
                command += ["--reference", reference]
            for name in ("cutoff", "min_cells_per_gene", "window_length", "max_centered_threshold"):
                command += ["--" + name.replace("_", "-"), str(parameters[name])]
            if parameters["reference_mean"]:
                command += ["--reference-mean"]
            if case["mode"]["enabled"]:
                command += ["--denoise"]
                if case["mode"]["fixed"] is not None:
                    command += ["--noise-filter", str(case["mode"]["fixed"])]
                if case["mode"]["logistic"]:
                    command += ["--noise-logistic"]
            time_path = prefix.with_suffix(".time.tsv")
            run_child(["/usr/bin/time", "-f", TIME_FORMAT, "-o", str(time_path), *command], prefix, timeout=300)
            report = json.loads(report_path.read_text())
            result = compare_report(report, case, before, after, genes, cells)
            fingerprint = report_fingerprint(report, threads)
            check(baseline is None or fingerprint == baseline, "native reports differ across workers")
            baseline = fingerprint
            results.append(dict(case=case_id, threads=threads, **result,
                                **_parse_time_file(time_path.read_text()), report_fingerprint=fingerprint,
                                report_sha256=file_sha256(report_path)))
            print(f"matched actual raw workflow {case_id}: workers={threads}, "
                  f"genes={result['genes']}, max_scaled_error={result['max_after_scaled_error']:.6g}", flush=True)
    check(file_sha256(binary) == binary_hash, "native binary changed")
    summary = dict(status="complete", cases=15, native_processes=len(results), binary_sha256=binary_hash,
                   witness_sha256=file_sha256(witness / "witness.json"), results=results,
                   scope="actual raw-to-samples/denoise compatibility; not a paired performance or HMM result")
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("synthetic", "shipped", "witness", "binary", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    run_comparison(args.synthetic, args.shipped, args.witness, args.binary, args.output)


if __name__ == "__main__":
    main()
