import importlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest

from test_infercnv_ingestion_workflow import bash_body, steps


ROOT = Path(__file__).resolve().parents[1]


class EcologyDiversityOracleTests(unittest.TestCase):
    def oracle(self):
        self.assertTrue((ROOT / 'scripts/ecology_diversity_oracle.py').is_file(),
                        'missing bounded ecology source witness')
        return importlib.import_module('ecology_diversity_oracle')

    def test_integer_fixture_routing_and_actual_unavailable_observations(self):
        oracle = self.oracle()
        for name in ('numpy', 'scipy', 'skbio', 'biom'):
            self.assertNotIn(name, sys.modules)
        with tempfile.TemporaryDirectory(prefix='ecology-oracle-contract-') as temporary:
            root = Path(temporary)
            inputs = oracle.prepare_inputs(root / 'inputs')
            self.assertEqual(inputs['primary-counts.tsv']['sha256'],
                             'b79678b55391e8c43a8e47a84de4a71e92f2d738353526cc9af6547de53559a7')
            self.assertEqual(inputs['legacy-small.tsv']['sha256'],
                             'c8f56e0111400ce52398a253b182fede1af2e1016b200526987573024f5e06b4')
            self.assertEqual(inputs['legacy-big.tsv']['sha256'],
                             '88ee972c87413a617591cc93bb679944ecb29811c19b70d72c00894e2fba46da')
            feature_ids, sample_ids, rows = oracle.read_counts(root / 'inputs/primary-counts.tsv')
            self.assertEqual((len(feature_ids), sample_ids, rows[0]),
                             (12, ['S1', 'S2', 'S3', 'S4', 'S5', 'S6'], [0, 5, 6, 6, 5, 0]))
            unavailable = oracle.observe(root / 'nan', lambda: {'value': float('nan')})
            self.assertEqual(unavailable['result'], {'value': {'nonfinite': 'nan'}})
            self.assertEqual(json.loads((root / 'nan/observation.json').read_text())['status'], 'returned')
            with self.assertRaisesRegex(RuntimeError, 'required source call failed'):
                oracle.observe(root / 'required-error', lambda: (_ for _ in ()).throw(ValueError('source error')))
            failure = json.loads((root / 'required-error/observation.json').read_text())
            self.assertEqual((failure['status'], failure['exception']['type']), ('raised', 'ValueError'))

    def test_hosted_workflow_and_cli_do_not_run_optional_packages_locally(self):
        self.oracle()
        workflow = ROOT / '.github/workflows/ecology-diversity-oracle.yml'
        self.assertTrue(workflow.is_file(), 'missing hosted ecology source workflow')
        text = workflow.read_text()
        routed = steps(text)
        self.assertIn('if: always()', routed['Retain source inputs, runtime identities and all observations'])
        self.assertNotIn('continue-on-error', text)
        for name, step in routed.items():
            if '        run: |\n' in step:
                body = re.sub(r'\$\{\{.*?\}\}', 'stub', bash_body(step))
                checked = subprocess.run(['bash', '-n'], input=body, text=True, capture_output=True, timeout=15)
                self.assertEqual(checked.returncode, 0, name + ': ' + checked.stderr)
                for program in re.findall(r"python3 -B - <<'PY'\n(.*?)\nPY", body, re.DOTALL):
                    compile(program, name, 'exec')
        help_run = subprocess.run([sys.executable, '-B', str(ROOT / 'scripts/ecology_diversity_oracle.py'), '--help'],
                                  text=True, capture_output=True, timeout=15)
        self.assertEqual(help_run.returncode, 0, help_run.stderr)
        self.assertIn('--output', help_run.stdout)


if __name__ == '__main__':
    unittest.main()
