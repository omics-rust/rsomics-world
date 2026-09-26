"""Tiny synthetic unit artifacts; these are not upstream oracle results."""

import hashlib
import io
import json
import stat
import tempfile
import unittest
import warnings
import zipfile
from pathlib import Path

from verify_infercnv_oracle_artifact import verify_artifact


TMP_ROOT = Path("/Volumes/KIOXIA/Developments/tmp")
SHA = "a" * 40
RUN_ID = 12345
ARTIFACT_ID = 67890


def sha(data):
    return hashlib.sha256(data).hexdigest()


class ArtifactVerifierTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir=TMP_ROOT)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.output = self.root / "extracted"
        self.files = {"oracle.json": b'{"unit_fixture": true}\n', "table.tsv": b"a\tb\n"}
        self.manifest = self.make_manifest(self.files)
        self.entries = {"shipped-bundle/" + name: data for name, data in self.files.items()}
        self.entries["shipped-bundle/sha256-manifest.tsv"] = self.manifest
        self.receipt = {
            "schema_version": 1,
            "repository": "omics-rust/rsomics-world",
            "workflow_path": ".github/workflows/infercnv-oracle.yml",
            "run_id": RUN_ID,
            "run_attempt": 2,
            "artifact_id": ARTIFACT_ID,
            "head_sha": SHA,
            "artifact_name": f"infercnv-oracle-{RUN_ID}-2",
            "artifact_sha256": "",
            "manifest_sha256": sha(self.manifest),
            "bundle_path": "shipped-bundle",
        }
        self.run = {
            "id": RUN_ID,
            "run_attempt": 2,
            "head_sha": SHA,
            "path": ".github/workflows/infercnv-oracle.yml",
            "repository": {"full_name": "omics-rust/rsomics-world"},
            "head_repository": {"full_name": "omics-rust/rsomics-world"},
            "status": "completed",
            "conclusion": "success",
            "event": "workflow_dispatch",
        }
        self.artifact = {
            "id": ARTIFACT_ID,
            "name": f"infercnv-oracle-{RUN_ID}-2",
            "expired": False,
            "digest": "",
            "size_in_bytes": 0,
            "workflow_run": {"id": RUN_ID, "head_sha": SHA},
        }
        self.write_zip()

    @staticmethod
    def make_manifest(files):
        return ("sha256\tpath\n" + "".join(
            f"{sha(data)}\t{name}\n" for name, data in sorted(files.items())
        )).encode("utf-8")

    def write_zip(self):
        self.write_zip_entries(self.entries.items())

    def write_zip_entries(self, entries, compression=zipfile.ZIP_DEFLATED):
        data = io.BytesIO()
        with zipfile.ZipFile(data, "w", compression=compression) as archive:
            for name, content in entries:
                with warnings.catch_warnings():
                    warnings.filterwarnings("ignore", message="Duplicate name:", category=UserWarning)
                    archive.writestr(name, content)
        self.zip_path = self.root / "unit.zip"
        self.zip_path.write_bytes(data.getvalue())
        self.receipt["artifact_sha256"] = sha(data.getvalue())
        self.artifact["digest"] = "sha256:" + self.receipt["artifact_sha256"]
        self.artifact["size_in_bytes"] = len(data.getvalue())

    def verify(self):
        receipt = self.root / "receipt.json"
        run = self.root / "run.json"
        artifact = self.root / "artifact.json"
        receipt.write_text(json.dumps(self.receipt), encoding="utf-8")
        run.write_text(json.dumps(self.run), encoding="utf-8")
        artifact.write_text(json.dumps(self.artifact), encoding="utf-8")
        return verify_artifact(receipt, run, artifact, self.zip_path, self.output)

    def test_valid_synthetic_artifact_extracts_verified_bundle(self):
        bundle = self.verify()
        self.assertEqual(bundle, self.output / "shipped-bundle")
        self.assertEqual((bundle / "oracle.json").read_bytes(), self.files["oracle.json"])
        self.assertEqual((bundle / "sha256-manifest.tsv").read_bytes(), self.manifest)

    def test_valid_whole_evidence_archive_keeps_safe_siblings_out_of_manifest_scope(self):
        self.entries.update({
            "bundle/synthetic.txt": b"unit synthetic bundle\n",
            "source/source.tar.gz": b"unit source archive bytes\n",
            "logs/run.log": b"unit run log\n",
            "top-level.log": b"unit top-level log\n",
        })
        self.write_zip()
        try:
            bundle = self.verify()
        except ValueError as exc:
            self.fail(f"safe whole-evidence archive rejected: {exc}")
        self.assertEqual(bundle, self.output / "shipped-bundle")
        self.assertEqual((self.output / "bundle/synthetic.txt").read_bytes(), b"unit synthetic bundle\n")
        self.assertEqual((self.output / "source/source.tar.gz").read_bytes(), b"unit source archive bytes\n")
        self.assertEqual((self.output / "logs/run.log").read_bytes(), b"unit run log\n")
        self.assertEqual((self.output / "top-level.log").read_bytes(), b"unit top-level log\n")
        self.assertEqual((bundle / "sha256-manifest.tsv").read_bytes(), self.manifest)

    def test_receipt_schema_rejects_missing_extra_wrong_type_and_bad_hash(self):
        for change in (
            lambda r: r.pop("run_id"),
            lambda r: r.update(extra="surprise"),
            lambda r: r.update(run_id=True),
            lambda r: r.update(run_attempt=0),
            lambda r: r.update(head_sha="A" * 40),
            lambda r: r.update(bundle_path="other"),
            lambda r: r.update(artifact_name="alias"),
            lambda r: r.update(manifest_sha256="A" * 64),
        ):
            with self.subTest(change=change):
                original = self.receipt.copy()
                change(self.receipt)
                with self.assertRaises(ValueError):
                    self.verify()
                self.receipt = original
                self.assertFalse(self.output.exists())

    def test_rejects_wrong_run_identity_status_attempt_and_repository(self):
        for key, value in (
            ("id", RUN_ID + 1),
            ("run_attempt", 1),
            ("head_sha", "b" * 40),
            ("path", "other.yml"),
            ("status", "in_progress"),
            ("conclusion", "failure"),
            ("event", "push"),
            ("repository", {"full_name": "other/repo"}),
            ("head_repository", {"full_name": "other/repo"}),
        ):
            with self.subTest(key=key):
                original = self.run[key]
                self.run[key] = value
                with self.assertRaises(ValueError):
                    self.verify()
                self.run[key] = original
                self.assertFalse(self.output.exists())

    def test_rejects_wrong_artifact_identity_link_expiry_digest_and_size(self):
        for key, value in (
            ("id", ARTIFACT_ID + 1),
            ("name", "alias"),
            ("expired", True),
            ("expired", 0),
            ("digest", "sha256:" + "b" * 64),
            ("size_in_bytes", self.artifact["size_in_bytes"] + 1),
            ("workflow_run", {"id": RUN_ID + 1, "head_sha": SHA}),
            ("workflow_run", {"id": RUN_ID, "head_sha": "b" * 40}),
        ):
            with self.subTest(key=key, value=value):
                original = self.artifact[key]
                self.artifact[key] = value
                with self.assertRaises(ValueError):
                    self.verify()
                self.artifact[key] = original
                self.assertFalse(self.output.exists())

    def test_wrong_zip_digest_rejected_before_extraction(self):
        self.receipt["artifact_sha256"] = "b" * 64
        self.artifact["digest"] = "sha256:" + "b" * 64
        with self.assertRaisesRegex(ValueError, "ZIP digest"):
            self.verify()
        self.assertFalse(self.output.exists())

    def test_wrong_manifest_digest_preserves_extracted_output(self):
        self.receipt["manifest_sha256"] = "b" * 64
        with self.assertRaisesRegex(ValueError, "manifest digest"):
            self.verify()
        self.assertTrue((self.output / "shipped-bundle" / "sha256-manifest.tsv").exists())

    def test_rejects_malformed_manifest_rows(self):
        for manifest in (
            b"wrong\tpath\n",
            b"sha256\tpath\n" + b"z" * 64 + b"\toracle.json\n",
            b"sha256\tpath\n" + f"{sha(b'x')}\t../oracle.json\n".encode(),
            b"sha256\tpath\n" + f"{sha(b'x')}\toracle.json\n".encode() * 2,
            b"sha256\tpath\n" + f"{sha(b'x')}\tsha256-manifest.tsv\n".encode(),
        ):
            with self.subTest(manifest=manifest):
                self.entries["shipped-bundle/sha256-manifest.tsv"] = manifest
                self.receipt["manifest_sha256"] = sha(manifest)
                self.write_zip()
                with self.assertRaises(ValueError):
                    self.verify()
                self.assertTrue(self.output.exists())
                self.output.rename(self.root / f"failed-{len(list(self.root.iterdir()))}")
                self.output = self.root / f"next-{len(list(self.root.iterdir()))}"

    def test_rejects_unsafe_archive_names_and_collisions(self):
        for bad in (
            "../outside", "/absolute", "shipped-bundle/../escape",
            "shipped-bundle/./oracle.json", "shipped-bundle//oracle.json",
            "shipped-bundle\\oracle.json", "C:/absolute", "other/../file",
            "other\\file",
        ):
            with self.subTest(bad=bad):
                self.write_zip_entries(list(self.entries.items()) + [(bad, b"bad")])
                with self.assertRaises(ValueError):
                    self.verify()
                self.assertFalse(self.output.exists())
        self.write_zip_entries(list(self.entries.items()) + [("shipped-bundle/oracle.json", b"duplicate")])
        with self.assertRaises(ValueError):
            self.verify()
        self.write_zip_entries(list(self.entries.items()) + [("shipped-bundle/oracle.json/child", b"collision")])
        with self.assertRaises(ValueError):
            self.verify()

    def test_rejects_symlink_and_special_member(self):
        for mode in (stat.S_IFLNK | 0o777, stat.S_IFIFO | 0o644):
            with self.subTest(mode=mode):
                member = zipfile.ZipInfo("shipped-bundle/link")
                member.create_system = 3
                member.external_attr = mode << 16
                self.write_zip_entries(list(self.entries.items()) + [(member, b"oracle.json")])
                with self.assertRaises(ValueError):
                    self.verify()
                self.assertFalse(self.output.exists())

    def test_rejects_corrupt_member_even_with_updated_zip_pin(self):
        self.write_zip_entries(self.entries.items(), compression=zipfile.ZIP_STORED)
        data = self.zip_path.read_bytes()
        needle = self.files["oracle.json"]
        self.assertIn(needle, data)
        self.zip_path.write_bytes(data.replace(needle, b"X" + needle[1:], 1))
        self.receipt["artifact_sha256"] = sha(self.zip_path.read_bytes())
        self.artifact["digest"] = "sha256:" + self.receipt["artifact_sha256"]
        with self.assertRaises(ValueError):
            self.verify()
        self.assertFalse(self.output.exists())

    def test_rejects_corrupt_zip_container(self):
        self.zip_path.write_bytes(b"not a ZIP")
        self.receipt["artifact_sha256"] = sha(b"not a ZIP")
        self.artifact["digest"] = "sha256:" + self.receipt["artifact_sha256"]
        self.artifact["size_in_bytes"] = len(b"not a ZIP")
        with self.assertRaises(ValueError):
            self.verify()
        self.assertFalse(self.output.exists())

    def test_rejects_file_content_not_matching_manifest(self):
        self.entries["shipped-bundle/table.tsv"] = b"changed\n"
        self.write_zip()
        with self.assertRaisesRegex(ValueError, "file digest"):
            self.verify()
        self.assertTrue(self.output.exists())

    def test_rejects_control_character_archive_name(self):
        self.entries["shipped-bundle/bad\rname"] = b"x"
        manifest = self.make_manifest({**self.files, "bad\rname": b"x"})
        self.entries["shipped-bundle/sha256-manifest.tsv"] = manifest
        self.receipt["manifest_sha256"] = sha(manifest)
        self.write_zip()
        with self.assertRaises(ValueError):
            self.verify()
        self.assertFalse(self.output.exists())

    def test_rejects_missing_extra_and_unlisted_bundle_files(self):
        for additions, removals in (
            ({}, ("shipped-bundle/oracle.json",)),
            ({"shipped-bundle/extra.txt": b"extra"}, ()),
        ):
            with self.subTest(additions=additions, removals=removals):
                entries = {**self.entries, **additions}
                for path in removals:
                    entries.pop(path)
                self.write_zip_entries(entries.items())
                with self.assertRaises(ValueError):
                    self.verify()
                if self.output.exists():
                    self.output.rename(self.root / f"failed-{len(list(self.root.iterdir()))}")
                self.output = self.root / f"next-{len(list(self.root.iterdir()))}"

    def test_preexisting_destination_is_preserved(self):
        self.output.mkdir()
        sentinel = self.output / "sentinel"
        sentinel.write_bytes(b"keep")
        with self.assertRaises(ValueError):
            self.verify()
        self.assertEqual(sentinel.read_bytes(), b"keep")

    def test_symlinked_output_parent_is_rejected(self):
        real_parent = self.root / "real-parent"
        real_parent.mkdir()
        link = self.root / "link"
        link.symlink_to(real_parent, target_is_directory=True)
        self.output = link / "extracted"
        with self.assertRaisesRegex(ValueError, "symlink"):
            self.verify()
        self.assertFalse((real_parent / "extracted").exists())


if __name__ == "__main__":
    unittest.main()
