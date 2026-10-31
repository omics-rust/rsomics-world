import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / 'scripts/peak_first_workflow_oracle.py'


class PeakSourcePreparationTest(unittest.TestCase):
    def test_full_four_route_preserves_originals_identity_failures_and_runtime_gates(self):
        self.assertTrue(SCRIPT.is_file(), 'peak full-four source producer is absent')
        spec = importlib.util.spec_from_file_location('peak_oracle', SCRIPT)
        oracle = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(oracle)
        self.assertFalse({'numpy', 'pandas', 'pysam', 'scipy'}.intersection(sys.modules))
        lock = json.loads((ROOT / 'assets/peak-first-runtime-20261031.json').read_text())
        with tempfile.TemporaryDirectory(prefix='peak-producer-check-', dir=os.environ['TMPDIR']) as directory:
            root = Path(directory)
            plan = oracle.case_plan(root / 'inputs', root / 'runtime', root / 'witness')
            self.assertEqual(len({job['name'] for job in plan}), len(plan))
            self.assertEqual({family: sum(job['family'] == family for job in plan)
                              for family in ['call', 'call-sparse', 'annotate', 'count']},
                             {'call': 5, 'call-sparse': 6, 'annotate': 3, 'count': 4})
            jobs = {job['name']: job for job in plan}
            self.assertNotIn('--nomodel', jobs['M1']['argv'])
            self.assertNotIn('--nomodel', jobs['M2']['argv'])
            self.assertIn('T2.se.bam', ' '.join(jobs['M2']['argv']))
            self.assertIn('C2.se.bam', ' '.join(jobs['M2']['argv']))
            self.assertIn('BAMPE', jobs['M4']['argv'])
            self.assertIn('BAMPE', jobs['M5']['argv'])
            for job in plan:
                if job['family'] == 'call':
                    self.assertEqual(job['argv'][job['argv'].index('--threads') + 1], '1')
                    self.assertEqual(job['argv'][job['argv'].index('--process-backend') + 1], 'serial')
            self.assertEqual(set(jobs['row-ledger']['deps']),
                             {'M2', 'S-control-norm-stringent', 'S-control-norm-relaxed',
                              'S-control-non-stringent', 'S-control-non-relaxed',
                              'S-numeric-stringent', 'S-numeric-relaxed'})
            self.assertIn('-p', jobs['C-proper']['argv'])
            self.assertNotIn('fragment', ' '.join(jobs['C-proper']['argv']))
            self.assertEqual(jobs['A-gtf-default']['deps'], ['row-ledger'])
            self.assertEqual(jobs['A-gtf-300']['deps'], ['row-ledger'])
            self.assertEqual(jobs['A-gtf-300']['argv'][-1], '300')
            self.assertEqual(jobs['A-gff3-default']['argv'][-1], 'unset')
            self.assertEqual(oracle.r_admission_mode({'Package': 'codetools', 'Priority': 'recommended'}),
                             'immutable-image')
            self.assertEqual(oracle.r_admission_mode({'Package': 'codetools', 'Priority': ''}), 'source')
            with self.assertRaisesRegex(ValueError, 'dependency'):
                oracle.check_constraints({'R': '4.6.1', 'codetools': '0.2.99'},
                                         {'codetools': {'Depends': 'R (>= 4.7)'}})
            oracle.check_constraints({'R': '4.6.1', 'codetools': '0.2.20'},
                                     {'codetools': {'Depends': 'R (>= 3.5)'}})
            self.assertTrue(callable(getattr(oracle, 'macs_build_environment', None)),
                            'MACS build must bind existing image compilers without changing other stages')
            original_environment = {'PATH': '/image/bin', 'CC': 'clang -pthread',
                                    'CXX': 'clang++ -pthread', 'LDSHARED': 'existing-linker-override'}
            build_environment = oracle.macs_build_environment(
                original_environment, '/usr/bin/gcc', '/usr/bin/g++')
            self.assertEqual(build_environment['CC'], '/usr/bin/gcc')
            self.assertEqual(build_environment['CXX'], '/usr/bin/g++')
            self.assertEqual(build_environment['LDSHARED'], 'existing-linker-override')
            self.assertEqual(original_environment['CC'], 'clang -pthread')
            self.assertEqual(set(build_environment), set(original_environment))
            with self.assertRaisesRegex(ValueError, 'bare absolute'):
                oracle.macs_build_environment(original_environment, '/usr/bin/gcc -pthread', '/usr/bin/g++')
            rows = [['chrA', '10', '20', 'duplicate', '0', '+']] * 2
            oracle.write_rows(root / 'rows', [('diagnostic', rows)])
            ledger = (root / 'rows/workflow-rows.tsv').read_text().splitlines()
            self.assertEqual(len(ledger), 3)
            self.assertNotEqual(ledger[1].split('\t')[0], ledger[2].split('\t')[0])
            self.assertEqual((root / 'rows/workflow.bed').read_text().splitlines(), ['\t'.join(rows[0])] * 2)
            execution = [
                {'name': 'fails', 'family': 'retention-check', 'deps': [],
                 'argv': [sys.executable, '-B', '-c', 'print("original failure"); raise SystemExit(7)']},
                {'name': 'dependent', 'family': 'retention-check', 'deps': ['fails'],
                 'argv': [sys.executable, '-B', '-c', 'raise SystemExit(99)']},
                {'name': 'independent', 'family': 'retention-check', 'deps': [],
                 'argv': [sys.executable, '-B', '-c', 'print("independent process retained")']}]
            statuses = oracle.execute_plan(execution, root / 'execution')
            self.assertEqual([statuses[n]['status'] for n in ['fails', 'dependent', 'independent']],
                             ['failed', 'blocked', 'returned'])
            self.assertEqual(statuses['fails']['returncode'], 7)
            captured_environment = json.loads((root / 'execution/calls/fails/request.json').read_text())['env']
            self.assertTrue({'CC', 'CXX', 'LDSHARED', 'LDCXXSHARED', 'CFLAGS', 'CXXFLAGS', 'LDFLAGS'}
                            <= set(captured_environment))
            self.assertEqual((root / 'execution/calls/fails/stdout.txt').read_text(), 'original failure\n')
            self.assertFalse((root / 'execution/calls/dependent/stdout.txt').exists())
            oracle.bind_evidence(root / 'execution')
            oracle.package_evidence(root / 'execution', root / 'original-evidence.tar')
            with tarfile.open(root / 'original-evidence.tar') as archive:
                self.assertIn('execution/calls/fails/stdout.txt', archive.getnames())
            self.assertEqual(lock['r_base_container']['digest'],
                             'sha256:81bf4c3e9f1b08b0ada0a45ef16ef51d03dc91b0b06cda4b408d667cb8c2c645')
            self.assertEqual(len(lock['python_packages']), 22)
            workflow = (ROOT / '.github/workflows/peak-first-workflow-oracle.yml').read_text()
            self.assertIn(lock['r_base_container']['digest'], workflow)
            self.assertNotIn('continue-on-error', workflow)
            self.assertIn('if: always()', workflow)
            self.assertIn('peak-first-workflow-oracle.tar', workflow)
            self.assertTrue((ROOT / 'scripts/peak_first_workflow_oracle.R').is_file())
            lines = workflow.splitlines()
            for index, line in enumerate(lines):
                if line != '        run: |':
                    continue
                body = []
                for current in lines[index + 1:]:
                    if current and not current.startswith('          '):
                        break
                    body.append(current[10:] if current else '')
                program = '\n'.join(body) + '\n'
                parsed = subprocess.run(['bash', '-n'], input=program, text=True, capture_output=True)
                self.assertEqual(parsed.returncode, 0, parsed.stderr)
                for block in re.findall(r"python3 -B - <<'PY'\n(.*?)\nPY", program, re.DOTALL):
                    compile(block, 'hosted storage preparation', 'exec')


if __name__ == '__main__':
    unittest.main()
