from dataclasses import replace
import math
import os
from pathlib import Path
import tempfile
import unittest

from infercnv_ingestion_campaign import (
    Sample, Slot, accept_campaign, expected_schedule, parse_region_metrics,
)


METRICS = {
    "schema_version": "1", "measurement": "raw_creation",
    "implementation": "rust", "input_case": "plain",
    "preparation_wall_seconds": "0", "region_wall_seconds": "1.25e+0",
    "region_user_seconds": "0.5", "region_system_seconds": "0.125",
    "region_child_user_seconds": "0", "region_child_system_seconds": "0",
    "baseline_rss_bytes": "1", "returned_rss_bytes": "18446744073709551615",
}


def table(changes=None):
    values = dict(METRICS)
    values.update(changes or {})
    return "key\tvalue\n" + "".join(f"{key}\t{value}\n" for key, value in values.items())


class MetricTests(unittest.TestCase):
    def test_all_twelve_fields_keep_typed_values_and_u64_precision(self):
        result = parse_region_metrics(table())
        self.assertEqual(result.schema_version, 1)
        self.assertIs(type(result.schema_version), int)
        self.assertEqual((result.measurement, result.implementation, result.input_case),
                         ("raw_creation", "rust", "plain"))
        self.assertEqual(result.preparation_wall_seconds, 0)
        self.assertEqual(result.region_wall_seconds, 1.25)
        self.assertEqual(result.region_user_seconds, 0.5)
        self.assertEqual(result.region_system_seconds, 0.125)
        self.assertEqual(result.region_child_user_seconds, 0)
        self.assertEqual(result.region_child_system_seconds, 0)
        self.assertEqual(result.baseline_rss_bytes, 1)
        self.assertEqual(result.returned_rss_bytes, 2**64 - 1)
        self.assertIs(type(result.returned_rss_bytes), int)

    def test_r_and_gzip_literals_are_accepted(self):
        result = parse_region_metrics(table({"implementation": "infercnv", "input_case": "gzip"}))
        self.assertEqual((result.implementation, result.input_case), ("infercnv", "gzip"))

    def test_numeric_plain_and_scientific_forms_are_accepted(self):
        for spelling, want in ((".25", .25), ("1.", 1.), ("+2E-1", .2), ("01.5", 1.5)):
            with self.subTest(spelling=spelling):
                self.assertEqual(parse_region_metrics(table({"region_wall_seconds": spelling})).region_wall_seconds, want)

    def test_wrong_header_blank_rows_and_column_counts_are_rejected(self):
        for text in (table().replace("key\tvalue", "metric\tvalue", 1),
                     table().replace("key\tvalue", '"key"\tvalue', 1),
                     table() + "\n", table() + "extra\t1\t2\n", table() + "extra\n"):
            with self.subTest(text=text), self.assertRaises(ValueError):
                parse_region_metrics(text)

    def test_missing_each_field_is_rejected(self):
        for key in METRICS:
            text = "key\tvalue\n" + "".join(f"{k}\t{v}\n" for k, v in METRICS.items() if k != key)
            with self.subTest(key=key), self.assertRaises(ValueError):
                parse_region_metrics(text)

    def test_duplicate_and_unknown_keys_are_rejected(self):
        for text in (table() + "region_wall_seconds\t1\n", table() + "unknown\t1\n",
                     table().replace("region_wall_seconds\t", '"region_wall_seconds"\t')):
            with self.subTest(text=text), self.assertRaises(ValueError):
                parse_region_metrics(text)

    def test_literal_schema_measurement_case_and_implementation_are_exact(self):
        for key, values in {
            "schema_version": ("true", "01", "1.0", '"1"', "2"),
            "measurement": ("prepared", '"raw_creation"', "raw_creation "),
            "implementation": ("R", "Rust", '"rust"', " rust"),
            "input_case": ("full", "Plain", '"plain"', "plain "),
        }.items():
            for value in values:
                with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                    parse_region_metrics(table({key: value}))

    def test_every_duration_rejects_nonfinite_negative_and_nonliteral_numbers(self):
        for key in (k for k in METRICS if k.endswith("_seconds")):
            for value in ("NaN", "nan", "Inf", "-Inf", "1e309", "-1", '"0"', "0_0",
                          " 0", "0 ", "\u00a00", "\u0660", "0x0", "", "0\r0"):
                with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                    parse_region_metrics(table({key: value}))

    def test_region_wall_must_be_positive_including_underflow(self):
        for value in ("0", "-0", "0e10", "1e-9999"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                parse_region_metrics(table({"region_wall_seconds": value}))

    def test_zero_self_cpu_preparation_and_signed_zero_child_are_accepted(self):
        result = parse_region_metrics(table({key: "-0" for key in METRICS
                                             if key.endswith("_seconds") and key != "region_wall_seconds"}))
        self.assertEqual(result.region_user_seconds, 0)
        self.assertEqual(result.region_child_system_seconds, 0)

    def test_each_nonzero_child_counter_is_rejected(self):
        for key in ("region_child_user_seconds", "region_child_system_seconds"):
            for value in ("1e-100", "0.01", "1"):
                with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                    parse_region_metrics(table({key: value}))

    def test_nonzero_duration_underflow_cannot_be_mislabeled_zero(self):
        for key in (k for k in METRICS if k.endswith("_seconds")):
            for value in ("1e-9999", "-1e-9999"):
                with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                    parse_region_metrics(table({key: value}))
        result = parse_region_metrics(table({"region_child_user_seconds": "0e-9999"}))
        self.assertEqual(result.region_child_user_seconds, 0)

    def test_each_rss_rejects_zero_overflow_and_non_ascii_integer_spellings(self):
        for key in ("baseline_rss_bytes", "returned_rss_bytes"):
            for value in ("0", "18446744073709551616", "-1", "+1", "1.0", "1e3",
                          '"1"', " 1", "1 ", "\u0661", "1_000", ""):
                with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                    parse_region_metrics(table({key: value}))

    def test_field_order_is_not_interpreted_as_value_order(self):
        rows = table().splitlines()
        result = parse_region_metrics("\n".join([rows[0], *reversed(rows[1:])]) + "\n")
        self.assertEqual((result.region_wall_seconds, result.returned_rss_bytes), (1.25, 2**64 - 1))


class CampaignTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir=os.environ["TMPDIR"])
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.full = self.root / "expected"
        self.full.mkdir()
        self.maps = self.root / "expected.maps.tsv"
        self.payload = {
            "expression.tsv": 'gene\t"c_a"\tc_obs\tc_b\n"g1"\t1\t2\t3\ng2\t4\t5\t6\n',
            "genes.tsv": 'gene\tchr\tstart\tstop\n"g1"\tchr1\t1\t2\ng2\tchr2\t3\t4\n',
            "cells.tsv": 'cell\tgroup\trole\n"c_a"\t"r_a"\treference\nc_obs\tobs\tobservation\nc_b\tr_b\treference\n',
            "maps.tsv": 'role\tgroup\tindex\tcell\nreference\tr_b\t3\tc_b\nreference\t"r_a"\t1\t"c_a"\nobservation\tobs\t2\tc_obs\n',
        }
        for filename, source in (("01.tsv", "expression.tsv"), ("01.genes.tsv", "genes.tsv"),
                                 ("01.cells.tsv", "cells.tsv")):
            (self.full / filename).write_text(self.payload[source], encoding="utf-8")
        self.maps.write_text(self.payload["maps.tsv"], encoding="utf-8")
        self.samples = []
        for case in ("plain", "gzip"):
            for pair in range(8):
                implementations = ("rust", "infercnv") if pair == 0 or pair % 2 else ("infercnv", "rust")
                for order, implementation in enumerate(implementations, 1):
                    directory = self.root / f"{case}-{pair}-{implementation}"
                    directory.mkdir()
                    for phase in ("warm", "measured"):
                        result = directory / phase
                        result.mkdir()
                        for name, text in self.payload.items():
                            (result / name).write_text(text, encoding="utf-8")
                    wall = pair if implementation == "rust" else pair + 10
                    if pair == 0:
                        wall = 999 if implementation == "rust" else 0.001
                    (directory / "metrics.tsv").write_text(table({"input_case": case,
                        "implementation": implementation, "region_wall_seconds": str(wall)}), encoding="utf-8")
                    self.samples.append(Sample(Slot(case, pair, order, pair == 0, implementation), directory, 0))

    def accept(self, samples=None):
        return accept_campaign(self.samples if samples is None else samples, self.full, self.maps)

    def change_metrics(self, sample, changes):
        path = sample.trial_dir / "metrics.tsv"
        rows = dict(row.split("\t") for row in path.read_text(encoding="utf-8").splitlines()[1:])
        rows.update(changes)
        path.write_text(table(rows), encoding="utf-8")

    def test_schedule_matches_independent_ordered_cases_pairs_and_pilots(self):
        self.assertEqual(tuple(sample.slot for sample in self.samples), expected_schedule())
        self.assertEqual(len(expected_schedule()), 32)
        self.assertEqual([(s.input_case, s.pair, s.order, s.pilot, s.implementation)
                          for s in expected_schedule()[:8]],
                         [("plain", 0, 1, True, "rust"), ("plain", 0, 2, True, "infercnv"),
                          ("plain", 1, 1, False, "rust"), ("plain", 1, 2, False, "infercnv"),
                          ("plain", 2, 1, False, "infercnv"), ("plain", 2, 2, False, "rust"),
                          ("plain", 3, 1, False, "rust"), ("plain", 3, 2, False, "infercnv")])

    def test_complete_campaign_checks_64_states_and_excludes_pilots_from_summaries(self):
        result = self.accept()
        self.assertEqual((len(result.trials), result.validated_results, result.measured_samples), (32, 64, 28))
        self.assertTrue(result.strict_wall_advantage)
        self.assertEqual([case.input_case for case in result.cases], ["plain", "gzip"])
        for case in result.cases:
            self.assertEqual(case.rust["region_wall_seconds"].raw, (1., 2., 3., 4., 5., 6., 7.))
            self.assertEqual(case.infercnv["region_wall_seconds"].raw, (11., 12., 13., 14., 15., 16., 17.))
            self.assertEqual((case.rust["region_wall_seconds"].median, case.infercnv["region_wall_seconds"].median), (4., 14.))
            self.assertEqual((case.signed_wall_gap_seconds, case.wall_overlap_seconds), (4., 0.))
            self.assertFalse(case.wall_ranges_overlap)
            self.assertEqual(case.rust["returned_rss_bytes"].median, 2**64 - 1)
        for trial in result.trials:
            self.assertEqual(tuple(trial.creation_checks), ("warm", "measured"))
            for check in trial.creation_checks.values():
                self.assertEqual((check["genes"], check["cells"]), (2, 3))
                self.assertEqual(set(check["output_sha256"]), set(self.payload))

    def test_omitted_extra_duplicate_reordered_and_empty_processes_are_rejected(self):
        for samples in ([], self.samples[:-1], self.samples[1:], self.samples + self.samples[:1],
                        [self.samples[1], self.samples[0], *self.samples[2:]],
                        [*self.samples[:10], self.samples[9], *self.samples[11:]],
                        self.samples[16:] + self.samples[:16]):
            with self.subTest(count=len(samples)), self.assertRaises(ValueError):
                self.accept(samples)

    def test_wrong_slot_identity_order_pilot_or_pair_is_rejected(self):
        for slot in (replace(self.samples[0].slot, input_case="gzip"),
                     replace(self.samples[0].slot, pair=1), replace(self.samples[0].slot, order=2),
                     replace(self.samples[0].slot, pilot=False), replace(self.samples[0].slot, implementation="infercnv")):
            with self.subTest(slot=slot), self.assertRaises(ValueError):
                self.accept([replace(self.samples[0], slot=slot), *self.samples[1:]])

    def test_bool_integer_aliases_non_bool_pilots_and_non_literal_ids_are_rejected(self):
        for changes in ({"pair": False}, {"order": True}, {"pair": 0.0}, {"order": 1.0},
                        {"pilot": 1}, {"input_case": b"plain"}, {"implementation": b"rust"}):
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                self.accept([replace(self.samples[0], slot=replace(self.samples[0].slot, **changes)), *self.samples[1:]])

    def test_failed_or_invalid_exit_status_cannot_be_dropped_even_for_pilot(self):
        for index in (0, 15, 31):
            for status in (1, -9, False, 0.0, "0"):
                changed = list(self.samples)
                changed[index] = replace(changed[index], exit_status=status)
                with self.subTest(index=index, status=status), self.assertRaises(ValueError):
                    self.accept(changed)

    def test_mixed_metrics_case_or_implementation_is_rejected(self):
        sample = self.samples[0]
        for key, value in (("input_case", "gzip"), ("implementation", "infercnv")):
            original = (sample.trial_dir / "metrics.tsv").read_text(encoding="utf-8")
            self.change_metrics(sample, {key: value})
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.accept()
            (sample.trial_dir / "metrics.tsv").write_text(original, encoding="utf-8")

    def test_missing_or_malformed_metrics_file_fails(self):
        path = self.samples[0].trial_dir / "metrics.tsv"
        original = path.read_text(encoding="utf-8")
        path.unlink()
        with self.assertRaises((ValueError, OSError)):
            self.accept()
        path.write_text(original + "unknown\t0\n", encoding="utf-8")
        with self.assertRaises(ValueError):
            self.accept()

    def test_duplicate_canonical_directories_and_symlink_aliases_fail(self):
        for directory in (self.samples[0].trial_dir, self.samples[0].trial_dir / ".." / self.samples[0].trial_dir.name):
            changed = list(self.samples)
            changed[1] = replace(changed[1], trial_dir=directory)
            with self.subTest(directory=directory), self.assertRaises(ValueError):
                self.accept(changed)
        alias = self.root / "alias"
        alias.symlink_to(self.samples[0].trial_dir, target_is_directory=True)
        changed = list(self.samples)
        changed[1] = replace(changed[1], trial_dir=alias)
        with self.assertRaises(ValueError):
            self.accept(changed)

    def test_single_symlink_trial_and_symlink_metrics_are_rejected(self):
        alias = self.root / "alias"
        alias.symlink_to(self.samples[0].trial_dir, target_is_directory=True)
        changed = [replace(self.samples[0], trial_dir=alias), *self.samples[1:]]
        with self.assertRaises(ValueError):
            self.accept(changed)
        path = self.samples[0].trial_dir / "metrics.tsv"
        destination = self.root / "metrics-copy.tsv"
        path.rename(destination)
        path.symlink_to(destination)
        with self.assertRaises(ValueError):
            self.accept()

    def test_every_process_both_results_are_required_including_pilots(self):
        for sample in self.samples:
            for phase in ("warm", "measured"):
                path = sample.trial_dir / phase / "maps.tsv"
                text = path.read_text(encoding="utf-8")
                path.unlink()
                with self.subTest(slot=sample.slot, phase=phase), self.assertRaises((ValueError, OSError)):
                    self.accept()
                path.write_text(text, encoding="utf-8")

    def test_exact_checker_rejects_one_ulp_signed_zero_and_ordered_map_mutations(self):
        directory = self.samples[-1].trial_dir / "measured"
        path = directory / "expression.tsv"
        original = path.read_text(encoding="utf-8")
        path.write_text(original.replace('"g1"\t1\t', f'"g1"\t{math.nextafter(1., math.inf)}\t'), encoding="utf-8")
        with self.assertRaises(ValueError):
            self.accept()
        path.write_text(original, encoding="utf-8")
        maps = directory / "maps.tsv"
        rows = self.payload["maps.tsv"].splitlines(keepends=True)
        maps.write_text(rows[0] + rows[2] + rows[1] + rows[3], encoding="utf-8")
        with self.assertRaises(ValueError):
            self.accept()
        maps.write_text(self.payload["maps.tsv"], encoding="utf-8")
        zero = original.replace('"g1"\t1\t', '"g1"\t0\t')
        (self.full / "01.tsv").write_text(zero, encoding="utf-8")
        for sample in self.samples:
            for phase in ("warm", "measured"):
                (sample.trial_dir / phase / "expression.tsv").write_text(zero, encoding="utf-8")
        self.assertTrue(self.accept().strict_wall_advantage)
        path.write_text(zero.replace('"g1"\t0\t', '"g1"\t-0\t'), encoding="utf-8")
        with self.assertRaises(ValueError):
            self.accept()

    def test_only_one_case_failing_cannot_pass_overall(self):
        self.change_metrics(self.samples[-1], {"region_wall_seconds": "7"})
        result = self.accept()
        self.assertTrue(result.cases[0].strict_wall_advantage)
        self.assertFalse(result.cases[1].strict_wall_advantage)
        self.assertFalse(result.strict_wall_advantage)

    def test_equal_boundary_is_failure_even_with_zero_overlap_width(self):
        self.change_metrics(self.samples[3], {"region_wall_seconds": "7"})
        case = self.accept().cases[0]
        self.assertFalse(case.strict_wall_advantage)
        self.assertEqual((case.signed_wall_gap_seconds, case.wall_overlap_seconds), (0., 0.))
        self.assertTrue(case.wall_ranges_overlap)

    def test_overlap_is_reported_without_tolerance_or_resource_fallback(self):
        self.change_metrics(self.samples[3], {"region_wall_seconds": "6"})
        for sample in self.samples:
            if sample.slot.implementation == "rust":
                self.change_metrics(sample, {"baseline_rss_bytes": "1", "returned_rss_bytes": "1"})
        case = self.accept().cases[0]
        self.assertFalse(case.strict_wall_advantage)
        self.assertEqual((case.signed_wall_gap_seconds, case.wall_overlap_seconds), (-1., 1.))
        self.assertTrue(case.wall_ranges_overlap)

    def test_one_ulp_positive_gap_passes_without_epsilon(self):
        self.change_metrics(self.samples[3], {"region_wall_seconds": str(math.nextafter(7., math.inf))})
        case = self.accept().cases[0]
        self.assertTrue(case.strict_wall_advantage)
        self.assertGreater(case.signed_wall_gap_seconds, 0)

    def test_rust_slower_disjoint_ranges_fail_despite_zero_overlap_width(self):
        for sample in self.samples:
            if sample.slot.input_case == "plain" and sample.slot.implementation == "rust":
                self.change_metrics(sample, {"region_wall_seconds": str(30 + sample.slot.pair)})
        case = self.accept().cases[0]
        self.assertFalse(case.strict_wall_advantage)
        self.assertFalse(case.wall_ranges_overlap)
        self.assertEqual(case.wall_overlap_seconds, 0)

    def test_preparation_cpu_and_rss_are_descriptive_not_alternate_gate(self):
        for sample in self.samples:
            if sample.slot.implementation == "rust":
                self.change_metrics(sample, {"preparation_wall_seconds": "9999", "region_user_seconds": "9999",
                    "region_system_seconds": "9999", "baseline_rss_bytes": str(2**64 - 1), "returned_rss_bytes": str(2**64 - 1)})
        result = self.accept()
        self.assertTrue(result.strict_wall_advantage)
        self.assertEqual(result.cases[0].rust["preparation_wall_seconds"].median, 9999)
        self.assertEqual(result.cases[0].rust["region_system_seconds"].median, 9999)


if __name__ == "__main__":
    unittest.main()
