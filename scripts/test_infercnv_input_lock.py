import copy
import unittest

from verify_infercnv_input_lock import verify_lock


class InputLockTests(unittest.TestCase):
    def setUp(self):
        self.before = {"version": 4, "package": [
            {"name": "rsomics-sc", "version": "0.1.0", "dependencies": ["nix", "rsomics-common"]},
            {"name": "rsomics-common", "version": "0.12.3"},
        ]}
        self.after = copy.deepcopy(self.before)
        self.after["package"][0]["dependencies"] = ["flate2", "nix", "rsomics-common"]
        for name in ("flate2", "crc32fast", "miniz_oxide", "adler2", "simd-adler32"):
            self.after["package"].append({
                "name": name, "version": "1.1.9" if name == "flate2" else "0.1.0",
                "source": "registry+https://github.com/rust-lang/crates.io-index",
                "checksum": "a" * 64,
            })

    def test_accepts_only_gzip_dependency_expansion(self):
        verify_lock(self.before, self.after)

    def test_rejects_existing_package_change(self):
        self.after["package"][1]["version"] = "0.12.4"
        with self.assertRaisesRegex(ValueError, "existing"):
            verify_lock(self.before, self.after)

    def test_rejects_wrong_root_expansion(self):
        self.after["package"][0]["dependencies"].append("extra")
        with self.assertRaisesRegex(ValueError, "root"):
            verify_lock(self.before, self.after)

    def test_rejects_wrong_added_identity(self):
        for key, value in (("version", "1.1.8"), ("source", "git+untrusted"),
                           ("checksum", "bad")):
            with self.subTest(key=key):
                variant = copy.deepcopy(self.after)
                variant["package"][2][key] = value
                with self.assertRaises(ValueError):
                    verify_lock(self.before, variant)

    def test_rejects_extra_missing_duplicate_nodes(self):
        for packages in (self.after["package"][:-1],
                         self.after["package"] + [self.after["package"][-1]],
                         self.after["package"] + [{"name": "other", "version": "1"}]):
            with self.subTest(packages=packages):
                with self.assertRaises(ValueError):
                    verify_lock(self.before, {"version": 4, "package": packages})
