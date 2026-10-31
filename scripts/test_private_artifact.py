"""One real private-artifact delivery contract, using only synthetic data."""

import importlib
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


class PrivateArtifactTest(unittest.TestCase):
    def test_complete_authenticated_delivery_never_promotes_failed_or_partial_output(self):
        self.assertTrue((ROOT / 'scripts/private_artifact.py').is_file(),
                        'private artifact delivery is not implemented')
        private = importlib.import_module('private_artifact')
        tool = private.prepare_tool(private.current_platform(),
            Path(os.environ['TMPDIR']) / 'private-artifact-tools-v1.3.2' / private.current_platform())
        root = Path(tempfile.mkdtemp(prefix='private-artifact-synthetic-', dir=os.environ['TMPDIR']))
        identity, recipients = root / 'synthetic.agekey', root / 'recipient.txt'
        generated = subprocess.run([tool / 'age-keygen', '-pq', '-o', identity], capture_output=True)
        self.assertEqual(generated.returncode, 0, 'synthetic key generation failed')
        identity.chmod(0o600)
        public = subprocess.run([tool / 'age-keygen', '-y', identity], capture_output=True, check=True).stdout
        self.assertTrue(public.startswith(b'age1pq1'))
        recipients.write_bytes(public)
        source = root / 'synthetic.tar'
        with tarfile.open(source, 'w') as archive:
            body = b'only synthetic values\nx\t7\n'
            entry = tarfile.TarInfo('synthetic.tsv'); entry.size = len(body)
            archive.addfile(entry, io.BytesIO(body))
        original = source.read_bytes()
        context = dict(repository='omics-rust/rsomics-world', product='rsomics-popgen',
                       run_id=1, run_attempt=1, head_sha='a' * 40, phase='qualification')
        ciphertext = root / 'synthetic.tar.age'
        receipt = private.encrypt(tool, recipients, ciphertext, input_file=source, context=context)
        self.assertEqual(receipt['status'], 'sealed')
        self.assertEqual(receipt['context'], context)
        self.assertEqual(receipt['producer_exit'], None)
        self.assertEqual(receipt['age_exit'], 0)
        self.assertEqual(receipt['ciphertext']['bytes'], ciphertext.stat().st_size)
        self.assertEqual(set(receipt), {'schema', 'status', 'context', 'ciphertext',
            'recipient_sha256', 'age_version', 'age_archive_sha256', 'producer_exit', 'age_exit'})
        self.assertNotIn('synthetic.tsv', json.dumps(receipt))
        self.assertNotIn('only synthetic values', json.dumps(receipt))
        plaintext = root / 'recovered.tar'
        self.assertEqual(private.decrypt(tool, identity, ciphertext, plaintext), plaintext)
        self.assertEqual(plaintext.read_bytes(), original)
        self.assertEqual(plaintext.stat().st_mode & 0o777, 0o600)
        with self.assertRaises((ValueError, FileExistsError)):
            private.encrypt(tool, recipients, ciphertext, input_file=source, context=context)
        streamed = root / 'stream.tar.age'
        command = [sys.executable, '-B', '-c',
                   'import pathlib,sys;sys.stdout.buffer.write(pathlib.Path(sys.argv[1]).read_bytes())', str(source)]
        stream_receipt = private.encrypt(tool, recipients, streamed, producer=command,
                                         plaintext_limit=len(original), context=context)
        self.assertEqual(stream_receipt['producer_exit'], 0)
        private.decrypt(tool, identity, streamed, root / 'stream-recovered.tar')
        self.assertEqual((root / 'stream-recovered.tar').read_bytes(), original)
        failed = root / 'failed.tar.age'
        with self.assertRaises(private.ArtifactError):
            private.encrypt(tool, recipients, failed, producer=[sys.executable, '-B', '-c',
                'import sys;sys.stdout.buffer.write(b"partial synthetic");sys.exit(7)'],
                plaintext_limit=1024, context=context)
        self.assertFalse(failed.exists())
        self.assertTrue(Path(str(failed) + '.partial').exists())
        damaged = root / 'damaged.age'; damaged.write_bytes(ciphertext.read_bytes()[:-1])
        with self.assertRaises(private.ArtifactError):
            private.decrypt(tool, identity, damaged, root / 'damaged.tar')
        self.assertFalse((root / 'damaged.tar').exists())
        with self.assertRaises(ValueError):
            private.encrypt(tool, recipients, root / 'forbidden.age', input_file=source,
                            context=dict(context, sample='must-not-be-public'))
        space = private.finalization_space(1000, 1024)
        self.assertEqual(space['ciphertext_bound_bytes'], 2098208)
        self.assertEqual(space['required_free_bytes'], 8592032800)
        self.assertEqual(space['peak_project_bytes'], 8592033800)
        self.assertEqual(private.finalization_space(1000, 1024, materialize_tar=True)['required_free_bytes'],
                         8592033824)
        self.assertEqual(source.read_bytes(), original)
        self.assertFalse(Path(str(ciphertext) + '.partial').exists())
        self.assertFalse(Path(str(plaintext) + '.partial').exists())
        self.assertTrue((tool / 'admission.json').is_file())
        print('retained synthetic private-artifact preparation:', root)


if __name__ == '__main__':
    unittest.main()
