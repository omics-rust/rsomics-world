"""Unit-only contracts; miniature matrices are not upstream measurements."""

import json
import math
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest

from measure_infercnv import (
    _parse_time_file,
    _run_child,
    _trial_argv,
    _trial_order,
    summarize_trials,
    validate_pins,
    validate_trial,
)


class MeasurementContracts(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory(dir=os.environ["TMPDIR"])
        self.root = Path(self.scratch.name)
        self.expected = self.root / "expected"
        self.result = self.root / "result"
        self.expected.mkdir()
        self.result.mkdir()
        self._fixture(self.expected, "1\t2\n3\t4")
        self._fixture(self.result, "1\t2\n3\t4")
        self._metrics()

    def tearDown(self):
        self.scratch.cleanup()

    def _fixture(self, directory, values):
        (directory / "14.tsv").write_text("gene\tc1\tc2\ng1\t" + values.split("\n")[0] +
                                            "\ng2\t" + values.split("\n")[1] + "\n")
        (directory / "14.genes.tsv").write_text("gene\tchr\tstart\tstop\ng1\tchr1\t1\t2\ng2\tchr1\t3\t4\n")
        (directory / "14.cells.tsv").write_text("cell\tgroup\trole\nc1\tr\treference\nc2\to\tobservation\n")

    def _metrics(self, changes=None):
        metrics = {
            "schema_version": "1", "implementation": "rust", "preparation_wall_seconds": "0.2",
            "region_wall_seconds": "1", "region_user_seconds": "0.5",
            "region_system_seconds": "0.1", "region_child_user_seconds": "0",
            "region_child_system_seconds": "0", "baseline_rss_bytes": "4096",
        }
        metrics.update(changes or {})
        (self.result / "metrics.tsv").write_text("metric\tvalue\n" +
            "".join(f"{k}\t{v}\n" for k, v in metrics.items()))

    def valid_trials(self, rust=1.0, infercnv=2.0):
        return [dict(phase="measured", pair=pair, implementation=implementation,
                     region_wall_seconds=rust if implementation == "rust" else infercnv,
                     region_user_seconds=0.5, region_system_seconds=0.1,
                     preparation_wall_seconds=0.2, baseline_rss_bytes=4096,
                     lifetime={"wall_seconds": 5.0, "user_seconds": 2.0,
                               "system_seconds": 0.5, "maximum_rss_bytes": 8192})
                for phase, pair, implementations in _trial_order()
                if phase == "measured" for implementation in implementations]

    def test_trial_accepts_exact_unit_output(self):
        result = validate_trial(self.result, self.expected, "rust")
        self.assertEqual(result["max_absolute_error"], 0)
        self.assertEqual(len(result["output_sha256"]), 3)

    def test_trial_rejects_missing_nonfinite_wrong_identity_and_tolerance(self):
        (self.result / "14.tsv").unlink()
        with self.assertRaises(ValueError):
            validate_trial(self.result, self.expected, "rust")
        self._fixture(self.result, "1\t2\n3\t4")
        for matrix in ("NaN\t2\n3\t4", "1\t2.0001\n3\t4"):
            self._fixture(self.result, matrix)
            with self.assertRaises(ValueError):
                validate_trial(self.result, self.expected, "rust")
        self._fixture(self.result, "1\t2\n3\t4")
        (self.result / "14.cells.tsv").write_text("cell\tgroup\trole\nc1\to\tobservation\nc2\tr\treference\n")
        with self.assertRaises(ValueError):
            validate_trial(self.result, self.expected, "rust")

    def test_metrics_reject_duplicate_missing_nan_negative_child_and_bad_rss(self):
        for changes in ({"region_wall_seconds": "NaN"}, {"region_wall_seconds": "-1"},
                        {"region_child_user_seconds": "0.01"}, {"baseline_rss_bytes": "0"},
                        {"baseline_rss_bytes": "4.2"}):
            self._metrics(changes)
            with self.assertRaises(ValueError):
                validate_trial(self.result, self.expected, "rust")
        self._metrics()
        path = self.result / "metrics.tsv"
        path.write_text(path.read_text() + "region_wall_seconds\t1\n")
        with self.assertRaises(ValueError):
            validate_trial(self.result, self.expected, "rust")
        path.write_text(path.read_text().replace("region_system_seconds\t0.1\n", ""))
        with self.assertRaises(ValueError):
            validate_trial(self.result, self.expected, "rust")

    def test_time_file_requires_unique_valid_units_and_zero_exit(self):
        valid = "wall_seconds\t3.2\nuser_seconds\t2\nsystem_seconds\t0.1\nmaximum_rss_kib\t40\nexit_status\t0\n"
        for data in (valid + "wall_seconds\t3.2\n", valid.replace("40", "0"),
                     valid.replace("exit_status\t0", "exit_status\t2"),
                     valid.replace("maximum_rss_kib\t40", "maximum_rss_kib\t40 MB"),
                     valid.replace("wall_seconds\t3.2", "wall_seconds\tNaN")):
            with self.assertRaises(ValueError):
                _parse_time_file(data)
        self.assertEqual(_parse_time_file(valid)["maximum_rss_bytes"], 40960)

    def test_real_unit_child_nonzero_and_timeout(self):
        with self.assertRaises(RuntimeError):
            _run_child([sys.executable, "-B", "-c", "raise SystemExit(7)"], self.root / "failed", 5)
        with self.assertRaises(RuntimeError):
            _run_child([sys.executable, "-B", "-c", "import time; time.sleep(2)"], self.root / "timeout", 0.01)
        self.assertTrue((self.root / "failed.stderr").exists())

    def test_trial_argv_uses_full_case_only_for_rust(self):
        bundle = self.root / "bundle"
        binary = self.root / "bench"
        r_script = self.root / "trial.R"
        output = self.root / "new result"
        self.assertEqual(_trial_argv("rust", bundle, binary, r_script, output),
                         [str(binary.resolve()), str((bundle / "full").resolve()), str(output)])
        self.assertEqual(_trial_argv("infercnv", bundle, binary, r_script, output),
                         ["Rscript", "--vanilla", str(r_script.resolve()), str(bundle.resolve()), str(output)])

    def test_timeout_terminates_wrapped_descendant_and_keeps_raw_output(self):
        pidfile = self.root / "descendant.pid"
        marker = self.root / "survived"
        child = ("import os,time,pathlib; p=pathlib.Path(" + repr(str(pidfile)) + "); "
                 "p.write_text(str(os.getpid())+' '+str(os.getpgrp())); "
                 "time.sleep(2); pathlib.Path(" + repr(str(marker)) + ").write_text('survived')")
        wrapper = "import subprocess,sys; subprocess.Popen([sys.executable,'-B','-c',sys.argv[1]]).wait()"
        prefix = self.root / "wrapped"
        try:
            with self.assertRaises(RuntimeError):
                _run_child([sys.executable, "-B", "-c", wrapper, child], prefix, 1)
            self.assertTrue(pidfile.exists(), "descendant never started")
            time.sleep(2.2)
            self.assertFalse(marker.exists(), "timed-out descendant survived its wrapper")
            self.assertTrue((self.root / "wrapped.stdout").exists())
            self.assertTrue((self.root / "wrapped.stderr").exists())
        finally:
            if pidfile.exists():
                pid, group = map(int, pidfile.read_text().split())
                if group != os.getpgrp():
                    try:
                        os.killpg(group, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                else:
                    try:
                        os.kill(pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass

    def test_missing_and_duplicate_samples_fail(self):
        with self.assertRaises(ValueError):
            summarize_trials([])
        trials = self.valid_trials()
        trials[-1] = trials[0].copy()
        with self.assertRaises(ValueError):
            summarize_trials(trials)

    def test_smoke_is_not_a_measured_sample(self):
        trials = self.valid_trials()
        trials[0]["phase"] = "smoke"
        with self.assertRaises(ValueError):
            summarize_trials(trials)

    def test_alternation_and_distribution_rule(self):
        self.assertEqual(_trial_order()[0], ("smoke", 0, ("rust", "infercnv")))
        self.assertEqual(_trial_order()[2], ("measured", 2, ("infercnv", "rust")))
        self.assertTrue(summarize_trials(self.valid_trials())["clear_rust_wall_advantage"])
        self.assertFalse(summarize_trials(self.valid_trials(2.0, 2.0))["clear_rust_wall_advantage"])
        self.assertEqual(summarize_trials(self.valid_trials())["implementations"]["rust"]
                         ["baseline_rss_bytes_median"], 4096)

    def test_summary_rejects_missing_or_nonfinite_metric(self):
        trials = self.valid_trials()
        del trials[0]["region_user_seconds"]
        with self.assertRaises(ValueError):
            summarize_trials(trials)
        trials = self.valid_trials()
        trials[0]["lifetime"]["maximum_rss_bytes"] = math.nan
        with self.assertRaises(ValueError):
            summarize_trials(trials)

    def test_receipt_rejects_changed_trusted_pin(self):
        receipt = json.loads((Path(__file__).resolve().parents[1] / ".autopilot/oracles/infercnv-measurement-input-2026-09-26.json").read_text())
        receipt["files"]["full/01.tsv"] = "0" * 64
        with self.assertRaises(ValueError):
            validate_pins(self.root, receipt)


if __name__ == "__main__":
    unittest.main()
