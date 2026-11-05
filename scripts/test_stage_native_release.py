import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
import zipfile


PRODUCT = "rsomics-fastq-preprocess"
VERSION = "0.2.0"
HEAD = "7" * 40
TARGETS = ("linux-aarch64", "linux-x86_64", "macos-aarch64", "macos-x86_64")


def identity(body):
    return {"bytes": len(body), "sha256": hashlib.sha256(body).hexdigest()}


def audited_fixture(root):
    source_name = f"{PRODUCT}-{VERSION}.crate"
    files = {source_name: b"verified source archive"}
    targets = {}
    for number, target in enumerate(TARGETS, start=100):
        archive = root / f"{target}.zip"
        carrier_name = f"{PRODUCT}-{VERSION}-{target}.tar.gz"
        carrier_body = f"original {target} carrier".encode()
        files[carrier_name] = carrier_body
        with zipfile.ZipFile(archive, "x") as bundle:
            bundle.writestr(source_name, files[source_name])
            bundle.writestr(carrier_name, carrier_body)
        targets[target] = {
            "artifact_id": number,
            "artifact_name": f"{PRODUCT}-delivery-{target}-{HEAD}-a2",
            "zip": identity(archive.read_bytes()),
            "source": identity(files[source_name]),
            "carrier": identity(carrier_body),
        }
    audit = {
        "schema": "rsomics.native-original-audit.v1",
        "repository": f"omics-rust/{PRODUCT}",
        "product": PRODUCT,
        "version": VERSION,
        "head": HEAD,
        "run_id": 1234,
        "attempt": 2,
        "targets": targets,
        "release_authorized": False,
    }
    (root / "AUDITED.json").write_text(json.dumps(audit))
    return audit, files


class ReleaseStagingTests(unittest.TestCase):
    def setUp(self):
        self.assertIsNotNone(importlib.util.find_spec("stage_native_release"),
                             "native release staging is missing")

    def test_stages_only_verified_source_and_native_carriers(self):
        from stage_native_release import stage_release

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            audit, files = audited_fixture(root)
            destination = root / "release"
            stage_release(root, destination)
            self.assertEqual(set(path.name for path in destination.iterdir()),
                             set(files) | {"SHA256SUMS", "STAGED.json"})
            for name, body in files.items():
                self.assertEqual((destination / name).read_bytes(), body)
            checksums = "".join(f"{identity(body)['sha256']}  {name}\n"
                                for name, body in sorted(files.items()))
            self.assertEqual((destination / "SHA256SUMS").read_text(), checksums)
            staged = json.loads((destination / "STAGED.json").read_bytes())
            self.assertEqual(staged["head"], HEAD)
            self.assertEqual((staged["run_id"], staged["attempt"]), (1234, 2))
            self.assertEqual(staged["audit"], identity((root / "AUDITED.json").read_bytes()))
            self.assertEqual(staged["assets"], {**{name: identity(body) for name, body in files.items()},
                                               "SHA256SUMS": identity(checksums.encode())})
            self.assertEqual(staged["repository"], audit["repository"])
            self.assertIs(staged["release_authorized"], False)

    def test_changed_archive_or_carrier_does_not_produce_a_packet(self):
        from stage_native_release import stage_release

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            audit, files = audited_fixture(root)
            target = "macos-aarch64"
            archive = root / f"{target}.zip"
            source = f"{PRODUCT}-{VERSION}.crate"
            with zipfile.ZipFile(archive, "w") as bundle:
                bundle.writestr(source, files[source])
                bundle.writestr(f"{PRODUCT}-{VERSION}-{target}.tar.gz", b"different carrier")
            destination = root / "release"
            with self.assertRaisesRegex(ValueError, "ZIP identity"):
                stage_release(root, destination)
            self.assertFalse(destination.exists())

            audit["targets"][target]["zip"] = identity(archive.read_bytes())
            (root / "AUDITED.json").write_text(json.dumps(audit))
            with self.assertRaisesRegex(ValueError, "carrier identity"):
                stage_release(root, destination)
            self.assertFalse(destination.exists())
