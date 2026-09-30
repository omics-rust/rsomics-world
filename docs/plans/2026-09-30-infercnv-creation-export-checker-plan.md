# Exact creation export checker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Independently reject any incorrect or missing warm/measured creation
export before a raw-ingestion timing can become a sample.

**Architecture:** A controller-only checker reuses existing safe-path and
grouped-map checks, reads literal TSV without CSV quote interpretation, then compares count binary64 bits and ordered identity
tables to the accepted stage-1 fixture. It verifies both factory results and
returns separate hashes, without assigning timings or performance claims.

**Tech Stack:** Python standard library, unittest, existing oracle helpers.

**Spec:** `2026-09-30-infercnv-raw-ingestion-measurement-design.md`.

## Global Constraints

- Owner: `/Volumes/Zane's HDD/Documents/rsomics-world`.
- Controller scratch: `/Volumes/KIOXIA/Developments/tmp`; Python bytecode off.
- No Cargo/R builds or downloads on the Mac boot disk, currently over 80%.
- Do not change production Rust, old prepared measurement code or receipts.
- Output counts compare exact binary64 bits; no tolerance or Unicode padding.
- IDs, roles, coordinates and maps compare literally and in source order.
- Validate warm and measured separately; missing or invalid results fail.
- Input/source/receipt pinning and process metrics belong to the subsequent
  driver, not this checker. No performance-ready or release claim here.

## Review Focus

- Adjacent floats, signed zero and malformed numeric spelling: exact rejection.
- Joint role/map mutation: reject against the independent expected fixture.
- Reference or member reordering: reject even if membership sets are unchanged.
- Missing warm/measured, extra files or escaped paths: reject complete trial.
- Numeric padding versus identity padding: allow only numeric ASCII padding.

## Task 1: Independent exact state and complete-trial validation

**Files:**

- Create: `scripts/validate_infercnv_ingestion_measurement.py`.
- Create: `scripts/test_validate_infercnv_ingestion_measurement.py`.
- Read unchanged: `scripts/validate_infercnv_samples_clustering_witness.py`.

**Interfaces:**

- `validate_creation_export(result_dir: Path, full_dir: Path, maps_path: Path)
  -> dict`: consumes `expression.tsv`, `genes.tsv`, `cells.tsv`, `maps.tsv`,
  expected `01.tsv`, `01.genes.tsv`, `01.cells.tsv` and witnessed maps;
  produces dimensions plus four actual output hashes.
- `validate_factory_results(trial_dir: Path, full_dir: Path, maps_path: Path)
  -> dict`: consumes exactly those four files in each of `warm/`, `measured/`;
  produces separate validated results under the same names.
- Existing `checked_file`, `finite`, `validate_maps`, `digest` retain
  their existing signatures; imports do not invoke package work.

- [x] **Step 1: Add these executable failing tests before adding the checker.**

```python
import importlib.util
import math
from pathlib import Path
import shutil
import tempfile
import unittest

class CreationTests(unittest.TestCase):
    def setUp(self):
        path = Path(__file__).with_name("validate_infercnv_ingestion_measurement.py")
        self.assertTrue(path.is_file(), "creation measurement checker absent")
        spec = importlib.util.spec_from_file_location("creation_checker", path)
        self.checker = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.checker)
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        root = Path(self.temp.name)
        self.full, self.trial = root / "full", root / "trial"
        self.full.mkdir(); self.trial.mkdir()
        self.maps = root / "maps.tsv"
        self.payload = {
            "expression.tsv": "gene\tc_a\tc_obs\tc_b\ng1\t1\t2\t3\ng2\t4\t5\t6\n",
            "genes.tsv": "gene\tchr\tstart\tstop\ng1\tchr1\t1\t2\ng2\tchr2\t3\t4\n",
            "cells.tsv": "cell\tgroup\trole\nc_a\tr_a\treference\nc_obs\tobs\tobservation\nc_b\tr_b\treference\n",
            "maps.tsv": "role\tgroup\tindex\tcell\nreference\tr_b\t3\tc_b\nreference\tr_a\t1\tc_a\nobservation\tobs\t2\tc_obs\n",
        }
        for name, source in (("01.tsv", "expression.tsv"),
                             ("01.genes.tsv", "genes.tsv"),
                             ("01.cells.tsv", "cells.tsv")):
            (self.full / name).write_text(self.payload[source])
        self.maps.write_text(self.payload["maps.tsv"])
        for phase in ("warm", "measured"):
            directory = self.trial / phase
            directory.mkdir()
            for name, text in self.payload.items():
                (directory / name).write_text(text)

    def validate(self):
        return self.checker.validate_factory_results(self.trial, self.full, self.maps)

    def test_both_results_and_numeric_ascii_padding(self):
        path = self.trial / "warm" / "expression.tsv"
        path.write_text(self.payload["expression.tsv"].replace("g1\t1\t", "g1\t 1.0000000000000000 \t"))
        result = self.validate()
        self.assertEqual(set(result), {"warm", "measured"})
        for value in result.values():
            self.assertEqual((value["genes"], value["cells"]), (2, 3))
            self.assertEqual(set(value["output_sha256"]), set(self.payload))

    def test_bad_numeric_values_in_either_factory_result(self):
        for phase in ("warm", "measured"):
            path = self.trial / phase / "expression.tsv"
            for value in (str(math.nextafter(1.0, math.inf)), "NaN", "Inf", "-1", "1_0", "\u00a01"):
                with self.subTest(phase=phase, value=value):
                    path.write_text(self.payload["expression.tsv"].replace("g1\t1\t", "g1\t" + value + "\t"))
                    with self.assertRaises(ValueError): self.validate()
            path.write_text(self.payload["expression.tsv"])

    def test_role_and_joint_map_mutation(self):
        directory = self.trial / "measured"
        (directory / "cells.tsv").write_text(self.payload["cells.tsv"].replace("c_a\tr_a\treference", "c_a\tr_a\tobservation"))
        (directory / "maps.tsv").write_text(self.payload["maps.tsv"].replace("reference\tr_a", "observation\tr_a"))
        with self.assertRaises(ValueError): self.validate()

    def test_map_order_membership_and_identity_mutations(self):
        path = self.trial / "warm" / "maps.tsv"
        rows = self.payload["maps.tsv"].splitlines(keepends=True)
        mutations = [rows[0] + rows[2] + rows[1] + rows[3],
                     self.payload["maps.tsv"].replace("\t3\tc_b", "\t1\tc_b"),
                     self.payload["maps.tsv"].replace("\t3\tc_b", "\t4\tc_b"),
                     "".join(rows[:-1]), "".join(rows + rows[1:2]),
                     self.payload["maps.tsv"].replace("c_b", " c_b")]
        for text in mutations:
            path.write_text(text)
            with self.assertRaises(ValueError): self.validate()

    def test_missing_phase_or_unrecorded_output(self):
        shutil.rmtree(self.trial / "warm")
        with self.assertRaises((ValueError, OSError)): self.validate()
        (self.trial / "warm").mkdir()
        for name, text in self.payload.items():
            (self.trial / "warm" / name).write_text(text)
        (self.trial / "measured" / "extra").write_text("unexpected")
        with self.assertRaises(ValueError): self.validate()

if __name__ == "__main__":
    unittest.main()
```

- [x] **Step 2: Run and retain the expected missing-checker red failure.**

```bash
TMPDIR=/Volumes/KIOXIA/Developments/tmp PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s scripts -p test_validate_infercnv_ingestion_measurement.py
```

- [x] **Step 3: Implement this complete checker; keep its imports internal.**

```python
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
    expression, genes, cells, maps = [_literal_table(path, header) for path, header in zip(paths, HEADERS)]
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
            if value < 0: raise ValueError("negative creation count")
            numbers.append(value)
        values.append(numbers)
    for column in zip(*values):
        try: total = math.fsum(column)
        except OverflowError as error: raise ValueError("creation depth overflows") from error
        if not math.isfinite(total) or total <= 0:
            raise ValueError("creation cell depth is not positive finite")
    return expression, genes, cells, maps, [[struct.pack("!d", value) for value in row] for row in values]

def validate_creation_export(result_dir: Path, full_dir: Path, maps_path: Path) -> dict:
    result_dir, full_dir, maps_path = map(Path, (result_dir, full_dir, maps_path))
    if any(path.is_symlink() for path in (result_dir, full_dir, maps_path.parent)):
        raise ValueError("symlink creation root")
    entries = list(result_dir.rglob("*"))
    if any(path.is_symlink() or not path.is_file() for path in entries):
        raise ValueError("non-file or symlink creation output")
    actual_files = {path.relative_to(result_dir).as_posix() for path in entries}
    if actual_files != set(OUTPUTS): raise ValueError("creation output inventory differs")
    actual_paths = [checked_file(result_dir, name) for name in OUTPUTS]
    expected_paths = [checked_file(full_dir, name) for name in ("01.tsv", "01.genes.tsv", "01.cells.tsv")]
    expected_paths.append(checked_file(maps_path.parent, maps_path.name))
    actual, expected = _state(actual_paths), _state(expected_paths)
    if actual[0][0] != expected[0][0] or [row[0] for row in actual[0][1:]] != [row[0] for row in expected[0][1:]] or \
            actual[1:] != expected[1:]:
        raise ValueError("creation state differs from accepted stage 1")
    return {"genes": len(actual[1]) - 1, "cells": len(actual[2]) - 1,
            "output_sha256": {name: digest(path) for name, path in zip(OUTPUTS, actual_paths)}}

def validate_factory_results(trial_dir: Path, full_dir: Path, maps_path: Path) -> dict:
    trial_dir = Path(trial_dir)
    if trial_dir.is_symlink(): raise ValueError("symlink factory trial root")
    return {phase: validate_creation_export(trial_dir / phase, full_dir, maps_path)
            for phase in ("warm", "measured")}
```

- [x] **Step 4: Add these fixture mutations and rerun the focused suite.**
  Insert methods inside CreationTests. Expected and actual zero are first
  confirmed accepted; only afterward does the signed-zero mutation occur.

```python
    def test_signed_zero_requires_the_same_binary64_bits(self):
        original = self.payload["expression.tsv"].replace("g1\t1\t", "g1\t0\t")
        (self.full / "01.tsv").write_text(original)
        for phase in ("warm", "measured"):
            (self.trial / phase / "expression.tsv").write_text(original)
        self.validate()
        (self.trial / "measured" / "expression.tsv").write_text(original.replace("g1\t0\t", "g1\t-0\t"))
        with self.assertRaises(ValueError): self.validate()

    def test_identity_shape_and_coordinate_mutations(self):
        mutations = {
            "expression.tsv": [
                self.payload["expression.tsv"].replace("c_a\tc_obs", "c_obs\tc_a"),
                "gene\tc_a\tc_obs\tc_b\ng2\t4\t5\t6\ng1\t1\t2\t3\n",
                "gene\tc_a\tc_obs\tc_b\ng1\t1\t2\t3\n",
                self.payload["expression.tsv"] + "g3\t7\t8\t9\n",
            ],
            "genes.tsv": [self.payload["genes.tsv"].replace("\t1\t2", "\t0\t2"),
                          self.payload["genes.tsv"].replace("g2", "g1"),
                          self.payload["genes.tsv"].replace("start", "begin")],
            "cells.tsv": [self.payload["cells.tsv"].replace("c_a", " c_a"),
                          self.payload["cells.tsv"].replace("c_b", "c_a")],
        }
        for name, changes in mutations.items():
            path = self.trial / "measured" / name
            for text in changes:
                with self.subTest(name=name, text=text):
                    path.write_text(text)
                    with self.assertRaises(ValueError): self.validate()
            path.write_text(self.payload[name])

    def test_missing_measured_and_escaped_output(self):
        shutil.rmtree(self.trial / "measured")
        with self.assertRaises((ValueError, OSError)): self.validate()
        (self.trial / "measured").mkdir()
        for name, text in self.payload.items():
            (self.trial / "measured" / name).write_text(text)
        path = self.trial / "measured" / "maps.tsv"
        path.unlink(); path.symlink_to(self.maps)
        with self.assertRaises(ValueError): self.validate()

    def test_member_order_is_not_only_a_membership_set(self):
        expression = "gene\tc_a\tc_obs\tc_b\tc_a2\ng1\t1\t2\t3\t7\ng2\t4\t5\t6\t8\n"
        cells = self.payload["cells.tsv"] + "c_a2\tr_a\treference\n"
        maps = self.payload["maps.tsv"].replace("observation\tobs", "reference\tr_a\t4\tc_a2\nobservation\tobs")
        (self.full / "01.tsv").write_text(expression)
        (self.full / "01.cells.tsv").write_text(cells)
        self.maps.write_text(maps)
        for phase in ("warm", "measured"):
            directory = self.trial / phase
            (directory / "expression.tsv").write_text(expression)
            (directory / "cells.tsv").write_text(cells)
            (directory / "maps.tsv").write_text(maps)
        self.validate()
        first, second = "reference\tr_a\t1\tc_a\n", "reference\tr_a\t4\tc_a2\n"
        (self.trial / "measured" / "maps.tsv").write_text(maps.replace(first + second, second + first))
        with self.assertRaises(ValueError): self.validate()

    def test_quotes_are_literal_identity_not_csv_escaping(self):
        path = self.trial / "measured" / "expression.tsv"
        path.write_text(self.payload["expression.tsv"].replace("\tc_a\t", '\t"c_a"\t'))
        with self.assertRaises(ValueError): self.validate()
        for name, text in self.payload.items():
            replacement = text.replace("c_a", '"c_a"')
            for phase in ("warm", "measured"):
                (self.trial / phase / name).write_text(replacement)
            if name == "maps.tsv": self.maps.write_text(replacement)
            else:
                expected = {"expression.tsv": "01.tsv", "genes.tsv": "01.genes.tsv", "cells.tsv": "01.cells.tsv"}[name]
                (self.full / expected).write_text(replacement)
        self.validate()
```
- [x] **Step 5: Run all controller tests and perform independent code review.**

```bash
TMPDIR=/Volumes/KIOXIA/Developments/tmp PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s scripts -p 'test_*.py'
git diff --check
```

- [ ] **Step 6: Commit only these files and the associated plan/state; push main
  and wait for exact-head Control plane CI before using the checker in another
  gate.** `test(sc): validate exact ingestion factory exports` is the commit
  subject. Keep inherited VCF state and unrelated scratch unstaged.

## Completion boundary

This delivers an exact controller check, not a timed factory or benchmark.
Subsequent private Rust export support must prove its own contract through
four-native debug/release CI. The actual R trial and paired driver must then
freeze all 32 processes/64 outputs and source/input/metrics provenance before
any ingestion-performance decision. Samples/Ward acceptance stays independent.
