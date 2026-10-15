import importlib.util
import copy
from dataclasses import replace
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile
import unittest
from unittest.mock import patch

try:
    import measure_infercnv_ingestion as owner
except ModuleNotFoundError:
    owner = None

from infercnv_ingestion_campaign import expected_schedule
import test_infercnv_ingestion_campaign as campaign_fixtures
from test_validate_infercnv_trial_metadata import Fixture


class ExternalFixture(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir=os.environ["TMPDIR"])
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / "literal space root"
        self.root.mkdir()

    def context(self):
        directories = {"artifact_root", "r_source_root", "native_snapshot_dir",
                       "native_source_root", "cargo_target_dir", "contract_root"}
        result = {"schema_version": 1, "code_sha256": dict(owner.FIXED_CODE)}
        result["code_sha256"]["scripts/measure_infercnv_ingestion.py"] = owner._sha256(Path(owner.__file__))
        for key in owner.CONTEXT_KEYS - {"schema_version", "code_sha256"}:
            path = self.root / key
            if key in directories:
                path.mkdir()
            else:
                path.write_text("fixture\n")
            result[key] = str(path)
        path = self.root / "context.json"
        path.write_text(json.dumps(result))
        return path, result


@unittest.skipIf(owner is None, "owner not yet implemented")
class AdmissionTests(ExternalFixture):
    def test_closed_context_rejects_unknown_missing_duplicate_nonfinite_and_bool_schema(self):
        path, data = self.context()
        self.assertEqual(owner._load_context(path), data)
        variants = [dict(data, extra=True), {k: v for k, v in data.items() if k != "python"},
                    dict(data, schema_version=True), dict(data, python=3),
                    dict(data, code_sha256={}), dict(data, code_sha256=dict(data["code_sha256"], extra="a" * 64))]
        for bad in variants:
            with self.subTest(bad=bad):
                path.write_text(json.dumps(bad))
                with self.assertRaises(ValueError):
                    owner._load_context(path)
        for raw in ('{"schema_version":1,"schema_version":1}', '{"x":NaN}', '\ufeff{}'):
            path.write_text(raw)
            with self.assertRaises(ValueError):
                owner._load_context(path)

    def test_context_rejects_noncanonical_symlink_and_directory_file_confusion(self):
        path, data = self.context()
        for value in ("relative", str(self.root) + "/./python", str(self.root) + "/../python", data["python"] + "/"):
            path.write_text(json.dumps(dict(data, python=value)))
            with self.subTest(value=value), self.assertRaises(ValueError):
                owner._load_context(path)
        link = self.root / "alias"
        link.symlink_to(self.root, target_is_directory=True)
        path.write_text(json.dumps(dict(data, python=str(link / "python"))))
        with self.assertRaises(ValueError):
            owner._load_context(path)
        path.write_text(json.dumps(dict(data, python=data["artifact_root"])))
        with self.assertRaises(ValueError):
            owner._load_context(path)

    def test_pinned_code_and_runtime_environment_are_not_self_issued(self):
        _, context = self.context()
        owner._check_code(context)
        for key in ("scripts/infercnv_trial_process.py", "scripts/measure_infercnv_ingestion.py"):
            bad = copy.deepcopy(context)
            bad["code_sha256"][key] = "0" * 64
            with self.subTest(key=key), self.assertRaises(ValueError):
                owner._check_code(bad)
        for key in ("PYTHONPATH", "PYTHONHOME"):
            with patch.dict(os.environ, {key: ""}), self.assertRaises(ValueError):
                owner._check_code(context)

    def test_physical_receipt_is_hashed_before_any_json_parsing(self):
        path = self.root / "invalid receipt.json"
        path.write_text("not json")
        with self.assertRaisesRegex(ValueError, "physical receipt hash"):
            owner._check_receipt(path, owner.INPUT_RECEIPT_SHA)

    def test_shadowing_bytecode_is_rejected_even_when_source_pins_match(self):
        _, context = self.context()
        shadow = self.root / "unexpected.pyc"
        shadow.write_bytes(b"not trusted bytecode")
        with patch.object(owner, "SCRIPTS", self.root), self.assertRaisesRegex(ValueError, "shadowing bytecode"):
            owner._check_code(context)

    def test_finally_dependency_loading_also_rejects_shadowing_bytecode(self):
        (self.root / "unsafe.pyc").write_bytes(b"not trusted bytecode")
        with patch.object(owner, "SCRIPTS", self.root), self.assertRaisesRegex(ValueError, "shadowing bytecode"):
            owner._module("infercnv_python_identity")

    def cargo(self):
        _, c = self.context()
        executable = Path(c["cargo_target_dir"]) / "release" / "deps" / "cnv_ingestion_matched-hash"
        executable.parent.mkdir(parents=True)
        executable.write_text("artifact")
        executable.chmod(0o700)
        record = {"reason": "compiler-artifact", "manifest_path": str(Path(c["native_source_root"]) / "Cargo.toml"),
                  "target": {"name": "cnv_ingestion_matched", "kind": ["bench"],
                             "crate_types": ["bin"], "src_path": str(Path(c["native_source_root"]) / "benches/cnv_ingestion_matched.rs")},
                  "features": ["infercnv-measurement"], "profile": {"opt_level": "3", "test": False, "debug_assertions": False},
                  "executable": str(executable), "filenames": [str(executable)]}
        return c, record, executable

    def write_cargo(self, c, records):
        Path(c["cargo_artifacts"]).write_text("".join(json.dumps(r) + "\n" for r in records))

    def test_cargo_uses_only_the_unique_optimized_successful_admitted_bench(self):
        c, record, executable = self.cargo()
        finish = {"reason": "build-finished", "success": True}
        self.write_cargo(c, [record, finish])
        self.assertEqual(owner._cargo_executable(c), executable)
        mutations = [{"target": dict(record["target"], kind=["bin"])},
                     {"target": dict(record["target"], name="cnv_matched")},
                     {"features": []}, {"manifest_path": str(self.root / "Cargo.toml")},
                     {"profile": dict(record["profile"], test=True)},
                     {"profile": dict(record["profile"], opt_level="0")},
                     {"profile": dict(record["profile"], debug_assertions=True)},
                     {"executable": str(Path(c["python"]))}]
        for change in mutations:
            self.write_cargo(c, [dict(record, **change), finish])
            with self.subTest(change=change), self.assertRaises(ValueError):
                owner._cargo_executable(c)
        for records in ([finish], [record], [record, record, finish], [record, finish, finish],
                        [record, {"reason": "build-finished", "success": False}],
                        [record, {"reason": "compiler-message", "message": {"level": "error"}}, finish]):
            self.write_cargo(c, records)
            with self.subTest(records=records), self.assertRaises(ValueError):
                owner._cargo_executable(c)

    def test_full_identity_catches_same_executable_different_library_and_every_other_field(self):
        identity = Fixture.profile()["Python"]
        owner._compare_identity(identity, identity)
        for key in identity:
            other = copy.deepcopy(identity)
            other[key] = {"/usr/lib/libpython3.12.so": "b" * 64} if key == "libpython_sha256" else identity[key] + "wrong"
            with self.subTest(key=key), self.assertRaises(ValueError):
                owner._compare_identity(other, identity)

    def test_literal_whole_process_time_rejects_bad_counters_and_nonzero_status(self):
        valid = "wall_seconds\t1.25\nuser_seconds\t0\nsystem_seconds\t0.01\nmaximum_rss_kib\t1024\nexit_status\t0\n"
        self.assertEqual(owner._parse_time(valid)["maximum_rss_kib"], 1024)
        for bad in (valid + "wall_seconds\t1\n", valid.replace("1.25", "NaN"),
                    valid.replace("1.25", "-1"), valid.replace("1.25", '"1"'), valid.replace("1.25", "1e-9999"),
                    valid.replace("1024", "0"), valid.replace("1024", str(2**64)),
                    valid.replace("exit_status\t0", "exit_status\t1"), valid.replace("\n", "\r\n")):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                owner._parse_time(bad)

    def test_factory_argv_preserves_spaces_and_exact_r_and_native_routes(self):
        _, c = self.context()
        trial = self.root / "new trial"
        native = self.root / "native binary"
        slots = expected_schedule()
        self.assertEqual(owner._factory_argv(c, native, slots[0], trial),
                         [str(native), c["artifact_root"], "plain", str(trial)])
        self.assertEqual(owner._factory_argv(c, native, slots[1], trial),
                         [c["rscript"], "--vanilla", str(owner.SCRIPTS / "infercnv_ingestion_matched.R"),
                          c["artifact_root"], "plain", c["r_source_root"], c["input_receipt"], str(owner.SCRIPTS), str(trial)])

    def test_profile_anchors_clock_and_functions_to_frozen_receipt_not_contract(self):
        path, c = self.context()
        profile = Fixture.profile()
        runtime = dict(system=profile["system"], namespaces=profile["namespaces"], blas=profile["blas"],
                       options_dput=profile["options_dput"], measured={"options_dput": profile["object_options_dput"]},
                       clock_source={"path": "clock-proc-time.txt", "sha256": "0" * 64})
        (Path(c["contract_root"]) / "runtime.json").write_text(json.dumps(runtime))
        proofs = {key: profile[key] for key in ("clock_source", "functions")}
        Path(c["installed_contract_receipt"]).write_text(json.dumps({"installed_source_proofs": proofs}))
        bindings = object()
        with patch.object(owner, "_check_receipt", return_value={"installed_source_proofs": proofs}), patch.object(owner, "_bindings", return_value=bindings), \
                patch.object(owner, "_metadata") as metadata, patch.object(owner, "_factory_results") as results:
            got = owner._build_profile(c, profile["Python"], {}, self.root / "native", "0.1.0")
        self.assertEqual(got, profile)
        self.assertEqual(metadata.call_args.args, (Path(c["contract_root"]), "infercnv", bindings))
        self.assertEqual(results.call_args.args[0], Path(c["contract_root"]))

    def test_failed_shell_identity_does_not_skip_the_independent_r_probe(self):
        _, c = self.context()
        identity = Fixture.profile()["Python"]
        c["python"] = identity["executable"]
        output = self.root / "probe evidence"
        output.mkdir()
        calls = []
        command = owner._command
        def actual_probe(argv, prefix, env):
            calls.append(argv)
            text = "malformed identity" if len(calls) == 1 else json.dumps(identity, sort_keys=True)
            return command([str(Path(sys.executable).resolve()), "-B", "-c", f"print({text!r})"], prefix, env)
        with patch.object(owner._module("infercnv_python_identity"), "capture_identity", return_value=identity), \
                patch.object(owner, "_command", actual_probe):
            with self.assertRaises(ValueError):
                owner._runtime_probes(c, output, "after", owner._environment(c), {key: identity for key in ("Python", "shell", "R")})
        self.assertEqual(len(calls), 2)
        self.assertEqual(calls[0], [c["python"], "-I", "-B", owner.SCRIPTS / "infercnv_python_identity.py"])
        self.assertEqual(calls[1][:3], [c["rscript"], "--vanilla", "-e"])
        self.assertTrue((output / "after-python-R.stdout").is_file())

    def test_generated_r_probe_inherits_owned_stdout_and_stderr(self):
        _, c = self.context()
        identity = Fixture.profile()["Python"]
        c["python"] = identity["executable"]
        output = self.root / "inherited probe streams"
        output.mkdir()
        routes = []
        command = owner._command
        def actual_probe(argv, prefix, env):
            routes.append(argv)
            code = f"import sys;print({json.dumps(identity, sort_keys=True)!r});print('retained diagnostic',file=sys.stderr)"
            return command([str(Path(sys.executable).resolve()), "-B", "-c", code], prefix, env)
        with patch.object(owner._module("infercnv_python_identity"), "capture_identity", return_value=identity), \
                patch.object(owner, "_command", actual_probe):
            owner._runtime_probes(c, output, "inherited", owner._environment(c))
        self.assertEqual(routes[1][:3], [c["rscript"], "--vanilla", "-e"])
        self.assertIn('stdout="",stderr=""', routes[1][3])
        self.assertEqual((output / "inherited-python-R.stderr").read_text(), "retained diagnostic\n")


@unittest.skipUnless(os.environ.get("RSOMICS_TEST_INSTALLED_R") == "1",
                     "set RSOMICS_TEST_INSTALLED_R=1 to require the installed R boundary")
class InstalledRBoundaryTests(ExternalFixture):
    def test_real_six_namespace_r_probe_retains_full_python_identity_and_raw_streams(self):
        owner._check_host()
        python = str(Path(sys.executable).resolve(strict=True))
        selected_rscript = shutil.which("Rscript")
        self.assertIsNotNone(selected_rscript, "installed Rscript is required when the boundary test is enabled")
        rscript = str(Path(selected_rscript).resolve(strict=True))
        context = {"python": python, "rscript": rscript,
                   "code_sha256": {"scripts/infercnv_python_identity.py":
                                    owner.FIXED_CODE["scripts/infercnv_python_identity.py"]}}
        env = owner._environment(context)
        home_prefix = self.root / "installed-R-home"
        owner._command([rscript, "--vanilla", "-e",
                        'cat(normalizePath(file.path(R.home(),"bin","exec","R"),mustWork=TRUE),"\\n",sep="")'],
                       home_prefix, env)
        r_home_output = owner._stdout(home_prefix)
        self.assertEqual(len(r_home_output.splitlines()), 1)
        context["r_executable"] = str(Path(r_home_output.rstrip("\n")).resolve(strict=True))
        self.assertEqual(r_home_output, context["r_executable"] + "\n")
        actual = owner._module("infercnv_python_identity").capture_identity()
        result = owner._runtime_probes(context, self.root, "installed", env)
        self.assertEqual(result, {key: actual for key in ("Python", "shell", "R")})
        self.assertEqual(set(actual), {"executable", "executable_sha256", "version", "prefix",
                                       "base_prefix", "stdlib_json", "libpython_sha256"})
        for surface in ("shell", "R"):
            prefix = self.root / ("installed-python-" + surface)
            self.assertEqual(owner._stdout(prefix), json.dumps(actual, sort_keys=True) + "\n")
            for suffix in (".stdout", ".stderr", ".process.json"):
                self.assertTrue(Path(str(prefix) + suffix).is_file())


@unittest.skipIf(owner is None, "owner not yet implemented")
class RouteTests(ExternalFixture):
    def setUp(self):
        super().setUp()
        self.context_path, self.c = self.context()
        self.output = self.root / "campaign output"
        self.payload = {
            "expression.tsv": "gene\tc_ref\tc_obs\ng1\t1\t2\n",
            "genes.tsv": "gene\tchr\tstart\tstop\ng1\tchr1\t1\t2\n",
            "cells.tsv": "cell\tgroup\trole\nc_ref\tr\treference\nc_obs\to\tobservation\n",
            "maps.tsv": "role\tgroup\tindex\tcell\nreference\tr\t1\tc_ref\nobservation\to\t2\tc_obs\n",
        }
        artifact = Path(self.c["artifact_root"])
        full = artifact / "shipped-bundle/full"
        full.mkdir(parents=True)
        for name, source in (("01.tsv", "expression.tsv"), ("01.genes.tsv", "genes.tsv"), ("01.cells.tsv", "cells.tsv")):
            (full / name).write_text(self.payload[source])
        maps = artifact / "ingestion-witness/shipped_full/plain/maps.tsv"
        maps.parent.mkdir(parents=True)
        maps.write_text(self.payload["maps.tsv"])
        self.wrapper = self.root / "whole process monitor"
        self.wrapper.write_text(f"#!{sys.executable}\n" +
            "import pathlib,subprocess,sys\n"
            "assert sys.argv[1]=='-f' and sys.argv[3]=='-o' and sys.argv[5]=='--'\n"
            "rc=subprocess.call(sys.argv[6:])\n"
            "pathlib.Path(sys.argv[4]).write_text('wall_seconds\\t0.1\\nuser_seconds\\t0\\nsystem_seconds\\t0\\nmaximum_rss_kib\\t100\\nexit_status\\t'+str(rc)+'\\n')\n"
            "sys.exit(rc)\n")
        self.wrapper.chmod(0o700)
        self.c["gnu_time"] = str(self.wrapper)
        installed = self.root / "historical installed receipt.json"
        installed.write_bytes((owner.ROOT / ".autopilot/oracles/infercnv-installed-creation-contracts-2026-10-15.json").read_bytes())
        self.c["installed_contract_receipt"] = str(installed)
        self.context_path.write_text(json.dumps(self.c))
        self.identity = Fixture.profile()["Python"]
        self.launched = []
        self.failure = None
        self.after_hook = None
        self.metadata_hook = None
        self.walls = {}
        self.native = Path(self.c["python"])
        self.runtime = {key: self.identity for key in ("Python", "shell", "R")}
        self.inputs = {"sha256": {"plain": "a" * 64}}
        substitutions = {
            "_check_host": lambda: None, "_check_inputs": lambda c: self.inputs,
            "_check_native": lambda c: {"native_version": "0.1.0"},
            "_cargo_executable": lambda c: self.native,
            "_check_executables": lambda c, n: {"sha256": owner._sha256(n)},
            "_runtime_probes": self.probe, "_versions": lambda *args: {"test": "bounded substitution"},
            "_build_profile": lambda *args: Fixture.profile(), "_machine": lambda: {"test": "not measurement evidence"},
            "_bindings": lambda *args: None, "_metadata": self.metadata,
            "_factory_argv": self.argv,
        }
        for name, replacement in substitutions.items():
            patcher = patch.object(owner, name, replacement)
            patcher.start()
            self.addCleanup(patcher.stop)

    def probe(self, c, output, phase, env, expected=None):
        if phase == "after" and self.after_hook:
            self.after_hook()
        return self.runtime

    def metadata(self, trial, implementation, bindings):
        if self.metadata_hook:
            self.metadata_hook(trial)
        from validate_infercnv_trial_metadata import CheckedMetadata
        return CheckedMetadata(implementation, "plain" if "plain" in trial.name else "gzip", 1, 1,
                               {"rchar": 1}, {p.relative_to(trial).as_posix(): owner._sha256(p)
                                             for p in trial.rglob("*") if p.is_file()})

    def argv(self, c, native, slot, trial):
        index = len(self.launched)
        self.launched.append((slot, trial))
        wall = self.walls.get(index, 1 if slot.implementation == "rust" else 10)
        if slot.pilot:
            wall = 999 if slot.implementation == "rust" else 0.001
        metrics = campaign_fixtures.table({"implementation": slot.implementation, "input_case": slot.input_case,
                                          "region_wall_seconds": str(wall)})
        script = ("import json,pathlib,sys\n"
                  "p=pathlib.Path(sys.argv[1]);p.mkdir()\n"
                  f"payload=json.loads({json.dumps(json.dumps(self.payload))})\n"
                  "for phase in ('warm','measured'):\n"
                  " d=p/phase;d.mkdir()\n"
                  " for name,text in payload.items(): (d/name).write_text(text)\n"
                  f"(p/'metrics.tsv').write_text({metrics!r})\n")
        if self.failure == ("child", index):
            script += "sys.exit(7)\n"
        if self.failure == ("missing", index):
            script += "(p/'warm/maps.tsv').rename(p/'warm/missing.tsv')\n"
        if self.failure == ("counter", index):
            script += "m=p/'metrics.tsv';m.write_text(m.read_text().replace('region_child_user_seconds\\t0','region_child_user_seconds\\t1'))\n"
        return [str(Path(sys.executable).resolve()), "-B", "-c", script, str(trial)]

    def run_owner(self):
        return owner.run_campaign(self.context_path, self.output)

    def test_all_32_ordered_children_keep_64_exports_and_28_nonpilot_samples(self):
        result = self.run_owner()
        self.assertEqual(len(self.launched), 32)
        independent = [(case, pair, order, implementation)
                       for case in ("plain", "gzip") for pair in range(8)
                       for order, implementation in enumerate(("rust", "infercnv") if pair % 2 or pair == 0 else ("infercnv", "rust"), 1)]
        self.assertEqual([(s.input_case, s.pair, s.order, s.implementation) for s, _ in self.launched], independent)
        self.assertEqual((result["campaign"]["validated_results"], result["campaign"]["measured_samples"]), (64, 28))
        self.assertTrue(result["strict_wall_advantage"])
        self.assertEqual(len(list(self.output.glob("*.accepted.json"))), 32)
        self.assertEqual(len(list(self.output.glob("*.process.json"))), 32)
        self.assertEqual(len(list(self.output.glob("*-result/*/maps.tsv"))), 64)
        for previous, current in zip(result["slots"], result["slots"][1:]):
            self.assertLessEqual(previous["process"]["end_ns"], current["process"]["start_ns"])
        for case in result["campaign"]["cases"]:
            self.assertEqual(case["rust"]["region_wall_seconds"]["raw"], [1.] * 7)
            self.assertEqual(case["infercnv"]["region_wall_seconds"]["raw"], [10.] * 7)
        self.assertTrue((self.output / "summary.json").is_file())

    def test_factory_error_stops_without_next_slot_and_postchecks_still_run(self):
        self.failure = ("child", 1)
        with self.assertRaisesRegex(RuntimeError, "status 7"):
            self.run_owner()
        self.assertEqual(len(self.launched), 2)
        failure = owner._read_json(self.output / "failure.json")
        self.assertEqual(failure["accepted_slots"], 1)
        self.assertEqual(len(list(self.output.glob("postcheck-*.json"))), 9)
        self.assertFalse((self.output / "summary.json").exists())
        self.assertEqual(len(list(self.output.glob("*-result"))), 2)

    def test_missing_export_stops_after_first_child(self):
        self.failure = ("missing", 0)
        with self.assertRaises(ValueError):
            self.run_owner()
        self.assertEqual(len(self.launched), 1)
        self.assertFalse((self.output / "summary.json").exists())

    def test_child_cpu_counter_rejected_before_next_slot(self):
        self.failure = ("counter", 0)
        with self.assertRaisesRegex(ValueError, "child CPU"):
            self.run_owner()
        self.assertEqual(len(self.launched), 1)

    def test_metadata_error_and_postprobe_error_preserve_original_error(self):
        def fail_metadata(trial):
            raise ValueError("original metadata mismatch")
        def fail_after():
            raise ValueError("after same exe different libpython")
        self.metadata_hook, self.after_hook = fail_metadata, fail_after
        with self.assertRaisesRegex(ValueError, "original metadata mismatch"):
            self.run_owner()
        result = owner._read_json(self.output / "failure.json")
        self.assertEqual(result["accepted_slots"], 0)
        self.assertIn("original metadata mismatch", result["error"])
        self.assertEqual([r["check"] for r in result["postcheck_failures"]], ["runtime"])
        self.assertEqual(len(list(self.output.glob("postcheck-*.json"))), 9)

    def test_postcheck_evidence_write_error_does_not_skip_remaining_checks_or_mask_factory_error(self):
        self.failure = ("child", 0)
        write = owner._write_json
        def fail_one(path, value):
            if path.name == "postcheck-code.json":
                raise OSError("postcheck evidence disk error")
            return write(path, value)
        with patch.object(owner, "_write_json", fail_one), self.assertRaisesRegex(RuntimeError, "status 7"):
            self.run_owner()
        failure = owner._read_json(self.output / "failure.json")
        self.assertEqual([r["check"] for r in failure["postcheck_failures"]], ["code"])
        self.assertTrue((self.output / "postcheck-machine.json").is_file())

    def test_postcheck_failure_preserves_complete_raw_trials_but_never_accepts(self):
        def fail():
            raise ValueError("postprobe mismatch")
        self.after_hook = fail
        with self.assertRaisesRegex(ValueError, "postchecks failed"):
            self.run_owner()
        self.assertEqual(len(self.launched), 32)
        self.assertEqual(len(list(self.output.glob("*.accepted.json"))), 32)
        self.assertFalse((self.output / "summary.json").exists())

    def test_context_drift_rejects_after_complete_trials(self):
        def drift(trial):
            self.context_path.write_text(self.context_path.read_text() + "\n")
        self.metadata_hook = drift
        with self.assertRaisesRegex(ValueError, "postchecks failed"):
            self.run_owner()
        failure = owner._read_json(self.output / "failure.json")
        self.assertEqual([row["check"] for row in failure["postcheck_failures"]], ["context"])
        self.assertFalse((self.output / "summary.json").exists())

    def test_physical_installed_receipt_drift_cannot_hide_behind_repository_code_pins(self):
        def drift(trial):
            path = Path(self.c["installed_contract_receipt"])
            path.write_bytes(path.read_bytes() + b"\n")
        self.metadata_hook = drift
        with self.assertRaisesRegex(ValueError, "postchecks failed"):
            self.run_owner()
        failure = owner._read_json(self.output / "failure.json")
        self.assertEqual([row["check"] for row in failure["postcheck_failures"]], ["installed_contract"])

    def test_input_source_and_executable_postchecks_are_independent(self):
        def changed_input(c):
            return self.inputs if not self.launched else {"sha256": {"plain": "b" * 64}}
        def changed_source(c):
            return {"native_version": "0.1.0" if not self.launched else "0.1.1"}
        def changed_executable(c, native):
            return {"sha256": "a" * 64 if not self.launched else "b" * 64}
        with patch.object(owner, "_check_inputs", changed_input), patch.object(owner, "_check_native", changed_source), \
                patch.object(owner, "_check_executables", changed_executable):
            with self.assertRaisesRegex(ValueError, "postchecks failed"):
                self.run_owner()
        failure = owner._read_json(self.output / "failure.json")
        self.assertEqual([row["check"] for row in failure["postcheck_failures"]], ["inputs", "source", "executables"])
        self.assertEqual(len(list(self.output.glob("postcheck-*.json"))), 9)

    def test_code_drift_is_rechecked_without_changing_trusted_repository_files(self):
        check = owner._check_code
        def drift(c):
            if self.launched:
                raise ValueError("code bytes drifted")
            return check(c)
        with patch.object(owner, "_check_code", drift), self.assertRaisesRegex(ValueError, "postchecks failed"):
            self.run_owner()
        failure = owner._read_json(self.output / "failure.json")
        self.assertEqual([row["check"] for row in failure["postcheck_failures"]], ["code"])

    def test_missing_process_report_never_accepts_successful_child(self):
        command = owner._command
        def alter(argv, prefix, env):
            completed = command(argv, prefix, env)
            path = Path(str(prefix) + ".process.json")
            path.rename(path.with_suffix(".preserved"))
            return completed
        with patch.object(owner, "_command", alter), self.assertRaises(OSError):
            self.run_owner()
        self.assertEqual(len(self.launched), 1)
        self.assertEqual(len(list(self.output.glob("*.accepted.json"))), 0)

    def test_out_of_order_process_bounds_are_rejected_before_next_launch(self):
        command = owner._command
        def alter(argv, prefix, env):
            completed = command(argv, prefix, env)
            return replace(completed, end_ns=completed.start_ns - 1)
        with patch.object(owner, "_command", alter), self.assertRaisesRegex(ValueError, "nonserial"):
            self.run_owner()
        self.assertEqual(len(self.launched), 1)

    def test_changed_trial_metrics_cannot_replace_original_accepted_evidence(self):
        def drift():
            path = self.launched[3][1] / "metrics.tsv"
            path.write_text(path.read_text().replace("region_wall_seconds\t10", "region_wall_seconds\t0.1"))
        self.after_hook = drift
        with self.assertRaisesRegex(ValueError, "accepted trial evidence drift"):
            self.run_owner()
        self.assertFalse((self.output / "summary.json").exists())
        self.assertEqual(len(list(self.output.glob("*.accepted.json"))), 32)

    def test_cli_nonadvantage_returns_nonzero_without_discarding_summary(self):
        self.walls[3] = 1
        with patch.object(owner, "run_campaign", wraps=owner.run_campaign):
            with self.assertRaises(SystemExit) as error:
                owner.main(["--context", str(self.context_path), "--output-dir", str(self.output)])
        self.assertNotEqual(error.exception.code, 0)
        self.assertTrue((self.output / "summary.json").exists())

    def test_equal_and_overlapping_cases_retain_complete_failed_performance_decision(self):
        self.walls[3] = 1
        self.walls[19] = 0.5
        result = self.run_owner()
        self.assertFalse(result["strict_wall_advantage"])
        self.assertEqual([case["signed_wall_gap_seconds"] for case in result["campaign"]["cases"]], [0., -0.5])
        self.assertTrue((self.output / "summary.json").is_file())
        self.assertFalse((self.output / "failure.json").exists())

    def test_preexisting_output_and_symlink_parent_are_never_overwritten(self):
        self.output.mkdir()
        sentinel = self.output / "preserved"
        sentinel.write_text("unchanged")
        with self.assertRaises(ValueError):
            self.run_owner()
        self.assertEqual(sentinel.read_text(), "unchanged")
        link = self.root / "alias-parent"
        link.symlink_to(self.root, target_is_directory=True)
        with self.assertRaises(ValueError):
            owner.run_campaign(self.context_path, link / "new-output")
        self.assertEqual(self.launched, [])


class OwnerAvailabilityTests(unittest.TestCase):
    def test_offline_campaign_owner_is_available(self):
        self.assertIsNotNone(importlib.util.find_spec("measure_infercnv_ingestion"),
                             "offline paired ingestion owner is missing")


if __name__ == "__main__":
    unittest.main()
