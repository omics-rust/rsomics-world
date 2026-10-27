import hashlib
import io
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest
import zipfile

from test_infercnv_ingestion_workflow import bash_body, steps


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / '.github/workflows/rnaseq-qc-candidate.yml'


class RnaSeqCandidateWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.text = WORKFLOW.read_text() if WORKFLOW.exists() else ''
        cls.steps = steps(cls.text)

    def step(self, name):
        self.assertTrue(name in self.steps, f'missing whole-feature workflow step: {name}')
        return self.steps[name]

    def program(self, name):
        return re.findall(r"python3 -B - <<'PY'\n(.*?)\nPY", bash_body(self.step(name)), re.DOTALL)[-1]

    def run_program(self, program, directory, **environment):
        return subprocess.run(['python3', '-B', '-c', program], cwd=directory,
                              env=dict(os.environ, GITHUB_WORKSPACE=str(ROOT), **environment),
                              capture_output=True, text=True, timeout=15)

    def test_bootstrap_is_not_the_four_native_gate_and_all_shells_parse(self):
        self.assertTrue(WORKFLOW.exists(), 'the hosted whole-report gate is missing')
        matrix = re.search(r"fromJSON\(inputs.bootstrap_dependencies && '(.*?)' \|\| '(.*?)'\)", self.text)
        self.assertIsNotNone(matrix)
        self.assertEqual([item['target'] for item in json.loads(matrix[1])], ['linux-x86_64'])
        self.assertEqual([item['target'] for item in json.loads(matrix[2])],
                         ['linux-x86_64', 'linux-aarch64', 'macos-x86_64', 'macos-aarch64'])
        self.assertRegex(self.text, r'bootstrap_dependencies:\n(?:        .*\n)*?        default: false\n')
        names = list(self.steps)
        compare = names.index('Compare complete native reports')
        for prerequisite in ['Verify frozen dependencies', 'Format and strict Clippy', 'Test debug', 'Test release']:
            self.assertLess(names.index(prerequisite), compare)
        self.assertIn('if: always()', self.step('Upload raw evidence'))
        self.assertIn('rnaseq-qc-${{ github.run_id }}-${{ github.run_attempt }}-${{ matrix.target }}',
                      self.step('Upload raw evidence'))
        for name, step in self.steps.items():
            if '        run: |\n' in step:
                with self.subTest(step=name):
                    body = re.sub(r'\$\{\{.*?\}\}', 'stub', bash_body(step))
                    result = subprocess.run(['bash', '-n'], input=body, capture_output=True, text=True, timeout=15)
                    self.assertEqual(result.returncode, 0, result.stderr)

    def test_postcheck_accepts_only_the_bootstrap_lock_change(self):
        program = self.program('Verify source remains unchanged')
        for mutation, bootstrap, accepted in [('none', False, True), ('lock', False, False),
                                               ('lock', True, True), ('source', True, False),
                                               ('added', True, False)]:
            with self.subTest(mutation=mutation, bootstrap=bootstrap), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                source, evidence = root / 'source', root / 'evidence'
                source.mkdir()
                evidence.mkdir()
                files = {'Cargo.lock': b'version = 4\n', 'src/lib.rs': b'pub fn count() -> usize { 5 }\n'}
                for name, data in files.items():
                    path = source / name
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_bytes(data)
                (evidence / 'files.sha256').write_text(''.join(
                    f'{hashlib.sha256(data).hexdigest()}  {name}\n' for name, data in sorted(files.items())))
                if mutation == 'lock':
                    (source / 'Cargo.lock').write_text('version = 4\n# resolved\n')
                elif mutation == 'source':
                    (source / 'src/lib.rs').write_text('pub fn count() -> usize { 0 }\n')
                elif mutation == 'added':
                    (source / 'unexpected.rs').touch()
                result = self.run_program(program, source, EVIDENCE=str(evidence),
                                          BOOTSTRAP_DEPENDENCIES=str(bootstrap).lower())
                self.assertEqual(result.returncode == 0, accepted, result.stderr)

    def test_actual_verifier_binds_api_zip_and_manifest_before_export(self):
        program = self.program('Acquire and verify actual RSeQC evidence')
        template = json.loads((ROOT / '.autopilot/oracles/rnaseq-qc-first-workflow-2026-10-27.json').read_text())
        for mutation, accepted in [('none', True), ('head', False), ('traversal', False), ('file', False)]:
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                acquisition = root / 'oracle-acquisition'
                acquisition.mkdir()
                files = {'cases/single/mapping/process.stdout': b'Total records: 88\n',
                         'cases/paired/mapping/process.stdout': b'Total records: 175\n'}
                witness = dict(schema_version=1, status='complete', failed_cases=[],
                               source_commit='0d2e017723c5c4b41eb3b938e1631692d187de1f',
                               files={name: hashlib.sha256(data).hexdigest() for name, data in files.items()})
                files['witness.json'] = (json.dumps(witness) + '\n').encode()
                manifest = ('sha256\tpath\n' + ''.join(
                    f'{hashlib.sha256(data).hexdigest()}\t{name}\n' for name, data in sorted(files.items()))).encode()
                buffer = io.BytesIO()
                with zipfile.ZipFile(buffer, 'w') as archive:
                    for name, data in files.items():
                        if mutation == 'file' and name.endswith('single/mapping/process.stdout'):
                            data = b'changed source output\n'
                        archive.writestr('workflows/' + name, data)
                    archive.writestr('workflows/sha256-manifest.tsv', manifest)
                    if mutation == 'traversal':
                        archive.writestr('../escape', 'unsafe')
                payload = buffer.getvalue()
                receipt = dict(template, artifact_sha256=hashlib.sha256(payload).hexdigest(),
                               size_in_bytes=len(payload), manifest_sha256=hashlib.sha256(manifest).hexdigest(),
                               witness_sha256=hashlib.sha256(files['witness.json']).hexdigest())
                run = dict(id=receipt['run_id'], run_attempt=receipt['run_attempt'],
                           head_sha='0' * 40 if mutation == 'head' else receipt['head_sha'],
                           path=receipt['workflow_path'], status='completed', conclusion='success',
                           event='workflow_dispatch', repository=dict(full_name=receipt['repository']),
                           head_repository=dict(full_name=receipt['repository']))
                artifact = dict(id=receipt['artifact_id'], name=receipt['artifact_name'], expired=False,
                                digest='sha256:' + receipt['artifact_sha256'], size_in_bytes=len(payload),
                                workflow_run=dict(id=receipt['run_id'], head_sha=receipt['head_sha']))
                for name, value in [('receipt', receipt), ('run', run), ('artifact', artifact)]:
                    (acquisition / (name + '.json')).write_text(json.dumps(value))
                (acquisition / 'artifact.zip').write_bytes(payload)
                result = self.run_program(program, ROOT, EVIDENCE=str(root))
                self.assertEqual(result.returncode == 0, accepted, result.stderr)
                self.assertFalse((root / 'escape').exists())
                if accepted:
                    self.assertEqual((acquisition / 'source/workflows/cases/single/mapping/process.stdout').read_bytes(),
                                     b'Total records: 88\n')

    def test_comparison_keeps_the_same_executable_and_propagates_failure(self):
        body = re.sub(r'\$\{\{ matrix.target \}\}', 'linux-x86_64',
                      bash_body(self.step('Compare complete native reports')))
        for failed in [False, True]:
            with self.subTest(failed=failed), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                evidence, target, workspace = root / 'evidence', root / 'target', root / 'workspace'
                evidence.mkdir()
                (target / 'release').mkdir(parents=True)
                (workspace / 'scripts').mkdir(parents=True)
                original = target / 'release/rsomics-rnaseq-qc'
                original.write_bytes(b'retained build fixture; not a native acceptance run\n')
                comparator = workspace / 'scripts/compare_rnaseq_qc_workflow.py'
                comparator.write_text('import argparse,json,os,sys\nfrom pathlib import Path\n'
                                      'p=argparse.ArgumentParser()\n'
                                      'for key in ("oracle","binary","output"): p.add_argument("--"+key,required=True)\n'
                                      'a=p.parse_args(); out=Path(a.output); out.mkdir()\n'
                                      '(out/"selection.json").write_text(json.dumps(vars(a)))\n'
                                      'sys.exit(17 if os.environ["FAIL"]=="1" else 0)\n')
                for name in ['test_compare_rnaseq_qc_workflow.py', 'rnaseq_qc_oracle.py',
                             'infercnv_trial_process.py', 'validate_infercnv_shipped_oracle.py']:
                    (workspace / 'scripts' / name).write_bytes((ROOT / 'scripts' / name).read_bytes())
                command = 'cargo() { test "$*" = "build --locked --release --bin rsomics-rnaseq-qc"; };\n' + body
                result = subprocess.run(['bash', '-c', command], cwd=root,
                                        env=dict(os.environ, EVIDENCE=str(evidence), CARGO_TARGET_DIR=str(target),
                                                 GITHUB_WORKSPACE=str(workspace), ORACLE_ROOT=str(root / 'oracle'),
                                                 FAIL=str(int(failed))), capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode, 17 if failed else 0, result.stderr)
                retained = evidence / 'native-binary/rsomics-rnaseq-qc-linux-x86_64'
                self.assertEqual(retained.read_bytes(), original.read_bytes())
                selection = json.loads((evidence / 'comparison/selection.json').read_text())
                self.assertEqual(selection['binary'], str(retained))
                self.assertEqual(selection['oracle'], str(root / 'oracle'))
                self.assertTrue((evidence / 'native-binary/checksums.sha256').is_file())


if __name__ == '__main__':
    unittest.main()
