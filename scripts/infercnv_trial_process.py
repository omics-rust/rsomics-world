"""Checked process-group ownership for serial, trusted inferCNV trial children."""

from contextlib import ExitStack
from dataclasses import dataclass
import json
import math
from numbers import Real
import os
from pathlib import Path
import signal
import subprocess
import threading
import time
from collections.abc import Sequence


@dataclass(frozen=True)
class CompletedTrial:
    argv: tuple[str, ...]
    pid: int
    returncode: int
    start_ns: int
    end_ns: int
    group_empty: bool


class TrialProcessError(RuntimeError):
    """A child failed, exceeded its limit, or did not leave an empty group."""


class TrialInterrupted(TrialProcessError):
    """A handled owner signal interrupted a child after bounded cleanup."""

    def __init__(self, signal_number):
        self.signal_number = signal_number
        super().__init__(f"trial interrupted by {signal.Signals(signal_number).name}")


def _positive_seconds(value, name):
    if isinstance(value, bool) or not isinstance(value, Real):
        raise TypeError(f"{name} must be a positive finite number")
    value = float(value)
    if not math.isfinite(value) or value <= 0:
        raise ValueError(f"{name} must be a positive finite number")
    return value


def _group_alive(pgid):
    if pgid <= 1 or pgid == os.getpgrp():
        raise TrialProcessError("refusing to inspect or signal an unowned group")
    try:
        os.killpg(pgid, 0)
    except ProcessLookupError:
        return False
    return True


def _signal_group(pgid, signum, actions):
    if not _group_alive(pgid):
        return
    action = {"signal": signal.Signals(signum).name, "time_ns": time.monotonic_ns()}
    actions.append(action)
    try:
        os.killpg(pgid, signum)
    except ProcessLookupError:
        action["already_empty"] = True


def _cleanup(proc, seconds, actions):
    deadline = time.monotonic() + seconds
    term_deadline = time.monotonic() + seconds / 2
    _signal_group(proc.pid, signal.SIGTERM, actions)
    while _group_alive(proc.pid) and time.monotonic() < term_deadline:
        proc.poll()
        time.sleep(min(0.01, max(0, term_deadline - time.monotonic())))
    if _group_alive(proc.pid):
        _signal_group(proc.pid, signal.SIGKILL, actions)
    while _group_alive(proc.pid) and time.monotonic() < deadline:
        proc.poll()
        time.sleep(min(0.01, max(0, deadline - time.monotonic())))
    proc.poll()
    if proc.returncode is None:
        proc.wait(timeout=max(0, deadline - time.monotonic()))


def _save_report(stream, report):
    stream.seek(0)
    json.dump(report, stream, sort_keys=True)
    stream.write("\n")
    stream.truncate()
    stream.flush()


def run_child(argv, prefix, timeout=1200, env=None, cleanup_timeout=5):
    """Retain exclusive raw evidence and accept only a zero exit with no group.

    Children must stay in the fresh process group; this is not a sandbox or
    subreaper. Orphan zombies can keep a group nonempty and therefore fail it.
    """
    if os.name != "posix":
        raise TrialProcessError("POSIX process groups are required")
    if isinstance(argv, (str, bytes)) or not isinstance(argv, Sequence) or not argv:
        raise TypeError("argv must be a nonempty sequence of strings")
    argv = tuple(argv)
    if not all(isinstance(value, str) for value in argv):
        raise TypeError("argv must be a nonempty sequence of strings")
    timeout = _positive_seconds(timeout, "timeout")
    cleanup_timeout = _positive_seconds(cleanup_timeout, "cleanup_timeout")
    prefix = Path(os.path.abspath(os.fspath(prefix)))
    parent = prefix.parent
    if not parent.is_dir() or parent.resolve(strict=True) != parent:
        raise ValueError("evidence prefix requires an existing real parent directory")
    destinations = {name: Path(str(prefix) + suffix) for name, suffix in (
        ("report", ".process.json"), ("stdout", ".stdout"), ("stderr", ".stderr")
    )}
    for path in destinations.values():
        if os.path.lexists(path):
            raise FileExistsError(f"evidence destination already exists: {path}")

    report = {
        "schema_version": 1, "argv": list(argv), "pid": None,
        "returncode": None, "start_ns": time.monotonic_ns(), "end_ns": None,
        "status": "launching", "timed_out": False, "interrupted_signal": None,
        "error": None, "leftovers_detected": False, "cleanup_actions": [],
        "cleanup_error": None, "final_group_alive": None, "group_empty": False,
    }
    proc = None
    failure = None
    pending_signal = None
    previous_handlers = {}

    def note_signal(signum, _frame):
        nonlocal pending_signal
        if pending_signal is None:
            pending_signal = signum

    with ExitStack() as files:
        report_file = files.enter_context(open(destinations["report"], "x", encoding="utf-8"))
        _save_report(report_file, report)
        try:
            stdout = files.enter_context(open(destinations["stdout"], "xb"))
            stderr = files.enter_context(open(destinations["stderr"], "xb"))
            if threading.current_thread() is threading.main_thread():
                for signum in (signal.SIGINT, signal.SIGTERM):
                    previous_handlers[signum] = signal.getsignal(signum)
                    signal.signal(signum, note_signal)
            try:
                proc = subprocess.Popen(argv, stdout=stdout, stderr=stderr, env=env,
                                        start_new_session=True)
                report["pid"] = proc.pid
                report["status"] = "running"
                _save_report(report_file, report)
                deadline = time.monotonic() + timeout
                while True:
                    if pending_signal is not None:
                        raise TrialInterrupted(pending_signal)
                    remaining = deadline - time.monotonic()
                    if remaining <= 0:
                        raise subprocess.TimeoutExpired(argv, timeout)
                    try:
                        proc.wait(timeout=min(0.02, remaining))
                        break
                    except subprocess.TimeoutExpired:
                        pass
            except BaseException as exc:
                failure = exc
                if isinstance(exc, TrialInterrupted):
                    report["status"] = "interrupted"
                elif isinstance(exc, subprocess.TimeoutExpired):
                    report["status"] = "timeout"
                    report["timed_out"] = True
                else:
                    report["status"] = "launch_error" if proc is None else "error"
                report["error"] = f"{type(exc).__name__}: {exc}"
        except BaseException as exc:
            failure = exc
            report["status"] = "evidence_error"
            report["error"] = f"{type(exc).__name__}: {exc}"
        finally:
            try:
                if proc is not None:
                    try:
                        report["returncode"] = proc.poll()
                        alive = _group_alive(proc.pid)
                        report["leftovers_detected"] = report["returncode"] is not None and alive
                        if alive:
                            _cleanup(proc, cleanup_timeout, report["cleanup_actions"])
                        report["returncode"] = proc.poll()
                        report["final_group_alive"] = _group_alive(proc.pid)
                        report["group_empty"] = not report["final_group_alive"]
                    except BaseException as exc:
                        report["cleanup_error"] = f"{type(exc).__name__}: {exc}"
                        failure = TrialProcessError("trial cleanup failed")
                        try:
                            report["returncode"] = proc.poll()
                            report["final_group_alive"] = _group_alive(proc.pid)
                            report["group_empty"] = not report["final_group_alive"]
                        except OSError:
                            pass
                    if failure is None:
                        if report["leftovers_detected"]:
                            report["status"] = "leftovers"
                            failure = TrialProcessError("trial parent left descendants in its group")
                        elif report["returncode"] != 0:
                            report["status"] = "nonzero"
                            failure = TrialProcessError(f"trial exited with status {report['returncode']}")
                    if not report["group_empty"]:
                        report["cleanup_error"] = report["cleanup_error"] or "process group remains nonempty"
                        if failure is None:
                            failure = TrialProcessError("trial group remains nonempty")
                if pending_signal is not None:
                    report["interrupted_signal"] = pending_signal
                    report["status"] = "interrupted"
                    failure = TrialInterrupted(pending_signal)
                if failure is None:
                    report["status"] = "completed"
                elif report["error"] is None:
                    report["error"] = f"{type(failure).__name__}: {failure}"
                report["end_ns"] = time.monotonic_ns()
                _save_report(report_file, report)
            finally:
                if previous_handlers:
                    previous_mask = signal.pthread_sigmask(signal.SIG_BLOCK, previous_handlers)
                    try:
                        for signum, handler in previous_handlers.items():
                            signal.signal(signum, handler)
                        if pending_signal is not None and report["interrupted_signal"] != pending_signal:
                            failure = TrialInterrupted(pending_signal)
                            report["interrupted_signal"] = pending_signal
                            report["status"] = "interrupted"
                            report["error"] = f"{type(failure).__name__}: {failure}"
                            report["end_ns"] = time.monotonic_ns()
                            _save_report(report_file, report)
                    finally:
                        signal.pthread_sigmask(signal.SIG_SETMASK, previous_mask)
    if failure is not None:
        if isinstance(failure, (TrialProcessError, KeyboardInterrupt, SystemExit)):
            raise failure
        raise TrialProcessError(f"trial failed: {failure}") from failure
    return CompletedTrial(argv, proc.pid, report["returncode"], report["start_ns"],
                          report["end_ns"], report["group_empty"])
