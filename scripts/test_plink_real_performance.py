"""One whole preparation contract; text transport only, no scientific execution."""

import gzip
import hashlib
import importlib
import io
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import tempfile
import unittest

from test_infercnv_ingestion_workflow import bash_body, steps


ROOT = Path(__file__).resolve().parents[1]


class PlinkRealPreparation(unittest.TestCase):
    def test_private_whole_workflow_identity_raw_stream_and_fixed_budgets(self):
        self.assertTrue((ROOT / 'scripts/plink_real_performance.py').is_file(),
                        'integrated private HGDP controller is not implemented')
        real = importlib.import_module('plink_real_performance')
        observer = importlib.import_module('plink_observer_admission')
        self.assertTrue(callable(getattr(real, 'finalization_budget', None)),
                        'complete retained-plus-ciphertext budget is not implemented')
        self.assertFalse({'numpy', 'pandas', 'pgenlib', 'scipy'}.intersection(sys.modules))
        spec = real.config()
        self.assertEqual(sum(row['bytes'] for row in spec['objects']), 274156684)
        self.assertTrue(all(row['sha256'] is None for row in spec['objects']))
        self.assertEqual(spec['sample_count'], 929)
        with tempfile.TemporaryDirectory(prefix='plink-real-preparation-', dir=os.environ['TMPDIR']) as name:
            root = Path(name)
            private = root / 'private'; private.mkdir(mode=0o700)
            self.assertEqual(real.private_directory(private), private.resolve())
            private.chmod(0o755)
            with self.assertRaisesRegex(ValueError, 'owner-only'):
                real.private_directory(private)
            private.chmod(0o700)
            (private / 'synthetic.tsv').write_text('synthetic\t7\n')
            (root / 'outside.txt').write_text('must not be followed\n')
            (private / 'alias').symlink_to('../outside.txt')
            real.write_private_manifest(private)
            carrier = io.BytesIO()
            real.archive_stream(private, carrier)
            with tarfile.open(fileobj=io.BytesIO(carrier.getvalue()), mode='r:') as archive:
                self.assertEqual(set(archive.getnames()), {'plink-private', 'plink-private/alias',
                    'plink-private/synthetic.tsv', 'plink-private/final-private-manifest.json'})
                self.assertEqual(archive.extractfile('plink-private/synthetic.tsv').read(), b'synthetic\t7\n')
                self.assertTrue(archive.getmember('plink-private/alias').issym())
                self.assertEqual(archive.getmember('plink-private/alias').linkname, '../outside.txt')
            self.assertLessEqual(len(carrier.getvalue()), real.archive_plan(private)['tar_bound_bytes'])
            self.assertTrue((root / 'outside.txt').is_file())
            self.assertEqual(real.finalization_budget(100, 1000, 8726284172)['required_free_bytes'], 8726284172)
            with self.assertRaisesRegex(ValueError, 'finalization'):
                real.finalization_budget(100, 1000, 8726284171)
            recipient = real.admit_recipient(ROOT)
            self.assertEqual(hashlib.sha256(recipient.read_bytes()).hexdigest(),
                             '8bb86dc97982fc7b64d7e33f7c3741e7f3d27a21690f48ddce9dada4a230680d')
            adopted, _ = real.committed_receipt(ROOT, real.DEFAULT_RECEIPT)
            self.assertEqual(adopted['snapshot'], 'green-2')
            self.assertEqual(set(adopted['controller_sha256']), set(real.source.CONTROLLERS))
            with self.assertRaisesRegex(ValueError, 'historical'):
                real.source.extract_native(root / 'absent.tar', root / 'absent', ROOT,
                                           expected_head=adopted['head_sha'])
            invalid_hashes = dict(adopted['controller_sha256'])
            invalid_hashes[next(iter(invalid_hashes))] = 'bad'
            with self.assertRaisesRegex(ValueError, 'historical'):
                real.source.extract_native(root / 'absent.tar', root / 'absent', ROOT,
                    expected_head=adopted['head_sha'], expected_controller_hashes=invalid_hashes)
            for tool in ('source', 'official', 'native'):
                for workers in (1, 4):
                    graph = real.graph(tool, workers, root / 'INPUT', root / tool / str(workers), root / 'binary')
                    self.assertEqual([row['name'] for row in graph if row['name'] in ('I', 'F', 'pairs', 'prune', 'P')],
                                     ['I', 'F', 'pairs', 'prune', 'P'])
                    jobs = {row['name']: row for row in graph}
                    self.assertIn(str(root / tool / str(workers) / 'I'), jobs['F']['argv'])
                    self.assertIn(str(root / tool / str(workers) / 'F'), jobs['pairs']['argv'])
                    self.assertIn(str(root / tool / str(workers) / 'prune.prune.in'), jobs['P']['argv'])
                    self.assertIn(str(root / tool / str(workers) / 'P'), jobs['P.qc']['argv'])
                    self.assertNotIn('--hwe', jobs['F']['argv'])
                    self.assertEqual(jobs['F']['argv'][jobs['F']['argv'].index('--maf') + 1], '0.05')
                    self.assertTrue(all(row['argv'][row['argv'].index('--threads') + 1] == str(workers) for row in graph))
                    self.assertEqual('--memory-mib' in jobs['I']['argv'], tool == 'native')
                    self.assertIn('0.2', jobs['pairs']['argv'])
            self.assertEqual([row['order'] for row in real.pair_schedule() if row['workers'] == 4],
                             [['native', 'source'], ['source', 'native'], ['native', 'source'],
                              ['source', 'native'], ['native', 'source']])
            prep = real.preparation_plan(root / 'providers', root / 'prepared', root / 'source')
            self.assertEqual(len(prep), 5)
            for chrom, job in zip((19, 20, 21, 22), prep):
                self.assertEqual(job['argv'][job['argv'].index('--chr') + 1], str(chrom))
                self.assertEqual(job['argv'][job['argv'].index('--from-bp') + 1], '20000001')
                self.assertEqual(job['argv'][job['argv'].index('--to-bp') + 1], '22000000')
            original = root / 'rows.pvar'
            original.write_text('##source=transport\n#CHROM\tPOS\tID\tREF\tALT\n'
                                '19\t20000000\t.\tA\tG\n19\t20000001\tdup\tA\tG,T\n'
                                '19\t22000000\tdup\tC\tT\n')
            result = real.rewrite_ids(original, root / 'ids.pvar', root / 'lineage.jsonl.gz', '19', 'a' * 64)
            self.assertEqual(result['rows'], 3)
            self.assertEqual((root / 'ids.pvar').read_text().splitlines()[3],
                             '19\t20000001\tHGDP20190516_C19_R000000002\tA\tG,T')
            with gzip.open(root / 'lineage.jsonl.gz', 'rt') as stream:
                lineage = [json.loads(line) for line in stream]
            self.assertEqual([row['original_id'] for row in lineage], ['.', 'dup', 'dup'])
            self.assertEqual([row['ordinal'] for row in lineage], [1, 2, 3])
            self.assertEqual(lineage[1]['original_line'], '19\t20000001\tdup\tA\tG,T\n')

            header = dict(schema=1, source='cc2c95e61c509d49aaf884d2ec0956a921770745', samples=2, variants=2,
                          semantics='raw codec; chromosome correction belongs to caller')
            rows = [dict(variant=0, provisional_ref=False, calls=[[0, 1, 1, 0, None, None], [1, 1, 0, 0, None, None]]),
                    dict(variant=1, provisional_ref=True, calls=[[1, 0, 1, 1, None, None], [-9, -9, 0, 0, None, None]])]
            raw = ('\n'.join(json.dumps(row) for row in [header, *rows]) + '\n').encode()
            receipt = real.raw_stream(io.BytesIO(raw), root / 'first.jsonl.gz', [2, 2])
            self.assertEqual(receipt['decompressed_sha256'], hashlib.sha256(raw).hexdigest())
            self.assertEqual(receipt['cells'], 4)
            self.assertEqual(gzip.decompress((root / 'first.jsonl.gz').read_bytes()), raw)
            real.raw_stream(io.BytesIO(raw), root / 'same.jsonl.gz', [2, 2], root / 'first.jsonl.gz')
            changed = raw.replace(b'[1, 0, 1, 1, null, null]', b'[0, 1, 0, 0, null, null]')
            with self.assertRaisesRegex(ValueError, 'raw'):
                real.raw_stream(io.BytesIO(changed), root / 'changed.jsonl.gz', [2, 2], root / 'first.jsonl.gz')
            with self.assertRaisesRegex(ValueError, 'dosage'):
                real.raw_stream(io.BytesIO(raw.replace(b'null, null', b'17000, null', 1)), root / 'dose.jsonl.gz', [2, 2])
            self.assertFalse((root / 'first.jsonl').exists())
            for prefix, ids, variants in [('parent', ['u', 'v'], [('a', 'A', 'G'), ('b', 'C', 'T')]),
                                          ('child', ['u'], [('b', 'C', 'T')])]:
                (root / (prefix + '.psam')).write_text('#IID\n' + ''.join(value + '\n' for value in ids))
                (root / (prefix + '.pvar')).write_text('#CHROM\tPOS\tID\tREF\tALT\n' + ''.join(
                    '19\t' + ('20000001' if value == 'a' else '20000002') + f'\t{value}\t{ref}\t{alt}\n'
                    for value, ref, alt in variants))
            child_raw = ('\n'.join(json.dumps(row) for row in [dict(header, samples=1, variants=1),
                dict(variant=0, provisional_ref=True, calls=[[1, 0, 1, 1, None, None]])]) + '\n').encode()
            real.raw_stream(io.BytesIO(child_raw), root / 'child.jsonl.gz', [1, 1])
            self.assertEqual(real.compare_projection(root / 'parent', root / 'child', root / 'first.jsonl.gz', root / 'child.jsonl.gz'), 1)
            real.raw_stream(io.BytesIO(child_raw.replace(b'[1, 0, 1, 1, null, null]', b'[0, 1, 0, 0, null, null]')),
                            root / 'bad-child.jsonl.gz', [1, 1])
            with self.assertRaisesRegex(ValueError, 'projection'):
                real.compare_projection(root / 'parent', root / 'child', root / 'first.jsonl.gz', root / 'bad-child.jsonl.gz')

            snapshot = ROOT / '.autopilot/snapshots/plink-native-2026-11-01/green-1/source.tar.gz'
            with tarfile.open(snapshot, 'r:gz') as archive:
                for entry in ('tests/compare_workflow.py', 'tests/COMPARISON.md', 'tests/fixtures/expected/SOURCE.json'):
                    target = root / 'comparison' / entry; target.parent.mkdir(parents=True, exist_ok=True)
                    target.write_bytes(archive.extractfile(entry).read())
            comparison = real.source.comparator(root / 'comparison')
            (root / 'expected.hardy').write_text('#ID\tMIDP\nv\t4.61606e-19\n')
            (root / 'actual.hardy').write_text('#ID\tMIDP\nv\t4.6160601e-19\n')
            self.assertEqual(real.compare_report(root / 'actual.hardy', root / 'expected.hardy', comparison), 1)
            (root / 'actual.hardy').write_text('#ID\tMIDP\nv\t0\n')
            with self.assertRaises((ValueError, AssertionError)):
                real.compare_report(root / 'actual.hardy', root / 'expected.hardy', comparison)
            changed_source = observer.widen((ROOT / 'scripts/plink_pgen_capture.cc').read_bytes())
            self.assertIn(b'variants.size() > 500000 || samples.size() > 1000', changed_source)
            with self.assertRaisesRegex(ValueError, 'identity'):
                observer.widen(changed_source)
            self.assertEqual(real.storage_bound(1024, [2048] * 6, 16 * 2**30)['retained_bound_bytes'], 1024 + 20 * 2048 + 26 * 2**20)
            with self.assertRaisesRegex(ValueError, '64'):
                real.storage_bound(64 * 2**30, [2048] * 6, 256 * 2**30)
            measurements = [dict(tool=tool, wall_seconds=wall + i, scientific_cli_wall_seconds=7 * stage,
                aggregate_tree_peak_bytes=None, stages=[dict(group=group, wall_seconds=stage) for group in
                    ('I', 'F', 'F.qc', 'pairs', 'prune', 'P', 'P.qc')])
                for tool, wall, stage in [('source', 100, 10), ('native', 50, 5)] for i in range(5)]
            self.assertTrue(real.decision(measurements)['passed'])
            for row in measurements:
                if row['tool'] == 'native': row['stages'][1]['wall_seconds'] = 11
            self.assertFalse(real.decision(measurements)['passed'])
            needs = 'Version needs section:\n File: libc.so.6 Cnt: 1\n Name: GLIBC_2.39 Flags: none\n'
            with self.assertRaisesRegex(ValueError, 'GLIBC_2.39'):
                real.check_version_closure(real.version_needs(needs), {'libc.so.6': 'Version definition section:\n Name: GLIBC_2.36\n'})
            workflow = (ROOT / '.github/workflows/plink-real-performance.yml').read_text()
            self.assertIn('runs-on: ubuntu-24.04', workflow)
            self.assertNotIn('runner_labels:', workflow)
            self.assertNotIn('private_storage:', workflow)
            upload = steps(workflow)['Upload encrypted originals only']
            paths = upload.split('path: |', 1)[1].split('retention-days:', 1)[0]
            self.assertIn('/delivery/hgdp.tar.age', paths)
            self.assertIn('/delivery/receipt.json', paths)
            self.assertNotIn('.stderr', paths)
            self.assertNotIn('*', paths)
            self.assertNotRegex(workflow.split('permissions:', 1)[0], r'\n  (push|pull_request|schedule):')
            for title, step in steps(workflow).items():
                if '        run: |\n' in step:
                    body = re.sub(r'\$\{\{.*?\}\}', 'route-only', bash_body(step))
                    checked = subprocess.run(['bash', '-n'], input=body, text=True, capture_output=True)
                    self.assertEqual(checked.returncode, 0, title + checked.stderr)
            denied = subprocess.run([sys.executable, '-B', str(ROOT / 'scripts/plink_real_performance.py'),
                                     'run', '--root', str(root / 'denied'), '--world', str(ROOT),
                                     '--native-receipt', '.autopilot/oracles/missing.json', '--cpus', '0,1,2,3'],
                                    env=dict(os.environ, GITHUB_ACTIONS='false'), capture_output=True, text=True)
            self.assertNotEqual(denied.returncode, 0)
            self.assertIn('hosted', denied.stderr)
            self.assertFalse((root / 'denied').exists())


if __name__ == '__main__':
    unittest.main()
