import copy
import hashlib
import json
from pathlib import Path
import tempfile
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
