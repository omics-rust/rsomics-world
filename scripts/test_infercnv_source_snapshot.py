import hashlib
import io
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
from infercnv_source_snapshot import _manifest, read_snapshot_manifest, verify_and_extract


def digest(data):
    return hashlib.sha256(data).hexdigest()


def manifest_bytes(files):
    return b"".join(f"{digest(data)}  {name}\n".encode() for name, data in sorted(files.items()))


class SourceSnapshotTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.snapshot = self.root / "snapshot"
        self.snapshot.mkdir()
        self.output = self.root / "output"
        self.files = {"Cargo.lock": b"lock\n", "Cargo.toml": b"package\n", "src/lib.rs": b"production\n"}
        self.make_snapshot()

    def make_checks(self, names=("README.md", "files.sha256", "source.tar.gz")):
        checks = b"".join(f"{digest((self.snapshot / name).read_bytes())}  {name}\n".encode() for name in names)
        (self.snapshot / "checksums.sha256").write_bytes(checks)

    def make_snapshot(self, members=None, archive_format=tarfile.USTAR_FORMAT):
        (self.snapshot / "files.sha256").write_bytes(manifest_bytes(self.files))
        (self.snapshot / "README.md").write_bytes(b"snapshot description\n")
        with tarfile.open(self.snapshot / "source.tar.gz", "w:gz", format=archive_format) as archive:
            for name, data, kind in members if members is not None else [(n, d, tarfile.REGTYPE) for n, d in self.files.items()]:
                info = tarfile.TarInfo(name)
                info.type = kind
                info.size = len(data) if kind in (tarfile.REGTYPE, tarfile.AREGTYPE, tarfile.CONTTYPE) else 0
                if kind in (tarfile.SYMTYPE, tarfile.LNKTYPE):
                    info.linkname = "Cargo.lock"
                if archive_format == tarfile.PAX_FORMAT:
                    info.pax_headers = {"mtime": "1.5"}
                archive.addfile(info, io.BytesIO(data) if info.size else None)
        self.make_checks()

    def rejected(self, pattern):
        with self.assertRaisesRegex(ValueError, pattern):
            verify_and_extract(self.snapshot, self.output)
        self.assertFalse(self.output.exists())

    def test_exact_ustar_snapshot(self):
        expected = {name: digest(data) for name, data in self.files.items()}
        self.assertEqual(read_snapshot_manifest(self.snapshot), expected)
        self.assertEqual(verify_and_extract(self.snapshot, self.output), expected)
        self.assertEqual({p.relative_to(self.output).as_posix(): p.read_bytes()
                          for p in self.output.rglob("*") if p.is_file()}, self.files)

    def test_pax_representation_preserves_logical_contract(self):
        self.make_snapshot(archive_format=tarfile.PAX_FORMAT)
        self.assertEqual(set(verify_and_extract(self.snapshot, self.output)), set(self.files))

    def test_checksum_must_include_readme(self):
        self.make_checks(("files.sha256", "source.tar.gz"))
        self.rejected("snapshot checksums mismatch")

    def test_checksum_cannot_include_extra(self):
        (self.snapshot / "extra").write_bytes(b"extra")
        self.make_checks(("README.md", "extra", "files.sha256", "source.tar.gz"))
        self.rejected("snapshot checksums mismatch")

    def test_checksum_readme_bytes_must_match(self):
        (self.snapshot / "README.md").write_bytes(b"changed description\n")
        self.rejected("snapshot checksums mismatch")

    def test_duplicate_checksum_path(self):
        path = self.snapshot / "checksums.sha256"
        path.write_bytes(path.read_bytes().splitlines(keepends=True)[0] + path.read_bytes())
        self.rejected("duplicate manifest path")

    def test_appledouble_and_other_extra_regular_members(self):
        complete = [(n, d, tarfile.REGTYPE) for n, d in self.files.items()]
        for name in ("._Cargo.toml", "src/._lib.rs", "__MACOSX/._Cargo.lock", "unlisted.rs"):
            with self.subTest(name=name):
                self.make_snapshot(complete + [(name, b"extra", tarfile.REGTYPE)])
                self.rejected("archive names/types")

    def test_missing_logical_member(self):
        self.make_snapshot([(n, d, tarfile.REGTYPE) for n, d in self.files.items() if n != "src/lib.rs"])
        self.rejected("archive names/types")

    def test_duplicate_logical_member(self):
        complete = [(n, d, tarfile.REGTYPE) for n, d in self.files.items()]
        self.make_snapshot(complete + [("Cargo.lock", b"again", tarfile.REGTYPE)])
        self.rejected("duplicate path")

    def test_unsafe_member_names(self):
        complete = [(n, d, tarfile.REGTYPE) for n, d in self.files.items()]
        for name in ("../bad", "/bad", "src/../bad", "src//bad", "./bad", "C:bad", "src\\bad", "bad\tname"):
            with self.subTest(name=name):
                self.make_snapshot(complete + [(name, b"bad", tarfile.REGTYPE)])
                self.rejected("unsafe path")

    def test_file_directory_collision(self):
        complete = [(n, d, tarfile.REGTYPE) for n, d in self.files.items()]
        self.make_snapshot(complete + [("src", b"bad", tarfile.REGTYPE)])
        self.rejected("file/directory collision")

    def test_nonregular_logical_member_types(self):
        for kind in (tarfile.SYMTYPE, tarfile.LNKTYPE, tarfile.DIRTYPE, tarfile.FIFOTYPE,
                     tarfile.CHRTYPE, tarfile.BLKTYPE, tarfile.AREGTYPE, tarfile.CONTTYPE):
            with self.subTest(kind=kind):
                self.make_snapshot([(n, d, kind if n == "src/lib.rs" else tarfile.REGTYPE)
                                    for n, d in self.files.items()])
                self.rejected("archive names/types")

    def test_member_hash_failure_precedes_output_creation(self):
        self.make_snapshot([(n, b"wrong bytes" if n == "src/lib.rs" else d, tarfile.REGTYPE)
                            for n, d in self.files.items()])
        self.rejected("archive hash mismatch: src/lib.rs")

    def test_manifest_changed_between_selection_and_extraction(self):
        expected = read_snapshot_manifest(self.snapshot)
        self.files["src/lib.rs"] = b"other valid snapshot\n"
        self.make_snapshot()
        with self.assertRaisesRegex(ValueError, "source manifest changed"):
            verify_and_extract(self.snapshot, self.output, expected)
        self.assertFalse(self.output.exists())

    def test_symlinked_snapshot_file(self):
        original = self.snapshot / "README.md"
        backup = self.root / "readme"
        original.rename(backup)
        original.symlink_to(backup)
        self.rejected("not a regular file")

    def test_symlinked_snapshot_directory(self):
        alias = self.root / "alias"
        alias.symlink_to(self.snapshot, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "symlinked path"):
            verify_and_extract(alias, self.output)
        self.assertFalse(self.output.exists())

    def test_existing_output_and_symlink_output(self):
        self.output.mkdir()
        with self.assertRaisesRegex(ValueError, "output directory already exists"):
            verify_and_extract(self.snapshot, self.output)
        broken = self.root / "broken"
        broken.symlink_to(self.root / "absent")
        with self.assertRaisesRegex(ValueError, "output directory already exists"):
            verify_and_extract(self.snapshot, broken)

    def test_output_parent_symlink(self):
        parent = self.root / "parent"
        parent.symlink_to(self.snapshot, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "symlinked path"):
            verify_and_extract(self.snapshot, parent / "output")

    def test_manifest_spelling_and_inventory(self):
        good = manifest_bytes(self.files)
        first = good.splitlines(keepends=True)[0]
        bad_cases = ((b"", "LF-terminated"), (good.rstrip(b"\n"), "LF-terminated"),
                     (good.replace(b"\n", b"\r\n"), "LF-terminated"),
                     (b"\xff\n", "UTF-8"), (good.replace(b"  ", b" *", 1), "malformed manifest row"),
                     (b"x" * 64 + b"  Cargo.lock\n", "malformed manifest row"),
                     (first + good, "duplicate manifest path"),
                     (b"".join(reversed(good.splitlines(keepends=True))), "empty or unsorted"))
        for data, pattern in bad_cases:
            with self.subTest(data=data):
                with self.assertRaisesRegex(ValueError, pattern):
                    _manifest(data)
        for names in (("src", "src/lib.rs"), ("../evil",), ("e\u0301",)):
            with self.subTest(names=names):
                data = b"".join(f"{'0' * 64}  {name}\n".encode() for name in sorted(names))
                with self.assertRaises(ValueError):
                    _manifest(data)


if __name__ == "__main__":
    unittest.main()
