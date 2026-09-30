"""Validate preserved installed-infercnv samples-clustering evidence."""

import argparse
import csv
import hashlib
import json
import math
import struct
import tarfile
from pathlib import Path, PurePosixPath


SOURCE_COMMIT = "b421d9405c97a309b081ef86d455e976df93eae4"
SOURCE_SHA256 = "b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207"
PACKAGE = {"name": "infercnv", "version": "1.28.0", "source_commit": SOURCE_COMMIT,
           "source_archive_sha256": SOURCE_SHA256}
WORKFLOWS = {(bundle, profile) for bundle, profiles in
             (("synthetic", ("grouped_bounds", "no_reference")), ("shipped", ("subset", "full")))
             for profile in profiles}
PROBES = {"probe_" + name + "_filter0" for name in (
    "small_groups_grouped", "small_groups_pooled", "equal_profiles_grouped",
    "equal_distances_grouped", "equal_distances_permuted_grouped",
    "multiple_reference_order_grouped", "multiple_reference_order_pooled",
    "pooled_reference_name_collision")}
PROBES |= {f"probe_filter_{name}_filter{value}" for name in
           ("boundary", "constant", "no_qualifying", "all_qualifying") for value in ("0", "02", "08")}
ISOLATED = {"p_val": 0.1, "k_nn": 20, "leiden_method": "PCA", "leiden_function": "CPM",
            "leiden_resolution": "auto", "leiden_method_per_chr": "simple",
            "leiden_function_per_chr": "modularity", "leiden_resolution_per_chr": 1,
            "hclust_method": "ward.D2", "partition_method": "none",
            "per_chr_hmm_subclusters": False, "per_chr_hmm_subclusters_references": False,
            "restrict_to_DE_genes": False}
RUN_FORMALS = set("""cutoff min_cells_per_gene out_dir window_length smooth_method num_ref_groups
ref_subtract_use_mean_bounds cluster_by_groups cluster_references k_obs_groups hclust_method
max_centered_threshold scale_data HMM HMM_transition_prob HMM_report_by HMM_type HMM_i3_pval
HMM_i3_use_KS BayesMaxPNormal sim_method sim_foreground reassignCNVs analysis_mode
tumor_subcluster_partition_method tumor_subcluster_pval k_nn leiden_method leiden_function
leiden_resolution leiden_method_per_chr leiden_function_per_chr leiden_resolution_per_chr
per_chr_hmm_subclusters per_chr_hmm_subclusters_references z_score_filter denoise noise_filter
sd_amplifier noise_logistic outlier_method_bound outlier_lower_bound outlier_upper_bound
final_scale_limits final_center_val debug num_threads plot_steps inspect_subclusters resume_mode
png_res plot_probabilities save_rds save_final_rds diagnostics remove_genes_at_chr_ends
prune_outliers mask_nonDE_genes mask_nonDE_pval test.use require_DE_all_normals
hspike_aggregate_normals no_plot no_prelim_plot write_expr_matrix write_phylo output_format
plot_chr_scale chr_lengths useRaster up_to_step""".split())


def integer(value):
    return isinstance(value, int) and not isinstance(value, bool)


def same_json(actual, expected):
    if isinstance(expected, dict):
        return isinstance(actual, dict) and actual.keys() == expected.keys() and \
            all(same_json(actual[key], value) for key, value in expected.items())
    if isinstance(expected, list):
        return isinstance(actual, list) and len(actual) == len(expected) and all(
            same_json(a, b) for a, b in zip(actual, expected))
    if isinstance(expected, bool):
        return actual is expected
    if isinstance(expected, (int, float)):
        return isinstance(actual, (int, float)) and not isinstance(actual, bool) and actual == expected
    return type(actual) is type(expected) and actual == expected


def finite(value, label):
    if isinstance(value, bool) or not isinstance(value, (str, int, float)):
        raise ValueError(f"invalid {label}")
    try:
        number = float(value)
    except ValueError as error:
        raise ValueError(f"invalid {label}") from error
    if not math.isfinite(number):
        raise ValueError(f"nonfinite {label}")
    return number


def validate_tree(tree, indices, cell_names):
    if not isinstance(indices, list) or any(not integer(i) or i < 1 or i > len(cell_names)
                                           for i in indices) or len(set(indices)) != len(indices):
        raise ValueError("invalid tree input indices")
    n = len(indices)
    if tree is None:
        if n > 2:
            raise ValueError("missing tree for large group")
        return indices[:]
    if n <= 2:
        raise ValueError("tree supplied for small group")
    if not isinstance(tree, dict) or tree.get("method") != "ward.D2":
        raise ValueError("invalid tree method")
    merges = tree.get("merge")
    heights = tree.get("height")
    if not isinstance(merges, list) or len(merges) != n - 1 or \
            not isinstance(heights, list) or len(heights) != n - 1:
        raise ValueError("invalid tree dimensions")
    heights = [finite(x, "height") for x in heights]
    if any(x < 0 for x in heights) or any(a > b for a, b in zip(heights, heights[1:])):
        raise ValueError("invalid Ward height order")
    uses = {i: 0 for i in range(1, n - 1)}
    leaves = []
    for row, pair in enumerate(merges, 1):
        if not isinstance(pair, list) or len(pair) != 2:
            raise ValueError("invalid merge shape")
        for child in pair:
            if not integer(child) or child == 0 or child > 0 and child >= row:
                raise ValueError("invalid merge reference")
            if child < 0:
                if child < -n:
                    raise ValueError("invalid leaf coverage")
                leaves.append(-child)
            else:
                uses[child] += 1
    if sorted(leaves) != list(range(1, n + 1)) or any(count != 1 for count in uses.values()):
        raise ValueError("invalid leaf coverage or reused merge")
    order = []
    pending = [n - 1]
    while pending:
        child = pending.pop()
        if child < 0:
            order.append(-child)
        else:
            left, right = merges[child - 1]
            pending.extend([right, left])
    if tree.get("order") != order or any(not integer(i) for i in tree.get("order", [])):
        raise ValueError("tree order differs from merge traversal")
    if tree.get("labels") != [cell_names[i - 1] for i in indices]:
        raise ValueError("tree labels differ from input indices")
    return [indices[i - 1] for i in order]


def checked_file(root, relative):
    if not isinstance(relative, str) or not relative or "\\" in relative:
        raise ValueError("invalid evidence path")
    path = PurePosixPath(relative)
    if path.is_absolute() or str(path) != relative or any(p in (".", "..") for p in path.parts):
        raise ValueError("invalid evidence path")
    candidate = root
    for part in path.parts:
        candidate = candidate / part
        if candidate.is_symlink():
            raise ValueError("symlink evidence path")
    if not candidate.is_file() or not candidate.resolve().is_relative_to(root.resolve()):
        raise ValueError("missing evidence path")
    return candidate


def validate_maps(cells, maps):
    names = [row[0] for row in cells]
    if not cells or len(set(names)) != len(names):
        raise ValueError("invalid cell identity")
    ordered = {"reference": {}, "observation": {}}
    assigned = set()
    completed = set()
    previous = None
    for row in maps:
        if len(row) != 4:
            raise ValueError("invalid map shape")
        role, group, text, cell = row
        try:
            index = int(text)
        except ValueError as error:
            raise ValueError("invalid map index") from error
        key = (role, group)
        if key != previous:
            if key in completed:
                raise ValueError("noncontiguous map group")
            completed.add(key)
            previous = key
        if role not in ordered or not group or index < 1 or index > len(cells) or \
                index in assigned or cells[index - 1] != [cell, group, role]:
            raise ValueError("map identity or coverage differs")
        values = ordered[role].setdefault(group, [])
        values.append(index)
        assigned.add(index)
    if assigned != set(range(1, len(cells) + 1)):
        raise ValueError("incomplete map coverage")
    return ordered


def validate_groups(groups, cells, maps, grouped, selected_genes, collision=False):
    ordered = validate_maps(cells, maps)
    observations = list(ordered["observation"].items()) if grouped else [
        ("all_observations", [i for indices in ordered["observation"].values() for i in indices])]
    expected = observations + list(ordered["reference"].items())
    if not isinstance(groups, list) or [g.get("name") for g in groups] != [name for name, _ in expected]:
        raise ValueError("group order differs from ordered maps")
    if not collision and len({name for name, _ in expected}) != len(expected):
        raise ValueError("group name collision")
    lookup = {}
    for name, indices in expected:
        lookup.setdefault(name, indices)
    names = [row[0] for row in cells]
    for actual, (name, declared) in zip(groups, expected):
        indices = lookup[name] if collision else declared
        if actual.get("input_indices") != indices or actual.get("declared_indices", declared) != declared:
            raise ValueError("group indices differ from ordered maps")
        expected_names = [names[i - 1] for i in indices] if len(indices) > 1 and selected_genes != 1 else None
        if actual.get("input_index_names") != expected_names:
            raise ValueError("input index names differ")
        tree = actual.get("tree")
        present = tree is not None
        if actual.get("tree_present") is not present or actual.get("tree_entry_present") is not present:
            raise ValueError("tree presence differs")
        members = validate_tree(tree, indices, names)
        subclusters = actual.get("subclusters")
        member_names = None if expected_names is None else [names[i - 1] for i in members]
        if subclusters != [{"name": name + "_s1", "indices": members, "index_names": member_names}]:
            raise ValueError("subcluster ordered membership or names differ")
        distances = actual.get("distances")
        count = len(indices) * (len(indices) - 1) // 2 if present else 0
        if not isinstance(distances, list) or len(distances) != count or \
                any(finite(x, "distance") < 0 for x in distances):
            raise ValueError("invalid distance shape or values")
        labels = [names[i - 1] for i in indices] if present else None
        if actual.get("distance_labels") != labels:
            raise ValueError("distance labels differ from group indices")


def validate_group_container(route):
    groups = route["groups"]
    trees = list(dict.fromkeys(group["name"] for group in groups if group["tree"] is not None))
    subclusters = list(dict.fromkeys(group["name"] for group in groups))
    if route.get("result_tree_group_names") != trees or route.get("result_subcluster_group_names") != subclusters or \
            not isinstance(route.get("hc_entry_exists"), bool) or trees and route["hc_entry_exists"] is not True:
        raise ValueError("result tree/subcluster container differs")


def digest(path):
    with path.open("rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def read_json(path):
    def unique(pairs):
        result = {}
        for name, value in pairs:
            if name in result:
                raise ValueError("duplicate JSON key")
            result[name] = value
        return result
    return json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique,
                      parse_constant=lambda value: (_ for _ in ()).throw(ValueError("nonfinite JSON value")))


def validate_inventory(root, files, excluded):
    if not isinstance(files, dict) or not files:
        raise ValueError("empty file inventory")
    actual = set()
    for path in root.rglob("*"):
        if path.is_symlink():
            raise ValueError("symlink evidence path")
        if path.is_file():
            actual.add(path.relative_to(root).as_posix())
    if actual - excluded != set(files):
        raise ValueError("file inventory differs from preserved files")
    for name, expected in files.items():
        if not isinstance(expected, str) or len(expected) != 64 or digest(checked_file(root, name)) != expected:
            raise ValueError("file hash differs: " + name)


def validate_runtime(runtime):
    if not isinstance(runtime, dict) or runtime.get("R_version") != "4.6.1" or \
            runtime.get("locale") != "C" or \
            not integer(runtime.get("seed")) or runtime["seed"] != 260930 or \
            not integer(runtime.get("requested_threads")) or runtime["requested_threads"] != 1:
        raise ValueError("unpinned runtime")
    if runtime.get("thread_environment") != {name: "1" for name in ("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS")}:
        raise ValueError("unpinned runtime thread environment")
    versions = runtime.get("package_versions")
    if not isinstance(versions, dict) or set(versions) != {"infercnv", "fastcluster", "parallelDist"} or \
            versions.get("infercnv") != "1.28.0" or \
            any(not isinstance(versions.get(name), str) or not versions[name] for name in ("fastcluster", "parallelDist")):
        raise ValueError("missing runtime package versions")
    libraries = runtime.get("package_library_paths")
    if not isinstance(libraries, dict) or set(libraries) != {"infercnv", "fastcluster", "parallelDist"} or \
            any(not isinstance(path, str) or not path or not PurePosixPath(path).is_absolute() or
                str(PurePosixPath(path)) != path or ".." in PurePosixPath(path).parts for path in libraries.values()):
        raise ValueError("invalid runtime loaded-namespace library paths")


def validate_case_inventory(cases):
    workflow_ids = {f"{bundle}_{profile}_{mode}_filter{value}" for bundle, profile in WORKFLOWS
                    for mode in ("grouped", "pooled") for value in ("0", "08")}
    if not isinstance(cases, dict) or set(cases) != workflow_ids | PROBES or \
            any(not isinstance(case, dict) or case.get("kind") != ("probe" if name in PROBES else "workflow")
                for name, case in cases.items()):
        raise ValueError("missing or extra case inventory")


def validate_arguments(arguments, grouped, zscore, full=False, source=None, case_id=None):
    expected = dict(ISOLATED, cluster_by_groups=grouped, z_score_filter=zscore)
    if not isinstance(arguments, dict) or not same_json(arguments.get("isolated"), expected):
        raise ValueError("isolated argument identity differs")
    if not full:
        if set(arguments) != {"isolated"}:
            raise ValueError("probe has non-isolated arguments")
        return
    run = arguments.get("full_run")
    if not isinstance(run, dict) or set(run) != RUN_FORMALS or set(arguments) != {"isolated", "full_run"}:
        raise ValueError("incomplete full-run arguments")
    fixed = {"up_to_step": 15, "analysis_mode": "samples", "hclust_method": "ward.D2",
             "cluster_by_groups": grouped, "z_score_filter": zscore, "num_threads": 1,
             "num_ref_groups": None, "HMM": False, "scale_data": False, "denoise": False,
             "prune_outliers": False, "mask_nonDE_genes": False, "remove_genes_at_chr_ends": False,
             "plot_steps": False, "inspect_subclusters": False, "resume_mode": False,
             "no_plot": True, "no_prelim_plot": True, "save_rds": True,
             "per_chr_hmm_subclusters": False, "per_chr_hmm_subclusters_references": False,
             "out_dir": f"cases/{case_id}/full_run/upstream"}
    if any(not same_json(run.get(key), value) for key, value in fixed.items()):
        raise ValueError("full-run boundary argument differs")
    for key, value in (source or {}).items():
        if key in run and key not in fixed and not same_json(run[key], value):
            raise ValueError("source profile argument differs: " + key)


def table(path, header, empty=False):
    with path.open(newline="", encoding="utf-8") as handle:
        rows = list(csv.reader(handle, delimiter="\t"))
    if not rows or rows[0][:len(header)] != list(header) or not empty and len(rows) < 2 or \
            any(len(row) != len(rows[0]) for row in rows[1:]):
        raise ValueError(f"invalid evidence table: {path}")
    return rows


def read_state(root, paths):
    if not isinstance(paths, dict) or set(paths) != {"expression", "genes", "cells", "maps"}:
        raise ValueError("invalid state paths")
    headers = {"expression": ("gene",), "genes": ("gene", "chr", "start", "stop"),
               "cells": ("cell", "group", "role"), "maps": ("role", "group", "index", "cell")}
    state = {key: table(checked_file(root, paths[key]), header) for key, header in headers.items()}
    expression = state["expression"]
    names = [row[0] for row in expression[1:]]
    cells = expression[0][1:]
    if not cells or len(set(cells)) != len(cells) or len(set(names)) != len(names) or \
            any(not x for x in cells + names) or [row[0] for row in state["genes"][1:]] != names or \
            [row[0] for row in state["cells"][1:]] != cells:
        raise ValueError("invalid state gene/cell identity")
    for row in expression[1:]:
        for value in row[1:]:
            finite(value, "expression")
    for row in state["genes"][1:]:
        if len(row) != 4 or not row[1] or not all(x.lstrip("-").isdigit() for x in row[2:]) or \
                int(row[2]) > int(row[3]):
            raise ValueError("invalid gene coordinates")
    validate_maps(state["cells"][1:], state["maps"][1:])
    return state


def validate_conditions(conditions, error=False):
    if not isinstance(conditions, list):
        raise ValueError("invalid warning records")
    for condition in conditions:
        if not isinstance(condition, dict) or not isinstance(condition.get("message"), str) or \
                not condition["message"] or not isinstance(condition.get("classes"), list) or \
                not condition["classes"] or any(not isinstance(x, str) or not x for x in condition["classes"]) or \
                condition.get("call") is not None and not isinstance(condition.get("call"), str):
            raise ValueError("invalid condition record")
        if error and "error" not in condition["classes"]:
            raise ValueError("error record lacks error class")


def validate_filter(root, diagnostic, state, zscore):
    ordered = validate_maps(state["cells"][1:], state["maps"][1:])
    active = zscore > 0 and bool(ordered["reference"])
    if not isinstance(diagnostic, dict) or diagnostic.get("filter_guard_active") is not active:
        raise ValueError("filter guard differs from source")
    n = len(state["genes"]) - 1
    selected = diagnostic.get("selected_gene_indices")
    outliers = diagnostic.get("outliers")
    if not isinstance(selected, list) or any(not integer(x) or x < 1 or x > n for x in selected) or \
            selected != sorted(set(selected)):
        raise ValueError("invalid filter selection")
    ambiguous = []
    if not active:
        if outliers is not None or selected != list(range(1, n + 1)):
            raise ValueError("filter selection differs without source guard")
    else:
        if not isinstance(outliers, list) or any(not integer(x) or x < 1 or x > n for x in outliers) or \
                outliers != sorted(set(outliers)):
            raise ValueError("invalid filter outliers")
        refs = [i for indices in ordered["reference"].values() for i in indices]
        rows = [[finite(row[i], "reference value") for i in refs] for row in state["expression"][1:]]
        values = [rows[j][i] for i in range(len(refs)) for j in range(n)]
        mean = math.fsum(values) / len(values)
        mean += math.fsum(x - mean for x in values) / len(values)
        sd = math.sqrt(math.fsum((x - mean) ** 2 for x in values) / (len(values) - 1)) if len(values) > 1 else 0
        expected = []
        for index, row in enumerate(rows, 1):
            if sd:
                score = math.fsum(abs((x - mean) / sd) for x in row) / len(refs)
                if abs(score - 0.8) <= 1e-12:
                    ambiguous.append(index)
                elif score >= 0.8:
                    expected.append(index)
        if set(outliers) - set(ambiguous) != set(expected):
            raise ValueError("filter outliers differ from literal 0.8 source threshold")
        expected_selection = [i for i in range(1, n + 1) if i not in outliers] if outliers else []
        if selected != expected_selection:
            raise ValueError("filter selection differs from source negative indexing")
    filtered = table(checked_file(root, diagnostic.get("filtered_expression")), ("gene",), empty=True)
    expected = [state["expression"][0]] + [state["expression"][i] for i in selected]
    if filtered[0] != expected[0] or len(filtered) != len(expected) or any(
            actual[0] != original[0] or any(struct.pack("!d", finite(a, "filtered value")) !=
                                          struct.pack("!d", finite(b, "input value"))
                for a, b in zip(actual[1:], original[1:])) for actual, original in zip(filtered[1:], expected[1:])):
        raise ValueError("filtered matrix differs from selected input rows")
    return ambiguous


def validate_route(root, route, state, grouped, zscore, workflow=True, collision=False, raw_list=True,
                   check_distances=True):
    validate_conditions(route.get("warnings"))
    if route.get("status") == "error":
        if workflow:
            raise ValueError("workflow error cannot be accepted")
        validate_conditions([route.get("error")], error=True)
        checked_file(root, route.get("pre_call_checkpoint"))
        ambiguous = validate_filter(root, route["diagnostic"], state, zscore) if route.get("diagnostic") else []
        return {"status": "characterization_error", "error": route["error"],
                "numerical_filter_boundary_genes": ambiguous}
    if route.get("status") != "success":
        raise ValueError("invalid route status")
    if read_state(root, route.get("state")) != state:
        raise ValueError("unchanged state differs from pre-clustering input")
    checked_file(root, route.get("checkpoint"))
    if raw_list:
        checked_file(root, route.get("second_result"))
        if route.get("second_result_is_null") is not True:
            raise ValueError("unexpected non-NULL second result")
    ambiguous = validate_filter(root, route.get("diagnostic"), state, zscore)
    selected = route["diagnostic"]["selected_gene_indices"]
    validate_groups(route.get("groups"), state["cells"][1:], state["maps"][1:], grouped, len(selected), collision)
    if check_distances:
        validate_distances(route["groups"], state, selected)
    return {"status": "workflow_success" if workflow else "characterization_success",
            "numerical_filter_boundary_genes": ambiguous,
            "numerical_filter_boundary_status": "unresolved_characterization" if ambiguous else "not_near_boundary"}


def validate_distances(groups, state, selected):
    max_error = 0.0
    indices = {i for group in groups if group.get("tree") is not None for i in group["input_indices"]}
    columns = {i: [finite(state["expression"][g][i], "distance input") for g in selected] for i in indices}
    for group in groups:
        if group.get("tree") is None:
            continue
        indices = group["input_indices"]
        expected = []
        for i, a in enumerate(indices[:-1]):
            for b in indices[i + 1:]:
                distance = math.dist(columns[a], columns[b])
                if not math.isfinite(distance):
                    raise ValueError("nonfinite Euclidean distance from input")
                expected.append(distance)
        actual = group.get("distances")
        if not isinstance(actual, list) or len(actual) != len(expected):
            raise ValueError("Euclidean distance shape differs")
        for value, distance in zip(actual, expected):
            error = abs(finite(value, "distance") - distance)
            max_error = max(max_error, error)
            if error > 1e-12 * max(1.0, distance):
                raise ValueError("Euclidean distance values or lower-column order differ")
    return max_error


def validate_metadata(root, metadata, runtime):
    versions = table(checked_file(root, metadata["packages"]), ("Package", "Version", "LibPath"))
    installed = {}
    for name, version, library in versions[1:]:
        key = (name, library)
        if key in installed:
            raise ValueError("duplicate package metadata")
        installed[key] = version
    if any(installed.get((name, runtime["package_library_paths"][name])) != version
           for name, version in runtime["package_versions"].items()):
        raise ValueError("package metadata differs from loaded namespace")
    with checked_file(root, metadata["source"]).open(encoding="utf-8") as handle:
        rows = list(csv.reader(handle, delimiter="\t"))
    expected = {"source_commit": SOURCE_COMMIT, "source_archive_sha256": SOURCE_SHA256,
                "index_base": "1", "distance_ordering": "lower-column-major"}
    if any(len(row) != 2 for row in rows) or len({row[0] for row in rows}) != len(rows) or dict(rows) != expected:
        raise ValueError("source metadata differs from witness pins")
    definitions = table(checked_file(root, metadata["functions"]),
                        ("function", "source_path", "source_sha256", "installed_text", "installed_text_sha256"))
    sources = {"run": "R/inferCNV_ops.R", ".get_relevant_args_list": "R/inferCNV_ops.R",
               "define_signif_tumor_subclusters": "R/inferCNV_tumor_subclusters.R",
               ".single_tumor_subclustering": "R/inferCNV_tumor_subclusters.R"}
    if len(definitions) != 5 or {row[0] for row in definitions[1:]} != set(sources):
        raise ValueError("installed function inventory differs")
    parent = PurePosixPath(metadata["functions"]).parent
    with tarfile.open(checked_file(root, metadata["source_archive"]), mode="r:gz") as archive:
        names = archive.getnames()
        if len(set(names)) != len(names):
            raise ValueError("duplicate pinned archive member")
        for name, source, source_sha, installed_text, installed_sha in definitions[1:]:
            if sources[name] != source:
                raise ValueError("function source path differs")
            path = "infercnv-" + SOURCE_COMMIT + "/" + source
            member = archive.getmember(path)
            if not member.isfile():
                raise ValueError("invalid pinned source member")
            with archive.extractfile(member) as handle:
                source_actual = hashlib.file_digest(handle, "sha256").hexdigest()
            if source_actual != source_sha:
                raise ValueError("function source hash differs from pinned archive")
            if digest(checked_file(root, str(parent / installed_text))) != installed_sha:
                raise ValueError("installed function text hash differs")


def load_bundle(root):
    manifest = table(checked_file(root, "sha256-manifest.tsv"), ("sha256", "path"))
    files = {}
    for sha, name in manifest[1:]:
        if name in files:
            raise ValueError("duplicate source manifest path")
        files[name] = sha
    validate_inventory(root, files, {"sha256-manifest.tsv"})
    record = read_json(checked_file(root, "oracle.json"))
    if record.get("package") != PACKAGE:
        raise ValueError("source bundle package identity differs")
    return record, files


def source_state(root, stage, references):
    expression = table(checked_file(root, stage["expression"]), ("gene",))
    genes = table(checked_file(root, stage["gene_order"]), ("gene", "chr", "start", "stop"))
    cells = table(checked_file(root, stage["cell_groups"]), ("cell", "group", "role"))
    if [row[0] for row in expression[1:]] != [row[0] for row in genes[1:]] or \
            expression[0][1:] != [row[0] for row in cells[1:]]:
        raise ValueError("source stage identity differs")
    actual_refs = {row[1] for row in cells[1:] if row[2] == "reference"}
    if len(set(references)) != len(references) or set(references) != actual_refs:
        raise ValueError("requested reference identity differs")
    observations = sorted({row[1] for row in cells[1:] if row[2] == "observation"})
    maps = [["role", "group", "index", "cell"]]
    for role, groups in (("reference", references), ("observation", observations)):
        for group in groups:
            maps += [[role, group, str(i), row[0]] for i, row in enumerate(cells[1:], 1)
                     if row[1:] == [group, role]]
    validate_maps(cells[1:], maps[1:])
    return {"expression": expression, "genes": genes, "cells": cells, "maps": maps}


def validate_workflow_inputs(case, name, bundle_root, record, hashes):
    inputs = case.get("inputs", {})
    bundle, profile = inputs.get("bundle"), inputs.get("profile")
    if (bundle, profile) not in WORKFLOWS or not name.startswith(bundle + "_" + profile + "_"):
        raise ValueError("source profile identity differs")
    entry = record["profiles" if bundle == "synthetic" else "cases"][profile]
    refs = entry["settings"]["ref_group_names"] if bundle == "synthetic" else entry["reference_groups"]
    if inputs.get("reference_groups") != refs:
        raise ValueError("requested reference order differs")
    stages = entry["stages"]
    sha = inputs.get("sha256s", {})
    expected_sha = {"manifest": digest(checked_file(bundle_root, "sha256-manifest.tsv")),
                    "oracle": hashes["oracle.json"]}
    for number, label in (("1", "stage1"), ("14", "stage14")):
        path = stages[number]["checkpoint"]
        if inputs.get(label + "_checkpoint") != path:
            raise ValueError("source checkpoint identity differs")
        expected_sha[label] = hashes[path]
    if sha != expected_sha:
        raise ValueError("source checkpoint/manifest hash differs")
    source_inputs = record["inputs"]
    paths = {"counts": source_inputs["counts.tsv" if bundle == "synthetic" else entry["input_counts"]],
             "positions": source_inputs["gene_order.tsv" if bundle == "synthetic" else "original_gene_order"],
             "annotations": source_inputs["annotations.tsv" if bundle == "synthetic" else "original_annotations"]}
    if inputs.get("source_files") != {key: {"path": path, "sha256": hashes[path]} for key, path in paths.items()}:
        raise ValueError("source input file/hash differs")
    settings = record["settings" if bundle == "synthetic" else "run_settings"].copy()
    if bundle == "synthetic":
        settings["ref_subtract_use_mean_bounds"] = entry["settings"]["ref_subtract_use_mean_bounds"]
    return source_state(bundle_root, stages["14"], refs), settings


def validate_witness(synthetic, shipped, witness):
    record = read_json(checked_file(witness, "witness.json"))
    if not integer(record.get("schema_version")) or record["schema_version"] != 1 or record.get("package") != PACKAGE or \
            not integer(record.get("index_base")) or record["index_base"] != 1:
        raise ValueError("unpinned witness identity")
    validate_runtime(record.get("runtime"))
    validate_inventory(witness, record.get("files"), {"witness.json", "sha256-manifest.tsv"})
    cases = record.get("cases")
    validate_case_inventory(cases)
    bundles = {"synthetic": (synthetic, *load_bundle(synthetic)), "shipped": (shipped, *load_bundle(shipped))}
    metadata = record.get("metadata")
    required_metadata = {"session", "packages", "source", "functions", "exporter", "oracle_io",
                         "synthetic_script", "shipped_script", "synthetic_manifest", "shipped_manifest", "source_archive"}
    if not isinstance(metadata, dict) or set(metadata) != required_metadata:
        raise ValueError("incomplete metadata paths")
    for path in metadata.values():
        checked_file(witness, path)
    if digest(checked_file(witness, metadata["source_archive"])) != SOURCE_SHA256:
        raise ValueError("pinned source archive bytes differ")
    validate_metadata(witness, metadata, record["runtime"])
    for bundle in ("synthetic", "shipped"):
        if digest(checked_file(witness, metadata[bundle + "_manifest"])) != \
                digest(checked_file(bundles[bundle][0], "sha256-manifest.tsv")):
            raise ValueError("preserved source manifest differs")
    results = {}
    for name, case in cases.items():
        if case.get("id") != name:
            raise ValueError("case id differs from inventory key")
        grouped = "_pooled_" not in name and "pooled_reference_name_collision" not in name
        zscore = 0.8 if name.endswith("filter08") else 0.2 if name.endswith("filter02") else 0
        if case["kind"] == "workflow":
            bundle = case.get("inputs", {}).get("bundle")
            if bundle not in bundles:
                raise ValueError("unknown source bundle")
            state, settings = validate_workflow_inputs(case, name, *bundles[bundle])
            validate_arguments(case.get("arguments"), grouped, zscore, full=True, source=settings, case_id=name)
            isolated, full = case.get("isolated", {}), case.get("full_run", {})
            result = validate_route(witness, isolated, state, grouped, zscore)
            validate_route(witness, full, state, grouped, zscore, raw_list=False, check_distances=False)
            validate_group_container(isolated)
            validate_group_container(full)
            checked_file(witness, isolated.get("result_list"))
            for key in ("stage14_checkpoint", "stage15_checkpoint", "preliminary_checkpoint"):
                checked_file(witness, full.get(key))
            for key in ("stage14_state", "stage15_state", "preliminary_state"):
                if read_state(witness, full.get(key)) != state:
                    raise ValueError("full-run checkpoint state differs: " + key)
            if case.get("route_agreement") is not True or isolated.get("groups") != full.get("groups"):
                raise ValueError("isolated/full-run routes differ")
            for key in ("selected_gene_indices", "outliers", "filter_guard_active"):
                if isolated["diagnostic"][key] != full["diagnostic"][key]:
                    raise ValueError("isolated/full-run filter routes differ")
            results[name] = result
        else:
            if "full_run" in case:
                raise ValueError("probe cannot claim full-run evidence")
            validate_arguments(case.get("arguments"), grouped, zscore)
            inputs = case.get("inputs", {})
            checked_file(witness, inputs.get("pre_call_checkpoint"))
            state = read_state(witness, inputs.get("pre_call_state"))
            refs = list(validate_maps(state["cells"][1:], state["maps"][1:])["reference"])
            if inputs.get("reference_groups") != refs:
                raise ValueError("probe reference order differs")
            route = case.get("isolated", {})
            if route.get("pre_call_checkpoint") != inputs["pre_call_checkpoint"]:
                raise ValueError("probe pre-call checkpoint differs")
            results[name] = validate_route(witness, route, state, grouped, zscore, workflow=False,
                                           collision="pooled_reference_name_collision" in name)
            if route["status"] == "success":
                checked_file(witness, route.get("result_list"))
                validate_group_container(route)
    for probe in ("boundary", "constant", "no_qualifying", "all_qualifying"):
        a = cases[f"probe_filter_{probe}_filter02"]["isolated"]
        b = cases[f"probe_filter_{probe}_filter08"]["isolated"]
        if a["status"] != b["status"] or a.get("groups") != b.get("groups"):
            raise ValueError("positive filter guard outcomes differ")
        for key in ("selected_gene_indices", "outliers", "filter_guard_active"):
            if (a.get("diagnostic") or {}).get(key) != (b.get("diagnostic") or {}).get(key):
                raise ValueError("positive filter guard selection differs")
    return {"scope": "installed_package_witness_structure_only", "workflow_cases": 16, "probe_cases": 20,
            "cases": results, "native_compatibility": "not_assessed",
            "distance_check": {"purpose": "artifact_math_and_order_screen_only", "relative_or_absolute_bound": 1e-12,
                               "native_tolerance": "not_established"},
            "acceptance": "pending_original_artifact_logs_conditions_and_source_audit"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("synthetic", type=Path)
    parser.add_argument("shipped", type=Path)
    parser.add_argument("witness", type=Path)
    args = parser.parse_args()
    print(json.dumps(validate_witness(args.synthetic, args.shipped, args.witness), indent=2))


if __name__ == "__main__":
    main()
