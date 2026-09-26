"""Focused unit fixtures for shipped-input and checkpoint validation.

These tiny matrices are test fixtures, never inferred upstream output.
"""

import gzip
import hashlib
import importlib.util
import io
import json
import math
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch


SCRIPT = Path(__file__).with_name("validate_infercnv_shipped_oracle.py")


def checker():
    spec = importlib.util.spec_from_file_location("shipped_checker", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ShippedInputTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.original = self.root / "original.gz"
        self.canonical = self.root / "canonical.tsv"
        self.original.write_bytes(gzip.compress(b"c1\tc2\tc3\nG1\t1.5\t2\t0\nG2\t0\t3\t4\n"))
        self.canonical.write_text("gene\tc1\tc2\tc3\nG1\t1.5\t2\t0\nG2\t0\t3\t4\n")

    def test_original_dialect_accepts_184_header_and_185_field_rows(self):
        cells = [f"c{i}" for i in range(184)]
        self.original.write_bytes(gzip.compress(("\t".join(cells) + "\n" +
            "G1\t" + "\t".join(["0.5"] * 184) + "\n").encode()))
        parsed = checker().read_original(self.original, expected_cells=184)
        self.assertEqual(parsed.cells, tuple(cells))
        self.assertEqual(parsed.genes, ("G1",))

    def test_original_dialect_rejects_malformed_width(self):
        self.original.write_bytes(gzip.compress(b"c1\tc2\tc3\nG1\t1\t2\n"))
        with self.assertRaisesRegex(ValueError, "width"):
            checker().read_original(self.original, expected_cells=3)

    def test_original_dialect_rejects_duplicate_or_missing_id(self):
        for body in (b"c1\tc1\tc3\nG1\t1\t2\t3\n",
                     b"c1\tc2\tc3\nG1\t1\t2\t3\nG1\t1\t2\t3\n",
                     b"c1\t\tc3\nG1\t1\t2\t3\n"):
            with self.subTest(body=body):
                self.original.write_bytes(gzip.compress(body))
                with self.assertRaisesRegex(ValueError, "ID"):
                    checker().read_original(self.original, expected_cells=3)

    def test_original_dialect_rejects_nonfinite_and_negative_counts(self):
        for value in ("nan", "inf", "-0.01"):
            with self.subTest(value=value):
                self.original.write_bytes(gzip.compress(
                    f"c1\tc2\tc3\nG1\t{value}\t2\t3\n".encode()))
                with self.assertRaisesRegex(ValueError, "count"):
                    checker().read_original(self.original, expected_cells=3)

    def test_canonical_rejects_changed_count(self):
        self.canonical.write_text("gene\tc1\tc2\tc3\nG1\t1.6\t2\t0\nG2\t0\t3\t4\n")
        with self.assertRaisesRegex(ValueError, "canonical"):
            checker().validate_canonical(checker().read_original(self.original, 3),
                                         checker().read_canonical(self.canonical, 3))

    def test_original_to_canonical_bridge_accepts_one_adjacent_value(self):
        self.original.write_bytes(gzip.compress(b"c1\tc2\tc3\nG1\t0.076439\t2\t3\n"))
        self.canonical.write_text("gene\tc1\tc2\tc3\nG1\t0.076439000000000007\t2\t3\n")
        module = checker()
        module.validate_canonical(module.read_original(self.original, 3),
                                  module.read_canonical(self.canonical, 3))

    def test_original_to_canonical_bridge_rejects_nonadjacent_value(self):
        self.original.write_bytes(gzip.compress(b"c1\tc2\tc3\nG1\t0.076439\t2\t3\n"))
        far = math.nextafter(math.nextafter(float("0.076439"), math.inf), math.inf)
        self.canonical.write_text(f"gene\tc1\tc2\tc3\nG1\t{far!r}\t2\t3\n")
        module = checker()
        with self.assertRaisesRegex(ValueError, "canonical"):
            module.validate_canonical(module.read_original(self.original, 3),
                                      module.read_canonical(self.canonical, 3))

    def test_downstream_expression_accepts_finite_negative_centered_values(self):
        self.canonical.write_text("gene\tc1\tc2\tc3\nG1\t-0.5\t2\t0\n")
        self.assertEqual(checker().read_canonical(self.canonical, 3, allow_negative=True).rows[0][0], -0.5)

    def test_downstream_expression_rejects_nonfinite_values(self):
        self.canonical.write_text("gene\tc1\tc2\tc3\nG1\tNaN\t2\t0\n")
        with self.assertRaisesRegex(ValueError, "nonfinite"):
            checker().read_canonical(self.canonical, 3, allow_negative=True)

    def test_stage1_and_stage2_reject_changed_values_and_membership(self):
        module = checker()
        original = module.read_original(self.original, 3)
        stage1 = module.read_canonical(self.canonical, 3)
        module.validate_stage1(original, stage1, ("G1", "G2"))
        self.canonical.write_text("gene\tc1\tc2\tc3\nG1\t1.5\t2\t0\nG2\t0\t3\t5\n")
        with self.assertRaisesRegex(ValueError, "stage 1"):
            module.validate_stage1(original, module.read_canonical(self.canonical, 3), ("G1", "G2"))
        self.canonical.write_text("gene\tc1\tc2\tc3\nG1\t1.5\t2\t0\nG2\t0\t3\t4\n")
        with self.assertRaisesRegex(ValueError, "stage 2"):
            module.validate_stage2(stage1, module.read_canonical(self.canonical, 3), 2, 3)
        filtered = self.root / "filtered.tsv"
        filtered.write_text("gene\tc1\tc2\tc3\nG1\t1.5\t2\t0\n")
        with self.assertRaisesRegex(ValueError, "stage 2"):
            module.validate_stage2(stage1, module.read_canonical(filtered, 3), 1, 2)

    def test_stage2_rejects_changed_retained_value(self):
        module = checker()
        stage1_file = self.root / "three-positive.tsv"
        stage2_file = self.root / "changed-retained.tsv"
        stage1_file.write_text("gene\tc1\tc2\tc3\nKEEP\t2\t3\t4\nDROP\t0\t0\t5\n")
        stage2_file.write_text("gene\tc1\tc2\tc3\nKEEP\t2\t3\t4\n")
        stage1 = module.read_canonical(stage1_file, 3)
        module.validate_stage2(stage1, module.read_canonical(stage2_file, 3))
        stage2_file.write_text("gene\tc1\tc2\tc3\nKEEP\t2\t3\t4.1\n")
        with self.assertRaisesRegex(ValueError, "stage 2 retained value changed"):
            module.validate_stage2(stage1, module.read_canonical(stage2_file, 3))

    def test_postcanonical_subset_stage1_stage2_reject_one_ulp_changes(self):
        module = checker()
        original = module.read_original(self.original, 3)
        canonical = module.read_canonical(self.canonical, 3)
        changed = self.root / "one-ulp.tsv"
        changed.write_text("gene\tc1\tc2\tc3\nG1\t1.5000000000000002\t2\t0\nG2\t0\t3\t4\n")
        shifted = module.read_canonical(changed, 3)
        with self.assertRaisesRegex(ValueError, "subset"):
            module.validate_subset(original, shifted, original.genes)
        with self.assertRaisesRegex(ValueError, "stage 1"):
            module.validate_stage1(canonical, shifted, canonical.genes)
        with self.assertRaisesRegex(ValueError, "stage 2"):
            module.validate_stage2(canonical, shifted, cutoff=0, min_cells=1)

    def test_subset_rejects_wrong_gene_membership(self):
        module = checker()
        original = module.read_original(self.original, 3)
        self.canonical.write_text("gene\tc1\tc2\tc3\nG2\t0\t3\t4\n")
        with self.assertRaisesRegex(ValueError, "subset"):
            module.validate_subset(original, module.read_canonical(self.canonical, 3), ("G1",))

    def test_stage_identity_rejects_changed_coordinate_and_group(self):
        module = checker()
        matrix = module.read_canonical(self.canonical, 3)
        gene_file = self.root / "genes.tsv"
        cell_file = self.root / "cells.tsv"
        entry = {"gene_order": "genes.tsv", "cell_groups": "cells.tsv"}
        gene_file.write_text("gene\tchr\tstart\tstop\nG1\tchr1\t1\t2\nG2\tchr1\t3\t4\n")
        cell_file.write_text("cell\tgroup\trole\nc1\tobs\tobservation\nc2\tobs\tobservation\nc3\tobs\tobservation\n")
        coords = {"G1": ("chr1", "1", "2"), "G2": ("chr1", "3", "4")}
        annotations = {"c1": "obs", "c2": "obs", "c3": "obs"}
        module._identities(self.root, entry, matrix, coords, annotations)
        gene_file.write_text("gene\tchr\tstart\tstop\nG1\tchr1\t1\t9\nG2\tchr1\t3\t4\n")
        with self.assertRaisesRegex(ValueError, "coordinate"):
            module._identities(self.root, entry, matrix, coords, annotations)
        gene_file.write_text("gene\tchr\tstart\tstop\nG1\tchr1\t1\t2\nG2\tchr1\t3\t4\n")
        cell_file.write_text("cell\tgroup\trole\nc1\twrong\tobservation\nc2\tobs\tobservation\nc3\tobs\tobservation\n")
        with self.assertRaisesRegex(ValueError, "group"):
            module._identities(self.root, entry, matrix, coords, annotations)

    def test_arithmetic_rejects_wrong_depth_log_and_inverse(self):
        module = checker()
        left = module.read_canonical(self.canonical, 3)
        right = self.root / "right.tsv"
        right.write_text("gene\tc1\tc2\tc3\nG1\t1.5\t2\t0\nG2\t0\t3\t4\n")
        same = module.read_canonical(right, 3)
        for operation in ("depth", "log2", "inverse", "clamp", "recenter"):
            with self.subTest(operation=operation):
                with self.assertRaisesRegex(ValueError, f"{operation} arithmetic mismatch"):
                    module.validate_arithmetic(left, same, operation)

    def test_path_and_hash_guards(self):
        module = checker()
        with self.assertRaisesRegex(ValueError, "path"):
            module.owned_file(self.root, "../escape")
        with self.assertRaisesRegex(ValueError, "missing"):
            module.owned_file(self.root, "missing.tsv")
        with self.assertRaisesRegex(ValueError, "SHA-256"):
            module.require_sha256(self.canonical, "0" * 64)

    def test_cli_missing_bundle_fails(self):
        result = subprocess.run([sys.executable, "-B", str(SCRIPT), str(self.root)],
                                capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("oracle.json", result.stderr)


class ShippedBundleUnitTests(unittest.TestCase):
    """Self-contained synthetic schema fixtures, not real upstream outputs."""

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.module = checker()
        self.cells = [f"c{i:03d}" for i in range(184)]
        self.genes = [f"G{i:02d}" for i in range(1, 23)]
        self._build_bundle()

    def _write(self, rel, contents):
        path = self.root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(contents if isinstance(contents, bytes) else contents.encode())
        return rel

    def _expression(self, genes, value):
        return ("gene\t" + "\t".join(self.cells) + "\n" + "".join(
            gene + "\t" + "\t".join([repr(value)] * 184) + "\n" for gene in genes))

    def _rehash(self, rel):
        self.record["sha256"][rel] = hashlib.sha256((self.root / rel).read_bytes()).hexdigest()
        self._save_record()

    def _save_record(self):
        (self.root / "oracle.json").write_text(json.dumps(self.record), encoding="utf-8")

    def _pins(self):
        return (patch.object(self.module, "SOURCE_SHA256", self.fixture_source_sha),
                patch.object(self.module, "ORIGINAL_SHA256", self.fixture_original_sha),
                patch.object(self.module, "CANONICAL_SHA256", self.fixture_canonical_sha, create=True))

    def _validate_fixture(self):
        source_patch, original_patch, canonical_patch = self._pins()
        with source_patch, original_patch, canonical_patch:
            return self.module.validate_bundle(self.root)

    def _build_bundle(self):
        inputs = {
            "source_archive": "source/infercnv-source.tar.gz",
            "original_counts": "inputs/original.gz",
            "original_annotations": "inputs/annotations.tsv",
            "original_gene_order": "inputs/coordinates.tsv",
            "canonical_full": "inputs/full.tsv",
            "canonical_subset": "inputs/subset.tsv",
        }
        subset_genes = ("G01", "G19", "G21")
        raw = "\t".join(self.cells) + "\n" + "".join(
            gene + "\t" + "\t".join(["2"] * 184) + "\n" for gene in self.genes)
        self._write(inputs["source_archive"], b"test-only archive bytes")
        self._write(inputs["original_counts"], gzip.compress(raw.encode()))
        groups = ([self.module.REFS[0]] * 19 + [self.module.REFS[1]] * 23 +
                  ["obs1"] * 35 + ["obs2"] * 33 + ["obs3"] * 34 + ["obs4"] * 40)
        self._write(inputs["original_annotations"], "".join(
            f"{cell}\t{group}\n" for cell, group in zip(self.cells, groups)))
        self._write(inputs["original_gene_order"], "".join(
            f"{gene}\tchr{i}\t1\t2\n" for i, gene in enumerate(self.genes, 1)))
        self._write(inputs["canonical_full"], self._expression(self.genes, 2))
        self._write(inputs["canonical_subset"], self._expression(subset_genes, 2))
        metadata = {"session": "metadata/session.txt", "packages": "metadata/packages.tsv",
                    "source": "metadata/source.txt"}
        self._write(metadata["session"], "R version 4.6.1\nPlatform: x86_64-pc-linux-gnu\n"
                    "BLAS: /usr/lib/blas.so\nLAPACK: /usr/lib/lapack.so\n"
                    "loaded via a namespace (and not attached):\n[1] infercnv_1.28.0\n")
        self._write(metadata["packages"], "Package\tVersion\tLibPath\n"
                    "infercnv\t1.28.0\t/runner/R/library\n")
        fixture_source_sha = hashlib.sha256((self.root / inputs["source_archive"]).read_bytes()).hexdigest()
        self._write(metadata["source"], f"source_commit\t{self.module.COMMIT}\n"
                    f"source_archive_sha256\t{fixture_source_sha}\n"
                    "source_url\thttps://github.com/bioconductor-source/infercnv\n")
        cases = {}
        for name, genes in (("subset", subset_genes), ("full", self.genes)):
            stages = {}
            for step in self.module.STAGES:
                prefix = f"{name}/{step:02d}"
                if step in (1, 2, 3):
                    value = 2
                elif step in (4, 8, 9, 10):
                    value = math.log2(3)
                elif step in (11, 12):
                    value = 0
                else:
                    value = 1
                self._write(prefix + ".tsv", self._expression(genes, value))
                self._write(prefix + ".genes.tsv", "gene\tchr\tstart\tstop\n" + "".join(
                    f"{gene}\tchr{int(gene[1:])}\t1\t2\n" for gene in genes))
                self._write(prefix + ".cells.tsv", "cell\tgroup\trole\n" + "".join(
                    f"{cell}\t{group}\t{'reference' if group in self.module.REFS else 'observation'}\n"
                    for cell, group in zip(self.cells, groups)))
                self._write(f"{name}/upstream/{step:02d}.infercnv_obj", b"test-only RDS placeholder")
                stages[str(step)] = {"expression": prefix + ".tsv",
                                     "gene_order": prefix + ".genes.tsv",
                                     "cell_groups": prefix + ".cells.tsv",
                                     "checkpoint": f"{name}/upstream/{step:02d}.infercnv_obj",
                                     "genes": len(genes), "cells": 184}
            cases[name] = {"input_counts": "canonical_subset" if name == "subset" else "canonical_full",
                           "chromosomes": ["chr1", "chr19", "chr21"] if name == "subset" else
                                          [f"chr{i}" for i in range(1, 23)],
                           "reference_groups": list(self.module.REFS), "stages": stages,
                           "additional_artifacts": []}
        paths = [p for p in self.root.rglob("*") if p.is_file()]
        hashes = {p.relative_to(self.root).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
        self.record = {"schema_version": 2,
                       "package": {"name": "infercnv", "version": "1.28.0",
                                   "source_commit": self.module.COMMIT,
                                   "source_archive_sha256": fixture_source_sha},
                       "inputs": inputs, "metadata": metadata,
                       "creation_settings": self.module.CREATION_SETTINGS.copy(),
                       "run_settings": self.module.RUN_SETTINGS.copy(),
                       "cases": cases, "sha256": hashes}
        self._save_record()
        self.fixture_source_sha = hashes[inputs["source_archive"]]
        self.fixture_original_sha = {key: hashes[inputs[key]] for key in self.module.ORIGINAL_SHA256}
        self.fixture_canonical_sha = {key: hashes[inputs[key]] for key in ("canonical_full", "canonical_subset")}

    def test_valid_unit_bundle_exercises_full_schema(self):
        self.assertEqual(self._validate_fixture()["subset"]["1"], [3, 184])

    def test_trusted_adjacent_bridge_passes_full_schema(self):
        adjacent = repr(math.nextafter(2.0, math.inf))
        for rel in ("inputs/full.tsv", "inputs/subset.tsv",
                    "subset/01.tsv", "subset/02.tsv", "full/01.tsv", "full/02.tsv"):
            path = self.root / rel
            path.write_text(path.read_text().replace("G01\t2\t", f"G01\t{adjacent}\t", 1))
            self._rehash(rel)
        inputs = self.record["inputs"]
        self.fixture_canonical_sha = {key: self.record["sha256"][inputs[key]]
                                      for key in ("canonical_full", "canonical_subset")}
        self.assertEqual(self._validate_fixture()["subset"]["1"], [3, 184])

    def test_self_reported_one_ulp_canonical_change_fails_trusted_pin(self):
        rel = self.record["inputs"]["canonical_full"]
        path = self.root / rel
        path.write_text(path.read_text().replace("G01\t2\t", "G01\t2.0000000000000004\t", 1))
        self._rehash(rel)
        with self.assertRaisesRegex(ValueError, "canonical.*SHA-256"):
            self._validate_fixture()

    def test_missing_checkpoint_from_valid_bundle_fails(self):
        rel = self.record["cases"]["subset"]["stages"]["14"]["checkpoint"]
        (self.root / rel).unlink()
        with self.assertRaisesRegex(ValueError, "missing referenced file"):
            self._validate_fixture()

    def test_rehashed_false_source_metadata_fails(self):
        rel = self.record["metadata"]["source"]
        self._write(rel, "source_commit\tfalse\n")
        self._rehash(rel)
        with self.assertRaisesRegex(ValueError, "source metadata"):
            self._validate_fixture()

    def test_rehashed_false_package_metadata_fails(self):
        rel = self.record["metadata"]["packages"]
        self._write(rel, "Package\tVersion\tLibPath\ninfercnv\t0.0.0\t/runner/R/library\n")
        self._rehash(rel)
        with self.assertRaisesRegex(ValueError, "package metadata"):
            self._validate_fixture()

    def test_same_package_in_distinct_libraries_is_valid(self):
        rel = self.record["metadata"]["packages"]
        self._write(rel, "Package\tVersion\tLibPath\n"
                    "infercnv\t1.28.0\t/runner/R/library\n"
                    "Matrix\t1.7-6\t/runner/R/library\n"
                    "Matrix\t1.7-5\t/opt/R/library\n")
        self._rehash(rel)
        self.assertEqual(self._validate_fixture()["full"]["1"], [22, 184])

    def test_same_package_and_library_duplicate_is_invalid(self):
        rel = self.record["metadata"]["packages"]
        self._write(rel, "Package\tVersion\tLibPath\n"
                    "infercnv\t1.28.0\t/runner/R/library\n"
                    "Matrix\t1.7-6\t/runner/R/library\n"
                    "Matrix\t1.7-5\t/runner/R/library\n")
        self._rehash(rel)
        with self.assertRaisesRegex(ValueError, "package metadata"):
            self._validate_fixture()

    def test_older_installed_infercnv_does_not_override_loaded_version(self):
        rel = self.record["metadata"]["packages"]
        for versions in (("1.28.0", "1.27.0"), ("1.27.0", "1.28.0")):
            with self.subTest(versions=versions):
                self._write(rel, "Package\tVersion\tLibPath\n" +
                            f"infercnv\t{versions[0]}\t/runner/R/library\n" +
                            f"infercnv\t{versions[1]}\t/opt/R/library\n")
                self._rehash(rel)
                self.assertEqual(self._validate_fixture()["subset"]["1"], [3, 184])

    def test_rehashed_empty_runtime_metadata_fails(self):
        rel = self.record["metadata"]["session"]
        self._write(rel, "")
        self._rehash(rel)
        with self.assertRaisesRegex(ValueError, "runtime metadata"):
            self._validate_fixture()

    def test_real_source_and_input_pins_reject_unit_fixture(self):
        with self.assertRaisesRegex(ValueError, "pinned source/package identity"):
            self.module.validate_bundle(self.root)
        source_patch, original_patch, canonical_patch = self._pins()
        with source_patch:
            with self.assertRaisesRegex(ValueError, "pinned input SHA-256"):
                self.module.validate_bundle(self.root)

    def test_stale_existing_manifest_fails_without_overwrite(self):
        manifest = self.root / "sha256-manifest.tsv"
        manifest.write_text("stale manifest\n")
        source_patch, original_patch, canonical_patch = self._pins()
        with source_patch, original_patch, canonical_patch:
            with self.assertRaisesRegex(ValueError, "manifest"):
                self.module.validate_bundle(self.root)
            with patch.object(sys, "argv", [str(SCRIPT), str(self.root), "--manifest"]):
                with self.assertRaisesRegex(ValueError, "manifest"):
                    with redirect_stdout(io.StringIO()):
                        self.module.main()
        self.assertEqual(manifest.read_text(), "stale manifest\n")

    def test_manifest_byte_change_is_not_normalized_away(self):
        source_patch, original_patch, canonical_patch = self._pins()
        with source_patch, original_patch, canonical_patch:
            with patch.object(sys, "argv", [str(SCRIPT), str(self.root), "--manifest"]):
                with redirect_stdout(io.StringIO()):
                    self.module.main()
            manifest = self.root / "sha256-manifest.tsv"
            manifest.write_bytes(manifest.read_bytes().replace(b"\n", b"\r\n"))
            with self.assertRaisesRegex(ValueError, "manifest"):
                self.module.validate_bundle(self.root)


if __name__ == "__main__":
    unittest.main()
