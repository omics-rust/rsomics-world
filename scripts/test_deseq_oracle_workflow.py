from pathlib import Path
import re
import subprocess
import unittest

from test_infercnv_ingestion_workflow import bash_body, steps


class DeseqOracleWorkflowTests(unittest.TestCase):
    def test_released_whole_call_uses_hosted_storage_and_retains_failed_evidence(self):
        root = Path(__file__).resolve().parents[1]
        self.assertTrue((root / 'scripts/deseq_workflow_oracle.R').is_file())
        text = (root / '.github/workflows/deseq-oracle.yml').read_text()
        self.assertIn("r-version: '4.6.1'", text)
        self.assertIn('DESeq2_1.52.0.tar.gz', text)
        self.assertIn('8c91699286336350e66eec132ce6fdf5bb4af78e2a4d015a5a61224f62a95984', text)
        self.assertIn('sha256sum --check', text)
        self.assertIn('if: always()', text)
        self.assertNotIn('continue-on-error', text)
        self.assertNotIn('cargo build', text)
        self.assertIn('scripts/deseq_workflow_oracle.R "$EVIDENCE/DESeq2" "$EVIDENCE/workflow"', text)
        for name, step in steps(text).items():
            if '        run: |\n' in step:
                with self.subTest(step=name):
                    body = re.sub(r'\$\{\{.*?\}\}', 'stub', bash_body(step))
                    result = subprocess.run(['bash', '-n'], input=body, capture_output=True, text=True, timeout=15)
                    self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == '__main__':
    unittest.main()
