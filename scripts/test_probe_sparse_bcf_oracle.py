from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

from probe_sparse_bcf_oracle import run


class ProbeTests(unittest.TestCase):
    @patch("probe_sparse_bcf_oracle.subprocess.run")
    def test_signal_exit_is_retained(self, execute):
        execute.return_value = subprocess.CompletedProcess(["oracle"], -11, "partial", "fault")
        root = Path("/Volumes/KIOXIA/Developments/tmp")
        result = run([Path("/oracle"), "index"], root)
        self.assertEqual(result["returncode"], -11)
        self.assertEqual(result["stdout"], "partial")
        self.assertEqual(result["stderr"], "fault")
        self.assertEqual(result["status"], "completed")
        execute.assert_called_once_with([Path("/oracle"), "index"], capture_output=True, text=True, timeout=10, cwd=root)

    @patch("probe_sparse_bcf_oracle.subprocess.run")
    def test_timeout_preserves_partial_output(self, execute):
        execute.side_effect = subprocess.TimeoutExpired(["oracle"], 10, output=b"partial\xff", stderr=b"waiting")
        result = run(["oracle"], Path("/Volumes/KIOXIA/Developments/tmp"))
        self.assertEqual(result["status"], "timeout")
        self.assertIsNone(result["returncode"])
        self.assertEqual(result["stdout"], "partial\ufffd")
        self.assertEqual(result["stderr"], "waiting")

    @patch("probe_sparse_bcf_oracle.subprocess.run")
    def test_timeout_without_output_is_recorded(self, execute):
        execute.side_effect = subprocess.TimeoutExpired(["oracle"], 10)
        result = run(["oracle"], Path("/Volumes/KIOXIA/Developments/tmp"))
        self.assertEqual(result["stdout"], "")
        self.assertEqual(result["stderr"], "")


if __name__ == "__main__":
    unittest.main()
