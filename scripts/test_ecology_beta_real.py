"""One connected beta preparation contract; no scientific import or execution."""
import importlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import tempfile
import unittest

from test_infercnv_ingestion_workflow import bash_body, steps

ROOT = Path(__file__).resolve().parents[1]
METRICS = ('braycurtis', 'jaccard', 'euclidean', 'canberra', 'cityblock',
           'unweighted_unifrac', 'weighted_unifrac', 'weighted_normalized_unifrac')


class EcologyBetaPreparation(unittest.TestCase):
    def test_fixed_original_beta_authority_schedule_bounds_and_retention(self):
        gate = importlib.import_module('ecology_beta_real')
        policy = gate.read_policy(ROOT)
        self.assertEqual(tuple(policy['policy']['metrics']), METRICS)
        old = json.loads((ROOT / 'assets/ecology-whole-real-20261101.json').read_bytes())
        self.assertEqual(policy['study'], old['study'])
        self.assertEqual(policy['python'], old['python'])
        self.assertEqual(policy['source_runtime']['wheels'], old['source_runtime']['wheels'])
        self.assertEqual(policy['source_runtime']['compiled_modules']['scipy.spatial._distance_pybind']['sha256'],
                         'e11e762210262b10593a77a1b5f433b14114d12da089f5516044641aced41ada')
        self.assertEqual(policy['source_runtime']['installed_python_sources']['skbio/io/format/lsmat.py'],
                         '6c886aaf0e0ad9201f3fa9d81de2ccf402f80bf5118b92c50068c721cdeb5263')
        receipt = gate.validate_native_receipt(ROOT)
        self.assertEqual(receipt['run_id'], 41280935272)
        self.assertEqual(receipt['source_files'], 120)
        self.assertEqual(receipt['captured_fixture_files'], 86)
        self.assertEqual(receipt['tests_passed_each_mode_each_target'], 6)
        with self.assertRaises((ValueError, OSError)):
            gate.validate_native_receipt(ROOT, '.autopilot/oracles/ecology-native-2026-11-01.json')
        rows = gate.schedule()
        expected = [('qualification', 0, e, n) for e, n in [('source', 1), ('native', 1), ('native', 4)]]
        expected += [('warmup', 0, e, n) for e, n in [('source', 1), ('native', 1), ('native', 4)]]
        for pair in range(1, 6):
            expected += [('measured', pair, e, n) for e, n in (
                [('source', 1), ('native', 1), ('native', 4)] if pair % 2 else
                [('native', 4), ('native', 1), ('source', 1)])]
        self.assertEqual([(r['phase'], r['pair'], r['engine'], r['workers']) for r in rows], expected)
        self.assertEqual(len(rows), 21)
        self.assertEqual(sum(r['engine'] == 'source' for r in rows), 7)
        self.assertEqual(gate.retention_plan()['initial_free_bytes'], 22 * 2**30)
        self.assertEqual(gate.retention_plan()['retained_limit_bytes'], 6 * 2**30)
        self.assertEqual(gate.retention_plan()['tar_bound_bytes'], 7 * 2**30)
        self.assertEqual(policy['policy']['max_matrix_bytes'], 64 * 2**20)
        self.assertEqual(policy['policy']['controller_deadline_seconds'], 310 * 60)
        self.assertEqual(policy['policy']['budgets'], dict(inputs=256 * 2**20, runtime=3072 * 2**20,
                         science=1536 * 2**20, metadata=1280 * 2**20))
        gate.reserve_call(0, 0, 1800, policy)
        with self.assertRaises(ValueError):
            gate.reserve_call(0, 310 * 60 - 1799, 1800, policy)
        good = [dict(source=dict(wall=t, rss_bytes=100, cpu=9),
                     native1=dict(wall=n, rss_bytes=200, cpu=20),
                     native4=dict(wall=n / 2, rss_bytes=210, cpu=30))
                for t, n in zip([10, 11, 12, 13, 14], [4, 5, 6, 7, 7.9])]
        decision = gate.evaluate(good)
        self.assertTrue(decision['passed'])
        self.assertTrue(decision['throughput'])
        self.assertGreater(decision['cpu_ratio'], 1)
        losing = json.loads(json.dumps(good))
        for row in losing:
            row['native1']['wall'] = 20
            row['native4']['wall'] = .1
        self.assertFalse(gate.evaluate(losing)['passed'])
        memory = [dict(source=dict(wall=10, rss_bytes=100, cpu=10),
                       native1=dict(wall=10.5, rss_bytes=50, cpu=20),
                       native4=dict(wall=5, rss_bytes=60, cpu=20)) for _ in range(5)]
        self.assertTrue(gate.evaluate(memory)['memory'])
        workflow = (ROOT / '.github/workflows/ecology-beta-real.yml').read_text()
        self.assertIn('timeout-minutes: 360', workflow)
        self.assertIn('fail-fast: false', workflow)
        self.assertNotIn('continue-on-error', workflow)
        matrix = json.loads(re.search(r"fromJSON\('([^']+)'\)", workflow)[1])
        self.assertEqual(tuple(matrix), METRICS)
        self.assertNotRegex(workflow.split('permissions:', 1)[0], r'\n  (push|pull_request|schedule):')
        routed = steps(workflow)
        self.assertIn('if: always()', routed['Bind and encrypt available metric originals'])
        self.assertIn('if: always()', routed['Upload metric ciphertext and transport receipt'])
        for name, step in routed.items():
            if '        run: |\n' in step:
                body = re.sub(r'\$\{\{.*?\}\}', 'route-only', bash_body(step))
                self.assertEqual(subprocess.run(['bash', '-n'], input=body, text=True,
                                 capture_output=True).returncode, 0, name)
        with tempfile.TemporaryDirectory(prefix='ecology-beta-prep-') as temporary:
            root = Path(temporary)
            reference, actual = root / 'reference.tsv', root / 'actual.tsv'
            reference.write_text('\ta\tb\na\t0\t1\nb\t1\t0\n')
            actual.write_bytes(reference.read_bytes())
            self.assertEqual(gate.compare_matrix(reference, actual, ['a', 'b'])['cells'], 4)
            actual.write_text('\ta\tb\na\t0\t1.00000000001\nb\t1.00000000001\t0\n')
            gate.compare_matrix(reference, actual, ['a', 'b'])
            with self.assertRaises(ValueError):
                gate.compare_matrix(reference, actual, ['a', 'b'], native_reference=reference)
            for bad in ['\ta\tb\na\t0\tnan\nb\tnan\t0\n',
                        '\tb\ta\na\t0\t1\nb\t1\t0\n',
                        '\ta\tb\na\t0\t1\nb\t2\t0\n']:
                actual.write_text(bad)
                with self.assertRaises(ValueError):
                    gate.compare_matrix(reference, actual, ['a', 'b'])
            self.assertLess(gate.label_overhead(['a', 'b']), 2**20)
            with self.assertRaises(ValueError):
                gate.label_overhead(['a' * 2**20])
            with self.assertRaises(RuntimeError):
                gate.capture(root, 'required-failure', [sys.executable, '-B', '-c',
                    'import sys; print("retained-beta"); print("stderr-beta", file=sys.stderr); sys.exit(9)'],
                    workers=1, timeout=10)
            failed = root / 'required-failure'
            self.assertIn(b'retained-beta', (failed / 'stdout.txt').read_bytes())
            self.assertEqual(json.loads((failed / 'status.json').read_bytes())['returncode'], 9)
            private = root / 'private'
            private.mkdir()
            gate.bind_context(private)
            gate.save(private / 'execution-status.json', dict(status='HOLD', performance_claim=False))
            gate.seal_manifest(private, policy, 'braycurtis')
            manifest = json.loads((private / 'artifact-manifest.json').read_bytes())
            self.assertEqual(manifest['metric'], 'braycurtis')
            self.assertEqual(manifest['execution_status']['status'], 'HOLD')
            self.assertTrue(manifest['source_failures_are_not_performance_wins'])
            self.assertFalse(manifest['complete_product'])


if __name__ == '__main__':
    unittest.main()
