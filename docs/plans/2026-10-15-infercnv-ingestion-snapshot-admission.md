# Raw-creation native snapshot admission

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans.

**Goal:** Reject prepared-only green source evidence at the raw-creation launcher boundary.

**Architecture:** New private wrapper delegates existing receipt/run/source/lock/archive/extraction guards unchanged. It additionally requires four successful unique raw trial/support steps on all four native targets, the pinned external oracle, and exact raw bench/test Cargo registrations and source files. Existing prepared verification is unchanged.

**Tech Stack:** Python3.11+ standard library, existing snapshot verifier and its test fixture.

**Spec:** `2026-09-30-infercnv-raw-ingestion-measurement-design.md`, implementation/acceptance steps2 and4.

## Constraints and review focus

- No Rust or old verifier changes, no public library or new receipt schema.
- No local build/R/install/download; external TMPDIR and bytecode disabled.
- Missing/skipped/failed/pending/duplicate raw steps on any native target reject before extraction.
- A prepared-only successful run remains accepted by its old consumer, never by this raw entry.
- Raw bench must have the exact path, harness=false, test=false and infercnv-measurement feature; both raw contract targets must register their exact paths/features and refer to regular files.
- Existing source/lock/receipt/run checks remain delegated; partial rejected extraction is preserved.
- R package run38690064479 is independent; do not alter its frozen head or accept it before original evidence review.

## Task 1: Behavioral RED

**Files:** Create scripts/test_verify_infercnv_ingestion_snapshot.py.
**Interfaces:** Reuse prepared test fixture only for building authenticated archives/APIs, without importing its TestCase into this module's discovery namespace. Consume new verify_ingestion_snapshot with the same five paths as the old verifier.

- [x] Add complete tests below, implementation absent.
- [x] Run TMPDIR=/Volumes/KIOXIA/Developments/tmp PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s scripts -p test_verify_infercnv_ingestion_snapshot.py. Require ModuleNotFoundError naming verify_infercnv_ingestion_snapshot.
- [x] Preserve raw log externally. No unrelated import, fixture, environment or parse error counts as intended RED.

```python
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
```

## Task 2: Minimal wrapper GREEN

**Files:** Create scripts/verify_infercnv_ingestion_snapshot.py.
**Produces:** verify_ingestion_snapshot(receipt_path, run_path, jobs_path, snapshot_dir, output_dir) -> Path; CLI mirrors the old five-path interface without changing old behavior.

- [x] After actual RED, add complete wrapper below; require all9 tests plus the old verifier suite pass.
- [x] Run full controller suite (283), architecture/whitespace, and the new CLI on original green-14 receipt/run/jobs to a fresh external directory; require149 exact source files and original four-native steps.
- [x] Fresh-context whole-change review; reproduce Important issues with specific RED/GREEN tests, ledger minor deferrals. Commit only new test/wrapper and this plan/ledger: test(sc): require raw-creation native source evidence.
- [x] Push main and wait exact-head Control plane CI (57721add,38690986186 success). No duplicate installed R run was dispatched for this component; R test diagnostics have their separate original failure and rerun ledger.

```python
"""Verify four-native raw-creation tests and the frozen private trial source."""

import argparse
from pathlib import Path
import tarfile
import tomllib

import verify_infercnv_measurement_snapshot as prepared


RAW_STEPS = ("Test ingestion trial debug", "Test ingestion trial release and compile bench",
             "Test ingestion support debug", "Test ingestion support release")


def _raw_jobs(receipt, run, jobs):
    prepared._run_jobs(receipt, run, jobs)
    for job in jobs["jobs"]:
        for name in RAW_STEPS:
            records = [step for step in job["steps"] if step.get("name") == name]
            if len(records) != 1 or records[0].get("status") != "completed" or records[0].get("conclusion") != "success":
                raise ValueError(f"raw creation job {job['name']} requires successful unique step: {name}")
        external = next(step for step in job["steps"] if step.get("name") == "Acquire and verify pinned external oracle")
        if external["conclusion"] != "success":
            raise ValueError("raw creation requires the pinned external oracle on all targets")


def _raw_cargo(root):
    try:
        cargo = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    except (OSError, UnicodeError, tomllib.TOMLDecodeError) as exc:
        raise ValueError("invalid raw creation Cargo.toml") from exc
    expected = {
        "bench": (("cnv_ingestion_matched", "benches/cnv_ingestion_matched.rs"),),
        "test": (("cnv_ingestion_trial_contract", "tests/cnv_ingestion_trial_contract.rs"),
                 ("cnv_ingestion_measurement_contract", "tests/cnv_ingestion_measurement_contract.rs")),
    }
    for kind, targets in expected.items():
        collection = cargo.get(kind, [])
        if not isinstance(collection, list) or any(not isinstance(entry, dict) for entry in collection):
            raise ValueError(f"raw creation Cargo target collection mismatch: {kind}")
        for name, path in targets:
            entries = [entry for entry in collection if entry.get("name") == name]
            if len(entries) != 1 or entries[0].get("path") != path or \
                    entries[0].get("required-features") != ["infercnv-measurement"] or \
                    kind == "bench" and (entries[0].get("harness") is not False or entries[0].get("test") is not False):
                raise ValueError(f"raw creation Cargo target mismatch: {name}")
            prepared._regular(root / path)


def verify_ingestion_snapshot(receipt_path, run_path, jobs_path, snapshot_dir, output_dir):
    receipt = prepared._read_json(receipt_path)
    prepared._receipt(receipt)
    _raw_jobs(receipt, prepared._read_json(run_path), prepared._read_json(jobs_path))
    root = prepared.verify_snapshot(receipt_path, run_path, jobs_path, snapshot_dir, output_dir)
    _raw_cargo(root)
    return root


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("receipt", "run-json", "jobs-json", "snapshot-dir", "output-dir"):
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    try:
        root = verify_ingestion_snapshot(args.receipt, args.run_json, args.jobs_json, args.snapshot_dir, args.output_dir)
    except (OSError, ValueError, tarfile.TarError) as exc:
        parser.exit(1, f"raw creation snapshot verification failed: {exc}\n")
    print(f"verified raw creation source: {root}")


if __name__ == "__main__":
    main()
```

## Pre-flight ruling

The existing native trial receipt was accepted by manual original-API/artifact/log audit including all raw steps and raw Cargo registration. Its existing base source verifier checks exact bytes but does not automatically enforce those extra raw gates. This new consumer wrapper encodes the already-required distinction; it does not invalidate the prior audited acceptance or redefine old prepared receipts. If wrong, raw future admission could accept prepared-only evidence; direct old/new consumer regression tests disprove that. Raw runtime/campaign root inventory and the authenticated process launcher remain later work.
