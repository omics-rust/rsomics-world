"""Miniature creation exports exercise contracts, not upstream performance."""

import importlib.util
import math
import os
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
        self.temp = tempfile.TemporaryDirectory(dir=os.environ["TMPDIR"])
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.full, self.trial = self.root / "full", self.root / "trial"
        self.full.mkdir()
        self.trial.mkdir()
        self.maps = self.root / "maps.tsv"
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
            for digest in value["output_sha256"].values():
                self.assertRegex(digest, r"\A[0-9a-f]{64}\Z")
        self.assertNotEqual(result["warm"]["output_sha256"]["expression.tsv"],
                            result["measured"]["output_sha256"]["expression.tsv"])

    def test_bad_numeric_values_in_either_factory_result(self):
        for phase in ("warm", "measured"):
            path = self.trial / phase / "expression.tsv"
            for value in (str(math.nextafter(1.0, math.inf)), "NaN", "Inf", "-1", "1_0", "\u00a01"):
                with self.subTest(phase=phase, value=value):
                    path.write_text(self.payload["expression.tsv"].replace("g1\t1\t", "g1\t" + value + "\t"))
                    with self.assertRaises(ValueError):
                        self.validate()
            path.write_text(self.payload["expression.tsv"])

    def test_role_and_joint_map_mutation(self):
        directory = self.trial / "measured"
        (directory / "cells.tsv").write_text(self.payload["cells.tsv"].replace("c_a\tr_a\treference", "c_a\tr_a\tobservation"))
        (directory / "maps.tsv").write_text(self.payload["maps.tsv"].replace("reference\tr_a", "observation\tr_a"))
        with self.assertRaises(ValueError):
            self.validate()

    def test_map_order_membership_and_identity_mutations(self):
        path = self.trial / "warm" / "maps.tsv"
        rows = self.payload["maps.tsv"].splitlines(keepends=True)
        mutations = [rows[0] + rows[2] + rows[1] + rows[3],
                     self.payload["maps.tsv"].replace("\t3\tc_b", "\t1\tc_b"),
                     self.payload["maps.tsv"].replace("\t3\tc_b", "\t4\tc_b"),
                     "".join(rows[:-1]), "".join(rows + rows[1:2]),
                     self.payload["maps.tsv"].replace("c_b", " c_b")]
        for text in mutations:
            with self.subTest(text=text):
                path.write_text(text)
                with self.assertRaises(ValueError):
                    self.validate()

    def test_missing_phase_or_unrecorded_output(self):
        shutil.rmtree(self.trial / "warm")
        with self.assertRaises((ValueError, OSError)):
            self.validate()
        (self.trial / "warm").mkdir()
        for name, text in self.payload.items():
            (self.trial / "warm" / name).write_text(text)
        (self.trial / "measured" / "extra").write_text("unexpected")
        with self.assertRaises(ValueError):
            self.validate()

    def test_signed_zero_requires_the_same_binary64_bits(self):
        original = self.payload["expression.tsv"].replace("g1\t1\t", "g1\t0\t")
        (self.full / "01.tsv").write_text(original)
        for phase in ("warm", "measured"):
            (self.trial / phase / "expression.tsv").write_text(original)
        self.validate()
        (self.trial / "measured" / "expression.tsv").write_text(original.replace("g1\t0\t", "g1\t-0\t"))
        with self.assertRaises(ValueError):
            self.validate()

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
                    with self.assertRaises(ValueError):
                        self.validate()
            path.write_text(self.payload[name])

    def test_missing_measured_and_escaped_output(self):
        shutil.rmtree(self.trial / "measured")
        with self.assertRaises((ValueError, OSError)):
            self.validate()
        (self.trial / "measured").mkdir()
        for name, text in self.payload.items():
            (self.trial / "measured" / name).write_text(text)
        path = self.trial / "measured" / "maps.tsv"
        path.unlink()
        path.symlink_to(self.maps)
        with self.assertRaises(ValueError):
            self.validate()

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
        with self.assertRaises(ValueError):
            self.validate()

    def test_added_csv_quotes_do_not_preserve_literal_identity(self):
        mutations = {
            "expression.tsv": [self.payload["expression.tsv"].replace("c_a", '"c_a"'),
                               self.payload["expression.tsv"].replace("g1", '"g1"'),
                               self.payload["expression.tsv"].replace("g1\t1\t", 'g1\t"1"\t')],
            "genes.tsv": [self.payload["genes.tsv"].replace("g1", '"g1"')],
            "cells.tsv": [self.payload["cells.tsv"].replace("c_a", '"c_a"'),
                          self.payload["cells.tsv"].replace("r_a", '"r_a"')],
            "maps.tsv": [self.payload["maps.tsv"].replace("c_a", '"c_a"'),
                         self.payload["maps.tsv"].replace("r_a", '"r_a"')],
        }
        for name, changes in mutations.items():
            path = self.trial / "measured" / name
            for text in changes:
                with self.subTest(name=name, text=text):
                    path.write_text(text)
                    with self.assertRaises(ValueError):
                        self.validate()
            path.write_text(self.payload[name])

    def test_literal_quotes_in_ids_and_groups_are_not_csv_syntax(self):
        for root in (self.full, self.trial / "warm", self.trial / "measured"):
            for path in root.iterdir():
                path.write_text(path.read_text().replace("c_a", '"c_a"')
                                .replace("r_a", '"r_a"').replace("g1", '"g1"'))
        self.maps.write_text(self.maps.read_text().replace("c_a", '"c_a"').replace("r_a", '"r_a"'))
        self.assertEqual(self.validate()["measured"]["cells"], 3)
        path = self.trial / "measured" / "expression.tsv"
        path.write_text(path.read_text().replace('"g1"', "g1"))
        with self.assertRaises(ValueError):
            self.validate()

    def test_phase_inventory_rejects_empty_directory(self):
        extra = self.trial / "warm" / "extra"
        extra.mkdir()
        with self.assertRaises(ValueError):
            self.validate()

    def test_phase_inventory_rejects_directory_symlink(self):
        (self.trial / "warm" / "extra").symlink_to(self.full, target_is_directory=True)
        with self.assertRaises(ValueError):
            self.validate()

    def test_phase_inventory_rejects_broken_symlink(self):
        (self.trial / "measured" / "extra").symlink_to(self.root / "absent")
        with self.assertRaises(ValueError):
            self.validate()

    def test_trial_metadata_is_outside_creation_export_inventory(self):
        (self.trial / "metrics.tsv").write_text("metric\tvalue\n")
        (self.trial / "provenance.json").write_text("{}")
        self.assertEqual(set(self.validate()), {"warm", "measured"})

    def test_symlink_trial_result_and_expected_roots_fail(self):
        trial_alias, result_alias = self.root / "trial-alias", self.root / "result-alias"
        full_alias, maps_alias = self.root / "full-alias", self.root / "maps-parent-alias"
        for alias, target in ((trial_alias, self.trial), (result_alias, self.trial / "warm"),
                              (full_alias, self.full), (maps_alias, self.root)):
            alias.symlink_to(target, target_is_directory=True)
        with self.assertRaises(ValueError):
            self.checker.validate_factory_results(trial_alias, self.full, self.maps)
        for result, full, maps in ((result_alias, self.full, self.maps),
                                   (self.trial / "warm", full_alias, self.maps),
                                   (self.trial / "warm", self.full, maps_alias / "maps.tsv")):
            with self.subTest(result=result, full=full, maps=maps):
                with self.assertRaises(ValueError):
                    self.checker.validate_creation_export(result, full, maps)

    def test_cell_headers_cannot_rename_roles_or_add_columns(self):
        path = self.trial / "measured" / "cells.tsv"
        for text in (self.payload["cells.tsv"].replace("role", "type"),
                     self.payload["cells.tsv"].replace("cell\tgroup\trole", "cell\tgroup\trole\textra")
                     .replace("\treference\n", "\treference\tx\n")
                     .replace("\tobservation\n", "\tobservation\tx\n")):
            with self.subTest(text=text):
                path.write_text(text)
                with self.assertRaises(ValueError):
                    self.validate()

    def test_invalid_coordinates_fail_even_if_expected_and_actual_match(self):
        for chromosome, start, stop in (("", "1", "2"), ("chr1", "3", "2"),
                                         ("chr1", "0", "0"), ("chr1", "-1", "2"),
                                         ("chr1", "١", "2"),
                                         ("chr1", "1", str(2**64))):
            with self.subTest(chromosome=chromosome, start=start, stop=stop):
                text = self.payload["genes.tsv"].replace("g1\tchr1\t1\t2",
                                                       f"g1\t{chromosome}\t{start}\t{stop}")
                (self.full / "01.genes.tsv").write_text(text)
                for phase in ("warm", "measured"):
                    (self.trial / phase / "genes.tsv").write_text(text)
                with self.assertRaises(ValueError):
                    self.validate()

    def test_zero_and_overflowing_depth_fail_even_if_oracle_matches(self):
        for expression in ("gene\tc_a\tc_obs\tc_b\ng1\t0\t2\t3\ng2\t0\t5\t6\n",
                           "gene\tc_a\tc_obs\tc_b\ng1\t1e308\t2\t3\ng2\t1e308\t5\t6\n"):
            with self.subTest(expression=expression):
                (self.full / "01.tsv").write_text(expression)
                for phase in ("warm", "measured"):
                    (self.trial / phase / "expression.tsv").write_text(expression)
                with self.assertRaises(ValueError):
                    self.validate()

    def test_malformed_numeric_spelling_cannot_alias_expected_value(self):
        expression = self.payload["expression.tsv"].replace("g1\t1\t", "g1\t10\t")
        (self.full / "01.tsv").write_text(expression)
        for phase in ("warm", "measured"):
            (self.trial / phase / "expression.tsv").write_text(expression)
        self.validate()
        path = self.trial / "measured" / "expression.tsv"
        for spelling in ("1_0", "١٠", "\u00a010", '"10"'):
            with self.subTest(spelling=spelling):
                path.write_text(expression.replace("g1\t10\t", f"g1\t{spelling}\t"))
                with self.assertRaises(ValueError):
                    self.validate()

    def test_each_export_is_required_in_both_results(self):
        for phase in ("warm", "measured"):
            for name, text in self.payload.items():
                with self.subTest(phase=phase, name=name):
                    path = self.trial / phase / name
                    path.unlink()
                    with self.assertRaises(ValueError):
                        self.validate()
                    path.write_text(text)


if __name__ == "__main__":
    unittest.main()
