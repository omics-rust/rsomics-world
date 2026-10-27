#!/usr/bin/env python3
"""Compare complete native raw discovery with actual PCA/simple Leiden evidence."""

import argparse
from collections import Counter
import csv
import json
import math
from pathlib import Path

from compare_infercnv_denoise_workflow import check, close, identities, matrix_error
from compare_infercnv_hmm_workflow import timed_native
from measure_infercnv_samples_cli import report_fingerprint
from validate_infercnv_shipped_oracle import COMMIT, SOURCE_SHA256, file_sha256, owned_file, read_canonical


def numeric_close(actual, expected, context):
    return close(actual, expected, 1e-6 + 1e-6 * abs(float(expected)), context)


def rectangular(values, rows, columns, context):
    check(len(values) == rows * columns and all(math.isfinite(value) for value in values), context)
    return [values[index * columns:(index + 1) * columns] for index in range(rows)]


def compare_pca(actual, expected):
    features = expected["feature_indices"]
    check(actual["feature_indices"] == features, "PCA ordered variable features differ")
    components = len(expected["standard_deviations"])
    check(actual["components"] == components and len(actual["standard_deviations"]) == components,
          "PCA component dimensions differ")
    check(len(actual["feature_scores"]) == len(features)
          and all(math.isfinite(value) and value >= 0 for value in actual["feature_scores"]),
          "invalid native variable-feature scores")
    scores = rectangular(actual["scores"], len(expected["scores"]), components, "PCA score dimensions differ")
    loadings = rectangular(actual["loadings"], len(features), components, "PCA loading dimensions differ")
    sd_error = max((numeric_close(left, right, "PCA standard deviation differs")
                    for left, right in zip(actual["standard_deviations"], expected["standard_deviations"])), default=0)
    distance_error = 0.0
    for index, row in enumerate(scores):
        for other in range(index):
            distance_error = max(distance_error, numeric_close(
                math.dist(row, scores[other]), math.dist(expected["scores"][index], expected["scores"][other]),
                "PCA pairwise score distance differs"))
    columns = list(zip(*loadings))
    source_columns = list(zip(*expected["loadings"]))
    cross_mass = 0.0
    for index, column in enumerate(columns):
        for other, right in enumerate(columns):
            numeric_close(math.fsum(left * value for left, value in zip(column, right)),
                          float(index == other), "PCA loadings are not orthonormal")
        cross_mass += math.fsum(math.fsum(left * right for left, right in zip(column, source)) ** 2
                                for source in source_columns)
    subspace_error = numeric_close(cross_mass, components, "PCA loading subspace differs")
    signed_error = 0.0
    for component in range(components):
        sign = 1 if math.fsum(left[component] * right[component]
                              for left, right in zip(scores, expected["scores"])) >= 0 else -1
        signed_error = max(signed_error, max((abs(sign * left[component] - right[component])
                                             for left, right in zip(scores, expected["scores"])), default=0))
    return dict(features=len(features), components=components, max_sd_scaled_error=sd_error,
                max_distance_scaled_error=distance_error, loading_subspace_scaled_error=subspace_error,
                max_sign_aligned_score_difference=signed_error)


def compare_graph(actual, expected, k):
    vertices = expected["vertices"]
    check(actual["vertices"] == vertices and len(actual["neighbors"]) == vertices,
          "graph vertex identity/dimensions differ")
    for left, right in zip(actual["neighbors"], expected["neighbors"]):
        check(len(left) == k and len(set(left)) == k and sorted(left) == sorted(right),
              "actual nearest-neighbor set differs")
    edges = [tuple(edge) for edge in actual["edges"]]
    check(edges == sorted(set(edges)) and all(type(left) is int and type(right) is int
          and 0 <= left <= right < vertices and math.isfinite(weight) and weight > 0
          for left, right, weight in edges), "native graph edges are not canonical finite undirected edges")
    check(edges == [tuple(edge) for edge in expected["edges"]], "actual graph edges, loops or weights differ")
    return dict(vertices=vertices, edges=len(edges), loops=sum(left == right for left, right, _ in edges))


def canonical_partition(labels):
    names = {}
    return [names.setdefault(label, len(names) + 1) for label in labels]


def adjusted_rand(left, right):
    choose2 = lambda count: count * (count - 1) / 2
    pairs = choose2(len(left))
    same = canonical_partition(left) == canonical_partition(right)
    if pairs == 0:
        return float(same)
    joint = sum(choose2(count) for count in Counter(zip(left, right)).values())
    first = sum(choose2(count) for count in Counter(left).values())
    second = sum(choose2(count) for count in Counter(right).values())
    expected = first * second / pairs
    denominator = (first + second) / 2 - expected
    return (joint - expected) / denominator if denominator else float(same)


def cpm_quality(labels, graph, resolution):
    return (2 * math.fsum(weight for left, right, weight in graph["edges"] if labels[left] == labels[right])
            - resolution * sum(count * count for count in Counter(labels).values()))


def compare_partition(actual, expected, graph, resolution):
    check(len(actual) == len(expected) == graph["vertices"]
          and all(type(label) is int and label > 0 for label in actual)
          and set(actual) == set(range(1, len(set(actual)) + 1)), "invalid complete native partition")
    adjacent = [set() for _ in actual]
    for left, right, _ in graph["edges"]:
        adjacent[left].add(right)
        adjacent[right].add(left)
    for label in set(actual):
        members = {index for index, value in enumerate(actual) if value == label}
        visited = set()
        pending = [next(iter(members))]
        while pending:
            vertex = pending.pop()
            if vertex not in visited:
                visited.add(vertex)
                pending.extend((adjacent[vertex] & members) - visited)
        check(visited == members, f"native community {label} is disconnected")
    native = cpm_quality(actual, graph, resolution)
    source = cpm_quality(expected, graph, resolution)
    check(math.isfinite(native) and math.isfinite(source) and native + 1e-9 >= source,
          f"native CPM quality is lower than actual source: {native:.17g} < {source:.17g}")
    return dict(native_clusters=len(set(actual)), source_clusters=len(set(expected)),
                native_cpm_quality=native, source_cpm_quality=source,
                same_partition=canonical_partition(actual) == canonical_partition(expected),
                adjusted_rand_index=adjusted_rand(actual, expected))


def tree_clades(tree, members, allow_missing=False):
    if tree is None:
        check(len(members) <= 1 or allow_missing, "missing native multi-cell Ward tree")
        return {}
    count = len(members)
    check(count >= 2 and len(set(members)) == count and len(tree["merge"]) == len(tree["height"]) == count - 1,
          "tree leaf/merge dimensions differ")
    nodes = []
    clades = {}
    for index, (merge, height) in enumerate(zip(tree["merge"], tree["height"])):
        check(len(merge) == 2 and math.isfinite(float(height)) and float(height) >= 0,
              "invalid tree merge/height")
        children = []
        for value in merge:
            check(type(value) is int and value != 0 and -count <= value <= index, "invalid tree merge reference")
            children.append(([members[-value - 1]], [-value - 1], 0.0) if value < 0 else nodes[value - 1])
        left, right = children
        check(not set(left[0]) & set(right[0]) and float(height) + 1e-12 >= max(left[2], right[2]),
              "tree reuses leaves or decreases toward its root")
        node = (left[0] + right[0], left[1] + right[1], float(height))
        nodes.append(node)
        clades[frozenset(node[0])] = float(height)
    check(set(nodes[-1][0]) == set(members) and nodes[-1][1] == tree["order"],
          "tree traversal does not identify its complete leaves")
    return clades


def named_matrix(root, record):
    with owned_file(root, record["tsv"]).open(newline="") as source:
        rows = csv.reader(source, delimiter="\t")
        header = next(rows)
        names, values = [], []
        for row in rows:
            check(len(row) == len(header), "source numeric table row width differs")
            names.append(row[0])
            values.append([float(value) for value in row[1:]])
    check([len(names), len(header) - 1] == record["dimensions"]
          and all(math.isfinite(value) for row in values for value in row), "source numeric table dimensions differ")
    return names, header[1:], values


def reference_group(root, group, genes, cells):
    members = [index - 1 for index in group["input_indices"]]
    names = [cells[index]["id"] for index in members]
    check(group["status"] == "success" and names == group["input_cells"], "source parent identities differ")
    source_tree = group["tree"]
    tree_members = members
    if source_tree is not None:
        by_cell = {cell["id"]: index for index, cell in enumerate(cells)}
        tree_members = [by_cell[name] for name in source_tree["labels"]]
        source_tree = dict(source_tree, order=[index - 1 for index in source_tree["order"]],
                           height=list(map(float, source_tree["height"])))
    result = dict(name=group["name"], member_indices=members, route=group["route"],
                  resolution=group["used_resolution"], tree=source_tree, tree_members=tree_members,
                  subclusters=[dict(name=row["name"], member_indices=[index - 1 for index in row["indices"]])
                               for row in group["partition"]["subclusters"]], pca=None, graph=None, partition=None)
    if group["graph"] is None:
        return result
    local = {cell: index for index, cell in enumerate(names)}
    source_members = group["partition"]["global_indices"]
    by_global = dict(zip((index - 1 for index in source_members), group["partition"]["membership"]))
    check(group["partition"]["matches_returned_subclusters"] is True, "source partition/member joins disagree")
    result["partition"] = [by_global[index] for index in members]
    with owned_file(root, group["graph"]["edges"]).open(newline="") as source:
        edges = []
        for row in csv.DictReader(source, delimiter="\t"):
            left, right = sorted((local[row["from_cell"]], local[row["to_cell"]]))
            edges.append((left, right, float(row.get("weight", 1))))
    if group["seurat"] is not None:
        seurat = group["seurat"]
        assay = next(assay for assay in seurat["assays"] if assay["name"] == seurat["default_assay"])
        reduction = next(reduction for reduction in seurat["reductions"] if reduction["name"] == "pca")
        score_names, _, scores = named_matrix(root, reduction["scores"])
        loading_names, _, loadings = named_matrix(root, reduction["loadings"])
        variable = owned_file(root, assay["variable_features"]).read_text().splitlines()
        loading_features = set(loading_names)
        check(loading_names == [name for name in variable if name in loading_features],
              "source variable-feature and loading order differs")
        by_gene = {gene["id"]: index for index, gene in enumerate(genes)}
        by_score = dict(zip(score_names, scores))
        result["pca"] = dict(feature_indices=[by_gene[name] for name in loading_names],
                             scores=[by_score[name] for name in names], loadings=loadings,
                             standard_deviations=list(map(float, reduction["standard_deviations"])))
        neighbor_record = next(graph["matrix"] for graph in seurat["graphs"]
                               if graph["name"] == seurat["default_assay"] + "_nn")
        row_names, column_names, matrix = named_matrix(root, neighbor_record)
        neighbor_rows = {name: [local[column_names[index]] for index, value in enumerate(row) if value != 0]
                         for name, row in zip(row_names, matrix)}
        neighbors = [neighbor_rows[name] for name in names]
    else:
        _, _, matrix = named_matrix(root, group["simple_knn"])
        neighbors = [[int(value) - 1 for value in row] for row in matrix]
    result["graph"] = dict(vertices=len(members), neighbors=neighbors, edges=sorted(edges))
    return result


def compare_group(actual, expected, k):
    check(actual["name"] == expected["name"] and actual["member_indices"] == expected["member_indices"],
          "parent cell identity/order differs")
    routes = {"fewer_than_three_cells": "too_few_cells"}
    check(actual["route"] == routes.get(expected["route"], expected["route"]), "actual discovery route differs")
    members = actual["member_indices"]
    bypass = expected["graph"] is None
    clades = tree_clades(actual["tree"], members, allow_missing=len(members) < 3)
    result = dict(parent=actual["name"], route=actual["route"])
    if bypass:
        check(actual["graph"] is None and actual["pca"] is None and actual["partition"] is None,
              "invented graph/PCA/partition for source bypass")
        check([(row["name"], row["member_indices"]) for row in actual["subclusters"]]
              == [(row["name"], row["member_indices"]) for row in expected["subclusters"]],
              "bypass subgroup names/membership differ")
        same = True
    else:
        resolution = float(expected["resolution"])
        close(actual["resolution"], resolution, 1e-12 + 1e-12 * abs(resolution), "automatic CPM resolution differs")
        result["graph"] = compare_graph(actual["graph"], expected["graph"], k)
        if expected["pca"] is None:
            check(actual["pca"] is None, "invented PCA for simple source route")
        else:
            check(actual["pca"] is not None, "actual PCA output is missing")
            result["pca"] = compare_pca(actual["pca"], expected["pca"])
        partition = compare_partition(actual["partition"], expected["partition"], actual["graph"], resolution)
        result.update(partition)
        same = partition["same_partition"]
        labels = actual["partition"]
        ordered = sorted(set(labels), key=lambda label: (-labels.count(label), label))
        subclusters = [dict(name=f"{actual['name']}_s{label}",
                            member_indices=[cell for cell, value in zip(members, labels) if value == label])
                       for label in ordered]
        check([(row["name"], row["member_indices"]) for row in actual["subclusters"]]
              == [(row["name"], row["member_indices"]) for row in subclusters],
              "native partition/subgroup identity, order or membership differs")
    for subgroup in actual["subclusters"]:
        selected = subgroup["member_indices"]
        check(len(selected) <= 1 or frozenset(selected) in clades,
              "stitched parent tree loses a complete native subcluster")
    if same:
        source_clades = tree_clades(expected["tree"], expected["tree_members"], allow_missing=len(members) < 3)
        check(clades.keys() == source_clades.keys(), "source/native parent tree clades differ")
        for clade, height in clades.items():
            oracle = source_clades[clade]
            close(height, oracle, 1e-12 + 1e-12 * abs(oracle), "source/native parent tree height differs")
    result["source_tree_compared"] = same
    return result


def compare_report(report, case, expression, genes, cells, references):
    check(report["schema"] == "rsomics-sc.cnv.samples.v1" and report["genes"] == genes
          and report["cells"] == cells, "native report schema or gene/cell identities differ")
    check(report["selected_gene_indices"] == [index - 1 for index in case["diagnostic"]["selected_gene_indices"]],
          "selected discovery genes differ")
    outliers = case["diagnostic"]["outliers"]
    check(report["outlier_gene_indices"] == (None if outliers is None else [index - 1 for index in outliers]),
          "reference gene outliers differ")
    expression_error = matrix_error(report["expression"], expression, 14)
    discovery = report["discovery"]
    method, k = case["run_parameters"]["leiden_method"], case["run_parameters"]["k_nn"]
    check((discovery["step"], discovery["method"], discovery["partition_method"], discovery["k_nn"],
           discovery["seed"], discovery["objective_function"], discovery["beta"], discovery["iterations"],
           discovery["restarts"])
          == (15, method, "leiden", k, 0, "CPM", .01, 2, 8), "native discovery model/parameters differ")
    check(len(discovery["groups"]) == len(references), "native parent count differs")
    parents = [compare_group(actual, expected, k) for actual, expected in zip(discovery["groups"], references)]
    expected_flat = [(parent["name"], subgroup) for parent in discovery["groups"] for subgroup in parent["subclusters"]]
    check(len(report["groups"]) == len(expected_flat), "flat subgroup count differs")
    for flat, (parent, subgroup) in zip(report["groups"], expected_flat):
        check(flat["name"] == parent and flat["subcluster_name"] == subgroup["name"]
              and flat["member_indices"] == subgroup["member_indices"], "flat native subgroup join differs")
        tree_clades(flat["tree"], flat["member_indices"],
                    allow_missing=len(flat["member_indices"]) < 3 and
                    any(group["name"] == parent and group["route"] == "too_few_cells" for group in discovery["groups"]))
    check(sorted(cell for flat in report["groups"] for cell in flat["member_indices"]) == list(range(len(cells))),
          "native subgroups do not cover every cell exactly once")
    check(all(name not in report for name in ("hmm", "bayesian", "denoised")), "unrequested workflow output appeared")
    return dict(genes=len(genes), cells=len(cells), parents=parents, subclusters=len(expected_flat),
                max_expression_scaled_error=expression_error)


def native_command(binary, shipped, case, threads, output):
    parameters = case["run_parameters"]
    command = [str(binary), "cnv", "samples", "--cluster-by-groups", "--subcluster-method",
               parameters["leiden_method"].lower(), "--subcluster-knn", str(parameters["k_nn"]),
               "--subcluster-seed", "0", "--include-expression", "--threads", str(threads), "--output", str(output)]
    flags = dict(counts="counts", gene_order="positions", annotations="annotations")
    for entry in case["inputs"]["raw_inputs"]:
        command += ["--" + flags[entry["role"]], str(owned_file(shipped, entry["source_path"]))]
    for reference in case["inputs"]["references"]:
        command += ["--reference", reference]
    for name in ("cutoff", "min_cells_per_gene", "window_length", "max_centered_threshold", "z_score_filter"):
        command += ["--" + name.replace("_", "-"), str(parameters[name])]
    if not parameters["ref_subtract_use_mean_bounds"]:
        command += ["--reference-mean"]
    return command


def run_comparison(shipped, oracle, binary, output):
    shipped, oracle, binary = [Path(path).resolve(strict=True) for path in (shipped, oracle, binary)]
    output = Path(output).absolute()
    output.mkdir(parents=True, exist_ok=False)
    record = json.loads((oracle / "witness.json").read_text())
    check(record["schema_version"] == 1 and record["status"] == "complete" and record["index_base"] == 1
          and list(record["cases"]) == ["PCA", "simple"] and not record["failed_cases"], "incomplete actual discovery evidence")
    check(record["package"] == dict(name="infercnv", version="1.28.0", source_commit=COMMIT,
                                    source_archive_sha256=SOURCE_SHA256), "actual discovery source identity differs")
    check(file_sha256(shipped / "sha256-manifest.tsv") == record["source_bundles"]["shipped_manifest"],
          "shipped raw-input bundle differs from actual discovery")
    for name, digest in record["files"].items():
        check(file_sha256(owned_file(oracle, name)) == digest, f"actual discovery export changed: {name}")
    binary_hash = file_sha256(binary)
    results = []
    for name, case in record["cases"].items():
        check(case["status"] == "success" and case["original_expression_unchanged"] is True
              and not case["partition_mismatch_groups"], f"failed or inconsistent actual profile: {name}")
        genes, cells = identities(oracle, case["step14_state"])
        expression = read_canonical(owned_file(oracle, case["step14_state"]["expression"]), len(cells))
        references = [reference_group(oracle, group, genes, cells) for group in case["groups"]]
        baseline = None
        for threads in (1, 4):
            prefix = output / f"{name}-threads-{threads}"
            report_path = prefix.with_suffix(".report.json")
            metrics = timed_native(native_command(binary, shipped, case, threads, report_path), prefix)
            report = json.loads(report_path.read_text())
            result = compare_report(report, case, expression, genes, cells, references)
            fingerprint = report_fingerprint(report, threads)
            check(baseline is None or fingerprint == baseline, "native discovery reports differ across worker counts")
            baseline = fingerprint
            results.append(dict(case=name, threads=threads, **result, **metrics,
                                report_fingerprint=fingerprint, report_sha256=file_sha256(report_path)))
            print(f"matched actual discovery model/graph: {name}, workers={threads}, subclusters={result['subclusters']}", flush=True)
    check(file_sha256(binary) == binary_hash, "native binary changed during discovery comparison")
    summary = dict(status="complete", native_processes=len(results), binary_sha256=binary_hash,
                   witness_sha256=file_sha256(oracle / "witness.json"), results=results,
                   tolerance=dict(pca_absolute=1e-6, pca_relative=1e-6, cpm_quality_absolute=1e-9),
                   scope="raw discovery graph/model/quality compatibility; not exact R RNG, biological equivalence or paired performance")
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("shipped", "oracle", "binary", "output"):
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    try:
        run_comparison(args.shipped, args.oracle, args.binary, args.output)
    except (OSError, ValueError, RuntimeError, KeyError, StopIteration) as error:
        parser.exit(1, f"discovery workflow comparison failed: {error}\n")


if __name__ == "__main__":
    main()
