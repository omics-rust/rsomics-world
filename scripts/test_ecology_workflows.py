"""One connected preparation contract; native/scientific execution stays hosted."""

import ast
import importlib
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile
import tempfile
import unittest

from test_infercnv_ingestion_workflow import bash_body, steps


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / '.github/workflows/ecology-candidate.yml'


class EcologyWorkflowTests(unittest.TestCase):
    def test_existing_ecology_source_native_four_and_original_retention(self):
        self.assertTrue(WORKFLOW.is_file(), 'missing unchanged Ecology native-four workflow')
        candidate = importlib.import_module('ecology_candidate')
        text = WORKFLOW.read_text()
        routes = json.loads(re.search(r"fromJSON\('([^']+)'\)", text)[1])
        self.assertEqual([(r['target'], r['runner'], r['host']) for r in routes], [
            ('linux-x86_64', 'ubuntu-24.04', 'x86_64-unknown-linux-gnu'),
            ('linux-aarch64', 'ubuntu-24.04-arm', 'aarch64-unknown-linux-gnu'),
            ('macos-x86_64', 'macos-15-intel', 'x86_64-apple-darwin'),
            ('macos-aarch64', 'macos-15', 'aarch64-apple-darwin')])
        self.assertNotRegex(text.split('permissions:', 1)[0], r'\n  (push|pull_request|schedule):')
        self.assertNotIn('continue-on-error', text)
        routed = steps(text)
        expected = ['Prepare native storage', 'Verify frozen source', 'Install native Rust 1.91',
                    'Admit compiler and locked dependencies', 'Formatting and strict Clippy',
                    'Full debug suite', 'Full release suite', 'Retain actual release executable',
                    'Verify unchanged source', 'Package complete originals', 'Upload complete originals']
        self.assertEqual(list(routed), expected)
        for name, step in routed.items():
            if '        run: |\n' in step:
                body = re.sub(r'\$\{\{.*?\}\}', 'route-only', bash_body(step))
                checked = subprocess.run(['bash', '-n'], input=body, text=True,
                                         capture_output=True, timeout=15)
                self.assertEqual(checked.returncode, 0, name + checked.stderr)
        for name in expected[-3:]:
            self.assertIn('if: always()', routed[name])
        for mode in ('debug', 'release'):
            command = candidate.test_command(mode)
            self.assertTrue({'--locked', '--offline', '--all-targets', '--all-features',
                             '--no-fail-fast'} <= set(command))
            self.assertEqual('--release' in command, mode == 'release')
            self.assertNotIn('--ignored', command)
        ast.parse((ROOT / 'scripts/ecology_candidate.py').read_text())
        with tempfile.TemporaryDirectory(prefix='ecology-gate-') as directory:
            root = Path(directory)
            for selector in ('missing', '../green-1', 'green-1/x'):
                with self.assertRaises((ValueError, OSError)):
                    candidate.prepare_source(ROOT, root / 'absent', selector)
            output = root / 'candidate'
            authority = candidate.prepare_source(ROOT, output, 'green-1')
            self.assertEqual(authority['owner_commit'], 'e1b03cb68876be2dd8bb9fbc520b5194be055d9e')
            self.assertEqual(authority['owner_tree'], 'b5996ddd92f292714c82c0c53a50c041f2b90249')
            self.assertEqual(len(authority['files']), 35)
            self.assertEqual(len(authority['fixtures']), 8)
            self.assertEqual(authority['source_run']['run_id'], 40634594900)
            manifest = {name: row['sha256'] for name, row in authority['files'].items()}
            candidate.check_tree(output / 'source', manifest)
            for name, row in authority['fixtures'].items():
                self.assertEqual(candidate._hash_file(output / 'source' / name), row['sha256'])
            changed = output / 'source/Cargo.lock'
            original = changed.read_bytes()
            changed.write_bytes(original + b'\nchanged\n')
            controller = root / 'controller'
            controller.mkdir()
            (controller / 'original.py').write_bytes(b'unchanged-controller\n')
            candidate.save(controller / 'checksums.json', candidate.tree_manifest(controller))
            previous = os.environ.get('EVIDENCE')
            os.environ['EVIDENCE'] = str(root)
            try:
                with self.assertRaises(ValueError):
                    candidate.finalize(output)
            finally:
                if previous is None:
                    del os.environ['EVIDENCE']
                else:
                    os.environ['EVIDENCE'] = previous
            self.assertTrue(json.loads((output / 'integrity-after.json').read_bytes())['failures'])
            changed.write_bytes(original)
            candidate.check_tree(output / 'source', manifest)
            with self.assertRaises(RuntimeError):
                candidate.capture(output / 'commands', 'failure', [os.sys.executable, '-B', '-c',
                    'import sys; print("original-nine"); print("stderr-nine", file=sys.stderr); sys.exit(9)'])
            failed = output / 'commands/failure'
            self.assertEqual(json.loads((failed / 'status.json').read_bytes())['status'], 9)
            self.assertIn(b'original-nine', (failed / 'stdout.txt').read_bytes())
            self.assertIn(b'stderr-nine', (failed / 'stderr.txt').read_bytes())
            archive_path = root / 'originals.tar'
            candidate.package(output, archive_path)
            with tarfile.open(archive_path) as archive:
                self.assertIn('candidate/commands/failure/status.json', archive.getnames())
                self.assertTrue(all(member.isfile() for member in archive.getmembers()))
                record = json.load(archive.extractfile('candidate/artifact-manifest.json'))
                self.assertEqual(record['schema'], 'rsomics.ecology.native-four.v1')
                self.assertEqual(record['files']['commands/failure/stdout.txt']['sha256'],
                                 candidate._hash_file(failed / 'stdout.txt'))
                self.assertFalse(record['performance_claim'])
                self.assertFalse(record['source_on_native_qualified'])

        whole_workflow = ROOT / '.github/workflows/ecology-whole-real.yml'
        self.assertTrue(whole_workflow.is_file(), 'missing full-study Ecology workflow')
        whole = importlib.import_module('ecology_whole_real')
        policy = whole.read_policy(ROOT)
        self.assertEqual(policy['study']['features'], 41328)
        self.assertEqual(policy['study']['samples'], 1105)
        self.assertEqual(policy['policy']['rarefaction'], {'depth': 3, 'seed': 42})
        accepted = whole.validate_native_receipt(ROOT, policy['native_receipt']['path'],
                                                policy['native_receipt']['sha256'])
        self.assertTrue(accepted['native_four_qualified'])
        self.assertEqual(set(accepted['targets']),
                         {'linux-x86_64', 'linux-aarch64', 'macos-x86_64', 'macos-aarch64'})
        schedule = whole.schedule()
        self.assertEqual([(r['phase'], r['workers'], r['pair'], r['engine']) for r in schedule], [
            ('qualification', 1, 0, 'source'), ('qualification', 4, 0, 'source'),
            ('qualification', 1, 0, 'native'), ('qualification', 4, 0, 'native'),
            ('warmup', 1, 0, 'source'), ('warmup', 1, 0, 'native'),
            ('measured', 1, 1, 'source'), ('measured', 1, 1, 'native'),
            ('measured', 1, 2, 'native'), ('measured', 1, 2, 'source'),
            ('measured', 1, 3, 'source'), ('measured', 1, 3, 'native'),
            ('measured', 1, 4, 'native'), ('measured', 1, 4, 'source'),
            ('measured', 1, 5, 'source'), ('measured', 1, 5, 'native'),
            ('warmup', 4, 0, 'source'), ('warmup', 4, 0, 'native'),
            ('measured', 4, 1, 'source'), ('measured', 4, 1, 'native'),
            ('measured', 4, 2, 'native'), ('measured', 4, 2, 'source'),
            ('measured', 4, 3, 'source'), ('measured', 4, 3, 'native'),
            ('measured', 4, 4, 'native'), ('measured', 4, 4, 'source'),
            ('measured', 4, 5, 'source'), ('measured', 4, 5, 'native')])
        reservation = whole.retention_plan()
        self.assertEqual(reservation['retained_limit_bytes'], 6 * 2**30)
        self.assertEqual(reservation['tar_bound_bytes'], 7 * 2**30)
        self.assertEqual(reservation['headroom_bytes'], 8 * 2**30)
        with tempfile.TemporaryDirectory(prefix='ecology-whole-prep-') as directory:
            root = Path(directory)
            source_alpha, native_alpha = root / 'source.tsv', root / 'native.tsv'
            source_alpha.write_text('sample\tobserved_features\tshannon\np\t2\t0.5\nz\t0\tnan\n')
            native_alpha.write_text('sample\tobserved_features\tshannon\np\t2\t0.50000000001\nz\t0\tnan\n')
            self.assertEqual(whole.compare_alpha(source_alpha, native_alpha,
                             ['p', 'z'], ['observed_features', 'shannon'])['cells'], 4)
            native_alpha.write_text('sample\tobserved_features\tshannon\np\t2\t0.5\nz\t0\t0\n')
            with self.assertRaises(ValueError):
                whole.compare_alpha(source_alpha, native_alpha,
                                    ['p', 'z'], ['observed_features', 'shannon'])
            original, expected, actual = root / 'original.tsv', root / 'expected.tsv', root / 'actual.tsv'
            original.write_text('feature\tp\tq\na\t4\t1\nb\t1\t4\n')
            expected.write_text('feature\tp\tq\na\t2\t1\nb\t1\t2\n')
            actual.write_bytes(expected.read_bytes())
            self.assertEqual(whole.compare_counts(original, expected, actual,
                             ['a', 'b'], ['p', 'q'], 3)['cells'], 4)
            actual.write_text('feature\tp\tq\na\t1\t2\nb\t2\t1\n')
            with self.assertRaises(ValueError):
                whole.compare_counts(original, expected, actual, ['a', 'b'], ['p', 'q'], 3)
            with self.assertRaises((ValueError, OSError)):
                whole.validate_native_receipt(root, '.autopilot/oracles/ecology-native-missing.json')
            good_pairs = [dict(source=dict(a=dict(wall=10, cpu=10, rss=100),
                                          w=dict(wall=20, cpu=20, rss=100),
                                          stages={'rarefy': 4, 'rarefied-alpha': 6}),
                               native=dict(a=dict(wall=5, cpu=5, rss=50),
                                           w=dict(wall=10, cpu=10, rss=50),
                                           stages={'rarefy': 2, 'rarefied-alpha': 3})) for _ in range(5)]
            self.assertTrue(whole.evaluate(good_pairs)['passed'])
            broken = json.loads(json.dumps(good_pairs))
            for pair in broken:
                pair['native']['stages']['rarefy'] = 5
            self.assertFalse(whole.evaluate(broken)['passed'])
            with self.assertRaises(RuntimeError):
                whole.capture(root, 'nonzero', [os.sys.executable, '-B', '-c',
                    'import sys; print("retained-five"); sys.exit(5)'], workers=1, timeout=10)
            self.assertIn(b'retained-five', (root / 'nonzero/stdout.txt').read_bytes())
            self.assertEqual(json.loads((root / 'nonzero/status.json').read_bytes())['returncode'], 5)
        for path in ('scripts/ecology_whole_real.py', 'scripts/ecology_source_workflow.py'):
            ast.parse((ROOT / path).read_text())
        for name, step in steps(whole_workflow.read_text()).items():
            if '        run: |\n' in step:
                body = re.sub(r'\$\{\{.*?\}\}', 'route-only', bash_body(step))
                checked = subprocess.run(['bash', '-n'], input=body, text=True,
                                         capture_output=True, timeout=15)
                self.assertEqual(checked.returncode, 0, name + checked.stderr)


if __name__ == '__main__':
    unittest.main()
