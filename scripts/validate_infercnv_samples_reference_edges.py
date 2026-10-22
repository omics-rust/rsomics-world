"""Validate retained reference-only observations, not their acceptance."""

import argparse
import csv
import hashlib
import json
from pathlib import Path, PurePosixPath
import re


PACKAGE = {"name": "infercnv", "version": "1.28.0",
           "source_commit": "b421d9405c97a309b081ef86d455e976df93eae4",
           "source_archive_sha256": "b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207"}
PACKAGES = ("infercnv", "jsonlite", "digest", "fastcluster", "parallelDist")
SOURCE_PINS = {"source_archive": PACKAGE["source_archive_sha256"],
               "source_files": ["68ceb769fc96f12d16ef73f7c0c8beac7e1e32c21bd057355ce0c23bbdf6f9fb"],
               "functions": ["01e4c43a8d8f6b39f8b7f4a0e58c0415b87a74713f9ad3c2f45931817d3b734c",
                             "f795c8debc72b0d7fe9c3c181607dc14341447a34087d0f6319c1c8d282659ee"],
               "io": "5a01aeb620451d0254bbab775c53111a61eeefcaa32ac9b96ab142563ca5839d"}
ARGUMENTS = {"p_val": 0.1, "k_nn": 20, "leiden_method": "PCA", "leiden_function": "CPM",
             "leiden_resolution": "auto", "leiden_method_per_chr": "simple",
             "leiden_function_per_chr": "modularity", "leiden_resolution_per_chr": 1,
             "hclust_method": "ward.D2", "partition_method": "none",
             "per_chr_hmm_subclusters": False, "per_chr_hmm_subclusters_references": False,
             "z_score_filter": 0, "restrict_to_DE_genes": False}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def keys(value: object, expected: set[str], label: str) -> None:
    require(isinstance(value, dict) and set(value) == expected, f"invalid {label} fields")


def strings(value: object) -> bool:
    return isinstance(value, list) and all(isinstance(item, str) for item in value)


def checked_file(root: Path, relative: str) -> Path:
    require(isinstance(relative, str) and bool(relative) and "\\" not in relative,
            "invalid relative path")
    path = PurePosixPath(relative)
    require(not path.is_absolute() and str(path) == relative and
            all(part not in (".", "..") for part in path.parts), "unsafe relative path")
    current = root
    for part in path.parts:
        current = current / part
        require(not current.is_symlink(), "symlink in evidence path")
    require(current.is_file(), f"missing regular file: {relative}")
    return current


def digest(path: Path) -> str:
    result = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            result.update(chunk)
    return result.hexdigest()


def validate_shape(value: object) -> None:
    require(isinstance(value, dict), "missing typed result shape")
    kind = value.get("typeof")
    base = {"typeof", "classes", "length", "names"}
    require(kind in ("NULL", "list", "integer", "double", "logical", "character", "raw",
                     "complex", "language", "symbol", "expression", "pairlist"), "unknown R shape type")
    field = "items" if kind in ("list", "expression", "pairlist") else (
        "text" if kind in ("language", "symbol") else "values")
    keys(value, base if kind == "NULL" else base | {field}, "shape")
    require(strings(value["classes"]) and bool(value["classes"]), "invalid shape classes")
    length = value["length"]
    require(type(length) is int and length >= 0, "invalid shape length")
    names = value["names"]
    require(names is None or (isinstance(names, list) and len(names) == length and
            all(name is None or isinstance(name, str) for name in names)), "invalid shape names")
    if kind == "NULL":
        require(length == 0 and names is None and value["classes"] == ["NULL"], "invalid NULL shape")
    elif field == "items":
        require(isinstance(value[field], list) and len(value[field]) == length, "invalid ordered shape items")
        for item in value[field]:
            validate_shape(item)
    elif field == "text":
        require(isinstance(value[field], str) and bool(value[field]), "missing language text")
    else:
        require(strings(value[field]) and len(value[field]) == length, "invalid typed literal values")
        patterns = {"integer": r"(?:NA|-?\d+)", "double": r"(?:NA|NaN|-?Inf|-?(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?)",
                    "logical": r"(?:NA|TRUE|FALSE)", "raw": r"[0-9a-f]{2}"}
        if kind in patterns:
            require(all(re.fullmatch(patterns[kind], item) for item in value[field]), "invalid R literal")


def validate_input(root: Path, value: dict, bound) -> None:
    keys(value, {"classes", "genes", "cells", "reference_groups", "observation_groups", "rds", "state"}, "input")
    require(value["classes"] == ["infercnv"] and type(value["genes"]) is int and value["genes"] == 4 and
            type(value["cells"]) is int and value["cells"] == 3 and value["reference_groups"] == ["ref"] and
            value["observation_groups"] == [], "reference-only input identity differs")
    bound(value["rds"])
    keys(value["state"], {"expression", "genes", "cells", "maps"}, "input state")
    tables = {}
    for name, relative in value["state"].items():
        with bound(relative).open(newline="") as stream:
            tables[name] = list(csv.reader(stream, delimiter="\t"))
    expected = {
        "genes": [["gene", "chr", "start", "stop"], *[
            [f"gene{i}", "chrProbe", str(i * 100), str(i * 100 + 49)] for i in range(1, 5)]],
        "cells": [["cell", "group", "role"], *[[f"cell{i}", "ref", "reference"] for i in range(1, 4)]],
        "maps": [["role", "group", "index", "cell"], *[
            ["reference", "ref", str(i), f"cell{i}"] for i in range(1, 4)]]}
    require(all(tables[name] == rows for name, rows in expected.items()), "input identities or maps differ")
    expression = tables["expression"]
    require(len(expression) == 5 and expression[0] == ["gene", "cell1", "cell2", "cell3"],
            "input expression dimensions differ")
    for i, row in enumerate(expression[1:], 1):
        require(len(row) == 4 and row[0] == f"gene{i}" and
                [float(v) for v in row[1:]] == [1 + (i + j * 4) / 8 for j in range(3)],
                "input expression values differ")


def validate(root: Path) -> dict:
    root = Path(root)
    require(root.is_dir() and not root.is_symlink(), "invalid evidence directory")
    record = json.loads(checked_file(root, "witness.json").read_text())
    keys(record, {"schema_version", "package", "runtime", "input", "arguments", "routes", "files"}, "witness")
    require(type(record["schema_version"]) is int and record["schema_version"] == 1 and
            record["package"] == PACKAGE, "package/source/schema pin mismatch")
    entries = record["files"]
    require(isinstance(entries, list) and bool(entries), "missing files inventory")
    hashes = {}
    for entry in entries:
        keys(entry, {"path", "sha256"}, "file inventory")
        relative, expected = entry["path"], entry["sha256"]
        path = checked_file(root, relative)
        require(relative != "witness.json" and relative not in hashes and isinstance(expected, str) and
                re.fullmatch(r"[0-9a-f]{64}", expected) is not None, "invalid file inventory")
        require(digest(path) == expected, f"file SHA-256 mismatch: {relative}")
        hashes[relative] = expected
    require(list(hashes) == sorted(hashes), "unsorted file inventory")
    actual = []
    for path in root.rglob("*"):
        require(not path.is_symlink(), "symlink in evidence directory")
        if not path.is_dir():
            require(path.is_file(), "nonregular evidence entry")
            actual.append(path.relative_to(root).as_posix())
    require(set(actual) == {"witness.json", *hashes}, "incomplete files inventory")

    def bound(relative):
        require(isinstance(relative, str) and relative in hashes, "unbound evidence file")
        return checked_file(root, relative)

    runtime = record["runtime"]
    keys(runtime, {"R_version", "package_versions", "loaded_paths", "source_root", "source_archive", "source_files",
                   "functions", "bindings", "exporter", "io", "arguments_file", "locale", "seed", "threads"}, "runtime")
    require(runtime["R_version"] == "4.6.1" and runtime["locale"] == "C" and
            type(runtime["seed"]) is int and runtime["seed"] == 260930 and
            type(runtime["threads"]) is int and runtime["threads"] == 1, "runtime pin mismatch")
    keys(runtime["package_versions"], set(PACKAGES), "package versions")
    keys(runtime["loaded_paths"], set(PACKAGES), "loaded paths")
    require(runtime["package_versions"]["infercnv"] == "1.28.0" and
            all(isinstance(v, str) and bool(v) for v in runtime["package_versions"].values()), "invalid package versions")
    require(all(isinstance(p, str) and p.startswith("/") for p in
                [runtime["source_root"], *runtime["loaded_paths"].values()]), "invalid runtime paths")
    require(runtime["bindings"] == {"hclust": "fastcluster::hclust", "parallelDist": "parallelDist::parallelDist"},
            "namespace binding identity differs")
    require(all(runtime[key] == path for key, path in {
        "source_archive": "source.tar.gz", "source_files": ["source.R"],
        "functions": ["function-main.txt", "function-single.txt"], "exporter": "exporter.R",
        "io": "io.R", "arguments_file": "arguments.json"}.items()), "runtime file bindings differ")
    require(digest(bound(runtime["source_archive"])) == SOURCE_PINS["source_archive"] and
            strings(runtime["source_files"]) and len(runtime["source_files"]) == 1 and
            [digest(bound(p)) for p in runtime["source_files"]] == SOURCE_PINS["source_files"] and
            digest(bound(runtime["io"])) == SOURCE_PINS["io"], "source bytes differ")
    require(strings(runtime["functions"]) and len(runtime["functions"]) == 2 and
            len(set(runtime["functions"])) == 2, "missing installed function bytes")
    for relative in [*runtime["functions"], runtime["exporter"]]:
        require(bound(relative).stat().st_size > 0, "empty runtime binding")
    require([digest(bound(p)) for p in runtime["functions"]] == SOURCE_PINS["functions"],
            "installed function bytes differ")
    for arguments in (record["arguments"], json.loads(bound(runtime["arguments_file"]).read_text())):
        keys(arguments, set(ARGUMENTS), "arguments")
        require(all(type(arguments[key]) is type(value) and arguments[key] == value
                    for key, value in ARGUMENTS.items()), "clustering arguments differ")
    validate_input(root, record["input"], bound)
    routes = record["routes"]
    require(isinstance(routes, list) and len(routes) == 2, "missing required routes")
    for route, identity in zip(routes, ("grouped", "pooled")):
        keys(route, {"id", "status", "warnings", "error", "log", "raw_rds", "result_shape", "unchanged_state"}, "route")
        require(route["id"] == identity and strings(route["warnings"]), "route identity or warnings differ")
        bound(route["log"])
        require(bound(route["raw_rds"]).stat().st_size > 0, "missing raw RDS")
        if route["status"] == "success":
            require(route["error"] is None and route["unchanged_state"] is True, "successful route changed input state")
            keys(route["result_shape"], {"tumor_subclusters", "second"}, "result shape")
            for value in route["result_shape"].values():
                validate_shape(value)
        else:
            require(route["status"] == "error" and route["result_shape"] is None and
                    route["unchanged_state"] is None, "invalid failed route")
            error = route["error"]
            keys(error, {"classes", "message", "call"}, "error")
            require(strings(error["classes"]) and "error" in error["classes"] and
                    isinstance(error["message"], str) and bool(error["message"]) and
                    (error["call"] is None or isinstance(error["call"], str)), "invalid captured condition")
    return record


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    try:
        record = validate(args.root)
    except (ValueError, OSError, TypeError, KeyError) as error:
        parser.exit(1, f"reference-only evidence invalid: {error}\n")
    print(json.dumps({"schema_version": 1, "routes": [
        {"id": route["id"], "status": route["status"]} for route in record["routes"]]}))


if __name__ == "__main__":
    main()
