import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
import zipfile


class RemotePrivateRecoveryTest(unittest.TestCase):
    def test_complete_stream_binding_and_failed_authentication_preserve_originals(self):
        self.assertIsNotNone(importlib.util.find_spec('remote_private_recovery'),
                             'the connected remote recovery controller is missing')
        import remote_private_recovery as recovery

        with tempfile.TemporaryDirectory(prefix='remote-recovery-contract-') as temporary:
            root = Path(temporary)
            remote = root / 'remote'
            remote.mkdir()
            plaintext = b'not-a-parsed-TAR\n' + b'0123456789abcdef' * 65536
            ciphertext = b'synthetic-age\n' + plaintext
            receipt = dict(schema='rsomics.private-artifact.v1', status='sealed',
                context=dict(repository='omics-rust/rsomics-world', product='rsomics-ecology',
                    run_id=123, run_attempt=1, head_sha='a' * 40,
                    workflow_path='.github/workflows/ecology-beta-real.yml', phase='beta-braycurtis'),
                ciphertext=dict(bytes=len(ciphertext), sha256=hashlib.sha256(ciphertext).hexdigest()),
                recipient_sha256='b' * 64, age_version='v1.3.2',
                age_archive_sha256='c' * 64, producer_exit=0, age_exit=0)
            carrier = remote / 'original.zip'
            with zipfile.ZipFile(carrier, 'x') as archive:
                archive.writestr('payload.tar.age', ciphertext)
                archive.writestr('receipt.json', json.dumps(receipt))
            original = carrier.read_bytes()
            request = dict(host='4090', remote_python='python3', remote_root=str(remote),
                zip_path=str(carrier), attempt='success',
                zip=dict(bytes=len(original), sha256=hashlib.sha256(original).hexdigest()),
                context=receipt['context'], recipient_sha256=receipt['recipient_sha256'],
                age_version='v1.3.2', producer_age_archive_sha256='c' * 64,
                tar_bound_bytes=2 * 2**20, retained_bound_bytes=2 * 2**20,
                minimum_free_bytes=8 * 2**20, free_floor_bytes=2 * 2**20,
                local_headroom_bytes=2 * 2**20, timeout_seconds=20,
                status_timeout_seconds=5, log_limit_bytes=2**20)

            def local_remote(specification, action):
                return [sys.executable, '-B', '-c', recovery.REMOTE_PROGRAM, action,
                        json.dumps(specification)]

            def synthetic_age(exit_code):
                return [sys.executable, '-B', '-c',
                    'import sys; data=sys.stdin.buffer.read(); '
                    'assert data.startswith(b"synthetic-age\\n"); '
                    'sys.stdout.buffer.write(data[14:]); sys.stdout.buffer.flush(); '
                    f'sys.exit({exit_code})']

            identity_path = root / 'identity-must-not-be-created'
            with patch.object(recovery, '_remote_command', side_effect=local_remote), \
                 patch.object(recovery, '_age_command', return_value=synthetic_age(0)):
                result = recovery.recover(request, root / 'local-success', root / 'unused-tool', identity_path)
            self.assertTrue(result['complete_input_authenticated'])
            self.assertFalse(result['archive_validated'])
            self.assertEqual(result['tar'], dict(bytes=len(plaintext), sha256=hashlib.sha256(plaintext).hexdigest()))
            self.assertEqual((remote / 'success/candidate.tar').read_bytes(), plaintext)
            self.assertTrue((remote / 'success/AUTHENTICATED.json').is_file())
            self.assertEqual(result['pipeline_returncodes'], dict(source=0, age=0, sink=0))

            failed = dict(request, attempt='age-failed')
            with patch.object(recovery, '_remote_command', side_effect=local_remote), \
                 patch.object(recovery, '_age_command', return_value=synthetic_age(7)):
                with self.assertRaises(recovery.RecoveryError):
                    recovery.recover(failed, root / 'local-age-failed', root / 'unused-tool', identity_path)
            status = json.loads((root / 'local-age-failed/status.json').read_bytes())
            self.assertFalse(status['complete_input_authenticated'])
            self.assertEqual(status['pipeline_returncodes']['age'], 7)
            self.assertFalse((remote / 'age-failed/AUTHENTICATED.json').exists())
            self.assertTrue((remote / 'age-failed/candidate.tar').exists())
            self.assertTrue((root / 'local-age-failed/age.stderr.txt').is_file())

            bounded = dict(request, attempt='sink-bounded', tar_bound_bytes=1024)
            with patch.object(recovery, '_remote_command', side_effect=local_remote), \
                 patch.object(recovery, '_age_command', return_value=synthetic_age(0)):
                with self.assertRaises(recovery.RecoveryError):
                    recovery.recover(bounded, root / 'local-sink-bounded', root / 'unused-tool', identity_path)
            bounded_status = json.loads((root / 'local-sink-bounded/status.json').read_bytes())
            self.assertFalse(bounded_status['complete_input_authenticated'])
            self.assertFalse((remote / 'sink-bounded/AUTHENTICATED.json').exists())
            self.assertLessEqual((remote / 'sink-bounded/candidate.tar').stat().st_size, 1024)

            def failing_source(specification, action):
                if action == 'source':
                    return [sys.executable, '-B', '-c',
                            'import sys; sys.stdout.buffer.write(b"partial"); sys.exit(9)']
                return local_remote(specification, action)

            source_failed = dict(request, attempt='source-failed')
            with patch.object(recovery, '_remote_command', side_effect=failing_source), \
                 patch.object(recovery, '_age_command', return_value=synthetic_age(0)):
                with self.assertRaises(recovery.RecoveryError):
                    recovery.recover(source_failed, root / 'local-source-failed', root / 'unused-tool', identity_path)
            source_status = json.loads((root / 'local-source-failed/status.json').read_bytes())
            self.assertEqual(source_status['pipeline_returncodes']['source'], 9)
            self.assertFalse((remote / 'source-failed/AUTHENTICATED.json').exists())
            self.assertEqual(carrier.read_bytes(), original)
            self.assertFalse(identity_path.exists())


if __name__ == '__main__':
    unittest.main()
