#!/usr/bin/env python3
"""Compare a native raw-to-Bayesian workflow with two real JAGS repetitions."""

import argparse
import csv
import json
import math
from pathlib import Path
import statistics

from compare_infercnv_denoise_workflow import check, close, identities
from compare_infercnv_hmm_workflow import compare_report as compare_hmm
from compare_infercnv_hmm_workflow import integer, native_command, read_reports, timed_native
from measure_infercnv_samples_cli import report_fingerprint
from validate_infercnv_shipped_oracle import file_sha256, owned_file, read_canonical


MIXTURE_FLOOR = .002
MC_STANDARD_ERRORS = 6
CELL_TOLERANCE = .05


def probabilities(values):
    check(len(values) == 6 and all(math.isfinite(value) and 0 <= value <= 1 for value in values)
          and abs(sum(values) - 1) < 1e-10, "invalid six-state posterior probabilities")


def compare_bayesian(report, reference, uncertainty, states, tables):
    result = report["bayesian"]
    check((result["step"], result["model"], result["sampler"], result["applied"],
           result["chains"], result["burn_in"], result["samples_per_chain"])
          == (19, "i6-shared-dirichlet-normal", "collapsed-gibbs", True, 6, 700, 1000),
          "Bayesian model or sampling protocol differs")
    originals = report["hmm"]["groups"]
    regions = {(group_index, region_index): (group, region)
               for group_index, group in enumerate(originals)
               for region_index, region in enumerate(group["regions"])}
    check(len(result["posteriors"]) == len(regions) == len(reference["regions"]),
          "posterior region count differs")
    threshold = report["settings"]["bayesian"]["max_normal_probability"]
    expected_states = [group["states"][:] for group in originals]
    expected_regions = [[] for _ in originals]
    maximum = cell_maximum = 0.0
    removed = 0
    seen = set()
    for posterior in result["posteriors"]:
        key = posterior["group_index"], posterior["region_index"]
        check(key in regions and key not in seen, "posterior original-region identity differs")
        seen.add(key)
        group, region = regions[key]
        name = region["name"]
        check(name in reference["regions"], "native region missing from actual Bayesian operation")
        expected = reference["regions"][name]
        check(expected["original_state"] == region["state"], "original CNV state differs")
        mixture = posterior["probabilities"]
        chains = posterior["chain_probabilities"]
        probabilities(mixture)
        check(len(chains) == 6, "native Bayesian chain count differs")
        for chain in chains:
            probabilities(chain)
        for state, value in enumerate(mixture, 1):
            means = [chain[state - 1] for chain in chains]
            close(statistics.mean(means), value, 1e-12, "native mixture and chain means disagree")
            diagnostic = uncertainty[(name, state)]
            oracle = reference["probabilities"][name][state - 1]
            close(diagnostic["mean"], oracle, 1e-12, "reduced samples and actual posterior table disagree")
            standard_error = diagnostic["standard_error"]
            check(math.isfinite(standard_error) and standard_error >= 0, "invalid actual batch MCSE")
            tolerance = MIXTURE_FLOOR + MC_STANDARD_ERRORS * math.sqrt(
                standard_error ** 2 + statistics.variance(means) / len(means))
            maximum = max(maximum, close(value, oracle, tolerance, f"mixture posterior differs for {name}/{state}"))
        members = group["member_indices"]
        check(len(posterior["cell_probabilities"]) == len(members), "posterior cell count differs")
        for cell, values in zip(members, posterior["cell_probabilities"]):
            probabilities(values)
            for value, oracle in zip(values, reference["cells"][(name, cell + 1)]):
                cell_maximum = max(cell_maximum, close(value, oracle, CELL_TOLERANCE,
                                                     f"cell posterior differs for {name}/{cell}"))
        filtered = mixture[2] > threshold
        final = 3 if filtered else max(range(6), key=mixture.__getitem__) + 1
        check(posterior["removed"] == filtered == expected["removed"]
              and posterior["final_state"] == final == expected["final_state"],
              f"actual filtering or reassignment differs for {name}")
        removed += filtered
        start, end = region["gene_range"]
        expected_states[key[0]][start:end] = [final] * (end - start)
        if not filtered:
            expected_regions[key[0]].append(dict(region, state=final))
    check(len(result["groups"]) == len(originals), "filtered HMM group count differs")
    check(states.genes == tuple(gene["id"] for gene in report["genes"])
          and states.cells == tuple(cell["id"] for cell in report["cells"]),
          "actual filtered matrix identity differs")
    projected = dict(pred_cnv_regions=[], pred_cnv_genes=[])
    for index, (original, actual) in enumerate(zip(originals, result["groups"])):
        check(actual["name"] == original["name"] and actual["member_indices"] == original["member_indices"]
              and actual["states"] == expected_states[index] and actual["regions"] == expected_regions[index],
              "filtered groups, paths or original region identities differ")
        for gene, row in enumerate(states.rows):
            check(all(row[cell] == actual["states"][gene] for cell in actual["member_indices"]),
                  f"actual Bayesian state matrix differs for {actual['name']} at gene {gene}")
        for region in actual["regions"]:
            projected["pred_cnv_regions"].append((actual["name"], region["name"], region["state"],
                                                   region["chromosome"], region["start"], region["end"]))
            start, end = region["gene_range"]
            projected["pred_cnv_genes"].extend((actual["name"], region["name"], region["state"],
                                                 gene["id"], gene["chromosome"], gene["start"], gene["end"])
                                                for gene in report["genes"][start:end])
    for name, rows in projected.items():
        check(rows == tables[name], f"actual adjusted {name} report differs")
    return dict(regions=len(regions), removed=removed, retained=len(regions) - removed,
                max_posterior_scaled_error=maximum, max_cell_scaled_error=cell_maximum)


def rows(root, path):
    with owned_file(root, path).open(newline="") as source:
        return list(csv.DictReader(source, delimiter="\t"))


def read_reference(root, case):
    regions = {}
    for row in rows(root, case["posterior_regions"]):
        name = row["region"]
        check(name not in regions, "duplicate actual posterior region")
        regions[name] = dict(original_state=integer(row["original_state"]))
    for row in rows(root, case["posterior_and_filter_record"].replace("posterior-and-filter-record.rds", "filter-regions.tsv")):
        check(row["removed"] in ("TRUE", "FALSE"), "invalid actual filtering flag")
        regions[row["region"]].update(removed=row["removed"] == "TRUE", final_state=integer(row["final_state"]))
    result = dict(regions=regions, probabilities={name: [None] * 6 for name in regions}, cells={})
    for row in rows(root, case["posterior_state_probabilities"]):
        result["probabilities"][row["region"]][integer(row["state"]) - 1] = float(row["probability"])
    for row in rows(root, case["posterior_cell_probabilities"]):
        key = row["region"], integer(row["cell_index"])
        result["cells"].setdefault(key, [None] * 6)[integer(row["state"]) - 1] = float(row["probability"])
    for values in [*result["probabilities"].values(), *result["cells"].values()]:
        probabilities(values)
    return result


def read_uncertainty(root, case_id, case):
    paths = {index + 1: sample["region"] for index, sample in enumerate(case["samples"])}
    result = {}
    for row in rows(root, case_id + ".tsv"):
        if not row["variable"].startswith("theta["):
            continue
        state = integer(row["state"])
        check(row["variable"] == f"theta[{state}]", "actual theta variable identity differs")
        key = paths[integer(row["region_index"])], state
        check(key not in result, "duplicate actual theta diagnostic")
        result[key] = dict(mean=float(row["posterior_mean"]), standard_error=float(row["batch_standard_error"]))
    check(len(result) == 6 * len(paths), "actual theta diagnostics incomplete")
    return result


def run_comparison(shipped, hmm, bayes, uncertainty, binary, output):
    shipped, hmm, bayes, uncertainty, binary = [Path(path).resolve(strict=True)
                                               for path in (shipped, hmm, bayes, uncertainty, binary)]
    output = Path(output).absolute()
    output.mkdir(parents=True, exist_ok=False)
    hmm_record = json.loads((hmm / "witness.json").read_text())
    record = json.loads((bayes / "witness.json").read_text())
    diagnostics = json.loads((uncertainty / "manifest.json").read_text())
    check(record["status"] == "complete" and record["schema_version"] == 1
          and list(record["cases"]) == ["full_repeat1", "full_repeat2"]
          and record["source_hmm_witness_sha256"] == file_sha256(hmm / "witness.json")
          and record["package"] == hmm_record["package"], "actual Bayesian replay/source identity differs")
    check(diagnostics["source_witness_sha256"] == file_sha256(bayes / "witness.json")
          and diagnostics["reducer_sha256"] == file_sha256(Path(__file__).with_name("infercnv_bayes_uncertainty.R")),
          "uncertainty reduction provenance differs")
    for root, files in [(hmm, hmm_record["files"]), (bayes, record["files"]), (uncertainty, diagnostics["files"])]:
        for name, digest in files.items():
            check(file_sha256(owned_file(root, name)) == digest, "retained Bayesian evidence changed")
    case = hmm_record["cases"]["full"]
    check(file_sha256(shipped / "sha256-manifest.tsv") == hmm_record["source_bundles"]["shipped"],
          "source shipped bundle differs")
    genes, cells = identities(hmm, case["expression"])
    expression = read_canonical(owned_file(hmm, case["expression"]["expression"]), len(cells))
    states = read_canonical(owned_file(hmm, case["states"]["expression"]), len(cells))
    spike_genes, spike_cells = identities(hmm, case["spike"])
    original_tables = read_reports(hmm, case["reports"])
    references = []
    for case_id, entry in record["cases"].items():
        check(entry["status"] == "success" and entry["BayesMaxPNormal"] == .5,
              "actual Bayesian workflow or threshold differs")
        paths = [path for path in entry["original_reports"]
                 if path.endswith((".cell_groupings", ".genes_used.dat"))] + entry["adjusted_reports"]
        final = read_canonical(owned_file(bayes, entry["final_hmm"]["expression"]), len(cells))
        references.append((case_id, read_reference(bayes, entry),
                           read_uncertainty(uncertainty, case_id, entry), final, read_reports(bayes, paths)))
    binary_hash = file_sha256(binary)
    baseline = None
    results = []
    for threads in (1, 4):
        prefix = output / f"full-threads-{threads}"
        report_path = prefix.with_suffix(".report.json")
        command = native_command(binary, shipped, case, hmm_record["seed"], threads, report_path)
        command += ["--bayes", "--bayes-seed", "261027", "--bayes-max-p-normal", "0.5"]
        metrics = timed_native(command, prefix)
        report = json.loads(report_path.read_text())
        original = compare_hmm(report, case, expression, states, genes, cells,
                               (len(spike_genes), len(spike_cells)), original_tables)
        comparisons = [dict(case=case_id, **compare_bayesian(report, reference, errors, final, tables))
                       for case_id, reference, errors, final, tables in references]
        fingerprint = report_fingerprint(report, threads)
        check(baseline is None or fingerprint == baseline, "native Bayesian reports differ across workers")
        baseline = fingerprint
        results.append(dict(threads=threads, original_hmm=original, comparisons=comparisons,
                            report_fingerprint=fingerprint, report_sha256=file_sha256(report_path), **metrics))
        print(f"matched actual Bayesian workflow: workers={threads}, regions={comparisons[0]['regions']}, "
              f"removed={comparisons[0]['removed']}, R_repetitions=2", flush=True)
    check(file_sha256(binary) == binary_hash, "native Bayesian binary changed")
    summary = dict(status="complete", native_processes=2, actual_R_repetitions=2,
                   binary_sha256=binary_hash, witness_sha256=file_sha256(bayes / "witness.json"), results=results,
                   tolerance=dict(mixture_floor=MIXTURE_FLOOR, MC_standard_errors=MC_STANDARD_ERRORS,
                                  cell_absolute=CELL_TOLERANCE),
                   scope="actual grouped-i6 Bayesian posterior/rectangle/report compatibility on one real fixture",
                   timing="native process observations; not paired R performance or biological validation")
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("shipped", "hmm", "bayes", "uncertainty", "binary", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    try:
        run_comparison(args.shipped, args.hmm, args.bayes, args.uncertainty, args.binary, args.output)
    except (OSError, ValueError, RuntimeError, KeyError) as error:
        parser.exit(1, f"Bayesian workflow comparison failed: {error}\n")


if __name__ == "__main__":
    main()
