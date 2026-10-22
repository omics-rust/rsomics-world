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
        self.assertIn('inputs.expected_red &&', self.text)
        self.assertIn("if: ${{ always() && steps.snapshot.outcome == 'success' }}",
                      self.steps['Verify source remains unchanged'])
        self.assertIn('if: always()', self.steps['Upload raw evidence'])
        self.assertIn('path: ${{ runner.temp }}/sc-cnv-evidence/', self.steps['Upload raw evidence'])

    def test_all_bash_blocks_parse(self):
        for name, step in self.steps.items():
            if '        run: |\n' not in step:
                continue
            body = re.sub(r'\$\{\{.*?\}\}', 'stub', bash_body(step))
            with self.subTest(step=name):
                result = subprocess.run(['bash', '-n'], input=body, capture_output=True,
                                        text=True, timeout=15)
                self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == '__main__':
    unittest.main()
