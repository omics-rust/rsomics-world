import copy
import unittest

from measure_infercnv_samples_cli import report_fingerprint, summarize, validate_report


def trial(threads, wall, rss, phase="measured"):
    return dict(threads=threads, phase=phase, wall_seconds=wall,
                user_seconds=wall, system_seconds=0.0, maximum_rss_bytes=rss,
                owner_wall_seconds=wall + 0.01)


class SamplesCliMeasurementTests(unittest.TestCase):
    def test_gbm_case_checks_real_cell_coverage_without_inventing_filtered_gene_counts(self):
        cells = 4103
        report = dict(schema="rsomics-sc.cnv.samples.v1", genes=[{"id": "a"}, {"id": "b"}],
                      cells=[{"id": str(i)} for i in range(cells)],
                      selected_gene_indices=[0, 1], outlier_gene_indices=None,
                      groups=[dict(name="all_observations", member_indices=list(range(cells)),
                                   tree=dict(merge=[[-1, -2]] + [[i, -i - 2] for i in range(1, cells - 1)],
                                             order=list(range(cells))))])
        validate_report(report, "gbm-4103")
        report["selected_gene_indices"] = [1, 0]
        with self.assertRaises(ValueError):
            validate_report(report, "gbm-4103")

    def test_only_thread_provenance_is_ignored_without_mutating_the_report(self):
        serial = {"settings": {"threads": 1, "cutoff": 1.0},
                  "genes": [{"id": "A"}], "groups": [{"member_indices": [2, 0, 1]}]}
        parallel = copy.deepcopy(serial)
        parallel["settings"]["threads"] = 4
        self.assertEqual(report_fingerprint(serial, 1), report_fingerprint(parallel, 4))
        self.assertEqual(serial["settings"]["threads"], 1)
        parallel["groups"][0]["member_indices"] = [0, 2, 1]
        self.assertNotEqual(report_fingerprint(serial, 1), report_fingerprint(parallel, 4))

    def test_wrong_report_worker_count_is_rejected(self):
        with self.assertRaises(ValueError):
            report_fingerprint({"settings": {"threads": 1}}, 4)

    def test_statistics_exclude_warmups_and_keep_slower_parallel_results(self):
        records = [trial(1, 100.0, 999, "warmup"), trial(4, 200.0, 999, "warmup"),
                   trial(1, 2.0, 100), trial(4, 4.0, 150),
                   trial(4, 6.0, 170), trial(1, 4.0, 200)]
        result = summarize(records)
        self.assertEqual(result["workers"]["1"]["wall_seconds"],
                         {"median": 3.0, "min": 2.0, "max": 4.0, "values": [2.0, 4.0]})
        self.assertEqual(result["workers"]["4"]["maximum_rss_bytes"]["median"], 160)
        self.assertEqual(result["serial_to_parallel_median_wall_ratio"], 0.6)
        self.assertFalse(result["strict_parallel_wall_separation"])

    def test_incomplete_worker_comparison_has_no_summary(self):
        for records in [[], [trial(1, 1.0, 100)],
                        [trial(1, 1.0, 100), trial(4, 1.0, 100), trial(1, 1.0, 100)]]:
            with self.subTest(records=records), self.assertRaises(ValueError):
                summarize(records)


if __name__ == "__main__":
    unittest.main()
