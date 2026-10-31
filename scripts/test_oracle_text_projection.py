import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import tarfile
import tempfile
import unittest
import zipfile


SCRIPT = Path(__file__).with_name('oracle_text_projection.py')


class OracleTextProjectionTest(unittest.TestCase):
    def test_original_bodies_are_verified_before_bounded_text_is_admitted(self):
        self.assertTrue(SCRIPT.is_file(), 'bounded original-archive text projector is absent')
        spec = importlib.util.spec_from_file_location('oracle_text_projection', SCRIPT)
        projection = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(projection)
        root = Path(tempfile.mkdtemp(prefix='oracle-projection-preparation-', dir=os.environ['TMPDIR']))
        bodies = {
            'failure.json': b'{"message":"unresolved ELF dependency: Biobase.so"}\n',
            'host.json': b'{"runner":"/__w/_temp"}\n',
            'runtime-check/ldd/2/process.json': b'{"argv":["ldd","Biobase.so"],"returncode":0}\n',
            'runtime-check/ldd/2/stdout.txt': b'libR.so => not found\nlibc.so.6 => /lib/libc.so.6\n',
            'runtime-check/ldd/2/stderr.txt': b'',
            'runtime-check/R-state/namespaces.tsv': b'namespace\tversion\nBiobase\t2.71.0\n',
            'runtime-check/R-state/DLLs.tsv': b'name\tpath\nBiobase\t/original/Biobase.so\n',
            'runtime-check/R-state/session.txt': b'R version pinned fixture\n',
            'runtime-check/R-state/object.rds': b'never deserialize this',
            'restored/R-library/pkg/A': b'private ELF body',
            'restored/R-library/pkg/a': b'casefold collision remains virtual',
            'integrity-after.json': b'x' * (2 * 1024 * 1024 + 1),
        }
        links = {'restored/R-library/pkg/alias.so': 'A',
                 'restored/R-library/pkg/external.so': '/original/lib/libR.so'}
        identities = {name: {'size': len(body), 'sha256': hashlib.sha256(body).hexdigest()}
                      for name, body in bodies.items()}
        identities.update({name: {'symlink': target} for name, target in links.items()})
        manifest = json.dumps({'files': identities}, sort_keys=True).encode()

        def carrier(name, changed=False, unsafe=False):
            data = io.BytesIO()
            with tarfile.open(fileobj=data, mode='w') as archive:
                for path, body in bodies.items():
                    actual = b'corrupted private ELF body' if changed and path.endswith('/pkg/A') else body
                    member = tarfile.TarInfo('capture/' + path)
                    member.size = len(actual)
                    archive.addfile(member, io.BytesIO(actual))
                for path, target in links.items():
                    member = tarfile.TarInfo('capture/' + path)
                    member.type, member.linkname = tarfile.SYMTYPE, target
                    archive.addfile(member)
                if unsafe:
                    member = tarfile.TarInfo('capture/../../escape')
                    member.size = 1
                    archive.addfile(member, io.BytesIO(b'x'))
                member = tarfile.TarInfo('capture/artifact-manifest.json')
                member.size = len(manifest)
                archive.addfile(member, io.BytesIO(manifest))
            tar_body = data.getvalue()
            path = root / name
            with zipfile.ZipFile(path, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
                archive.writestr('__w/_temp/original.tar', tar_body)
                archive.writestr('__w/_temp/original.tar.json', json.dumps({
                    'sha256': hashlib.sha256(tar_body).hexdigest(), 'size': len(tar_body)}))
            return path, tar_body

        original, tar_body = carrier('original.zip')
        digest = hashlib.sha256(original.read_bytes()).hexdigest()
        delivery = root / 'delivery'
        proof = projection.project_archive(original, root / 'staging', delivery, 'capture/', digest,
                                           {'kind': 'local stdlib preparation fixture, not historical source'})
        wanted = sorted([
            'failure.json', 'host.json', 'runtime-check/ldd/2/process.json',
            'runtime-check/ldd/2/stdout.txt', 'runtime-check/ldd/2/stderr.txt',
            'runtime-check/R-state/namespaces.tsv', 'runtime-check/R-state/DLLs.tsv',
            'runtime-check/R-state/session.txt'])
        self.assertEqual([row['path'] for row in proof['projected']], wanted)
        self.assertEqual(proof['status'], 'verified text projection, not full original retention or source admission')
        self.assertEqual(proof['original_tar']['sha256'], hashlib.sha256(tar_body).hexdigest())
        self.assertEqual(proof['manifest']['sha256'], hashlib.sha256(manifest).hexdigest())
        self.assertEqual(proof['verified_regular_bodies'], len(bodies))
        self.assertEqual(proof['verified_symlinks'], 2)
        self.assertEqual((delivery / 'text/runtime-check/ldd/2/stdout.txt').read_bytes(),
                         b'libR.so => not found\nlibc.so.6 => /lib/libc.so.6\n')
        self.assertEqual(proof['omitted_text'][0]['path'], 'integrity-after.json')
        self.assertEqual(proof['omitted_text'][0]['reason'], 'exceeds 2MiB selected-text cap; original body fully verified')
        self.assertEqual(proof['casefold_collision_groups'], 1)
        self.assertFalse(any(path.is_symlink() for path in delivery.rglob('*')))
        self.assertFalse((delivery / 'text/runtime-check/R-state/object.rds').exists())
        self.assertFalse((delivery / 'text/restored').exists())
        self.assertEqual(json.loads((delivery / 'projection-proof.json').read_text()), proof)
        for name, changed, unsafe, message in [('changed', True, False, 'manifest identity'),
                                               ('traversal', False, True, 'unsafe archive path')]:
            with self.subTest(name=name):
                bad, _ = carrier(name + '.zip', changed=changed, unsafe=unsafe)
                output = root / name
                with self.assertRaisesRegex(ValueError, message):
                    projection.project_archive(bad, root / (name + '-staging'), output, 'capture/',
                                               hashlib.sha256(bad.read_bytes()).hexdigest(), {})
                self.assertFalse((output / 'projection-proof.json').exists())
        with self.assertRaisesRegex(ValueError, 'ZIP digest'):
            projection.project_archive(original, root / 'digest-staging', root / 'digest-output',
                                       'capture/', '0' * 64, {})
        print('retained preparation fixture:', root)


if __name__ == '__main__':
    unittest.main()
