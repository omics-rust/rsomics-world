import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest
import json

from test_infercnv_ingestion_workflow import bash_body, steps


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / '.github/workflows/sc-cnv-core-candidate.yml'
WITNESS = 'infercnv-samples-clustering-witness-2026-09-30'
LEGACY = 'infercnv-shipped-2026-09-26'


class SamplesCandidateWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.text = WORKFLOW.read_text()
        cls.steps = steps(cls.text)

    def test_exact_choices_and_early_selection(self):
        selection = self.text.split('      oracle_receipt:\n', 1)[1].split('      resolve_input_lock:', 1)[0]
        self.assertIn('        default: none\n', selection)
        self.assertEqual(re.findall(r'^          - (.+)$', selection, re.MULTILINE),
                         ['none', LEGACY, WITNESS])
        self.assertIn('Validate oracle selection', self.steps)
        names = list(self.steps)
        self.assertLess(names.index('Validate oracle selection'),
                        names.index('Verify and extract frozen source'))
        self.assertLess(names.index('Validate oracle selection'),
                        names.index('Acquire and verify pinned external oracle'))

    def test_real_selector_rejects_unknown_and_paths(self):
        self.assertIn('Validate oracle selection', self.steps)
        body = bash_body(self.steps['Validate oracle selection'])
        for selector, feature in [('none', ''), (LEGACY, 'external-infercnv-oracle'),
                                  (WITNESS, 'external-infercnv-samples-oracle'),
                                  ('', None), ('../' + WITNESS, None), ('unknown', None)]:
            with self.subTest(selector=selector), tempfile.TemporaryDirectory() as directory:
                output = Path(directory) / 'env'
                result = subprocess.run(['bash', '-c', body], cwd=ROOT,
                                        env=dict(os.environ, ORACLE_RECEIPT=selector,
                                                 GITHUB_ENV=str(output)),
                                        capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode == 0, feature is not None, result.stderr)
                if feature is not None:
                    self.assertEqual(output.read_text(), f'ORACLE_FEATURES={feature}\n')
                else:
                    self.assertFalse(output.exists())

    def test_authenticated_acquisition_preserves_legacy_and_adds_full_witness(self):
        step = self.steps['Acquire and verify pinned external oracle']
        self.assertIn("if: ${{ inputs.oracle_receipt != 'none' }}", step)
        body = bash_body(step)
        self.assertNotIn('test "$ORACLE_RECEIPT" = ' + LEGACY, body)
        for name in ['verify_infercnv_oracle_artifact.py', 'validate_infercnv_oracle.py',
                     'validate_infercnv_shipped_oracle.py',
                     'validate_infercnv_samples_clustering_witness.py']:
            self.assertIn(name, body)
        witness_gate = body.index('if [[ "$ORACLE_RECEIPT" == ' + WITNESS)
        validation = body.index('validate_infercnv_samples_clustering_witness.py')
        export = body.index('RSOMICS_INFER_CNV_SAMPLES_ROOT=')
        self.assertLess(witness_gate, validation)
        self.assertLess(validation, export)
        self.assertIn('RSOMICS_INFER_CNV_ORACLE_ROOT=', body)
        self.assertIn('"$RUNNER_TEMP/infercnv-oracle-verified/bundle"', body)
        self.assertIn('"$oracle_root"', body)
        self.assertIn('"$RUNNER_TEMP/infercnv-oracle-verified/samples-clustering"', body)
        self.assertNotIn('cp -r', body)

    def test_closed_feature_reaches_both_tests_and_strict_clippy(self):
        for name in ['Test debug', 'Test release']:
            body = bash_body(self.steps[name])
            self.assertIn('if [[ -n "$ORACLE_FEATURES" ]]', body)
            self.assertIn('test_args+=(--features "$ORACLE_FEATURES")', body)
            self.assertIn('--all-targets --no-fail-fast', body)
        body = bash_body(self.steps['Format and strict Clippy'])
        self.assertIn('clippy_args+=(--all-features)', body)
        self.assertIn('elif [[ -n "$ORACLE_FEATURES" ]]', body)
        self.assertIn('clippy_args+=(--features "$ORACLE_FEATURES")', body)
        self.assertIn('-- -D warnings', body)

    def test_real_acquisition_shell_routes_and_rejects_failed_validation(self):
        body = bash_body(self.steps['Acquire and verify pinned external oracle'])
        for selector, fail in [(LEGACY, False), (WITNESS, False), (WITNESS, True)]:
            with self.subTest(selector=selector, fail=fail), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                (root / 'scripts').mkdir()
                (root / '.autopilot/oracles').mkdir(parents=True)
                (root / 'evidence').mkdir()
                (root / '.autopilot/oracles' / (selector + '.json')).write_text(json.dumps({
                    'repository': 'omics-rust/rsomics-world', 'run_id': 1, 'artifact_id': 2}))
                stub = root / 'scripts/stub.py'
                stub.write_text('import os, sys\nfrom pathlib import Path\n'
                                'name = Path(sys.argv[0]).name\n'
                                'with open(os.environ["TRACE"], "a") as trace: trace.write(name + "\\n")\n'
                                'if name == "verify_infercnv_oracle_artifact.py":\n'
                                ' p = Path(sys.argv[sys.argv.index("--output-dir") + 1]) / "shipped-bundle"\n'
                                ' p.mkdir(parents=True)\n'
                                ' for n in ("sha256-manifest.tsv", "oracle.json"): (p/n).touch()\n'
                                'if name.startswith("validate_infercnv_samples") and os.environ["FAIL"] == "1": sys.exit(3)\n')
                for name in ['verify_infercnv_oracle_artifact.py', 'validate_infercnv_oracle.py',
                             'validate_infercnv_shipped_oracle.py',
                             'validate_infercnv_samples_clustering_witness.py']:
                    (root / 'scripts' / name).write_text(stub.read_text())
                output = root / 'environment'
                result = subprocess.run(['bash', '-c', 'gh() { printf "{}"; };\n' + body], cwd=root,
                                        env=dict(os.environ, ORACLE_RECEIPT=selector, EVIDENCE=str(root/'evidence'),
                                                 GITHUB_WORKSPACE=str(root), RUNNER_TEMP=str(root),
                                                 GITHUB_ENV=str(output), TRACE=str(root/'trace'), FAIL=str(int(fail))),
                                        capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode == 0, not fail, result.stderr)
                calls = (root / 'trace').read_text().splitlines()
                self.assertEqual(calls[:3], ['verify_infercnv_oracle_artifact.py',
                                            'validate_infercnv_oracle.py', 'validate_infercnv_shipped_oracle.py'])
                exports = output.read_text()
                self.assertIn('RSOMICS_INFER_CNV_ORACLE_ROOT=', exports)
                self.assertEqual('RSOMICS_INFER_CNV_SAMPLES_ROOT=' in exports, selector == WITNESS and not fail)
                self.assertEqual(len(calls), 4 if selector == WITNESS else 3)

    def test_both_missing_root_gates_require_actual_failed_test(self):
        legacy = bash_body(self.steps['Require explicit external oracle configuration'])
        self.assertIn('env -u RSOMICS_INFER_CNV_ORACLE_ROOT cargo test --locked', legacy)
        self.assertIn('test result: FAILED', legacy)
        name = 'Require explicit samples oracle configuration'
        self.assertIn(name, self.steps)
        step = self.steps[name]
        self.assertIn("inputs.oracle_receipt == '" + WITNESS + "'", step)
        self.assertIn("!inputs.expected_red && matrix.target == 'linux-x86_64'", step)
        body = bash_body(step)
        self.assertIn('env -u RSOMICS_INFER_CNV_SAMPLES_ROOT cargo test --locked', body)
        self.assertIn('--features external-infercnv-samples-oracle --test cnv_external_samples_oracle', body)
        self.assertIn('RSOMICS_INFER_CNV_SAMPLES_ROOT is required', body)
        self.assertIn('test result: FAILED', body)
        self.assertIn('test "$test_rc" -ne 0', body)

    def test_native_matrix_postchecks_and_upload_are_preserved(self):
        for target in ['linux-x86_64', 'linux-aarch64', 'macos-x86_64', 'macos-aarch64']:
            self.assertIn('"target":"' + target + '"', self.text)
        self.assertIn('inputs.expected_red || inputs.samples_benchmark', self.text)
        self.assertIn("if: ${{ always() && steps.snapshot.outcome == 'success' }}",
                      self.steps['Verify source remains unchanged'])
        self.assertIn('if: always()', self.steps['Upload raw evidence'])
        self.assertIn('path: ${{ runner.temp }}/sc-cnv-evidence/', self.steps['Upload raw evidence'])

    def test_hmm_runs_the_complete_real_workflow_on_each_native_target(self):
        acquire = self.steps['Acquire actual HMM workflow evidence']
        self.assertIn('if: ${{ inputs.hmm_workflows || inputs.bayes_workflows }}', acquire)
        body = bash_body(acquire)
        self.assertIn('test "$EXPECT_RED" = false', body)
        self.assertIn('infercnv-hmm-workflow-2026-10-27.json', body)
        self.assertIn('verify_infercnv_oracle_artifact.py', body)
        compare = self.steps['Compare complete native HMM workflows']
        self.assertNotIn("matrix.target ==", compare)
        body = bash_body(compare)
        self.assertIn('compare_infercnv_hmm_workflow.py', body)
        for argument in ['--synthetic', '--shipped', '--witness', '--binary', '--output']:
            self.assertIn(argument, body)
        self.assertIn('$CARGO_TARGET_DIR/release/rsomics-sc', body)
        self.assertLess(list(self.steps).index('Test release'),
                        list(self.steps).index('Compare complete native HMM workflows'))

    def test_bayesian_selector_rejects_mixed_campaigns_before_source_or_downloads(self):
        selection = self.text.split('      bayes_workflows:\n', 1)[1].split('      measurement:', 1)[0]
        self.assertIn('        default: false\n', selection)
        name = 'Validate Bayesian workflow selection'
        body = bash_body(self.steps[name])
        self.assertLess(list(self.steps).index(name), list(self.steps).index('Verify and extract frozen source'))
        base = dict(os.environ, ORACLE_RECEIPT='none', EXPECT_RED='false', SAMPLES_BENCHMARK='false',
                    HMM_WORKFLOWS='false', MEASUREMENT='false', INGESTION_MEASUREMENT='false',
                    INGESTION_TRIAL='false', RESOLVE_MEASUREMENT_LOCK='false', RESOLVE_INPUT_LOCK='false')
        for change in [{}, {'EXPECT_RED': 'true'}, {'HMM_WORKFLOWS': 'true'}, {'ORACLE_RECEIPT': WITNESS}]:
            with self.subTest(change=change):
                result = subprocess.run(['bash', '-c', body], env=dict(base, **change),
                                        capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode == 0, not change, result.stderr)

    def test_bayesian_campaign_verifies_actual_replay_and_runs_each_native_binary(self):
        acquire = self.steps['Acquire actual Bayesian workflow evidence']
        self.assertIn('if: ${{ inputs.bayes_workflows }}', acquire)
        body = bash_body(acquire)
        self.assertIn('infercnv-bayes-workflow-2026-10-27.json', body)
        self.assertIn('verify_infercnv_oracle_artifact.py', body)
        compare = self.steps['Compare complete native Bayesian workflow']
        self.assertNotIn('matrix.target ==', compare)
        body = bash_body(compare)
        self.assertIn('compare_infercnv_bayes_workflow.py', body)
        for argument in ['--shipped', '--hmm', '--bayes', '--uncertainty', '--binary', '--output']:
            self.assertIn(argument, body)
        self.assertIn('$CARGO_TARGET_DIR/release/rsomics-sc', body)
        self.assertLess(list(self.steps).index('Test release'),
                        list(self.steps).index('Compare complete native Bayesian workflow'))

    def test_discovery_selector_admits_only_complete_standalone_and_bootstrap_campaigns(self):
        for option in ['discovery_workflows', 'bootstrap_dependencies']:
            self.assertRegex(self.text, rf'      {option}:\n(?:        .*\n)*?        default: false\n')
        matrix = re.search(r"fromJSON\(\((.*?)\) && '(.*?)' \|\| '(.*?)'\)", self.text)
        self.assertEqual(matrix[1], 'inputs.expected_red || inputs.samples_benchmark || inputs.bootstrap_dependencies')
        self.assertEqual([target['target'] for target in json.loads(matrix[2])], ['linux-x86_64'])
        self.assertEqual([target['target'] for target in json.loads(matrix[3])],
                         ['linux-x86_64', 'linux-aarch64', 'macos-x86_64', 'macos-aarch64'])
        name = 'Validate discovery workflow selection'
        self.assertIn(name, self.steps)
        body = bash_body(self.steps[name])
        self.assertLess(list(self.steps).index(name), list(self.steps).index('Verify and extract frozen source'))
        base = dict(os.environ, ORACLE_RECEIPT='none', EXPECT_RED='false', SAMPLES_BENCHMARK='false',
                    HMM_WORKFLOWS='false', BAYES_WORKFLOWS='false', MEASUREMENT='false',
                    INGESTION_MEASUREMENT='false', INGESTION_TRIAL='false',
                    RESOLVE_MEASUREMENT_LOCK='false', RESOLVE_INPUT_LOCK='false',
                    DISCOVERY_WORKFLOWS='true', BOOTSTRAP_DEPENDENCIES='false')
        for change, accepted in [({}, True), ({'BOOTSTRAP_DEPENDENCIES': 'true'}, True),
                                 ({'DISCOVERY_WORKFLOWS': 'false', 'BOOTSTRAP_DEPENDENCIES': 'true'}, False),
                                 ({'EXPECT_RED': 'true'}, False), ({'HMM_WORKFLOWS': 'true'}, False),
                                 ({'BAYES_WORKFLOWS': 'true'}, False), ({'MEASUREMENT': 'true'}, False),
                                 ({'SAMPLES_BENCHMARK': 'true'}, False), ({'ORACLE_RECEIPT': WITNESS}, False)]:
            with self.subTest(change=change):
                result = subprocess.run(['bash', '-c', body], env=dict(base, **change),
                                        capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode == 0, accepted, result.stderr)

    def test_bootstrap_reports_new_nodes_and_rejects_changed_existing_pins(self):
        name = 'Bootstrap discovery dependency lock'
        self.assertIn(name, self.steps)
        body = bash_body(self.steps[name])
        self.assertIn('cargo metadata --all-features --format-version 1', body)
        self.assertNotIn('cargo update', body)
        self.assertNotIn('cargo generate-lockfile', body)
        self.assertIn('"${BOOTSTRAP_DEPENDENCIES:-false}" == true',
                      bash_body(self.steps['Verify source remains unchanged']))
        program = re.findall(r"python3 -B - <<'PY'\n(.*?)\nPY", body, re.DOTALL)[0]
        before = ('version = 4\n[[package]]\nname = "rsomics-sc"\nversion = "0.1.0"\n'
                  'dependencies = ["clap"]\n[[package]]\nname = "clap"\nversion = "4.6.7"\n'
                  'source = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "aaa"\n')
        after = before.replace('["clap"]', '["clap", "cc"]') + (
            '[[package]]\nname = "cc"\nversion = "1.2.63"\n'
            'source = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "bbb"\n')
        for mutation, accepted in [('', True), ('checksum', False), ('version', False)]:
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                own = after.replace('checksum = "aaa"', 'checksum = "changed"') if mutation == 'checksum' else after
                if mutation == 'version':
                    own = own.replace('version = "4.6.7"', 'version = "4.6.8"')
                (root / 'Cargo.before.lock').write_text(before)
                (root / 'Cargo.lock').write_text(own)
                result = subprocess.run(['python3', '-B', '-c', program], cwd=root,
                                        env=dict(os.environ, EVIDENCE=str(root)),
                                        capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode == 0, accepted, result.stderr)
                summary = json.loads((root / 'lock-resolution-summary.json').read_text())
                self.assertEqual([package['name'] for package in summary['added_packages']],
                                 ['cc'] if mutation != 'version' else ['cc', 'clap'])
                self.assertEqual(summary['preserved_existing_pins'], accepted)

    def test_discovery_acquisition_keeps_both_actual_receipts_and_same_native_binary(self):
        name = 'Acquire actual discovery workflow evidence'
        self.assertIn(name, self.steps)
        body = bash_body(self.steps[name])
        for fail in [False, True]:
            with self.subTest(fail=fail), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                (root / 'scripts').mkdir()
                (root / '.autopilot/oracles').mkdir(parents=True)
                (root / 'evidence').mkdir()
                for name, run in [('infercnv-leiden-workflow-2026-10-27', 20),
                                  ('infercnv-leiden-k5-workflow-2026-10-27', 5)]:
                    (root / '.autopilot/oracles' / (name + '.json')).write_text(json.dumps({
                        'repository': 'omics-rust/rsomics-world', 'run_id': run, 'artifact_id': run + 1}))
                (root / 'scripts/verify_infercnv_oracle_artifact.py').write_text(
                    'import os,sys\nfrom pathlib import Path\n'
                    'out = Path(sys.argv[sys.argv.index("--output-dir") + 1])\n'
                    'with open(os.environ["TRACE"], "a") as trace: trace.write(out.name + "\\n")\n'
                    'if os.environ["FAIL"] == "1": sys.exit(23)\n'
                    '(out/"shipped-bundle").mkdir(parents=True)\n'
                    '(out/"leiden-workflows").mkdir()\n'
                    'for path in ["shipped-bundle/sha256-manifest.tsv", "leiden-workflows/witness.json", '
                    '"leiden-workflows/sha256-manifest.tsv"]: (out/path).touch()\n')
                environment = root / 'environment'
                result = subprocess.run(['bash', '-c', 'gh() { printf "{}"; };\n' + body], cwd=root,
                                        env=dict(os.environ, EVIDENCE=str(root / 'evidence'),
                                                 GITHUB_WORKSPACE=str(root), RUNNER_TEMP=str(root),
                                                 GITHUB_ENV=str(environment), TRACE=str(root / 'trace'),
                                                 FAIL=str(int(fail))), capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode, 23 if fail else 0, result.stderr)
                exports = environment.read_text() if environment.exists() else ''
                self.assertEqual('DISCOVERY_K20_ROOT=' in exports, not fail)
                self.assertEqual('DISCOVERY_K5_ROOT=' in exports, not fail)
                self.assertEqual((root / 'trace').read_text().splitlines(),
                                 ['discovery-k20'] if fail else ['discovery-k20', 'discovery-k5'])
        compare = self.steps['Compare complete native discovery workflows']
        self.assertNotIn('matrix.target ==', compare)
        self.assertIn('compare_infercnv_discovery_workflow.py', compare)
        self.assertIn('for profile in k20 k5', compare)
        self.assertIn('$CARGO_TARGET_DIR/release/rsomics-sc', compare)
        self.assertLess(list(self.steps).index('Test release'),
                        list(self.steps).index('Compare complete native discovery workflows'))

    def test_all_bash_blocks_parse(self):
        for name, step in self.steps.items():
            if '        run: |\n' not in step:
                continue
            body = re.sub(r'\$\{\{.*?\}\}', 'stub', bash_body(step))
            with self.subTest(step=name):
                result = subprocess.run(['bash', '-n'], input=body, capture_output=True,
                                        text=True, timeout=15)
                self.assertEqual(result.returncode, 0, result.stderr)

    def test_metadata_admits_only_private_product_and_optional_single_binary(self):
        body = bash_body(self.steps['Verify dependency resolution and output path'])
        program = re.findall(r"python3 -B - <<'PY'\n(.*?)\nPY", body, re.DOTALL)[-1]
        for binaries, publish, accepted in [([], [], True), (['rsomics-sc'], [], True),
                                             (['other'], [], False),
                                             (['rsomics-sc', 'other'], [], False),
                                             (['rsomics-sc'], None, False)]:
            with self.subTest(binaries=binaries, publish=publish), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                metadata = {
                    'target_directory': str(root / 'target'),
                    'packages': [{'name': 'rsomics-sc', 'publish': publish,
                                  'targets': [{'name': 'rsomics_sc', 'kind': ['lib']}] +
                                  [{'name': name, 'kind': ['bin']} for name in binaries]}],
                }
                (root / 'metadata.json').write_text(json.dumps(metadata))
                result = subprocess.run(['python3', '-B', '-c', program],
                                        env=dict(os.environ, EVIDENCE=str(root),
                                                 CARGO_TARGET_DIR=str(root / 'target')),
                                        capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode == 0, accepted, result.stderr)

    def test_benchmark_selector_rejects_unbound_or_mixed_measurements(self):
        body = bash_body(self.steps['Validate samples benchmark selection'])
        base = dict(os.environ, ORACLE_RECEIPT=WITNESS, EXPECT_RED='false',
                    MEASUREMENT='false', INGESTION_MEASUREMENT='false', INGESTION_TRIAL='false',
                    RESOLVE_MEASUREMENT_LOCK='false', RESOLVE_INPUT_LOCK='false')
        for change in [{}, {'ORACLE_RECEIPT': 'none'}, {'EXPECT_RED': 'true'},
                       {'MEASUREMENT': 'true'}, {'INGESTION_MEASUREMENT': 'true'},
                       {'INGESTION_TRIAL': 'true'}, {'RESOLVE_INPUT_LOCK': 'true'},
                       {'RESOLVE_MEASUREMENT_LOCK': 'true'}]:
            with self.subTest(change=change):
                result = subprocess.run(['bash', '-c', body], env=dict(base, **change),
                                        capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode == 0, not change, result.stderr)

    def test_failed_benchmark_retains_the_exact_binary_before_invocation(self):
        body = bash_body(self.steps['Measure canonical samples CLI worker scaling'])
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            evidence = root / 'evidence'
            evidence.mkdir()
            (root / 'target/release').mkdir(parents=True)
            binary = root / 'target/release/rsomics-sc'
            binary.write_bytes(b'exact candidate bytes')
            (root / 'scripts').mkdir()
            for name in ['measure_infercnv_samples_cli.py', 'infercnv_trial_process.py',
                         'measure_infercnv.py', 'validate_infercnv_shipped_oracle.py',
                         'test_measure_infercnv_samples_cli.py']:
                (root / 'scripts' / name).write_text('fixture\n')
            commands = ('cargo() { :; }; lscpu() { :; }; python3() { return 47; };\n'
                        'sha256sum() { shasum -a 256 "$@"; };\n')
            result = subprocess.run(['bash', '-c', commands + body], cwd=root,
                                    env=dict(os.environ, EVIDENCE=str(evidence),
                                             GITHUB_WORKSPACE=str(root),
                                             CARGO_TARGET_DIR=str(root / 'target'),
                                             RSOMICS_INFER_CNV_ORACLE_ROOT=str(root / 'oracle')),
                                    capture_output=True, text=True, timeout=15)
            self.assertEqual(result.returncode, 47, result.stderr)
            retained = evidence / 'samples-benchmark-binary/rsomics-sc-linux-x86_64'
            self.assertEqual(retained.read_bytes(), b'exact candidate bytes')
            self.assertTrue((evidence / 'samples-benchmark-binary.sha256').is_file())
            self.assertFalse((evidence / 'samples-cli-benchmark/summary.json').exists())


if __name__ == '__main__':
    unittest.main()
