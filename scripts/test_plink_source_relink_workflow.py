"""One integrated source-delivery preparation check; no compilation."""

import importlib
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest

from test_infercnv_ingestion_workflow import bash_body, steps

ROOT = Path(__file__).resolve().parents[1]


class PlinkSourceRelinkPreparationTest(unittest.TestCase):
    def test_complete_offline_source_and_controlled_codec_replacement(self):
        module = ROOT / 'scripts/plink_source_relink.py'
        workflow = ROOT / '.github/workflows/plink-source-relink.yml'
        self.assertTrue(module.is_file() and workflow.is_file(),
                        'integrated private source-delivery gate is absent')
        gate = importlib.import_module('plink_source_relink')
        bundles=importlib.import_module('plink_source_bundle')
        self.assertIn('private-green-3',bundles.BUNDLES,'new source authority has no separate delivery selector')
        self.assertTrue(hasattr(gate, 'compare_scientific'), 'base/modified exact scientific comparison is absent')
        text = workflow.read_text()
        self.assertNotIn('continue-on-error', text)
        self.assertNotRegex(text.split('permissions:', 1)[0], r'\n  (push|pull_request|schedule):')
        for name, step in steps(text).items():
            if '        run: |\n' in step:
                body = re.sub(r'\$\{\{.*?\}\}', 'preparation-only', bash_body(step))
                result = subprocess.run(['bash', '-n'], input=body, text=True, capture_output=True)
                self.assertEqual(result.returncode, 0, name + result.stderr)
        with tempfile.TemporaryDirectory(prefix='plink-source-delivery-preparation-') as directory:
            root = Path(directory)
            left, right = root / 'left', root / 'right'
            left.mkdir(); right.mkdir()
            for folder in (left, right):
                (folder / 'counts.tsv').write_text('ID\tN\nx\t2\n')
            compared = gate.compare_scientific(left, right, ['counts.tsv'])
            self.assertEqual(compared['file_count'], 1)
            (right / 'counts.tsv').write_text('ID\tN\nx\t3\n')
            with self.assertRaises(ValueError): gate.compare_scientific(left, right, ['counts.tsv'])
            original = root / 'original'
            model = gate.unpack_bundle(ROOT, 'private-green-3', original)
            self.assertEqual(model['scope'], 'private-green-3; not public-install qualified')
            self.assertEqual(model['snapshot'], 'green-3')
            self.assertEqual(len(model['dependencies']), 85)
            self.assertEqual(len(model['owner_files']), 303)
            self.assertIn('src/numeric.rs',model['owner_files'])
            self.assertIn('private-green-3',(original/'delivery/CHANGES.md').read_text())
            binding=dict(reports=dict.fromkeys(gate.native.report_paths('green-3')),
                generations=[dict(files={f'{number}.{extension}':{} for extension in ('pgen','pvar','psam','raw.json')}) for number in range(36)])
            self.assertEqual(len(gate.scientific_paths(binding,'green-3')),340)
            with self.assertRaises(ValueError):gate.scientific_paths(binding,'green-2')
            self.assertEqual((original / 'code/Cargo.lock').read_bytes(),
                             (original / 'authority/lock.copy').read_bytes())
            self.assertIn('directory = "../dependencies"',
                          (original / 'code/.cargo/config.toml').read_text())
            self.assertEqual(gate.verify_original(original), model)
            for row in model['dependencies']:
                self.assertTrue((original / 'dependencies' / row['directory'] / '.cargo-checksum.json').is_file())
            derivative = root / 'derivative'
            proof = gate.replace_codec(original, derivative)
            self.assertEqual(proof['changed_paths'], ['code/vendor/pgenlib/include/pgenlib_read.cc'])
            before = (original / proof['changed_paths'][0]).read_bytes()
            after = (derivative / proof['changed_paths'][0]).read_bytes()
            self.assertNotIn(gate.MARKER, before)
            self.assertEqual(after.count(gate.MARKER), 1)
            self.assertIn(b'PgrInit(', after)
            self.assertEqual(gate.verify_original(original), model)
            with self.assertRaises(ValueError): gate.verify_original(derivative)
            gate.verify_derivative(derivative, model, proof)
            (derivative / 'code/Cargo.lock').write_text('unrelated mutation\n')
            with self.assertRaises(ValueError): gate.verify_derivative(derivative, model, proof)
            for selector in ('green-1', '../private-green-1', 'private-green-0'):
                with self.assertRaises((ValueError, OSError)):
                    gate.unpack_bundle(ROOT, selector, root / 'invalid')
            blocked = subprocess.run([sys.executable, '-B', str(module), 'run',
                '--bundle', 'private-green-1', '--work', str(root / 'blocked'),
                '--evidence', str(root / 'evidence')],
                env=dict(os.environ, GITHUB_ACTIONS='false'), text=True, capture_output=True)
            self.assertNotEqual(blocked.returncode, 0)
            self.assertIn('hosted Linux', blocked.stderr)
            self.assertFalse((root / 'blocked').exists())
            self.assertFalse((root / 'evidence').exists())


if __name__ == '__main__':
    unittest.main()
