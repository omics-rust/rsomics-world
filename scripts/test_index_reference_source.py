import hashlib
import importlib
import io
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))


class IndexReferenceSourcePreparationTest(unittest.TestCase):
    def test_connected_source_preparation_retains_inputs_and_rejects_changed_acquisition(self):
        self.assertTrue((ROOT / 'scripts/index_reference_source.py').is_file(),
                        'manual index reference source controller is absent')
        source = importlib.import_module('index_reference_source')
        with tempfile.TemporaryDirectory(prefix='index-reference-preparation-', dir=os.environ['TMPDIR']) as directory:
            root = Path(directory)
            inputs = source.prepare_inputs(root / 'inputs')
            self.assertEqual(hashlib.sha256(inputs['fasta-donor'].read_bytes()).hexdigest(),
                             'f6aa242815f8cd80990041fcd2fff32cda7a5a76fb1da5e50622f54336a41331')
            self.assertEqual(hashlib.sha256(inputs['dict-donor'].read_bytes()).hexdigest(),
                             '35bba8db8b245b3443f7c00c6c2eaebf17da1fef60ba191720d5cf35ad934e3f')
            body = inputs['workflow'].read_bytes()
            self.assertGreater(len(body), 262177)
            self.assertIn(b'>crlf description\r\n', body)
            self.assertFalse(body.endswith(b'\n'))
            self.assertIn(b'>colon:2-7\n', body)
            metadata = source.load(root / 'inputs/specification.json')
            self.assertEqual(metadata['records']['long']['bases'], 262177)
            self.assertEqual(metadata['records']['long']['line_bases'], 101)
            self.assertEqual(metadata['records']['crlf']['bases'], 127)
            self.assertEqual(metadata['records']['last']['bases'], 83)
            self.assertEqual(metadata['records']['last']['line_bases'], 83)
            jobs = source.query_jobs(Path('/oracle/samtools'), inputs['workflow'], inputs['regions'], root / 'queries')
            self.assertEqual([job['name'] for job in jobs], ['default', 'unwrapped', 'four-workers'])
            self.assertEqual(jobs[0]['argv'][-3:], [inputs['workflow'], 'long:1-350', 'crlf'])
            self.assertEqual(jobs[1]['argv'][jobs[1]['argv'].index('-n') + 1], '0')
            self.assertEqual(jobs[2]['argv'][jobs[2]['argv'].index('-@') + 1], '3')
            self.assertIn('{colon}:2-7', inputs['regions'].read_text().splitlines())
            self.assertIn('{colon:2-7}', inputs['regions'].read_text().splitlines())
            payload = root / 'release.tar.bz2'
            with tarfile.open(payload, 'w:bz2') as archive:
                member = tarfile.TarInfo('release-1/README')
                member.size = 5
                archive.addfile(member, io.BytesIO(b'body\n'))
            spec = dict(size=payload.stat().st_size, sha256=hashlib.sha256(payload.read_bytes()).hexdigest(),
                        prefix='release-1', unpacked_limit=1024)
            source.verify_body(payload, spec)
            extracted = source.extract_release(payload, root / 'extracted', spec)
            self.assertEqual((extracted / 'README').read_bytes(), b'body\n')
            payload.write_bytes(payload.read_bytes() + b'changed')
            with self.assertRaisesRegex(ValueError, 'identity'):
                source.verify_body(payload, spec)
            workflow = ROOT / '.github/workflows/index-reference-source.yml'
            source.validate_workflow(workflow)
            retained = root / 'retained'
            retained.mkdir()
            executable = retained / 'program'
            executable.write_bytes(b'synthetic runtime body\n')
            executable.chmod(0o755)
            (retained / 'library-link').symlink_to('program')
            source.package(retained, root / 'retained.tar')
            with tarfile.open(root / 'retained.tar') as archive:
                self.assertEqual(archive.getmember('index-reference-source/program').mode, 0o755)
                self.assertTrue(archive.getmember('index-reference-source/library-link').issym())
                self.assertEqual(archive.extractfile('index-reference-source/program').read(),
                                 executable.read_bytes())
            self.assertIn('${{ env.INDEX_CARRIER }}', workflow.read_text())
            altered = root / 'automatic.yml'
            altered.write_text(workflow.read_text().replace('workflow_dispatch:', 'push:'))
            with self.assertRaisesRegex(ValueError, 'manual'):
                source.validate_workflow(altered)
            for match in source.shell_blocks(workflow):
                checked = subprocess.run(['bash', '-n'], input=match, text=True, capture_output=True)
                self.assertEqual(checked.returncode, 0, checked.stderr)
        self.assertFalse({'numpy', 'allel', 'pysam'}.intersection(sys.modules))
        with self.assertRaisesRegex(RuntimeError, 'hosted'):
            source.hosted_root()


if __name__ == '__main__':
    unittest.main()
