"""Controller contracts for separate reference-only R evidence."""

import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).with_name("validate_infercnv_samples_reference_edges.py")
ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "scripts/infercnv_samples_reference_edges.R"
PIN = {"name": "infercnv", "version": "1.28.0",
       "source_commit": "b421d9405c97a309b081ef86d455e976df93eae4",
       "source_archive_sha256": "b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207"}


def shape(kind="NULL", values=None):
    result = {"typeof": kind, "classes": ["NULL" if kind == "NULL" else kind],
              "length": 0, "names": None}
    if kind == "list":
        result["items"] = [] if values is None else values
        result["length"] = len(result["items"])
    elif kind != "NULL":
        result["values"] = [] if values is None else values
        result["length"] = len(result["values"])
    return result


class ReferenceEdgesTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue(SCRIPT.is_file(), "separate reference-only validator is missing")
        spec = importlib.util.spec_from_file_location("reference_edges", SCRIPT)
        self.validator = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.validator)
        self.temp = tempfile.TemporaryDirectory(dir=os.environ.get("TMPDIR"))
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.record = self.fixture()

    def fixture(self):
        files = {name: b"retained bytes\n" for name in (
            "input.rds", "grouped.rds", "pooled.rds", "grouped.log", "pooled.log",
            "function-main.txt", "function-single.txt", "source.R", "source.tar.gz",
            "exporter.R", "io.R")}
        files.update({"expression.tsv": ("gene\tcell1\tcell2\tcell3\n" + "".join(
            f"gene{i}\t{1+i/8}\t{1+(i+4)/8}\t{1+(i+8)/8}\n" for i in range(1, 5))).encode(),
            "genes.tsv": ("gene\tchr\tstart\tstop\n" + "".join(
                f"gene{i}\tchrProbe\t{i*100}\t{i*100+49}\n" for i in range(1, 5))).encode(),
            "cells.tsv": b"cell\tgroup\trole\ncell1\tref\treference\ncell2\tref\treference\ncell3\tref\treference\n",
            "maps.tsv": b"role\tgroup\tindex\tcell\nreference\tref\t1\tcell1\nreference\tref\t2\tcell2\nreference\tref\t3\tcell3\n"})
        arguments = {"p_val": 0.1, "k_nn": 20, "leiden_method": "PCA", "leiden_function": "CPM",
                     "leiden_resolution": "auto", "leiden_method_per_chr": "simple",
                     "leiden_function_per_chr": "modularity", "leiden_resolution_per_chr": 1,
                     "hclust_method": "ward.D2", "partition_method": "none",
                     "per_chr_hmm_subclusters": False, "per_chr_hmm_subclusters_references": False,
                     "z_score_filter": 0, "restrict_to_DE_genes": False}
        files["arguments.json"] = json.dumps(arguments).encode()
        for name, data in files.items():
            (self.root / name).write_bytes(data)
        runtime = {"R_version": "4.6.1", "locale": "C", "seed": 260930, "threads": 1,
                   "package_versions": {p: "1.0" for p in self.validator.PACKAGES},
                   "loaded_paths": {p: f"/external/library/{p}" for p in self.validator.PACKAGES},
                   "source_root": "/external/source", "source_archive": "source.tar.gz",
                   "source_files": ["source.R"], "functions": ["function-main.txt", "function-single.txt"],
                   "bindings": {"hclust": "fastcluster::hclust", "parallelDist": "parallelDist::parallelDist"},
                   "exporter": "exporter.R", "io": "io.R", "arguments_file": "arguments.json"}
        runtime["package_versions"]["infercnv"] = "1.28.0"
        return {"schema_version": 1, "package": PIN, "runtime": runtime,
                "input": {"classes": ["infercnv"], "genes": 4, "cells": 3,
                          "reference_groups": ["ref"], "observation_groups": [],
                          "rds": "input.rds", "state": {k: f"{k}.tsv" for k in ("expression", "genes", "cells", "maps")}},
                "arguments": arguments,
                "routes": [{"id": route, "status": "success", "warnings": [], "error": None,
                            "log": f"{route}.log", "raw_rds": f"{route}.rds",
                            "result_shape": {"tumor_subclusters": shape("list"), "second": shape()},
                            "unchanged_state": True} for route in ("grouped", "pooled")],
                "files": [{"path": name, "sha256": hashlib.sha256(files[name]).hexdigest()}
                          for name in sorted(files)]}

    def validate(self):
        (self.root / "witness.json").write_text(json.dumps(self.record))
        # Tiny fixtures exercise schema without pretending to contain the real archive.
        self.validator.SOURCE_PINS = {
            "source_archive": hashlib.sha256((self.root / "source.tar.gz").read_bytes()).hexdigest(),
            "source_files": [hashlib.sha256((self.root / "source.R").read_bytes()).hexdigest()],
            "functions": [hashlib.sha256((self.root / name).read_bytes()).hexdigest()
                          for name in ("function-main.txt", "function-single.txt")],
            "io": hashlib.sha256((self.root / "io.R").read_bytes()).hexdigest()}
        return self.validator.validate(self.root)

    def test_success_and_error_are_observations(self):
        self.validate()
        self.record["routes"][1].update(status="error", error={"classes": ["simpleError", "error", "condition"],
            "message": "actual error", "call": None}, result_shape=None, unchanged_state=None)
        self.validate()

    def test_null_empty_list_and_empty_indices_remain_distinct(self):
        for value in (shape(), shape("list"), shape("integer"), shape("double", ["NA", "NaN", "Inf", "-Inf", "1.2345678901234567"])):
            self.record["routes"][0]["result_shape"]["second"] = value
            self.assertEqual(self.validate()["routes"][0]["result_shape"]["second"], value)

    def test_absent_list_key_differs_from_present_null(self):
        absent = shape("list")
        present = shape("list", [shape()])
        present["names"] = ["hc"]
        self.assertNotEqual(absent, present)
        for value in (absent, present):
            self.record["routes"][0]["result_shape"]["tumor_subclusters"] = value
            self.validate()

    def test_missing_fields_are_rejected(self):
        original = copy.deepcopy(self.record)
        for group in (None, "runtime", "input", "arguments"):
            target = original if group is None else original[group]
            for key in target:
                with self.subTest(group=group, key=key):
                    self.record = copy.deepcopy(original)
                    del (self.record if group is None else self.record[group])[key]
                    with self.assertRaises(ValueError):
                        self.validate()

    def test_route_omissions_and_invalid_results_fail(self):
        original = copy.deepcopy(self.record)
        changes = [("status", "unknown"), ("unchanged_state", False), ("unchanged_state", 1),
                   ("result_shape", None), ("warnings", "warning"), ("error", {}),
                   ("raw_rds", "missing.rds"), ("id", "grouped")]
        for key, value in changes:
            with self.subTest(key=key, value=value):
                self.record = copy.deepcopy(original)
                self.record["routes"][1][key] = value
                with self.assertRaises(ValueError):
                    self.validate()
        self.record = copy.deepcopy(original)
        self.record["routes"].pop()
        with self.assertRaises(ValueError):
            self.validate()

    def test_shape_type_names_lengths_and_omissions_fail(self):
        for value in ({}, {"typeof": "NULL"}, shape("integer", [1]),
                      dict(shape("list"), length=1), dict(shape(), names=[]),
                      dict(shape(), length=True), dict(shape(), values=[]),
                      dict(shape("integer"), classes="integer")):
            with self.subTest(value=value):
                self.record["routes"][0]["result_shape"]["second"] = value
                with self.assertRaises(ValueError):
                    self.validate()

    def test_file_hash_and_complete_inventory(self):
        self.record["files"][0]["sha256"] = "0" * 64
        with self.assertRaises(ValueError):
            self.validate()
        self.record = self.fixture()
        (self.root / "unlisted").write_text("bytes")
        with self.assertRaises(ValueError):
            self.validate()

    def test_path_alias_escape_and_symlink_fail(self):
        for path in ("../outside", "/absolute", "./input.rds", "a//b", "a\\b", "witness.json"):
            self.record = self.fixture()
            self.record["files"][0]["path"] = path
            with self.subTest(path=path), self.assertRaises(ValueError):
                self.validate()
        self.record = self.fixture()
        (self.root / "link").symlink_to(self.root / "input.rds")
        with self.assertRaises(ValueError):
            self.validate()

    def test_runtime_source_and_input_pins_fail(self):
        original = copy.deepcopy(self.record)
        for group, key, value in (("runtime", "R_version", "4.6.0"), ("runtime", "seed", True),
                ("runtime", "threads", 2), ("runtime", "locale", "en_US"),
                ("runtime", "source_archive", "input.rds"), ("runtime", "functions", []),
                ("runtime", "loaded_paths", {}), ("runtime", "bindings", {}),
                ("input", "genes", 5), ("input", "cells", True),
                ("input", "observation_groups", ["obs"]), ("package", "version", "1.27.0")):
            self.record = copy.deepcopy(original)
            self.record[group][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.validate()

    def test_cli_explicit_error(self):
        process = subprocess.run([sys.executable, "-B", str(SCRIPT), str(self.root)], capture_output=True, text=True)
        self.assertNotEqual(process.returncode, 0)
        self.assertIn("reference-only evidence invalid:", process.stderr)

    def test_route_fields_and_error_forms_are_complete(self):
        original = copy.deepcopy(self.record)
        for key in original["routes"][0]:
            self.record = copy.deepcopy(original)
            del self.record["routes"][0][key]
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.validate()
        self.record = copy.deepcopy(original)
        error_route = self.record["routes"][1]
        error_route.update(status="error", result_shape=None, unchanged_state=None)
        for error in (None, {}, {"classes": ["error"], "message": "actual"},
                      {"classes": [], "message": "actual", "call": None},
                      {"classes": ["error"], "message": "", "call": None},
                      {"classes": ["error"], "message": "actual", "call": []}):
            error_route["error"] = error
            with self.subTest(error=error), self.assertRaises(ValueError):
                self.validate()

    def test_inventory_order_duplicates_and_missing_bytes_fail(self):
        original = copy.deepcopy(self.record)
        for entries in (list(reversed(original["files"])), original["files"] + [original["files"][0]],
                        original["files"][1:]):
            self.record = copy.deepcopy(original)
            self.record["files"] = entries
            with self.assertRaises(ValueError):
                self.validate()
        self.record = copy.deepcopy(original)
        (self.root / "grouped.rds").unlink()
        with self.assertRaises(ValueError):
            self.validate()

    def test_source_and_function_hashes_are_independently_pinned(self):
        self.validate()
        for key in ("source_archive", "source_files", "io", "functions"):
            before = copy.deepcopy(self.validator.SOURCE_PINS)
            self.validator.SOURCE_PINS[key] = ["0" * 64] if isinstance(before[key], list) else "0" * 64
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.validator.validate(self.root)
            self.validator.SOURCE_PINS = before

    def test_argument_boolean_and_integer_types_are_not_interchangeable(self):
        self.record["arguments"]["z_score_filter"] = False
        (self.root / "arguments.json").write_text(json.dumps(self.record["arguments"]))
        for entry in self.record["files"]:
            if entry["path"] == "arguments.json":
                entry["sha256"] = hashlib.sha256((self.root / "arguments.json").read_bytes()).hexdigest()
        with self.assertRaises(ValueError):
            self.validate()

    def test_capture_exclusive_output_and_real_factory(self):
        source = SOURCE.read_text()
        self.assertIn('if (file.exists(root)) stop(', source)
        self.assertIn('infercnv::CreateInfercnvObject(', source)
        self.assertNotIn('@observation_grouped_cell_indices <-', source)
        self.assertNotIn('@reference_grouped_cell_indices <-', source)
        self.assertNotIn('source(file.path(script_dir, "infercnv_samples_clustering_witness.R")', source)
        self.assertIn('matrix(seq_len(12L)/8+1, 4L, 3L)', source)

    def test_locale_initializer_is_sourceable_without_capture(self):
        source = SOURCE.read_text()
        self.assertIn('initialize_samples_locale <- function()', source)
        self.assertIn('    initialize_samples_locale()', source)
        self.assertTrue(source.rstrip().endswith('if (sys.nframe() == 0L) main()'))
        regression = SOURCE.with_name('test_infercnv_samples_reference_locale.R').read_text()
        self.assertIn('source(file.path(dirname(script), "infercnv_samples_reference_edges.R"), local=capture)', regression)
        self.assertIn('capture$initialize_samples_locale()', regression)
        self.assertIn('"4.6.1"', regression)
        self.assertIn('"C.UTF-8"', regression)
        for category in ('LC_MESSAGES', 'LC_PAPER', 'LC_MEASUREMENT'):
            self.assertIn(f'"{category}"', regression)

    def test_workflow_routes_locale_regression_before_dependencies(self):
        workflow = (ROOT / ".github/workflows/infercnv-oracle.yml").read_text()
        setup = workflow.index('      - uses: r-lib/actions/setup-r@v2')
        start = workflow.index('      - name: Test samples locale contract')
        self.assertLess(setup, start)
        self.assertLess(start, workflow.index('      - name: Install JAGS and system headers'))
        self.assertLess(start, workflow.index('      - name: Download pinned source and record byte identity'))
        step = workflow[start:workflow.index('\n      - name:', start + 1)]
        self.assertIn("if: env.SAMPLES_CLUSTERING == 'true'", step)
        self.assertIn('set -euo pipefail', step)
        self.assertIn('Rscript --vanilla scripts/test_infercnv_samples_reference_locale.R', step)

    def test_workflow_uses_existing_opt_in_and_adjacent_order(self):
        workflow = (ROOT / ".github/workflows/infercnv-oracle.yml").read_text()
        start = workflow.index('      - name: Capture reference-only samples edges')
        canonical = workflow.index('"$EVIDENCE_DIR/samples-clustering-validation.log"')
        self.assertLess(canonical, start)
        step = workflow[start:workflow.index('\n      - name:', start + 1)]
        self.assertIn("if: env.SAMPLES_CLUSTERING == 'true'", step)
        self.assertIn('shell: bash', step)
        self.assertIn('set -euo pipefail', step)
        self.assertIn('"$EVIDENCE_DIR/infercnv-$INFER_CNV_SOURCE_COMMIT"', step)
        self.assertIn('scripts/validate_infercnv_samples_reference_edges.py', step)

    def test_workflow_bash_routes_real_source_root_and_new_output(self):
        workflow = (ROOT / ".github/workflows/infercnv-oracle.yml").read_text()
        start = workflow.index('      - name: Capture reference-only samples edges')
        step = workflow[start:workflow.index('\n      - name:', start + 1)]
        script = '\n'.join(line[10:] for line in step.split('        run: |\n', 1)[1].splitlines())
        tools = self.root / "tools"
        tools.mkdir()
        recorder = tools / "recorder"
        recorder.write_text(f'#!{sys.executable}\nimport json, os, sys\n'
                            'with open(os.environ["COMMAND_LOG"], "a") as stream:\n'
                            '    stream.write(json.dumps([os.path.basename(sys.argv[0]), *sys.argv[1:]]) + "\\n")\n')
        recorder.chmod(0o755)
        for name in ("python3", "Rscript", "sha256sum"):
            (tools / name).symlink_to(recorder)
        evidence = self.root / "evidence with spaces"
        evidence.mkdir()
        log = self.root / "commands.jsonl"
        env = dict(os.environ, PATH=f"{tools}:{os.environ['PATH']}", EVIDENCE_DIR=str(evidence),
                   INFER_CNV_SOURCE_COMMIT=PIN["source_commit"], COMMAND_LOG=str(log))
        process = subprocess.run(["bash", "-c", script], env=env, capture_output=True, text=True)
        self.assertEqual(process.returncode, 0, process.stderr)
        commands = [json.loads(line) for line in log.read_text().splitlines()]
        self.assertEqual([cmd[0] for cmd in commands], ["python3", "sha256sum", "Rscript", "python3"])
        self.assertEqual(commands[2], ["Rscript", "--vanilla", "scripts/infercnv_samples_reference_edges.R",
            str(evidence / f"infercnv-{PIN['source_commit']}"), str(evidence / "samples-reference-edges")])
        self.assertEqual(commands[3][-2:], ["scripts/validate_infercnv_samples_reference_edges.py",
                                         str(evidence / "samples-reference-edges")])


if __name__ == "__main__":
    unittest.main()
