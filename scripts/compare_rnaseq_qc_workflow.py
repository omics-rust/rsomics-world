#!/usr/bin/env python3
"""Compare the complete shared report with retained actual RSeQC5.0.5 calls."""

import argparse
import json
import math
from pathlib import Path

from infercnv_trial_process import TrialProcessError, run_child
from rnaseq_qc_oracle import parse_coverage, parse_integrity, parse_stdout, sha256
from validate_infercnv_shipped_oracle import owned_file


METRICS = ("mapping", "strandedness", "distribution", "coverage", "integrity")
SOURCE_COMMIT = "0d2e017723c5c4b41eb3b938e1631692d187de1f"


def check(condition, message):
    if not condition:
        raise ValueError(message)


def close(actual, expected, name):
    check(type(actual) in (float, int) and math.isfinite(actual)
          and abs(actual - float(expected)) <= 1e-9 + 1e-9 * abs(float(expected)),
          f"{name} differs: {actual!r} != {expected!r}")


def expected_metrics(root, profile):
    case = root / "cases" / profile
    values = {metric: parse_stdout(metric, (case / metric / "process.stdout").read_text(), profile)
              for metric in ("mapping", "strandedness", "distribution")}
    values["coverage"] = parse_coverage((case / "coverage/coverage.geneBodyCoverage.txt").read_text())
    values["integrity"] = parse_integrity((case / f"integrity/{profile}.tin.xls").read_text(),
                                        (case / f"integrity/{profile}.summary.txt").read_text())
    return values


def compare_report(report, expected, profile):
    check(report["schema_version"] == "1.0" and report["profile"] == "rseqc-5.0.5"
          and report["sample"] == profile and report["metrics"] == list(METRICS),
          "complete report identity/profile/selection differs")
    check(report["mapping"] == expected["mapping"], "mapping categories or exclusive precedence differ")
    actual, source = report["strandedness"], expected["strandedness"]
    check(actual["layout"] == source["layout"] and actual["fractions"].keys() == source["fractions"].keys(),
          "strandedness layout or fraction labels differ")
    for label, value in source["fractions"].items():
        observed = actual["fractions"][label]
        check(type(observed) in (float, int) and math.isfinite(observed) and 0 <= observed <= 1
              and f"{observed:.4f}" == f"{value:.4f}", "strandedness source-rounded fraction differs")
    actual, source = report["distribution"], expected["distribution"]
    check(actual["totals"] == source["totals"] and actual["groups"].keys() == source["groups"].keys(),
          "distribution totals or categories differ")
    for label, value in source["groups"].items():
        row = actual["groups"][label]
        check(row["bases"] == value["bases"] and row["tags"] == value["tags"],
              f"distribution bases or tags differ: {label}")
        check(type(row["tags_per_kb"]) in (float, int) and math.isfinite(row["tags_per_kb"])
              and f"{row['tags_per_kb']:.2f}" == f"{value['tags_per_kb']:.2f}",
              f"distribution source-rounded density differs: {label}")
    actual, source = report["coverage"], expected["coverage"]
    check(actual["sample"] == source["sample"] and actual["coverage"] == source["coverage"],
          "raw100-point coverage differs")
    actual, source = report["integrity"], expected["integrity"]
    check(actual["eligible_transcripts"] == source["eligible_transcripts"]
          and len(actual["transcripts"]) == len(source["transcripts"]), "TIN population differs")
    for row, reference in zip(actual["transcripts"], source["transcripts"]):
        check(row["geneID"] == reference["geneID"] and row["chrom"] == reference["chrom"]
              and row["tx_start"] == int(reference["tx_start"])
              and row["tx_end"] == int(reference["tx_end"]), "TIN transcript identity/order differs")
        close(row["TIN"], reference["TIN"], f"TIN {reference['geneID']}")
    for name in ("mean", "median", "stdev"):
        close(actual["summary"][name], source["summary"][name], f"TIN summary {name}")
    return dict(mapping_records=report["mapping"]["Total records:"],
                coverage_points=len(report["coverage"]["coverage"]),
                coverage_sum=sum(report["coverage"]["coverage"]),
                tin_transcripts=len(actual["transcripts"]),
                eligible_transcripts=len(actual["eligible_transcripts"]))


def verify_outputs(directory, profile, records):
    manifest = json.loads((directory / "manifest.json").read_text())
    check(manifest["status"] == "complete" and manifest["profile"] == "rseqc-5.0.5"
          and manifest["selected_metrics"] == list(METRICS)
          and manifest["bam_traversals"] == 1 and manifest["original_records"] == records,
          "report manifest does not describe one complete shared scan")
    expected = {"report.json", "mapping.tsv", "strandedness.tsv", "distribution.tsv",
                "distribution-totals.tsv", "coverage.tsv", "integrity.tsv", "integrity-summary.tsv"}
    check({entry["path"] for entry in manifest["outputs"]} == expected
          and len(manifest["outputs"]) == len(expected), "transactional output set differs")
    for entry in manifest["outputs"]:
        path = owned_file(directory, entry["path"])
        check(path.stat().st_size == entry["bytes"] and sha256(path) == entry["sha256"],
              f"manifest output identity differs: {entry['path']}")


def run_comparison(oracle, binary, output):
    oracle, binary = (Path(path).resolve(strict=True) for path in (oracle, binary))
    output = Path(output).absolute()
    output.mkdir(parents=True, exist_ok=False)
    witness = json.loads((oracle / "witness.json").read_text())
    check(witness["schema_version"] == 1 and witness["status"] == "complete"
          and witness["source_commit"] == SOURCE_COMMIT and not witness["failed_cases"]
          and witness["package"]["package"] == "RSeQC" and witness["package"]["version"] == "5.0.5"
          and list(witness["cases"]) == ["single", "paired"], "incomplete or wrong actual source witness")
    for name, digest in witness["files"].items():
        check(sha256(owned_file(oracle, name)) == digest, f"actual source evidence changed: {name}")
    binary_digest = sha256(binary)
    results = []
    for profile, case in witness["cases"].items():
        check(case["status"] == "complete" and set(case["metrics"]) == set(METRICS)
              and all(row["status"] == "success" for row in case["metrics"].values()),
              "actual five-metric case is incomplete")
        expected = expected_metrics(oracle, profile)
        inputs = oracle / "cases" / profile / "inputs"
        baseline = None
        for workers in (1, 4):
            name = f"{profile}-workers-{workers}"
            directory = output / name
            argv = [str(binary), "report", "--bam", str(inputs / f"{profile}.bam"),
                    "--genes", str(inputs / "genes.bed12"), "--out", str(directory),
                    "--profile", "rseqc-5.0.5", "--threads", str(workers)]
            child = run_child(argv, output / (name + ".trial"), timeout=300)
            report = json.loads((directory / "report.json").read_text())
            result = compare_report(report, expected, profile)
            verify_outputs(directory, profile, case["alignment_records"])
            check(baseline is None or baseline == report, "shared-report metrics differ across worker counts")
            baseline = report
            results.append(dict(case=profile, workers=workers, **result,
                                owner_wall_seconds=(child.end_ns - child.start_ns) / 1e9,
                                report_sha256=sha256(directory / "report.json")))
            print(f"matched actual five-metric shared report: {profile}, workers={workers}", flush=True)
    check(sha256(binary) == binary_digest, "candidate binary changed during comparison")
    summary = dict(status="complete", binary_sha256=binary_digest,
                   witness_sha256=sha256(oracle / "witness.json"), results=results,
                   scope="bounded actual five-metric synthetic semantics; not general pileup-cap parity or performance")
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("oracle", "binary", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    try:
        run_comparison(args.oracle, args.binary, args.output)
    except (OSError, ValueError, KeyError, TrialProcessError) as error:
        parser.exit(1, f"RNA-seq QC complete workflow comparison failed: {error}\n")


if __name__ == "__main__":
    main()
