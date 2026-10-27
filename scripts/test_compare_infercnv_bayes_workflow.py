import copy
from array import array
import unittest

from compare_infercnv_bayes_workflow import compare_bayesian
from validate_infercnv_shipped_oracle import Matrix


def example():
    genes = [dict(id=f"g{i}", chromosome="chr1", start=i + 1, end=i + 1) for i in range(3)]
    cells = [dict(id=f"c{i}", group="obs") for i in range(12)]
    regions = [dict(name=f"chr1-region_{2 * i + 1}", state=state, chromosome="chr1",
                    start=i + 1, end=i + 1, gene_range=[i, i + 1])
               for i, state in enumerate((2, 4, 6))]
    assignments = [[3] * 12, [4] * 12, [3] * 4 + [6] * 3 + [1] * 2 + [2, 4, 5]]
    means = [[1 / 18, 1 / 18, 13 / 18, 1 / 18, 1 / 18, 1 / 18],
             [1 / 18, 1 / 18, 1 / 18, 13 / 18, 1 / 18, 1 / 18],
             [3 / 18, 2 / 18, 5 / 18, 2 / 18, 2 / 18, 4 / 18]]
    final_regions = [dict(regions[1]), dict(regions[2], state=3)]
    group = dict(name="obs.obs_s1", member_indices=list(range(12)), states=[2, 4, 6], regions=regions)
    report = dict(genes=genes, cells=cells, hmm=dict(groups=[group]),
                  settings=dict(bayesian=dict(seed=19, max_normal_probability=.5)),
                  bayesian=dict(step=19, model="i6-shared-dirichlet-normal", sampler="collapsed-gibbs",
                                applied=True, chains=6, burn_in=700, samples_per_chain=1000,
                                groups=[dict(group, states=[3, 4, 3], regions=final_regions)], posteriors=[]))
    reference = dict(regions={}, probabilities={}, cells={})
    uncertainty = {}
    for index, (region, probabilities, cell_states) in enumerate(zip(regions, means, assignments)):
        name = region["name"]
        cell_probabilities = [[float(state == assigned) for state in range(1, 7)] for assigned in cell_states]
        report["bayesian"]["posteriors"].append(dict(
            group_index=0, region_index=index, probabilities=probabilities,
            cell_probabilities=cell_probabilities, chain_probabilities=[probabilities[:] for _ in range(6)],
            removed=index == 0, final_state=(3, 4, 3)[index]))
        reference["regions"][name] = dict(original_state=region["state"], removed=index == 0,
                                            final_state=(3, 4, 3)[index])
        reference["probabilities"][name] = probabilities[:]
        for cell, probabilities in enumerate(cell_probabilities):
            reference["cells"][(name, cell + 1)] = probabilities[:]
        for state, probability in enumerate(means[index], 1):
            uncertainty[(name, state)] = dict(mean=probability, standard_error=.001)
    states = Matrix(tuple(cell["id"] for cell in cells), tuple(gene["id"] for gene in genes),
                    tuple(array("d", [value] * 12) for value in [3, 4, 3]))
    tables = dict(
        pred_cnv_regions=[("obs.obs_s1", "chr1-region_3", 4, "chr1", 2, 2),
                          ("obs.obs_s1", "chr1-region_5", 3, "chr1", 3, 3)],
        pred_cnv_genes=[("obs.obs_s1", "chr1-region_3", 4, "g1", "chr1", 2, 2),
                        ("obs.obs_s1", "chr1-region_5", 3, "g2", "chr1", 3, 3)])
    return report, reference, uncertainty, states, tables


class BayesianComparisonTests(unittest.TestCase):
    def test_actual_posteriors_keep_cell_identity_and_retained_neutral_region_ids(self):
        result = compare_bayesian(*example())
        self.assertEqual(result["regions"], 3)
        self.assertEqual(result["removed"], 1)
        self.assertEqual(result["retained"], 2)
        self.assertEqual(result["max_posterior_scaled_error"], 0)

    def test_posterior_cell_joins_states_and_adjusted_region_identity_are_checked(self):
        for mutation in ("posterior", "cell", "state", "region"):
            arguments = copy.deepcopy(example())
            report = arguments[0]
            if mutation == "posterior":
                report["bayesian"]["posteriors"][0]["probabilities"][2] -= .1
                report["bayesian"]["posteriors"][0]["probabilities"][0] += .1
            elif mutation == "cell":
                cells = report["bayesian"]["posteriors"][2]["cell_probabilities"]
                cells[0], cells[4] = cells[4], cells[0]
            elif mutation == "state":
                report["bayesian"]["groups"][0]["states"][2] = 6
            else:
                report["bayesian"]["groups"][0]["regions"][1]["name"] = "chr1-region_4"
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                compare_bayesian(*arguments)

    def test_source_indeterminate_maximum_is_labeled_without_losing_structural_checks(self):
        arguments = example()
        report, reference, uncertainty, states, tables = arguments
        name = "chr1-region_5"
        values = reference["probabilities"][name]
        values[2], values[5] = values[5], values[2]
        reference["regions"][name]["final_state"] = 6
        for state in range(1, 7):
            uncertainty[(name, state)]["mean"] = values[state - 1]
        for state in (3, 6):
            uncertainty[(name, state)]["standard_error"] = .01
        for cell in range(12):
            states.rows[2][cell] = 6
        tables["pred_cnv_regions"][1] = ("obs.obs_s1", name, 6, "chr1", 3, 3)
        tables["pred_cnv_genes"][1] = ("obs.obs_s1", name, 6, "g2", "chr1", 3, 3)
        result = compare_bayesian(*arguments)
        self.assertEqual(result["ambiguous_reassignments"], [dict(
            region=name, admissible_states=[3, 6], native_state=3, observed_state=6)])
        self.assertEqual(result["reassignment_differences"], 1)
        reference["regions"][name]["final_state"] = 2
        with self.assertRaises(ValueError):
            compare_bayesian(*arguments)


if __name__ == "__main__":
    unittest.main()
