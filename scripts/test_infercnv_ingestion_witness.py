import gzip
import hashlib
import tempfile
import unittest
from pathlib import Path

from validate_infercnv_ingestion_witness import (
    checked_file, validate_arguments, validate_case, validate_small,
)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class WitnessCaseTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / "source"
        self.case = self.root / "case"
        self.source.mkdir()
        self.case.mkdir()
        self.counts = self.source / "counts.tsv"
        self.counts.write_text("gene\tn\tt\na\t2\t3\nb\t4\t5\n")
        self.positions = self.source / "positions.tsv"
        self.positions.write_text("a\tchr1\t1\t2\nb\tchr1\t3\t4\n")
        self.annotations = self.source / "annotations.tsv"
        self.annotations.write_text("n\tnormal\nt\ttumor\n")
        self.expected = self.source / "stage1.tsv"
        self.expected.write_text(self.counts.read_text())
        (self.case / "counts.tsv.gz").write_bytes(gzip.compress(self.counts.read_bytes()))
        outputs = {}
        for branch in ("plain", "gzip"):
            folder = self.case / branch
            folder.mkdir()
            (folder / "expression.tsv").write_text(self.expected.read_text())
            (folder / "genes.tsv").write_text("gene\tchr\tstart\tstop\na\tchr1\t1\t2\nb\tchr1\t3\t4\n")
            (folder / "cells.tsv").write_text("cell\tgroup\trole\nn\tnormal\treference\nt\ttumor\tobservation\n")
            (folder / "maps.tsv").write_text("role\tgroup\tindex\tcell\nreference\tnormal\t1\tn\nobservation\ttumor\t2\tt\n")
            outputs[branch] = {name: digest(folder / name) for name in
                               ("expression.tsv", "genes.tsv", "cells.tsv", "maps.tsv")}
        self.record = dict(counts_sha256=digest(self.counts),
                           gzip_sha256=digest(self.case / "counts.tsv.gz"),
                           positions_sha256=digest(self.positions),
                           annotations_sha256=digest(self.annotations),
                           genes=2, cells=2, references=["normal"],
                           reference_maps={"normal": [1]},
                           observation_maps={"tumor": [2]}, **outputs)

    def test_accepts_exact_plain_gzip_and_ordered_maps(self):
        self.assertEqual(validate_case(self.case, self.counts, self.positions,
                                       self.annotations, self.expected, self.record), (2, 2))

    def test_rejects_changed_source_count_bytes(self):
        self.counts.write_text(self.counts.read_text().replace("\t2\t3", "\t9\t3"))
        with self.assertRaisesRegex(ValueError, "count hash"):
            validate_case(self.case, self.counts, self.positions,
                          self.annotations, self.expected, self.record)

    def test_rejects_changed_gzip_contents_even_with_updated_digest(self):
        (self.case / "counts.tsv.gz").write_bytes(gzip.compress(b"wrong"))
        self.record["gzip_sha256"] = digest(self.case / "counts.tsv.gz")
        with self.assertRaisesRegex(ValueError, "gzip bytes"):
            validate_case(self.case, self.counts, self.positions,
                          self.annotations, self.expected, self.record)

    def test_rejects_wrong_ordered_map(self):
        for branch in ("plain", "gzip"):
            path = self.case / branch / "maps.tsv"
            path.write_text(path.read_text().replace("1\tn", "2\tn"))
            self.record[branch]["maps.tsv"] = digest(path)
        with self.assertRaisesRegex(ValueError, "ordered map differs from cells"):
            validate_case(self.case, self.counts, self.positions,
                          self.annotations, self.expected, self.record)

    def test_accepts_jsonlite_empty_reference_list(self):
        for branch in ("plain", "gzip"):
            cells = self.case / branch / "cells.tsv"
            cells.write_text(cells.read_text().replace("normal\treference", "normal\tobservation"))
            maps = self.case / branch / "maps.tsv"
            maps.write_text(maps.read_text().replace("reference\tnormal", "observation\tnormal"))
            self.record[branch]["cells.tsv"] = digest(cells)
            self.record[branch]["maps.tsv"] = digest(maps)
        self.record["references"] = []
        self.record["reference_maps"] = []
        self.record["observation_maps"] = {"normal": [1], "tumor": [2]}
        self.assertEqual(validate_case(self.case, self.counts, self.positions,
                                       self.annotations, self.expected, self.record), (2, 2))

    def test_rejects_path_escape(self):
        with self.assertRaisesRegex(ValueError, "bundle path"):
            checked_file(self.case, "../source/counts.tsv")

    def test_rejects_rehashed_changed_coordinates_against_stage1(self):
        expected_genes = self.source / "stage1.genes.tsv"
        expected_genes.write_bytes((self.case / "plain/genes.tsv").read_bytes())
        for branch in ("plain", "gzip"):
            path = self.case / branch / "genes.tsv"
            path.write_text(path.read_text().replace("chr1\t1\t2", "chr1\t1\t9"))
            self.record[branch]["genes.tsv"] = digest(path)
        with self.assertRaisesRegex(ValueError, "stage-1 genes"):
            validate_case(self.case, self.counts, self.positions, self.annotations,
                          self.expected, self.record, expected_genes=expected_genes)

    def test_rejects_rehashed_nonfinite_numeric_output(self):
        for branch in ("plain", "gzip"):
            path = self.case / branch / "expression.tsv"
            path.write_text(path.read_text().replace("\t2\t3", "\tNaN\t3"))
            self.record[branch]["expression.tsv"] = digest(path)
        with self.assertRaisesRegex(ValueError, "finite"):
            validate_case(self.case, self.counts, self.positions, self.annotations,
                          None, self.record)


class SmallWitnessTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "expression.tsv").write_text(
            "gene\tt_b\tref2\tobs_a\tref1\nG_A\t2\t2\t2\t2\n"
            "G_D\t0.076439000000000007\t2\t2\t2\n"
            "G_B\t0.076439000000000007\t2\t2\t2\nG_C\t2\t2\t2\t2\n")
        (self.root / "genes.tsv").write_text(
            "gene\tchr\tstart\tstop\nG_A\tchr10\t30\t31\nG_D\tchr2\t10\t11\n"
            "G_B\tchr2\t20\t21\nG_C\tchr1\t5\t6\n")
        (self.root / "cells.tsv").write_text(
            "cell\tgroup\trole\nt_b\tz_obs\tobservation\nref2\ta_ref\treference\n"
            "obs_a\ta_obs\tobservation\nref1\tz_ref\treference\n")
        (self.root / "maps.tsv").write_text(
            "role\tgroup\tindex\tcell\nreference\tz_ref\t4\tref1\nreference\ta_ref\t2\tref2\n"
            "observation\ta_obs\t3\tobs_a\nobservation\tz_obs\t1\tt_b\n")

    def test_accepts_hand_checked_small_state(self):
        validate_small(self.root)

    def test_rejects_adjacent_wrong_decimal_despite_unchanged_metadata(self):
        path = self.root / "expression.tsv"
        path.write_text(path.read_text().replace("0.076439000000000007", "0.076439"))
        with self.assertRaisesRegex(ValueError, "small numeric"):
            validate_small(self.root)

    def test_rejects_wrong_gene_order(self):
        path = self.root / "genes.tsv"
        rows = path.read_text().splitlines()
        rows[1], rows[2] = rows[2], rows[1]
        path.write_text("\n".join(rows) + "\n")
        with self.assertRaisesRegex(ValueError, "small gene"):
            validate_small(self.root)


class ArgumentTests(unittest.TestCase):
    def setUp(self):
        self.record = {"source_checkpoint": "full/upstream/01.rds",
                       "creation_arguments": {
            "raw_counts_matrix": "/runner/shipped-bundle/inputs/counts.tsv",
            "gene_order_file": "/runner/shipped-bundle/inputs/genes.tsv",
            "annotations_file": "/runner/shipped-bundle/inputs/annotations.tsv",
            "ref_group_names": ["normal"], "delim": "\t",
            "max_cells_per_group": None, "min_max_counts_per_cell": [1, "Inf"],
            "chr_exclude": ["chrX", "chrY", "chrM"],
        }}

    def check(self):
        validate_arguments(self.record, "shipped-bundle", "inputs/counts.tsv",
                           "inputs/genes.tsv", "inputs/annotations.tsv",
                           ["normal"], "full/upstream/01.rds")

    def test_accepts_recorded_arguments(self):
        self.check()

    def test_rejects_changed_checkpoint_path(self):
        self.record["source_checkpoint"] = "full/upstream/other.rds"
        with self.assertRaisesRegex(ValueError, "checkpoint path"):
            self.check()

    def test_rejects_changed_filter_arguments(self):
        self.record["creation_arguments"]["min_max_counts_per_cell"] = [100, "Inf"]
        with self.assertRaisesRegex(ValueError, "creation arguments"):
            self.check()

    def test_rejects_wrong_or_noncanonical_source_path(self):
        for path in ("/runner/bundle/inputs/counts.tsv",
                     "/runner/shipped-bundle/../shipped-bundle/inputs/counts.tsv"):
            with self.subTest(path=path):
                self.record["creation_arguments"]["raw_counts_matrix"] = path
                with self.assertRaisesRegex(ValueError, "creation input path"):
                    self.check()


if __name__ == "__main__":
    unittest.main()
