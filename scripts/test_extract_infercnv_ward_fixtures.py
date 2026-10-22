import copy
import hashlib
import io
import json
from pathlib import Path
import tempfile
import subprocess
import sys
import unittest
from unittest.mock import patch

import extract_infercnv_ward_fixtures as ward


def group():
    return {
        "input_indices": [5, 3, 4],
        "tree_entry_present": True,
        "tree_present": True,
        "tree": {
            "merge": [[-1, -3], [-2, 1]],
            "height": ["0.5", "0.8660254037844386"],
            "order": [2, 1, 3],
            "labels": ["c5", "c3", "c4"],
            "method": "ward.D2",
        },
        "distances": ["1", "0.5", "0.5"],
    }


def document():
    route = {"status": "success", "groups": [group()]}
    return {
        "cases": {
            "z_workflow": {
                "kind": "workflow",
                "isolated": copy.deepcopy(route),
                "full_run": copy.deepcopy(route),
            },
            "a_probe": {"kind": "probe", "isolated": copy.deepcopy(route)},
        }
    }


class WardFixtures(unittest.TestCase):
    def profile_fixture(self, root, features=2):
        doc = document()
        files = {
            "cells.tsv": "cell\tgroup\trole\n" + "".join(
                f"c{i}\tg\tobservation\n" for i in range(1, 8)),
            "genes.tsv": "gene\tchr\tstart\tstop\ng1\t1\t1\t2\ng2\t1\t3\t4\n",
            "matrix.tsv": "gene\t" + "\t".join(f"c{i}" for i in range(1, 8)) + "\n"
                + "".join(f"g{i}\t" + "\t".join(["0"] * 7) + "\n"
                          for i in range(1, features + 1)),
        }
        doc["files"] = {}
        for name, data in files.items():
            (root / name).write_text(data)
            doc["files"][name] = hashlib.sha256(data.encode()).hexdigest()
        for case in doc["cases"].values():
            for route in ("isolated", "full_run"):
                if route in case:
                    case[route]["state"] = {"cells": "cells.tsv", "genes": "genes.tsv"}
                    case[route]["diagnostic"] = {
                        "filtered_expression": "matrix.tsv",
                        "selected_gene_indices": list(range(1, features + 1)),
                    }
                    case[route]["groups"][0]["declared_indices"] = [1, 2, 3]
        return doc

    def run_profiles(self, root, doc, counts=True):
        raw = json.dumps(doc).encode()
        source = root / "witness.json"
        source.write_bytes(raw)
        expected = {"cases": 2, "routes": 3, "groups": 3,
                    "trees": 3, "pairs": 9, "max_leaves": 3}
        with patch.object(ward, "WITNESS_SHA256", hashlib.sha256(raw).hexdigest()):
            with patch.object(ward, "EXPECTED_COUNTS", expected if counts else ward.EXPECTED_COUNTS):
                return ward.extract_profiles(source, root / "profiles.tsv")

    def test_profiles_actual_order_global_dimensions_and_zero_features(self):
        for features in (0, 2):
            with tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                doc = self.profile_fixture(root, features)
                result = self.run_profiles(root, doc)
                data = (root / "profiles.tsv").read_bytes()
                self.assertEqual(data.decode().splitlines(), [
                    "tree\tmatrix\tfeatures\tcells\tindices",
                    *[f"{name}\tmatrix.tsv\t{features}\t7\t4,2,3" for name in (
                        "a_probe/isolated/0", "z_workflow/isolated/0", "z_workflow/full_run/0")]])
                self.assertEqual(result["fixture_sha256"], hashlib.sha256(data).hexdigest())
                with self.assertRaises(FileExistsError):
                    self.run_profiles(root, doc)

    def test_profiles_reject_before_output(self):
        for mutation in ("hash", "path", "symlink", "header", "selection", "boolean",
                         "duplicate", "bounds", "counts", "cells", "columns", "witness"):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                doc = self.profile_fixture(root)
                route = doc["cases"]["a_probe"]["isolated"]
                if mutation == "hash":
                    doc["files"]["matrix.tsv"] = "0" * 64
                elif mutation == "path":
                    route["diagnostic"]["filtered_expression"] = "../matrix.tsv"
                elif mutation == "symlink":
                    (root / "link.tsv").symlink_to(root / "matrix.tsv")
                    route["diagnostic"]["filtered_expression"] = "link.tsv"
                elif mutation in ("header", "cells"):
                    name = "matrix.tsv" if mutation == "header" else "cells.tsv"
                    path = root / name
                    path.write_text(path.read_text().replace("c1", "wrong", 1))
                    doc["files"][name] = hashlib.sha256(path.read_bytes()).hexdigest()
                elif mutation == "columns":
                    path = root / "cells.tsv"
                    path.write_text("".join(line + "\textra\n" for line in path.read_text().splitlines()))
                    doc["files"]["cells.tsv"] = hashlib.sha256(path.read_bytes()).hexdigest()
                elif mutation == "selection":
                    route["diagnostic"]["selected_gene_indices"] = [1, 3]
                elif mutation in ("boolean", "duplicate"):
                    route["diagnostic"]["selected_gene_indices"] = (
                        [True] if mutation == "boolean" else [1, 1])
                elif mutation == "bounds":
                    route["groups"][0]["input_indices"] = [8, 3, 4]
                with self.assertRaises(ValueError):
                    if mutation == "witness":
                        (root / "witness.json").write_text(json.dumps(doc))
                        ward.extract_profiles(root / "witness.json", root / "profiles.tsv")
                    else:
                        self.run_profiles(root, doc, counts=mutation != "counts")
                self.assertFalse((root / "profiles.tsv").exists())

    def test_cli_requires_exactly_one_output(self):
        for outputs in ([], ["--output", "a", "--profiles-output", "b"]):
            result = subprocess.run([sys.executable, "-B", str(Path(ward.__file__)),
                                     "--witness", "missing", *outputs], capture_output=True)
            self.assertEqual(result.returncode, 2)

    def test_cli_preserves_old_output_and_selects_profiles(self):
        for option in ("--output", "--profiles-output"):
            with tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                doc = self.profile_fixture(root)
                raw = json.dumps(doc).encode()
                source, target = root / "witness.json", root / "output.tsv"
                source.write_bytes(raw)
                counts = {"cases": 2, "routes": 3, "groups": 3,
                          "trees": 3, "pairs": 9, "max_leaves": 3}
                with patch.object(ward, "WITNESS_SHA256", hashlib.sha256(raw).hexdigest()), \
                        patch.object(ward, "EXPECTED_COUNTS", counts), \
                        patch.object(sys, "argv", ["extract", "--witness", str(source),
                                                   option, str(target)]), \
                        patch("sys.stdout", new_callable=io.StringIO) as output:
                    ward.main()
                    self.assertEqual(json.loads(output.getvalue())["counts"], counts)
                prefix = "tree\ta_probe/isolated/0\t3\n" if option == "--output" else (
                    "tree\tmatrix\tfeatures\tcells\tindices\n")
                self.assertTrue(target.read_text().startswith(prefix))

    def test_preserves_scalar_strings_and_source_order(self):
        records = ward._records(document())
        self.assertEqual([r[0] for r in records], [
            "a_probe/isolated/0", "z_workflow/isolated/0", "z_workflow/full_run/0",
        ])
        self.assertEqual(records[0][1:], (
            3, ["1", "0.5", "0.5"], [[-1, -3], [-2, 1]],
            ["0.5", "0.8660254037844386"], [2, 1, 3],
        ))

    def test_requires_actual_workflow_routes_and_success(self):
        for change in ("isolated", "full_run", "failure", "probe_extra", "kind"):
            with self.subTest(change=change):
                doc = document()
                if change in ("isolated", "full_run"):
                    del doc["cases"]["z_workflow"][change]
                elif change == "failure":
                    doc["cases"]["a_probe"]["isolated"]["status"] = "failure"
                elif change == "probe_extra":
                    doc["cases"]["a_probe"]["full_run"] = copy.deepcopy(
                        doc["cases"]["a_probe"]["isolated"])
                else:
                    doc["cases"]["a_probe"]["kind"] = "unknown"
                with self.assertRaises(ValueError):
                    ward._records(doc)

    def test_no_tree_is_only_valid_for_small_groups(self):
        doc = document()
        g = doc["cases"]["a_probe"]["isolated"]["groups"][0]
        g.update(input_indices=[2, 1], tree_entry_present=False,
                 tree_present=False, tree=None, distances=[])
        self.assertEqual(len(ward._records(doc)), 2)
        for key, value in (("input_indices", [1, 2, 3]), ("tree", {}),
                           ("distances", ["1"]), ("tree_entry_present", True)):
            changed = copy.deepcopy(doc)
            changed["cases"]["a_probe"]["isolated"]["groups"][0][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                ward._records(changed)

    def test_rejects_nonliteral_nonfinite_or_negative_scalars(self):
        for value in (1, True, "NaN", "Inf", "-0.5", " 1", "1\t2", "1e999"):
            doc = document()
            doc["cases"]["a_probe"]["isolated"]["groups"][0]["distances"][0] = value
            with self.subTest(value=value), self.assertRaises(ValueError):
                ward._records(doc)

    def test_rejects_shapes_topologies_and_wrong_leaf_order(self):
        mutations = [
            ("distances", ["1"]),
            ("tree_present", False),
            ("tree_entry_present", False),
            ("input_indices", [1, True, 3]),
            ("input_indices", [1, 1, 3]),
            ("tree.merge", [[-1, -3], [-2, 2]]),
            ("tree.merge", [[-1, -3], [-1, 1]]),
            ("tree.merge", [[-1, -3]]),
            ("tree.merge", [[-1, -3], [-2, True]]),
            ("tree.height", ["0.5"]),
            ("tree.height", ["0.5", "0.25"]),
            ("tree.order", [1, 2, 3]),
            ("tree.order", [2, 1, 1]),
            ("tree.labels", ["c1"]),
            ("tree.method", "ward.D"),
        ]
        for key, value in mutations:
            doc = document()
            target = doc["cases"]["a_probe"]["isolated"]["groups"][0]
            parts = key.split(".")
            for part in parts[:-1]:
                target = target[part]
            target[parts[-1]] = value
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                ward._records(doc)

    def test_rejects_unsafe_or_empty_case_identity(self):
        for name in ("", "a/b", "a\tb", "../a"):
            doc = document()
            doc["cases"][name] = doc["cases"].pop("a_probe")
            with self.subTest(name=name), self.assertRaises(ValueError):
                ward._records(doc)

    def test_extract_is_hash_count_bound_and_exclusive(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, output = root / "witness.json", root / "trees.tsv"
            raw = json.dumps(document()).encode()
            source.write_bytes(raw)
            with self.assertRaises(ValueError):
                ward.extract(source, output)
            self.assertFalse(output.exists())
            expected = {"cases": 2, "routes": 3, "groups": 3,
                        "trees": 3, "pairs": 9, "max_leaves": 3}
            with patch.object(ward, "WITNESS_SHA256", hashlib.sha256(raw).hexdigest()):
                with self.assertRaises(ValueError):
                    ward.extract(source, output)
                self.assertFalse(output.exists())
                with patch.object(ward, "EXPECTED_COUNTS", expected):
                    result = ward.extract(source, output)
                    self.assertEqual(result["counts"], expected)
                    text = output.read_text()
                    self.assertEqual(len(text.splitlines()), 18)
                    self.assertTrue(text.startswith("tree\ta_probe/isolated/0\t3\n"))
                    self.assertIn("merge\t-1,-3\t-2,1\n", text)
                    self.assertIn("order\t2\t1\t3\nend\n", text)
                    with self.assertRaises(FileExistsError):
                        ward.extract(source, output)
                    self.assertEqual(output.read_text(), text)


if __name__ == "__main__":
    unittest.main()
