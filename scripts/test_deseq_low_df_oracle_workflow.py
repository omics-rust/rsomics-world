from pathlib import Path
import re
import subprocess
import unittest

from test_infercnv_ingestion_workflow import bash_body, steps


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / '.github/workflows/deseq-low-df-oracle.yml'


class DeseqLowDfOracleWorkflowTests(unittest.TestCase):
    def test_source_only_low_df_driver_reuses_hosted_release_recipe(self):
        self.assertTrue(WORKFLOW.is_file(), 'missing low-df source qualification workflow')
        self.assertTrue((ROOT / 'scripts/deseq_low_df_workflows.R').is_file())
        text = WORKFLOW.read_text()
        self.assertIn("r-version: '4.6.1'", text)
        self.assertIn('DESeq2_1.52.0.tar.gz', text)
        self.assertIn('8c91699286336350e66eec132ce6fdf5bb4af78e2a4d015a5a61224f62a95984', text)
        self.assertNotIn('continue-on-error', text)
        routed = steps(text)
        capture = bash_body(routed['Qualify fixed constants and actual low-df workflows'])
        for helper in ('deseq_workflow_oracle.R', 'deseq_design_workflows.R'):
            self.assertIn('scripts/' + helper, capture)
        self.assertIn('Rscript --vanilla scripts/deseq_low_df_workflows.R "$EVIDENCE/DESeq2" "$EVIDENCE/qualification"', capture)
        self.assertIn('if: always()', routed['Retain source and complete qualification evidence'])
        prepare = bash_body(routed['Prepare hosted source and library storage'])
        for name in ('TMPDIR', 'R_LIBS_USER', 'R_USER_CACHE_DIR', 'CARGO_HOME', 'CARGO_TARGET_DIR'):
            self.assertRegex(prepare, rf'export {name}="\$RUNNER_TEMP/')
        for name, step in routed.items():
            if '        run: |\n' in step:
                with self.subTest(step=name):
                    body = re.sub(r'\$\{\{.*?\}\}', 'stub', bash_body(step))
                    result = subprocess.run(['bash', '-n'], input=body, capture_output=True,
                                            text=True, timeout=15)
                    self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == '__main__':
    unittest.main()
