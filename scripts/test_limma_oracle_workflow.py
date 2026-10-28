from pathlib import Path
import re
import subprocess
import unittest

from test_infercnv_ingestion_workflow import bash_body, steps


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / '.github/workflows/limma-oracle.yml'


class LimmaOracleWorkflowTests(unittest.TestCase):
    def test_current_voom_source_workflow_routes_and_retains_complete_evidence(self):
        self.assertTrue(WORKFLOW.is_file(), 'missing current limma source workflow')
        self.assertTrue((ROOT / 'scripts/limma_workflow_oracle.R').is_file())
        text = WORKFLOW.read_text()
        for source in ('edgeR_4.10.7.tar.gz', 'limma_3.68.5.tar.gz', 'DESeq2_1.52.0.tar.gz'):
            self.assertIn(source, text)
        self.assertIn("r-version: '4.6.1'", text)
        self.assertNotIn('continue-on-error', text)
        self.assertNotIn('R CMD INSTALL --preclean --clean --library="$R_LIBS_USER" "$EVIDENCE/DESeq2"', text)
        routed = steps(text)
        capture = bash_body(routed['Capture the complete current voom fitted workflow'])
        for helper in ('deseq_workflow_oracle.R', 'edger_workflow_oracle.R'):
            self.assertIn(helper, capture)
        self.assertIn('scripts/limma_workflow_oracle.R "$EVIDENCE/edgeR" "$EVIDENCE/limma" "$EVIDENCE/DESeq2" "$EVIDENCE/workflow"', capture)
        self.assertIn('if: always()', routed['Retain original sources and complete workflow evidence'])
        acquire = bash_body(routed['Acquire the exact released sources and original input carrier'])
        for member in ('edgeR/R/glmfit.R', 'edgeR/R/voomLmFit.R', 'limma/R/lmfit.R', 'limma/R/toptable.R'):
            self.assertIn(member, acquire)
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
