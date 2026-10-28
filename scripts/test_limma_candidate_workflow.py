import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

from test_infercnv_ingestion_workflow import bash_body, steps

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / '.github/workflows/limma-candidate.yml'


class LimmaCandidateWorkflowTests(unittest.TestCase):
    def test_four_native_routes_preserve_complete_source_gate_and_failures(self):
        text = WORKFLOW.read_text()
        matrix = re.search(r"fromJSON\('([^']+)'\)", text)
        self.assertIsNotNone(matrix)
        self.assertEqual([(r['target'], r['system'], r['arch']) for r in json.loads(matrix[1])],
                         [('linux-x86_64', 'Linux', 'x86_64'), ('linux-aarch64', 'Linux', 'aarch64'),
                          ('macos-x86_64', 'Darwin', 'x86_64'), ('macos-aarch64', 'Darwin', 'arm64')])
        routed = steps(text)
        order = list(routed)
        for name in ('Verify frozen dependencies', 'Format and strict Clippy', 'Test debug', 'Test release'):
            self.assertLess(order.index(name), order.index('Compare complete fitted workflow'))
        acquisition = bash_body(routed['Acquire and verify actual current-voom evidence'])
        self.assertIn('limma-first-workflow-2026-10-28.json', acquisition)
        self.assertIn('run_id=40606516204', acquisition)
        self.assertIn('artifact_id=12780957275', acquisition)
        self.assertIn('len(rows) == 203', acquisition)
        comparison = bash_body(routed['Compare complete fitted workflow'])
        self.assertIn('for workers in 1 4', comparison)
        self.assertIn('scripts/check-workflow.py', comparison)
        self.assertIn('--witness "$ORACLE_ROOT"', comparison)
        self.assertNotIn('Rscript', comparison)
        self.assertIn('if: always()', routed['Upload raw evidence'])
        self.assertNotIn('continue-on-error', text)
        for name, step in routed.items():
            if '        run: |\n' in step:
                with self.subTest(step=name):
                    bash = re.sub(r'\$\{\{.*?\}\}', 'stub', bash_body(step))
                    result = subprocess.run(['bash', '-n'], input=bash, text=True,
                                            capture_output=True, timeout=15)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    for program in re.findall(r"python3 -B - <<'PY'\n(.*?)\nPY", bash, re.DOTALL):
                        compile(program, name, 'exec')

    def test_worker_gate_checks_full_state_contrast_state_and_exports(self):
        program = re.findall(r"python3 -B - <<'PY'\n(.*?)\nPY",
                             bash_body(steps(WORKFLOW.read_text())['Check worker determinism']), re.DOTALL)[0]
        bounds = dict(tmm=[1e-10, 1e-10], library=[1e-8, 1e-10], expression=[1e-10, 1e-10],
                      linear=[1e-7, 1e-6], trend=[1e-7, 1e-6], weights=[1e-6, 1e-5],
                      eb=[1e-6, 1e-4], df=[1e-3, 1e-4], odds=[1e-5, 5e-4], probability=[1e-12, 5e-4])
        with tempfile.TemporaryDirectory(prefix='limma-native-route-') as directory:
            root = Path(directory)
            (root / 'native-binary').mkdir()
            (root / 'native-binary/rsomics-limma-linux-x86_64').write_bytes(b'fixture')
            digest = hashlib.sha256(b'fixture').hexdigest()
            for workers in (1, 4):
                out = root / 'comparison' / f'workers-{workers}'
                out.mkdir(parents=True)
                state = dict(schema=1, product='rsomics-limma', workers=workers, base=dict(prior=0.25))
                bundle = json.dumps(state).encode()
                (out / 'fit.json').write_bytes(bundle)
                (out / 'comparison-contract.json').write_text(json.dumps(
                    dict(workers=workers, binary_sha256=digest, bounds=bounds, positive_probability_log_bound=1e-3)))
                (out / 'comparison.json').write_text(json.dumps(
                    dict(passed=True, failure_count=0, bounds=bounds, raw_genes=14599, kept_genes=8066,
                         native_bundle_sha256=hashlib.sha256(bundle).hexdigest())))
                (out / 'failures.json').write_text('[]')
                (out / 'commands.json').write_text(json.dumps([dict(exit=0)] * 8))
                for name in ('forward.tsv', 'reverse.tsv', 'mixed.tsv', 'logCPM.tsv',
                             'weights.tsv', 'fitted.tsv', 'residuals.tsv',
                             'forward.json', 'reverse.json', 'mixed.json'):
                    (out / name).write_text('stable fixture\n')
            env = dict(os.environ, EVIDENCE=str(root), TARGET='linux-x86_64')
            def execute():
                return subprocess.run(['python3', '-B', '-c', program], env=env,
                                      text=True, capture_output=True, timeout=15)
            result = execute()
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(json.loads((root / 'worker-determinism.json').read_text())['passed'])
            for filename in ('weights.tsv', 'mixed.json', 'fit.json'):
                changed = root / 'comparison/workers-4' / filename
                original = changed.read_bytes()
                changed.write_text('changed\n')
                self.assertNotEqual(execute().returncode, 0, filename)
                changed.write_bytes(original)


if __name__ == '__main__':
    unittest.main()

