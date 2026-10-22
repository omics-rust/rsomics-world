import itertools
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / '.github/workflows/infercnv-oracle.yml'
PLAN = ROOT / 'docs/plans/2026-10-15-infercnv-r-package-contracts.md'


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

    def test_all_literal_bash_blocks_parse(self):
        for name, step in self.steps.items():
            if '        run: |' not in step:
                continue
            with self.subTest(step=name):
                result = subprocess.run(['bash', '-n'], input=bash_body(step),
                                        capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode, 0, result.stderr)

    def test_embedded_workflow_matches_real_file(self):
        section = PLAN.read_text().split('## Complete pinned-oracle workflow\n', 1)[1]
        embedded = section.split('```yaml\n', 1)[1].split('\n```', 1)[0]
        self.assertEqual(embedded + '\n', self.text)


if __name__ == '__main__':
    unittest.main()
