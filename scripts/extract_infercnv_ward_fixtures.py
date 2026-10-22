"""Extract exact Ward tree fixtures from the accepted installed-package witness."""

import argparse
import hashlib
import json
import math
from pathlib import Path
import re

from validate_infercnv_samples_clustering_witness import checked_file, table


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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--witness", type=Path, required=True)
    outputs = parser.add_mutually_exclusive_group(required=True)
    outputs.add_argument("--output", type=Path)
    outputs.add_argument("--profiles-output", type=Path)
    args = parser.parse_args()
    try:
        extractor = extract_profiles if args.profiles_output else extract
        print(json.dumps(extractor(args.witness, args.profiles_output or args.output), sort_keys=True))
    except (OSError, ValueError) as exc:
        parser.exit(1, f"Ward fixture extraction failed: {exc}\n")


if __name__ == "__main__":
    main()
