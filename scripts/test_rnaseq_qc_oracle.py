import importlib
from pathlib import Path
import signal
import tempfile
import unittest
from unittest.mock import patch


class RnaSeqOracleTests(unittest.TestCase):
    def oracle(self):
        try:
            return importlib.import_module("rnaseq_qc_oracle")
        except ModuleNotFoundError:
            self.fail("the coordinated RNA-seq oracle driver is missing")

    def test_fixture_keeps_informative_integrity_cases_and_unmixed_layouts(self):
        oracle = self.oracle()
        for profile in ("single", "paired"):
            plan = oracle.fixture_plan(profile)
            spike = [read for read in plan["records"] if 70000 <= read["start"] < 71200]
            zero = [read for read in plan["records"] if 100000 <= read["start"] < 101200]
            self.assertEqual(len({read["start"] for read in spike}), 12)
            self.assertEqual(len({read["start"] for read in zero}), 11)
            self.assertEqual({read["start"] for read in spike if read["start"] < 70100}, {70000})
            self.assertTrue(all(read["start"] >= 100100 for read in zero))
            self.assertTrue(all(bool(read["flag"] & 1) == (profile == "paired")
                                for read in plan["records"] if not read["flag"] & 4))
            lengths = {read["name"]: oracle.query_length(read["cigar"]) for read in plan["records"]}
            self.assertEqual(lengths["soft-clip"], 80)
            self.assertEqual(lengths["eqx"], 50)

    def test_mapping_keeps_counters_whose_fixed_width_label_reaches_the_value(self):
        text = ("Total records: 88\nmapq >= mapq_cut (unique): 82\n"
                "Non primary hits 1\nProper-paired reads map to different chrom:0\n")
        values = self.oracle().parse_stdout("mapping", text, "single")
        self.assertEqual(values["Proper-paired reads map to different chrom:"], 0)
        self.assertEqual(values["Non primary hits"], 1)

    def test_numeric_capture_rejects_empty_success_and_keeps_eligible_zero_in_mean(self):
        oracle = self.oracle()
        coverage = "Percentile\t" + "\t".join(map(str, range(1, 101))) + "\n"
        with self.assertRaises(ValueError):
            oracle.parse_coverage(coverage)
        row = "single\t" + "\t".join(["1"] + ["0"] * 99) + "\n"
        self.assertEqual(oracle.parse_coverage(coverage + row)["coverage"][0], 1)
        tin = ("geneID\tchrom\ttx_start\ttx_end\tTIN\n"
               "plus\tchr1\t10000\t11200\t80.0\n"
               "minus\tchr1\t40000\t41200\t60.0\n"
               "entropy_positive\tchr1\t70000\t71200\t1.0\n"
               "eligible_zero\tchr1\t100000\t101200\t0.0\n"
               "empty\tchr1\t130000\t131200\t0.0\n")
        header = "Bam_file\tTIN(mean)\tTIN(median)\tTIN(stdev)\n"
        result = oracle.parse_integrity(tin, header + "single.bam\t35.25\t30.5\t35\n")
        self.assertEqual(result["summary"]["mean"], 35.25)
        with self.assertRaises(ValueError):
            oracle.parse_integrity(tin, header + "single.bam\t47\t60\t35\n")

    def test_user_interruption_stops_the_campaign_instead_of_advancing_metrics(self):
        oracle = self.oracle()
        from infercnv_trial_process import TrialInterrupted
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "cases/single/inputs").mkdir(parents=True)
            with patch.object(oracle, "build_fixture", return_value=(root / "input.bam", root / "genes.bed", 1)), \
                    patch.object(oracle, "run_child", side_effect=TrialInterrupted(signal.SIGTERM)):
                with self.assertRaises(TrialInterrupted):
                    oracle.run_profile(root, "single", {"mapping": root / "bam_stat.py"})


if __name__ == "__main__":
    unittest.main()
