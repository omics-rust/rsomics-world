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
SEED = ROOT / '.autopilot/snapshots/structure-first-workflow-inputs-2026-10-28'


class StructureFirstWorkflowOracleTests(unittest.TestCase):
    def oracle(self):
        self.assertTrue((ROOT / 'scripts/structure_first_workflow_oracle.py').is_file(),
                        'missing hosted structure source witness')
        return importlib.import_module('structure_first_workflow_oracle')

    def test_verified_fixture_unpack_and_required_source_failure_are_not_silent(self):
        oracle = self.oracle()
        for name in ('numpy', 'Bio', 'gemmi'):
            self.assertNotIn(name, sys.modules)
        with tempfile.TemporaryDirectory(prefix='structure-source-contract-') as temporary:
            output = Path(temporary)
            provenance = oracle.prepare_inputs(SEED, output / 'inputs')
            self.assertEqual(provenance['assets']['rsomics-pdb-core']['commit'],
                             '7452a78326384fc9307e5234f40cc2362d9fb2be')
            self.assertEqual(provenance['assets']['rsomics-pdb-chain']['commit'],
                             '91ee3115f9467ef215962e44d4d232e329d2cb3d')
            retained = output / 'inputs/historical/rsomics-pdb-chain/tests/golden/small.pdb'
            self.assertEqual(len([line for line in retained.read_text().splitlines()
                                  if line.startswith('ATOM')]), 3)
            boundary = oracle.observe(output / 'nonfinite', {'operation': 'explicit-boundary'},
                                      lambda: {'value': float('nan')})
            self.assertEqual(boundary['result']['value'], {'nonfinite': 'nan'})
            with self.assertRaisesRegex(RuntimeError, 'required source call failed'):
                oracle.observe(output / 'failure', {'operation': 'required'},
                               lambda: (_ for _ in ()).throw(ValueError('source error')))
            failure = json.loads((output / 'failure/observation.json').read_text())
            self.assertEqual((failure['status'], failure['exception']['type']), ('raised', 'ValueError'))
            expected = oracle.observe(output / 'expected', {'operation': 'explicit-boundary',
                'allowed_exceptions': ['ValueError']},
                lambda: (_ for _ in ()).throw(ValueError('unrepresentable input')))
            self.assertEqual(expected['status'], 'raised')

    def test_hosted_routing_and_help_do_not_import_scientific_packages(self):
        self.oracle()
        workflow = ROOT / '.github/workflows/structure-first-workflow-oracle.yml'
        self.assertTrue(workflow.is_file(), 'missing hosted structure source workflow')
        text = workflow.read_text()
        routed = steps(text)
        self.assertIn('if: always()', routed['Retain complete source witness including failures'])
        self.assertNotIn('continue-on-error', text)
        for name, step in routed.items():
            if '        run: |\n' in step:
                body = re.sub(r'\$\{\{.*?\}\}', 'stub', bash_body(step))
                result = subprocess.run(['bash', '-n'], input=body, text=True,
                                        capture_output=True, timeout=15)
                self.assertEqual(result.returncode, 0, name + ': ' + result.stderr)
                for program in re.findall(r"python3 -B - <<'PY'\n(.*?)\nPY", body, re.DOTALL):
                    compile(program, name, 'exec')
        result = subprocess.run([sys.executable, '-B', str(ROOT / 'scripts/structure_first_workflow_oracle.py'),
                                 '--help'], text=True, capture_output=True, timeout=15)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('--output', result.stdout)
        for name in ('numpy', 'Bio', 'gemmi'):
            self.assertNotIn(name, sys.modules)


if __name__ == '__main__':
    unittest.main()
