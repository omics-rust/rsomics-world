"""Check exact creation exports independently of timing and provenance gates."""

import csv
import math
from pathlib import Path
import re
import struct

from validate_infercnv_samples_clustering_witness import (
    checked_file, digest, finite, validate_maps,
)


OUTPUTS = ("expression.tsv", "genes.tsv", "cells.tsv", "maps.tsv")
HEADERS = (("gene",), ("gene", "chr", "start", "stop"),
           ("cell", "group", "role"), ("role", "group", "index", "cell"))
NUMBER = re.compile(r"[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?\Z")


def _literal_table(path, header):
    with path.open(newline="", encoding="utf-8") as source:
        rows = list(csv.reader(source, delimiter="\t", quoting=csv.QUOTE_NONE))
    if len(rows) < 2 or rows[0][:len(header)] != list(header) or \
            any(len(row) != len(rows[0]) for row in rows[1:]):
        raise ValueError("invalid literal creation table: " + str(path))
    return rows


def _state(paths):
    expression, genes, cells, maps = [_literal_table(path, header)
                                    for path, header in zip(paths, HEADERS)]
    if any(rows[0] != list(header) for rows, header in zip((genes, cells, maps), HEADERS[1:])):
        raise ValueError("creation table header differs")
    gene_ids = [row[0] for row in genes[1:]]
    cell_ids = [row[0] for row in cells[1:]]
    if any(not name for name in gene_ids + cell_ids) or len(set(gene_ids)) != len(gene_ids) or \
            len(set(cell_ids)) != len(cell_ids) or expression[0] != ["gene"] + cell_ids or \
            [row[0] for row in expression[1:]] != gene_ids:
        raise ValueError("creation matrix identity differs")
    for _, chromosome, start, stop in genes[1:]:
        if not chromosome or not start.isascii() or not stop.isascii() or \
                not start.isdecimal() or not stop.isdecimal() or int(start) > int(stop) or \
                int(stop) > 2**64 - 1 or (int(start), int(stop)) == (0, 0):
            raise ValueError("invalid creation gene coordinates")
    validate_maps(cells[1:], maps[1:])
    values = []
    for row in expression[1:]:
        numbers = []
        for raw in row[1:]:
            text = raw.strip(" \t\r\n\v\f")
            if NUMBER.fullmatch(text) is None:
                raise ValueError("invalid creation count spelling")
            value = finite(text, "creation count")
            if value < 0:
                raise ValueError("negative creation count")
            numbers.append(value)
        values.append(numbers)
    for column in zip(*values):
        try:
            total = math.fsum(column)
        except OverflowError as error:
            raise ValueError("creation depth overflows") from error
        if not math.isfinite(total) or total <= 0:
            raise ValueError("creation cell depth is not positive finite")
    bits = [[struct.pack("!d", value) for value in row] for row in values]
    return expression, genes, cells, maps, bits


def validate_creation_export(result_dir: Path, full_dir: Path, maps_path: Path) -> dict:
    result_dir, full_dir, maps_path = map(Path, (result_dir, full_dir, maps_path))
    if any(path.is_symlink() for path in (result_dir, full_dir, maps_path.parent)):
        raise ValueError("symlink creation root")
    entries = list(result_dir.rglob("*"))
    if any(path.is_symlink() or not path.is_file() for path in entries):
        raise ValueError("nonregular creation output entry")
    actual_files = {path.relative_to(result_dir).as_posix() for path in entries}
    if actual_files != set(OUTPUTS):
        raise ValueError("creation output inventory differs")
    actual_paths = [checked_file(result_dir, name) for name in OUTPUTS]
    expected_paths = [checked_file(full_dir, name)
                      for name in ("01.tsv", "01.genes.tsv", "01.cells.tsv")]
    expected_paths.append(checked_file(maps_path.parent, maps_path.name))
    actual, expected = _state(actual_paths), _state(expected_paths)
    if actual[0][0] != expected[0][0] or \
            [row[0] for row in actual[0][1:]] != [row[0] for row in expected[0][1:]] or \
            actual[1:] != expected[1:]:
        raise ValueError("creation state differs from accepted stage 1")
    return {"genes": len(actual[1]) - 1, "cells": len(actual[2]) - 1,
            "output_sha256": {name: digest(path) for name, path in zip(OUTPUTS, actual_paths)}}


def validate_factory_results(trial_dir: Path, full_dir: Path, maps_path: Path) -> dict:
    trial_dir = Path(trial_dir)
    if trial_dir.is_symlink():
        raise ValueError("symlink factory trial root")
    return {phase: validate_creation_export(trial_dir / phase, full_dir, maps_path)
            for phase in ("warm", "measured")}
