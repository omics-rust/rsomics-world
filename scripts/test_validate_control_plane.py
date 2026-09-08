import contextlib
import io
import unittest
from pathlib import Path
from unittest.mock import patch

import validate_control_plane as validator


class ProductCoverageTests(unittest.TestCase):
    def setUp(self):
        self.products, _ = validator.allowlist()

    def validate_roadmap(self, names):
        roadmap = "Targets:\n\n" + "".join(f"- `{name}`\n" for name in names)
        read_text = Path.read_text

        def read(path, *args, **kwargs):
            if path == validator.ROOT / "ROADMAP.md":
                return roadmap
            return read_text(path, *args, **kwargs)

        with patch.object(Path, "read_text", read), contextlib.redirect_stdout(io.StringIO()):
            validator.main()

    def test_roadmap_accepts_each_allowed_product_once(self):
        self.validate_roadmap(sorted(self.products))

    def test_roadmap_rejects_a_retired_boundary(self):
        with self.assertRaisesRegex(SystemExit, "roadmap targets"):
            self.validate_roadmap(sorted(self.products) + ["rsomics-workflow"])

    def test_roadmap_rejects_a_missing_product(self):
        with self.assertRaisesRegex(SystemExit, "roadmap targets"):
            self.validate_roadmap(sorted(self.products - {"rsomics-cnv"}))

    def test_roadmap_rejects_duplicate_target_ownership(self):
        with self.assertRaisesRegex(SystemExit, "roadmap targets"):
            self.validate_roadmap(sorted(self.products) + ["rsomics-vcf"])


if __name__ == "__main__":
    unittest.main()
