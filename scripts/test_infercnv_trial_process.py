"""Real, harmless process-group lifecycle contracts for private trial runs."""

import dataclasses
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest

from infercnv_trial_process import CompletedTrial, TrialInterrupted, TrialProcessError, run_child


@unittest.skipUnless(os.name == "posix", "POSIX process groups required")
class TrialProcessTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="trial process ")
        self.root = Path(self.temp.name)
        self.prefix = self.root / "evidence with spaces"

    def tearDown(self):
        try:
            groups = set()
            for report in self.root.glob("*.process.json"):
                data = json.loads(report.read_text())
                pid = data.get("pid")
                if pid:
                    groups.add(pid)
            for pid_file in self.root.glob("*.pid"):
                try:
                    groups.add(os.getpgid(int(pid_file.read_text())))
                except ProcessLookupError:
                    pass
            leaks = []
            for pgid in groups:
                if pgid > 1 and pgid != os.getpgrp():
                    live = self.live_group_members(pgid)
                    if live:
                        os.killpg(pgid, signal.SIGKILL)
                        leaks.extend(live)
            self.assertEqual(leaks, [], "trial leaked live group members")
        finally:
            self.temp.cleanup()

    @staticmethod
    def live_group_members(pgid):
        rows = subprocess.check_output(
            ["ps", "-ax", "-o", "pid=,pgid=,stat="], text=True
        ).splitlines()
        return [int(parts[0]) for row in rows
                if len(parts := row.split()) == 3
                and int(parts[1]) == pgid and not parts[2].startswith("Z")]

    def command(self, code, *args):
        return [sys.executable, "-B", "-c", code, *map(str, args)]

    def report(self, prefix=None):
        return json.loads(Path(str(prefix or self.prefix) + ".process.json").read_text())

    def run_code(self, code, *args, **kwargs):
        return run_child(self.command(code, *args), self.prefix, **kwargs)

    def test_zero_exit_exact_arguments_fresh_session_and_report_chronology(self):
        argv = self.command(
            "import json,os,sys; print(json.dumps([sys.argv[1:],os.getpid(),os.getpgrp(),os.getsid(0),os.environ['TRIAL_TOKEN']]))",
            "one argument", "literal;$dollar", "", self.root / "path with spaces",
        )
        env = dict(os.environ, TRIAL_TOKEN="value with spaces")
        before = time.monotonic_ns()
        result = run_child(argv, self.prefix, env=env)
        after = time.monotonic_ns()
        payload = json.loads(Path(str(self.prefix) + ".stdout").read_text())
        self.assertIsInstance(result, CompletedTrial)
        self.assertEqual(result.argv, tuple(argv))
        self.assertGreater(result.pid, 0)
        self.assertEqual(payload, [["one argument", "literal;$dollar", "", str(self.root / "path with spaces")],
                                   result.pid, result.pid, result.pid, "value with spaces"])
        self.assertNotEqual(result.pid, os.getpgrp())
        self.assertEqual(result.returncode, 0)
        self.assertTrue(result.group_empty)
        self.assertLessEqual(before, result.start_ns)
        self.assertLessEqual(result.start_ns, result.end_ns)
        self.assertLessEqual(result.end_ns, after)
        with self.assertRaises(dataclasses.FrozenInstanceError):
            result.pid = 3
        report = self.report()
        self.assertEqual(report["argv"], argv)
        self.assertEqual(report["pid"], result.pid)
        self.assertEqual(report["returncode"], 0)
        self.assertEqual(report["start_ns"], result.start_ns)
        self.assertEqual(report["end_ns"], result.end_ns)
        self.assertTrue(report["group_empty"])
        self.assertFalse(report["final_group_alive"])
        self.assertFalse(report["leftovers_detected"])
        self.assertEqual(report["cleanup_actions"], [])
        self.assertNotIn("value with spaces", json.dumps(report))

    def test_preserves_raw_stdout_and_stderr_on_nonzero_exit(self):
        with self.assertRaises(TrialProcessError):
            self.run_code("import os,sys; os.write(1,b'out\\x00\\xff\\r\\n'); os.write(2,b'err\\x80\\n'); sys.exit(7)")
        self.assertEqual(Path(str(self.prefix) + ".stdout").read_bytes(), b"out\x00\xff\r\n")
        self.assertEqual(Path(str(self.prefix) + ".stderr").read_bytes(), b"err\x80\n")
        report = self.report()
        self.assertEqual(report["returncode"], 7)
        self.assertEqual(report["status"], "nonzero")
        self.assertTrue(report["group_empty"])

    def test_launch_failure_has_reserved_report_and_raw_files(self):
        with self.assertRaises(TrialProcessError):
            run_child([str(self.root / "missing executable")], self.prefix)
        report = self.report()
        self.assertEqual(report["status"], "launch_error")
        self.assertIsNone(report["pid"])
        self.assertIsNone(report["returncode"])
        self.assertIn("FileNotFoundError", report["error"])
        self.assertTrue(Path(str(self.prefix) + ".stdout").is_file())
        self.assertTrue(Path(str(self.prefix) + ".stderr").is_file())
        self.assertLessEqual(report["start_ns"], report["end_ns"])

    def test_finite_timeout_kills_and_reaps_parent(self):
        start = time.monotonic()
        with self.assertRaises(TrialProcessError):
            self.run_code("import time; time.sleep(30)", timeout=0.15, cleanup_timeout=0.5)
        self.assertLess(time.monotonic() - start, 3)
        report = self.report()
        self.assertEqual(report["status"], "timeout")
        self.assertTrue(report["timed_out"])
        self.assertIsNotNone(report["returncode"])
        self.assertEqual(self.live_group_members(report["pid"]), [])
        with self.assertRaises(ChildProcessError):
            os.waitpid(report["pid"], os.WNOHANG)

    def background_code(self, status, resistant=False):
        return (
            "import os,signal,time\n"
            "pid=os.fork()\n"
            "if pid == 0:\n"
            + (" signal.signal(signal.SIGTERM,signal.SIG_IGN)\n" if resistant else "")
            + " open(" + repr(str(self.root / "background.pid")) + ",'w').write(str(os.getpid()))\n"
            " time.sleep(30)\n"
            "else:\n"
            " while not os.path.exists(" + repr(str(self.root / "background.pid")) + "): time.sleep(0.005)\n"
            " os._exit(" + str(status) + ")\n"
        )

    def test_successful_parent_background_child_is_rejected_and_cleaned(self):
        with self.assertRaises(TrialProcessError):
            self.run_code(self.background_code(0), cleanup_timeout=0.4)
        report = self.report()
        self.assertEqual(report["returncode"], 0)
        self.assertTrue(report["leftovers_detected"])
        self.assertEqual(self.live_group_members(report["pid"]), [])
        self.assertIn("SIGTERM", [action["signal"] for action in report["cleanup_actions"]])

    def test_nonzero_parent_background_child_is_also_cleaned(self):
        with self.assertRaises(TrialProcessError):
            self.run_code(self.background_code(11), cleanup_timeout=0.4)
        report = self.report()
        self.assertEqual(report["returncode"], 11)
        self.assertTrue(report["leftovers_detected"])
        self.assertEqual(self.live_group_members(report["pid"]), [])

    def test_term_resistant_background_child_is_killed_within_bound(self):
        start = time.monotonic()
        with self.assertRaises(TrialProcessError):
            self.run_code(self.background_code(0, resistant=True), cleanup_timeout=0.4)
        self.assertLess(time.monotonic() - start, 3)
        report = self.report()
        self.assertEqual([item["signal"] for item in report["cleanup_actions"]], ["SIGTERM", "SIGKILL"])
        self.assertEqual(self.live_group_members(report["pid"]), [])

    def signal_case(self, signum):
        ready = self.root / "ready.pid"
        code = (
            "import os,signal,time\n"
            "signal.signal(signal.SIGTERM,signal.SIG_IGN)\n"
            "pid=os.fork()\n"
            "if pid == 0: time.sleep(30)\n"
            "else:\n"
            " open(" + repr(str(ready)) + ",'w').write(str(pid))\n"
            " time.sleep(30)\n"
        )
        wrapper = (
            "import signal,sys\n"
            "from infercnv_trial_process import run_child,TrialInterrupted\n"
            "before={s:signal.getsignal(s) for s in (signal.SIGINT,signal.SIGTERM)}\n"
            "try: run_child(" + repr(self.command(code)) + "," + repr(str(self.prefix)) + ",cleanup_timeout=0.5)\n"
            "except TrialInterrupted as exc:\n"
            " assert exc.signal_number == " + str(signum) + "\n"
            " assert all(signal.getsignal(s)==h for s,h in before.items())\n"
            " sys.exit(43)\n"
            "sys.exit(9)\n"
        )
        owner = subprocess.Popen(self.command(wrapper), cwd=Path(__file__).parent)
        try:
            deadline = time.monotonic() + 5
            while not ready.exists() and owner.poll() is None and time.monotonic() < deadline:
                time.sleep(0.01)
            self.assertTrue(ready.exists(), "harmless child did not become ready")
            os.kill(owner.pid, signum)
            self.assertEqual(owner.wait(timeout=4), 43)
        finally:
            if owner.poll() is None:
                if ready.exists():
                    try:
                        pgid = os.getpgid(int(ready.read_text()))
                        if pgid > 1 and pgid != os.getpgrp():
                            os.killpg(pgid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                owner.kill()
                owner.wait()
        report = self.report()
        self.assertEqual(report["status"], "interrupted")
        self.assertEqual(report["interrupted_signal"], signum)
        self.assertEqual(report["returncode"], -signal.SIGKILL)
        self.assertEqual([item["signal"] for item in report["cleanup_actions"]], ["SIGTERM", "SIGKILL"])
        self.assertEqual(self.live_group_members(report["pid"]), [])

    def test_sigint_cleans_parent_and_descendant_and_restores_handlers(self):
        self.signal_case(signal.SIGINT)

    def test_sigterm_cleans_parent_and_descendant_and_restores_handlers(self):
        self.signal_case(signal.SIGTERM)

    def test_signal_handlers_restored_on_success_and_failure(self):
        before = {s: signal.getsignal(s) for s in (signal.SIGINT, signal.SIGTERM)}
        self.run_code("pass")
        self.assertEqual({s: signal.getsignal(s) for s in before}, before)
        with self.assertRaises(TrialProcessError):
            run_child(["/definitely/missing/trial-program"], self.root / "failure")
        self.assertEqual({s: signal.getsignal(s) for s in before}, before)

    def test_rejects_existing_or_symlink_evidence_without_reading_or_launching(self):
        for index, suffix in enumerate((".stdout", ".stderr", ".process.json")):
            for symlink in (False, True):
                with self.subTest(suffix=suffix, symlink=symlink):
                    prefix = self.root / f"collision {index} {symlink}"
                    destination = Path(str(prefix) + suffix)
                    if symlink:
                        destination.symlink_to(self.root / "does not exist")
                    else:
                        destination.write_bytes(b"KEEP\x00\xff")
                    marker = self.root / "launched"
                    with self.assertRaises((FileExistsError, TrialProcessError)):
                        run_child(self.command("import pathlib,sys; pathlib.Path(sys.argv[1]).touch()", marker), prefix)
                    self.assertFalse(marker.exists())
                    if not symlink:
                        self.assertEqual(destination.read_bytes(), b"KEEP\x00\xff")
                    self.assertEqual(len(list(self.root.glob(prefix.name + ".*"))), 1)
                    if suffix == ".process.json":
                        destination.unlink()

    def test_rejects_invalid_argv_and_time_bounds_before_evidence(self):
        for argv in ([], "not a sequence", b"bytes", [sys.executable, 1], [None]):
            with self.subTest(argv=argv), self.assertRaises((TypeError, ValueError)):
                run_child(argv, self.prefix)
        for name in ("timeout", "cleanup_timeout"):
            for value in (0, -1, True, False, float("inf"), float("nan"), "1", None, 1j):
                with self.subTest(name=name, value=value), self.assertRaises((TypeError, ValueError)):
                    run_child([sys.executable, "-B", "-c", "pass"], self.prefix, **{name: value})
        self.assertEqual(list(self.root.iterdir()), [])

    def test_rejects_missing_or_symlink_parent(self):
        link = self.root / "parent link"
        link.symlink_to(self.root, target_is_directory=True)
        for parent in (self.root / "missing", link):
            with self.subTest(parent=parent), self.assertRaises((ValueError, OSError)):
                run_child(self.command("pass"), parent / "prefix")


if __name__ == "__main__":
    unittest.main()
