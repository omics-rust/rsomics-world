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
        for outputs in ([], ["--output", "a", "--profiles-output", "b"],
                        ["--output", "a", "--filter-output", "b"],
                        ["--profiles-output", "a", "--filter-output", "b"]):
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


class FilterRouteFixtures(unittest.TestCase):
    HEADER = ("route\texpression\tgenes\tcells\tmaps\tfiltered_expression\tfeatures\tcell_count"
              "\treference_indices\tz_score_filter\tfilter_guard_active\toutliers"
              "\tselected_gene_indices")
    COUNTS = {"cases": 2, "routes": 3, "groups": 3,
              "trees": 3, "pairs": 9, "max_leaves": 3}

    def fixture(self, root, z=0.8, references=True, outliers=(1, 2, 3, 7, 8),
                selected=(4, 5, 6)):
        doc = document()
        source = {key: "input/" + key + ".tsv" for key in
                  ("expression", "genes", "cells", "maps")}
        state = {key: "state/" + key + ".tsv" for key in source}
        labels = {1: "a_ref", 2: "z_ref", 6: "z_ref"} if references else {}
        rows = ["gene\t" + "\t".join(f"c{i}" for i in range(1, 8))]
        rows += [f"g{i}\t0\t-0\t{i}.25\t{i}.5\t{i}\t{i}.75\t{i}e-1"
                 for i in range(1, 9)]
        texts = {
            "expression": "\n".join(rows) + "\n",
            "genes": "gene\tchr\tstart\tstop\n" + "".join(
                f"g{i}\tchr1\t{i * 10}\t{i * 10 + 1}\n" for i in range(1, 9)),
            "cells": "cell\tgroup\trole\n" + "".join(
                f"c{i}\t{labels.get(i, 'obs')}\t"
                + ("reference\n" if i in labels else "observation\n")
                for i in range(1, 8)),
            "maps": "role\tgroup\tindex\tcell\n" + "".join(
                ("reference" if i in labels else "observation")
                + f"\t{labels.get(i, 'obs')}\t{i}\tc{i}\n"
                for i in ([6, 2, 1, 5, 3, 4, 7] if references else [5, 3, 4, 7, 1, 6, 2])),
        }
        doc["files"] = {}
        for key, text in texts.items():
            for path in (source[key], state[key]):
                self.write(root, doc, path, text)
        self.write(root, doc, "filtered.tsv", "\n".join([rows[0]] + [rows[i] for i in selected]) + "\n")
        for case in doc["cases"].values():
            case["inputs"] = {"reference_groups": ["z_ref", "a_ref"] if references else []}
            case["arguments"] = {"isolated": {"z_score_filter": z}}
            if case["kind"] == "probe":
                case["inputs"]["pre_call_state"] = source.copy()
            else:
                case["arguments"]["full_run"] = {"z_score_filter": z}
                case["route_agreement"] = True
                case["full_run"]["stage14_state"] = source.copy()
            for route in ("isolated", "full_run"):
                if route not in case:
                    continue
                case[route]["state"] = state.copy()
                case[route]["diagnostic"] = {
                    "filtered_expression": "filtered.tsv", "selected_gene_indices": list(selected),
                    "outliers": None if outliers is None else list(outliers),
                    "filter_guard_active": z > 0 and references,
                }
        return doc

    def write(self, root, doc, relative, text):
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        doc["files"][relative] = hashlib.sha256(text.encode()).hexdigest()

    def replace(self, root, doc, relative, old, new):
        self.write(root, doc, relative, (root / relative).read_text().replace(old, new, 1))

    def run_extract(self, root, doc, counts=True, cli=False):
        raw = json.dumps(doc).encode()
        source, target = root / "witness.json", root / "filter.tsv"
        source.write_bytes(raw)
        with patch.object(ward, "WITNESS_SHA256", hashlib.sha256(raw).hexdigest()), \
                patch.object(ward, "EXPECTED_COUNTS", self.COUNTS if counts else ward.EXPECTED_COUNTS):
            if not cli:
                return ward.extract_filter_routes(source, target)
            with patch.object(sys, "argv", ["extract", "--witness", str(source),
                                           "--filter-output", str(target)]), \
                    patch("sys.stdout", new_callable=io.StringIO) as output:
                ward.main()
                return json.loads(output.getvalue())

    def test_records_actual_inputs_reference_order_and_literal_selection(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = self.run_extract(root, self.fixture(root))
            data = (root / "filter.tsv").read_bytes()
            suffix = ("input/expression.tsv\tinput/genes.tsv\tinput/cells.tsv\tinput/maps.tsv"
                      "\tfiltered.tsv\t8\t7\t5,1,0\t0.8\ttrue\t0,1,2,6,7\t3,4,5")
            isolated = suffix.replace("input/", "state/")
            self.assertEqual(data.decode().splitlines(), [self.HEADER,
                "a_probe/isolated\t" + suffix, "z_workflow/isolated\t" + isolated,
                "z_workflow/full_run\t" + suffix])
            self.assertEqual(result["counts"], self.COUNTS)
            self.assertEqual(result["fixture_sha256"], hashlib.sha256(data).hexdigest())
            self.assertEqual(result["fixture_bytes"], len(data))
            with self.assertRaises(FileExistsError):
                self.run_extract(root, self.fixture(root))

    def test_null_empty_inactive_and_zero_feature_routes(self):
        for z, refs, outliers, selected, suffix in (
            (0, True, None, range(1, 9), "5,1,0\t0\tfalse\tNULL\t0,1,2,3,4,5,6,7"),
            (-1, True, None, range(1, 9), "5,1,0\t-1\tfalse\tNULL\t0,1,2,3,4,5,6,7"),
            (0.8, False, None, range(1, 9), "[]\t0.8\tfalse\tNULL\t0,1,2,3,4,5,6,7"),
            (0.2, True, [], [], "5,1,0\t0.2\ttrue\t[]\t[]"),
            (0.8, True, range(1, 9), [], "5,1,0\t0.8\ttrue\t0,1,2,3,4,5,6,7\t[]"),
        ):
            with self.subTest(z=z, refs=refs, outliers=outliers), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self.run_extract(root, self.fixture(root, z, refs, outliers, selected))
                self.assertTrue(all(row.endswith(suffix) for row in
                                    (root / "filter.tsv").read_text().splitlines()[1:]))

    def test_positive_arguments_keep_literal_boundary_selection(self):
        for z in (0.2, 0.8):
            with self.subTest(z=z), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                doc = self.fixture(root, z=z)
                self.replace(root, doc, "filtered.tsv", "g4\t0", "g4\t0.0")
                self.run_extract(root, doc)
                self.assertTrue(all(row.endswith(f"{z}\ttrue\t0,1,2,6,7\t3,4,5") for row in
                                    (root / "filter.tsv").read_text().splitlines()[1:]))

    def test_rejects_inactive_outliers_and_nonidentity_selection(self):
        for outliers, selected in (([], range(1, 9)), (None, (4, 5, 6))):
            with self.subTest(outliers=outliers), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                doc = self.fixture(root, z=0, outliers=outliers, selected=selected)
                with self.assertRaises(ValueError):
                    self.run_extract(root, doc)
                self.assertFalse((root / "filter.tsv").exists())

    def test_rejects_unbound_sources_and_wrong_explicit_input(self):
        for mutation in ("hash", "missing_hash", "path", "symlink", "counts", "pre_call",
                         "stage14", "isolated_drift", "route_state_drift", "missing_state_key"):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                doc = self.fixture(root)
                probe = doc["cases"]["a_probe"]
                workflow = doc["cases"]["z_workflow"]
                if mutation == "hash":
                    doc["files"]["input/maps.tsv"] = "0" * 64
                elif mutation == "missing_hash":
                    del doc["files"]["input/genes.tsv"]
                elif mutation == "path":
                    probe["inputs"]["pre_call_state"]["expression"] = "../expression.tsv"
                elif mutation == "symlink":
                    (root / "link.tsv").symlink_to(root / "input/expression.tsv")
                    probe["inputs"]["pre_call_state"]["expression"] = "link.tsv"
                elif mutation == "pre_call":
                    del probe["inputs"]["pre_call_state"]
                elif mutation == "stage14":
                    del workflow["full_run"]["stage14_state"]
                elif mutation in ("isolated_drift", "route_state_drift"):
                    text = (root / "state/expression.tsv").read_text().replace("g1\t0", "g1\t-0", 1)
                    self.write(root, doc, "drift.tsv", text)
                    target = workflow if mutation == "isolated_drift" else probe
                    target["isolated"]["state"]["expression"] = "drift.tsv"
                elif mutation == "missing_state_key":
                    del probe["inputs"]["pre_call_state"]["maps"]
                with self.assertRaises(ValueError):
                    self.run_extract(root, doc, counts=mutation != "counts")
                self.assertFalse((root / "filter.tsv").exists())

    def test_rejects_malformed_maps_cells_genes_and_dimensions(self):
        mutations = (
            ("maps", "\t6\tc6", "\t8\tc6"),
            ("maps", "\t6\tc6", "\tTrue\tc6"),
            ("maps", "reference\tz_ref\t2\tc2\n", ""),
            ("maps", "reference\tz_ref\t2\tc2", "reference\tz_ref\t6\tc6"),
            ("maps", "reference\tz_ref\t6\tc6", "observation\tz_ref\t6\tc6"),
            ("maps", "reference\ta_ref\t1\tc1", "reference\tz_ref\t1\tc1"),
            ("cells", "c2\tz_ref", "c1\tz_ref"),
            ("cells", "cell\tgroup\trole", "cell\twrong\trole"),
            ("genes", "g2\tchr1", "g1\tchr1"),
            ("genes", "g1\tchr1\t10\t11", "g1\tchr1\t12\t11"),
            ("expression", "gene\tc1\tc2", "gene\tc2\tc1"),
            ("expression", "g1\t0\t-0", "g1\t0"),
            ("expression", "g8\t0\t-0\t8.25\t8.5\t8\t8.75\t8e-1\n", ""),
            ("expression", "g1\t0", "g1\tNaN"),
        )
        for key, old, new in mutations:
            with self.subTest(key=key, old=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                doc = self.fixture(root)
                for prefix in ("input", "state"):
                    self.replace(root, doc, f"{prefix}/{key}.tsv", old, new)
                with self.assertRaises(ValueError):
                    self.run_extract(root, doc)
                self.assertFalse((root / "filter.tsv").exists())
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            doc = self.fixture(root)
            doc["cases"]["a_probe"]["inputs"]["reference_groups"] = ["a_ref", "z_ref"]
            with self.assertRaises(ValueError):
                self.run_extract(root, doc)

    def test_rejects_selection_guard_argument_and_filtered_row_drift(self):
        for mutation in ("selection_bool", "selection_bounds", "selection_order", "selection_duplicate",
                         "outliers_null", "outliers_bool", "outliers_order", "complement",
                         "guard", "argument_bool", "argument_nan", "filtered_header", "filtered_gene",
                         "filtered_value", "filtered_negative_zero", "filtered_width", "filtered_extra",
                         "filtered_missing", "filtered_order"):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                doc = self.fixture(root)
                case = doc["cases"]["a_probe"]
                diagnostic = case["isolated"]["diagnostic"]
                if mutation.startswith("selection_"):
                    diagnostic["selected_gene_indices"] = {
                        "selection_bool": [True], "selection_bounds": [9],
                        "selection_order": [6, 5, 4], "selection_duplicate": [4, 4],
                    }[mutation]
                elif mutation.startswith("outliers_"):
                    diagnostic["outliers"] = {
                        "outliers_null": None, "outliers_bool": [True], "outliers_order": [8, 1],
                    }[mutation]
                elif mutation == "complement":
                    diagnostic["outliers"] = []
                elif mutation == "guard":
                    diagnostic["filter_guard_active"] = 1
                elif mutation.startswith("argument_"):
                    case["arguments"]["isolated"]["z_score_filter"] = (
                        True if mutation == "argument_bool" else float("nan"))
                else:
                    changes = {"filtered_header": ("gene\tc1\tc2", "gene\tc2\tc1"),
                        "filtered_gene": ("g4\t", "g2\t"),
                        "filtered_value": ("g4\t0", "g4\t1e-300"),
                        "filtered_negative_zero": ("g4\t0", "g4\t-0"),
                        "filtered_width": ("g4\t0\t-0", "g4\t0"),
                        "filtered_extra": ("g6\t", "extra\t0\t0\t0\t0\t0\t0\t0\ng6\t"),
                        "filtered_missing": ("g6\t0\t-0\t6.25\t6.5\t6\t6.75\t6e-1\n", ""),
                        "filtered_order": ("g4\t", "g5\t")}
                    self.replace(root, doc, "filtered.tsv", *changes[mutation])
                with self.assertRaises(ValueError):
                    self.run_extract(root, doc)
                self.assertFalse((root / "filter.tsv").exists())

    def test_cli_selects_filter_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = self.run_extract(root, self.fixture(root), cli=True)
            self.assertEqual(result["counts"], self.COUNTS)
            self.assertEqual((root / "filter.tsv").read_text().splitlines()[0], self.HEADER)


if __name__ == "__main__":
    unittest.main()
