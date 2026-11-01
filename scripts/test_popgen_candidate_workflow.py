"""One stdlib preparation contract; no native or upstream scientific execution."""

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
WORKFLOW = ROOT / '.github/workflows/popgen-candidate.yml'


class PopgenCandidateTests(unittest.TestCase):
    def test_integrated_native_four_snapshot_and_original_retention(self):
        self.assertTrue(WORKFLOW.is_file(), 'missing private Popgen native-four workflow')
        candidate = importlib.import_module('popgen_candidate')
        text = WORKFLOW.read_text()
        routes = json.loads(re.search(r"fromJSON\('([^']+)'\)", text)[1])
        self.assertEqual([(r['target'], r['runner'], r['host']) for r in routes], [
            ('linux-x86_64', 'ubuntu-24.04', 'x86_64-unknown-linux-gnu'),
            ('linux-aarch64', 'ubuntu-24.04-arm', 'aarch64-unknown-linux-gnu'),
            ('macos-x86_64', 'macos-15-intel', 'x86_64-apple-darwin'),
            ('macos-aarch64', 'macos-15', 'aarch64-apple-darwin')])
        self.assertNotRegex(text.split('permissions:', 1)[0], r'\n  (push|pull_request|schedule):')
        self.assertNotIn('continue-on-error', text)
        self.assertIn('dtolnay/rust-toolchain@1.91.0', text)
        self.assertNotIn('source-on-native', text)
        routed = steps(text)
        names = list(routed)
        order = ['Prepare native storage', 'Verify frozen source', 'Install native Rust 1.91',
                 'Admit compiler and locked dependencies', 'Formatting and strict Clippy',
                 'Full debug suite', 'Full release suite', 'Retain actual release executable',
                 'Verify unchanged source', 'Package complete originals', 'Upload complete originals']
        self.assertEqual(names, order)
        for name, step in routed.items():
            if '        run: |\n' in step:
                body = re.sub(r'\$\{\{.*?\}\}', 'route-only', bash_body(step))
                checked = subprocess.run(['bash', '-n'], input=body, text=True, capture_output=True, timeout=15)
                self.assertEqual(checked.returncode, 0, name + checked.stderr)
        for name in order[-3:]:
            self.assertIn('if: always()', routed[name])
        for name in ('Formatting and strict Clippy', 'Full debug suite', 'Full release suite'):
            self.assertNotIn('matrix.target ==', routed[name])
        for mode in ('debug', 'release'):
            command = candidate.test_command(mode)
            self.assertTrue({'--locked', '--offline', '--all-targets', '--all-features', '--no-fail-fast'} <= set(command))
            self.assertEqual('--release' in command, mode == 'release')
            self.assertNotIn('--ignored', command)
        ast.parse((ROOT / 'scripts/popgen_candidate.py').read_text())
        with tempfile.TemporaryDirectory(prefix='popgen-candidate-') as directory:
            root = Path(directory)
            for selector in ('missing', '../green-1', 'green-1/x', 'green-0'):
                with self.assertRaises((ValueError, OSError)):
                    candidate.prepare_source(ROOT, root / 'absent', selector)
            output = root / 'candidate'
            authority = candidate.prepare_source(ROOT, output, 'green-1')
            self.assertEqual(authority['owner_commit'], 'ff8e951344b5cd8f8c049c96afa9e2c585d14995')
            self.assertEqual(authority['source_run']['run_id'], 41252735560)
            self.assertEqual(len(authority['files']), 48)
            self.assertEqual(len(authority['goldens']), 17)
            self.assertEqual(len(authority['synthetic_inputs']), 5)
            self.assertEqual(authority['registry_packages'], 69)
            self.assertEqual(authority['policy']['canonical_abs_floor'], 2.0 ** -44)
            manifest = {name: row['sha256'] for name, row in authority['files'].items()}
            candidate.check_tree(output / 'source', manifest)
            fixture = output / 'source/tests/fixtures/neutral'
            golden_names = json.loads((fixture / 'scientific-files.json').read_bytes())
            self.assertEqual(set(golden_names), set(authority['goldens']))
            self.assertEqual(set(golden_names), {*(f'{c}/{n}' for c in ('a', 'b') for n in (
                'sites.tsv.gz', 'windows.tsv.gz', 'region.json', 'sfs.json', 'fst-jackknife.json',
                'jackknife-membership.tsv.gz')), *(f'vcftools/{n}' for n in (
                'site-pi.sites.pi', 'window-pi.windowed.pi', 'tajima-d.Tajima.D',
                'weir-fst.weir.fst', 'window-weir-fst.windowed.weir.fst'))})
            changed = output / 'source/Cargo.lock'
            original = changed.read_bytes()
            changed.write_bytes(original + b'\nchanged\n')
            controller = root / 'controller'
            controller.mkdir()
            (controller / 'original.py').write_bytes(b'controller-original\n')
            candidate.save(controller / 'checksums.json', candidate.tree_manifest(controller))
            previous_evidence = os.environ.get('EVIDENCE')
            os.environ['EVIDENCE'] = str(root)
            try:
                with self.assertRaises(ValueError):
                    candidate.finalize(output)
            finally:
                if previous_evidence is None:
                    del os.environ['EVIDENCE']
                else:
                    os.environ['EVIDENCE'] = previous_evidence
            self.assertTrue(json.loads((output / 'integrity-after.json').read_bytes())['failures'])
            changed.write_bytes(original)
            candidate.check_tree(output / 'source', manifest)
            with self.assertRaises(RuntimeError):
                candidate.capture(output / 'commands', 'failure', [os.sys.executable, '-B', '-c',
                    'import sys; print("original-seven"); print("stderr-seven", file=sys.stderr); sys.exit(7)'])
            failed = output / 'commands/failure'
            self.assertEqual(json.loads((failed / 'status.json').read_bytes())['status'], 7)
            self.assertIn(b'original-seven', (failed / 'stdout.txt').read_bytes())
            self.assertIn(b'stderr-seven', (failed / 'stderr.txt').read_bytes())
            archive_path = root / 'originals.tar'
            candidate.package(output, archive_path)
            with tarfile.open(archive_path) as archive:
                names = archive.getnames()
                self.assertIn('candidate/commands/failure/status.json', names)
                self.assertIn('candidate/source/tests/neutral_workflow.rs', names)
                self.assertTrue(all(m.isfile() for m in archive.getmembers()))
                record = json.load(archive.extractfile('candidate/artifact-manifest.json'))
                self.assertEqual(record['schema'], 'rsomics.popgen.native-four.v1')
                self.assertEqual(record['files']['commands/failure/stdout.txt']['sha256'],
                                 candidate._hash_file(failed / 'stdout.txt'))
                self.assertFalse(record['performance_claim'])
                self.assertFalse(record['source_on_native_qualified'])


if __name__ == '__main__':
    unittest.main()
