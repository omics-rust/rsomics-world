import copy
import importlib
import unittest


class DiscoveryComparisonTests(unittest.TestCase):
    def comparator(self):
        try:
            return importlib.import_module("compare_infercnv_discovery_workflow")
        except ModuleNotFoundError:
            self.fail("the complete discovery comparator is missing")

    def test_pca_basis_rotation_keeps_geometry_but_not_wrong_features_or_distances(self):
        comparator = self.comparator()
        expected = dict(feature_indices=[7, 3], standard_deviations=[2, 2],
                        scores=[[-1, -1], [-1, 1], [1, -1], [1, 1]],
                        loadings=[[1, 0], [0, 1]])
        actual = dict(feature_indices=[7, 3], feature_scores=[1, 1], components=2,
                      scores=[1, -1, -1, -1, 1, 1, -1, 1],
                      loadings=[0, 1, -1, 0], standard_deviations=[2, 2])
        result = comparator.compare_pca(actual, expected)
        self.assertEqual(result["max_distance_scaled_error"], 0)
        for change in ("features", "geometry", "loadings"):
            wrong = copy.deepcopy(actual)
            if change == "features":
                wrong["feature_indices"] = [3, 7]
            elif change == "geometry":
                wrong["scores"][0] += 1
            else:
                wrong["loadings"] = [1, 0, 1, 0]
            with self.subTest(change=change), self.assertRaises(ValueError):
                comparator.compare_pca(wrong, expected)

    def test_graph_and_cpm_quality_gate_labels_without_requiring_one_random_partition(self):
        comparator = self.comparator()
        graph = dict(vertices=4, neighbors=[[0, 1], [1, 0], [2, 3], [3, 2]],
                     edges=[[0, 0, 1], [0, 1, 1], [0, 2, 1], [0, 3, 1],
                            [1, 1, 1], [1, 2, 1], [1, 3, 1],
                            [2, 2, 1], [2, 3, 1], [3, 3, 1]])
        comparator.compare_graph(graph, graph, 2)
        result = comparator.compare_partition([1, 2, 1, 2], [1, 1, 2, 2], graph, 1)
        self.assertEqual(result["native_cpm_quality"], 4)
        self.assertEqual(result["source_cpm_quality"], 4)
        self.assertAlmostEqual(result["adjusted_rand_index"], -0.5)
        self.assertFalse(result["same_partition"])
        wrong = copy.deepcopy(graph)
        wrong["edges"][1][2] = .5
        with self.assertRaises(ValueError):
            comparator.compare_graph(wrong, graph, 2)
        separated = dict(vertices=4, neighbors=graph["neighbors"],
                         edges=[[0, 0, 1], [0, 1, 1], [1, 1, 1],
                                [2, 2, 1], [2, 3, 1], [3, 3, 1]])
        with self.assertRaises(ValueError):
            comparator.compare_partition([1, 1, 1, 1], [1, 1, 2, 2], separated, .3)

    def test_tree_clades_use_global_cells_and_reject_a_false_traversal(self):
        comparator = self.comparator()
        first = dict(merge=[[-1, -2], [-3, -4], [1, 2]],
                     height=[1, 2, 4], order=[0, 1, 2, 3])
        expected = {frozenset((5, 2)): 1, frozenset((9, 0)): 2,
                    frozenset((5, 2, 9, 0)): 4}
        self.assertEqual(comparator.tree_clades(first, [5, 2, 9, 0]), expected)
        second = dict(merge=[[-3, -4], [-1, -2], [1, 2]],
                      height=[1, 2, 4], order=[2, 3, 0, 1])
        self.assertEqual(comparator.tree_clades(second, [9, 0, 5, 2]), expected)
        second["order"] = [0, 1, 2, 3]
        with self.assertRaises(ValueError):
            comparator.tree_clades(second, [9, 0, 5, 2])


if __name__ == "__main__":
    unittest.main()
