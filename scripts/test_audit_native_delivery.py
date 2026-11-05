import hashlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import zipfile


HOST = "x86_64-unknown-linux-gnu"
PRODUCT = "demo-audit"
VERSION = "0.2.0"
TARGET = "linux-x86_64"


def ident(body):
    return {"bytes": len(body), "sha256": hashlib.sha256(body).hexdigest()}


def tar_bytes(prefix, files):
    stream = io.BytesIO()
    with tarfile.open(fileobj=stream, mode="w:gz") as archive:
        for name, body in files.items():
            member = tarfile.TarInfo(f"{prefix}/{name}")
            member.size = len(body)
            member.mode = 0o755 if name == PRODUCT else 0o644
            archive.addfile(member, io.BytesIO(body))
    return stream.getvalue()


def fixture(root):
    source = {
        "Cargo.toml": b'[package]\nname = "demo-audit"\nversion = "0.2.0"\nedition = "2024"\n',
        "Cargo.lock": (
            b'version = 4\n[[package]]\nname = "dep"\nversion = "1.0.0"\n'
            b'source = "registry+https://github.com/rust-lang/crates.io-index"\n'
            b'checksum = "' + b"7" * 64 + b'"\n'
        ),
        "src/main.rs": b"fn main() {}\n",
        "LICENSE-MIT": b"original MIT\n",
        "LICENSE-APACHE-2.0": b"original Apache\n",
        "THIRD_PARTY_NOTICES.md": b"original notices\n",
    }
    repo = root / "repo"
    repo.mkdir()
    for name, body in source.items():
        path = repo / name
        path.parent.mkdir(exist_ok=True)
        path.write_bytes(body)
    git = ["git", "-C", str(repo)]
    subprocess.run([*git, "init", "-q"], check=True)
    subprocess.run([*git, "add", "--all"], check=True)
    subprocess.run(
        [*git, "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
         "-c", "commit.gpgsign=false", "commit", "-qm", "original source"], check=True,
    )
    head = subprocess.check_output([*git, "rev-parse", "HEAD"], text=True).strip()
    prefix = f"{PRODUCT}-{VERSION}"
    source_body = tar_bytes(prefix, {
        **source, "Cargo.toml.orig": source["Cargo.toml"],
        ".cargo_vcs_info.json": json.dumps({"git": {"sha1": head}, "path_in_vcs": ""}).encode(),
    })
    binary = b"\x7fELF\x02\x01\x01" + b"\0" * 9 + b"\x03\0\x3e\0fixture"
    notice = b"dependency original\n"
    index = json.dumps({
        "native_host": HOST, "edges": ["normal", "build"],
        "packages": [{
            "package": "registry+https://github.com/rust-lang/crates.io-index#dep@1.0.0",
            "license_file": None, "license": "MIT", "registry_archive_sha256": "7" * 64,
            "originals": {"LICENSE": ident(notice)},
        }],
    }).encode()
    source_receipt = {
        "product": PRODUCT, "version": VERSION, "head_sha": head, "native_host": HOST,
        "rustc": f"rustc 1.91.0\nhost: {HOST}\n", "lock": ident(source["Cargo.lock"]),
        "helper": ident(Path(__file__).with_name("native_delivery.py").read_bytes()),
        "source_package": {
            "file": f"{prefix}.crate", "archive": ident(source_body),
            "original_members": len(source), "package_members": len(source) + 2,
            "normalized_metadata_match": True,
        },
        "dependency_notice_index": ident(index),
    }
    lifecycle = json.dumps({
        "source_head": head, "version": VERSION, "binary": ident(binary), "native_calls": 1,
    }).encode()
    build = {
        **source_receipt, "target": TARGET, "binary": ident(binary),
        "binary_format": "ELF 64-bit LSB pie executable, x86-64",
        "dynamic_libraries": "libc.so.6",
        "lifecycle": ident(lifecycle), "run_id": "1234", "run_attempt": "2",
    }
    carrier_files = {
        **{k: v for k, v in source.items() if k.startswith(("LICENSE-", "THIRD_PARTY_"))},
        PRODUCT: binary, "BUILD.json": json.dumps(build).encode(),
        "dependency-licenses/INDEX.json": index,
        "dependency-licenses/cargo-tree.txt": b"demo-audit v0.2.0\ndep v1.0.0\n",
        "dependency-licenses/dep-1.0.0/LICENSE": notice,
    }
    return repo, head, prefix, source_body, source_receipt, carrier_files, lifecycle, binary


def make_zip(archive, fixture_data, change=None):
    _, head, prefix, source, receipt, files, lifecycle, _ = fixture_data
    files = dict(files)
    if change:
        files[change] = files.get(change, b"") + b"changed"
    carrier = tar_bytes(f"{prefix}-{TARGET}", files)
    source_name, carrier_name = f"{prefix}.crate", f"{prefix}-{TARGET}.tar.gz"
    with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as bundle:
        for name, body in {
            source_name: source, carrier_name: carrier, "SOURCE.json": json.dumps(receipt).encode(),
            "SHA256SUMS": (f"{ident(carrier)['sha256']}  {carrier_name}\n"
                           f"{ident(source)['sha256']}  {source_name}\n").encode(),
            "dependency-licenses/INDEX.json": files["dependency-licenses/INDEX.json"],
            "lifecycle/VERIFIED.json": lifecycle,
            "lifecycle/commands.json": b'[{"argv":["/installed/demo-audit","--version"],"returncode":0}]',
            "lifecycle/1.stdout": b"demo-audit 0.2.0\n", "lifecycle/1.stderr": b"",
        }.items():
            bundle.writestr(name, body)
    return {"id": 42, "name": f"{PRODUCT}-delivery-{TARGET}-{head}-a2",
            "size_in_bytes": archive.stat().st_size, "expired": False,
            "digest": f"sha256:{ident(archive.read_bytes())['sha256']}",
            "workflow_run": {"id": 1234, "head_sha": head}}


def changed_normalized(data, suffix):
    repo, head, prefix, source, receipt, files, lifecycle, binary = data
    with tarfile.open(fileobj=io.BytesIO(source), mode="r:gz") as archive:
        members = {member.name.split("/", 1)[1]: archive.extractfile(member).read()
                   for member in archive}
    members["Cargo.toml"] += suffix
    source = tar_bytes(prefix, members)
    receipt = {**receipt, "source_package": {**receipt["source_package"], "archive": ident(source)}}
    build = {**json.loads(files["BUILD.json"]), **receipt}
    files = {**files, "BUILD.json": json.dumps(build).encode()}
    return repo, head, prefix, source, receipt, files, lifecycle, binary


class OriginalAuditTests(unittest.TestCase):
    def setUp(self):
        self.assertIsNotNone(importlib.util.find_spec("audit_native_delivery"), "original audit is missing")

    def test_four_artifacts_are_from_the_exact_run_and_attempt(self):
        from audit_native_delivery import artifact_targets

        head = "a" * 40
        rows = [
            {"id": n, "name": f"{PRODUCT}-delivery-{target}-{head}-a2",
             "size_in_bytes": 500, "digest": "sha256:" + "1" * 64,
             "expired": False, "workflow_run": {"id": 1234, "head_sha": head}}
            for n, target in enumerate(
                ("linux-x86_64", "linux-aarch64", "macos-x86_64", "macos-aarch64"), start=1)
        ]
        self.assertEqual(len(artifact_targets({"artifacts": rows}, PRODUCT, head, 1234, 2)), 4)
        with self.assertRaisesRegex(ValueError, "four native"):
            artifact_targets({"artifacts": rows}, PRODUCT, head, 1234, 3)
        rows[0]["workflow_run"]["id"] = 1235
        with self.assertRaisesRegex(ValueError, "run or head"):
            artifact_targets({"artifacts": rows}, PRODUCT, head, 1234, 2)

    def test_original_zip_binds_exact_source_installed_binary_and_notice(self):
        from audit_native_delivery import verify_archive

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            data = fixture(root)
            repo, head, _, source, _, _, _, binary = data
            original = root / "original.zip"
            meta = make_zip(original, data)
            passed = verify_archive(repo, head, HOST, 1234, 2, meta, original, root / "good")
            self.assertEqual(passed["source"], ident(source))
            self.assertEqual(passed["binary"], ident(binary))
            self.assertEqual(passed["notice_bodies"], 1)
            with self.assertRaisesRegex(ValueError, "original ZIP identity"):
                verify_archive(repo, head, HOST, 1234, 2, {**meta, "digest": "sha256:" + "0" * 64},
                               original, root / "bad-zip")

            for index, (member, reason) in enumerate([
                (PRODUCT, "installed binary"),
                ("dependency-licenses/dep-1.0.0/LICENSE", "dependency notice body"),
                ("unexpected.txt", "carrier membership"),
            ]):
                changed = root / f"modified-{index}.zip"
                changed_meta = make_zip(changed, data, member)
                with self.assertRaisesRegex(ValueError, reason):
                    verify_archive(repo, head, HOST, 1234, 2, changed_meta, changed,
                                   root / f"modified-{index}")

    def test_generated_manifest_is_bounded_and_independently_checked(self):
        import audit_native_delivery as audit

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            data = fixture(root)
            repo, head = data[:2]
            for index, (suffix, reason) in enumerate([
                (b"\n[features]\nnot-in-git = []\n", "normalized Cargo metadata differs"),
                (b"#" + b" " * (64 << 10) + b"\n", "source crate exceeds unpacked limit"),
            ]):
                with self.subTest(reason=reason), patch.object(audit, "MAX_UNPACKED", 32 << 10):
                    archive = root / f"manifest-{index}.zip"
                    meta = make_zip(archive, changed_normalized(data, suffix))
                    with self.assertRaisesRegex(ValueError, reason):
                        audit.verify_archive(repo, head, HOST, 1234, 2, meta, archive, root / f"check-{index}")
