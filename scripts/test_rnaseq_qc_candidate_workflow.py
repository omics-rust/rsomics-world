import hashlib
import importlib
import copy
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

    def test_explicit_junction_candidate_preserves_exact_source_and_failure_evidence(self):
        controller = ROOT / 'scripts/rnaseq_qc_junction_candidate.py'
        available = subprocess.run(['python3', '-B', str(controller), '--help'],
                                   capture_output=True, text=True, timeout=15)
        self.assertEqual(available.returncode, 0,
                         'the additive junction authority controller is not available: ' + available.stderr)
        candidate = importlib.import_module('rnaseq_qc_junction_candidate')
        selector = bash_body(self.step('Verify and extract frozen source')).split(
            'python3 -B scripts/infercnv_source_snapshot.py', 1)[0]
        selector = selector[selector.index('case "${CANDIDATE_AUTHORITY') :]
        for authority, snapshot, bootstrap, accepted in [
            ('five-metric', 'green-6', 'false', True),
            ('junctions', 'green-1', 'false', True),
            ('junctions', 'green-6', 'false', False),
            ('junctions', 'green-1', 'true', False),
            ('unknown', 'green-1', 'false', False),
        ]:
            with self.subTest(authority=authority, snapshot=snapshot, bootstrap=bootstrap):
                selected = subprocess.run(['bash', '-c', 'set -euo pipefail\n' + selector],
                    env=dict(os.environ, CANDIDATE_AUTHORITY=authority, SNAPSHOT_ID=snapshot,
                             BOOTSTRAP_DEPENDENCIES=bootstrap, GITHUB_WORKSPACE=str(ROOT)),
                    capture_output=True, text=True, timeout=15)
                self.assertEqual(selected.returncode == 0, accepted, selected.stderr)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, evidence = root / 'source', root / 'evidence'
            from infercnv_source_snapshot import verify_and_extract
            snapshot = ROOT / '.autopilot/snapshots/rnaseq-qc-junctions-2026-11-01/green-1'
            verify_and_extract(snapshot, source)
            evidence.mkdir()
            prepared = candidate.prepare(ROOT, source, evidence)
            self.assertEqual(prepared['source_files'], 66)
            self.assertEqual(prepared['captured_files'], 45)
            self.assertEqual(prepared['owner_commit'], 'a618e6ab293e2fb7a113d949197e1bb43ecc7733')
            self.assertTrue((evidence / 'junction-authority/receipt.json').is_file())
            self.assertEqual(len(list((evidence / 'junction-authority/fixtures').rglob('input.bam'))), 5)
            authority = candidate.authority(ROOT)
            candidate.verify_source(source, authority)
            production = source / 'src/metrics/junctions.rs'
            body = production.read_bytes()
            production.write_bytes(body + b'\n')
            with self.assertRaises(ValueError):
                candidate.verify_source(source, authority)
            production.write_bytes(body)
            production.chmod(0o600)
            with self.assertRaises(ValueError):
                candidate.verify_source(source, authority)
            production.chmod(0o644)
            fixture = source / 'tests/fixtures/junctions/informative-single'
            expected = candidate.source_expected(fixture, 'informative-single')
            self.assertEqual(expected['counts'], dict(total_events=11, known_events=6,
                partial_novel_events=1, novel_events=3, filtered_events=1, total_junctions=6,
                known_junctions=2, partial_novel_junctions=1, novel_junctions=3))
            self.assertEqual([(row['start'], row['end'], row['read_count']) for row in expected['rows']],
                [(120, 170, 5), (120, 270, 1), (120, 171, 1), (190, 240, 1), (320, 370, 1), (522, 572, 1)])
            native = root / 'native-fixture'
            native.mkdir()
            report = dict(schema_version='1.0', sample='input', profile='rseqc-5.0.5', metrics=['junctions'],
                          junctions={key: copy.deepcopy(expected[key]) for key in ('rows', 'counts')})
            def native_output(value, mutation=None):
                (native / 'report.json').write_text(json.dumps(value))
                detail = 'chrom\tintron_start\tintron_end\tread_count\tannotation\n'
                detail += ''.join('\t'.join(str(row[key]) for key in ('chrom','start','end','read_count','annotation')) + '\n'
                                  for row in value['junctions']['rows'])
                (native / 'junctions.tsv').write_text(detail)
                (native / 'junctions-summary.tsv').write_text('metric\tvalue\n' + ''.join(
                    f'{key}\t{count}\n' for key, count in value['junctions']['counts'].items()))
                outputs = [dict(path=name, bytes=(native / name).stat().st_size,
                    sha256=hashlib.sha256((native / name).read_bytes()).hexdigest())
                    for name in ('report.json','junctions.tsv','junctions-summary.tsv')]
                manifest = dict(status='complete', profile='rseqc-5.0.5', selected_metrics=['junctions'], omitted_metrics=['mapping'],
                    bam_traversals=1, original_records=16, inflate_workers=1, outputs=outputs,
                    policy=dict(junctions=dict(excluded_flags=0x704, minimum_mapq=30, minimum_intron=50,
                        omitted_outputs=['R','PDF','BED12','Interact','junction-saturation'])))
                if mutation == 'policy':
                    manifest['policy']['junctions']['excluded_flags'] = 0
                elif mutation == 'digest':
                    manifest['outputs'][0]['sha256'] = '0' * 64
                (native / 'manifest.json').write_text(json.dumps(manifest))
            native_output(report)
            candidate.verify_output(native, expected, 'rseqc-5.0.5', 'junctions', 1, 16)
            for mutation in ('order', 'support', 'counter', 'policy', 'digest'):
                changed = copy.deepcopy(report)
                if mutation == 'order':
                    changed['junctions']['rows'].reverse()
                elif mutation == 'support':
                    changed['junctions']['rows'][0]['read_count'] = 99
                elif mutation == 'counter':
                    changed['junctions']['counts']['filtered_events'] = 0
                native_output(changed, mutation)
                with self.subTest(native_mutation=mutation), self.assertRaises(ValueError):
                    candidate.verify_output(native, expected, 'rseqc-5.0.5', 'junctions', 1, 16)
            for profile in ('debug', 'release'):
                names = authority['tests']['unit'] + authority['tests']['workflow']
                log = '\n'.join(f'test {name} ... ok' for name in names)
                log += '\nrunning 4 tests\nrunning 0 tests\nrunning 8 tests\n'
                log += '\n'.join(f'test result: ok. {n} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s' for n in (4, 0, 8)) + '\n'
                (evidence / f'test-{profile}.log').write_text(log)
            candidate.verify_test_logs(evidence, authority)
            with (evidence / 'test-release.log').open('a') as stream:
                stream.write('test unexpected ... ignored\n')
            with self.assertRaises(ValueError):
                candidate.verify_test_logs(evidence, authority)
            result = subprocess.run(['python3', '-B', str(controller), 'finalize', '--workspace', str(ROOT),
                '--source', str(source), '--evidence', str(evidence)], capture_output=True, text=True, timeout=15)
            self.assertNotEqual(result.returncode, 0)
            final = json.loads((evidence / 'junction-finalization.json').read_text())
            self.assertEqual(final['status'], 'failed')
            self.assertTrue(final['retained_files'])
            self.assertTrue((evidence / 'test-release.log').is_file())
            (evidence / 'junction-finalization.log').write_text(result.stdout + result.stderr)
            candidate.seal(evidence)
            integrity = json.loads((evidence / 'junction-integrity.json').read_text())
            self.assertEqual(integrity['finalization_status'], 'failed')
            self.assertEqual(integrity['files']['junction-finalization.log']['sha256'],
                             hashlib.sha256((result.stdout + result.stderr).encode()).hexdigest())
        acquisition = self.step('Acquire and verify actual RSeQC evidence')
        self.assertIn("if: ${{ inputs.authority != 'junctions' }}", acquisition)
        finalize = self.step('Finalize junction candidate evidence')
        self.assertIn("if: ${{ always() && inputs.authority == 'junctions' }}", finalize)


if __name__ == '__main__':
    unittest.main()
