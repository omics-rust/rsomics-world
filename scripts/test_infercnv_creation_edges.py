import importlib.util
import tempfile
import unittest
from pathlib import Path


CHECKER = Path(__file__).with_name("validate_infercnv_creation_edges.py")


class CreationEdgesTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue(CHECKER.exists(), "creation edge validator is not implemented")
        spec = importlib.util.spec_from_file_location("creation_edges", CHECKER)
        self.module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.module)
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.record = {"creation_arguments": {
            "raw_counts_matrix": "/remote/creation-edges/inputs/counts.tsv",
            "gene_order_file": "/remote/creation-edges/inputs/positions.tsv",
            "annotations_file": "/remote/creation-edges/inputs/annotations.tsv",
            "ref_group_names": ["ref"], "delim": "\t",
            "max_cells_per_group": None, "min_max_counts_per_cell": [3, 5],
            "chr_exclude": ["chrX", "chrY", "chrM"]}, "warnings": []}
        self.write_state()

    def write_state(self):
        (self.root / "expression.tsv").write_text(
            "gene\tmin\tmax\nC\t1\t1\nB\t1\t2\nA\t1\t2\nD\t0\t0\n")
        (self.root / "genes.tsv").write_text(
            "gene\tchr\tstart\tstop\nC\tchr10\t30\t40\n"
            "B\tchr2\t10\t20\nA\tchr2\t10\t20\nD\tchr1\t1\t2\n")
        (self.root / "cells.tsv").write_text(
            "cell\tgroup\trole\nmin\tref\treference\nmax\ta_obs\tobservation\n")
        (self.root / "maps.tsv").write_text(
            "role\tgroup\tindex\tcell\nreference\tref\t1\tmin\n"
            "observation\ta_obs\t2\tmax\n")

    def test_accepts_literal_inclusive_reduced_total_state(self):
        self.module.validate_tiny(self.root, "integer_limits", self.record)

    def test_rejects_changed_limit_even_with_correct_output(self):
        self.record["creation_arguments"]["min_max_counts_per_cell"] = [2, 5]
        with self.assertRaisesRegex(ValueError, "arguments"):
            self.module.validate_tiny(self.root, "integer_limits", self.record)

    def test_rejects_boundary_cell_replacement(self):
        path = self.root / "expression.tsv"
        path.write_text(path.read_text().replace("\tmax", "\thigh"))
        with self.assertRaisesRegex(ValueError, "expression"):
            self.module.validate_tiny(self.root, "integer_limits", self.record)

    def test_rejects_gene_id_tie_break(self):
        path = self.root / "genes.tsv"
        rows = path.read_text().splitlines()
        rows[2], rows[3] = rows[3], rows[2]
        path.write_text("\n".join(rows) + "\n")
        with self.assertRaisesRegex(ValueError, "genes"):
            self.module.validate_tiny(self.root, "integer_limits", self.record)

    def test_rejects_wrong_map_even_with_same_cell_table(self):
        path = self.root / "maps.tsv"
        path.write_text(path.read_text().replace("\t2\tmax", "\t1\tmax"))
        with self.assertRaisesRegex(ValueError, "maps"):
            self.module.validate_tiny(self.root, "integer_limits", self.record)

    def test_accepts_min_zero_and_null_clamp_one(self):
        (self.root / "expression.tsv").write_text(
            "gene\tlow\tmin\tmax\thigh\tunit\nC\t0\t1\t1\t2\t0\n"
            "B\t1\t1\t2\t2\t1\nA\t1\t1\t2\t2\t0\nD\t0\t0\t0\t0\t0\n")
        (self.root / "cells.tsv").write_text(
            "cell\tgroup\trole\nlow\ta_obs\tobservation\nmin\tref\treference\n"
            "max\ta_obs\tobservation\nhigh\tz_obs\tobservation\nunit\tz_obs\tobservation\n")
        (self.root / "maps.tsv").write_text(
            "role\tgroup\tindex\tcell\nreference\tref\t2\tmin\n"
            "observation\ta_obs\t1\tlow\nobservation\ta_obs\t3\tmax\n"
            "observation\tz_obs\t4\thigh\nobservation\tz_obs\t5\tunit\n")
        for label, limits in (("min_zero", [0, "Inf"]), ("min_null", None)):
            self.record["creation_arguments"]["min_max_counts_per_cell"] = limits
            self.module.validate_tiny(self.root, label, self.record)

    def test_rejects_missing_real_failure_message(self):
        with self.assertRaisesRegex(ValueError, "error"):
            self.module.validate_error("absent_annotation", {"error": "", "warnings": []})

    def test_rejects_error_record_that_claims_success(self):
        with self.assertRaisesRegex(ValueError, "error"):
            self.module.validate_error("absent_annotation", {"error": "some error", "warnings": [], "outputs": {}})

    def test_accepts_preserved_failure_and_source_hashes(self):
        source = self.root / "counts.tsv"
        source.write_text("gene\tmin\tmax\nA\t1\t2\n")
        record = {"error": "Please make sure that all the annotated cell names match a sample in your data matrix. Attention to: absent", "warnings": [],
                  "error_class": ["simpleError", "error", "condition"],
                  "error_call": "eval(expr, envir)",
                  "input_sha256": {"raw_counts_matrix": self.module.digest(source)},
                  "creation_arguments": {"raw_counts_matrix": "/remote/creation-edges/inputs/counts.tsv"}}
        self.module.validate_error("absent_annotation", record)
        suffixes = {"raw_counts_matrix": "creation-edges/inputs/counts.tsv"}
        self.module.validate_files(self.root, record, {"raw_counts_matrix": source}, suffixes)
        source.write_text("gene\tmin\tmax\nA\t9\t2\n")
        with self.assertRaisesRegex(ValueError, "input hash"):
            self.module.validate_files(self.root, record, {"raw_counts_matrix": source}, suffixes)

    def test_rejects_wrong_recorded_input_path(self):
        source = self.root / "counts.tsv"
        source.write_text("gene\tmin\tmax\nA\t1\t2\n")
        record = {"error": "missing annotation cell", "warnings": [],
                  "input_sha256": {"raw_counts_matrix": self.module.digest(source)},
                  "creation_arguments": {"raw_counts_matrix": "/elsewhere/other.tsv"}}
        with self.assertRaisesRegex(ValueError, "input arguments"):
            self.module.validate_files(self.root, record, {"raw_counts_matrix": source},
                                       {"raw_counts_matrix": "creation-edges/inputs/counts.tsv"})

    def test_rejects_same_basename_from_wrong_source_directory(self):
        source = self.root / "counts.tsv"
        source.write_text("gene\tmin\tmax\nA\t1\t2\n")
        record = {"error": "failure", "input_sha256": {"raw_counts_matrix": self.module.digest(source)},
                  "creation_arguments": {"raw_counts_matrix": "/remote/other/inputs/counts.tsv"}}
        with self.assertRaisesRegex(ValueError, "input arguments"):
            self.module.validate_files(self.root, record, {"raw_counts_matrix": source},
                                       {"raw_counts_matrix": "creation-edges/inputs/counts.tsv"})

    def test_rejects_noncanonical_and_relative_input_paths(self):
        source = self.root / "counts.tsv"
        source.write_text("gene\tmin\tmax\nA\t1\t2\n")
        for argument in ("creation-edges/inputs/counts.tsv", "/remote//creation-edges/inputs/counts.tsv",
                         "/remote/../creation-edges/inputs/counts.tsv"):
            record = {"error": "failure", "input_sha256": {"raw_counts_matrix": self.module.digest(source)},
                      "creation_arguments": {"raw_counts_matrix": argument}}
            with self.subTest(argument=argument), self.assertRaisesRegex(ValueError, "input arguments"):
                self.module.validate_files(self.root, record, {"raw_counts_matrix": source},
                                           {"raw_counts_matrix": "creation-edges/inputs/counts.tsv"})

    def test_rejects_extra_creation_argument(self):
        self.record["creation_arguments"]["unexpected_policy"] = True
        with self.assertRaisesRegex(ValueError, "arguments"):
            self.module.validate_tiny(self.root, "integer_limits", self.record)

    def test_rejects_untyped_warning(self):
        self.record["warnings"] = [42]
        with self.assertRaisesRegex(ValueError, "warning"):
            self.module.validate_tiny(self.root, "integer_limits", self.record)

    def test_rejects_unrelated_runtime_failure(self):
        record = {"error": "cannot open the connection", "warnings": [],
                  "error_class": ["simpleError", "error", "condition"],
                  "error_call": "file(file, 'rt')"}
        for label in ("absent_annotation", "removed_reference", "no_common_genes"):
            with self.subTest(label=label), self.assertRaisesRegex(ValueError, "error"):
                self.module.validate_error(label, record)

    def test_rejects_expected_message_from_unrelated_call(self):
        record = {"error": "invalid argument type", "warnings": [],
                  "error_class": ["simpleError", "error", "condition"],
                  "error_call": "!something_unrelated"}
        with self.assertRaisesRegex(ValueError, "error"):
            self.module.validate_error("removed_reference", record)


if __name__ == "__main__":
    unittest.main()
