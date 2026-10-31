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
WORKFLOW = ROOT / '.github/workflows/structure-candidate.yml'


class StructureCandidateWorkflowTests(unittest.TestCase):
    def test_four_native_routes_extract_exact_source_and_compare_complete_worker_models(self):
        self.assertTrue(WORKFLOW.is_file(), 'missing structure candidate workflow')
        text = WORKFLOW.read_text()
        routes = json.loads(re.search(r"fromJSON\('([^']+)'\)", text)[1])
        self.assertEqual([(r['target'], r['runner'], r['system'], r['arch'], r['host']) for r in routes], [
            ('linux-x86_64', 'ubuntu-24.04', 'Linux', 'x86_64', 'x86_64-unknown-linux-gnu'),
            ('linux-aarch64', 'ubuntu-24.04-arm', 'Linux', 'aarch64', 'aarch64-unknown-linux-gnu'),
            ('macos-x86_64', 'macos-15-intel', 'Darwin', 'x86_64', 'x86_64-apple-darwin'),
            ('macos-aarch64', 'macos-15', 'Darwin', 'arm64', 'aarch64-apple-darwin')])
        self.assertNotIn('continue-on-error', text)
        self.assertNotIn('runner.temp', text.split('    steps:', 1)[0])
        self.assertNotRegex(text.split('permissions:', 1)[0], r'\n  (push|pull_request|schedule):')
        routed = steps(text)
        self.assertIn('if: always()', routed['Upload raw evidence'])
        self.assertIn("matrix.target == 'linux-x86_64'", routed['Format and strict Clippy'])
        for mode in ('debug', 'release'):
            body = bash_body(routed['Test ' + mode])
            self.assertIn('RSOMICS_STRUCTURE_EVIDENCE=', body)
            self.assertIn('--no-fail-fast -- --nocapture', body)
            self.assertNotIn('--all-targets', body)
        bodies = {}
        for name, step in routed.items():
            if '        run: |\n' in step:
                body = re.sub(r'\$\{\{.*?\}\}', 'route-only', bash_body(step))
                bodies[name] = body
                result = subprocess.run(['bash', '-n'], input=body, text=True,
                                        capture_output=True, timeout=15)
                self.assertEqual(result.returncode, 0, result.stderr)
                for program in re.findall(r"python3 -B - <<'PY'\n(.*?)\nPY", body, re.DOTALL):
                    compile(program, name, 'exec')
        with tempfile.TemporaryDirectory(prefix='structure-native-route-') as directory:
            runner = Path(directory)
            environment = runner / 'environment'
            env = dict(os.environ, RUNNER_TEMP=str(runner), GITHUB_WORKSPACE=str(ROOT),
                       GITHUB_ENV=str(environment), SNAPSHOT_ID='green-1', TARGET='route-only',
                       CONTROLLER_COMMIT='route-only')

            def execute(name, current=env):
                return subprocess.run(['bash', '-c', bodies[name]], env=current, cwd=ROOT,
                                      text=True, capture_output=True, timeout=30)

            result = execute('Prepare hosted storage')
            self.assertEqual(result.returncode, 0, result.stderr)
            env.update(line.split('=', 1) for line in environment.read_text().splitlines())
            evidence = Path(env['EVIDENCE'])
            storage = json.loads((evidence / 'storage.json').read_text())
            self.assertTrue(all(Path(value).is_relative_to(runner.resolve())
                                for value in storage['paths'].values()))
            for invalid in ('../green-1', 'green-1/other'):
                result = execute('Verify and extract frozen source', dict(env, SNAPSHOT_ID=invalid))
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse((runner / 'structure-candidate').exists())
            result = execute('Verify and extract frozen source')
            self.assertEqual(result.returncode, 0, result.stderr)
            snapshot = ROOT / '.autopilot/snapshots/structure-core-2026-10-31/green-1'
            manifest = (snapshot / 'files.sha256').read_text().splitlines()
            self.assertEqual(len(manifest), 122)
            for name, expected in (
                    ('tests/fixtures/analysis/hsexposure-4hhb.pdb',
                     'c62774ac679a220a493aab5d177ab34248ab700235dc6d941f5f3cbefde334ed'),
                    ('notices/biopython-LICENSE.rst',
                     '11d1ebcaad2eb9f0f09a3df1deb4aca99a4da366ae9ae70f828c7447a24de453')):
                self.assertEqual(hashlib.sha256((runner / 'structure-candidate' / name).read_bytes()).hexdigest(), expected)
            self.assertIn('2aee715c91bcf5f748e45409f9e689ae78f399e7', (evidence / 'README.md').read_text())
            result = execute('Verify source remains unchanged')
            self.assertEqual(result.returncode, 0, result.stderr)
            changed = runner / 'structure-candidate/tests/fixtures/models-and-blanks.pdb'
            original = changed.read_bytes()
            changed.write_bytes(original + b'REMARK route mutation\n')
            self.assertNotEqual(execute('Verify source remains unchanged').returncode, 0)
            changed.write_bytes(original)
            for operation in ('contacts', 'exposure'):
                for workers in (1, 4):
                    out = evidence / 'workers' / operation / str(workers)
                    out.mkdir(parents=True)
                    result = dict(operation=operation, workers=workers,
                                  models=[dict(model_index=0, pairs=[[1, 2]], rows=[dict(cn=3)])])
                    (out / 'stdout.json').write_text(json.dumps(dict(result=result)))
                    (out / 'command.json').write_text(json.dumps(dict(status=0)))
            result = execute('Check complete worker models')
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(json.loads((evidence / 'worker-equality.json').read_text())['compared_workers'], [1, 4])
            altered = evidence / 'workers/contacts/4/stdout.json'
            value = json.loads(altered.read_text())
            value['result']['models'][0]['pairs'][0][0] = 9
            altered.write_text(json.dumps(value))
            self.assertNotEqual(execute('Check complete worker models').returncode, 0)


if __name__ == '__main__':
    unittest.main()
