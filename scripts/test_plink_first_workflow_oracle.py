"""One integrated stdlib-only preparation contract; no scientific execution."""

import copy
import importlib.util
import hashlib
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


def load(name):
    path = ROOT / 'scripts' / (name + '.py')
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


class PlinkWholeProducerPreparationTest(unittest.TestCase):
    def test_shared_fileset_truth_routes_provenance_failure_and_reload(self):
        self.assertTrue((ROOT / 'scripts/plink_first_workflow_oracle.py').is_file(),
                        'complete hosted whole-six source producer is absent')
        inputs = load('plink_first_workflow_inputs')
        oracle = load('plink_first_workflow_oracle')
        self.assertFalse({'numpy', 'pandas', 'pgenlib', 'scipy'}.intersection(sys.modules))
        lock = json.loads((ROOT / 'assets/plink-first-runtime-20261031.json').read_text())
        oracle.validate_spec(lock)
        with tempfile.TemporaryDirectory(prefix='plink-preparation-', dir=os.environ['TMPDIR']) as directory:
            root = Path(directory)
            truth = inputs.generate(root / 'inputs')
            check = inputs.check_truth(truth)
            self.assertEqual(check['samples'], 72)
            self.assertEqual(check['founders'], 64)
            self.assertEqual(check['baseline_variants'], 64)
            self.assertEqual(check['rich_variants'], 24)
            self.assertEqual(check['sentinel_genotypes'], [16, 32, 16])
            self.assertEqual(check['founder_ld'], {'x_y': '0', 'x_z': '1/4', 'x_copy': '1'})
            hds_prefix = root / 'hds-track-protocol/data'
            hds_prefix.parent.mkdir()
            Path(str(hds_prefix) + '.pvar').write_text('#CHROM\tPOS\tID\tREF\tALT\n1\t100\td00\tA\tG\n')
            Path(str(hds_prefix) + '.psam').write_text('#IID\nS0004\nS0006\nS0007\nS0013\n')
            hds_raw = [dict(samples=4, variants=1), dict(variant=0, calls=[
                [-9, -9, 0, 0, 8192, 8192],
                [1, 0, 1, 1, 14746, None],
                [1, 0, 1, 1, None, None],
                [-9, -9, 0, 0, None, None]])]
            self.assertEqual(oracle.check_dosage_truth(truth, hds_prefix, hds_raw, 'hds'), 3)
            for track in ('explicit-dphase', 'implicit-hardcall-phase'):
                dropped_phase = copy.deepcopy(hds_raw)
                if track == 'explicit-dphase':
                    dropped_phase[1]['calls'][0][5] = None
                else:
                    dropped_phase[1]['calls'][2][:4] = [0, 1, 0, 0]
                with self.subTest(HDS_track=track), self.assertRaisesRegex(ValueError, 'HDS.*phase'):
                    oracle.check_dosage_truth(truth, hds_prefix, dropped_phase, 'hds')
            samples_mask, variant_mask = oracle.expected_filtered_baseline(truth)
            self.assertEqual(samples_mask, list(range(63)) + list(range(64, 70)))
            self.assertNotIn('v10', variant_mask)
            self.assertNotIn('v12', variant_mask)
            self.assertIn('v14', variant_mask)
            self.assertNotIn('v15', variant_mask)
            with self.assertRaisesRegex(ValueError, 'rounding interval'):
                oracle.printed_close('1e-20', 3e-20, slack='0')
            protocol = root / 'bound-generation'
            protocol.mkdir()
            prefix = protocol / 'data'
            suffixes = ['.pgen', '.pvar', '.psam']
            for suffix in suffixes:
                Path(str(prefix) + suffix).write_text('stdlib metadata-hash protocol fixture; not a PGEN\n')
            (protocol / 'bound-files.json').write_text(json.dumps(dict(files={
                suffix: oracle.sha256(str(prefix) + suffix) for suffix in suffixes})))
            oracle.verify_generation_snapshot(prefix)
            Path(str(prefix) + '.psam').write_text('permuted declared names/order\n')
            with self.assertRaisesRegex(ValueError, 'manifest-bound generation'):
                oracle.verify_generation_snapshot(prefix)
            self.assertEqual((root / 'inputs/baseline.bed').stat().st_size, 1155)
            broken = copy.deepcopy(truth)
            broken['samples'][1]['iid'] = broken['samples'][0]['iid']
            with self.assertRaisesRegex(ValueError, 'identity'):
                inputs.check_truth(broken)
            broken = copy.deepcopy(truth)
            next(v for v in broken['baseline'] if v['id'] == 'x')['gt'][0] = 2
            with self.assertRaisesRegex(ValueError, 'sentinel'):
                inputs.check_truth(broken)
            manifest = json.loads((root / 'inputs/manifest.json').read_text())
            self.assertEqual(manifest['origin'], 'prepared synthetic engineering fixture; not oracle output')
            self.assertNotIn('expected_plink_outputs', manifest)
            plan = oracle.case_plan(Path('/qualified/plink2'), Path('/qualified/capture'),
                                    root / 'inputs', root / 'case', 4)
            jobs = {j['name']: j for j in plan}
            self.assertEqual(len(jobs), len(plan))
            for lane in ('gt', 'bed', 'pgen', 'ds', 'hds', 'rich'):
                for generation in ('F', 'P'):
                    self.assertIn(f'{lane}.{generation}.qc', jobs)
                    self.assertIn(f'{lane}.{generation}.decode', jobs)
                self.assertEqual(jobs[f'{lane}.F.qc']['deps'], [f'{lane}.F.make'])
                self.assertEqual(jobs[f'{lane}.P.make']['deps'], [f'{lane}.F.prune'])
                argv = jobs[f'{lane}.P.make']['argv']
                self.assertIn('--extract', argv)
                self.assertTrue(argv[argv.index('--extract') + 1].endswith('.prune.in'))
                fmake = jobs[f'{lane}.F.make']['argv']
                self.assertNotIn('--freq', fmake)
                self.assertNotIn('--indep-pairwise', fmake)
                self.assertEqual(fmake[fmake.index('--threads') + 1], '4')
                self.assertNotIn('--from-bp', fmake)
            self.assertIn('--ac-founders', jobs['gt.F.qc']['argv'])
            self.assertIn('log10', jobs['gt.F.hwe-tail']['argv'])
            self.assertTrue(jobs['negative.49founders']['expect_failure'])
            self.assertFalse(jobs['gt.F.make']['expect_failure'])
            execution = [
                dict(name='broken', deps=[], expect_failure=False,
                     argv=[sys.executable, '-B', '-c', 'print("retained failure"); raise SystemExit(7)']),
                dict(name='child', deps=['broken'], expect_failure=False,
                     argv=[sys.executable, '-B', '-c', 'raise SystemExit(99)']),
                dict(name='negative', deps=[], expect_failure=True, error_pattern='expected input error',
                     argv=[sys.executable, '-B', '-c', 'import sys; sys.stderr.write("expected input error"); sys.exit(3)']),
                dict(name='wrong-negative', deps=[], expect_failure=True, error_pattern='right class',
                     argv=[sys.executable, '-B', '-c', 'raise SystemExit(2)']),
                dict(name='permissive-diagnostic', deps=[], expect_failure=False, observe_outcome=True,
                     diagnostic='classification only, no native policy',
                     argv=[sys.executable, '-B', '-c', 'print("observed acceptance")']),
            ]
            result = oracle.execute_jobs(execution, root / 'calls')
            self.assertEqual({k: v['status'] for k, v in result.items()},
                             {'broken': 'failed', 'child': 'blocked',
                              'negative': 'expected_failure', 'wrong-negative': 'failed',
                              'permissive-diagnostic': 'observed_return'})
            self.assertIn('retained failure', (root / 'calls/broken/stdout.txt').read_text())
            self.assertEqual(json.loads((root / 'calls/broken/request.json').read_text())['argv'][-1],
                             'print("retained failure"); raise SystemExit(7)')
            with self.assertRaisesRegex(RuntimeError, 'hosted Linux x86_64'):
                oracle.hosted(root)
            objects = root / 'objects'
            objects.mkdir()
            (objects / 'upper').mkdir()
            (objects / 'lower').mkdir()
            (objects / 'upper/A').write_text('upper')
            (objects / 'lower/a').write_text('lower')
            (objects / 'alias').symlink_to('upper/A')
            oracle.bind_manifest(objects)
            oracle.verify_manifest(objects)
            archive = root / 'complete.tar'
            oracle.package(objects, archive)
            with tarfile.open(archive) as stream:
                members = {m.name: m for m in stream.getmembers()}
                self.assertTrue(members['objects/alias'].issym())
                self.assertIn('objects/upper/A', members)
                self.assertIn('objects/lower/a', members)
            receipt = json.loads((root / 'complete.tar.json').read_text())
            self.assertEqual(receipt['sha256'], oracle.sha256(archive))
            probe = root / 'case-probe'
            try:
                observed = oracle.case_sensitive_probe(probe)
            except ValueError:
                self.assertTrue((probe / 'A').samefile(probe / 'a'), 'case-sensitive host wrongly rejected')
            else:
                self.assertTrue(observed['case_sensitive'])
                self.assertEqual((probe / 'A').read_text(), 'upper\n')
                self.assertEqual((probe / 'a').read_text(), 'lower\n')
            (objects / 'upper/A').write_text('tampered')
            with self.assertRaisesRegex(ValueError, 'manifest'):
                oracle.verify_manifest(objects)
            bad_spec = copy.deepcopy(lock)
            bad_spec['source']['commit'] = 'master'
            with self.assertRaisesRegex(ValueError, 'source'):
                oracle.validate_spec(bad_spec)
            bad_spec = copy.deepcopy(lock)
            bad_spec['image']['platform'] = 'linux/arm64'
            with self.assertRaisesRegex(ValueError, 'container'):
                oracle.validate_spec(bad_spec)
            source_tar = root / 'synthetic-source.tar.gz'
            payload = b'engineering source-tree verification fixture\n'
            link = b'../file'
            tree = dict(sha=oracle.COMMIT, truncated=False, tree=[
                dict(path='file', type='blob', mode='100644',
                     sha=hashlib.sha1(b'blob ' + str(len(payload)).encode() + b'\0' + payload).hexdigest()),
                dict(path='nested/link', type='blob', mode='120000',
                     sha=hashlib.sha1(b'blob ' + str(len(link)).encode() + b'\0' + link).hexdigest())])
            with tarfile.open(source_tar, 'w:gz') as stream:
                member = tarfile.TarInfo('plink-ng-' + oracle.COMMIT + '/file')
                member.size = len(payload)
                stream.addfile(member, io.BytesIO(payload))
                member = tarfile.TarInfo('plink-ng-' + oracle.COMMIT + '/nested/link')
                member.type = tarfile.SYMTYPE
                member.linkname = '../file'
                stream.addfile(member)
            verified = oracle.verify_source_archive(source_tar, tree, {}, root / 'synthetic-source', 1000)
            self.assertEqual(verified['files'], 2)
            self.assertEqual((root / 'synthetic-source/nested/link').read_bytes(), payload)
            workflow = (ROOT / '.github/workflows/plink-first-workflow-oracle.yml').read_text()
            self.assertIn('if: always()', workflow)
            self.assertIn('plink-first-workflow-oracle.tar.json', workflow)
            self.assertIn('--action run', workflow)


if __name__ == '__main__':
    unittest.main()
