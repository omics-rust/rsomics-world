"""Independently check creation edge exports and their retained inputs."""

import argparse
import json
from pathlib import Path, PurePosixPath

from validate_infercnv_ingestion_witness import (
    OUTPUTS, SOURCE_COMMIT, SOURCE_SHA256, checked_file, digest, table,
)
from validate_infercnv_shipped_oracle import ORIGINAL_SHA256, REFS


COUNTS = ("gene\tlow\tmin\tmax\thigh\tzero\tunit\textra\n"
          "B\t1\t1\t2\t2\t0\t1\t1\n"
          "A\t1\t1\t2\t2\t0\t0\t1\n"
          "C\t0\t1\t1\t2\t0\t0\t1\n"
          "D\t0\t0\t0\t0\t0\t0\t1\n"
          "X\t1000000\t1000000\t1000000\t1000000\t1000000\t1000000\t1000000\n"
          "ONLY_COUNTS\t1000000\t1000000\t1000000\t1000000\t1000000\t1000000\t1000000\n"
          "ZERO_POS\t1000000\t1000000\t1000000\t1000000\t1000000\t1000000\t1000000\n")
POSITIONS = ("POS_ONLY\tchr10\t1\t2\nA\tchr2\t10\t20\n"
             "B\tchr2\t10\t20\nC\tchr10\t30\t40\nD\tchr1\t1\t2\n"
             "X\tchrX\t10\t20\nZERO_POS\tchr3\t0\t0\n")
ANNOTATIONS = ("high\tz_obs\nmax\ta_obs\nmin\tref\nlow\ta_obs\n"
               "zero\tz_obs\nunit\tz_obs\n")
LIMITS = {"integer_limits": [3, 5], "min_zero": [0, "Inf"], "min_null": None}
ERRORS = ("absent_annotation", "removed_reference", "no_common_genes")


def validate_arguments(record, references, limits):
    args = record.get("creation_arguments")
    paths = {"raw_counts_matrix", "gene_order_file", "annotations_file"}
    policies = {"ref_group_names", "delim", "max_cells_per_group",
                "min_max_counts_per_cell", "chr_exclude"}
    if not isinstance(args, dict) or set(args) != paths | policies:
        raise ValueError("creation arguments differ")
    policy = {key: args.get(key) for key in (
        "ref_group_names", "delim", "max_cells_per_group",
        "min_max_counts_per_cell", "chr_exclude")}
    if policy != {"ref_group_names": references, "delim": "\t",
                  "max_cells_per_group": None, "min_max_counts_per_cell": limits,
                  "chr_exclude": ["chrX", "chrY", "chrM"]}:
        raise ValueError("creation arguments differ")
    if not isinstance(record.get("warnings"), list) or any(
            not isinstance(message, str) or not message.strip() for message in record["warnings"]):
        raise ValueError("invalid warning record")


def validate_tiny(directory, label, record):
    validate_arguments(record, ["ref"], LIMITS[label])
    if label == "integer_limits":
        cells = ["min", "max"]
        numeric = [[1, 1], [1, 2], [1, 2], [0, 0]]
        cell_rows = [["min", "ref", "reference"], ["max", "a_obs", "observation"]]
        maps = [["reference", "ref", "1", "min"],
                ["observation", "a_obs", "2", "max"]]
    else:
        cells = ["low", "min", "max", "high", "unit"]
        numeric = [[0, 1, 1, 2, 0], [1, 1, 2, 2, 1],
                   [1, 1, 2, 2, 0], [0, 0, 0, 0, 0]]
        cell_rows = [["low", "a_obs", "observation"], ["min", "ref", "reference"],
                     ["max", "a_obs", "observation"], ["high", "z_obs", "observation"],
                     ["unit", "z_obs", "observation"]]
        maps = [["reference", "ref", "2", "min"],
                ["observation", "a_obs", "1", "low"],
                ["observation", "a_obs", "3", "max"],
                ["observation", "z_obs", "4", "high"],
                ["observation", "z_obs", "5", "unit"]]
    expression = table(checked_file(directory, "expression.tsv"), ("gene",))
    if expression[0] != ["gene", *cells] or [row[0] for row in expression[1:]] != ["C", "B", "A", "D"]:
        raise ValueError("tiny expression identities differ")
    if [[float(value) for value in row[1:]] for row in expression[1:]] != numeric:
        raise ValueError("tiny expression values differ")
    expected = {
        "genes.tsv": [["gene", "chr", "start", "stop"],
                      ["C", "chr10", "30", "40"], ["B", "chr2", "10", "20"],
                      ["A", "chr2", "10", "20"], ["D", "chr1", "1", "2"]],
        "cells.tsv": [["cell", "group", "role"], *cell_rows],
        "maps.tsv": [["role", "group", "index", "cell"], *maps],
    }
    for name, rows in expected.items():
        if table(checked_file(directory, name), tuple(rows[0])) != rows:
            raise ValueError(f"tiny {name} differs")


def validate_error(label, record):
    if not isinstance(record.get("error"), str) or not record["error"].strip() or \
            "outputs" in record or not isinstance(record.get("warnings"), list) or \
            any(not isinstance(message, str) or not message.strip() for message in record["warnings"]) or \
            record.get("error_class") != ["simpleError", "error", "condition"] or \
            not isinstance(record.get("error_call"), str) or not record["error_call"].strip():
        raise ValueError("missing or invalid expected error")
    message = " ".join(record["error"].split())
    call = " ".join(record["error_call"].split())
    expected = {
        "absent_annotation": ("all the annotated cell", "names match a sample in your data matrix.", "Attention to: absent"),
        "removed_reference": ("invalid argument type",),
        "no_common_genes": ("argument is of length zero",),
    }
    if label not in expected or any(fragment not in message for fragment in expected[label]):
        raise ValueError("unexpected creation error message")
    if label == "removed_reference" and "!all.equal(ref_group_names, orig_ref_group_names)" not in call:
        raise ValueError("unexpected creation error call")
    if label == "no_common_genes" and not call.startswith("if (num_genes_removed > 0)"):
        raise ValueError("unexpected creation error call")


def validate_files(directory, record, expected_paths, expected_suffixes):
    if set(expected_suffixes) != set(expected_paths):
        raise ValueError("creation input arguments differ")
    if set(record.get("input_sha256", {})) != set(expected_paths):
        raise ValueError("missing input hashes")
    for name, path in expected_paths.items():
        if digest(path) != record["input_sha256"][name]:
            raise ValueError("input hash mismatch")
        argument = record.get("creation_arguments", {}).get(name)
        if not isinstance(argument, str) or "\\" in argument:
            raise ValueError("creation input arguments differ")
        actual = PurePosixPath(argument)
        suffix = PurePosixPath(expected_suffixes[name]).parts
        if not actual.is_absolute() or str(actual) != argument or ".." in actual.parts or \
                actual.parts[-len(suffix):] != suffix:
            raise ValueError("creation input arguments differ")
    if "error" not in record:
        if set(record.get("outputs", {})) != set(OUTPUTS):
            raise ValueError("missing output hashes")
        for name in OUTPUTS:
            if digest(checked_file(directory, name)) != record["outputs"][name]:
                raise ValueError("output hash mismatch")


def validate_edges(shipped, witness):
    record = json.loads(checked_file(witness, "edges.json").read_text())
    source = json.loads(checked_file(shipped, "oracle.json").read_text())
    package = {"name": "infercnv", "version": "1.28.0", "source_commit": SOURCE_COMMIT,
               "source_archive_sha256": SOURCE_SHA256}
    if record.get("schema_version") != 1 or record.get("package") != package or source.get("package") != package:
        raise ValueError("unpinned package/source identity")
    runtime = record.get("runtime", {})
    if runtime.get("R_version") != "4.6.1" or not runtime.get("locale") or \
            not isinstance(runtime.get("longdouble_digits"), int) or runtime["longdouble_digits"] < 53 or \
            not isinstance(runtime.get("sizeof_longdouble"), int) or runtime["sizeof_longdouble"] < 8:
        raise ValueError("invalid runtime or long-double identity")
    if digest(checked_file(witness, "session.txt")) != record.get("session_sha256"):
        raise ValueError("session hash mismatch")
    cases = record.get("cases", {})
    if set(cases) != {"shipped_raw", *LIMITS, *ERRORS}:
        raise ValueError("missing or extra creation case")
    inputs = source["inputs"]
    paths = {"raw_counts_matrix": checked_file(shipped, inputs["original_counts"]),
             "gene_order_file": checked_file(shipped, inputs["original_gene_order"]),
             "annotations_file": checked_file(shipped, inputs["original_annotations"])}
    for name, pinned in ORIGINAL_SHA256.items():
        if digest(checked_file(shipped, inputs[name])) != pinned:
            raise ValueError("original shipped bytes differ")
    if digest(checked_file(shipped, inputs["source_archive"])) != SOURCE_SHA256:
        raise ValueError("source archive hash mismatch")
    stage = source["cases"]["full"]["stages"]["1"]
    shipped_case = cases["shipped_raw"]
    refs = source["cases"]["full"]["reference_groups"]
    if refs != list(REFS) or (stage.get("genes"), stage.get("cells")) != (9939, 184):
        raise ValueError("shipped stage-1 identity differs")
    validate_arguments(shipped_case, refs, [1, "Inf"])
    validate_files(witness / "shipped_raw", shipped_case, paths, {
        "raw_counts_matrix": f"shipped-bundle/{inputs['original_counts']}",
        "gene_order_file": f"shipped-bundle/{inputs['original_gene_order']}",
        "annotations_file": f"shipped-bundle/{inputs['original_annotations']}"})
    checkpoint = checked_file(shipped, stage["checkpoint"])
    if digest(checkpoint) != source["sha256"].get(stage["checkpoint"]) or \
            digest(checkpoint) != shipped_case.get("stage1_sha256") or \
            shipped_case.get("source_checkpoint") != stage["checkpoint"]:
        raise ValueError("stage-1 checkpoint hash mismatch")
    for name, key in (("expression.tsv", "expression"), ("genes.tsv", "gene_order"),
                      ("cells.tsv", "cell_groups")):
        expected = checked_file(shipped, stage[key])
        if digest(expected) != source["sha256"].get(stage[key]) or \
                checked_file(witness / "shipped_raw", name).read_bytes() != expected.read_bytes():
            raise ValueError(f"shipped stage-1 {name} differs")
    cells = table(checked_file(shipped, stage["cell_groups"]), ("cell", "group", "role"))[1:]
    expected_maps = [["role", "group", "index", "cell"]]
    for role, names in (("reference", refs), ("observation", sorted(
            {group for _, group, kind in cells if kind == "observation"}))):
        for group in names:
            expected_maps += [[role, group, str(i), cell] for i, (cell, name, kind)
                              in enumerate(cells, 1) if name == group and kind == role]
    if table(checked_file(witness / "shipped_raw", "maps.tsv"), tuple(expected_maps[0])) != expected_maps:
        raise ValueError("shipped maps differ from trusted cells and references")
    tiny = witness / "inputs"
    raw = {"counts.tsv": COUNTS, "positions.tsv": POSITIONS, "annotations.tsv": ANNOTATIONS,
           "absent.tsv": ANNOTATIONS + "absent\tz_obs\n",
           "removed.tsv": ANNOTATIONS.replace("low\ta_obs", "low\tlow_ref"),
           "no_common.tsv": "UNMATCHED\tchr1\t1\t2\n"}
    for name, content in raw.items():
        if checked_file(tiny, name).read_bytes() != content.encode():
            raise ValueError("literal tiny input bytes differ")
    for label in (*LIMITS, *ERRORS):
        annotation = "absent.tsv" if label == "absent_annotation" else \
            "removed.tsv" if label == "removed_reference" else "annotations.tsv"
        positions = "no_common.tsv" if label == "no_common_genes" else "positions.tsv"
        validate_files(witness / label, cases[label], {
            "raw_counts_matrix": tiny / "counts.tsv", "gene_order_file": tiny / positions,
            "annotations_file": tiny / annotation}, {
            "raw_counts_matrix": "creation-edges/inputs/counts.tsv",
            "gene_order_file": f"creation-edges/inputs/{positions}",
            "annotations_file": f"creation-edges/inputs/{annotation}"})
        if label in LIMITS:
            validate_tiny(witness / label, label, cases[label])
        else:
            validate_arguments(cases[label], ["low_ref"] if label == "removed_reference" else ["ref"], [3, 5])
            validate_error(label, cases[label])
    return {"shipped_raw": [len(cells), stage["genes"]], "tiny_cases": list(LIMITS),
            "expected_errors": list(ERRORS)}


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("shipped", type=Path)
    parser.add_argument("witness", type=Path)
    args = parser.parse_args()
    print(json.dumps(validate_edges(args.shipped, args.witness), sort_keys=True))
