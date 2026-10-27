import itertools
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / '.github/workflows/infercnv-oracle.yml'


def steps(text):
    starts = list(re.finditer(r'^      - name: (.+)$', text, re.MULTILINE))
    if len({match[1] for match in starts}) != len(starts):
        raise ValueError('duplicate workflow step name')
    return {match[1]: text[match.start():starts[index + 1].start()
                              if index + 1 < len(starts) else len(text)]
            for index, match in enumerate(starts)}


def bash_body(step):
    match = re.search(r'^        run: \|\n((?:          .*\n|\n)*)', step, re.MULTILINE)
    if match is None:
        raise ValueError('missing literal Bash block')
    return ''.join(line[10:] if line.startswith('          ') else line
                   for line in match[1].splitlines(keepends=True))


def entry_argv(step, filename):
    lines = [line for line in bash_body(step).splitlines()
             if line.startswith('python3 ') and filename in line]
    if len(lines) != 1:
        raise ValueError('ambiguous Python entrypoint')
    return shlex.split(lines[0].removesuffix('\\').strip())


class IngestionWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.text = WORKFLOW.read_text()
        cls.steps = steps(cls.text)

    def test_raw_selector_is_optional_and_explicit(self):
        inputs = self.text.split('\npermissions:', 1)[0]
        self.assertRegex(inputs, r"(?m)^      ingestion_snapshot:\n"
                         r"        description: [^\n]+\n        type: string\n"
                         r"        required: false\n        default: ''$")
        self.assertIn("INGESTION_SNAPSHOT: ${{ inputs.ingestion_snapshot || '' }}", self.text)

    def test_selector_matrix_executes_real_gate_without_side_effects(self):
        selector = bash_body(self.steps['Validate optional matched snapshot selector'])
        prepared = json.loads((ROOT / '.autopilot/oracles/infercnv-measurement-candidate-2026-09-26.json').read_text())['snapshot_name']
        raw = json.loads((ROOT / '.autopilot/oracles/infercnv-ingestion-trial-candidate-2026-10-15.json').read_text())['snapshot_name']
        for dataset, matched, clustering, contracts, snapshot in itertools.product(
                ('synthetic', 'shipped'), ('', prepared), ('false', 'true'),
                ('false', 'true'), ('', raw, 'unknown-snapshot')):
            label = dataset, matched, clustering, contracts, snapshot
            old_valid = ((contracts == 'false' or dataset == 'shipped' and not matched and clustering == 'false')
                         and (clustering == 'false' or dataset == 'shipped')
                         and (not matched or dataset == 'shipped'))
            valid = old_valid and (not snapshot or snapshot == raw and dataset == 'shipped'
                                  and contracts == 'true' and not matched and clustering == 'false')
            with self.subTest(selection=label), tempfile.TemporaryDirectory(prefix='rsomics-selector-') as directory:
                env = dict(os.environ, ORACLE_DATASET=dataset, MATCHED_SNAPSHOT=matched,
                           SAMPLES_CLUSTERING=clustering, INGESTION_CONTRACTS=contracts,
                           INGESTION_SNAPSHOT=snapshot, EVIDENCE_DIR=directory,
                           GITHUB_ENV=str(Path(directory) / 'environment'))
                result = subprocess.run(['bash', '-c', selector], cwd=ROOT, env=env,
                                        capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode == 0, valid, result.stderr)

    def test_added_steps_are_guarded_and_consume_raw_sources(self):
        names = ('Fetch native raw-ingestion source', 'Authenticate paired ingestion controller',
                 'Build exact raw-ingestion bench', 'Run real paired raw-ingestion campaign')
        for name in names:
            with self.subTest(step=name):
                self.assertTrue(name in self.steps, name)
                self.assertIn("if: env.INGESTION_SNAPSHOT != ''", self.steps[name])
        source = self.steps[names[0]]
        self.assertIn('verify_infercnv_ingestion_snapshot.py', source)
        self.assertIn('infercnv-ingestion-trial-candidate-2026-10-15.json', source)
        build = self.steps[names[2]]
        self.assertIn('--bench cnv_ingestion_matched --message-format=json', build)
        self.assertIn('cargo +1.91.0 build --locked --release --features infercnv-measurement', build)
        self.assertIn('rustup which --toolchain 1.91.0 rustc', build)
        run = self.steps[names[3]]
        self.assertIn('scripts/measure_infercnv_ingestion.py', run)
        self.assertIn('--context "$EVIDENCE_DIR/ingestion-context.json"', run)
        self.assertIn('--output-dir "$EVIDENCE_DIR/ingestion-campaign"', run)
        self.assertIn('set -euo pipefail', run)

    def test_paired_pins_are_literal_complete_and_checked_before_source_use(self):
        name = 'Authenticate paired ingestion controller'
        self.assertTrue(name in self.steps, name)
        body = bash_body(self.steps[name])
        match = re.search(r'cat > "\$EVIDENCE_DIR/ingestion-paired-fixed.sha256" <<\'PINS\'\n(.*?)\nPINS\n',
                          body, re.DOTALL)
        self.assertIsNotNone(match, 'literal pinned-code heredoc is missing')
        pins = {}
        for line in match[1].splitlines():
            self.assertRegex(line, r'^[0-9a-f]{64}  [A-Za-z0-9_./-]+$')
            digest, path = line.split('  ', 1)
            self.assertNotIn(path, pins)
            pins[path] = digest
            self.assertEqual(hashlib.sha256((ROOT / path).read_bytes()).hexdigest(), digest, path)
        import measure_infercnv_ingestion as owner
        self.assertEqual(set(pins), owner.CODE_NAMES)
        self.assertIn('sha256sum --check "$EVIDENCE_DIR/ingestion-paired-fixed.sha256"', body)
        self.assertLess(self.text.index('- name: ' + name),
                        self.text.index('- name: Fetch native raw-ingestion source'))

    def test_failure_postchecks_and_upload_are_unconditional(self):
        self.assertTrue('Verify paired ingestion source after campaign' in self.steps)
        post = self.steps['Verify paired ingestion source after campaign']
        self.assertIn("if: ${{ always() && inputs.ingestion_snapshot != '' }}", post)
        self.assertIn('sha256sum --check', post)
        self.assertIn('verify_extracted', post)
        self.assertIn('if: always()', self.steps['Upload raw evidence even on failure'])

    def test_upload_retains_hidden_authenticated_receipts_only_from_evidence_directory(self):
        upload = self.steps['Upload raw evidence even on failure']
        self.assertIn('include-hidden-files: true', upload)
        self.assertIn('path: ${{ runner.temp }}/infercnv-oracle/', upload)
        self.assertNotIn('${{ github.workspace }}', upload)

    def test_real_r_boundary_gate_is_opted_in_after_installation_and_retained(self):
        name = 'Test actual installed R boundary'
        self.assertTrue(name in self.steps, name)
        step = self.steps[name]
        self.assertIn("if: env.INGESTION_SNAPSHOT != ''", step)
        self.assertIn("RSOMICS_TEST_INSTALLED_R: '1'", step)
        self.assertIn('RSOMICS_TEST_EVIDENCE_DIR:', step)
        self.assertIn('ingestion-installed-boundary', step)
        self.assertIn('-k InstalledRBoundaryTests -v', step)
        self.assertIn("'^Ran [1-9][0-9]* tests? in '", step)
        self.assertIn("grep -q 'skipped'", step)
        self.assertIn('sha256sum --check "$EVIDENCE_DIR/ingestion-paired-tests.sha256"', step)
        self.assertLess(self.text.index('- name: Install exact source and verify package'),
                        self.text.index('- name: ' + name))
        self.assertLess(self.text.index('- name: ' + name),
                        self.text.index('- name: Run real paired raw-ingestion campaign'))

    def test_raw_entrypoint_loads_authenticated_siblings_under_safe_path(self):
        argv = entry_argv(self.steps['Fetch native raw-ingestion source'],
                          'verify_infercnv_ingestion_snapshot.py')
        env = dict(os.environ, PYTHONSAFEPATH='1', PYTHONDONTWRITEBYTECODE='1')
        env.pop('PYTHONPATH', None)
        env.pop('PYTHONHOME', None)
        result = subprocess.run([*argv, '--help'], cwd=ROOT, env=env,
                                capture_output=True, text=True, timeout=15)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('--receipt', result.stdout)

    def test_owner_entrypoint_loads_real_fallback_process_owner_under_safe_path(self):
        import test_measure_infercnv_ingestion as fixtures
        fixture = fixtures.ExternalFixture('runTest')
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        context, _ = fixture.context()
        output = fixture.root / 'failed admission evidence'
        argv = entry_argv(self.steps['Run real paired raw-ingestion campaign'],
                          'measure_infercnv_ingestion.py')
        env = dict(os.environ, PYTHONSAFEPATH='1', PYTHONDONTWRITEBYTECODE='1')
        env.pop('PYTHONPATH', None)
        env.pop('PYTHONHOME', None)
        result = subprocess.run([*argv, '--context', str(context), '--output-dir', str(output)],
                                cwd=ROOT, env=env, capture_output=True, text=True, timeout=30)
        self.assertNotEqual(result.returncode, 0)
        failure = json.loads((output / 'failure.json').read_text())
        self.assertNotIn('ModuleNotFoundError', json.dumps(failure))
        self.assertTrue((output / 'after-python-shell.process.json').is_file())

    def postcheck_fixture(self):
        directory = tempfile.TemporaryDirectory(prefix='rsomics-fallback-')
        self.addCleanup(directory.cleanup)
        root = Path(directory.name).resolve() / 'retained build evidence'
        root.mkdir()
        auth = bash_body(self.steps['Authenticate paired ingestion controller'])
        for name in ('ingestion-paired-fixed.sha256', 'ingestion-paired-tests.sha256'):
            literal = re.search(re.escape(f'cat > "$EVIDENCE_DIR/{name}"') +
                                r" <<'PINS'\n(.*?)\nPINS\n", auth, re.DOTALL)
            self.assertIsNotNone(literal)
            (root / name).write_text(literal[1] + '\n')
        from infercnv_source_snapshot import verify_and_extract
        source = root / 'verified source'
        verify_and_extract(ROOT / '.autopilot/snapshots/sc-cnv-core-2026-09-26/green-14', source)
        target = root / 'target'
        native = target / 'release/deps/cnv_ingestion_matched-fixture'
        native.parent.mkdir(parents=True)
        paths = dict(native=native, rustc=root / 'rustc', python=root / 'python',
                     rscript=root / 'Rscript', r=root / 'R')
        for path in paths.values():
            path.write_bytes(b'controlled executable identity; never executed\n')
            path.chmod(0o700)
        paths['time'] = Path('/usr/bin/time').resolve(strict=True)
        names = dict(python='python', rscript='rscript', r='r', rustc='rustc')
        for key, record_name in names.items():
            (root / f'ingestion-{record_name}-executable.txt').write_text(str(paths[key]) + '\n')
        artifact = dict(reason='compiler-artifact', package_id=f'path+file://{source}#rsomics-sc@0.1.0',
            manifest_path=str(source / 'Cargo.toml'),
            target=dict(kind=['bench'], crate_types=['bin'], name='cnv_ingestion_matched',
                        src_path=str(source / 'benches/cnv_ingestion_matched.rs'), edition='2024',
                        doc=False, doctest=False, test=False, **{'required-features': ['infercnv-measurement']}),
            profile=dict(opt_level='3', debuginfo=0, debug_assertions=False, overflow_checks=False, test=True),
            features=['infercnv-measurement'], filenames=[str(native)], executable=str(native), fresh=False)
        (root / 'ingestion-cargo-build.jsonl').write_text(json.dumps(artifact) + '\n' +
                                                      json.dumps(dict(reason='build-finished', success=True)) + '\n')
        def manifest(keys):
            return ''.join(hashlib.sha256(paths[key].read_bytes()).hexdigest() + '  ' + str(paths[key]) + '\n'
                           for key in keys)
        (root / 'ingestion-executables.sha256').write_text(manifest(('python', 'rscript', 'r')))
        (root / 'ingestion-native-executables.sha256').write_text(manifest(paths))
        env = dict(os.environ, EVIDENCE_DIR=str(root), RUST_SOURCE=str(source),
                   CARGO_TARGET_DIR=str(target), INGESTION_SNAPSHOT='green-14',
                   PYTHONSAFEPATH='1', PYTHONDONTWRITEBYTECODE='1')
        env.pop('PYTHONPATH', None)
        env.pop('PYTHONHOME', None)
        return root, env, paths

    def test_build_identity_capture_uses_actual_cargo_artifact_and_all_six_executables(self):
        body = bash_body(self.steps['Build exact raw-ingestion bench'])
        code = re.search(r"^python3 -B - <<'PY' > \"\$EVIDENCE_DIR/ingestion-native-executables.sha256\"\n(.*?)^PY$",
                         body, re.MULTILINE | re.DOTALL)
        self.assertIsNotNone(code, 'independent built-executable identity capture is missing')
        root, env, paths = self.postcheck_fixture()
        result = subprocess.run(['python3', '-B', '-'], input=code[1], cwd=ROOT, env=env,
                                capture_output=True, text=True, timeout=15)
        self.assertEqual(result.returncode, 0, result.stderr)
        actual = dict((name, digest) for digest, name in
                      (line.split('  ', 1) for line in result.stdout.splitlines()))
        self.assertEqual(actual, {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
                                  for path in paths.values()})

    def test_fallback_catches_native_and_rustc_drift_when_owner_never_started(self):
        body = bash_body(self.steps['Verify paired ingestion source after campaign'])
        for key in ('native', 'rustc'):
            with self.subTest(executable=key):
                root, env, paths = self.postcheck_fixture()
                paths[key].write_bytes(b'changed after successful build, before context creation\n')
                result = subprocess.run(['bash', '-c', body], cwd=ROOT, env=env,
                                        capture_output=True, text=True, timeout=20)
                self.assertNotEqual(result.returncode, 0)
                log = root / 'ingestion-native-executables-after.log'
                self.assertTrue(log.is_file(), 'native fallback postcheck was omitted')
                self.assertIn(str(paths[key]) + ': FAILED', log.read_text())
                self.assertTrue((root / 'ingestion-source-after.json').is_file())
                self.assertEqual(json.loads((root / 'ingestion-source-after.json').read_text())['verified_files'], 149)

    def test_missing_native_identity_baseline_fails_without_skipping_source_and_old_checks(self):
        root, env, _ = self.postcheck_fixture()
        baseline = root / 'ingestion-native-executables.sha256'
        baseline.rename(root / 'preserved-before-unavailability.sha256')
        result = subprocess.run(['bash', '-c', bash_body(self.steps['Verify paired ingestion source after campaign'])],
                                cwd=ROOT, env=env, capture_output=True, text=True, timeout=20)
        self.assertNotEqual(result.returncode, 0)
        log = root / 'ingestion-native-executables-after.log'
        self.assertTrue(log.is_file(), 'unavailable native baseline was not recorded')
        self.assertIn('ingestion-native-executables.sha256', log.read_text())
        self.assertTrue((root / 'ingestion-paired-executables-after.log').is_file())
        self.assertEqual(json.loads((root / 'ingestion-source-after.json').read_text())['verified_files'], 149)

    def test_all_literal_bash_blocks_parse(self):
        for name, step in self.steps.items():
            if '        run: |' not in step:
                continue
            with self.subTest(step=name):
                result = subprocess.run(['bash', '-n'], input=bash_body(step),
                                        capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode, 0, result.stderr)

    def test_denoise_selector_is_closed_and_rejects_mixed_campaigns(self):
        body = bash_body(self.steps['Validate optional matched snapshot selector'])
        candidate = 'infercnv-denoise-candidate-2026-10-26'
        for receipt, dataset, clustering, valid in [
            ('none', 'synthetic', 'false', True),
            (candidate, 'shipped', 'false', True),
            (candidate, 'synthetic', 'false', False),
            (candidate, 'shipped', 'true', False),
            ('../' + candidate, 'shipped', 'false', False),
        ]:
            with self.subTest(receipt=receipt, dataset=dataset, clustering=clustering):
                env = dict(os.environ, DENOISE_NATIVE_RECEIPT=receipt, ORACLE_DATASET=dataset,
                           SAMPLES_CLUSTERING=clustering, MATCHED_SNAPSHOT='',
                           INGESTION_SNAPSHOT='', INGESTION_CONTRACTS='false')
                result = subprocess.run(['bash', '-c', body], cwd=ROOT, env=env,
                                        capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode == 0, valid, result.stderr)

    def test_hmm_selector_requires_shipped_inputs_and_an_unmixed_workflow(self):
        body = bash_body(self.steps['Validate optional matched snapshot selector'])
        candidate = 'infercnv-denoise-candidate-2026-10-26'
        for dataset, clustering, denoise, contracts, valid in [
            ('shipped', 'false', 'none', 'false', True),
            ('synthetic', 'false', 'none', 'false', False),
            ('shipped', 'true', 'none', 'false', False),
            ('shipped', 'false', candidate, 'false', False),
            ('shipped', 'false', 'none', 'true', False),
        ]:
            with self.subTest(selection=(dataset, clustering, denoise, contracts)):
                env = dict(os.environ, HMM_WORKFLOWS='true', DENOISE_NATIVE_RECEIPT=denoise,
                           ORACLE_DATASET=dataset, SAMPLES_CLUSTERING=clustering,
                           MATCHED_SNAPSHOT='', INGESTION_SNAPSHOT='',
                           INGESTION_CONTRACTS=contracts)
                result = subprocess.run(['bash', '-c', body], cwd=ROOT, env=env,
                                        capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode == 0, valid, result.stderr)

    def test_extension_selector_rejects_unrelated_campaigns_before_installing(self):
        body = bash_body(self.steps['Validate optional matched snapshot selector'])
        for dataset, hmm, clustering, valid in [
            ('shipped', 'false', 'false', True),
            ('synthetic', 'false', 'false', False),
            ('shipped', 'true', 'false', False),
            ('shipped', 'false', 'true', False),
        ]:
            with self.subTest(selection=(dataset, hmm, clustering)):
                env = dict(os.environ, HMM_EXTENSION_WORKFLOWS='true', HMM_WORKFLOWS=hmm,
                           ORACLE_DATASET=dataset, SAMPLES_CLUSTERING=clustering,
                           DENOISE_NATIVE_RECEIPT='none', MATCHED_SNAPSHOT='',
                           INGESTION_SNAPSHOT='', INGESTION_CONTRACTS='false')
                result = subprocess.run(['bash', '-c', body], cwd=ROOT, env=env,
                                        capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode == 0, valid, result.stderr)

    def test_hmm_workflow_uses_real_r_and_skips_unrelated_ingestion_probes(self):
        self.assertTrue('Capture integrated i6 HMM workflows' in self.steps,
                        'missing integrated HMM step')
        step = self.steps['Capture integrated i6 HMM workflows']
        self.assertIn("if: env.HMM_WORKFLOWS == 'true'", step)
        self.assertIn('Rscript --vanilla scripts/infercnv_hmm_workflow.R', step)
        for name in ('Test ingestion witness checker', 'Capture creation-stage raw-input witnesses',
                     'Capture creation format and filter edge probes'):
            self.assertIn("env.HMM_WORKFLOWS != 'true'", self.steps[name])


if __name__ == '__main__':
    unittest.main()
