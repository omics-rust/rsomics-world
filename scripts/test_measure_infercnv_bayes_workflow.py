import copy
import unittest

from measure_infercnv_bayes_workflow import compare_adjusted, summarize


class WholeBayesianMeasurementTests(unittest.TestCase):
    def test_state_ambiguity_never_hides_structure_or_a_stable_state(self):
        expected = {"pred_cnv_regions": [("g", "r", 3, "chr1", 1, 2)]}
        actual = {"pred_cnv_regions": [("g", "r", 4, "chr1", 1, 2)]}
        self.assertEqual(compare_adjusted(actual, expected, {"r": {3, 4}}), ["r"])
        with self.assertRaises(ValueError):
            compare_adjusted(actual, expected, {})
        actual["pred_cnv_regions"][0] = ("g", "r", 4, "chr1", 1, 3)
        with self.assertRaises(ValueError):
            compare_adjusted(actual, expected, {"r": {3, 4}})

    def test_speed_decision_excludes_R_startup_and_requires_complete_distributions(self):
        rows = [dict(phase="measured", implementation="R", public_call_seconds=2,
                     wall_seconds=100, maximum_rss_bytes=1000) for _ in range(3)]
        rows += [dict(phase="measured", implementation="native", threads=worker,
                      wall_seconds=3 if worker == 1 else 1, maximum_rss_bytes=100)
                 for worker in (1, 4) for _ in range(5)]
        result = summarize(rows)
        self.assertFalse(result["strict_serial_wall_advantage"])
        self.assertTrue(result["strict_parallel_wall_separation"])
        with self.assertRaises(ValueError):
            summarize(rows[:-1])
        warm = copy.deepcopy(rows)
        warm.append(dict(phase="warmup", implementation="R", public_call_seconds=999))
        self.assertEqual(summarize(warm), result)
