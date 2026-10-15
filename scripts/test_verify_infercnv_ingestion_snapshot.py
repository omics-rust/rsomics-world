import copy
import json
from pathlib import Path
import subprocess
import sys
import unittest

import test_verify_infercnv_measurement_snapshot as prepared_tests
from verify_infercnv_ingestion_snapshot import verify_ingestion_snapshot


RAW_STEPS = ("Test ingestion trial debug", "Test ingestion trial release and compile bench",
             "Test ingestion support debug", "Test ingestion support release")
RAW_CARGO = b"""
[[bench]]
name = 'cnv_ingestion_matched'
path = 'benches/cnv_ingestion_matched.rs'
harness = false
test = false
required-features = ['infercnv-measurement']

[[test]]
name = 'cnv_ingestion_trial_contract'
path = 'tests/cnv_ingestion_trial_contract.rs'
required-features = ['infercnv-measurement']

[[test]]
name = 'cnv_ingestion_measurement_contract'
path = 'tests/cnv_ingestion_measurement_contract.rs'
required-features = ['infercnv-measurement']
"""


class RawSnapshotTests(unittest.TestCase):
    def setUp(self):
        self.fixture = prepared_tests.SnapshotTest()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.serial = 0
        self.fixture.files["Cargo.toml"] += RAW_CARGO
        for path in ("benches/cnv_ingestion_matched.rs", "tests/cnv_ingestion_trial_contract.rs",
                     "tests/cnv_ingestion_measurement_contract.rs"):
            self.fixture.files[path] = b"private fixture\n"
        for job in self.fixture.jobs["jobs"]:
            job["steps"].extend({"name": name, "status": "completed", "conclusion": "success"} for name in RAW_STEPS)
            for step in job["steps"]:
                if step["name"] in ("Acquire and verify pinned external oracle", "Require explicit external oracle configuration"):
                    step["conclusion"] = "success"
        self.fixture.make_snapshot()

    def invoke(self):
        f = self.fixture
        self.serial += 1
        f.output = f.root / f"raw-{self.serial}"
        f.receipt_path.write_text(json.dumps(f.receipt))
        run, jobs = f.root / "run.json", f.root / "jobs.json"
        run.write_text(json.dumps(f.run))
        jobs.write_text(json.dumps(f.jobs))
        return verify_ingestion_snapshot(f.receipt_path, run, jobs, f.snapshot, f.output)

    def test_raw_four_native_source_extracts_exact_files(self):
        root = self.invoke()
        actual = {p.relative_to(root).as_posix(): p.read_bytes() for p in root.rglob("*") if p.is_file()}
        self.assertEqual(actual, self.fixture.files)

    def test_each_raw_step_must_be_unique_completed_success_on_each_target(self):
        original = copy.deepcopy(self.fixture.jobs)
        for index in range(4):
            for name in RAW_STEPS:
                for mode in ("missing", "duplicate", "skipped", "failure", "pending"):
                    with self.subTest(target=index, step=name, mode=mode):
                        self.fixture.jobs = copy.deepcopy(original)
                        steps = self.fixture.jobs["jobs"][index]["steps"]
                        step = next(s for s in steps if s["name"] == name)
                        if mode == "missing":
                            steps.remove(step)
                        elif mode == "duplicate":
                            steps.append(dict(step, conclusion="skipped"))
                        elif mode == "pending":
                            step["status"] = "in_progress"
                        else:
                            step["conclusion"] = mode
                        with self.assertRaisesRegex(ValueError, "raw creation job"):
                            self.invoke()
                        self.assertFalse(self.fixture.output.exists())

    def test_prepared_only_success_stays_valid_prepared_but_is_not_raw_acceptance(self):
        for job in self.fixture.jobs["jobs"]:
            job["steps"] = [step for step in job["steps"] if step["name"] not in RAW_STEPS]
        self.fixture.invoke()
        with self.assertRaisesRegex(ValueError, "raw creation job"):
            self.invoke()
        self.assertFalse(self.fixture.output.exists())

    def test_skipped_external_oracle_is_not_raw_acceptance(self):
        for job in self.fixture.jobs["jobs"]:
            for step in job["steps"]:
                if step["name"] in ("Acquire and verify pinned external oracle", "Require explicit external oracle configuration"):
                    step["conclusion"] = "skipped"
        with self.assertRaisesRegex(ValueError, "requires the pinned external oracle"):
            self.invoke()
        self.assertFalse(self.fixture.output.exists())

    def test_raw_bench_registration_path_features_and_harness_are_required(self):
        original = self.fixture.files["Cargo.toml"]
        mutations = (
            (b"name = 'cnv_ingestion_matched'", b"name = 'other'"),
            (b"path = 'benches/cnv_ingestion_matched.rs'", b"path = 'benches/cnv_matched.rs'"),
            (b"path = 'benches/cnv_ingestion_matched.rs'\nharness = false", b"path = 'benches/cnv_ingestion_matched.rs'\nharness = true"),
            (b"path = 'benches/cnv_ingestion_matched.rs'\nharness = false\ntest = false", b"path = 'benches/cnv_ingestion_matched.rs'\nharness = false\ntest = true"),
            (RAW_CARGO, RAW_CARGO.replace(b"['infercnv-measurement']", b"['external-infercnv-oracle']", 1)),
            (RAW_CARGO, RAW_CARGO + RAW_CARGO.split(b"[[test]]", 1)[0]),
        )
        for old, new in mutations:
            with self.subTest(new=new):
                self.fixture.files["Cargo.toml"] = original.replace(old, new, 1)
                self.fixture.make_snapshot()
                with self.assertRaisesRegex(ValueError, "raw creation Cargo target mismatch"):
                    self.invoke()

    def test_each_raw_test_registration_is_required(self):
        original = self.fixture.files["Cargo.toml"]
        for name in ("cnv_ingestion_trial_contract", "cnv_ingestion_measurement_contract"):
            for old, new in ((f"name = '{name}'".encode(), b"name = 'other'"),
                             (f"path = 'tests/{name}.rs'".encode(), b"path = 'tests/other.rs'")):
                with self.subTest(name=name, changed=old):
                    self.fixture.files["Cargo.toml"] = original.replace(old, new, 1)
                    self.fixture.make_snapshot()
                    with self.assertRaisesRegex(ValueError, "raw creation Cargo target mismatch"):
                        self.invoke()

    def test_registered_raw_source_file_must_exist(self):
        for path in ("benches/cnv_ingestion_matched.rs", "tests/cnv_ingestion_trial_contract.rs",
                     "tests/cnv_ingestion_measurement_contract.rs"):
            with self.subTest(path=path):
                data = self.fixture.files.pop(path)
                self.fixture.make_snapshot()
                with self.assertRaises(OSError):
                    self.invoke()
                self.fixture.files[path] = data

    def test_base_source_and_run_identity_guards_still_apply(self):
        self.fixture.run["head_sha"] = "b" * 40
        with self.assertRaisesRegex(ValueError, "identity/status"):
            self.invoke()
        self.assertFalse(self.fixture.output.exists())

    def test_malformed_test_table_has_domain_diagnostic_and_preserves_extraction(self):
        original = self.fixture.files["Cargo.toml"]
        bad = RAW_CARGO.split(b"[[test]]", 1)[0] + b"[test]\nname = 'wrong'\n"
        self.fixture.files["Cargo.toml"] = original.replace(RAW_CARGO, bad, 1)
        self.fixture.make_snapshot()
        with self.assertRaisesRegex(ValueError, "raw creation Cargo target collection mismatch: test"):
            self.invoke()
        f = self.fixture
        self.assertTrue((f.output / "Cargo.toml").is_file())
        destination = f.root / "bad-cli"
        result = subprocess.run([sys.executable, "-B", str(Path(__file__).with_name("verify_infercnv_ingestion_snapshot.py")),
            "--receipt", str(f.receipt_path), "--run-json", str(f.root / "run.json"),
            "--jobs-json", str(f.root / "jobs.json"), "--snapshot-dir", str(f.snapshot),
            "--output-dir", str(destination)], text=True, capture_output=True)
        self.assertEqual(result.returncode, 1)
        self.assertTrue(result.stderr.startswith("raw creation snapshot verification failed:"))
        self.assertNotIn("Traceback", result.stderr)
        self.assertEqual((destination / "Cargo.toml").read_bytes(), self.fixture.files["Cargo.toml"])


if __name__ == "__main__":
    unittest.main()
