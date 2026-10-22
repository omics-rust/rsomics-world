"""Extract exact samples clustering fixtures from the accepted installed-package witness."""

import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import re
import struct

from validate_infercnv_samples_clustering_witness import checked_file, finite, table, validate_maps


WITNESS_SHA256 = "26333ff388b0916d79736c7b3370b9f907fb943ca918a92ef5529a4d8bdfd304"
EXPECTED_COUNTS = {"cases": 36, "routes": 52, "groups": 160,
                   "trees": 152, "pairs": 107331, "max_leaves": 142}
SCALAR = re.compile(r"[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?\Z")


def _integer(value):
    return type(value) is int


def _scalars(values):
    if not isinstance(values, list) or any(
        not isinstance(v, str) or not SCALAR.fullmatch(v)
        or not math.isfinite(float(v)) or float(v) < 0 for v in values
    ):
        raise ValueError("invalid literal finite nonnegative scalars")


def _tree_record(identity, group):
    indices = group["input_indices"]
    if not isinstance(indices, list) or any(not _integer(i) or i < 1 for i in indices) \
            or len(set(indices)) != len(indices):
        raise ValueError("invalid input indices")
    n = len(indices)
    present = group["tree_present"]
    entry = group["tree_entry_present"]
    if type(present) is not bool or type(entry) is not bool:
        raise ValueError("invalid tree flags")
    if n <= 2:
        if present or entry or group["tree"] is not None or group["distances"] != []:
            raise ValueError("small group unexpectedly contains a tree")
        return None
    if not present or not entry:
        raise ValueError("missing large-group tree")
    tree = group["tree"]
    distances, height = group["distances"], tree["height"]
    _scalars(distances)
    _scalars(height)
    merge, order = tree["merge"], tree["order"]
    if len(distances) != n * (n - 1) // 2 or len(height) != n - 1 \
            or not isinstance(merge, list) or len(merge) != n - 1 \
            or tree["method"] != "ward.D2" or not isinstance(tree["labels"], list) \
            or len(tree["labels"]) != n or not all(isinstance(v, str) for v in tree["labels"]):
        raise ValueError("invalid tree shape or method")
    if any(float(a) > float(b) for a, b in zip(height, height[1:])):
        raise ValueError("unordered heights")
    for row, children in enumerate(merge, 1):
        if not isinstance(children, list) or len(children) != 2 or any(
            not _integer(c) or not (-n <= c <= -1 or 1 <= c < row) for c in children
        ):
            raise ValueError("invalid R merge topology")
    stack, leaves = [n - 1], []
    while stack:
        node = stack.pop()
        if node < 0:
            leaves.append(-node)
        else:
            left, right = merge[node - 1]
            stack.extend((right, left))
        if len(leaves) > n:
            raise ValueError("repeated tree leaves")
    if not isinstance(order, list) or any(not _integer(i) for i in order) \
            or leaves != order or sorted(leaves) != list(range(1, n + 1)):
        raise ValueError("wrong tree leaf permutation or orientation")
    return identity, n, distances, merge, height, order


def _records(document):
    try:
        cases, records = document["cases"], []
        if not isinstance(cases, dict) or not cases:
            raise ValueError("missing cases")
        for name in sorted(cases):
            if not isinstance(name, str) or not re.fullmatch(r"[a-z0-9_]+", name):
                raise ValueError("invalid case identity")
            case = cases[name]
            routes = ("isolated", "full_run") if case["kind"] == "workflow" else ("isolated",)
            if case["kind"] not in ("probe", "workflow") \
                    or {r for r in ("isolated", "full_run") if r in case} != set(routes):
                raise ValueError("missing or unexpected routes")
            for route in routes:
                result = case[route]
                if result["status"] != "success" or not isinstance(result["groups"], list):
                    raise ValueError("unsuccessful or malformed route")
                for index, group in enumerate(result["groups"]):
                    record = _tree_record(f"{name}/{route}/{index}", group)
                    if record is not None:
                        records.append(record)
        return records
    except (KeyError, TypeError, IndexError) as exc:
        raise ValueError("malformed witness") from exc


def _accepted(witness_path):
    raw = witness_path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != WITNESS_SHA256:
        raise ValueError("witness hash differs from accepted original")
    document = json.loads(raw)
    records = _records(document)
    routes = [c[r] for c in document["cases"].values()
              for r in ("isolated", "full_run") if r in c]
    counts = {"cases": len(document["cases"]), "routes": len(routes),
              "groups": sum(len(r["groups"]) for r in routes), "trees": len(records),
              "pairs": sum(len(r[2]) for r in records),
              "max_leaves": max((r[1] for r in records), default=0)}
    if counts != EXPECTED_COUNTS:
        raise ValueError("witness counts differ from accepted original")
    return document, records, counts


def extract(witness_path: Path, output_path: Path) -> dict:
    _, records, counts = _accepted(witness_path)
    lines = []
    for name, n, distances, merge, height, order in records:
        lines.extend((f"tree\t{name}\t{n}", "distances\t" + "\t".join(distances),
                      "merge\t" + "\t".join(f"{a},{b}" for a, b in merge),
                      "heights\t" + "\t".join(height),
                      "order\t" + "\t".join(map(str, order)), "end"))
    data = ("\n".join(lines) + "\n").encode("utf-8")
    with output_path.open("xb") as target:
        target.write(data)
    return {"counts": counts, "witness_sha256": WITNESS_SHA256,
            "fixture_sha256": hashlib.sha256(data).hexdigest(), "fixture_bytes": len(data)}


def extract_profiles(witness_path: Path, output_path: Path) -> dict:
    document, records, counts = _accepted(witness_path)
    root, verified, routes = witness_path.parent, {}, {}

    def source(relative):
        path = checked_file(root, relative)
        if relative not in verified:
            expected = document["files"].get(relative)
            with path.open("rb") as handle:
                actual = hashlib.file_digest(handle, "sha256").hexdigest()
            if actual != expected:
                raise ValueError("source file hash differs: " + relative)
            verified[relative] = path
        return verified[relative]

    lines = ["tree\tmatrix\tfeatures\tcells\tindices"]
    try:
        for identity, *_ in records:
            name, route_name, index = identity.split("/")
            route = document["cases"][name][route_name]
            route_id = (name, route_name)
            if route_id not in routes:
                cells = table(source(route["state"]["cells"]), ("cell", "group", "role"))
                if cells[0] != ["cell", "group", "role"]:
                    raise ValueError("invalid state cell columns")
                names = [row[0] for row in cells[1:]]
                if not names or len(set(names)) != len(names) or any(not n for n in names):
                    raise ValueError("invalid state cell identities")
                genes = table(source(route["state"]["genes"]), ("gene", "chr", "start", "stop"))
                if genes[0] != ["gene", "chr", "start", "stop"]:
                    raise ValueError("invalid state gene columns")
                selected = route["diagnostic"]["selected_gene_indices"]
                if not isinstance(selected, list) or any(
                    not _integer(i) or i < 1 or i >= len(genes) for i in selected
                ) or selected != sorted(set(selected)):
                    raise ValueError("invalid selected gene indices")
                matrix = route["diagnostic"]["filtered_expression"]
                with source(matrix).open(encoding="utf-8", newline="") as handle:
                    header = handle.readline()
                if header != "gene\t" + "\t".join(names) + "\n":
                    raise ValueError("matrix header differs from state cell order")
                routes[route_id] = matrix, len(selected), names
            matrix, features, names = routes[route_id]
            group = route["groups"][int(index)]
            indices = group["input_indices"]
            if any(i > len(names) for i in indices) or group["tree"]["labels"] != [
                names[i - 1] for i in indices
            ]:
                raise ValueError("actual input indices differ from state cells")
            lines.append(f"{identity}\t{matrix}\t{features}\t{len(names)}\t"
                         + ",".join(str(i - 1) for i in indices))
    except (KeyError, TypeError, IndexError) as exc:
        raise ValueError("malformed profile metadata") from exc
    data = ("\n".join(lines) + "\n").encode("utf-8")
    with output_path.open("xb") as target:
        target.write(data)
    return {"counts": counts, "witness_sha256": WITNESS_SHA256,
            "fixture_sha256": hashlib.sha256(data).hexdigest(), "fixture_bytes": len(data)}


def _filter_state(paths, source):
    if not isinstance(paths, dict) or set(paths) != {"expression", "genes", "cells", "maps"}:
        raise ValueError("invalid filter input state paths")
    return {key: source(relative) for key, relative in paths.items()}


def _filter_indices(values, features):
    if not isinstance(values, list) or any(
        not _integer(i) or not 1 <= i <= features for i in values
    ) or values != sorted(set(values)):
        raise ValueError("invalid literal filter indices")
    return values


def _filter_matrix(source_path, filtered_path, genes, cells, selected):
    header = ["gene", *cells]
    with source_path.open(encoding="utf-8", newline="") as original, \
            filtered_path.open(encoding="utf-8", newline="") as filtered:
        rows, selection = csv.reader(original, delimiter="\t"), csv.reader(filtered, delimiter="\t")
        if next(rows, None) != header or next(selection, None) != header:
            raise ValueError("filter matrix header differs from state cell order")
        pending = iter(selected)
        selected_index = next(pending, None)
        count = 0
        for count, row in enumerate(rows, 1):
            if count > len(genes) or len(row) != len(header) or row[0] != genes[count - 1]:
                raise ValueError("source matrix differs from state gene dimensions or order")
            values = [struct.pack("!d", finite(value, "source expression")) for value in row[1:]]
            if count == selected_index:
                actual = next(selection, None)
                if actual is None or len(actual) != len(header) or actual[0] != row[0] or \
                        [struct.pack("!d", finite(value, "filtered expression"))
                         for value in actual[1:]] != values:
                    raise ValueError("filtered matrix differs from selected source row bits")
                selected_index = next(pending, None)
        if count != len(genes) or selected_index is not None or next(selection, None) is not None:
            raise ValueError("source or filtered matrix row count differs")


def extract_filter_routes(witness_path: Path, output_path: Path) -> dict:
    document, _, counts = _accepted(witness_path)
    root, verified = witness_path.parent, {}

    def source(relative):
        if not isinstance(relative, str) or any(c in relative for c in "\t\r\n"):
            raise ValueError("invalid filter evidence path")
        path = checked_file(root, relative)
        if relative not in verified:
            expected = document["files"].get(relative)
            with path.open("rb") as handle:
                actual = hashlib.file_digest(handle, "sha256").hexdigest()
            if actual != expected:
                raise ValueError("source file hash differs: " + relative)
            verified[relative] = path
        return verified[relative]

    def indices_text(values):
        return ",".join(str(i - 1) for i in values) if values else "[]"

    lines = ["route\texpression\tgenes\tcells\tmaps\tfiltered_expression\tfeatures\tcell_count"
             "\treference_indices\tz_score_filter\tfilter_guard_active\toutliers"
             "\tselected_gene_indices"]
    route_count = 0
    try:
        for name, case in sorted(document["cases"].items()):
            routes = ("isolated", "full_run") if case["kind"] == "workflow" else ("isolated",)
            explicit = case["full_run"]["stage14_state"] if case["kind"] == "workflow" else (
                case["inputs"]["pre_call_state"])
            _filter_state(explicit, source)
            for route_name in routes:
                route = case[route_name]
                state = route["state"]
                paths = _filter_state(state, source)
                if any(document["files"][state[key]] != document["files"][explicit[key]]
                       for key in explicit):
                    raise ValueError("route state hashes differ from explicit pre-clustering input")
                inputs = state if case["kind"] == "workflow" and route_name == "isolated" else explicit
                cells = table(paths["cells"], ("cell", "group", "role"))
                genes = table(paths["genes"], ("gene", "chr", "start", "stop"))
                maps = table(paths["maps"], ("role", "group", "index", "cell"))
                if cells[0] != ["cell", "group", "role"] or \
                        genes[0] != ["gene", "chr", "start", "stop"] or \
                        maps[0] != ["role", "group", "index", "cell"]:
                    raise ValueError("invalid filter state columns")
                cell_names, gene_names = [row[0] for row in cells[1:]], [row[0] for row in genes[1:]]
                if any(not x for x in cell_names + gene_names) or len(set(gene_names)) != len(gene_names):
                    raise ValueError("invalid filter gene/cell identities")
                if any(not row[1] or not all(re.fullmatch(r"-?[0-9]+", value) for value in row[2:])
                       or int(row[2]) > int(row[3]) for row in genes[1:]):
                    raise ValueError("invalid filter gene coordinates")
                ordered = validate_maps(cells[1:], maps[1:])
                if case["inputs"]["reference_groups"] != list(ordered["reference"]):
                    raise ValueError("reference group order differs from original maps")
                references = [i for group in ordered["reference"].values() for i in group]
                zscore = case["arguments"][route_name]["z_score_filter"]
                if isinstance(zscore, bool) or not isinstance(zscore, (int, float)) or not math.isfinite(zscore):
                    raise ValueError("invalid filter argument")
                diagnostic = route["diagnostic"]
                active = zscore > 0 and bool(references)
                if diagnostic["filter_guard_active"] is not active:
                    raise ValueError("filter guard differs from source")
                selected = _filter_indices(diagnostic["selected_gene_indices"], len(gene_names))
                outliers = diagnostic["outliers"]
                if active:
                    _filter_indices(outliers, len(gene_names))
                    excluded = set(outliers)
                    expected = [i for i in range(1, len(gene_names) + 1) if i not in excluded] if outliers else []
                    if selected != expected:
                        raise ValueError("filter selection differs from literal negative indices")
                elif outliers is not None or selected != list(range(1, len(gene_names) + 1)):
                    raise ValueError("inactive filter selection differs")
                matrix = diagnostic["filtered_expression"]
                _filter_matrix(paths["expression"], source(matrix), gene_names, cell_names, selected)
                fields = [f"{name}/{route_name}", *(inputs[key] for key in
                          ("expression", "genes", "cells", "maps")), matrix, str(len(gene_names)),
                          str(len(cell_names)), indices_text(references), str(zscore),
                          "true" if active else "false", "NULL" if outliers is None else indices_text(outliers),
                          indices_text(selected)]
                lines.append("\t".join(fields))
                route_count += 1
    except (KeyError, TypeError, IndexError, OverflowError) as exc:
        raise ValueError("malformed filter route metadata") from exc
    if route_count != counts["routes"]:
        raise ValueError("incomplete filter route coverage")
    data = ("\n".join(lines) + "\n").encode("utf-8")
    with output_path.open("xb") as target:
        target.write(data)
    return {"counts": counts, "witness_sha256": WITNESS_SHA256,
            "fixture_sha256": hashlib.sha256(data).hexdigest(), "fixture_bytes": len(data)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--witness", type=Path, required=True)
    outputs = parser.add_mutually_exclusive_group(required=True)
    outputs.add_argument("--output", type=Path)
    outputs.add_argument("--profiles-output", type=Path)
    outputs.add_argument("--filter-output", type=Path)
    args = parser.parse_args()
    try:
        if args.filter_output:
            extractor, output = extract_filter_routes, args.filter_output
        elif args.profiles_output:
            extractor, output = extract_profiles, args.profiles_output
        else:
            extractor, output = extract, args.output
        print(json.dumps(extractor(args.witness, output), sort_keys=True))
    except (OSError, ValueError) as exc:
        parser.exit(1, f"Ward fixture extraction failed: {exc}\n")


if __name__ == "__main__":
    main()
