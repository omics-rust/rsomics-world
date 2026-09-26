import copy
import hashlib
import io
import json
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from verify_infercnv_measurement_snapshot import verify_snapshot, verify_extracted, record_extracted_check


def digest(data):
    return hashlib.sha256(data).hexdigest()


TARGETS = ["linux-x86_64", "linux-aarch64", "macos-x86_64", "macos-aarch64"]
STEPS = ["Verify and extract frozen source", "Install Rust 1.91",
         "Verify dependency resolution and output path", "Test measurement support debug",
         "Test measurement support release and compile bench", "Test debug", "Test release",
         "Verify source remains unchanged", "Upload raw evidence"]


class SnapshotTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.snapshot = self.root / ".autopilot/snapshots/sc-cnv-core-2026-09-26/green-1"
        self.snapshot.mkdir(parents=True)
        self.receipt_path = self.root / ".autopilot/oracles/candidate.json"
        self.receipt_path.parent.mkdir(parents=True)
        self.output = self.root / "extracted"
        self.files = {"Cargo.lock": b"lock\n", "Cargo.toml": b"[package]\nname = 'rsomics-sc'\nrust-version = '1.91'\n\n[dependencies]\nrsomics-common = '=0.12.3'\n\n[features]\ninfercnv-measurement = []\n\n[target.'cfg(target_os = \"linux\")'.dev-dependencies]\nnix = { version = '=0.29.0', default-features = false, features = ['resource'] }\n\n[[bench]]\nname = 'cnv_matched'\nharness = false\ntest = false\nrequired-features = ['infercnv-measurement']\n",
                      "src/lib.rs": b"production\n", "benches/cnv_matched.rs": b"bench\n"}
        self.receipt = {"schema_version": 1, "repository": "omics-rust/rsomics-world",
                        "workflow_path": ".github/workflows/sc-cnv-core-candidate.yml",
                        "run_id": 10, "run_attempt": 2, "head_sha": "a" * 40,
                        "targets": TARGETS, "snapshot_name": "green-1",
                        "archive_sha256": "", "manifest_sha256": "",
                        "lock_sha256": digest(self.files["Cargo.lock"]),
                        "production_sha256": {"src/lib.rs": digest(self.files["src/lib.rs"])}}
        self.run = {"id": 10, "run_attempt": 2, "head_sha": "a" * 40,
                    "path": self.receipt["workflow_path"], "repository": {"full_name": self.receipt["repository"]},
                    "head_repository": {"full_name": self.receipt["repository"]},
                    "event": "workflow_dispatch", "status": "completed", "conclusion": "success"}
        self.jobs = {"total_count": 4, "jobs": [
            {"name": target, "run_id": 10, "run_attempt": 2, "head_sha": "a" * 40,
             "status": "completed", "conclusion": "success",
             "steps": [{"name": name, "status": "completed", "conclusion": "success"}
                       for name in STEPS + (["Format and strict Clippy"] if target == "linux-x86_64" else [])] +
                      [{"name": "Acquire and verify pinned external oracle", "status": "completed", "conclusion": "skipped"}] +
                      ([{"name": "Require explicit external oracle configuration", "status": "completed", "conclusion": "skipped"}]
                       if target == "linux-x86_64" else [])}
            for target in TARGETS]}
        self.make_snapshot()

    def make_snapshot(self, members=None):
        manifest = b"".join(f"{digest(data)}  {name}\n".encode() for name, data in sorted(self.files.items()))
        (self.snapshot / "files.sha256").write_bytes(manifest)
        (self.snapshot / "README.md").write_bytes(b"snapshot\n")
        with tarfile.open(self.snapshot / "source.tar.gz", "w:gz") as archive:
            for name, data, kind in members or [(n, d, tarfile.REGTYPE) for n, d in self.files.items()]:
                info = tarfile.TarInfo(name)
                info.type = kind
                info.size = len(data) if kind == tarfile.REGTYPE else 0
                archive.addfile(info, io.BytesIO(data) if kind == tarfile.REGTYPE else None)
        self.receipt["archive_sha256"] = digest((self.snapshot / "source.tar.gz").read_bytes())
        self.receipt["manifest_sha256"] = digest(manifest)
        checks = b"".join(f"{digest((self.snapshot / n).read_bytes())}  {n}\n".encode()
                          for n in ("README.md", "files.sha256", "source.tar.gz"))
        (self.snapshot / "checksums.sha256").write_bytes(checks)

    def invoke(self):
        self.receipt_path.write_text(json.dumps(self.receipt))
        run_path, jobs_path = self.root / "run.json", self.root / "jobs.json"
        run_path.write_text(json.dumps(self.run))
        jobs_path.write_text(json.dumps(self.jobs))
        return verify_snapshot(self.receipt_path, run_path, jobs_path, self.snapshot, self.output)

    def test_valid_snapshot_extracts_exact_files(self):
        self.invoke()
        self.assertEqual({str(p.relative_to(self.output)) for p in self.output.rglob("*") if p.is_file()}, set(self.files))
        self.assertEqual((self.output / "src/lib.rs").read_bytes(), self.files["src/lib.rs"])

    def test_wrong_run_fields_fail(self):
        for field, wrong in (("id", 11), ("run_attempt", 1), ("head_sha", "b" * 40),
                             ("path", "wrong"), ("conclusion", "failure"), ("event", "push")):
            with self.subTest(field=field):
                old = self.run[field]
                self.run[field] = wrong
                with self.assertRaises(ValueError): self.invoke()
                self.run[field] = old

    def test_wrong_jobs_and_steps_fail(self):
        variants = []
        for field, wrong in (("run_attempt", 1), ("head_sha", "b" * 40), ("conclusion", "failure")):
            value = copy.deepcopy(self.jobs); value["jobs"][0][field] = wrong; variants.append(value)
        for change in ("missing", "duplicate", "skipped"):
            value = copy.deepcopy(self.jobs)
            if change == "missing": value["jobs"].pop()
            if change == "duplicate": value["jobs"][0]["name"] = value["jobs"][1]["name"]
            if change == "skipped": value["jobs"][0]["steps"][0]["conclusion"] = "skipped"
            variants.append(value)
        for value in variants:
            with self.subTest(value=value):
                self.jobs = value
                with self.assertRaises(ValueError): self.invoke()

    def test_digest_and_production_mismatches_fail(self):
        for key in ("archive_sha256", "manifest_sha256", "lock_sha256"):
            with self.subTest(key=key):
                old = self.receipt[key]; self.receipt[key] = "0" * 64
                with self.assertRaises(ValueError): self.invoke()
                self.receipt[key] = old
        self.files["src/lib.rs"] = b"changed production\n"
        self.make_snapshot()
        with self.assertRaises(ValueError): self.invoke()

    def test_removed_and_extra_source_fail(self):
        del self.files["src/lib.rs"]
        self.make_snapshot()
        with self.assertRaises(ValueError): self.invoke()
        self.files["src/lib.rs"] = b"production\n"
        self.files["src/extra.rs"] = b"new\n"
        self.make_snapshot()
        with self.assertRaises(ValueError): self.invoke()

    def test_unsafe_archive_members_fail(self):
        complete = [(n, d, tarfile.REGTYPE) for n, d in self.files.items()]
        for name in ("../evil", "/evil"):
            with self.subTest(name=name):
                self.make_snapshot(complete + [(name, b"bad", tarfile.REGTYPE)])
                with self.assertRaisesRegex(ValueError, "unsafe path"):
                    self.invoke()
                self.assertFalse(self.output.exists())
        for kind in (tarfile.SYMTYPE, tarfile.LNKTYPE, tarfile.DIRTYPE):
            name = "src/lib.rs"
            with self.subTest(name=name, kind=kind):
                self.make_snapshot([(n, d, kind if n == name else tarfile.REGTYPE)
                                    for n, d in self.files.items()])
                with self.assertRaisesRegex(ValueError, "archive names/types"):
                    self.invoke()
                self.assertFalse(self.output.exists())
        self.make_snapshot(complete + [("Cargo.lock", b"again", tarfile.REGTYPE)])
        with self.assertRaisesRegex(ValueError, "duplicate path"):
            self.invoke()
        self.assertFalse(self.output.exists())

    def test_duplicate_required_step_even_if_skipped_fails(self):
        self.jobs["jobs"][0]["steps"].append({"name": "Test debug", "status": "completed", "conclusion": "skipped"})
        with self.assertRaisesRegex(ValueError, "duplicate"):
            self.invoke()
        self.assertFalse(self.output.exists())

    def test_external_step_must_be_completed(self):
        for job in self.jobs["jobs"]:
            next(step for step in job["steps"] if step["name"] == "Acquire and verify pinned external oracle")["status"] = "in_progress"
        with self.assertRaisesRegex(ValueError, "external oracle step"):
            self.invoke()
        self.assertFalse(self.output.exists())

    def test_preexisting_output_fails(self):
        self.output.mkdir()
        with self.assertRaises(ValueError): self.invoke()

    def test_post_trial_recheck_rejects_drift_and_new_files(self):
        self.invoke()
        verify_extracted(self.receipt_path, self.snapshot, self.output)
        (self.output / "src/lib.rs").write_bytes(b"drift")
        with self.assertRaises(ValueError):
            verify_extracted(self.receipt_path, self.snapshot, self.output)
        (self.output / "src/lib.rs").write_bytes(self.files["src/lib.rs"])
        (self.output / "new-file").write_bytes(b"extra")
        with self.assertRaises(ValueError):
            verify_extracted(self.receipt_path, self.snapshot, self.output)

    def test_post_trial_failure_preserves_diagnostic_and_partial_source(self):
        self.invoke()
        evidence = self.root / "evidence"
        evidence.mkdir()
        (self.output / "Cargo.lock").write_bytes(b"changed lock")
        with self.assertRaisesRegex(ValueError, "source/lock"):
            record_extracted_check(self.receipt_path, self.snapshot, self.output, evidence)
        self.assertEqual((evidence / "source-after.status").read_text(), "failed\n")
        self.assertIn("source/lock", (evidence / "source-after.error.txt").read_text())
        self.assertFalse((evidence / "source-after.json").exists())
        self.assertEqual((self.output / "Cargo.lock").read_bytes(), b"changed lock")

    def test_post_trial_success_records_verified_digest(self):
        self.invoke()
        evidence = self.root / "evidence"
        evidence.mkdir()
        record_extracted_check(self.receipt_path, self.snapshot, self.output, evidence)
        self.assertEqual((evidence / "source-after.status").read_text(), "passed\n")
        self.assertEqual(json.loads((evidence / "source-after.json").read_text())["verified_files"], len(self.files))
        self.assertFalse((evidence / "source-after.error.txt").exists())


if __name__ == "__main__":
    unittest.main()
