from array import array
import copy
import unittest

from compare_infercnv_hmm_workflow import compare_report
from validate_infercnv_shipped_oracle import Matrix


def example():
    genes = [dict(id=name, chromosome=chromosome, start=start, end=end)
             for name, chromosome, start, end in [
                 ("a", "chr1", 1, 5), ("b", "chr1", 10, 15), ("c", "chr1", 20, 25),
                 ("d", "chrS", 1, 2), ("e", "chr2", 4, 9), ("f", "chr2", 10, 20)]]
    cells = [dict(id="t1", group="obs"), dict(id="t2", group="obs")]
    expression = Matrix(("t1", "t2"), ("a", "b", "c", "d", "e", "f"),
                        tuple(array("d", [1, 1]) for _ in range(6)))
    states = Matrix(expression.cells, expression.genes,
                    tuple(array("d", pair) for pair in [(2, 2), (3, 3), (4, 4), (3, 3), (6, 6), (6, 6)]))
    case = dict(groups=[dict(name="obs", indices=[1, 2])], calibration=dict(
        means={"cnv:1": dict(mean=1, sd=.1), "cnv:0.01": dict(mean=.1, sd=.1),
               "cnv:0.5": dict(mean=.5, sd=.1), "cnv:1.5": dict(mean=1.5, sd=.1),
               "cnv:2": dict(mean=2, sd=.1), "cnv:3": dict(mean=3, sd=.1)},
        coefficients={name: [-2.3, -.1] for name in
                      ["cnv:1", "cnv:0.01", "cnv:0.5", "cnv:1.5", "cnv:2", "cnv:3"]}))
    report = dict(schema="rsomics-sc.cnv.samples.v1", genes=copy.deepcopy(genes), cells=copy.deepcopy(cells),
                  groups=[dict(name="obs", subcluster_name="obs_s1", member_indices=[1, 0])],
                  expression=dict(step=14, layout="column-major", shape=[6, 2], values=[1] * 12),
                  hmm=dict(step=17, model="i6", grouping="samples", bayesian_filter=False,
                           calibration=dict(means=[.1, .5, 1, 1.5, 2, 3], standard_deviations=[.1] * 6,
                                            log_sd_fits=[[-2.3, -.1] for _ in range(6)],
                                            simulated_genes=4400, simulated_cells=200),
                           groups=[dict(name="obs.obs_s1", member_indices=[1, 0], states=[2, 3, 4, 3, 6, 6],
                                        regions=[dict(name="chr1-region_1", state=2, chromosome="chr1", start=1, end=5, gene_range=[0, 1]),
                                                 dict(name="chr1-region_3", state=4, chromosome="chr1", start=20, end=25, gene_range=[2, 3]),
                                                 dict(name="chr2-region_4", state=6, chromosome="chr2", start=4, end=20, gene_range=[4, 6])])]))
    tables = dict(
        cell_groupings=[("obs.obs_s1", "t2"), ("obs.obs_s1", "t1")],
        genes_used=[("a", "chr1", 1, 5), ("b", "chr1", 10, 15), ("c", "chr1", 20, 25),
                    ("d", "chrS", 1, 2), ("e", "chr2", 4, 9), ("f", "chr2", 10, 20)],
        pred_cnv_regions=[("obs.obs_s1", "chr1-region_1", 2, "chr1", 1, 5),
                          ("obs.obs_s1", "chr1-region_3", 4, "chr1", 20, 25),
                          ("obs.obs_s1", "chr2-region_4", 6, "chr2", 4, 20)],
        pred_cnv_genes=[("obs.obs_s1", "chr1-region_1", 2, "a", "chr1", 1, 5),
                        ("obs.obs_s1", "chr1-region_3", 4, "c", "chr1", 20, 25),
                        ("obs.obs_s1", "chr2-region_4", 6, "e", "chr2", 4, 9),
                        ("obs.obs_s1", "chr2-region_4", 6, "f", "chr2", 10, 20)])
    return report, case, expression, states, genes, cells, (4400, 200), tables


class HmmComparisonTests(unittest.TestCase):
    def test_actual_report_projection_preserves_neutral_gaps_and_membership_order(self):
        result = compare_report(*example())
        self.assertEqual(result["regions"], 3)
        self.assertEqual(result["predicted_genes"], 4)
        self.assertEqual(result["state_values"], 12)
        self.assertEqual(result["max_calibration_scaled_error"], 0.0)
        self.assertEqual(result["max_expression_scaled_error"], 0.0)

    def test_calibration_states_and_each_original_report_remain_independent_checks(self):
        for mutation in ["calibration", "state", "groupings", "regions", "genes", "used"]:
            arguments = example()
            report, *_, tables = arguments
            if mutation == "calibration":
                report["hmm"]["calibration"]["log_sd_fits"][0][0] += .01
            elif mutation == "state":
                report["hmm"]["groups"][0]["states"][1] = 2
            elif mutation == "groupings":
                tables["cell_groupings"].reverse()
            elif mutation == "regions":
                tables["pred_cnv_regions"][1] = ("obs.obs_s1", "chr1-region_2", 4, "chr1", 20, 25)
            elif mutation == "genes":
                tables["pred_cnv_genes"].pop()
            else:
                tables["genes_used"][0] = ("a", "chr1", 1, 6)
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                compare_report(*arguments)


if __name__ == "__main__":
    unittest.main()
