"""Check a pinned infercnv creation witness against its source bundle."""

import argparse
import csv
import gzip
import hashlib
import json
import math
from pathlib import Path, PurePosixPath


SOURCE_COMMIT = "b421d9405c97a309b081ef86d455e976df93eae4"
SOURCE_SHA256 = "b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207"
OUTPUTS = ("expression.tsv", "genes.tsv", "cells.tsv", "maps.tsv")


def checked_file(root: Path, relative: str) -> Path:
    if not isinstance(relative, str) or not relative or "\\" in relative:
        raise ValueError("invalid bundle path")
    path = PurePosixPath(relative)
    if path.is_absolute() or any(part in (".", "..") for part in path.parts):
        raise ValueError("invalid bundle path")
    resolved = root.joinpath(*path.parts).resolve()
    if not resolved.is_relative_to(root.resolve()) or not resolved.is_file():
        raise ValueError("invalid bundle path")
    return resolved


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def table(path: Path, header: tuple[str, ...]) -> list[list[str]]:
    with path.open(newline="", encoding="utf-8") as handle:
        rows = list(csv.reader(handle, delimiter="\t"))
    if not rows or tuple(rows[0][:len(header)]) != header or len(rows) < 2:
        raise ValueError(f"invalid table: {path}")
    if any(len(row) != len(rows[0]) for row in rows[1:]):
        raise ValueError(f"ragged table: {path}")
    return rows


def _indices(value):
    if isinstance(value, int) and not isinstance(value, bool):
        return [value]
    if isinstance(value, list) and all(isinstance(x, int) and not isinstance(x, bool) for x in value):
        return value
    raise ValueError("invalid map indices")


def validate_arguments(record: dict, source_name: str, counts: str, positions: str,
                       annotations: str, references: list[str],
                       checkpoint: str | None = None) -> None:
    if checkpoint is not None and record.get("source_checkpoint") != checkpoint:
        raise ValueError("source checkpoint path mismatch")
    arguments = record.get("creation_arguments")
    paths = {"raw_counts_matrix": counts, "gene_order_file": positions,
             "annotations_file": annotations}
    expected = {"ref_group_names": references, "delim": "\t",
                "max_cells_per_group": None, "min_max_counts_per_cell": [1, "Inf"],
                "chr_exclude": ["chrX", "chrY", "chrM"]}
    if not isinstance(arguments, dict) or set(arguments) != set(paths) | set(expected) or \
            any(arguments[key] != value for key, value in expected.items()):
        raise ValueError("creation arguments mismatch")
    for key, relative in paths.items():
        value = arguments[key]
        if not isinstance(value, str) or "\\" in value:
            raise ValueError("creation input path mismatch")
        path = PurePosixPath(value)
        suffix = PurePosixPath(source_name, relative).parts
        if not path.is_absolute() or str(path) != value or ".." in path.parts or \
                path.parts[-len(suffix):] != suffix:
            raise ValueError("creation input path mismatch")


def validate_case(case_dir: Path, counts: Path, positions: Path, annotations: Path,
                  expected_expression: Path | None, record: dict, *,
                  expected_genes: Path | None = None,
                  expected_cells: Path | None = None) -> tuple[int, int]:
    for key, path, label in (("counts_sha256", counts, "count"),
                             ("positions_sha256", positions, "position"),
                             ("annotations_sha256", annotations, "annotation"),
                             ("gzip_sha256", checked_file(case_dir, "counts.tsv.gz"), "gzip")):
        if digest(path) != record.get(key):
            raise ValueError(f"{label} hash mismatch")
    try:
        decoded = gzip.decompress((case_dir / "counts.tsv.gz").read_bytes())
    except (OSError, EOFError) as error:
        raise ValueError("invalid gzip bytes") from error
    if decoded != counts.read_bytes():
        raise ValueError("gzip bytes differ from plain counts")

    for branch in ("plain", "gzip"):
        if set(record.get(branch, {})) != set(OUTPUTS):
            raise ValueError("incomplete output hashes")
        for name in OUTPUTS:
            if digest(checked_file(case_dir, f"{branch}/{name}")) != record[branch][name]:
                raise ValueError(f"{branch} output hash mismatch: {name}")
            if (case_dir / branch / name).read_bytes() != (case_dir / "plain" / name).read_bytes():
                raise ValueError(f"plain/gzip output differs: {name}")
    if expected_expression is not None and \
            (case_dir / "plain/expression.tsv").read_bytes() != expected_expression.read_bytes():
        raise ValueError("stage-1 expression differs")
    for name, expected in (("genes", expected_genes), ("cells", expected_cells)):
        if expected is not None and (case_dir / f"plain/{name}.tsv").read_bytes() != expected.read_bytes():
            raise ValueError(f"stage-1 {name} differs")

    expression = table(case_dir / "plain/expression.tsv", ("gene",))
    genes = table(case_dir / "plain/genes.tsv", ("gene", "chr", "start", "stop"))
    cells = table(case_dir / "plain/cells.tsv", ("cell", "group", "role"))
    maps = table(case_dir / "plain/maps.tsv", ("role", "group", "index", "cell"))
    try:
        values = [float(value) for row in expression[1:] for value in row[1:]]
    except ValueError as error:
        raise ValueError("expression must be finite numeric values") from error
    if any(not math.isfinite(value) or value < 0 for value in values):
        raise ValueError("expression must be finite nonnegative values")
    gene_ids = [row[0] for row in expression[1:]]
    cell_ids = expression[0][1:]
    if not gene_ids or not cell_ids or len(set(gene_ids)) != len(gene_ids) or \
            len(set(cell_ids)) != len(cell_ids) or \
            [row[0] for row in genes[1:]] != gene_ids or \
            [row[0] for row in cells[1:]] != cell_ids or \
            (len(gene_ids), len(cell_ids)) != (record.get("genes"), record.get("cells")):
        raise ValueError("gene/cell identity or dimensions differ")

    by_role = {"reference": {}, "observation": {}}
    assigned = set()
    for role, group, index_text, cell in maps[1:]:
        try:
            index = int(index_text)
        except ValueError as error:
            raise ValueError("invalid map index") from error
        if role not in by_role or index < 1 or index > len(cell_ids) or index in assigned or \
                cell_ids[index - 1] != cell or \
                cells[index][1:] != [group, role]:
            raise ValueError("ordered map differs from cells")
        assigned.add(index)
        by_role[role].setdefault(group, []).append(index)
    if assigned != set(range(1, len(cell_ids) + 1)):
        raise ValueError("map coverage differs")
    for role, key in (("reference", "reference_maps"),
                      ("observation", "observation_maps")):
        declared = record.get(key)
        if declared == [] and not by_role[role]:
            declared = {}
        if not isinstance(declared, dict) or list(declared) != list(by_role[role]) or \
                {name: _indices(value) for name, value in declared.items()} != by_role[role]:
            raise ValueError("ordered map differs from manifest")
    if list(by_role["reference"]) != record.get("references", []):
        raise ValueError("reference map order differs")
    return len(gene_ids), len(cell_ids)


def validate_small(directory: Path) -> None:
    expression = table(checked_file(directory, "expression.tsv"), ("gene",))
    if expression[0] != ["gene", "t_b", "ref2", "obs_a", "ref1"] or \
            [row[0] for row in expression[1:]] != ["G_A", "G_D", "G_B", "G_C"]:
        raise ValueError("small expression identities differ")
    upper = float.fromhex("0x1.391819d2391d6p-4")
    if [[float(x) for x in row[1:]] for row in expression[1:]] != [
        [2.0, 2.0, 2.0, 2.0], [upper, 2.0, 2.0, 2.0],
        [upper, 2.0, 2.0, 2.0], [2.0, 2.0, 2.0, 2.0],
    ]:
        raise ValueError("small numeric values differ")
    expected = {
        "genes.tsv": [["gene", "chr", "start", "stop"],
                      ["G_A", "chr10", "30", "31"], ["G_D", "chr2", "10", "11"],
                      ["G_B", "chr2", "20", "21"], ["G_C", "chr1", "5", "6"]],
        "cells.tsv": [["cell", "group", "role"], ["t_b", "z_obs", "observation"],
                      ["ref2", "a_ref", "reference"], ["obs_a", "a_obs", "observation"],
                      ["ref1", "z_ref", "reference"]],
        "maps.tsv": [["role", "group", "index", "cell"],
                     ["reference", "z_ref", "4", "ref1"],
                     ["reference", "a_ref", "2", "ref2"],
                     ["observation", "a_obs", "3", "obs_a"],
                     ["observation", "z_obs", "1", "t_b"]],
    }
    for name, rows in expected.items():
        if table(checked_file(directory, name), tuple(rows[0])) != rows:
            raise ValueError(f"small {name} differs")


def validate_witness(synthetic: Path, witness: Path, shipped: Path | None = None) -> dict:
    record = json.loads(checked_file(witness, "witness.json").read_text())
    package = record.get("package", {})
    if record.get("schema_version") != 1 or package != {
        "name": "infercnv", "version": "1.28.0", "source_commit": SOURCE_COMMIT,
        "source_archive_sha256": SOURCE_SHA256,
    }:
        raise ValueError("unpinned package/source identity")
    runtime = record.get("runtime", {})
    if runtime.get("R_version") != "4.6.1" or not isinstance(runtime.get("locale"), str) or \
            not runtime["locale"]:
        raise ValueError("missing pinned R runtime or locale")
    if digest(checked_file(witness, "session.txt")) != record.get("session_sha256"):
        raise ValueError("session hash mismatch")
    cases = record.get("cases")
    expected_names = {"synthetic_grouped_bounds", "synthetic_no_reference", "small"}
    if shipped is not None:
        expected_names.add("shipped_full")
    if not isinstance(cases, dict) or set(cases) != expected_names:
        raise ValueError("missing or extra witness case")

    synthetic_record = json.loads(checked_file(synthetic, "oracle.json").read_text())
    if synthetic_record.get("package") != package:
        raise ValueError("synthetic source identity differs")
    results = {}
    for profile in ("grouped_bounds", "no_reference"):
        label = f"synthetic_{profile}"
        source = synthetic_record["profiles"][profile]["stages"]["1"]
        args = synthetic_record["inputs"]
        validate_arguments(cases[label], "bundle", args["counts.tsv"],
                           args["gene_order.tsv"], args["annotations.tsv"],
                           synthetic_record["profiles"][profile]["settings"]["ref_group_names"],
                           source["checkpoint"])
        if digest(checked_file(synthetic, source["checkpoint"])) != cases[label].get("stage1_sha256"):
            raise ValueError("stage-1 checkpoint hash mismatch")
        results[label] = validate_case(
            witness / label, checked_file(synthetic, args["counts.tsv"]),
            checked_file(synthetic, args["gene_order.tsv"]),
            checked_file(synthetic, args["annotations.tsv"]),
            checked_file(synthetic, source["expression"]), cases[label],
            expected_genes=checked_file(synthetic, source["gene_order"]),
            expected_cells=checked_file(synthetic, source["cell_groups"]))
        if cases[label]["references"] != synthetic_record["profiles"][profile]["settings"]["ref_group_names"]:
            raise ValueError("reference names differ from source profile")
    if shipped is not None:
        shipped_record = json.loads(checked_file(shipped, "oracle.json").read_text())
        if shipped_record.get("package") != package:
            raise ValueError("shipped source identity differs")
        source = shipped_record["cases"]["full"]
        stage = source["stages"]["1"]
        args = shipped_record["inputs"]
        validate_arguments(cases["shipped_full"], "shipped-bundle",
                           args[source["input_counts"]], args["original_gene_order"],
                           args["original_annotations"], source["reference_groups"],
                           stage["checkpoint"])
        if digest(checked_file(shipped, stage["checkpoint"])) != cases["shipped_full"].get("stage1_sha256"):
            raise ValueError("shipped stage-1 checkpoint hash mismatch")
        results["shipped_full"] = validate_case(
            witness / "shipped_full", checked_file(shipped, args[source["input_counts"]]),
            checked_file(shipped, args["original_gene_order"]),
            checked_file(shipped, args["original_annotations"]),
            checked_file(shipped, stage["expression"]), cases["shipped_full"],
            expected_genes=checked_file(shipped, stage["gene_order"]),
            expected_cells=checked_file(shipped, stage["cell_groups"]))
        if cases["shipped_full"]["references"] != source["reference_groups"]:
            raise ValueError("reference names differ from shipped profile")
    small = witness / "small_inputs"
    validate_arguments(cases["small"], "ingestion-witness/small_inputs",
                       "counts.tsv", "positions.tsv", "annotations.tsv", ["z_ref", "a_ref"])
    results["small"] = validate_case(
        witness / "small", checked_file(small, "counts.tsv"),
        checked_file(small, "positions.tsv"),
        checked_file(small, "annotations.tsv"),
        None, cases["small"])
    validate_small(witness / "small/plain")
    if results["small"] != (4, 4) or cases["small"].get("decimal") != {
        "literal": "0.076439", "numeric17": "0.076439000000000007",
        "hex": "0x1.391819d2391d6p-4",
    }:
        raise ValueError("small decimal or shape witness differs")
    return results


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("synthetic", type=Path)
    parser.add_argument("witness", type=Path)
    parser.add_argument("--shipped", type=Path)
    options = parser.parse_args()
    print(json.dumps(validate_witness(options.synthetic, options.witness, options.shipped),
                     sort_keys=True))
