"""Behavior tests for the independent inferCNV oracle bundle validator."""

import json
import hashlib
import math
import statistics
import tempfile
import unittest
from pathlib import Path

from validate_infercnv_oracle import validate_bundle


PROFILES = ("single_reference", "grouped_bounds", "grouped_mean", "no_reference")
STAGES = (1, 2, 3, 4, 8, 9, 10, 11, 12, 14)
COMMIT = "b421d9405c97a309b081ef86d455e976df93eae4"
CELLS = ("a1", "a2", "b1", "b2", "b3", "gain1", "gain2", "loss1", "loss2")
GROUPS = ("normal_a", "normal_a", "normal_b", "normal_b", "normal_b",
          "gain_obs", "gain_obs", "loss_obs", "loss_obs")
GENES = ("N1", "N2", "G1", "LOW", "L1", "S1")
ORDER = {"N1": ("chrNeutral", 1, 10), "N2": ("chrNeutral", 11, 20),
         "G1": ("chrGain", 1, 10), "LOW": ("chrGain", 11, 20),
         "L1": ("chrLoss", 1, 10), "S1": ("chrSolo", 1, 10)}
COUNTS = {"N1": (20,) * 9, "N2": (21,) * 9,
          "G1": (20, 20, 24, 24, 24, 40, 40, 20, 20),
          "LOW": (0,) * 9,
          "L1": (20, 20, 20, 20, 20, 5, 5, 5, 5),
          "S1": (20,) * 9,
          "ABSENT": (1,) * 9}


class BundleTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.data = {
            "schema_version": 1,
            "package": {"name": "infercnv", "version": "1.28.0", "source_commit": COMMIT,
                        "source_archive_sha256": "a" * 64},
            "seed": 260926,
            "inputs": {},
            "input_sha256": {},
            "metadata": {},
            "settings": {"cutoff": 0.1, "min_cells_per_gene": 3, "window_length": 101,
                         "smooth_method": "pyramidinal", "max_centered_threshold": 3,
                         "scale_data": False, "HMM": False, "denoise": False,
                         "analysis_mode": "samples", "num_threads": 1,
                         "resume_mode": False, "plot_steps": False, "no_plot": True,
                         "no_prelim_plot": True, "save_rds": True,
                         "remove_genes_at_chr_ends": False, "prune_outliers": False,
                         "mask_nonDE_genes": False, "up_to_step": 14,
                         "num_ref_groups": None, "cluster_by_groups": True,
                         "cluster_references": True,
                         "tumor_subcluster_partition_method": "leiden",
                         "BayesMaxPNormal": 0.5},
            "creation_settings": {"max_cells_per_group": None,
                                  "min_max_counts_per_cell": [1, "Inf"],
                                  "chr_exclude": ["chrX", "chrY", "chrM"]},
            "profiles": {},
        }
        input_text = {
            "counts.tsv": "gene\t" + "\t".join(CELLS) + "\n" + "".join(
                gene + "\t" + "\t".join(str(x) for x in COUNTS[gene]) + "\n"
                for gene in ("ABSENT", "S1", "G1", "N2", "LOW", "L1", "N1")),
            "annotations.tsv": "".join(f"{cell}\t{group}\n" for cell, group in zip(CELLS, GROUPS)),
            "gene_order.tsv": "".join(
                f"{gene}\t{ORDER[gene][0]}\t{ORDER[gene][1]}\t{ORDER[gene][2]}\n"
                for gene in GENES),
        }
        for name in ("counts.tsv", "annotations.tsv", "gene_order.tsv"):
            (self.root / name).write_text(input_text[name])
            self.data["inputs"][name] = name
            self.data["input_sha256"][name] = hashlib.sha256(input_text[name].encode()).hexdigest()
        for name in ("session.txt", "packages.tsv", "source.txt"):
            (self.root / name).write_text("evidence\n")
            self.data["metadata"][name] = name
        for profile in PROFILES:
            stages = {}
            ref_names = [] if profile == "no_reference" else (
                ["normal_a"] if profile == "single_reference" else ["normal_a", "normal_b"])
            retained = tuple(gene for gene in GENES if gene != "LOW")
            totals = [sum(COUNTS[gene][j] for gene in retained) for j in range(len(CELLS))]
            depth = statistics.median(totals)
            for stage in STAGES:
                prefix = f"{profile}/{stage:02d}"
                (self.root / profile).mkdir(exist_ok=True)
                genes = GENES if stage == 1 else retained
                values = {}
                for gene in genes:
                    raw = COUNTS[gene]
                    if stage in (1, 2):
                        values[gene] = raw
                    elif stage == 3:
                        values[gene] = tuple(raw[j] * depth / totals[j] for j in range(len(CELLS)))
                    elif stage == 4:
                        values[gene] = tuple(math.log2(raw[j] * depth / totals[j] + 1)
                                             for j in range(len(CELLS)))
                    elif stage in (8, 9, 10, 11):
                        offset = PROFILES.index(profile) * 0.1
                        values[gene] = tuple(1 + offset if gene == "S1" else
                                             1 + offset + (j % 3) * 0.01 for j in range(len(CELLS)))
                    elif stage == 12:
                        values[gene] = tuple(
                            math.log2(1.5) if gene == "G1" and GROUPS[j] == "gain_obs" else
                            -1.0 if gene == "L1" and GROUPS[j] in ("gain_obs", "loss_obs") else
                            0.0 for j in range(len(CELLS)))
                    else:
                        values[gene] = tuple(
                            1.5 if gene == "G1" and GROUPS[j] == "gain_obs" else
                            0.5 if gene == "L1" and GROUPS[j] in ("gain_obs", "loss_obs") else
                            1.0 for j in range(len(CELLS)))
                matrix = "gene\t" + "\t".join(CELLS) + "\n" + "".join(
                    gene + "\t" + "\t".join(format(x, ".17g") for x in values[gene]) + "\n"
                    for gene in genes)
                (self.root / f"{prefix}.tsv").write_text(matrix)
                (self.root / f"{prefix}.genes.tsv").write_text(
                    "gene\tchr\tstart\tstop\n" + "".join(
                        f"{gene}\t{ORDER[gene][0]}\t{ORDER[gene][1]}\t{ORDER[gene][2]}\n"
                        for gene in genes
                    )
                )
                (self.root / f"{prefix}.cells.tsv").write_text(
                    "cell\tgroup\trole\n" + "".join(
                        f"{cell}\t{group}\t{'reference' if group in ref_names else 'observation'}\n"
                        for cell, group in zip(CELLS, GROUPS))
                )
                (self.root / f"{prefix}.infercnv_obj").write_bytes(b"rds\n")
                stages[str(stage)] = {
                    "expression": f"{prefix}.tsv",
                    "gene_order": f"{prefix}.genes.tsv",
                    "cell_groups": f"{prefix}.cells.tsv",
                    "checkpoint": f"{prefix}.infercnv_obj",
                }
            self.data["profiles"][profile] = {
                "settings": {"ref_group_names": ref_names,
                             "ref_subtract_use_mean_bounds": profile != "grouped_mean"},
                "stages": stages,
            }
        self.save()

    def save(self):
        (self.root / "oracle.json").write_text(json.dumps(self.data))

    def change_expression(self, profile, stage, gene, cell, value):
        path = self.root / f"{profile}/{stage:02d}.tsv"
        rows = [line.split("\t") for line in path.read_text().splitlines()]
        rows[[row[0] for row in rows].index(gene)][rows[0].index(cell)] = str(value)
        path.write_text("\n".join("\t".join(row) for row in rows) + "\n")

    def test_accepts_complete_bundle(self):
        self.assertEqual(validate_bundle(self.root)["stages"], 40)

    def test_rejects_wrong_version(self):
        self.data["package"]["version"] = "1.23.0"
        self.save()
        with self.assertRaisesRegex(ValueError, "version"):
            validate_bundle(self.root)

    def test_rejects_malformed_package_with_value_error(self):
        self.data["package"] = None
        self.save()
        with self.assertRaisesRegex(ValueError, "package"):
            validate_bundle(self.root)

    def test_rejects_wrong_commit(self):
        self.data["package"]["source_commit"] = "0" * 40
        self.save()
        with self.assertRaisesRegex(ValueError, "commit"):
            validate_bundle(self.root)

    def test_rejects_input_hash_mismatch(self):
        self.data["input_sha256"]["counts.tsv"] = "0" * 64
        self.save()
        with self.assertRaisesRegex(ValueError, "SHA-256"):
            validate_bundle(self.root)

    def test_rejects_wrong_profile_reference_groups(self):
        self.data["profiles"]["grouped_bounds"]["settings"]["ref_group_names"] = ["normal_a"]
        self.save()
        with self.assertRaisesRegex(ValueError, "reference settings"):
            validate_bundle(self.root)

    def test_rejects_reference_role_in_no_reference_profile(self):
        path = self.root / "no_reference/01.cells.tsv"
        path.write_text(path.read_text().replace("b1\tnormal_b\tobservation",
                                                 "b1\tnormal_b\treference"))
        with self.assertRaisesRegex(ValueError, "reference role"):
            validate_bundle(self.root)

    def test_rejects_missing_profile(self):
        del self.data["profiles"]["no_reference"]
        self.save()
        with self.assertRaisesRegex(ValueError, "profile"):
            validate_bundle(self.root)

    def test_rejects_missing_stage(self):
        del self.data["profiles"]["single_reference"]["stages"]["14"]
        self.save()
        with self.assertRaisesRegex(ValueError, "stage"):
            validate_bundle(self.root)

    def test_rejects_duplicate_gene_id(self):
        path = self.root / "single_reference/01.tsv"
        path.write_text(path.read_text().replace("N2\t", "N1\t", 1))
        with self.assertRaisesRegex(ValueError, "duplicate"):
            validate_bundle(self.root)

    def test_rejects_duplicate_cell_id(self):
        path = self.root / "single_reference/01.tsv"
        path.write_text(path.read_text().replace("\ta2\t", "\ta1\t", 1))
        with self.assertRaisesRegex(ValueError, "duplicate"):
            validate_bundle(self.root)

    def test_rejects_gene_order_mismatch(self):
        path = self.root / "single_reference/01.genes.tsv"
        rows = path.read_text().splitlines()
        rows[1], rows[2] = rows[2], rows[1]
        path.write_text("\n".join(rows) + "\n")
        with self.assertRaisesRegex(ValueError, "gene order"):
            validate_bundle(self.root)

    def test_rejects_cell_order_mismatch(self):
        path = self.root / "single_reference/01.cells.tsv"
        rows = path.read_text().splitlines()
        rows[1], rows[2] = rows[2], rows[1]
        path.write_text("\n".join(rows) + "\n")
        with self.assertRaisesRegex(ValueError, "cell order"):
            validate_bundle(self.root)

    def test_rejects_ragged_matrix(self):
        path = self.root / "single_reference/01.tsv"
        rows = path.read_text().splitlines()
        rows[1] = rows[1].rsplit("\t", 1)[0]
        path.write_text("\n".join(rows) + "\n")
        with self.assertRaisesRegex(ValueError, "ragged"):
            validate_bundle(self.root)

    def test_rejects_nonfinite_matrix(self):
        self.change_expression("single_reference", 1, "N1", "a1", "NaN")
        with self.assertRaisesRegex(ValueError, "finite"):
            validate_bundle(self.root)

    def test_rejects_path_escape(self):
        self.data["profiles"]["single_reference"]["stages"]["1"]["expression"] = "../escape.tsv"
        self.save()
        with self.assertRaisesRegex(ValueError, "path"):
            validate_bundle(self.root)

    def test_rejects_missing_referenced_file(self):
        self.data["profiles"]["single_reference"]["stages"]["1"]["checkpoint"] = "missing.rds"
        self.save()
        with self.assertRaisesRegex(ValueError, "missing"):
            validate_bundle(self.root)

    def test_rejects_unchanged_normalization(self):
        path = self.root / "single_reference/03.tsv"
        path.write_bytes((self.root / "single_reference/02.tsv").read_bytes())
        with self.assertRaisesRegex(ValueError, "unchanged"):
            validate_bundle(self.root)

    def test_rejects_missing_named_reference_group(self):
        path = self.root / "grouped_bounds/01.cells.tsv"
        path.write_text(path.read_text().replace("\tnormal_b\t", "\tnormal_a\t"))
        with self.assertRaisesRegex(ValueError, "reference group"):
            validate_bundle(self.root)

    def test_rejects_identical_reference_modes(self):
        single = (self.root / "single_reference/08.tsv").read_bytes()
        for profile in ("grouped_bounds", "grouped_mean", "no_reference"):
            (self.root / f"{profile}/08.tsv").write_bytes(single)
        with self.assertRaisesRegex(ValueError, "profile contrast"):
            validate_bundle(self.root)

    def test_rejects_absent_gain_and_loss_signal(self):
        for cell in ("gain1", "gain2"):
            self.change_expression("single_reference", 12, "G1", cell, 0)
            self.change_expression("single_reference", 12, "L1", cell, 0)
            self.change_expression("single_reference", 14, "G1", cell, 1)
            self.change_expression("single_reference", 14, "L1", cell, 1)
        with self.assertRaisesRegex(ValueError, "gain/loss"):
            validate_bundle(self.root)

    def test_rejects_changed_gene_coordinate_across_stages(self):
        path = self.root / "single_reference/02.genes.tsv"
        path.write_text(path.read_text().replace("N1\tchrNeutral\t1\t10",
                                                 "N1\tchrNeutral\t2\t10"))
        with self.assertRaisesRegex(ValueError, "coordinate"):
            validate_bundle(self.root)

    def test_rejects_changed_cell_annotation_across_stages(self):
        path = self.root / "single_reference/02.cells.tsv"
        path.write_text(path.read_text().replace("gain1\tgain_obs\tobservation",
                                                 "gain1\tother\tobservation"))
        with self.assertRaisesRegex(ValueError, "annotation"):
            validate_bundle(self.root)

    def test_rejects_stage_four_without_log2_transform(self):
        self.change_expression("single_reference", 4, "G1", "gain1", 9)
        with self.assertRaisesRegex(ValueError, "log2"):
            validate_bundle(self.root)

    def test_rejects_stage_fourteen_without_inverse_log2(self):
        self.change_expression("single_reference", 14, "G1", "gain1", 1.25)
        with self.assertRaisesRegex(ValueError, "inverse"):
            validate_bundle(self.root)

    def test_rejects_unequal_normalized_depth(self):
        self.change_expression("single_reference", 3, "N1", "a1", 99)
        with self.assertRaisesRegex(ValueError, "depth"):
            validate_bundle(self.root)

    def test_rejects_changed_singleton_from_smoothing(self):
        self.change_expression("single_reference", 10, "S1", "gain1", 8)
        with self.assertRaisesRegex(ValueError, "singleton"):
            validate_bundle(self.root)

    def test_rejects_wrong_creation_settings(self):
        self.data["creation_settings"]["min_max_counts_per_cell"] = [0, "Inf"]
        self.save()
        with self.assertRaisesRegex(ValueError, "creation"):
            validate_bundle(self.root)

    def test_rejects_negative_stage_twelve_gain_median_despite_final_gain(self):
        self.change_expression("single_reference", 12, "G1", "gain1", -2)
        self.change_expression("single_reference", 12, "G1", "gain2", 1)
        self.change_expression("single_reference", 14, "G1", "gain1", 0.25)
        self.change_expression("single_reference", 14, "G1", "gain2", 2)
        with self.assertRaisesRegex(ValueError, "stage 12 gain/loss"):
            validate_bundle(self.root)

    def test_rejects_missing_explicit_run_argument_record(self):
        del self.data["settings"]["BayesMaxPNormal"]
        self.save()
        with self.assertRaisesRegex(ValueError, "oracle settings"):
            validate_bundle(self.root)


if __name__ == "__main__":
    unittest.main()
