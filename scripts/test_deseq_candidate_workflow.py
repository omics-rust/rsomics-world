import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

from test_infercnv_ingestion_workflow import bash_body, steps


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / '.github/workflows/deseq-candidate.yml'


class DeseqCandidateWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.text = WORKFLOW.read_text() if WORKFLOW.exists() else ''
        cls.steps = steps(cls.text)

    def test_four_native_routes_execute_the_complete_frozen_workflow(self):
        self.assertTrue(WORKFLOW.is_file(), 'missing four-native DESeq candidate gate')
        matrix = re.search(r"fromJSON\('([^']+)'\)", self.text)
        self.assertIsNotNone(matrix, 'missing explicit native runner matrix')
        self.assertEqual(
            [(row['target'], row['system'], row['arch']) for row in json.loads(matrix[1])],
            [('linux-x86_64', 'Linux', 'x86_64'), ('linux-aarch64', 'Linux', 'aarch64'),
             ('macos-x86_64', 'Darwin', 'x86_64'), ('macos-aarch64', 'Darwin', 'arm64')])
        names = list(self.steps)
        compare = names.index('Compare complete fitted workflow')
        for prerequisite in ('Verify frozen dependencies', 'Format and strict Clippy',
                             'Test debug', 'Test release'):
            self.assertLess(names.index(prerequisite), compare)
        body = bash_body(self.steps['Compare complete fitted workflow'])
        self.assertIn('for workers in 1 4', body)
        self.assertIn('scripts/check-workflow.py', body)
        self.assertIn('scripts/check-bundle-contract.py', body)
        self.assertIn('if: always()', self.steps['Upload raw evidence'])
        self.assertNotIn('continue-on-error', self.text)
        for name, step in self.steps.items():
            if '        run: |\n' in step:
                with self.subTest(step=name):
                    body = re.sub(r'\$\{\{.*?\}\}', 'stub', bash_body(step))
                    result = subprocess.run(['bash', '-n'], input=body,
                                            capture_output=True, text=True, timeout=15)
                    self.assertEqual(result.returncode, 0, result.stderr)

    def test_worker_comparison_rejects_changed_exports_or_fitted_state(self):
        self.assertIn('Check worker determinism', self.steps, 'missing worker-output comparison')
        body = bash_body(self.steps['Check worker determinism'])
        program = re.findall(r"python3 -B - <<'PY'\n(.*?)\nPY", body, re.DOTALL)[0]
        with tempfile.TemporaryDirectory(prefix='deseq-candidate-route-') as directory:
            root = Path(directory)
            for workers in (1, 4):
                out = root / 'comparison' / f'workers-{workers}'
                out.mkdir(parents=True)
                (out / 'fit.json').write_text(json.dumps(
                    dict(workers=workers, genes=[dict(beta=[0.25, -0.5])], schema=1)))
                for name in ('forward.tsv', 'reverse.tsv', 'normalized-counts.tsv', 'vst.tsv'):
                    (out / name).write_text('gene\tvalue\ng1\t0.25\n')
            env = dict(os.environ, EVIDENCE=str(root))

            def execute():
                return subprocess.run(['python3', '-B', '-c', program], env=env,
                                      capture_output=True, text=True, timeout=15)

            good = execute()
            self.assertEqual(good.returncode, 0, good.stderr)
            self.assertTrue(json.loads((root / 'worker-determinism.json').read_text())['passed'])
            changed = root / 'comparison/workers-4/vst.tsv'
            changed.write_text('gene\tvalue\ng1\t0.75\n')
            self.assertNotEqual(execute().returncode, 0, 'changed numeric export was accepted')
            changed.write_text('gene\tvalue\ng1\t0.25\n')
            bundle = root / 'comparison/workers-4/fit.json'
            fit = json.loads(bundle.read_text())
            fit['genes'][0]['beta'][1] = 1.0
            bundle.write_text(json.dumps(fit))
            self.assertNotEqual(execute().returncode, 0, 'changed fitted covariance state was accepted')


if __name__ == '__main__':
    unittest.main()
