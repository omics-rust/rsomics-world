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
GENERAL = ['interaction-ratio-formula', 'interaction-ratio-matrix', 'interaction-sf-formula',
           'interaction-nf-formula', 'interaction-sf-nf-formula', 'nointercept-sf-formula',
           'intercept-sf-formula']
DESIGN_CASES = GENERAL + [f'cook-{size}-{replacement}-{route}'
                          for size in ('7x7', '7x3')
                          for replacement in ('default', 'disabled')
                          for route in ('formula', 'matrix')]


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
        self.assertIn('scripts/check-design-workflows.py', body)
        self.assertIn('--include-disabled-cook', body)
        self.assertIn('--include-default-cook', body)
        self.assertIn('--witness "$DESIGN_ORACLE_ROOT"', body)
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
                    dict(workers=workers, genes=[dict(beta=[0.25, -0.5])], schema=3)))
                for name in ('forward.tsv', 'reverse.tsv', 'normalized-counts.tsv', 'vst.tsv'):
                    (out / name).write_text('gene\tvalue\ng1\t0.25\n')
                design = root / 'design-comparison' / f'workers-{workers}'
                design.mkdir(parents=True)
                (design / 'comparison-contract.json').write_text(json.dumps(
                    dict(workers=workers, cases=DESIGN_CASES, excluded_cases=[])))
                for case in DESIGN_CASES:
                    target = design / case
                    target.mkdir()
                    (target / 'fit.json').write_text(json.dumps(
                        dict(workers=workers, genes=[dict(beta=[0.25, -0.5])], schema=3)))
                    exports = ['forward.tsv', 'reverse.tsv', 'normalized-counts.tsv', 'vst.tsv']
                    if case != 'intercept-sf-formula':
                        exports.append('mixed.tsv')
                    for name in exports:
                        (target / name).write_text('gene\tvalue\ng1\t0.25\n')
            env = dict(os.environ, EVIDENCE=str(root))

            def execute():
                return subprocess.run(['python3', '-B', '-c', program], env=env,
                                      capture_output=True, text=True, timeout=15)

            good = execute()
            self.assertEqual(good.returncode, 0, good.stderr)
            report = json.loads((root / 'worker-determinism.json').read_text())
            self.assertTrue(report['passed'])
            self.assertEqual(set(report['design_cases']), set(DESIGN_CASES))
            changed = root / 'comparison/workers-4/vst.tsv'
            changed.write_text('gene\tvalue\ng1\t0.75\n')
            self.assertNotEqual(execute().returncode, 0, 'changed numeric export was accepted')
            changed.write_text('gene\tvalue\ng1\t0.25\n')
            for replacement in ('disabled', 'default'):
                changed = root / f'design-comparison/workers-4/cook-7x3-{replacement}-formula/mixed.tsv'
                changed.write_text('gene\tvalue\ng1\t0.75\n')
                self.assertNotEqual(execute().returncode, 0,
                                    f'changed {replacement}-Cook export was accepted')
                changed.write_text('gene\tvalue\ng1\t0.25\n')
            bundle = root / 'design-comparison/workers-4/interaction-nf-formula/fit.json'
            fit = json.loads(bundle.read_text())
            fit['genes'][0]['beta'][1] = 1.0
            bundle.write_text(json.dumps(fit))
            self.assertNotEqual(execute().returncode, 0, 'changed design fitted state was accepted')


if __name__ == '__main__':
    unittest.main()
