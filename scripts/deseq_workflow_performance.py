#!/usr/bin/env python3
"""Measure the accepted pasilla count-to-export workflow without rebuilding it."""

import argparse
import csv
import importlib.util
import json
import math
import os
from pathlib import Path
import platform
import shutil
import stat
import statistics
import sys
import zipfile

from infercnv_source_snapshot import verify_and_extract
from infercnv_trial_process import TrialProcessError, run_child
from verify_infercnv_oracle_artifact import HEX64, _archive_name, _artifact, _extract, _load_json, _run, _sha256


ROOT = Path(__file__).resolve().parents[1]
NATIVE = dict(repository="omics-rust/rsomics-world", run_id=40611588271, run_attempt=1,
              head_sha="42f378fe633649b7234c5cedaf424d0d4ab62da5", workflow_path=".github/workflows/deseq-candidate.yml",
              artifact_id=12782868065, artifact_name="deseq-40611588271-1-linux-x86_64", size_in_bytes=325788430,
              artifact_sha256="037b9c76ecda311280ecadaebccce244da78e2490be28bf366c1bcd775d9ba9b")
BINARY_SHA = "630431e494e5fcba34eb175c7cfe4a68ff071afd249e29619e2385ab7c061059"
SOURCE = dict(schema_version=1, repository="omics-rust/rsomics-world", run_id=40561801963, run_attempt=1,
              workflow_path=".github/workflows/deseq-oracle.yml", head_sha="89ece7f45e59517cf7631e9c5931a48c4a7760c1",
              artifact_id=12761202566, artifact_name="deseq-oracle-40561801963-1", size_in_bytes=26172835,
              artifact_sha256="6adaf380cccd55829c0b7d4d022d1d405f31f8f4345971f10f7ab961a8b2e7a5",
              manifest_sha256="423e585319836ceb0b7c54a47bb2a42431e97dff9929301e9273195ddf3270f8",
              witness_sha256="8f2dc0a7b0377a22c2e1afda549003842577aa4dea24c9a675af5293bcb94bd7", bundle_path="workflow")
ARCHIVE_SHA = "f72b71c96bfed743ce13756ce6265f7745d8dadf1be2bacc8a8ee68d0c5ea8c8"
MANIFEST_SHA = "b75297f3621a8e18864daee09005fec0814dd8dd013f60cadd6b2951ebaddc1e"
LOCK_SHA = "501544e3615498b7f26c06bb5ea36c4d027b61f7366cce4a11b6efcbfc82e241"
SOURCE_ARCHIVE_SHA = "8c91699286336350e66eec132ce6fdf5bb4af78e2a4d015a5a61224f62a95984"
TARGETS = {"linux-x86_64", "linux-aarch64", "macos-x86_64", "macos-aarch64"}
OUTPUTS = ("forward.tsv", "reverse.tsv", "normalized-counts.tsv", "vst.tsv")
BOUNDS = dict(normalization=(1e-10, 1e-10), dispersion=(1e-6, 1e-4), coefficient=(1e-4, 1e-5),
              probability=(1e-7, 1e-4), mu=(1e-6, 1e-5), diagnostic=(1e-6, 1e-4), vst=(1e-5, 1e-6))
TIME_FORMAT = ('{"wall_seconds":%e,"user_seconds":%U,"system_seconds":%S,"max_rss_kib":%M,'
               '"filesystem_inputs":%I,"filesystem_outputs":%O,"major_faults":%F,"minor_faults":%R,"exit_status":%x}')


def check(condition, message):
    if not condition:
        raise ValueError(message)


def save(path, value):
    with Path(path).open("x") as stream:
        json.dump(value, stream, indent=2, allow_nan=False)
        stream.write("\n")


def extract_zip(path, output, required):
    with zipfile.ZipFile(path) as archive:
        files, seen = set(), set()
        for member in archive.infolist():
            name = _archive_name(member.filename, member.is_dir())
            kind = stat.S_IFMT(member.external_attr >> 16) if member.create_system == 3 else 0
            check(member.orig_filename == member.filename and not member.flag_bits & 1 and
                  kind in (0, stat.S_IFDIR if member.is_dir() else stat.S_IFREG), "invalid ZIP member")
            check(name not in seen, "duplicate ZIP member")
            seen.add(name)
            if not member.is_dir():
                files.add(name)
        check(all('/'.join(name.split('/')[:i]) not in files for name in seen for i in range(1, len(name.split('/')))),
              "ZIP file/directory collision")
        check(required in files and archive.testzip() is None, "ZIP lacks required evidence or has bad CRC")
        _extract(archive, output)
    return files


def select_native(acquisition):
    acquisition = Path(acquisition)
    _run(acquisition / "run.json", NATIVE)
    jobs = _load_json(acquisition / "jobs.json", "native jobs")
    check(jobs["total_count"] == len(jobs["jobs"]) == 4 and {row["name"] for row in jobs["jobs"]} == TARGETS,
          "native gate lacks all four exact platform jobs")
    check(all(row["run_id"] == NATIVE["run_id"] and row["run_attempt"] == 1 and row["head_sha"] == NATIVE["head_sha"]
              and row["status"] == "completed" and row["conclusion"] == "success" for row in jobs["jobs"]),
          "native job is unfinished, failed or belongs to another head/attempt")
    listing = _load_json(acquisition / "artifacts.json", "native artifacts")
    check(listing["total_count"] == len(listing["artifacts"]), "native artifact listing is incomplete")
    name = NATIVE["artifact_name"]
    selected = [row for row in listing["artifacts"] if row["name"] == name]
    check(len(selected) == 1, "exact native Linux artifact is missing or ambiguous")
    artifact = selected[0]
    check(artifact["expired"] is False and artifact["digest"].startswith("sha256:") and
          HEX64.fullmatch(artifact["digest"][7:]), "native artifact has no live SHA256 identity")
    check(artifact["id"] == NATIVE["artifact_id"] and artifact["size_in_bytes"] == NATIVE["size_in_bytes"] and
          artifact["digest"] == "sha256:" + NATIVE["artifact_sha256"], "native artifact differs from retained acceptance identity")
    selection = dict(NATIVE, artifact_id=artifact["id"], artifact_name=name,
                     artifact_sha256=artifact["digest"][7:], size_in_bytes=artifact["size_in_bytes"])
    save(acquisition / "selection.json", selection)
    return selection


def verify_native_workflows(root, binary_sha):
    general = ["interaction-ratio-formula", "interaction-ratio-matrix", "interaction-sf-formula",
               "interaction-nf-formula", "interaction-sf-nf-formula", "nointercept-sf-formula", "intercept-sf-formula"]
    groups = [("design", general + [f"cook-{size}-{mode}-{route}" for size in ("7x7", "7x3")
                                    for mode in ("default", "disabled") for route in ("formula", "matrix")]),
              ("low-df", ["df-1", "df-2", "df-3", "df-4"])]
    determinism = _load_json(root / "worker-determinism.json", "native determinism")
    check(determinism["passed"] is True and determinism["compared_workers"] == [1, 4] and
          determinism["compared_workflows"] == 20 and determinism["pasilla"]["fitted_state_equal"] is True,
          "accepted native workflow inventory is incomplete")
    for group, cases in groups:
        records = determinism[group.replace("-", "_") + "_cases"]
        check(set(records) == set(cases) and all(row["fitted_state_equal"] is True for row in records.values()),
              "accepted native workflow determinism is incomplete")
    for workers in (1, 4):
        comparison = root / "comparison" / f"workers-{workers}"
        contract = _load_json(comparison / "comparison-contract.json", "native comparison contract")
        result = _load_json(comparison / "comparison.json", "native comparison result")
        check(contract["binary_sha256"] == binary_sha and contract["workers"] == workers and
              result["passed"] is True and result["failure_count"] == 0,
              "accepted complete native fit was not made by this executable")
        check(_load_json(comparison / "fit.json", "accepted fitted state")["schema"] == 4,
              "accepted state is not schema4")
        for group, cases in groups:
            comparison = root / f"{group}-comparison" / f"workers-{workers}"
            contract = _load_json(comparison / "comparison-contract.json", "native workflow contract")
            result = _load_json(comparison / "summary.json", "native workflow summary")
            check(contract["binary_sha256"] == binary_sha and contract["workers"] == workers and
                  contract["cases"] == cases and contract["excluded_cases"] == [] and result["passed"] is True and
                  [row["case"] for row in result["cases"]] == cases and
                  all(row["passed"] is True and row["failure_count"] == 0 for row in result["cases"]),
                  "accepted native workflow coverage is incomplete")
            check(all(_load_json(comparison / case / "fit.json", "accepted workflow state")["schema"] == 4
                      for case in cases), "accepted workflow state is not schema4")
    return dict(schema=4, workflow_count=20, workers=[1, 4])


def prepare_runtime(acquisition):
    acquisition = Path(acquisition).resolve(strict=True)
    native, source = acquisition / "native", acquisition / "source"
    selected = _load_json(native / "selection.json", "selected native artifact")
    check(all(selected.get(name) == value for name, value in NATIVE.items()), "selected native run differs")
    _run(native / "run.json", selected)
    _artifact(native / "artifact.json", selected, native / "artifact.zip")
    check(_sha256(native / "artifact.zip") == selected["artifact_sha256"], "native ZIP differs from fresh API")
    extract_zip(native / "artifact.zip", native / "artifact", "worker-determinism.json")
    root = native / "artifact"
    for name, digest in {"source.tar.gz": ARCHIVE_SHA, "files.sha256": MANIFEST_SHA, "Cargo.lock": LOCK_SHA}.items():
        check(_sha256(root / name) == digest, f"accepted green5 identity differs: {name}")
    files = verify_and_extract(root, native / "candidate")
    check(len(files) == 36 and files["Cargo.lock"] == LOCK_SHA, "green5 source inventory/lock differs")
    binary = root / "native-binary/rsomics-deseq-linux-x86_64"
    line = (binary.parent / "checksums.sha256").read_text().strip()
    digest, name = line.split("  ", 1)
    check(digest == BINARY_SHA and Path(name).name == binary.name and _sha256(binary) == digest,
          "accepted executable checksum differs")
    check((root / "host.log").read_text().splitlines()[0] == NATIVE["head_sha"] and
          "snapshot=green-5" in (root / "host.log").read_text(), "native source snapshot/head binding differs")
    coverage = verify_native_workflows(root, digest)
    checker = native / "candidate/scripts/check-workflow.py"
    check(_sha256(root / "controller/check-workflow.py") == files["scripts/check-workflow.py"], "accepted comparator differs")
    spec = importlib.util.spec_from_file_location("accepted_deseq_comparator", checker)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    check(module.BOUNDS == BOUNDS, "accepted numerical bounds differ")
    check(_load_json(source / "receipt.json", "original source receipt") == SOURCE, "original source receipt changed")
    _run(source / "run.json", SOURCE)
    _artifact(source / "artifact.json", SOURCE, source / "artifact.zip")
    check(_sha256(source / "artifact.zip") == SOURCE["artifact_sha256"], "original source ZIP differs")
    members = extract_zip(source / "artifact.zip", source / "artifact", "workflow/sha256-manifest.tsv")
    witness = source / "artifact/workflow"
    check(_sha256(witness / "sha256-manifest.tsv") == SOURCE["manifest_sha256"], "original source manifest differs")
    with (witness / "sha256-manifest.tsv").open(newline="") as stream:
        rows = csv.reader(stream, delimiter="\t")
        check(next(rows) == ["sha256", "path"], "invalid original source manifest")
        recorded = {}
        for value, name in rows:
            _archive_name(name, False)
            check(HEX64.fullmatch(value) and name not in recorded, "invalid source manifest entry")
            recorded[name] = value
    actual = {name.removeprefix("workflow/") for name in members if name.startswith("workflow/")}
    check(len(recorded) == 55 and actual == set(recorded) | {"sha256-manifest.tsv"} and
          all(_sha256(witness / name) == value for name, value in recorded.items()), "original source inventory/bytes differ")
    check(_sha256(witness / "witness.json") == SOURCE["witness_sha256"], "original source witness differs")
    oracle = _load_json(witness / "witness.json", "original source witness")
    check(oracle["status"] == "complete" and oracle["inputs"]["genes"] == 14599 and oracle["inputs"]["samples"] == 7
          and oracle["inputs"]["all_zero_rows"] == 2240, "original source population differs")
    check(_sha256(source / "artifact/DESeq2_1.52.0.tar.gz") == SOURCE_ARCHIVE_SHA, "released source archive differs")
    binary.chmod(0o755)
    save(acquisition / "verification.json", dict(native=selected, binary_sha256=digest,
        archive_sha256=ARCHIVE_SHA, manifest_sha256=MANIFEST_SHA, lock_sha256=LOCK_SHA,
        native_source_files=files, native_workflows=coverage, source=SOURCE, source_files=recorded,
        checker_sha256=_sha256(checker), bounds=BOUNDS))


def measured(command, directory, cores):
    directory.mkdir()
    usage = directory / "time.json"
    argv = ["/usr/bin/time", "-f", TIME_FORMAT, "-o", str(usage), "taskset", "-c", cores, *map(str, command)]
    child = run_child(argv, directory / "process", timeout=1800)
    row = _load_json(usage, "GNU time")
    check(row["exit_status"] == 0 and all(type(value) in (int, float) and math.isfinite(value) and value >= 0
                                         for value in row.values()), "invalid GNU time measurement")
    row.update(argv=list(map(str, command)), owner_wall_seconds=(child.end_ns - child.start_ns) / 1e9)
    save(directory / "measurement.json", row)
    return row


def aggregate_native(rows):
    summed = ("wall_seconds", "user_seconds", "system_seconds", "filesystem_inputs", "filesystem_outputs",
              "major_faults", "minor_faults", "owner_wall_seconds")
    return {**{name: sum(row[name] for row in rows) for name in summed},
            "max_rss_kib": max(row["max_rss_kib"] for row in rows), "commands": rows}


def output_manifest(output, bundle):
    return {name: dict(sha256=_sha256(output / name), size_bytes=(output / name).stat().st_size)
            for name in (bundle, *OUTPUTS)}


def compare_source(actual, inspected, witness, output):
    failures, metrics, iterations = [], {}, []
    def point(label, a, e, kind):
        if a in ("NA", "NaN") or e in ("NA", "NaN"):
            if a != e:
                failures.append(dict(field=label, actual=a, expected=e))
            return
        a, e = float(a), float(e)
        absolute, relative = BOUNDS[kind]
        scaled = abs(a - e) / (absolute + relative * abs(e))
        record = metrics.setdefault(label, dict(count=0, failed=0, max_scaled=0.0))
        record["count"] += 1
        ok = math.isfinite(a) and math.isfinite(e) and scaled <= 1
        record["max_scaled"] = max(record["max_scaled"], scaled)
        if not ok:
            record["failed"] += 1
            failures.append(dict(field=label, actual=a, expected=e))
    def table(path):
        with path.open(newline="") as stream:
            return [[value.strip() for value in row] for row in csv.reader(stream, delimiter="\t")]
    def compare_tables(actual_path, reference_path, kind=None, diagnostics=False, exact=False):
        a, e = table(actual_path), table(reference_path)
        check(a[0] == e[0] and len(a) == len(e) and all(len(row) == len(a[0]) for row in a + e), "source table shape/header differs")
        check([row[0] for row in a[1:]] == [row[0] for row in e[1:]], "source feature/sample identity order differs")
        for row, reference in zip(a[1:], e[1:]):
            for name, value, expected in zip(a[0][1:], row[1:], reference[1:]):
                label = actual_path.name + ":" + name
                if diagnostics and name in ("dispGeneIter", "dispIter", "betaIter"):
                    if value != expected:
                        iterations.append(dict(gene=row[0], field=name, actual=value, expected=expected))
                elif exact or diagnostics and name in ("allZero", "dispOutlier", "betaConv"):
                    if value != expected:
                        failures.append(dict(field=label, gene=row[0], actual=value, expected=expected))
                else:
                    selected = kind or ("normalization" if name in ("baseMean", "baseVar", "size_factor") else
                        "probability" if name in ("pvalue", "padj") or name.startswith("WaldPvalue_") else
                        "dispersion" if name.startswith("disp") else "diagnostic" if name in ("deviance", "maxCooks") else "coefficient")
                    point(label, value, expected, selected)
                    if name == "padj" and (value not in ("NA", "NaN") and float(value) < .1) != (expected not in ("NA", "NaN") and float(expected) < .1):
                        failures.append(dict(field=label + " discovery", gene=row[0], actual=value, expected=expected))
    for name, relative, kind in (("forward.tsv", "results/forward.tsv", None), ("reverse.tsv", "reload/reverse.tsv", None),
                                 ("normalized-counts.tsv", "results/normalized-counts.tsv", "normalization"), ("vst.tsv", "results/vst.tsv", "vst")):
        compare_tables(actual / name, witness / relative, kind)
    compare_tables(inspected / "row-diagnostics.tsv", witness / "fit/row-diagnostics.tsv", diagnostics=True)
    for name, kind in (("coefficients.tsv", "coefficient"), ("size-factors.tsv", "normalization"),
                       ("assay-mu.tsv", "mu"), ("assay-H.tsv", "diagnostic"), ("assay-cooks.tsv", "diagnostic")):
        compare_tables(inspected / name, witness / "fit" / name, kind)
    for name in ("model-matrix.tsv", "assay-counts.tsv"):
        compare_tables(inspected / name, witness / "fit" / name, exact=True)
    def compare_metadata(a, e, label, kind):
        if isinstance(e, dict):
            check(isinstance(a, dict) and set(a) == set(e), "source metadata fields differ")
            for key in e:
                compare_metadata(a[key], e[key], label + ":" + key, kind)
        elif isinstance(e, list):
            check(isinstance(a, list) and len(a) == len(e), "source metadata length differs")
            for x, y in zip(a, e):
                compare_metadata(x, y, label, kind)
        elif type(e) in (int, float):
            if label.endswith("numRej"):
                if a != e:
                    failures.append(dict(field=label, actual=a, expected=e))
            else:
                point(label, str(a), str(e), kind)
        elif a != e:
            failures.append(dict(field=label, actual=a, expected=e))
    compare_metadata(_load_json(inspected / "filter-and-result-metadata.json", "measured filtering"),
                     _load_json(witness / "results/filter-and-result-metadata.json", "source filtering"), "filter", "normalization")
    fit = _load_json(inspected / "fit-summary.json", "measured fit summary")
    expected = _load_json(witness / "witness.json", "source witness")["fitted"]
    check((fit["genes"], fit["samples"], fit["initial_discoveries"]) == (14599, 7, 1330), "source population/discoveries differ")
    check(fit["actual_dispersion_fit"] == expected["actual_dispersion_fit"] and
          fit["initial_result_na_counts"] == expected["forward_na_counts"], "source fit type/missingness differs")
    compare_metadata(fit["dispersion_attributes"], expected["dispersion_attributes"], "dispersion attributes", "dispersion")
    save(output / "comparison.json", dict(passed=not failures, failure_count=len(failures), bounds=BOUNDS, metrics=metrics))
    save(output / "failures.json", failures)
    save(output / "iteration-diagnostics.json", iterations)
    check(not failures, "measured source differs from the unchanged original witness")


def native_trial(directory, witness, binary, checker, workers, cores):
    directory.mkdir()
    output = directory / "output"
    output.mkdir()
    terms = ["--contrast=(Intercept)=0", "--contrast=typesingle-read=0", "--contrast=conditiontreated=1"]
    bundle = output / "fit.json"
    commands = [
        ["run", "--counts", witness / "inputs/counts.tsv", "--samples", witness / "inputs/samples.tsv",
         "--design", witness / "inputs/model-matrix.tsv", "--bundle", bundle, *terms],
        ["results", "--bundle", bundle, "--output", output / "forward.tsv", *terms],
        ["results", "--bundle", bundle, "--output", output / "reverse.tsv", *terms[:-1], "--contrast=conditiontreated=-1"],
        ["transform", "--bundle", bundle, "--output", output / "normalized-counts.tsv", "--kind", "normalized"],
        ["transform", "--bundle", bundle, "--output", output / "vst.tsv", "--kind", "vst"],
    ]
    rows = [measured([binary, "--json", "--threads", str(workers), *command], directory / f"command-{index}", cores)
            for index, command in enumerate(commands)]
    original = output_manifest(output, "fit.json")
    save(directory / "timed-output-manifest.json", original)
    validation = directory / "validation"
    validation.mkdir()
    shutil.copyfile(bundle, validation / "fit.json")
    run_child([sys.executable, "-B", str(checker), "--binary", str(binary), "--witness", str(witness),
               "--output", str(validation), "--workers", str(workers), "--reuse"], directory / "validation-process", timeout=1800)
    check(_load_json(validation / "comparison.json", "complete native comparison")["passed"] is True, "native compatibility failed")
    check(original == output_manifest(output, "fit.json") and
          all(_sha256(validation / name) == original[name]["sha256"] for name in ("fit.json", *OUTPUTS)),
          "untimed native validation replaced or disagreed with measured outputs")
    result = aggregate_native(rows)
    result.update(workers=workers, outputs=original, comparison_passed=True)
    save(directory / "measurement.json", result)
    fitted = json.loads(bundle.read_text())
    check(fitted.pop("workers") == workers and fitted["schema"] == 4, "measured native state identity differs")
    return fitted, result


def source_trial(directory, witness, qualification, cores):
    directory.mkdir()
    output = directory / "output"
    driver = ROOT / "scripts/deseq_workflow_performance.R"
    row = measured(["Rscript", "--vanilla", driver, "run", witness, output, qualification], directory / "command", cores)
    original = output_manifest(output, "fit.rds")
    save(directory / "timed-output-manifest.json", original)
    inner = _load_json(output / "inner-timing.json", "source inner workflow timing")
    check(all(type(inner[name]) in (int, float) and math.isfinite(inner[name]) and inner[name] >= 0
              for name in ("inner_elapsed_seconds", "inner_user_seconds", "inner_system_seconds")), "invalid source inner timing")
    validation = directory / "validation"
    validation.mkdir()
    inspected = validation / "fit"
    run_child(["Rscript", "--vanilla", str(driver), "inspect", str(output), str(inspected), str(qualification)],
              validation / "inspection-process", timeout=1800)
    compare_source(output, inspected, witness, validation)
    check(original == output_manifest(output, "fit.rds"), "untimed source inspection changed measured outputs")
    row.update({name: inner[name] for name in ("inner_elapsed_seconds", "inner_user_seconds", "inner_system_seconds")})
    row.update(outputs=original, comparison_passed=True)
    save(directory / "measurement.json", row)
    return row


def distribution(rows):
    names = ("wall_seconds", "user_seconds", "system_seconds", "max_rss_kib", "filesystem_inputs", "filesystem_outputs",
             "major_faults", "minor_faults", "owner_wall_seconds", "inner_elapsed_seconds", "inner_user_seconds", "inner_system_seconds")
    return {name: dict(values=[row[name] for row in rows], minimum=min(row[name] for row in rows),
                       maximum=max(row[name] for row in rows), median=statistics.median(row[name] for row in rows))
            for name in names if name in rows[0]}


def performance_decision(stats):
    source, native = stats["source"], stats["native"]
    variants = {}
    for workers, values in native.items():
        strict = source["inner_elapsed_seconds"]["minimum"] > values["wall_seconds"]["maximum"]
        memory = source["max_rss_kib"]["minimum"] > values["max_rss_kib"]["maximum"]
        variants[workers] = dict(strict_throughput_excluding_R_startup=strict, strict_process_peak_rss=memory,
            advantage_passed=strict or memory, median_whole_process_speedup=source["wall_seconds"]["median"] / values["wall_seconds"]["median"],
            median_inner_lower_bound_speedup=source["inner_elapsed_seconds"]["median"] / values["wall_seconds"]["median"],
            median_process_peak_rss_ratio=source["max_rss_kib"]["median"] / values["max_rss_kib"]["median"])
    return dict(advantage_passed=any(row["advantage_passed"] for row in variants.values()), workers=variants,
        strict_worker_scaling=native["1"]["wall_seconds"]["minimum"] > native["4"]["wall_seconds"]["maximum"],
        median_worker_scaling=native["1"]["wall_seconds"]["median"] / native["4"]["wall_seconds"]["median"],
        rule="fixed before measurement: source inner minimum > native complete-path maximum, or source process-RSS minimum > native process-RSS maximum; overlap is not a scaling win")


def run_benchmark(acquisition, source_tree, output):
    check(platform.system() == "Linux" and platform.machine() == "x86_64", "benchmark requires native Linux x86_64")
    allowed = sorted(os.sched_getaffinity(0))
    check(len(allowed) >= 4, "four available CPUs are required")
    os.sched_setaffinity(0, allowed[:4])
    cores = ",".join(map(str, allowed[:4]))
    for name in ("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"):
        os.environ[name] = "1"
    acquisition, source_tree, output = Path(acquisition).resolve(strict=True), Path(source_tree).resolve(strict=True), Path(output).absolute()
    output.mkdir()
    record = _load_json(acquisition / "verification.json", "accepted acquisition")
    witness = acquisition / "source/artifact/workflow"
    binary = acquisition / "native/artifact/native-binary/rsomics-deseq-linux-x86_64"
    candidate = acquisition / "native/candidate"
    checker = candidate / "scripts/check-workflow.py"
    qualification = output / "source-qualification"
    run_child(["Rscript", "--vanilla", str(ROOT / "scripts/deseq_workflow_performance.R"), "qualify", str(source_tree), str(qualification)],
              output / "source-qualification-process", timeout=1800)
    runtime = _load_json(qualification / "runtime.json", "source runtime")
    installed = Path(runtime["installed_path"])
    with (qualification / "installed-DESeq2-sha256.tsv").open(newline="") as stream:
        package_files = {row["path"]: row["sha256"] for row in csv.DictReader(stream, delimiter="\t")}
    immutable = {**{candidate / name: digest for name, digest in record["native_source_files"].items()},
                 **{witness / name: digest for name, digest in record["source_files"].items()},
                 **{installed / name: digest for name, digest in package_files.items()}, binary: record["binary_sha256"]}
    for name in ("deseq_workflow_performance.py", "deseq_workflow_performance.R", "deseq_workflow_oracle.R",
                 "infercnv_source_snapshot.py", "infercnv_trial_process.py", "verify_infercnv_oracle_artifact.py"):
        path = ROOT / "scripts" / name
        immutable[path] = _sha256(path)
    for path in source_tree.rglob("*"):
        if path.is_file():
            immutable[path] = _sha256(path)
    def verify_immutable():
        check(all(_sha256(path) == digest for path, digest in immutable.items()), "source/binary/input/controller bytes changed")
    save(output / "immutability-before.json", {str(path): value for path, value in immutable.items()})
    save(output / "contract.json", dict(native=record["native"], binary_sha256=record["binary_sha256"], source=SOURCE,
        bounds=BOUNDS, input=dict(genes=14599, samples=7, all_zero_rows=2240, prefilter=False),
        source_warmups=1, source_repetitions=3, native_warmups_per_worker=1, native_repetitions_per_worker=5,
        workers=[1, 4], native_order="alternate [1,4] and [4,1]", source_default_serial=True,
        source_encoding="default-compressed full RDS fitted bundle; original gzip/CSV inputs with identity join",
        native_encoding="portable uncompressed full JSON bundle; same original values in frozen uncompressed TSV inputs",
        units="native wall/CPU/I/O/faults are sums over five real CLI processes; peak RSS is max process KiB, not sum or aggregate live memory; GNUtime filesystem counters are blocks, not logical bytes",
        throughput_rule="native complete-path maximum < source inner-workflow minimum; resource rule source process-RSS minimum > native maximum; no post-result retuning"))
    save(output / "host.json", dict(platform=platform.platform(), machine=platform.machine(), cpu_affinity=list(os.sched_getaffinity(0)),
        cpuinfo=Path("/proc/cpuinfo").read_text(), meminfo=Path("/proc/meminfo").read_text(), load_average=os.getloadavg(),
        disk=shutil.disk_usage(output)._asdict(), source_runtime=runtime, native_rustc=(acquisition / "native/artifact/rustc.log").read_text(),
        native_cargo=(acquisition / "native/artifact/cargo.log").read_text(), time_format=TIME_FORMAT))
    source, native = [], {1: [], 4: []}
    baseline = None
    try:
        source_trial(output / "source-warmup", witness, qualification, cores)
        for workers in (1, 4):
            fit, _ = native_trial(output / f"native-warmup-{workers}", witness, binary, checker, workers, cores)
            if baseline is None:
                baseline = fit
            check(fit == baseline, "native worker count changed complete fitted state")
        export_hashes = {name: _sha256(output / "native-warmup-1/output" / name) for name in OUTPUTS}
        check(all(_sha256(output / "native-warmup-4/output" / name) == digest for name, digest in export_hashes.items()),
              "native warmup exports differ between worker counts")
        for index in range(5):
            if index < 3:
                source.append(source_trial(output / f"source-{index + 1}", witness, qualification, cores))
                print(f"source repetition {index + 1}/3 matches original witness", flush=True)
            for workers in ((1, 4) if index % 2 == 0 else (4, 1)):
                trial = output / f"native-{index + 1}-workers-{workers}"
                fit, row = native_trial(trial, witness, binary, checker, workers, cores)
                check(fit == baseline and all(row["outputs"][name]["sha256"] == export_hashes[name] for name in OUTPUTS),
                      "complete native fitted state/export determinism failed")
                native[workers].append(row)
                print(f"native repetition {index + 1}/5, workers={workers}, matches original witness", flush=True)
            verify_immutable()
    finally:
        verify_immutable()
        save(output / "immutability-after.json", dict(unchanged=True, checked_files=len(immutable)))
    stats = dict(source=distribution(source), native={str(workers): distribution(rows) for workers, rows in native.items()})
    decision = performance_decision(stats)
    summary = dict(status="complete" if decision["advantage_passed"] else "rejected_no_advantage", binary_sha256=record["binary_sha256"],
        native=record["native"], source=SOURCE, statistics=stats, decision=decision, source_trials=source, native_trials=native,
        native_worker_state_equal=True, native_exports_sha256=export_hashes,
        scope="bounded full original14599x7 pasilla workflow, warm caches, default serial R versus accepted native1/4; not broad DESeq2 completeness, fastest-tuned R, cold I/O, large-cohort scaling, or publication acceptance")
    save(output / "summary.json", summary)
    check(decision["advantage_passed"], "complete workflow measured, but no strict replacement advantage")
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    for name in ("select-native", "prepare-runtime", "run"):
        child = sub.add_parser(name)
        child.add_argument("--acquisition", required=True, type=Path)
        if name == "run":
            child.add_argument("--source-tree", required=True, type=Path)
            child.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    try:
        if args.command == "select-native":
            selected = select_native(args.acquisition)
            print(selected["artifact_id"])
        elif args.command == "prepare-runtime":
            prepare_runtime(args.acquisition)
        else:
            summary = run_benchmark(args.acquisition, args.source_tree, args.output)
            print(json.dumps(dict(status=summary["status"], decision=summary["decision"])), flush=True)
    except (OSError, ValueError, KeyError, TrialProcessError) as error:
        if args.command == "run" and args.output.is_dir() and not (args.output / "failure.json").exists():
            save(args.output / "failure.json", dict(error=f"{type(error).__name__}: {error}"))
        parser.exit(1, f"complete DESeq workflow measurement failed: {error}\n")


if __name__ == "__main__":
    main()
