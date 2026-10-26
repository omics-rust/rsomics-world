from array import array
import copy
import unittest

from compare_infercnv_denoise_workflow import compare_report
from validate_infercnv_shipped_oracle import Matrix


def example():
    before = Matrix(("n", "t", "u", "v"), ("a", "b"),
                    (array("d", [1, 1.4, 2, .8]), array("d", [1, .7, .5, 1.2])))
    after = Matrix(before.cells, before.genes,
                   (array("d", [1, 1, 2, 1]), array("d", [1, 1, .5, 1])))
    case = dict(mode=dict(enabled=True), noise=dict(center=1.0, threshold=.5),
                diagnostic=dict(selected_gene_indices=[1, 2], outliers=None),
                groups=[dict(name="all_observations",
                             tree=dict(merge=[[-1, -2], [-3, 1]], order=[3, 1, 2],
                                       height=[.6324555320336759, 1.2489995996796797]),
                             subclusters=[dict(name="all_observations_s1", indices=[4, 2, 3])]),
                        dict(name="ref", tree=None,
                             subclusters=[dict(name="ref_s1", indices=[1])])])
    genes = [dict(id="a", chromosome="chr1", start=1, end=2),
             dict(id="b", chromosome="chr1", start=3, end=4)]
    cells = [dict(id="n", group="ref"), dict(id="t", group="obs"),
             dict(id="u", group="obs"), dict(id="v", group="obs")]
    report = dict(schema="rsomics-sc.cnv.samples.v1", genes=copy.deepcopy(genes),
                  cells=copy.deepcopy(cells), selected_gene_indices=[0, 1], outlier_gene_indices=None,
                  groups=[dict(name="all_observations", subcluster_name="all_observations_s1",
                               member_indices=[3, 1, 2],
                               tree=dict(merge=[[-1, -2], [-3, 1]], order=[2, 0, 1],
                                         height=[.6324555320336759, 1.2489995996796797])),
                          dict(name="ref", subcluster_name="ref_s1", member_indices=[0], tree=None)],
                  expression=dict(step=14, layout="column-major", shape=[2, 4], values=[1, 1, 1.4, .7, 2, .5, .8, 1.2]),
                  denoised=dict(bounds=dict(center=1.0, threshold=.5),
                                expression=dict(step=22, layout="column-major", shape=[2, 4], values=[1, 1, 1, 1, 2, .5, 1, 1])))
    return report, case, before, after, genes, cells


class DenoiseComparisonTests(unittest.TestCase):
    def test_literal_raw_workflow_projection_matches_and_converts_r_indices_once(self):
        result = compare_report(*example())
        self.assertEqual(result["genes"], 2)
        self.assertEqual(result["cells"], 4)
        self.assertEqual(result["trees"], 1)
        self.assertEqual(result["max_after_scaled_error"], 0.0)

    def test_wrong_matrix_values_or_stage_are_rejected(self):
        for field in ["before", "after", "step"]:
            report, *rest = example()
            if field == "before":
                report["expression"]["values"][2] = 1.0
            elif field == "after":
                report["denoised"]["expression"]["values"][2] = 1.01
            else:
                report["denoised"]["expression"]["step"] = 14
            with self.assertRaises(ValueError):
                compare_report(report, *rest)

    def test_exact_identity_selection_and_membership_are_not_floating_tolerances(self):
        for field in ["identity", "selection", "members", "merge", "order", "height"]:
            report, *rest = example()
            if field == "identity":
                report["genes"].reverse()
            elif field == "selection":
                report["selected_gene_indices"] = [1, 0]
            elif field == "members":
                report["groups"][0]["member_indices"] = [2, 1]
            elif field == "merge":
                report["groups"][0]["tree"]["merge"][0] = [-2, -1]
            elif field == "order":
                report["groups"][0]["tree"]["order"] = [0, 1, 2]
            else:
                report["groups"][0]["tree"]["height"][1] += .01
            with self.assertRaises(ValueError):
                compare_report(report, *rest)

    def test_disabled_case_preserves_old_report_contract_and_no_fake_denoise(self):
        report, case, before, _, genes, cells = example()
        case["mode"]["enabled"] = False
        case["noise"] = None
        del report["denoised"]
        compare_report(report, case, before, before, genes, cells)
        report["denoised"] = {}
        with self.assertRaises(ValueError):
            compare_report(report, case, before, before, genes, cells)


if __name__ == "__main__":
    unittest.main()
