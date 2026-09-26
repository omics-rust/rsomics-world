"""Validate an inferCNV 1.28.0 oracle bundle and optionally write a byte manifest.

``oracle.json`` schema 1 has ``package`` (name ``infercnv``, version
``1.28.0``, the pinned 40-character ``source_commit`` and 64-character
``source_archive_sha256``), integer ``seed``, explicit shared run ``settings``
(including null ``num_ref_groups`` and true clustering flags),
``creation_settings`` (null max cells/group, ``[1,"Inf"]`` cell-count limits,
and X/Y/M exclusions),
``inputs`` and ``metadata`` maps of bundle-relative file paths, an
``input_sha256`` map for the three input files, and exactly
four ``profiles``. Each profile has ``settings`` with ``ref_group_names`` and
``ref_subtract_use_mean_bounds``, plus ``stages`` keyed by decimal step number.
The required steps are 1,2,3,4,8,9,10,11,12,14. Each stage gives relative
``expression``, ``gene_order``, ``cell_groups``, and original ``checkpoint``
paths. Expression TSV starts ``gene<TAB>cell...`` and contains finite numeric
values; gene-order TSV starts ``gene<TAB>chr<TAB>start<TAB>stop``; cell-group
TSV starts ``cell<TAB>group<TAB>role`` with role ``reference`` or
``observation``. Row and column order is authoritative and must align within
and across stages and with input identity/coordinates. The checker independently
tests depth normalization, log2/inverse-log2, unchanged singleton smoothing,
stage-12 centered and stage-14 final gain/loss signs, and divergent
reference-mode exports. No RDS payload
is interpreted by this checker.
"""

import argparse
import csv
import hashlib
import json
import math
import statistics
from pathlib import Path, PurePosixPath


COMMIT = "b421d9405c97a309b081ef86d455e976df93eae4"
PROFILES = ("single_reference", "grouped_bounds", "grouped_mean", "no_reference")
STAGES = (1, 2, 3, 4, 8, 9, 10, 11, 12, 14)
SETTINGS = {
    "cutoff": 0.1, "min_cells_per_gene": 3, "window_length": 101,
    "smooth_method": "pyramidinal", "max_centered_threshold": 3,
    "scale_data": False, "HMM": False, "denoise": False,
    "analysis_mode": "samples", "num_threads": 1,
    "resume_mode": False, "plot_steps": False, "no_plot": True,
    "no_prelim_plot": True, "save_rds": True,
    "remove_genes_at_chr_ends": False, "prune_outliers": False,
    "mask_nonDE_genes": False, "up_to_step": 14,
    "num_ref_groups": None, "cluster_by_groups": True,
    "cluster_references": True,
    "tumor_subcluster_partition_method": "leiden",
    "BayesMaxPNormal": 0.5,
}
CREATION_SETTINGS = {
    "max_cells_per_group": None,
    "min_max_counts_per_cell": [1, "Inf"],
    "chr_exclude": ["chrX", "chrY", "chrM"],
}


def _file(root: Path, value: str) -> Path:
    if not isinstance(value, str) or not value or "\\" in value:
        raise ValueError("invalid bundle path")
    rel = PurePosixPath(value)
    if rel.is_absolute() or any(part in (".", "..") for part in rel.parts):
        raise ValueError(f"path escapes bundle: {value}")
    path = root.joinpath(*rel.parts)
    if not path.resolve().is_relative_to(root.resolve()):
        raise ValueError(f"path escapes bundle: {value}")
    if not path.is_file():
        raise ValueError(f"missing referenced file: {value}")
    return path


def _table(path: Path, header: tuple[str, ...], variable_columns: bool = False) -> list[list[str]]:
    with path.open(newline="", encoding="utf-8") as handle:
        rows = list(csv.reader(handle, delimiter="\t"))
    if not rows or (rows[0][:1] != list(header[:1]) if variable_columns else rows[0] != list(header)):
        raise ValueError(f"invalid TSV header: {path}")
    if variable_columns and len(rows[0]) < 2:
        raise ValueError(f"empty cell columns: {path}")
    width = len(rows[0])
    if any(len(row) != width for row in rows[1:]):
        raise ValueError(f"ragged TSV rows: {path}")
    if len(rows) < 2:
        raise ValueError(f"empty TSV: {path}")
    return rows


def _unique(values: list[str], what: str) -> None:
    if any(not value for value in values) or len(values) != len(set(values)):
        raise ValueError(f"duplicate or blank {what} ID")


def _stage(root: Path, entry: dict, refs: list[str], raw_order: dict, annotations: dict) -> dict:
    if not isinstance(entry, dict) or set(entry) != {"expression", "gene_order", "cell_groups", "checkpoint"}:
        raise ValueError("stage artifact paths incomplete")
    expression = _table(_file(root, entry["expression"]), ("gene",), True)
    genes = [row[0] for row in expression[1:]]
    cells = expression[0][1:]
    _unique(genes, "gene")
    _unique(cells, "cell")
    values = []
    for row in expression[1:]:
        try:
            numeric = tuple(float(x) for x in row[1:])
        except ValueError as exc:
            raise ValueError("non-numeric expression") from exc
        if not all(math.isfinite(x) for x in numeric):
            raise ValueError("expression must be finite")
        values.append(numeric)
    order = _table(_file(root, entry["gene_order"]), ("gene", "chr", "start", "stop"))
    order_genes = [row[0] for row in order[1:]]
    _unique(order_genes, "gene")
    if order_genes != genes:
        raise ValueError("gene order mismatch")
    coordinates = {}
    for row in order[1:]:
        try:
            start, stop = int(row[2]), int(row[3])
        except ValueError as exc:
            raise ValueError("invalid gene coordinates") from exc
        if not row[1] or start < 1 or stop < start:
            raise ValueError("invalid gene coordinates")
        coordinates[row[0]] = (row[1], start, stop)
        if coordinates[row[0]] != raw_order.get(row[0]):
            raise ValueError("gene coordinate differs from input")
    groups = _table(_file(root, entry["cell_groups"]), ("cell", "group", "role"))
    group_cells = [row[0] for row in groups[1:]]
    _unique(group_cells, "cell")
    if group_cells != cells:
        raise ValueError("cell order mismatch")
    if any(not row[1] or row[2] not in ("reference", "observation") for row in groups[1:]):
        raise ValueError("invalid cell group or role")
    if any(ref not in [row[1] for row in groups[1:]] for ref in refs):
        raise ValueError("missing requested reference group")
    if any((row[1] in refs) != (row[2] == "reference") for row in groups[1:]):
        raise ValueError("reference role does not match profile settings")
    if any(annotations.get(row[0]) != row[1] for row in groups[1:]):
        raise ValueError("cell annotation differs from input")
    _file(root, entry["checkpoint"])
    return {"genes": genes, "cells": cells, "values": tuple(values),
            "coordinates": coordinates, "groups": {row[0]: row[1] for row in groups[1:]}}


def _inputs(root: Path, paths: dict) -> tuple[dict, dict, dict, list[str]]:
    counts = _table(_file(root, paths["counts.tsv"]), ("gene",), True)
    raw_cells = counts[0][1:]
    _unique(raw_cells, "raw cell")
    raw_counts = {}
    for row in counts[1:]:
        if not row[0] or row[0] in raw_counts:
            raise ValueError("duplicate raw gene ID")
        try:
            raw_counts[row[0]] = tuple(float(x) for x in row[1:])
        except ValueError as exc:
            raise ValueError("non-numeric raw count") from exc
        if any(not math.isfinite(x) or x < 0 for x in raw_counts[row[0]]):
            raise ValueError("invalid raw count")
    with _file(root, paths["annotations.tsv"]).open(newline="", encoding="utf-8") as handle:
        annotation_rows = list(csv.reader(handle, delimiter="\t"))
    annotations = {}
    for row in annotation_rows:
        if len(row) != 2 or not all(row) or row[0] in annotations:
            raise ValueError("invalid raw cell annotation")
        annotations[row[0]] = row[1]
    if not annotations or set(annotations) != set(raw_cells):
        raise ValueError("raw cell annotation identity mismatch")
    with _file(root, paths["gene_order.tsv"]).open(newline="", encoding="utf-8") as handle:
        order_rows = list(csv.reader(handle, delimiter="\t"))
    raw_order = {}
    for row in order_rows:
        if len(row) != 4 or not row[0] or not row[1] or row[0] in raw_order:
            raise ValueError("invalid raw gene order")
        try:
            start, stop = int(row[2]), int(row[3])
        except ValueError as exc:
            raise ValueError("invalid raw gene coordinate") from exc
        if start < 1 or stop < start:
            raise ValueError("invalid raw gene coordinate")
        raw_order[row[0]] = (row[1], start, stop)
    if not raw_order:
        raise ValueError("empty raw gene order")
    return raw_counts, raw_order, annotations, raw_cells


def _same_numbers(left: tuple, right: tuple) -> bool:
    return len(left) == len(right) and all(
        math.isclose(a, b, rel_tol=1e-9, abs_tol=1e-9)
        for left_row, right_row in zip(left, right)
        for a, b in zip(left_row, right_row)
    )


def _contrast(stages: dict) -> None:
    single = stages["single_reference"]
    final = single[14]
    rows = {gene: i for i, gene in enumerate(final["genes"])}
    columns = {cell: i for i, cell in enumerate(final["cells"])}
    gain_rows = [rows[g] for g in final["genes"] if final["coordinates"][g][0] == "chrGain"]
    loss_rows = [rows[g] for g in final["genes"] if final["coordinates"][g][0] == "chrLoss"]
    gain_cells = [columns[c] for c in final["cells"] if final["groups"][c] == "gain_obs"]
    if not gain_rows or not loss_rows or not gain_cells:
        raise ValueError("gain/loss fixture identities absent")
    gain = statistics.median(final["values"][i][j] for i in gain_rows for j in gain_cells)
    loss = statistics.median(final["values"][i][j] for i in loss_rows for j in gain_cells)
    if gain <= 1 or loss >= 1:
        raise ValueError("gain/loss fixture signal absent")
    centered = single[12]
    centered_gain = statistics.median(centered["values"][i][j] for i in gain_rows for j in gain_cells)
    centered_loss = statistics.median(centered["values"][i][j] for i in loss_rows for j in gain_cells)
    if centered_gain <= 0 or centered_loss >= 0:
        raise ValueError("stage 12 gain/loss fixture signal absent")
    for left, right in (("single_reference", "grouped_bounds"),
                        ("grouped_bounds", "grouped_mean"),
                        ("single_reference", "no_reference")):
        if _same_numbers(stages[left][8]["values"], stages[right][8]["values"]):
            raise ValueError(f"profile contrast absent: {left} vs {right}")


def validate_bundle(root: Path) -> dict:
    """Return profile/stage counts for a complete bundle; reject with ValueError."""
    root = Path(root)
    try:
        data = json.loads((root / "oracle.json").read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ValueError("missing or invalid oracle.json") from exc
    if not isinstance(data, dict) or data.get("schema_version") != 1:
        raise ValueError("unsupported oracle schema version")
    package = data.get("package", {})
    if not isinstance(package, dict):
        raise ValueError("invalid package identity")
    if package.get("name") != "infercnv" or package.get("version") != "1.28.0":
        raise ValueError("wrong infercnv package version")
    if package.get("source_commit") != COMMIT:
        raise ValueError("wrong infercnv source commit")
    archive_hash = package.get("source_archive_sha256")
    if not isinstance(archive_hash, str) or len(archive_hash) != 64 or any(c not in "0123456789abcdef" for c in archive_hash):
        raise ValueError("missing source archive SHA-256")
    if not isinstance(data.get("seed"), int):
        raise ValueError("missing integer seed")
    settings = data.get("settings", {})
    if not isinstance(settings, dict) or set(settings) != set(SETTINGS) or any(
        type(settings[k]) is not type(v) or settings[k] != v for k, v in SETTINGS.items()
    ):
        raise ValueError("oracle settings mismatch")
    if data.get("creation_settings") != CREATION_SETTINGS:
        raise ValueError("oracle creation settings mismatch")
    for key, names in (("inputs", ("counts.tsv", "annotations.tsv", "gene_order.tsv")),
                       ("metadata", ("session.txt", "packages.tsv", "source.txt"))):
        mapping = data.get(key)
        if not isinstance(mapping, dict) or set(mapping) != set(names):
            raise ValueError(f"missing {key} evidence")
        for value in mapping.values():
            _file(root, value)
    input_hashes = data.get("input_sha256")
    if not isinstance(input_hashes, dict) or set(input_hashes) != set(data["inputs"]):
        raise ValueError("missing input SHA-256 hashes")
    for name, value in data["inputs"].items():
        if input_hashes[name] != hashlib.sha256(_file(root, value).read_bytes()).hexdigest():
            raise ValueError(f"input SHA-256 mismatch: {name}")
    raw_counts, raw_order, annotations, raw_cells = _inputs(root, data["inputs"])
    profiles = data.get("profiles")
    if not isinstance(profiles, dict) or set(profiles) != set(PROFILES):
        raise ValueError("missing or unexpected profile")
    total = 0
    all_stages = {}
    first_stage = None
    for name in PROFILES:
        profile = profiles[name]
        if not isinstance(profile, dict) or not isinstance(profile.get("settings"), dict):
            raise ValueError(f"missing {name} profile settings")
        psettings = profile["settings"]
        refs = psettings.get("ref_group_names")
        bounds = psettings.get("ref_subtract_use_mean_bounds")
        expected_refs = [] if name == "no_reference" else (["normal_a"] if name == "single_reference" else ["normal_a", "normal_b"])
        if refs != expected_refs:
            raise ValueError(f"invalid {name} reference settings")
        if bounds is not (name != "grouped_mean"):
            raise ValueError(f"invalid {name} bounds setting")
        stages = profile.get("stages")
        if not isinstance(stages, dict) or set(stages) != {str(x) for x in STAGES}:
            raise ValueError(f"missing or unexpected stage in {name}")
        prior = None
        stage_data = {}
        for number in STAGES:
            current = _stage(root, stages[str(number)], refs, raw_order, annotations)
            genes, cells, values = current["genes"], current["cells"], current["values"]
            if number == 1:
                if not set(genes).issubset(raw_counts) or not set(cells).issubset(raw_cells):
                    raise ValueError("incoming stage identities not in raw input")
                if [cell for cell in raw_cells if cell in set(cells)] != cells:
                    raise ValueError("incoming cell order differs from counts input")
                for gene, row in zip(genes, values):
                    expected = tuple(raw_counts[gene][raw_cells.index(cell)] for cell in cells)
                    if row != expected:
                        raise ValueError("incoming expression differs from raw counts")
                if first_stage is None:
                    first_stage = current
                elif (genes != first_stage["genes"] or cells != first_stage["cells"] or
                      values != first_stage["values"]):
                    raise ValueError("profile incoming identities or counts differ")
            if prior is not None and cells != prior["cells"]:
                raise ValueError(f"cell order changed across stages in {name}")
            if prior is not None:
                if number == 2:
                    if [gene for gene in prior["genes"] if gene in set(genes)] != genes:
                        raise ValueError(f"gene order changed across stages in {name}")
                elif genes != prior["genes"]:
                    raise ValueError(f"gene order changed across stages in {name}")
            if number in (3, 4) and values == prior["values"]:
                raise ValueError(f"unexpectedly unchanged stage {number} in {name}")
            if number == 3:
                expected_depth = statistics.median(sum(row[j] for row in prior["values"])
                                                   for j in range(len(cells)))
                if any(not math.isclose(sum(row[j] for row in values), expected_depth,
                                        rel_tol=1e-9, abs_tol=1e-9) for j in range(len(cells))):
                    raise ValueError("normalized depth differs across cells")
            if number == 4 and any(
                not math.isclose(value, math.log2(prior["values"][i][j] + 1),
                                 rel_tol=1e-9, abs_tol=1e-9)
                for i, row in enumerate(values) for j, value in enumerate(row)
            ):
                raise ValueError("stage 4 log2 transform mismatch")
            if number == 10:
                by_chromosome = {}
                for gene in genes:
                    chr_name = current["coordinates"][gene][0]
                    by_chromosome[chr_name] = by_chromosome.get(chr_name, 0) + 1
                for i, gene in enumerate(genes):
                    if by_chromosome[current["coordinates"][gene][0]] == 1 and values[i] != prior["values"][i]:
                        raise ValueError("singleton chromosome changed during smoothing")
            if number == 14 and any(
                not math.isclose(value, 2 ** prior["values"][i][j],
                                 rel_tol=1e-9, abs_tol=1e-9)
                for i, row in enumerate(values) for j, value in enumerate(row)
            ):
                raise ValueError("stage 14 inverse log2 mismatch")
            prior = current
            stage_data[number] = current
            total += 1
        all_stages[name] = stage_data
    _contrast(all_stages)
    return {"profiles": len(PROFILES), "stages": total}


def _manifest(root: Path) -> Path:
    target = root / "sha256-manifest.tsv"
    lines = ["sha256\tpath\n"]
    for path in sorted(root.rglob("*")):
        if path.is_file() and path != target:
            lines.append(f"{hashlib.sha256(path.read_bytes()).hexdigest()}\t{path.relative_to(root).as_posix()}\n")
    target.write_text("".join(lines), encoding="utf-8")
    return target


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--manifest", action="store_true", help="write sorted sha256-manifest.tsv after validation")
    args = parser.parse_args()
    counts = validate_bundle(args.directory)
    if args.manifest:
        print(_manifest(args.directory))
    print(json.dumps(counts, sort_keys=True))
