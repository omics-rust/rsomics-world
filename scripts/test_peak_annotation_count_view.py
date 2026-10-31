import csv
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / 'scripts/peak_annotation_count_view.py'


class PeakViewPreparationTest(unittest.TestCase):
    def test_restored_originals_view_routes_and_failed_observations_are_not_admission(self):
        self.assertTrue(SCRIPT.is_file(), 'adopted source-view producer is absent')
        spec = importlib.util.spec_from_file_location('peak_view', SCRIPT)
        view = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(view)
        root = Path(tempfile.mkdtemp(prefix='peak-view-preparation-', dir=os.environ['TMPDIR']))
        inputs = root / 'inputs'
        view.make_inputs(inputs)
        annotation = list(csv.reader((inputs / 'annotation.bed').read_text().splitlines(), delimiter='\t'))
        self.assertEqual(len(annotation), 10)
        self.assertEqual(annotation[0], ['chrA', '3980002', '3980003', 'plus-promoter', '0', '+'])
        self.assertEqual(annotation[6], ['chrA', '3922170', '3922171', 'minus-5UTR', '0', '-'])
        counts = list(csv.reader((inputs / 'count.bed').read_text().splitlines(), delimiter='\t'))
        self.assertEqual(len(counts), 16)
        self.assertEqual(counts[9], counts[10])
        self.assertEqual(counts[12][1:3], ['8702440', '8702450'])
        rows = list(csv.DictReader((inputs / 'count-rows.tsv').read_text().splitlines(), delimiter='\t'))
        self.assertEqual(len({x['row_key'] for x in rows}), 16)
        plan = view.case_plan(root / 'restored', inputs, root / 'calls')
        view.save(root / 'execution-plan.json', plan)
        self.assertEqual(json.loads((root / 'execution-plan.json').read_text()), plan)
        self.assertEqual([x['family'] for x in plan].count('annotate'), 3)
        self.assertEqual([x['family'] for x in plan].count('count'), 5)
        self.assertEqual([x['family'] for x in plan].count('reload'), 1)
        jobs = {x['name']: x for x in plan}
        self.assertIn('-q', jobs['C-q20']['argv'])
        self.assertNotIn('-q', jobs['C-split']['argv'])
        self.assertNotIn('-q', jobs['C-include']['argv'])
        self.assertIn('-D', jobs['C-include']['argv'])
        self.assertIn('-F', jobs['C-include']['argv'])
        self.assertEqual(jobs['reload']['deps'], ['A-gtf-narrow', 'A-gff3-narrow', 'A-gtf-default'])
        old_body, new_body = root / 'old-function.R', root / 'new-function.R'
        old_body.write_text('function(x) x + 1\n<bytecode: 0x123>\n<environment: namespace:ChIPseeker>\n')
        new_body.write_text('function(x) x + 1\n<bytecode: 0x456>\n<environment: namespace:ChIPseeker>\n')
        view.check_namespace_body(old_body, new_body)
        new_body.write_text('function(x) x + 2\n<bytecode: 0x456>\n<environment: namespace:ChIPseeker>\n')
        with self.assertRaisesRegex(ValueError, 'namespace body differs'):
            view.check_namespace_body(old_body, new_body)
        self.assertIsNone(view.dll_file({'name': 'base', 'path': 'base'}, root / 'restored'))
        with self.assertRaises(ValueError):
            view.dll_file({'name': 'unbound', 'path': 'base'}, root / 'restored')
        payload = {'R-library/pkg/a': b'original library', 'inputs/a.bam': b'original BAM'}
        manifest = {name: dict(size=len(data), sha256=hashlib.sha256(data).hexdigest())
                    for name, data in payload.items()}
        manifest['R-library/pkg/alias'] = {'symlink': 'a'}
        archive = root / 'small-original.tar'

        def write_archive(extra=None, changed=False):
            with tarfile.open(archive, 'w') as out:
                entries = dict(payload)
                entries['artifact-manifest.json'] = json.dumps({'files': manifest}).encode()
                if extra:
                    entries[extra] = b'escape'
                if changed:
                    entries['inputs/a.bam'] = b'changed BAM'
                for name, data in entries.items():
                    entry = tarfile.TarInfo('peak-first-workflow-oracle/' + name)
                    entry.size = len(data)
                    out.addfile(entry, io.BytesIO(data))
                entry = tarfile.TarInfo('peak-first-workflow-oracle/R-library/pkg/alias')
                entry.type = tarfile.SYMTYPE
                entry.linkname = 'a'
                out.addfile(entry)

        write_archive()
        restored = root / 'restore-check'
        result = view.verify_restore(archive, restored, lambda name: name.startswith('R-library/'))
        self.assertEqual(result['regular_verified'], 2)
        self.assertEqual((restored / 'R-library/pkg/a').read_bytes(), b'original library')
        self.assertFalse((restored / 'inputs/a.bam').exists())
        self.assertTrue((restored / 'R-library/pkg/alias').is_symlink())
        self.assertEqual((restored / 'R-library/pkg/alias').read_bytes(), b'original library')
        for extra in ['../../escape', '/absolute', 'R-library/pkg/../escape']:
            write_archive(extra=extra)
            with self.assertRaises(ValueError):
                view.verify_restore(archive, root / ('bad-' + str(len(extra))), lambda name: True)
        write_archive(changed=True)
        with self.assertRaisesRegex(ValueError, 'identity'):
            view.verify_restore(archive, root / 'changed', lambda name: True)
        failed = view.original.capture(root / 'failed-call',
            [sys.executable, '-B', '-c', 'import sys; print("raw"); print("diagnostic",file=sys.stderr); sys.exit(7)'])
        self.assertEqual(failed['returncode'], 7)
        self.assertEqual((root / 'failed-call/stderr.txt').read_text(), 'diagnostic\n')
        self.assertEqual((root / 'failed-call/stdout.txt').read_text(), 'raw\n')
        for argv in [
            ['--action', 'manifest', '--output', str(root / 'failed-call')],
            ['--action', 'package', '--output', str(root / 'failed-call'),
             '--archive', str(root / 'failed-original.tar')],
        ]:
            result = subprocess.run([sys.executable, '-B', str(SCRIPT), *argv], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
        with tarfile.open(root / 'failed-original.tar') as retained:
            self.assertEqual(retained.extractfile('failed-call/stderr.txt').read(), b'diagnostic\n')
            self.assertEqual(json.load(retained.extractfile('failed-call/process.json'))['returncode'], 7)
        self.assertEqual(json.loads((root / 'failed-original.tar.json').read_text())['sha256'],
                         hashlib.sha256((root / 'failed-original.tar').read_bytes()).hexdigest())
        count_values = [1, 1, 1, 0, 0, 1, 2, 2, 1, 1, 1, 1, 1, 1, 0, 2]
        matrices = {name: [[v, v] for v in count_values]
                    for name in ['C-default', 'C-q20', 'C-proper', 'C-include', 'C-split']}
        for index in [0, 8, 11, 12, 13, 15]:
            matrices['C-q20'][index] = [0, 0]
        matrices['C-proper'][5] = [0, 0]
        matrices['C-include'][3] = [1, 1]
        matrices['C-include'][4] = [1, 1]
        matrices['C-split'][12] = [0, 0]
        view.check_count_effects(matrices)
        matrices['C-split'][12] = [1, 1]
        with self.assertRaisesRegex(ValueError, 'N-gap'):
            view.check_count_effects(matrices)
        local = subprocess.run([sys.executable, '-B', str(SCRIPT), '--action', 'run',
                                '--output', str(root / 'not-hosted')], capture_output=True, text=True,
                               env=dict(os.environ, GITHUB_ACTIONS='false'))
        self.assertNotEqual(local.returncode, 0)
        self.assertIn('hosted', (root / 'not-hosted/failure.json').read_text())
        self.assertFalse((root / 'not-hosted/qualification.json').exists())
        (root / 'runner').mkdir()
        escaped = subprocess.run([sys.executable, '-B', str(SCRIPT), '--action', 'run',
                                  '--output', str(root / 'escaped-output')], capture_output=True, text=True,
                                 env=dict(os.environ, GITHUB_ACTIONS='true', RUNNER_TEMP=str(root / 'runner')))
        self.assertNotEqual(escaped.returncode, 0)
        self.assertFalse((root / 'escaped-output').exists())
        workflow = (ROOT / '.github/workflows/peak-annotation-count-view.yml').read_text()
        bodies = []
        for chunk in workflow.split('run: |')[1:]:
            lines = []
            for line in chunk.splitlines()[1:]:
                if line and not line.startswith('          '):
                    break
                lines.append(line[10:])
            bodies.append('\n'.join(lines).replace('${{ github.token }}', 'not-a-token'))
        self.assertGreaterEqual(len(bodies), 3)
        for body in bodies:
            checked = subprocess.run(['bash', '-n'], input=body, capture_output=True, text=True)
            self.assertEqual(checked.returncode, 0, checked.stderr)
        self.assertFalse({'numpy', 'pandas', 'pysam', 'rpy2'}.intersection(sys.modules))
        print('retained stdlib preparation objects:', root)


if __name__ == '__main__':
    unittest.main()
