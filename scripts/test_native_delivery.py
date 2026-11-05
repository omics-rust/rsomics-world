import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest


class SourcePackageTests(unittest.TestCase):
    def test_exact_package_rejects_changed_git_body_before_extraction(self):
        self.assertIsNotNone(importlib.util.find_spec("native_delivery"), "source verifier is missing")
        from native_delivery import extract_exact_package

        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo = root / "repo"
            repo.mkdir()
            manifest = b'[package]\nname = "example"\nversion = "0.1.0"\n'
            source = b'fn main() { println!("exact"); }\n'
            (repo / "Cargo.toml").write_bytes(manifest)
            (repo / "src").mkdir()
            (repo / "src/main.rs").write_bytes(source)
            subprocess.run(["git", "init", "-q", str(repo)], check=True)
            subprocess.run(["git", "-C", str(repo), "add", "Cargo.toml", "src"], check=True)
            subprocess.run(
                ["git", "-C", str(repo), "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
                 "-c", "commit.gpgsign=false", "commit", "-qm", "source fixture"],
                check=True,
            )
            head = subprocess.check_output(["git", "-C", str(repo), "rev-parse", "HEAD"], text=True).strip()

            def package(archive, body):
                members = {
                    "Cargo.toml": b"# Normalized by Cargo\n" + manifest,
                    "Cargo.toml.orig": manifest,
                    ".cargo_vcs_info.json": json.dumps({"git": {"sha1": head}, "path_in_vcs": ""}).encode(),
                    "src/main.rs": body,
                }
                with tarfile.open(archive, "w:gz") as tar:
                    for name, content in members.items():
                        info = tarfile.TarInfo("example-0.1.0/" + name)
                        info.size = len(content)
                        tar.addfile(info, io.BytesIO(content))

            archive = root / "example-0.1.0.crate"
            exact = root / "exact/example-0.1.0"
            package(archive, source)
            extract_exact_package(repo, head, archive, exact)
            self.assertEqual((exact / "src/main.rs").read_bytes(), source)
            changed = root / "changed/example-0.1.0"
            package(archive, source.replace(b"exact", b"changed"))
            with self.assertRaisesRegex(ValueError, "source package body differs: src/main.rs"):
                extract_exact_package(repo, head, archive, changed)
            self.assertFalse(changed.exists())
