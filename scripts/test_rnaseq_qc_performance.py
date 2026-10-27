import json
from pathlib import Path
import re
import tempfile
import unittest

from rnaseq_qc_performance import TIME_FORMAT, aggregate_source, integrity_tables, performance_decision
from test_infercnv_ingestion_workflow import bash_body, steps


class WholeReportMeasurementTests(unittest.TestCase):
    def test_tin_keeps_duplicate_names_and_eligible_zero_rows(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            tin, summary = root / "tin.tsv", root / "summary.tsv"
            tin.write_text("geneID\tchrom\ttx_start\ttx_end\tTIN\n"
                           "same\tchr21\t0\t100\t10.0\nsame\tchr21\t200\t300\t0.0\nempty\tchr21\t400\t500\t0.0\n")
            summary.write_text("Bam_file\tTIN(mean)\tTIN(median)\tTIN(stdev)\npaired.bam\t5\t5\t5\n")
            traces = [dict(geneID=name, chrom="chr21", tx_start=start, tx_end=start + 100,
                           sampled_positions=102, eligible=eligible)
                      for name, start, eligible in [("same", 0, True), ("same", 200, True), ("empty", 400, False)]]
            result = integrity_tables(tin, summary, traces)
            self.assertEqual(len(result["transcripts"]), 3)
            self.assertEqual(result["eligible_transcripts"], ["same", "same"])
            self.assertEqual([row["eligible"] for row in result["transcripts"]], [True, True, False])
            traces[1]["tx_start"] += 1
            with self.assertRaisesRegex(ValueError, "identity/order"):
                integrity_tables(tin, summary, traces)

    def test_source_workflow_sums_cost_but_never_sums_peak_memory(self):
        row = dict(wall_seconds=1.0, user_seconds=0.5, system_seconds=0.2, filesystem_inputs=2,
                   filesystem_outputs=3, major_faults=1, minor_faults=7, owner_wall_seconds=1.1,
                   max_rss_kib=1024)
        result = aggregate_source([row, dict(row, wall_seconds=2.0, max_rss_kib=2048)])
        self.assertEqual(result["wall_seconds"], 3.0)
        self.assertEqual(result["filesystem_outputs"], 6)
        self.assertEqual(result["max_rss_kib"], 2048)
        usage = json.loads(re.sub(r"%[a-zA-Z]", "1", TIME_FORMAT))
        self.assertEqual(usage["exit_status"], 1)
        self.assertEqual(set(usage), set(row) - {"owner_wall_seconds"} | {"exit_status"})

    def test_hosted_workflow_reuses_the_accepted_binary_and_propagates_failures(self):
        workflow = Path(__file__).resolve().parents[1] / ".github/workflows/rnaseq-qc-performance.yml"
        text = workflow.read_text()
        self.assertIn("40590326515", text)
        self.assertIn("12773004319", text)
        self.assertIn("if: always()", text)
        self.assertNotIn("continue-on-error", text)
        self.assertNotIn("cargo build", text)
        self.assertRegex(text, r"options:\n\s+- chr21\n\s+- whole\n")
        import subprocess
        for name, step in steps(text).items():
            if "        run: |\n" in step:
                body = re.sub(r"\$\{\{.*?\}\}", "stub", bash_body(step))
                with self.subTest(step=name):
                    result = subprocess.run(["bash", "-n"], input=body, capture_output=True, text=True, timeout=15)
                    self.assertEqual(result.returncode, 0, result.stderr)

    def test_no_advantage_is_explicitly_rejected_even_when_measurements_are_complete(self):
        source = dict(wall_seconds=dict(minimum=2, maximum=4, median=3),
                      max_rss_kib=dict(minimum=100, maximum=200, median=150))
        native = dict(wall_seconds=dict(minimum=1, maximum=3, median=2),
                      max_rss_kib=dict(minimum=100, maximum=300, median=200))
        result = performance_decision(dict(source=source, native={"1": native, "4": native}))
        self.assertGreater(result["median_serial_speedup"], 1)
        self.assertFalse(result["advantage_passed"])
        native["wall_seconds"] = dict(minimum=1, maximum=1.5, median=1.25)
        self.assertTrue(performance_decision(dict(source=source, native={"1": native, "4": native}))["advantage_passed"])


if __name__ == "__main__":
    unittest.main()
