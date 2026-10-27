#!/usr/bin/env python3
"""Measure the accepted complete RNA-seq report against five actual source CLIs."""

import argparse
import csv
import importlib.metadata
import json
import math
import os
from pathlib import Path
import platform
import runpy
import stat
import statistics
import sys
import sysconfig
import zipfile

from compare_rnaseq_qc_workflow import METRICS, check, compare_report, verify_outputs
from infercnv_trial_process import TrialProcessError, run_child
from rnaseq_qc_benchmark_inputs import prepare_inputs
from rnaseq_qc_oracle import SCRIPTS, parse_coverage, parse_stdout, sha256
from verify_infercnv_oracle_artifact import _archive_name, _artifact, _extract, _run


NATIVE = dict(repository="omics-rust/rsomics-world", run_id=40546148508, run_attempt=1,
              head_sha="1224fbf18d7af1b4b76869fc6e41b1611d7bb43b",
              workflow_path=".github/workflows/rnaseq-qc-candidate.yml", artifact_id=12751813613,
              artifact_name="rnaseq-qc-40546148508-1-linux-x86_64",
              artifact_sha256="3e8130c10199aa0828a63ba964709cef9810713facc0d9e1bd3751e96d967d11")
BINARY_SHA = "ae968a1a490433086b6e50867a15b9797bea47e903faf032967b5a3b377fa840"
WHEEL_SHA = "a1fca6d455886b19bc2d58aa1e13b233977a2c835f676ef93bd0c6f4099b8452"
SOURCE_SHA = "8c9d34c208bf7852f147dd365312a43a546148d1bf76581c5c0ede73fdb254b8"
TIME_FORMAT = ('{"wall_seconds":%e,"user_seconds":%U,"system_seconds":%S,"max_rss_kib":%M,'
               '"filesystem_inputs":%I,"filesystem_outputs":%O,"major_faults":%F,"minor_faults":%R,"exit_status":%x}')


def save(path, value):
    with Path(path).open("x") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")


def source_acquisition(acquisition):
    return Path(acquisition) / "artifact/oracle-acquisition/source/acquisition"


def extract_native(archive, output):
    files, seen = set(), set()
    for member in archive.infolist():
        name = _archive_name(member.filename, member.is_dir())
        kind = stat.S_IFMT(member.external_attr >> 16) if member.create_system == 3 else 0
        check(member.orig_filename == member.filename and not member.flag_bits & 1
              and kind in (0, stat.S_IFDIR if member.is_dir() else stat.S_IFREG), "invalid native ZIP member")
        check(name not in seen, "duplicate native ZIP member")
        seen.add(name)
        if not member.is_dir():
            files.add(name)
    check(all('/'.join(name.split('/')[:index]) not in files
              for name in seen for index in range(1, len(name.split('/')))), "native ZIP path collision")
    check("comparison/summary.json" in files and archive.testzip() is None, "native ZIP lacks complete evidence or has bad CRC")
    _extract(archive, output)


def prepare_runtime(acquisition):
    acquisition = Path(acquisition).resolve(strict=True)
    archive = acquisition / "artifact.zip"
    _run(acquisition / "run.json", NATIVE)
    _artifact(acquisition / "artifact.json", NATIVE, archive)
    check(sha256(archive) == NATIVE["artifact_sha256"], "native ZIP differs from accepted run")
    with zipfile.ZipFile(archive) as zipped:
        extract_native(zipped, acquisition / "artifact")
    root = acquisition / "artifact"
    for name, digest in {
        "source.tar.gz": "a451612a20af21f1d73e15fadb60f039f18ef9c613a16a1652ba3f7c9177ddb0",
        "files.sha256": "a7449cb79fc889a3d2fcc4fd6fb8a56813d2c21a70d91481d2332af55f50a562",
        "Cargo.lock": "c09dea6b5397ca673d7052042d403fa55d2663e45c99672dc6f5ea6f51f308e2",
        "native-binary/rsomics-rnaseq-qc-linux-x86_64": BINARY_SHA,
    }.items():
        check(sha256(root / name) == digest, f"accepted native identity differs: {name}")
    summary = json.loads((root / "comparison/summary.json").read_text())
    check(summary["status"] == "complete" and summary["binary_sha256"] == BINARY_SHA
          and [(row["case"], row["workers"]) for row in summary["results"]]
          == [(case, workers) for case in ("single", "paired") for workers in (1, 4)],
          "accepted native workflow is incomplete")
    source = source_acquisition(acquisition)
    wheel = source / "rseqc-5.0.5-py3-none-any.whl"
    check(sha256(wheel) == WHEEL_SHA and sha256(source / "rseqc-source.tar.gz") == SOURCE_SHA,
          "accepted source package differs")
    lines = (source / "installed-freeze.txt").read_text().splitlines()
    packages = dict(line.split("==", 1) for line in lines if "==" in line)
    check(len(packages) == len(lines) - 1 and sum(line.startswith("RSeQC @ file:") for line in lines) == 1,
          "accepted source dependency freeze is not fully pinned")
    with (acquisition / "requirements.lock.txt").open("x") as stream:
        stream.write("\n".join([line for line in lines if "==" in line] + [str(wheel)]) + "\n")
    binary = root / "native-binary/rsomics-rnaseq-qc-linux-x86_64"
    binary.chmod(0o755)
    save(acquisition / "verification.json", dict(native=NATIVE, binary_sha256=BINARY_SHA,
                                                wheel_sha256=WHEEL_SHA, source_sha256=SOURCE_SHA,
                                                dependencies=packages))
    return binary


def verify_installed_source(acquisition):
    record = json.loads((Path(acquisition) / "verification.json").read_text())
    for name, version in dict(record["dependencies"], RSeQC="5.0.5").items():
        check(importlib.metadata.version(name) == version, f"installed dependency differs: {name}")
    installed, identities = Path(sysconfig.get_path("scripts")), {}
    with zipfile.ZipFile(source_acquisition(acquisition) / "rseqc-5.0.5-py3-none-any.whl") as wheel:
        for name in SCRIPTS.values():
            member = f"rseqc-5.0.5.data/scripts/{name}"
            packaged, actual = wheel.read(member), (installed / name).read_bytes()
            normalize = lambda value: value.split(b"\n", 1)[1] if value.startswith(b"#!") else value
            check(normalize(packaged) == normalize(actual), f"installed source script differs: {name}")
            identities[name] = sha256(installed / name)
        modules = [name for name in wheel.namelist() if name.startswith("qcmodule/") and name.endswith(".py")]
        check(len(modules) >= 7, "source QC modules missing from wheel")
        for name in modules:
            check(wheel.read(name) == (Path(sysconfig.get_path("purelib")) / name).read_bytes(),
                  f"installed source module differs: {name}")
    return {metric: installed / script for metric, script in SCRIPTS.items()}, dict(
        python=sys.version, scripts=identities, dependencies=dict(record["dependencies"], RSeQC="5.0.5"))


def trace_eligibility(script, inputs, output):
    import pysam

    source = runpy.run_path(str(script), run_name="rsomics_untimed_tin_eligibility")
    rows = []
    with pysam.AlignmentFile(inputs["bam"], "rb") as bam:
        for name, chrom, start, end, _, positions in source["genomic_positions"](Path(inputs["genes"]), 100):
            rows.append(dict(geneID=name, chrom=chrom, tx_start=start, tx_end=end,
                             sampled_positions=len(positions),
                             eligible=source["check_min_reads"](bam, chrom, start, end, 10)))
    save(output, dict(scope="untimed original source function calls, no numeric-score interception", rows=rows))
    check(any(row["eligible"] for row in rows), "real source has no eligible TIN population")
    return rows


def integrity_tables(tin, summary, eligibility):
    with Path(tin).open() as stream:
        rows = list(csv.DictReader(stream, delimiter="\t"))
    with Path(summary).open() as stream:
        summaries = list(csv.DictReader(stream, delimiter="\t"))
    check(len(rows) == len(eligibility) and len(summaries) == 1, "actual TIN table population differs")
    for row, trace in zip(rows, eligibility):
        check(row["geneID"] == trace["geneID"] and row["chrom"] == trace["chrom"]
              and int(row["tx_start"]) == trace["tx_start"] and int(row["tx_end"]) == trace["tx_end"],
              "actual source eligibility trace identity/order differs")
        check(math.isfinite(float(row["TIN"])), "nonfinite actual TIN score")
        row.update(eligible=trace["eligible"], sampled_positions=trace["sampled_positions"])
    return dict(transcripts=rows,
                eligible_transcripts=[row["geneID"] for row in eligibility if row["eligible"]],
                summary={key: float(summaries[0][column]) for key, column in
                         (("mean", "TIN(mean)"), ("median", "TIN(median)"), ("stdev", "TIN(stdev)"))})


def read_source_report(directory, eligibility):
    values = {metric: parse_stdout(metric, (directory / metric / "process.stdout").read_text(), "paired")
              for metric in METRICS[:3]}
    values["coverage"] = parse_coverage((directory / "coverage/coverage.geneBodyCoverage.txt").read_text())
    values["integrity"] = integrity_tables(directory / "integrity/paired.tin.xls",
                                           directory / "integrity/paired.summary.txt", eligibility)
    return values


def measured(command, directory, cores):
    usage = directory / "time.json"
    argv = ["/usr/bin/time", "-f", TIME_FORMAT, "-o", str(usage), "taskset", "-c", cores, *map(str, command)]
    child = run_child(argv, directory / "process", timeout=2400)
    row = json.loads(usage.read_text())
    check(row["exit_status"] == 0 and all(type(value) in (int, float) and math.isfinite(value)
                                         and value >= 0 for value in row.values()), "invalid GNU time evidence")
    row.update(argv=list(map(str, command)), owner_wall_seconds=(child.end_ns - child.start_ns) / 1e9)
    save(directory / "measurement.json", row)
    return row


def aggregate_source(rows):
    sums = ("wall_seconds", "user_seconds", "system_seconds", "filesystem_inputs", "filesystem_outputs",
            "major_faults", "minor_faults", "owner_wall_seconds")
    return {**{name: sum(row[name] for row in rows) for name in sums},
            "max_rss_kib": max(row["max_rss_kib"] for row in rows), "scripts": rows}


def source_trial(directory, inputs, tools, cores, eligibility):
    directory.mkdir()
    rows = []
    for metric, tool in tools.items():
        own = directory / metric
        own.mkdir()
        args = ["-i", inputs["bam"]]
        if metric != "mapping":
            args += ["-r", inputs["genes"]]
        if metric in ("mapping", "strandedness"):
            args += ["-q", "30"]
        if metric == "strandedness":
            args += ["-s", "200000"]
        if metric == "coverage":
            args += ["-l", "100", "--skip-plot", "-o", str(own / "coverage")]
        if metric == "integrity":
            args += ["-c", "10", "-n", "100", "-o", str(own)]
        previous = Path.cwd()
        try:
            os.chdir(own)
            rows.append(measured([sys.executable, "-B", tool, *args], own, cores))
        finally:
            os.chdir(previous)
        print(f"{directory.name}: source {metric} complete", flush=True)
    result = aggregate_source(rows)
    values = read_source_report(directory, eligibility)
    save(directory / "metrics.json", values)
    save(directory / "measurement.json", result)
    return values, result


def native_trial(directory, inputs, binary, cores, workers, expected):
    directory.mkdir()
    output = directory / "report"
    command = [binary, "report", "--bam", inputs["bam"], "--genes", inputs["genes"], "--out", output,
               "--profile", "rseqc-5.0.5", "--threads", str(workers)]
    result = measured(command, directory, cores)
    report = json.loads((output / "report.json").read_text())
    comparison = compare_report(report, expected, "paired")
    for row, reference in zip(report["integrity"]["transcripts"], expected["integrity"]["transcripts"]):
        check(row["eligible"] == reference["eligible"] and row["sampled_positions"] == reference["sampled_positions"],
              "native TIN eligibility or original sampling-list denominator differs")
    verify_outputs(output, "paired", inputs["records"])
    check(comparison["mapping_records"] == inputs["records"], "report original-record count differs")
    save(directory / "comparison.json", comparison)
    result.update(workers=workers, report_sha256=sha256(output / "report.json"))
    print(f"{directory.name}: complete native report matches actual source", flush=True)
    return report, result


def distribution(rows):
    result = {}
    for name in ("wall_seconds", "user_seconds", "system_seconds", "max_rss_kib",
                 "filesystem_inputs", "filesystem_outputs", "major_faults", "minor_faults"):
        values = [row[name] for row in rows]
        result[name] = dict(values=values, minimum=min(values), maximum=max(values), median=statistics.median(values))
    return result


def performance_decision(stats):
    source_wall, serial_wall = stats["source"]["wall_seconds"], stats["native"]["1"]["wall_seconds"]
    source_rss, serial_rss = stats["source"]["max_rss_kib"], stats["native"]["1"]["max_rss_kib"]
    throughput = source_wall["minimum"] > serial_wall["maximum"]
    memory = source_rss["minimum"] > serial_rss["maximum"]
    return dict(advantage_passed=throughput or memory, strict_throughput=throughput,
                strict_process_peak_rss=memory,
                median_serial_speedup=source_wall["median"] / serial_wall["median"],
                median_four_worker_speedup=serial_wall["median"] / stats["native"]["4"]["wall_seconds"]["median"])


def run_benchmark(acquisition, profile, output):
    check(platform.system() == "Linux" and platform.machine() == "x86_64", "benchmark requires native Linux x86_64")
    allowed = sorted(os.sched_getaffinity(0))
    check(len(allowed) >= 4, "four available CPUs are required for the declared worker comparison")
    cores = ",".join(map(str, allowed[:4]))
    os.sched_setaffinity(0, allowed[:4])
    for name in ("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS", "NUMEXPR_NUM_THREADS"):
        os.environ[name] = "1"
    output = Path(output).absolute()
    output.mkdir(parents=True)
    tools, runtime = verify_installed_source(acquisition)
    binary = Path(acquisition).resolve() / "artifact/native-binary/rsomics-rnaseq-qc-linux-x86_64"
    check(sha256(binary) == BINARY_SHA, "measured binary differs from accepted four-native candidate")
    save(output / "host.json", dict(platform=platform.platform(), machine=platform.machine(),
                                   cpu_affinity=list(os.sched_getaffinity(0)), cores=cores,
                                   cpuinfo=Path("/proc/cpuinfo").read_text(), load_average=os.getloadavg(),
                                   time_format=TIME_FORMAT, runtime=runtime, native=NATIVE, binary_sha256=BINARY_SHA))
    inputs = prepare_inputs(output / "inputs", profile)
    save(output / "inputs.json", inputs)
    eligibility = trace_eligibility(tools["integrity"], inputs, output / "source-eligibility.json")
    expected, _ = source_trial(output / "source-warmup", inputs, tools, cores, eligibility)
    baseline, _ = native_trial(output / "native-warmup-1", inputs, binary, cores, 1, expected)
    report, _ = native_trial(output / "native-warmup-4", inputs, binary, cores, 4, expected)
    check(report == baseline, "complete warmup report differs across worker counts")
    source, native = [], {1: [], 4: []}
    for index in range(5):
        if index < 3:
            values, row = source_trial(output / f"source-{index + 1}", inputs, tools, cores, eligibility)
            compare_report(baseline, values, "paired")
            source.append(row)
        for workers in ((1, 4) if index % 2 == 0 else (4, 1)):
            report, row = native_trial(output / f"native-{index + 1}-workers-{workers}", inputs,
                                       binary, cores, workers, expected)
            check(report == baseline, "complete native report changed across trials/workers")
            native[workers].append(row)
    check(sha256(binary) == BINARY_SHA, "measured binary changed")
    stats = dict(source=distribution(source), native={str(workers): distribution(rows) for workers, rows in native.items()})
    decision = performance_decision(stats)
    status = "complete" if decision["advantage_passed"] else "rejected_no_advantage"
    summary = dict(status=status, profile=profile, binary_sha256=BINARY_SHA,
                   inputs_sha256=sha256(output / "inputs.json"), report_sha256=sha256(output / "native-warmup-1/report/report.json"),
                   source_trials=source, native_trials=native, statistics=stats, decision=decision,
                   scope="warm-cache complete five-script source versus shared native CLI; startup and numeric outputs included; "
                         "source RSS is maximum individual script process, I/O values are GNU-time filesystem blocks, not logical bytes; "
                         "not aggregate fork memory, cold-I/O, high-depth or biological equivalence; chr21 does not substitute for whole-input evidence")
    save(output / "summary.json", summary)
    check(decision["advantage_passed"], "complete workflow shows no strict replacement advantage")
    print(json.dumps(dict(profile=profile, decision=decision, status="complete")), flush=True)
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    runtime = sub.add_parser("prepare-runtime")
    runtime.add_argument("--acquisition", type=Path, required=True)
    run = sub.add_parser("run")
    run.add_argument("--acquisition", type=Path, required=True)
    run.add_argument("--output", type=Path, required=True)
    run.add_argument("--profile", choices=("chr21", "whole"), required=True)
    args = parser.parse_args()
    try:
        if args.command == "prepare-runtime":
            prepare_runtime(args.acquisition)
        else:
            run_benchmark(args.acquisition, args.profile, args.output)
    except (OSError, ValueError, KeyError, TrialProcessError) as error:
        parser.exit(1, f"RNA-seq QC real complete-report measurement failed: {error}\n")


if __name__ == "__main__":
    main()
