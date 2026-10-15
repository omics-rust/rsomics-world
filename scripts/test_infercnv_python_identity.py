import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from infercnv_python_identity import _hash_file, _parse_maps, capture_identity


SCRIPT = Path(__file__).with_name("infercnv_python_identity.py").resolve()
MAPS = Path("/proc/self/maps")


class MapsParserTests(unittest.TestCase):
    def test_sorts_and_deduplicates_named_runtime_segments(self):
        maps = "\n".join([
            "1000-2000 r--p 00000000 08:01 123 /opt/libpython3.12.so",
            "2000-3000 r-xp 00001000 08:01 123 /opt/libpython3.12.so",
            "3000-4000 rw-p 00000000 08:01 124 /a/libpython3.11.so",
            "4000-5000 r--p 00000000 08:01 125 /opt/libpython-custom",
        ])
        self.assertEqual(_parse_maps(maps), [
            "/a/libpython3.11.so", "/opt/libpython-custom", "/opt/libpython3.12.so",
        ])

    def test_preserves_spaces_in_pathnames(self):
        self.assertEqual(_parse_maps(
            "1000-2000 r-xp 00000000 08:01 123 /opt/runtime space/libpython 3.12.so\n"
        ), ["/opt/runtime space/libpython 3.12.so"])

    def test_accepts_anonymous_bracket_and_nonruntime_mappings(self):
        maps = "\n".join([
            "1000-2000 rw-p 00000000 00:00 0",
            "2000-3000 rw-p 00000000 00:00 0 [heap]",
            "3000-4000 r-xp 00000000 00:00 0 [vdso]",
            "4000-5000 rw-s 00000000 08:01 123 /opt/libpython-dir/other.so",
            "5000-6000 r--p 00000000 08:01 124 /other/file with spaces",
        ])
        self.assertEqual(_parse_maps(maps), [])

    def test_rejects_empty_maps(self):
        for maps in ("", "\n", "  \t\n"):
            with self.subTest(maps=maps), self.assertRaises(ValueError):
                _parse_maps(maps)

    def test_rejects_malformed_rows_including_after_valid_rows(self):
        rows = [
            "not a map",
            "1000-2000 r-xp 00000000 08:01",
            "bogus r-xp 00000000 08:01 123",
            "2000-1000 r-xp 00000000 08:01 123",
            "1000-1000 r-xp 00000000 08:01 123",
            "1000-2000 rwxz 00000000 08:01 123",
            "1000-2000 r-xp nothex 08:01 123",
            "1000-2000 r-xp 00000000 bad 123",
            "1000-2000 r-xp 00000000 08:01 -1",
            "1000-2000 r-xp 00000000 08:01 notanumber",
            "1000-2000 r-xp 00000000 08:01 123 /lib/libpython\x00.so",
            "",
        ]
        for row in rows:
            with self.subTest(row=row), self.assertRaises(ValueError):
                _parse_maps("1000-2000 rw-p 00000000 00:00 0\n" + row + "\n")

    def test_rejects_deleted_named_runtime_files(self):
        with self.assertRaises(ValueError):
            _parse_maps("1000-2000 r-xp 00000000 08:01 123 /lib/libpython3.so (deleted)")

    def test_rejects_nonabsolute_named_runtime_files(self):
        for path in ("libpython3.so", "relative/libpython3.so"):
            with self.subTest(path=path), self.assertRaises(ValueError):
                _parse_maps("1000-2000 r-xp 00000000 08:01 123 " + path)


class FileHashTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="infercnv-python-identity-")
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)

    def test_hashes_actual_binary_file_contents(self):
        payload = bytes(range(256)) * 8193
        path = self.root / "libpython with spaces.so"
        path.write_bytes(payload)
        self.assertEqual(_hash_file(path), hashlib.sha256(payload).hexdigest())
        path.write_bytes(b"changed")
        self.assertEqual(_hash_file(path),
                         "d67e2e944994496c8d8ec76eed0cf9f09679448d584b532bebf941852a37f5ed")

    def test_hashes_empty_regular_file(self):
        path = self.root / "empty"
        path.touch()
        self.assertEqual(_hash_file(path),
                         "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")

    def test_missing_file_fails(self):
        with self.assertRaises(OSError):
            _hash_file(self.root / "missing")

    def test_directory_fails(self):
        with self.assertRaises(OSError):
            _hash_file(self.root)

    @unittest.skipUnless(hasattr(os, "mkfifo"), "requires named pipes")
    def test_nonregular_file_fails_without_opening_pipe(self):
        path = self.root / "pipe"
        os.mkfifo(path)
        with self.assertRaises(OSError):
            _hash_file(path)

    @unittest.skipIf(hasattr(os, "geteuid") and os.geteuid() == 0,
                     "root bypasses file read permissions")
    def test_unreadable_file_fails(self):
        path = self.root / "unreadable"
        path.write_bytes(b"private")
        path.chmod(0)
        self.addCleanup(path.chmod, 0o600)
        with self.assertRaises(PermissionError):
            _hash_file(path)


class ActualProcessTests(unittest.TestCase):
    def run_cli(self, *arguments):
        return subprocess.run([sys.executable, "-I", "-B", str(SCRIPT), *arguments],
                              capture_output=True, text=True, check=False)

    def test_rejects_all_cli_arguments_without_success_output(self):
        for arguments in (("--help",), ("--maps", "/tmp/maps"),
                          ("--executable", sys.executable), ("unexpected",)):
            with self.subTest(arguments=arguments):
                result = self.run_cli(*arguments)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(result.stdout, "")
                self.assertTrue(result.stderr.strip())
                self.assertNotIn("Traceback", result.stderr)

    @unittest.skipIf(MAPS.exists(), "unsupported-procfs failure requires absent procfs")
    def test_absent_actual_procfs_fails_cleanly(self):
        with self.assertRaises(OSError):
            capture_identity()
        result = self.run_cli()
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")
        self.assertIn("/proc/self/maps", result.stderr)
        self.assertEqual(len(result.stderr.splitlines()), 1)
        self.assertNotIn("Traceback", result.stderr)

    @unittest.skipUnless(sys.platform == "linux" or MAPS.exists(),
                         "actual Linux procfs is unavailable locally")
    def test_actual_isolated_cli_matches_running_interpreter(self):
        result = self.run_cli()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr, "")
        self.assertEqual(len(result.stdout.splitlines()), 1)
        identity = json.loads(result.stdout)
        self.assertEqual(result.stdout, json.dumps(identity, sort_keys=True) + "\n")
        self.assertEqual(set(identity), {
            "executable", "executable_sha256", "version", "prefix", "base_prefix",
            "stdlib_json", "libpython_sha256",
        })
        executable = Path(sys.executable).resolve(strict=True)
        self.assertEqual(identity["executable"], str(executable))
        self.assertEqual(identity["executable_sha256"],
                         hashlib.sha256(executable.read_bytes()).hexdigest())
        self.assertEqual(identity["version"], sys.version)
        self.assertEqual(identity["prefix"], sys.prefix)
        self.assertEqual(identity["base_prefix"], sys.base_prefix)
        self.assertEqual(identity["stdlib_json"], json.__file__)
        mapped_libraries = set()
        for line in MAPS.read_text().splitlines():
            columns = line.split(None, 5)
            if len(columns) == 6 and Path(columns[5]).name.startswith("libpython"):
                mapped_libraries.add(columns[5])
        self.assertEqual(set(identity["libpython_sha256"]), mapped_libraries)
        for path, digest in identity["libpython_sha256"].items():
            self.assertEqual(digest, hashlib.sha256(Path(path).read_bytes()).hexdigest())
        self.assertEqual(capture_identity(), identity)


if __name__ == "__main__":
    unittest.main()
