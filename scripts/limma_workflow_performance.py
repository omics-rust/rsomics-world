#!/usr/bin/env python3
"""Measure the accepted complete current-voom lifecycle, without rebuilding it."""

import argparse
import csv
import importlib.util
import json
import math
import os
from pathlib import Path
import platform
import shutil
import sys
import tarfile

from deseq_workflow_performance import (TIME_FORMAT, aggregate_native, check, distribution,
    extract_zip, measured, performance_decision, save, select_native as select_retained_artifact)
from infercnv_source_snapshot import verify_and_extract
from infercnv_trial_process import TrialProcessError, run_child
from verify_infercnv_oracle_artifact import HEX64, _archive_name, _artifact, _load_json, _run, _sha256


ROOT = Path(__file__).resolve().parents[1]
NATIVE = dict(repository="omics-rust/rsomics-world", run_attempt=1,
              workflow_path=".github/workflows/limma-candidate.yml")
NATIVE_RECEIPT = ROOT / ".autopilot/oracles/limma-native-green1-2026-10-28.json"
ARCHIVE_SHA = "a4b9aad772e460e688d4b6df60eb8eda9adecd9f1cf7cc9cb0f9f499f3a23133"
MANIFEST_SHA = "9d508832b0c038ef5e1dac6e181d27ba62fa1fea82aac9efa719b15575034403"
LOCK_SHA = "db5108b10ff8780c1102c1a9859cc9b49f1c7bd22bbe3a35cddbf5b579c5a794"
SOURCE = dict(schema_version=1, repository="omics-rust/rsomics-world", run_attempt=1,
    workflow_path=".github/workflows/limma-oracle.yml", run_id=40606516204,
    artifact_id=12780957275, head_sha="d5e136a87c681e53095293638441a5a26c4dc503",
    artifact_name="limma-oracle-40606516204-1", size_in_bytes=84976843,
    artifact_sha256="fac61c690c3f742335446c141ac2ad78c84fe3df33d20df8e0181cc31e43f5a2",
    bundle_path="workflow", manifest_sha256="af682fed901644550411e6ae74e22b6bda45a7e567ca15e2dddc2c6b17027e97",
    witness_sha256="091f476ca726487d23e83de7d62b5bffa7cea9021b533a4dda2043c89289a577")
PACKAGES = dict(
    edgeR=dict(name="edgeR", version="4.10.7", bioconductor="3.23",
        source_commit="27ba7b39d819e68df2ecdea9f32a4b6039b2a0b5",
        source_archive_sha256="a3cc0f66be29f1a61a74021f9848ec940794e653905c4332327a719dd9ad8393"),
    limma=dict(name="limma", version="3.68.5", bioconductor="3.23",
        source_commit="825d1c8330d024adb6c54af66d79b0f4ec258530",
        source_archive_sha256="40b1d2059a76ad98db4bf80139760acf8e8bae92a8265aaaf16ac199831533f4"))
CARRIER_SHA = "8c91699286336350e66eec132ce6fdf5bb4af78e2a4d015a5a61224f62a95984"
OUTPUTS = ("forward.tsv", "reverse.tsv", "mixed.tsv", "logCPM.tsv", "weights.tsv", "fitted.tsv", "residuals.tsv")
STATES = ("forward.json", "reverse.json", "mixed.json")
BOUNDS = dict(tmm=(1e-10, 1e-10), library=(1e-8, 1e-10), expression=(1e-10, 1e-10),
    linear=(1e-7, 1e-6), trend=(1e-7, 1e-6), weights=(1e-6, 1e-5),
    eb=(1e-6, 1e-4), df=(1e-3, 1e-4), odds=(1e-5, 5e-4), probability=(1e-12, 5e-4))
LOG_PROBABILITY_BOUND = 1e-3
CONTROLLERS = ("limma_workflow_performance.py", "limma_workflow_performance.R", "limma_workflow_oracle.R",
    "edger_workflow_oracle.R", "deseq_workflow_oracle.R", "deseq_workflow_performance.py",
    "infercnv_source_snapshot.py", "infercnv_trial_process.py", "verify_infercnv_oracle_artifact.py", "test_limma_workflow_performance.py")


def load_native_receipt(path=NATIVE_RECEIPT):
    receipt = _load_json(path, "retained current-voom native receipt")
    expected = dict(NATIVE, schema_version=1, snapshot="green-1", product="rsomics-limma",
        source_archive_sha256=ARCHIVE_SHA, source_manifest_sha256=MANIFEST_SHA, lock_sha256=LOCK_SHA,
        source_file_count=29, fit_schema=1, workflow_count=1, workers=[1, 4])
    check(all(receipt.get(key) == value for key, value in expected.items()), "retained native receipt identity differs")
    check(receipt.get("native_targets") == ["linux-x86_64", "linux-aarch64", "macos-x86_64", "macos-aarch64"] and
          all(isinstance(receipt.get(key), str) and HEX64.fullmatch(receipt[key])
              for key in ("artifact_sha256", "binary_sha256", "verification_sha256")) and
          all(type(receipt.get(key)) is int and receipt[key] > 0 for key in ("run_id", "artifact_id", "size_in_bytes")) and
          isinstance(receipt.get("head_sha"), str) and len(receipt["head_sha"]) == 40 and
          all(c in "0123456789abcdef" for c in receipt["head_sha"]), "retained native receipt is incomplete")
    check(receipt.get("artifact_name") == f"limma-{receipt['run_id']}-1-linux-x86_64", "retained native artifact name differs")
    return receipt


def select_native(acquisition, receipt=None):
    return select_retained_artifact(acquisition, load_native_receipt() if receipt is None else receipt)


def verify_native_workflow(root, binary_sha):
    determinism = _load_json(root / "worker-determinism.json", "accepted worker determinism")
    expected = dict(passed=True, compared_workers=[1, 4], compared_workflows=1, fitted_state_equal=True,
                    complete_contrast_states=3, export_tables=7, binary_sha256=binary_sha)
    check(all(determinism.get(key) == value for key, value in expected.items()), "accepted whole-workflow determinism differs")
    fits = []
    for workers in (1, 4):
        output = root / f"comparison/workers-{workers}"
        contract = _load_json(output / "comparison-contract.json", "native source contract")
        report = _load_json(output / "comparison.json", "native source comparison")
        check(contract["workers"] == workers and contract["binary_sha256"] == binary_sha and
              contract["source_run"] == SOURCE["run_id"] and contract["manifest_sha256"] == SOURCE["manifest_sha256"] and
              contract["witness_sha256"] == SOURCE["witness_sha256"] and
              contract["bounds"] == {name: list(value) for name, value in BOUNDS.items()} and
              contract["positive_probability_log_bound"] == LOG_PROBABILITY_BOUND, "accepted native source contract differs")
        check(report["passed"] is True and report["failure_count"] == 0 and report["raw_genes"] == 14599 and
              report["kept_genes"] == 8066 and report["residual_df_distribution"] == {"2": 7, "4": 8059} and
              report["bounds"] == contract["bounds"] and
              json.loads((output / "failures.json").read_text()) == [], "accepted native source workflow failed")
        fit = _load_json(output / "fit.json", "accepted full native state")
        check(fit["schema"] == 1 and fit["product"] == "rsomics-limma" and fit.pop("workers") == workers and
              report["native_bundle_sha256"] == _sha256(output / "fit.json"), "accepted full state identity differs")
        fits.append(fit)
        commands = json.loads((output / "commands.json").read_text())
        check(len(commands) == 8 and all(row["exit"] == 0 for row in commands), "accepted complete native CLI path differs")
        for name in (*OUTPUTS, *STATES):
            check(_sha256(output / name) == determinism["exports_sha256"][name], "accepted native state/table determinism differs")
    check(fits[0] == fits[1], "accepted complete fitted state differs between workers")
    check(_load_json(root / "source-after.json", "native source preservation")["unchanged_files"] == 29,
          "accepted native source changed")
    return dict(schema=1, product="rsomics-limma", workflow_count=1, workers=[1, 4], complete_contrast_states=3, export_tables=7)


def prepare_runtime(acquisition):
    acquisition = Path(acquisition).resolve(strict=True)
    native, source = acquisition / "native", acquisition / "source"
    receipt = load_native_receipt(native / "receipt.json")
    check(_load_json(native / "selection.json", "selected executable") == receipt, "selected native run differs from retained receipt")
    _run(native / "run.json", receipt)
    _artifact(native / "artifact.json", receipt, native / "artifact.zip")
    check(_sha256(native / "artifact.zip") == receipt["artifact_sha256"] and
          (native / "artifact.zip").stat().st_size == receipt["size_in_bytes"], "native ZIP differs from retained receipt")
    extract_zip(native / "artifact.zip", native / "artifact", "worker-determinism.json")
    root = native / "artifact"
    for name, digest in {"source.tar.gz": ARCHIVE_SHA, "files.sha256": MANIFEST_SHA, "Cargo.lock": LOCK_SHA}.items():
        check(_sha256(root / name) == digest, "green1 source identity differs: " + name)
    files = verify_and_extract(root, native / "candidate")
    check(len(files) == 29 and files["Cargo.lock"] == LOCK_SHA, "green1 source inventory/lock differs")
    binary = root / "native-binary/rsomics-limma-linux-x86_64"
    digest, name = (binary.parent / "checksums.sha256").read_text().strip().split("  ", 1)
    check(digest == receipt["binary_sha256"] == _sha256(binary) and Path(name).name == binary.name,
          "accepted executable checksum differs")
    check((root / "host.log").read_text().splitlines()[0] == receipt["head_sha"] and
          "snapshot=green-1" in (root / "host.log").read_text(), "native snapshot/head binding differs")
    coverage = verify_native_workflow(root, digest)
    checker = native / "candidate/scripts/check-workflow.py"
    check(_sha256(root / "controller/check-workflow.py") == files["scripts/check-workflow.py"], "accepted comparator bytes differ")
    spec = importlib.util.spec_from_file_location("accepted_limma_comparator", checker)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    check(module.BOUNDS == BOUNDS and module.LOG_PROBABILITY_BOUND == LOG_PROBABILITY_BOUND, "accepted numerical bounds differ")
    check(_load_json(source / "receipt.json", "original source receipt") == SOURCE, "original source receipt differs")
    _run(source / "run.json", SOURCE)
    _artifact(source / "artifact.json", SOURCE, source / "artifact.zip")
    check(_sha256(source / "artifact.zip") == SOURCE["artifact_sha256"] and
          (source / "artifact.zip").stat().st_size == SOURCE["size_in_bytes"], "original source ZIP differs")
    members = extract_zip(source / "artifact.zip", source / "artifact", "workflow/sha256-manifest.tsv")
    witness = source / "artifact/workflow"
    check(_sha256(witness / "sha256-manifest.tsv") == SOURCE["manifest_sha256"] and
          _sha256(witness / "witness.json") == SOURCE["witness_sha256"], "original source manifest/witness differs")
    with (witness / "sha256-manifest.tsv").open(newline="") as stream:
        rows = csv.reader(stream, delimiter="\t")
        check(next(rows) == ["sha256", "path"], "invalid original source manifest")
        recorded = {}
        for value, name in rows:
            _archive_name(name, False)
            check(HEX64.fullmatch(value) and name not in recorded, "invalid source manifest entry")
            recorded[name] = value
    actual = {name.removeprefix("workflow/") for name in members if name.startswith("workflow/")}
    check(len(recorded) == 203 and actual == set(recorded) | {"sha256-manifest.tsv"} and
          all(_sha256(witness / name) == value for name, value in recorded.items()), "original source inventory/bytes differ")
    oracle = _load_json(witness / "witness.json", "original source witness")
    check(oracle["status"] == "complete" and oracle["failure"] is None and all(oracle["checks"].values()) and
          oracle["packages"] == PACKAGES and oracle["files"] == {name: value for name, value in recorded.items() if name != "witness.json"} and
          oracle["branch_coverage"]["selected_moderation"] == "fitFDistUnequalDF1" and
          oracle["branch_coverage"]["uncovered"] == [], "original current-voom source qualification differs")
    source_tree = acquisition / "source-tree"
    source_tree.mkdir()
    archives = {f"{name}_{pin['version']}.tar.gz": (name, pin["source_archive_sha256"])
                for name, pin in PACKAGES.items()}
    archives["DESeq2_1.52.0.tar.gz"] = ("DESeq2", CARRIER_SHA)
    for name, (package, digest) in archives.items():
        archive_path = source / "artifact" / name
        check(_sha256(archive_path) == digest, "original released source archive differs: " + name)
        with tarfile.open(archive_path) as archive:
            for member in archive.getmembers():
                member_name = _archive_name(member.name.rstrip("/") + ("/" if member.isdir() else ""), member.isdir())
                check(member_name.split("/")[0] == package and (member.isfile() or member.isdir()), "unsafe source archive member")
            archive.extractall(source_tree, filter="data")
    binary.chmod(0o755)
    save(acquisition / "verification.json", dict(native=receipt, binary_sha256=receipt["binary_sha256"],
        native_source_files=files, native_workflows=coverage, source=SOURCE, source_files=recorded,
        checker_sha256=_sha256(checker), packages=PACKAGES, bounds=BOUNDS, positive_probability_log_bound=LOG_PROBABILITY_BOUND))


def native_commands(witness, output):
    terms = ["--contrast=(Intercept)=0", "--contrast=typesingle-read=0", "--contrast=conditiontreated=1"]
    bundle = output / "fit.json"
    commands = [["run", "--counts", witness / "inputs/counts.tsv", "--samples", witness / "inputs/samples.tsv",
                 "--design", witness / "inputs/model-matrix.tsv", "--bundle", bundle, *terms]]
    for name, weights in (("forward", terms), ("reverse", terms[:-1] + ["--contrast=conditiontreated=-1"]),
                          ("mixed", [terms[0], "--contrast=typesingle-read=-1", terms[2]])):
        commands.append(["results", "--bundle", bundle, "--output", output / f"{name}.tsv",
                         "--state", output / f"{name}.json", *weights])
    commands += [["export", "--bundle", bundle, "--output", output / f"{name}.tsv", "--kind", kind]
                 for kind, name in (("log-cpm", "logCPM"), ("weights", "weights"), ("fitted", "fitted"), ("residuals", "residuals"))]
    return commands


def output_manifest(output, source=False):
    names = ("fit.rds", "forward.rds", "reverse.rds", "mixed.rds") if source else ("fit.json", *STATES)
    return {name: dict(sha256=_sha256(output / name), size_bytes=(output / name).stat().st_size)
            for name in (*names, *OUTPUTS)}


def native_trial(directory, witness, binary, checker, workers, cores, accepted):
    directory.mkdir()
    output = directory / "output"
    output.mkdir()
    rows = [measured([binary, "--json", "--threads", str(workers), *command], directory / f"command-{index}", cores)
            for index, command in enumerate(native_commands(witness, output))]
    original = output_manifest(output)
    save(directory / "timed-output-manifest.json", original)
    validation = directory / "validation"
    validation.mkdir()
    fitted = _load_json(output / "fit.json", "measured native full state")
    expected = _load_json(accepted / "fit.json", "accepted same-binary full state")
    check(fitted.pop("workers") == workers and expected.pop("workers") == workers and
          fitted["schema"] == 1 and fitted["product"] == "rsomics-limma", "measured native state identity differs")
    save(validation / "cross-host-binding.json", dict(accepted_bundle=str(accepted / "fit.json"),
        accepted_sha256=_sha256(accepted / "fit.json"), measured_sha256=original["fit.json"]["sha256"],
        ignored_fields=["workers"], exact_cross_host_state_equal=fitted == expected,
        exact_cross_host_exports_equal={name: _sha256(accepted / name) == original[name]["sha256"] for name in (*STATES, *OUTPUTS)},
        policy="diagnostic only; numerical acceptance uses the original fixed source bounds"))
    shutil.copyfile(output / "fit.json", validation / "fit.json")
    run_child([sys.executable, "-B", str(checker), "--binary", str(binary), "--witness", str(witness),
               "--output", str(validation), "--workers", str(workers), "--reuse"], directory / "validation-process", timeout=1800)
    check(_load_json(validation / "comparison.json", "measured native source comparison")["passed"] is True,
          "measured native compatibility failed")
    check(all(_sha256(validation / name) == original[name]["sha256"] for name in original),
          "untimed reuse validation disagreed with actual timed states/tables")
    check(output_manifest(output) == original, "native validation changed actual timed outputs")
    result = aggregate_native(rows)
    result.update(workers=workers, outputs=original, comparison_passed=True,
                  stages={name: row for name, row in zip(("run", "forward", "reverse", "mixed", "logCPM", "weights", "fitted", "residuals"), rows)})
    save(directory / "measurement.json", result)
    return fitted, result


def source_trial(directory, witness, qualification, cores):
    directory.mkdir()
    output = directory / "output"
    driver = ROOT / "scripts/limma_workflow_performance.R"
    row = measured(["Rscript", "--vanilla", driver, "run", witness, output, qualification], directory / "command", cores)
    original = output_manifest(output, source=True)
    save(directory / "timed-output-manifest.json", original)
    inner = _load_json(output / "inner-timing.json", "source inner whole-workflow timing")
    for name in ("inner_elapsed_seconds", "inner_user_seconds", "inner_system_seconds"):
        check(type(inner[name]) in (int, float) and math.isfinite(inner[name]) and inner[name] >= 0, "invalid source inner timing")
        row[name] = inner[name]
    validation = directory / "validation"
    run_child(["Rscript", "--vanilla", str(driver), "inspect", str(output), str(validation), str(qualification), str(witness)],
              directory / "inspection-process", timeout=1800)
    comparison = _load_json(validation / "comparison.json", "measured source complete-state comparison")
    check(comparison["passed"] is True and comparison["bounds"] == {key: list(pair) for key, pair in BOUNDS.items()} and
          comparison["positive_probability_log_bound"] == LOG_PROBABILITY_BOUND, "measured source compatibility failed")
    check(output_manifest(output, source=True) == original, "untimed source inspection changed actual timed outputs")
    row.update(outputs=original, comparison_passed=True, discoveries=comparison["discoveries"])
    save(directory / "measurement.json", row)
    return row


def run_benchmark(acquisition, source_tree, output):
    check(platform.system() == "Linux" and platform.machine() == "x86_64", "measurement requires native Linux x86_64")
    allowed = sorted(os.sched_getaffinity(0))
    check(len(allowed) >= 4, "four available logical CPUs are required")
    os.sched_setaffinity(0, allowed[:4])
    cores = ",".join(map(str, allowed[:4]))
    for name in ("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"):
        os.environ[name] = "1"
    acquisition, source_tree, output = Path(acquisition).resolve(strict=True), Path(source_tree).resolve(strict=True), Path(output).absolute()
    output.mkdir()
    record = _load_json(acquisition / "verification.json", "verified native/source acquisition")
    check(record["native"] == load_native_receipt(acquisition / "native/receipt.json") and
          record["native_workflows"] == dict(schema=1, product="rsomics-limma", workflow_count=1,
              workers=[1, 4], complete_contrast_states=3, export_tables=7), "measurement lacks accepted complete native admission")
    witness = acquisition / "source/artifact/workflow"
    native_root = acquisition / "native/artifact"
    binary = native_root / "native-binary/rsomics-limma-linux-x86_64"
    candidate = acquisition / "native/candidate"
    checker = candidate / "scripts/check-workflow.py"
    check(_sha256(binary) == record["native"]["binary_sha256"] == record["binary_sha256"] and
          _sha256(checker) == record["checker_sha256"], "measurement binary/comparator changed after admission")
    qualification = output / "source-qualification"
    run_child(["Rscript", "--vanilla", str(ROOT / "scripts/limma_workflow_performance.R"), "qualify",
               str(source_tree / "edgeR"), str(source_tree / "limma"), str(qualification)],
              output / "source-qualification-process", timeout=1800)
    runtime = _load_json(qualification / "runtime.json", "source runtime")
    immutable = {**{candidate / name: digest for name, digest in record["native_source_files"].items()},
                 **{witness / name: digest for name, digest in record["source_files"].items()},
                 binary: record["binary_sha256"], witness / "sha256-manifest.tsv": SOURCE["manifest_sha256"]}
    for package in PACKAGES:
        with (qualification / f"installed-{package}-sha256.tsv").open(newline="") as stream:
            immutable.update({Path(runtime[package]["installed_path"]) / row["path"]: row["sha256"]
                              for row in csv.DictReader(stream, delimiter="\t")})
    for tree in (source_tree, qualification, native_root / "comparison"):
        immutable.update({path: _sha256(path) for path in tree.rglob("*") if path.is_file()})
    immutable.update({ROOT / "scripts" / name: _sha256(ROOT / "scripts" / name) for name in CONTROLLERS})
    workflow = ROOT / ".github/workflows/limma-performance.yml"
    immutable[workflow] = _sha256(workflow)
    for relative in ("verification.json", "native/receipt.json", "source/receipt.json"):
        path = acquisition / relative
        immutable[path] = _sha256(path)

    def verify_immutable():
        check(all(_sha256(path) == digest for path, digest in immutable.items()), "source/input/binary/controller bytes changed")

    verify_immutable()
    save(output / "immutability-before.json", {str(path): value for path, value in immutable.items()})
    save(output / "bounds.json", BOUNDS)
    save(output / "contract.json", dict(native=record["native"], binary_sha256=record["binary_sha256"], source=SOURCE,
        packages=PACKAGES, bounds=BOUNDS, positive_probability_log_bound=LOG_PROBABILITY_BOUND,
        input=dict(raw_genes=14599, samples=7, kept_genes=8066, prefilter=False),
        source_inputs=["pasilla_gene_counts.tsv.gz", "pasilla_sample_annotation.csv"],
        native_inputs=["counts.tsv", "samples.tsv", "model-matrix.tsv"],
        source_warmups=1, source_repetitions=3, native_warmups_per_worker=1, native_repetitions_per_worker=5,
        workers=[1, 4], native_order="alternate [1,4] and [4,1]", source_default_serial=True,
        source_encoding="default gzip-compressed full analysis RDS and three full contrast/moderated/table RDS; original gzip/CSV input read and source design construction",
        native_encoding="portable uncompressed full analysis JSON and three contrast JSON; same-value uncompressed named TSV inputs",
        units="native wall/CPU/I/O/faults sum eight real CLI processes; peak RSS is max process KiB, not sum or aggregate live memory; GNUtime filesystem counters are blocks, not logical bytes",
        throughput_rule="source inner minimum > native complete-path maximum; OR source process-RSS minimum > native maximum; no post-result retuning",
        cross_host_policy="earlier runner byte equality is diagnostic; same-host full state ignoring only workers and all three states/seven tables must be deterministic"))
    save(output / "host.json", dict(platform=platform.platform(), machine=platform.machine(), cpu_affinity=list(os.sched_getaffinity(0)),
        cpuinfo=Path("/proc/cpuinfo").read_text(), meminfo=Path("/proc/meminfo").read_text(), load_average=os.getloadavg(),
        disk=shutil.disk_usage(output)._asdict(), source_runtime=runtime, time_format=TIME_FORMAT,
        native_rustc=(native_root / "rustc.log").read_text(), native_cargo=(native_root / "cargo.log").read_text()))
    save(output / "input-bytes.json", {path.name: dict(size_bytes=path.stat().st_size, sha256=_sha256(path))
        for path in (witness / "inputs").iterdir() if path.is_file()})
    source, native = [], {1: [], 4: []}
    baseline, baseline_hashes = None, None
    source_baseline, source_byte_diagnostics = None, []
    try:
        warm_source = source_trial(output / "source-warmup", witness, qualification, cores)
        source_baseline = warm_source["outputs"]
        for workers in (1, 4):
            fitted, row = native_trial(output / f"native-warmup-{workers}", witness, binary, checker, workers, cores,
                                       native_root / f"comparison/workers-{workers}")
            hashes = {name: row["outputs"][name]["sha256"] for name in (*STATES, *OUTPUTS)}
            if baseline is None:
                baseline, baseline_hashes = fitted, hashes
            check(fitted == baseline and hashes == baseline_hashes, "native warmup full state/state/table determinism failed")
        for index in range(5):
            if index < 3:
                row = source_trial(output / f"source-{index+1}", witness, qualification, cores)
                source_byte_diagnostics.append({name: row["outputs"][name] == source_baseline[name]
                                                for name in source_baseline})
                source.append(row)
                print(f"source repetition {index+1}/3: actual saved states/outputs match fixed source", flush=True)
            for workers in ((1, 4) if index % 2 == 0 else (4, 1)):
                fitted, row = native_trial(output / f"native-{index+1}-workers-{workers}", witness, binary, checker,
                    workers, cores, native_root / f"comparison/workers-{workers}")
                check(fitted == baseline and all(row["outputs"][name]["sha256"] == baseline_hashes[name] for name in baseline_hashes),
                      "same-host native full state/state/table determinism failed")
                native[workers].append(row)
                print(f"native repetition {index+1}/5 workers={workers}: actual saved states/outputs match fixed source", flush=True)
            verify_immutable()
    finally:
        verify_immutable()
        save(output / "immutability-after.json", dict(unchanged=True, checked_files=len(immutable)))
    stats = dict(source=distribution(source), native={str(workers): distribution(rows) for workers, rows in native.items()})
    stages = {str(workers): {name: distribution([row["stages"][name] for row in rows]) for name in rows[0]["stages"]}
              for workers, rows in native.items()}
    decision = performance_decision(stats)
    summary = dict(status="complete" if decision["advantage_passed"] else "rejected_no_advantage", native=record["native"],
        source=SOURCE, binary_sha256=record["binary_sha256"], statistics=stats, native_stages=stages,
        source_trials=source, native_trials=native, decision=decision, same_host_complete_state_and_outputs_equal=True,
        native_exports_sha256=baseline_hashes, source_outputs=source_baseline,
        source_trial_byte_equality_diagnostic=source_byte_diagnostics,
        scope="one complete original14599x7 current-voom lifecycle, warm caches, serial R versus accepted native1/4; not broad limma coverage, fastest-tuned R, cold I/O, large-cohort scaling or publication acceptance")
    save(output / "summary.json", summary)
    check(decision["advantage_passed"], "complete workflow measured but no strict replacement advantage")
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
            print(select_native(args.acquisition)["artifact_id"])
        elif args.command == "prepare-runtime":
            prepare_runtime(args.acquisition)
        else:
            summary = run_benchmark(args.acquisition, args.source_tree, args.output)
            print(json.dumps(dict(status=summary["status"], decision=summary["decision"])), flush=True)
    except (OSError, ValueError, KeyError, TrialProcessError, tarfile.TarError) as error:
        if args.command == "run" and args.output.is_dir() and not (args.output / "failure.json").exists():
            save(args.output / "failure.json", dict(error=f"{type(error).__name__}: {error}"))
        parser.exit(1, f"complete limma workflow measurement failed: {error}\n")


if __name__ == "__main__":
    main()
