import copy
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import tempfile
import tarfile
import io
import unittest
from unittest import mock


CHECKER = Path(__file__).with_name("validate_infercnv_samples_clustering_witness.py")


class TreeTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue(CHECKER.exists(), "samples clustering checker is absent")
        spec = importlib.util.spec_from_file_location("samples_checker", CHECKER)
        self.checker = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.checker)
        self.tree = {"merge": [[-1, -2], [-3, 1]], "height": ["0", "2"],
                     "order": [3, 1, 2], "labels": ["a", "b", "c"],
                     "method": "ward.D2"}

    def test_accepts_complete_ward_tree_and_shuffled_global_indices(self):
        self.assertEqual(self.checker.validate_tree(self.tree, [3, 1, 2], ["b", "c", "a"]),
                         [2, 3, 1])

    def test_rejects_current_or_future_merge_reference(self):
        for child in [1, 2, 3]:
            tree = copy.deepcopy(self.tree)
            tree["merge"][0][1] = child
            with self.assertRaisesRegex(ValueError, "merge reference"):
                self.checker.validate_tree(tree, [1, 2, 3], ["a", "b", "c"])

    def test_rejects_duplicate_leaf_and_missing_leaf(self):
        tree = copy.deepcopy(self.tree)
        tree["merge"][0][1] = -1
        with self.assertRaisesRegex(ValueError, "leaf coverage"):
            self.checker.validate_tree(tree, [1, 2, 3], ["a", "b", "c"])

    def test_rejects_leaf_order_not_matching_merge_tree(self):
        tree = copy.deepcopy(self.tree)
        tree["order"] = [1, 2, 3]
        with self.assertRaisesRegex(ValueError, "tree order"):
            self.checker.validate_tree(tree, [1, 2, 3], ["a", "b", "c"])

    def test_rejects_labels_not_matching_input_indices(self):
        tree = copy.deepcopy(self.tree)
        tree["labels"][0] = "other"
        with self.assertRaisesRegex(ValueError, "tree labels"):
            self.checker.validate_tree(tree, [1, 2, 3], ["a", "b", "c"])

    def test_rejects_negative_nonfinite_or_decreasing_heights(self):
        for heights in [["-1", "2"], ["NaN", "2"], ["0", "Inf"], ["3", "2"]]:
            tree = copy.deepcopy(self.tree)
            tree["height"] = heights
            with self.assertRaisesRegex(ValueError, "height"):
                self.checker.validate_tree(tree, [1, 2, 3], ["a", "b", "c"])

    def test_tree_absence_is_only_valid_for_small_groups(self):
        self.assertEqual(self.checker.validate_tree(None, [2, 1], ["a", "b"]), [2, 1])
        with self.assertRaisesRegex(ValueError, "missing tree"):
            self.checker.validate_tree(None, [1, 2, 3], ["a", "b", "c"])

    def test_rejects_tree_for_single_cell(self):
        tree = {"merge": [], "height": [], "order": [1], "labels": ["a"], "method": "ward.D2"}
        with self.assertRaisesRegex(ValueError, "small group"):
            self.checker.validate_tree(tree, [1], ["a"])

    def test_rejects_noninteger_merge_indices_and_wrong_method(self):
        for field, value in [("merge", [[-1, -2], [True, 1]]), ("method", "ward.D")]:
            tree = copy.deepcopy(self.tree)
            tree[field] = value
            with self.assertRaises(ValueError):
                self.checker.validate_tree(tree, [1, 2, 3], ["a", "b", "c"])


class GroupTests(unittest.TestCase):
    def setUp(self):
        TreeTests.setUp(self)
        self.cells = [["a", "B", "observation"], ["b", "A", "observation"],
                      ["c", "B", "observation"], ["d", "R2", "reference"],
                      ["e", "R1", "reference"]]
        self.maps = [["reference", "R1", "5", "e"], ["reference", "R2", "4", "d"],
                     ["observation", "A", "2", "b"], ["observation", "B", "1", "a"],
                     ["observation", "B", "3", "c"]]

    def group(self, name, indices, tree=None):
        names = [self.cells[i - 1][0] for i in indices]
        ordered = indices if tree is None else [indices[i - 1] for i in tree["order"]]
        return {"name": name, "input_indices": indices, "input_index_names": names if len(indices) > 1 else None,
                "tree_present": tree is not None, "tree_entry_present": tree is not None, "tree": tree,
                "subclusters": [{"name": name + "_s1", "indices": ordered,
                                 "index_names": [self.cells[i - 1][0] for i in ordered] if len(indices) > 1 else None}],
                "distances": [] if tree is None else ["1", "1", "2"],
                "distance_labels": None if tree is None else names}

    def groups(self):
        return [self.group("A", [2]), self.group("B", [1, 3]),
                self.group("R1", [5]), self.group("R2", [4])]

    def test_accepts_ordered_groups_and_small_missing_trees(self):
        self.checker.validate_groups(self.groups(), self.cells, self.maps, True, 2)

    def test_rejects_reference_reordering(self):
        groups = self.groups()
        groups[2], groups[3] = groups[3], groups[2]
        with self.assertRaisesRegex(ValueError, "group order"):
            self.checker.validate_groups(groups, self.cells, self.maps, True, 2)

    def test_pooled_members_follow_map_concatenation_not_column_order(self):
        tree = {"merge": [[-2, -3], [-1, 1]], "height": ["1", "2"],
                "order": [1, 2, 3], "labels": ["b", "a", "c"], "method": "ward.D2"}
        groups = [self.group("all_observations", [2, 1, 3], tree),
                  self.group("R1", [5]), self.group("R2", [4])]
        self.checker.validate_groups(groups, self.cells, self.maps, False, 2)
        groups[0]["input_indices"] = [1, 2, 3]
        with self.assertRaisesRegex(ValueError, "group indices"):
            self.checker.validate_groups(groups, self.cells, self.maps, False, 2)

    def test_rejects_subcluster_member_reordering(self):
        groups = self.groups()
        groups[1]["subclusters"][0]["indices"] = [3, 1]
        with self.assertRaisesRegex(ValueError, "subcluster"):
            self.checker.validate_groups(groups, self.cells, self.maps, True, 2)

    def test_rejects_missing_map_cell_and_overlapping_maps(self):
        for maps in [self.maps[:-1], self.maps + [self.maps[-1]]]:
            with self.assertRaisesRegex(ValueError, "map"):
                self.checker.validate_groups(self.groups(), self.cells, maps, True, 2)

    def test_rejects_inconsistent_missing_tree_presence(self):
        groups = self.groups()
        groups[0]["tree_entry_present"] = True
        with self.assertRaisesRegex(ValueError, "tree presence"):
            self.checker.validate_groups(groups, self.cells, self.maps, True, 2)

    def test_rejects_bad_distance_shape_and_nonfinite_distances(self):
        tree = {"merge": [[-2, -3], [-1, 1]], "height": ["1", "2"],
                "order": [1, 2, 3], "labels": ["b", "a", "c"], "method": "ward.D2"}
        for values in [["1"], ["1", "NaN", "2"], ["1", "-1", "2"]]:
            groups = [self.group("all_observations", [2, 1, 3], tree),
                      self.group("R1", [5]), self.group("R2", [4])]
            groups[0]["distances"] = values
            with self.assertRaisesRegex(ValueError, "distance"):
                self.checker.validate_groups(groups, self.cells, self.maps, False, 2)

    def test_rejects_named_indices_that_hide_source_null_names(self):
        groups = self.groups()
        groups[0]["input_index_names"] = ["b"]
        with self.assertRaisesRegex(ValueError, "index names"):
            self.checker.validate_groups(groups, self.cells, self.maps, True, 2)

    def test_probe_maps_preserve_deliberate_nonascending_order(self):
        maps = copy.deepcopy(self.maps)
        maps[-2], maps[-1] = maps[-1], maps[-2]
        groups = self.groups()
        groups[1] = self.group("B", [3, 1])
        self.checker.validate_groups(groups, self.cells, maps, True, 2)

    def test_group_container_cannot_hide_tree_or_subcluster_entry(self):
        route = {"groups": self.groups(), "hc_entry_exists": False, "result_tree_group_names": [],
                 "result_subcluster_group_names": ["A", "B", "R1", "R2"]}
        self.checker.validate_group_container(route)
        route["result_subcluster_group_names"] = ["A", "B", "R1"]
        with self.assertRaisesRegex(ValueError, "container"):
            self.checker.validate_group_container(route)


class PathTests(unittest.TestCase):
    def setUp(self):
        TreeTests.setUp(self)

    def test_rejects_traversal_and_symlink(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "target").write_text("data")
            (root / "link").symlink_to(root / "target")
            for path in ["../target", "/target", "a/../target", "link"]:
                with self.assertRaisesRegex(ValueError, "path"):
                    self.checker.checked_file(root, path)


class RouteTests(unittest.TestCase):
    def setUp(self):
        TreeTests.setUp(self)
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        outputs = {"expression": "gene\ta\tb\tc\ng\t0\t0\t2\nh\t1\t1\t1\n",
                   "genes": "gene\tchr\tstart\tstop\ng\tchr1\t1\t2\nh\tchr1\t3\t4\n",
                   "cells": "cell\tgroup\trole\na\tT\tobservation\nb\tT\tobservation\nc\tT\tobservation\n",
                   "maps": "role\tgroup\tindex\tcell\nobservation\tT\t1\ta\nobservation\tT\t2\tb\nobservation\tT\t3\tc\n"}
        paths = {}
        for key, value in outputs.items():
            path = self.root / (key + ".tsv")
            path.write_text(value)
            paths[key] = path.name
        self.state = self.checker.read_state(self.root, paths)
        (self.root / "result.rds").write_bytes(b"preserved RDS fixture")
        (self.root / "second.rds").write_bytes(b"preserved NULL fixture")
        (self.root / "filtered.tsv").write_text(outputs["expression"])
        self.route = {"status": "success", "warnings": [], "state": paths,
                      "checkpoint": "result.rds", "second_result": "second.rds", "second_result_is_null": True,
                      "diagnostic": {"selected_gene_indices": [1, 2], "outliers": None,
                                     "filter_guard_active": False, "filtered_expression": "filtered.tsv"},
                      "groups": [{"name": "T", "input_indices": [1, 2, 3],
                                  "declared_indices": [1, 2, 3], "input_index_names": ["a", "b", "c"],
                                  "tree_present": True, "tree_entry_present": True, "tree": self.tree,
                                  "subclusters": [{"name": "T_s1", "indices": [3, 1, 2],
                                                   "index_names": ["c", "a", "b"]}],
                                  "distances": ["0", "2", "2"], "distance_labels": ["a", "b", "c"]}]}

    def test_accepts_bound_state_and_no_filter_diagnostic(self):
        self.checker.validate_route(self.root, self.route, self.state, True, 0)

    def test_rejects_expression_mutation_even_when_both_routes_would_match(self):
        (self.root / "expression.tsv").write_text("gene\ta\tb\tc\ng\t0\t0\t99\nh\t1\t1\t1\n")
        with self.assertRaisesRegex(ValueError, "unchanged state"):
            self.checker.validate_route(self.root, self.route, self.state, True, 0)

    def test_rejects_filtered_value_mutation(self):
        (self.root / "filtered.tsv").write_text("gene\ta\tb\tc\ng\t0\t0\t99\nh\t1\t1\t1\n")
        with self.assertRaisesRegex(ValueError, "filtered matrix"):
            self.checker.validate_route(self.root, self.route, self.state, True, 0)

    def test_rejects_filter_selection_without_source_guard(self):
        self.route["diagnostic"]["selected_gene_indices"] = [1]
        with self.assertRaisesRegex(ValueError, "filter selection"):
            self.checker.validate_route(self.root, self.route, self.state, True, 0)

    def test_workflow_cannot_pass_as_characterization_error(self):
        self.route = {"status": "error", "warnings": [],
                      "error": {"message": "actual error", "classes": ["simpleError", "error", "condition"], "call": "real_call()"}}
        with self.assertRaisesRegex(ValueError, "workflow error"):
            self.checker.validate_route(self.root, self.route, self.state, True, 0)

    def test_missing_preserved_result_fails(self):
        self.route["checkpoint"] = "missing.rds"
        with self.assertRaisesRegex(ValueError, "path"):
            self.checker.validate_route(self.root, self.route, self.state, True, 0)

    def test_zero_outliers_active_filter_selects_no_rows_not_all_rows(self):
        state = copy.deepcopy(self.state)
        state["cells"][1][1:] = ["R", "reference"]
        state["maps"][1] = ["reference", "R", "1", "a"]
        diagnostic = {"filter_guard_active": True, "outliers": [], "selected_gene_indices": [],
                      "filtered_expression": "filtered.tsv"}
        (self.root / "filtered.tsv").write_text("gene\ta\tb\tc\n")
        self.checker.validate_filter(self.root, diagnostic, state, 0.2)
        diagnostic["selected_gene_indices"] = [1, 2]
        with self.assertRaisesRegex(ValueError, "negative indexing"):
            self.checker.validate_filter(self.root, diagnostic, state, 0.2)

    def test_full_run_does_not_invent_list_return_or_second_result(self):
        self.route.pop("second_result")
        self.route.pop("second_result_is_null")
        self.checker.validate_route(self.root, self.route, self.state, True, 0, raw_list=False)

    def test_probe_error_is_characterization_not_workflow_success(self):
        error = {"status": "error", "warnings": [], "pre_call_checkpoint": "result.rds",
                 "error": {"message": "actual source failure", "classes": ["error", "condition"], "call": "source_call()"}}
        result = self.checker.validate_route(self.root, error, self.state, True, 0, workflow=False)
        self.assertEqual(result["status"], "characterization_error")
        error["error"]["classes"] = ["condition"]
        with self.assertRaisesRegex(ValueError, "error class"):
            self.checker.validate_route(self.root, error, self.state, True, 0, workflow=False)

    def test_boundary_filter_is_explicitly_unresolved(self):
        state = copy.deepcopy(self.state)
        state["expression"] = [["gene", "a", "b", "c"]]
        offsets = [-math.sqrt(1.829998), -0.801, -0.8, -0.799, 0.799, 0.8, 0.801, math.sqrt(1.829998)]
        state["expression"] += [[f"g{i}", str(2 + x), str(2 + x), "2"] for i, x in enumerate(offsets, 1)]
        state["genes"] = [["gene", "chr", "start", "stop"]] + [[f"g{i}", "chr1", str(i), str(i + 1)] for i in range(1, 9)]
        state["cells"][1][1:] = ["R", "reference"]
        state["cells"][2][1:] = ["R", "reference"]
        state["maps"] = [["role", "group", "index", "cell"], ["reference", "R", "1", "a"],
                         ["reference", "R", "2", "b"], ["observation", "T", "3", "c"]]
        diagnostic = {"filter_guard_active": True, "outliers": [1, 2, 3, 6, 7, 8],
                      "selected_gene_indices": [4, 5], "filtered_expression": "filtered.tsv"}
        (self.root / "filtered.tsv").write_text("\n".join("\t".join(row) for row in
            [state["expression"][0], state["expression"][4], state["expression"][5]]) + "\n")
        self.assertEqual(self.checker.validate_filter(self.root, diagnostic, state, 0.8), [3, 6])

    def test_filter_accepts_ascii_numeric_padding_without_normalizing_identity(self):
        state = copy.deepcopy(self.state)
        state["expression"][1][1] = "                 0"
        self.checker.validate_filter(self.root, self.route["diagnostic"], state, 0)
        (self.root / "filtered.tsv").write_text("gene\ta\tb\tc\ng\t0\t0\t2.0000000000000004\nh\t1\t1\t1\n")
        with self.assertRaisesRegex(ValueError, "filtered matrix"):
            self.checker.validate_filter(self.root, self.route["diagnostic"], state, 0)

    def test_filter_does_not_trim_gene_or_cell_identifiers(self):
        (self.root / "filtered.tsv").write_text("gene\ta\tb\tc\n g\t0\t0\t2\nh\t1\t1\t1\n")
        with self.assertRaisesRegex(ValueError, "filtered matrix"):
            self.checker.validate_filter(self.root, self.route["diagnostic"], self.state, 0)

    def test_distances_bind_values_and_lower_column_order_to_actual_input(self):
        self.checker.validate_distances(self.route["groups"], self.state, [1, 2])
        self.route["groups"][0]["distances"] = ["2", "0", "2"]
        with self.assertRaisesRegex(ValueError, "Euclidean distance"):
            self.checker.validate_distances(self.route["groups"], self.state, [1, 2])


class ArgumentsTests(unittest.TestCase):
    def setUp(self):
        TreeTests.setUp(self)
        self.args = {"isolated": dict(self.checker.ISOLATED, cluster_by_groups=True, z_score_filter=0)}

    def test_all_isolated_formals_are_bound(self):
        self.checker.validate_arguments(self.args, True, 0)
        self.args["isolated"].pop("restrict_to_DE_genes")
        with self.assertRaisesRegex(ValueError, "argument identity"):
            self.checker.validate_arguments(self.args, True, 0)

    def test_wrong_ward_or_partition_cannot_pass(self):
        for key, value in [("hclust_method", "ward.D"), ("partition_method", "leiden"), ("cluster_by_groups", False)]:
            args = copy.deepcopy(self.args)
            args["isolated"][key] = value
            with self.assertRaisesRegex(ValueError, "argument identity"):
                self.checker.validate_arguments(args, True, 0)

    def test_boolean_cannot_alias_numeric_argument(self):
        self.args["isolated"]["leiden_resolution_per_chr"] = True
        with self.assertRaisesRegex(ValueError, "argument identity"):
            self.checker.validate_arguments(self.args, True, 0)


class IntegrationTests(unittest.TestCase):
    def setUp(self):
        RouteTests.setUp(self)
        self.witness = self.root / "witness"
        self.witness.mkdir()
        self.bundles = {}
        self.pin = copy.deepcopy(self.checker.PACKAGE)
        archive = io.BytesIO()
        self.source_payloads = {"R/inferCNV_ops.R": b"synthetic ops fixture", "R/inferCNV_tumor_subclusters.R": b"synthetic cluster fixture"}
        with tarfile.open(fileobj=archive, mode="w:gz") as tar:
            for name, data in self.source_payloads.items():
                member = tarfile.TarInfo("infercnv-" + self.checker.SOURCE_COMMIT + "/" + name)
                member.size = len(data)
                tar.addfile(member, io.BytesIO(data))
        self.archive = archive.getvalue()
        self.pin["source_archive_sha256"] = hashlib.sha256(self.archive).hexdigest()
        patch = mock.patch.object(self.checker, "PACKAGE", self.pin)
        patch.start()
        self.addCleanup(patch.stop)
        patch = mock.patch.object(self.checker, "SOURCE_SHA256", self.pin["source_archive_sha256"])
        patch.start()
        self.addCleanup(patch.stop)
        for bundle in ("synthetic", "shipped"):
            root = self.root / bundle
            root.mkdir()
            for key, path in self.route["state"].items():
                (root / path).write_bytes((self.root / path).read_bytes())
            for name in ("stage1.rds", "stage14.rds", "counts", "positions", "annotations"):
                (root / name).write_bytes(b"structural fixture, not installed-package evidence")
            stage = {"expression": "expression.tsv", "gene_order": "genes.tsv", "cell_groups": "cells.tsv"}
            entry = {"stages": {"1": dict(stage, checkpoint="stage1.rds"), "14": dict(stage, checkpoint="stage14.rds")}}
            inputs = {"counts.tsv": "counts", "gene_order.tsv": "positions", "annotations.tsv": "annotations"} if bundle == "synthetic" else {
                "canonical": "counts", "original_gene_order": "positions", "original_annotations": "annotations"}
            entries = {}
            for source_bundle, profile in self.checker.WORKFLOWS:
                if source_bundle != bundle:
                    continue
                item = copy.deepcopy(entry)
                if bundle == "synthetic":
                    item["settings"] = {"ref_group_names": [], "ref_subtract_use_mean_bounds": True}
                else:
                    item.update(reference_groups=[], input_counts="canonical")
                entries[profile] = item
            source = {"package": self.pin, "inputs": inputs,
                      "profiles" if bundle == "synthetic" else "cases": entries,
                      "settings" if bundle == "synthetic" else "run_settings": {}}
            (root / "oracle.json").write_text(json.dumps(source))
            hashes = {p.name: self.checker.digest(p) for p in root.iterdir()}
            (root / "sha256-manifest.tsv").write_text("sha256\tpath\n" + "".join(f"{sha}\t{name}\n" for name, sha in sorted(hashes.items())))
            self.bundles[bundle] = (root, source, hashes)
        metadata = {key: "metadata/" + key for key in ("session", "packages", "source", "functions", "exporter", "oracle_io",
                    "synthetic_script", "shipped_script", "synthetic_manifest", "shipped_manifest", "source_archive")}
        (self.witness / "metadata").mkdir()
        for key, path in metadata.items():
            payload = self.archive if key == "source_archive" else b"metadata fixture"
            if key.endswith("_manifest"):
                payload = (self.bundles[key.split("_")[0]][0] / "sha256-manifest.tsv").read_bytes()
            (self.witness / path).write_bytes(payload)
        (self.witness / metadata["packages"]).write_text("Package\tVersion\tLibPath\ninfercnv\t1.28.0\t/home/runner/work/_temp/Library\nfastcluster\t1.3.0\t/home/runner/work/_temp/Library\nparallelDist\t0.2.7\t/home/runner/work/_temp/Library\n")
        (self.witness / metadata["source"]).write_text(f"source_commit\t{self.checker.SOURCE_COMMIT}\nsource_archive_sha256\t{self.pin['source_archive_sha256']}\nindex_base\t1\ndistance_ordering\tlower-column-major\n")
        functions = ["function\tsource_path\tsource_sha256\tinstalled_text\tinstalled_text_sha256"]
        for name, source in (("run", "R/inferCNV_ops.R"), (".get_relevant_args_list", "R/inferCNV_ops.R"),
                             ("define_signif_tumor_subclusters", "R/inferCNV_tumor_subclusters.R"),
                             (".single_tumor_subclustering", "R/inferCNV_tumor_subclusters.R")):
            installed = "function-" + name + ".txt"
            (self.witness / "metadata" / installed).write_bytes(b"installed text fixture")
            functions.append("\t".join([name, source, hashlib.sha256(self.source_payloads[source]).hexdigest(), installed,
                                         hashlib.sha256(b"installed text fixture").hexdigest()]))
        (self.witness / metadata["functions"]).write_text("\n".join(functions) + "\n")
        cases = {}
        workflows = {f"{b}_{p}_{m}_filter{v}" for b, p in self.checker.WORKFLOWS for m in ("grouped", "pooled") for v in ("0", "08")}
        for name in sorted(workflows | self.checker.PROBES):
            grouped = "_pooled_" not in name and "pooled_reference_name_collision" not in name
            z = 0.8 if name.endswith("filter08") else 0.2 if name.endswith("filter02") else 0
            args = {"isolated": dict(self.checker.ISOLATED, cluster_by_groups=grouped, z_score_filter=z)}
            probe = name in self.checker.PROBES
            route = self.make_route(name, "isolated", grouped)
            inputs = {}
            case = {"id": name, "kind": "probe" if probe else "workflow", "arguments": args, "isolated": route}
            if probe:
                before = self.make_route(name, "before", True)
                inputs = {"reference_groups": [], "pre_call_checkpoint": before["checkpoint"], "pre_call_state": before["state"]}
                route["pre_call_checkpoint"] = inputs["pre_call_checkpoint"]
            else:
                b, p = next((b, p) for b, p in self.checker.WORKFLOWS if name.startswith(b + "_" + p + "_"))
                root, _, hashes = self.bundles[b]
                inputs = {"bundle": b, "profile": p, "reference_groups": [], "stage1_checkpoint": "stage1.rds",
                          "stage14_checkpoint": "stage14.rds", "sha256s": {"manifest": self.checker.digest(root / "sha256-manifest.tsv"),
                          "oracle": hashes["oracle.json"], "stage1": hashes["stage1.rds"], "stage14": hashes["stage14.rds"]},
                          "source_files": {key: {"path": path, "sha256": hashes[path]} for key, path in
                                           (("counts", "counts"), ("positions", "positions"), ("annotations", "annotations"))}}
                run = {key: None for key in self.checker.RUN_FORMALS}
                run.update(up_to_step=15, analysis_mode="samples", hclust_method="ward.D2", cluster_by_groups=grouped,
                    z_score_filter=z, num_threads=1, num_ref_groups=None, HMM=False, scale_data=False, denoise=False,
                    prune_outliers=False, mask_nonDE_genes=False, remove_genes_at_chr_ends=False, plot_steps=False,
                    inspect_subclusters=False, resume_mode=False, no_plot=True, no_prelim_plot=True, save_rds=True,
                    per_chr_hmm_subclusters=False, per_chr_hmm_subclusters_references=False,
                    out_dir=f"cases/{name}/full_run/upstream")
                if b == "synthetic":
                    run["ref_subtract_use_mean_bounds"] = True
                args["full_run"] = run
                full = self.make_route(name, "full_run", grouped)
                full.pop("second_result")
                full.pop("second_result_is_null")
                for key in ("stage14", "stage15", "preliminary"):
                    checkpoint = self.make_route(name, key, grouped)
                    full[key + "_checkpoint"] = checkpoint["checkpoint"]
                    full[key + "_state"] = checkpoint["state"]
                case.update(full_run=full, route_agreement=True)
            case["inputs"] = inputs
            cases[name] = case
        self.record = {"schema_version": 1, "package": self.pin, "index_base": 1, "metadata": metadata, "cases": cases,
                       "runtime": {"R_version": "4.6.1", "locale": "C", "seed": 260930, "requested_threads": 1,
                                   "thread_environment": {"OMP_NUM_THREADS": "1", "OPENBLAS_NUM_THREADS": "1", "MKL_NUM_THREADS": "1"},
                                   "package_library_paths": {name: "/home/runner/work/_temp/Library" for name in ("infercnv", "fastcluster", "parallelDist")},
                                   "package_versions": {"infercnv": "1.28.0", "fastcluster": "1.3.0", "parallelDist": "0.2.7"}}}

    def make_route(self, name, kind, grouped):
        prefix = f"cases/{name}/{kind}"
        directory = self.witness / prefix
        directory.mkdir(parents=True)
        route = copy.deepcopy(self.route)
        for key, path in route["state"].items():
            (directory / path).write_bytes((self.root / path).read_bytes())
            route["state"][key] = prefix + "/" + path
        for key in ("checkpoint", "second_result"):
            path = route[key]
            (directory / path).write_bytes((self.root / path).read_bytes())
            route[key] = prefix + "/" + path
        (directory / "list.rds").write_bytes(b"list fixture")
        route["result_list"] = prefix + "/list.rds"
        (directory / "filtered.tsv").write_bytes((self.root / "filtered.tsv").read_bytes())
        route["diagnostic"]["filtered_expression"] = prefix + "/filtered.tsv"
        if not grouped:
            route["groups"][0]["name"] = "all_observations"
            route["groups"][0]["subclusters"][0]["name"] = "all_observations_s1"
        route["hc_entry_exists"] = True
        route["result_tree_group_names"] = [route["groups"][0]["name"]]
        route["result_subcluster_group_names"] = [route["groups"][0]["name"]]
        return route

    def validate(self):
        self.record["files"] = {p.relative_to(self.witness).as_posix(): self.checker.digest(p)
                                for p in self.witness.rglob("*") if p.is_file() and p.name != "witness.json"}
        (self.witness / "witness.json").write_text(json.dumps(self.record))
        return self.checker.validate_witness(self.bundles["synthetic"][0], self.bundles["shipped"][0], self.witness)

    def test_structural_fixture_runs_all_cases_without_claiming_package_acceptance(self):
        result = self.validate()
        self.assertEqual((result["workflow_cases"], result["probe_cases"]), (16, 20))
        self.assertEqual(result["native_compatibility"], "not_assessed")
        self.assertIn("pending_original_artifact", result["acceptance"])

    def test_missing_preliminary_checkpoint_fails_even_with_updated_inventory(self):
        case = next(case for case in self.record["cases"].values() if case["kind"] == "workflow")
        del case["full_run"]["preliminary_checkpoint"]
        with self.assertRaisesRegex(ValueError, "path"):
            self.validate()

    def test_source_checkpoint_hash_mutation_fails(self):
        case = next(case for case in self.record["cases"].values() if case["kind"] == "workflow")
        case["inputs"]["sha256s"]["stage14"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "checkpoint/manifest hash"):
            self.validate()

    def test_route_disagreement_fails_not_deferred_as_probe(self):
        case = next(case for case in self.record["cases"].values() if case["kind"] == "workflow")
        case["full_run"]["groups"][0]["tree"]["height"][1] = "3"
        with self.assertRaisesRegex(ValueError, "routes differ"):
            self.validate()

    def test_function_source_hash_mutation_fails_with_updated_inventory(self):
        path = self.witness / self.record["metadata"]["functions"]
        text = path.read_text()
        path.write_text(text.replace(hashlib.sha256(self.source_payloads["R/inferCNV_ops.R"]).hexdigest(), "0" * 64))
        with self.assertRaisesRegex(ValueError, "function source hash"):
            self.validate()

    def test_hidden_installed_function_text_is_rejected_before_upload(self):
        path = self.witness / self.record["metadata"]["functions"]
        old = "function-.single_tumor_subclustering.txt"
        hidden = ".single_tumor_subclustering.function.txt"
        (self.witness / "metadata" / old).rename(self.witness / "metadata" / hidden)
        path.write_text(path.read_text().replace(old, hidden))
        with self.assertRaisesRegex(ValueError, "hidden archive path"):
            self.validate()

    def test_upload_omission_cannot_pass_recorded_inventory(self):
        self.validate()
        omitted = self.witness / "metadata" / "function-.get_relevant_args_list.txt"
        omitted.unlink()
        with self.assertRaisesRegex(ValueError, "file inventory"):
            self.checker.validate_witness(self.bundles["synthetic"][0],
                                          self.bundles["shipped"][0], self.witness)

    def test_packages_metadata_cannot_disagree_with_runtime(self):
        path = self.witness / self.record["metadata"]["packages"]
        path.write_text(path.read_text().replace("1.3.0", "1.2.0"))
        with self.assertRaisesRegex(ValueError, "package metadata"):
            self.validate()

    def test_original_duplicate_matrix_inventory_is_preserved(self):
        path = self.witness / self.record["metadata"]["packages"]
        original = path.read_text()
        rows = "Matrix\t1.7-6\t/home/runner/work/_temp/Library\nMatrix\t1.7-5\t/opt/R/4.6.1/lib/R/library\n"
        path.write_text(original + rows)
        result = self.validate()
        self.assertEqual(result["workflow_cases"], 16)
        self.assertEqual(path.read_text(), original + rows)

    def test_runtime_version_must_match_loaded_library_not_another_install(self):
        path = self.witness / self.record["metadata"]["packages"]
        path.write_text(path.read_text() + "fastcluster\t1.2.0\t/opt/R/4.6.1/lib/R/library\n")
        self.record["runtime"]["package_versions"]["fastcluster"] = "1.2.0"
        with self.assertRaisesRegex(ValueError, "^package metadata differs from loaded namespace$"):
            self.validate()

    def test_runtime_library_path_must_match_inventory_not_just_version(self):
        self.record["runtime"]["package_library_paths"]["parallelDist"] = "/opt/R/4.6.1/lib/R/library"
        with self.assertRaisesRegex(ValueError, "^package metadata differs from loaded namespace$"):
            self.validate()

    def test_duplicate_package_at_distinct_library_paths_is_valid(self):
        path = self.witness / self.record["metadata"]["packages"]
        path.write_text(path.read_text() + "fastcluster\t1.2.0\t/opt/R/4.6.1/lib/R/library\n")
        self.assertEqual(self.validate()["workflow_cases"], 16)

    def test_duplicate_package_in_same_library_is_invalid(self):
        path = self.witness / self.record["metadata"]["packages"]
        path.write_text(path.read_text() + "fastcluster\t1.2.0\t/home/runner/work/_temp/Library\n")
        with self.assertRaisesRegex(ValueError, "duplicate package metadata"):
            self.validate()

    def test_jointly_mutated_route_distances_do_not_make_an_oracle(self):
        case = next(case for case in self.record["cases"].values() if case["kind"] == "workflow")
        for key in ("isolated", "full_run"):
            case[key]["groups"][0]["distances"] = ["0", "99", "99"]
        with self.assertRaisesRegex(ValueError, "Euclidean distance"):
            self.validate()

    def test_runtime_requires_c_locale_and_all_thread_environment_limits(self):
        for key, value in [("locale", "en_US.UTF-8"), ("thread_environment", {"OMP_NUM_THREADS": "2"})]:
            runtime = copy.deepcopy(self.record["runtime"])
            runtime[key] = value
            with self.assertRaisesRegex(ValueError, "runtime"):
                self.checker.validate_runtime(runtime)


class ProvenanceTests(unittest.TestCase):
    def setUp(self):
        TreeTests.setUp(self)
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "data").write_bytes(b"original")
        self.files = {"data": hashlib.sha256(b"original").hexdigest()}
        (self.root / "witness.json").write_text("{}")

    def test_accepts_exact_file_inventory(self):
        self.checker.validate_inventory(self.root, self.files, {"witness.json"})

    def test_rejects_mutated_bytes(self):
        (self.root / "data").write_bytes(b"mutated")
        with self.assertRaisesRegex(ValueError, "hash"):
            self.checker.validate_inventory(self.root, self.files, {"witness.json"})

    def test_rejects_unrecorded_file(self):
        (self.root / "extra").write_bytes(b"unrecorded")
        with self.assertRaisesRegex(ValueError, "inventory"):
            self.checker.validate_inventory(self.root, self.files, {"witness.json"})

    def test_rejects_duplicate_json_key(self):
        (self.root / "witness.json").write_text('{"cases":{},"cases":{}}')
        with self.assertRaisesRegex(ValueError, "duplicate"):
            self.checker.read_json(self.root / "witness.json")

    def test_rejects_incomplete_case_inventory(self):
        with self.assertRaisesRegex(ValueError, "case inventory"):
            self.checker.validate_case_inventory({})

    def test_runtime_requires_all_pins_and_numeric_not_boolean_seed(self):
        runtime = {"R_version": "4.6.1", "locale": "C", "seed": 260930,
                   "thread_environment": {"OMP_NUM_THREADS": "1", "OPENBLAS_NUM_THREADS": "1", "MKL_NUM_THREADS": "1"},
                   "package_library_paths": {name: "/home/runner/work/_temp/Library" for name in ("infercnv", "fastcluster", "parallelDist")},
                   "requested_threads": 1, "package_versions": {"infercnv": "1.28.0", "fastcluster": "1.3.0", "parallelDist": "0.2.7"}}
        self.checker.validate_runtime(runtime)
        for key, value in [("R_version", "4.5.0"), ("seed", True), ("requested_threads", 2), ("package_versions", {})]:
            bad = copy.deepcopy(runtime)
            bad[key] = value
            with self.assertRaisesRegex(ValueError, "runtime"):
                self.checker.validate_runtime(bad)

    def test_loaded_namespace_library_paths_are_complete_and_absolute(self):
        runtime = {"R_version": "4.6.1", "locale": "C", "seed": 260930, "requested_threads": 1,
                   "thread_environment": {"OMP_NUM_THREADS": "1", "OPENBLAS_NUM_THREADS": "1", "MKL_NUM_THREADS": "1"},
                   "package_versions": {"infercnv": "1.28.0", "fastcluster": "1.3.0", "parallelDist": "0.2.7"}}
        for paths in ({}, {"infercnv": "/library"},
                      {name: "relative" for name in ("infercnv", "fastcluster", "parallelDist")}):
            runtime["package_library_paths"] = paths
            with self.assertRaisesRegex(ValueError, "runtime"):
                self.checker.validate_runtime(runtime)


if __name__ == "__main__":
    unittest.main()
