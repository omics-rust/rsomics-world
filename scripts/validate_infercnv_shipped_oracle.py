"""Validate the pinned inferCNV 1.28.0 shipped-data oracle bundle.

``oracle.json`` schema 2 has exactly ``schema_version``, ``package``,
``inputs``, ``metadata``, ``creation_settings``, ``run_settings``, ``cases``
and ``sha256``. Package identifies source commit/archive and installed version;
inputs map the three original shipped bytes, canonical full/subset counts and
source archive to owned relative paths. Metadata maps session/package/source
text. Cases are exactly ``subset`` and ``full``; each records its input-count
key, selected chromosomes, expected reference groups, and ten stage entries.
Each stage 1,2,3,4,8,9,10,11,12,14 records owned expression, gene-order,
cell-group and original RDS paths plus actual gene/cell dimensions. ``sha256``
maps every referenced file path to its byte digest. Source and original input
hashes are independently pinned here, outside generated metadata. TSV identity
and numerical validation is streamed per stage pair using compact double
arrays; no ten-matrix Python-float collection is retained. RDS bytes are
preserved and hashed, not parsed by this independent checker.
"""

import argparse
from array import array
import csv
import gzip
import hashlib
import json
import math
import re
import statistics
import sys
from pathlib import Path, PurePosixPath
from typing import NamedTuple


COMMIT = "b421d9405c97a309b081ef86d455e976df93eae4"
SOURCE_SHA256 = "b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207"
ORIGINAL_SHA256 = {
    "original_counts": "7b7c618e6b03d589ea36979b074d0b83e127dcaa01e059ba38f73900e7609ba3",
    "original_annotations": "345493e0686d75418427e9c4401f3f7bbb55ae3f61deee074ccee8882dc4dc63",
    "original_gene_order": "4ec63e049ea8299948fb730c2f60ca5a038a798b77cec3bb918fda572b2dc52e",
}
STAGES = (1, 2, 3, 4, 8, 9, 10, 11, 12, 14)
REFS = ("Microglia/Macrophage", "Oligodendrocytes (non-malignant)")
RUN_SETTINGS = {
    "cutoff": 1, "min_cells_per_gene": 3, "window_length": 101,
    "smooth_method": "pyramidinal", "max_centered_threshold": 3,
    "scale_data": False, "HMM": False, "denoise": False,
    "analysis_mode": "samples", "num_threads": 1,
    "resume_mode": False, "plot_steps": False, "no_plot": True,
    "no_prelim_plot": True, "save_rds": True,
    "remove_genes_at_chr_ends": False, "prune_outliers": False,
    "mask_nonDE_genes": False, "up_to_step": 14,
    "num_ref_groups": None, "cluster_by_groups": True,
    "cluster_references": False, "ref_subtract_use_mean_bounds": True,
    "tumor_subcluster_partition_method": "leiden", "BayesMaxPNormal": 0.5,
}
CREATION_SETTINGS = {"delim": "\t", "max_cells_per_group": None,
                     "min_max_counts_per_cell": [1, "Inf"],
                     "chr_exclude": ["chrX", "chrY", "chrM"],
                     "ref_group_names": list(REFS)}


class Matrix(NamedTuple):
    cells: tuple[str, ...]
    genes: tuple[str, ...]
    rows: tuple[array, ...]


def _unique(values, kind):
    if not values or any(not x for x in values) or len(set(values)) != len(values):
        raise ValueError(f"duplicate or missing {kind} ID")


def _read_matrix(path, expected_cells, original, allow_negative=False):
    opening = gzip.open if original else open
    with opening(path, "rt", encoding="utf-8", newline="") as handle:
        records = csv.reader(handle, delimiter="\t")
        try:
            header = next(records)
        except StopIteration as exc:
            raise ValueError("empty count matrix") from exc
        if not original:
            if not header or header[0] != "gene":
                raise ValueError("canonical count header missing gene")
            header = header[1:]
        if len(header) != expected_cells:
            raise ValueError("count header width mismatch")
        _unique(header, "cell")
        genes, rows = [], []
        for record in records:
            if len(record) != expected_cells + 1:
                raise ValueError("count data row width mismatch")
            genes.append(record[0])
            try:
                numeric = array("d", (float(x) for x in record[1:]))
            except ValueError as exc:
                raise ValueError("invalid count value") from exc
            if any(not math.isfinite(x) or (x < 0 and not allow_negative) for x in numeric):
                raise ValueError("nonfinite or negative count value")
            rows.append(numeric)
    _unique(genes, "gene")
    return Matrix(tuple(header), tuple(genes), tuple(rows))


def read_original(path, expected_cells=184):
    return _read_matrix(path, expected_cells, True)


def read_canonical(path, expected_cells=184, allow_negative=False):
    return _read_matrix(path, expected_cells, False, allow_negative)


def validate_canonical(original, canonical):
    if original.cells != canonical.cells or original.genes != canonical.genes or original.rows != canonical.rows:
        raise ValueError("canonical count derivation differs from original gzip")


def validate_subset(original, subset, expected_genes):
    source = {gene: row for gene, row in zip(original.genes, original.rows)}
    if subset.cells != original.cells or subset.genes != tuple(expected_genes) or any(
        row != source[gene] for gene, row in zip(subset.genes, subset.rows)
    ):
        raise ValueError("subset canonical membership/value differs from pinned input")


def validate_stage1(original, stage, expected_genes):
    if stage.cells != original.cells or stage.genes != tuple(expected_genes):
        raise ValueError("stage 1 identity differs from pinned input")
    source = {gene: row for gene, row in zip(original.genes, original.rows)}
    if any(stage_row != source[gene] for gene, stage_row in zip(stage.genes, stage.rows)):
        raise ValueError("stage 1 raw value differs from pinned input")
    if any(sum(row[j] for row in stage.rows) <= 0 for j in range(len(stage.cells))):
        raise ValueError("stage 1 nonpositive cell depth")


def validate_stage2(stage1, stage2, cutoff=1, min_cells=3):
    retained = tuple(g for g, row in zip(stage1.genes, stage1.rows)
                     if sum(row) / len(stage1.cells) >= cutoff and
                     sum(x > 0 for x in row) >= min_cells)
    if stage2.cells != stage1.cells or stage2.genes != retained:
        raise ValueError("stage 2 retained gene membership differs from input")
    source = {g: row for g, row in zip(stage1.genes, stage1.rows)}
    if any(row != source[g] for g, row in zip(stage2.genes, stage2.rows)):
        raise ValueError("stage 2 retained value changed")


def _close(actual, expected):
    return math.isclose(actual, expected, rel_tol=1e-9, abs_tol=1e-9)


def validate_arithmetic(left, right, operation):
    if left.genes != right.genes or left.cells != right.cells:
        raise ValueError(f"{operation} identity differs")
    if operation == "depth":
        depths = [sum(row[j] for row in left.rows) for j in range(len(left.cells))]
        if any(x <= 0 for x in depths):
            raise ValueError("depth normalization has nonpositive input")
        median = statistics.median(depths)
    elif operation == "recenter":
        centers = [statistics.median(row[j] for row in left.rows) for j in range(len(left.cells))]
    for source, output in zip(left.rows, right.rows):
        for j, (x, y) in enumerate(zip(source, output)):
            if operation == "depth":
                expected = x / depths[j] * median
            elif operation == "log2":
                expected = math.log2(x + 1)
            elif operation == "inverse":
                expected = 2 ** x
            elif operation == "clamp":
                expected = min(3, max(-3, x))
            elif operation == "recenter":
                expected = x - centers[j]
            else:
                raise ValueError(f"unknown arithmetic operation: {operation}")
            if not _close(y, expected):
                raise ValueError(f"{operation} arithmetic mismatch")


def owned_file(root, value):
    if not isinstance(value, str) or not value or "\\" in value:
        raise ValueError("invalid owned path")
    rel = PurePosixPath(value)
    if rel.is_absolute() or any(part in (".", "..") for part in value.split("/")):
        raise ValueError(f"unsafe owned path: {value}")
    path = root.joinpath(*rel.parts)
    if not path.resolve().is_relative_to(root.resolve()):
        raise ValueError(f"unsafe owned path: {value}")
    if not path.is_file():
        raise ValueError(f"missing referenced file: {value}")
    return path


def file_sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def require_sha256(path, expected):
    if not isinstance(expected, str) or len(expected) != 64 or file_sha256(path) != expected:
        raise ValueError(f"SHA-256 mismatch: {path}")


def _table(path, width):
    with path.open(encoding="utf-8", newline="") as handle:
        rows = list(csv.reader(handle, delimiter="\t"))
    if not rows or any(len(row) != width for row in rows):
        raise ValueError(f"malformed TSV: {path}")
    return rows


def _identities(root, stage, matrix, coordinates, annotations):
    genes = _table(owned_file(root, stage["gene_order"]), 4)
    cells = _table(owned_file(root, stage["cell_groups"]), 3)
    if genes[0] != ["gene", "chr", "start", "stop"] or cells[0] != ["cell", "group", "role"]:
        raise ValueError("stage identity header mismatch")
    if tuple(row[0] for row in genes[1:]) != matrix.genes or tuple(row[0] for row in cells[1:]) != matrix.cells:
        raise ValueError("stage gene/cell identity order mismatch")
    for gene, chromosome, start, stop in genes[1:]:
        if (chromosome, start, stop) != coordinates.get(gene):
            raise ValueError("stage coordinate differs from input")
    for cell, group, role in cells[1:]:
        if annotations.get(cell) != group or role != ("reference" if group in REFS else "observation"):
            raise ValueError("stage group/role differs from input")


def _original_order(path):
    rows = _table(path, 4)
    result = {}
    chromosomes = []
    for gene, chromosome, start, stop in rows:
        if not gene or gene in result or not chromosome:
            raise ValueError("duplicate or missing gene coordinate ID")
        try:
            begin, end = int(start), int(stop)
        except ValueError as exc:
            raise ValueError("invalid gene coordinate") from exc
        if begin < 1 or end < begin:
            raise ValueError("invalid gene coordinate")
        result[gene] = (chromosome, start, stop)
        if chromosome not in chromosomes:
            chromosomes.append(chromosome)
    return result, {name: i for i, name in enumerate(chromosomes)}


def _annotations(path, cells):
    rows = _table(path, 2)
    result = {}
    for cell, group in rows:
        if not cell or not group or cell in result:
            raise ValueError("duplicate or missing annotation ID")
        result[cell] = group
    if set(result) != set(cells) or len(result) != 184:
        raise ValueError("annotation cell identity mismatch")
    if any(sum(group == ref for group in result.values()) != count for ref, count in zip(REFS, (19, 23))):
        raise ValueError("reference group size mismatch")
    if len(set(result.values())) != 6:
        raise ValueError("annotation group count mismatch")
    return result


def _require_exact(mapping, keys, context):
    if not isinstance(mapping, dict) or set(mapping) != set(keys):
        raise ValueError(f"{context} schema mismatch")


def _validate_metadata(root, metadata, archive_sha):
    source_rows = _table(owned_file(root, metadata["source"]), 2)
    source = {}
    for key, value in source_rows:
        if not key or not value or key in source:
            raise ValueError("source metadata invalid")
        source[key] = value
    if source != {"source_commit": COMMIT, "source_archive_sha256": archive_sha,
                  "source_url": "https://github.com/bioconductor-source/infercnv"}:
        raise ValueError("source metadata identity mismatch")
    packages = _table(owned_file(root, metadata["packages"]), 3)
    if packages[0] != ["Package", "Version", "LibPath"] or len(packages) < 2:
        raise ValueError("package metadata header or rows invalid")
    seen = set()
    found = False
    for name, version, library in packages[1:]:
        identity = (name, library)
        if not name or not version or not library or identity in seen:
            raise ValueError("package metadata row invalid")
        seen.add(identity)
        if name == "infercnv" and version == "1.28.0":
            found = True
    if not found:
        raise ValueError("package metadata infercnv version mismatch")
    session = owned_file(root, metadata["session"]).read_text(encoding="utf-8")
    required_lines = (r"^R version 4\.6\.1(?:\s|$)", r"^Platform:\s+\S+",
                      r"^BLAS:\s+\S+", r"^LAPACK:\s+\S+")
    if any(re.search(pattern, session, re.MULTILINE) is None for pattern in required_lines) or \
            re.search(r"\binfercnv_1\.28\.0\b", session) is None:
        raise ValueError("runtime metadata missing R, platform, BLAS, LAPACK or infercnv identity")


def _manifest_text(root, hashes):
    expected = dict(hashes)
    expected["oracle.json"] = file_sha256(root / "oracle.json")
    return "sha256\tpath\n" + "".join(f"{expected[rel]}\t{rel}\n" for rel in sorted(expected))


def validate_bundle(root):
    root = Path(root)
    try:
        data = json.loads((root / "oracle.json").read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ValueError("missing or invalid oracle.json") from exc
    _require_exact(data, ("schema_version", "package", "inputs", "metadata",
                          "creation_settings", "run_settings", "cases", "sha256"), "root")
    if data["schema_version"] != 2:
        raise ValueError("unsupported oracle schema")
    package = data["package"]
    _require_exact(package, ("name", "version", "source_commit", "source_archive_sha256"), "package")
    if package != {"name": "infercnv", "version": "1.28.0", "source_commit": COMMIT,
                   "source_archive_sha256": SOURCE_SHA256}:
        raise ValueError("pinned source/package identity mismatch")
    if data["creation_settings"] != CREATION_SETTINGS or data["run_settings"] != RUN_SETTINGS:
        raise ValueError("creation/run arguments mismatch")
    inputs = data["inputs"]
    metadata = data["metadata"]
    _require_exact(inputs, ("source_archive", "original_counts", "original_annotations",
                            "original_gene_order", "canonical_full", "canonical_subset"), "inputs")
    _require_exact(metadata, ("session", "packages", "source"), "metadata")
    cases = data["cases"]
    _require_exact(cases, ("subset", "full"), "cases")
    paths = set(inputs.values()) | set(metadata.values())
    for name, case in cases.items():
        _require_exact(case, ("input_counts", "chromosomes", "reference_groups", "stages",
                              "additional_artifacts"), name)
        if case["input_counts"] != ("canonical_subset" if name == "subset" else "canonical_full"):
            raise ValueError("case raw input mapping mismatch")
        expected_chrs = ["chr1", "chr19", "chr21"] if name == "subset" else [f"chr{i}" for i in range(1, 23)]
        if case["chromosomes"] != expected_chrs or case["reference_groups"] != list(REFS):
            raise ValueError("case chromosome/reference settings mismatch")
        stages = case["stages"]
        _require_exact(stages, (str(x) for x in STAGES), f"{name} stages")
        for step in stages.values():
            _require_exact(step, ("expression", "gene_order", "cell_groups", "checkpoint", "genes", "cells"), "stage")
            if type(step["genes"]) is not int or type(step["cells"]) is not int or step["genes"] < 1 or step["cells"] != 184:
                raise ValueError("stage dimensions mismatch")
            paths.update(step[k] for k in ("expression", "gene_order", "cell_groups", "checkpoint"))
        extras = case["additional_artifacts"]
        if not isinstance(extras, list) or any(not isinstance(p, str) or not p.startswith(f"{name}/upstream/") for p in extras):
            raise ValueError("invalid additional upstream artifacts")
        paths.update(extras)
    expected_path_count = 9 + 2 * 10 * 4 + sum(len(case["additional_artifacts"]) for case in cases.values())
    if len(paths) != expected_path_count:
        raise ValueError("duplicate artifact path")
    hashes = data["sha256"]
    _require_exact(hashes, paths, "sha256")
    for rel in sorted(paths):
        require_sha256(owned_file(root, rel), hashes[rel])
    for key, expected in ORIGINAL_SHA256.items():
        if hashes[inputs[key]] != expected:
            raise ValueError(f"pinned input SHA-256 mismatch: {key}")
    if hashes[inputs["source_archive"]] != SOURCE_SHA256:
        raise ValueError("pinned source SHA-256 mismatch")
    actual = {p.relative_to(root).as_posix() for p in root.rglob("*") if p.is_file()}
    if actual != paths | {"oracle.json"} and actual != paths | {"oracle.json", "sha256-manifest.tsv"}:
        raise ValueError("unexpected or missing bundle artifact")
    _validate_metadata(root, metadata, SOURCE_SHA256)
    manifest = root / "sha256-manifest.tsv"
    if manifest.is_file() and manifest.read_bytes() != _manifest_text(root, hashes).encode("utf-8"):
        raise ValueError("existing manifest differs from preserved bundle bytes")
    original = read_original(owned_file(root, inputs["original_counts"]))
    full = read_canonical(owned_file(root, inputs["canonical_full"]))
    validate_canonical(original, full)
    coords, chr_rank = _original_order(owned_file(root, inputs["original_gene_order"]))
    annotations = _annotations(owned_file(root, inputs["original_annotations"]), original.cells)
    subset = read_canonical(owned_file(root, inputs["canonical_subset"]))
    subset_expected = tuple(g for g in original.genes if coords.get(g, (None,))[0] in {"chr1", "chr19", "chr21"})
    validate_subset(original, subset, subset_expected)
    summaries = {}
    for name, case in cases.items():
        prepared = subset if name == "subset" else full
        selected = set(case["chromosomes"])
        expected_genes = tuple(sorted((g for g in prepared.genes if g in coords and coords[g][0] in selected),
            key=lambda g: (chr_rank[coords[g][0]], int(coords[g][1]), int(coords[g][2]))))
        if set(coords[g][0] for g in expected_genes) != selected:
            raise ValueError("case chromosome coverage mismatch")
        previous = None
        summaries[name] = {}
        for number in STAGES:
            stage = case["stages"][str(number)]
            matrix = read_canonical(owned_file(root, stage["expression"]), allow_negative=number >= 8)
            if len(matrix.genes) != stage["genes"] or len(matrix.cells) != stage["cells"]:
                raise ValueError("stage actual dimensions mismatch")
            _identities(root, stage, matrix, coords, annotations)
            if number == 1:
                validate_stage1(original, matrix, expected_genes)
            elif number == 2:
                validate_stage2(previous, matrix)
            else:
                if matrix.genes != previous.genes or matrix.cells != previous.cells:
                    raise ValueError("downstream identity/order mismatch")
                if number == 3:
                    validate_arithmetic(previous, matrix, "depth")
                elif number == 4:
                    validate_arithmetic(previous, matrix, "log2")
                elif number == 9:
                    validate_arithmetic(previous, matrix, "clamp")
                elif number == 11:
                    validate_arithmetic(previous, matrix, "recenter")
                elif number == 14:
                    validate_arithmetic(previous, matrix, "inverse")
            summaries[name][str(number)] = [len(matrix.genes), len(matrix.cells)]
            previous = matrix
    return summaries


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("--manifest", action="store_true")
    args = parser.parse_args()
    summary = validate_bundle(args.root)
    if args.manifest:
        manifest = args.root / "sha256-manifest.tsv"
        if not manifest.exists():
            data = json.loads((args.root / "oracle.json").read_text(encoding="utf-8"))
            manifest.write_text(_manifest_text(args.root, data["sha256"]), encoding="utf-8")
    print(json.dumps(summary, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, TypeError, KeyError) as exc:
        print(f"shipped oracle validation failed: {exc}", file=sys.stderr)
        sys.exit(1)
