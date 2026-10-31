"""Stdlib-only preparation tests; these do not qualify scientific outputs."""

import ast
import importlib
import io
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
WORKFLOW = ROOT / '.github/workflows/signal-candidate.yml'


class SignalCandidateTests(unittest.TestCase):
    def test_integrated_gate_routes_real_native_classes_and_same_file_source(self):
        self.assertTrue(WORKFLOW.is_file(), 'missing whole signal native4/source-on-native gate')
        candidate = importlib.import_module('signal_candidate')
        interop = importlib.import_module('signal_source_on_native')
        text = WORKFLOW.read_text()
        routes = json.loads(re.search(r"fromJSON\('([^']+)'\)", text)[1])
        self.assertEqual([(r['target'], r['runner'], r['system'], r['arch']) for r in routes], [
            ('linux-x86_64', 'ubuntu-24.04', 'Linux', 'x86_64'),
            ('linux-aarch64', 'ubuntu-24.04-arm', 'Linux', 'aarch64'),
            ('macos-x86_64', 'macos-15-intel', 'Darwin', 'x86_64'),
            ('macos-aarch64', 'macos-15', 'Darwin', 'arm64')])
        self.assertNotIn('continue-on-error', text)
        self.assertNotRegex(text.split('permissions:', 1)[0], r'\n  (push|pull_request|schedule):')
        routed = steps(text)
        bodies = {}
        for name, step in routed.items():
            if '        run: |\n' not in step:
                continue
            body = re.sub(r'\$\{\{.*?\}\}', 'route-only', bash_body(step))
            bodies[name] = body
            checked = subprocess.run(['bash', '-n'], input=body, text=True, capture_output=True, timeout=15)
            self.assertEqual(checked.returncode, 0, name + checked.stderr)
        for mode in ('debug', 'release'):
            self.assertIn('--mode ' + mode, bodies['Test ' + mode])
        self.assertIn("matrix.target == 'linux-x86_64'", routed['Pinned source on the same native files'])
        self.assertIn('if: always()', routed['Retain complete original evidence'])
        self.assertIn('if: always()', routed['Package originals and failures'])
        for script in ('signal_candidate.py', 'signal_source_on_native.py'):
            ast.parse((ROOT / 'scripts' / script).read_text())
        with tempfile.TemporaryDirectory(prefix='signal-candidate-route-') as directory:
            root = Path(directory)
            envfile = root / 'environment'
            env = dict(os.environ, RUNNER_TEMP=str(root), GITHUB_WORKSPACE=str(ROOT),
                       GITHUB_ENV=str(envfile), SNAPSHOT_ID='missing', TARGET='route-only',
                       CONTROLLER_COMMIT='route-only')
            result = subprocess.run(['bash', '-c', bodies['Prepare hosted storage']], env=env,
                                    cwd=ROOT, text=True, capture_output=True, timeout=30)
            self.assertEqual(result.returncode, 0, result.stderr)
            env.update(line.split('=', 1) for line in envfile.read_text().splitlines())
            storage = candidate.storage(env, require_space=0)
            self.assertTrue(all(Path(p).is_relative_to(root.resolve()) for p in storage['paths'].values()))
            outside = dict(env, TMPDIR=str(root.parent))
            with self.assertRaises(ValueError):
                candidate.storage(outside, require_space=0)
            for selector in ('missing', '../green-1', 'green-1/extra', 'green-0'):
                with self.assertRaises((ValueError, OSError)):
                    candidate.prepare(ROOT, root / selector.replace('/', '_'), selector)
            fixture = root / 'portable-witness'
            record = candidate.extract_fixtures(ROOT, fixture)
            self.assertGreater(len(record['files']), 30)
            self.assertTrue((fixture / 'generated-inputs/a.raw.bam').is_file())
            self.assertEqual(record['accepted_source']['run_id'], 41192955170)
            self.assertLess((ROOT / 'assets/signal-native-fixtures-20261031.tar.gz').stat().st_size, 5 * 2**20)
            # These are real retained source containers decoded with stdlib, not scientific imports.
            summary = candidate.npz_model(fixture / 'results/summary-bw-bins.npz')
            self.assertEqual(summary['matrix']['header']['shape'], (25, 2))
            self.assertEqual(summary['labels']['values'], ['A', 'B'])
            matrix = candidate.matrix_model(fixture / 'results/matrix-reference.gz')
            self.assertEqual(matrix['header']['sample_boundaries'], [0, 15, 30])
            self.assertEqual(len(matrix['identities']), 4)
            self.assertEqual(len(candidate.bam_model(fixture / 'results/a.excluded.bam')['records']), 4)
            candidate.compare_model(dict(values=[1.0, float('nan')], shape=[2]),
                                    dict(values=[1.0, float('nan')], shape=[2]))
            with self.assertRaises(ValueError):
                candidate.compare_model(dict(values=[1.0], shape=[1]), dict(values=[2.0], shape=[1]))
            with self.assertRaises(RuntimeError):
                candidate.capture(root / 'retained-failure', 'required',
                                  [os.sys.executable, '-B', '-c', 'import sys; print("original failure"); sys.exit(7)'])
            failed = root / 'retained-failure/required'
            self.assertEqual(json.loads((failed / 'status.json').read_text())['status'], 7)
            self.assertIn('original failure', (failed / 'stdout.txt').read_text())
            for name in ('../escape', '/absolute', 'a/../b', 'A', 'a/b'):
                rows = [('a', b'ok'), (name, b'bad')]
                with self.subTest(unsafe_archive=name), self.assertRaises((ValueError, tarfile.TarError)):
                    candidate.validate_archive(self.archive(rows), {'a': candidate.digest(b'ok')})
            archive = self.archive([('a', b'ok')], symlink=True)
            with self.assertRaises(ValueError):
                candidate.validate_archive(archive, {'a': candidate.digest(b'ok')})
            with self.assertRaises(ValueError):
                candidate.portable_names(['A/x', 'a/y'])
            self.assertEqual(candidate.numeric_equal(float('nan'), float('nan')), True)
            self.assertFalse(candidate.numeric_equal(float('nan'), 0.0))
            self.assertFalse(candidate.numeric_equal(float('inf'), -float('inf')))
            self.assertFalse(candidate.numeric_equal(1.0, 1.0000001))
            for workers in (1, 4):
                plan = candidate.worker_plan(root / 'native', fixture, root / str(workers), workers)
                self.assertEqual(plan['workers'], workers)
                self.assertIn('average.bw', ' '.join(sum((c['args'] for c in plan['commands']), [])))
                self.assertTrue(all(c['args'][0:2] == ['--threads', str(workers)] for c in plan['commands']))
            interop.assert_frozen_bands()

    @staticmethod
    def archive(rows, symlink=False):
        stream = io.BytesIO()
        with tarfile.open(fileobj=stream, mode='w:gz') as archive:
            for name, data in rows:
                info = tarfile.TarInfo(name)
                info.size = len(data)
                if symlink:
                    info.type, info.linkname, info.size = tarfile.SYMTYPE, '/escape', 0
                archive.addfile(info, io.BytesIO(data))
        stream.seek(0)
        return stream


if __name__ == '__main__':
    unittest.main()
