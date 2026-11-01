"""One connected, stdlib-only preparation contract; no scientific execution."""

import copy
import hashlib
import importlib
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))


class IndexBgzfPreparation(unittest.TestCase):
    def test_whole_graph_custody_recorder_and_primary_failure_contract(self):
        self.assertTrue((ROOT / 'scripts/index_bgzf_real_performance.py').is_file(),
                        'the complete BGZF/tabix operational controller is absent')
        p = importlib.import_module('index_bgzf_real_performance')
        policy = p.read_policy()
        context = p.verify_context(ROOT, policy)
        self.assertEqual(context['owner_files'], 116)
        self.assertFalse(policy['recovery']['allocated'])
        self.assertEqual(policy['recovery']['bytes'], 81604378624)
        self.assertEqual(policy['timer']['deb']['bytes'], 50776)
        self.assertEqual(policy['timer']['deb']['sha256'],
                         '10257c6fe888e6578550b1f20705d282dd630b76f8df0c006aec3031fefe5e23')
        with self.assertRaisesRegex(RuntimeError, 'recovery'):
            p.require_recovery(False, '')
        p.require_recovery(True, 'root-verified-external-76GiB-record')
        lanes = ['compress_text_t1', 'compress_text_t4', 'compress_binary_t1', 'compress_binary_t4',
                 'decompress_text_t1', 'decompress_binary_t1', 'partial_binary_512m',
                 'build_tbi', 'build_csi', 'query_sparse_t1', 'query_dense_t4',
                 'query_overlap_unique_t4', 'query_targets_t4']
        benchmark, qualification = p.schedule(), p.qualification_schedule()
        self.assertEqual(len(benchmark), 338)
        self.assertEqual(len(qualification), 42)
        self.assertEqual([n['lane'] for n in benchmark[::26]], lanes)
        self.assertEqual(sum(n['timeout'] for n in benchmark), 13650)
        self.assertEqual(sum(n['timeout'] for n in qualification), 1750)
        self.assertEqual([n['engine'] for n in benchmark[:10]],
                         ['source', 'native', 'native', 'source', 'source', 'native',
                          'source', 'native', 'native', 'source'])
        self.assertEqual(sum(n['phase'] == 'warmup' for n in benchmark), 78)
        self.assertEqual(sum(n['phase'] == 'trial' for n in benchmark), 260)
        self.assertEqual([sum(n['kind'] == kind for n in qualification)
                          for kind in ('lane', 'cross-decode', 'cross-index')], [26, 8, 8])
        self.assertEqual(len({n['id'] for n in benchmark + qualification}), 380)
        root = Path(tempfile.mkdtemp(prefix='index-bgzf-connected-', dir=os.environ['TMPDIR']))
        print('preparation_evidence=' + str(root))
        programs = dict(native=Path('/exact/native'), bgzip=Path('/exact/bgzip'),
                        tabix=Path('/exact/tabix'), timer=Path('/exact/time'))
        for row in benchmark:
            argv, output = p.command_for(root, programs, row)
            if row['lane'].startswith('compress'):
                self.assertEqual(argv[argv.index('--threads') + 1], str(row['workers']))
                self.assertEqual(argv[argv.index('--compress-level') + 1], '6')
                self.assertEqual('--binary' in argv, 'binary' in row['lane'])
                self.assertIsNotNone(output)
            elif row['lane'].startswith('build'):
                self.assertIsNotNone(output)
                self.assertEqual('--csi' in argv, row['lane'] == 'build_csi')
            else:
                self.assertIsNone(output)
            if row['lane'].startswith('query') and row['workers'] == 4:
                self.assertEqual(argv[argv.index('--threads') + 1],
                                 '4' if row['engine'] == 'native' else '3')
        read = next(n for n in benchmark if n['lane'] == 'decompress_text_t1')
        argv, _ = p.command_for(root, programs, read)
        timed = p.timed_argv(programs['timer'], argv, [2], '/dev/null', root / 'usage.json', 3 * 2**30)
        self.assertEqual(timed[:3], ['bash', '-c', p.EXEC_REDIRECT])
        self.assertIn('/dev/null', timed)
        at = timed.index('/exact/time')
        self.assertEqual(timed[at + 1:at + 3], ['-f', p.time_format()])
        self.assertEqual(timed[timed.index('taskset'):timed.index('taskset') + 3], ['taskset', '-c', '2'])
        self.assertNotIn(sys.executable, timed[at:])
        usage = root / 'usage.json'
        usage.write_text(json.dumps(dict(wall_seconds=9.0, user_seconds=8.0, system_seconds=.5,
            max_rss_kib=3000, filesystem_inputs=2, filesystem_outputs=4,
            major_faults=1, minor_faults=20, exit_status=0)))
        measured = p.parse_usage(usage)
        self.assertEqual(measured['peak_rss_bytes'], 3072000)
        self.assertEqual(measured['total_cpu_seconds'], 8.5)
        self.assertEqual(measured['raw_gnu_time']['filesystem_outputs'], 4)
        records = [dict(row, status='returned', returncode=0, group_empty=True,
                        qualification_passed=True, **dict(measured, wall_seconds=10.0 if row['engine'] == 'source' else 9.0,
                        peak_rss_bytes=1000000 if row['engine'] == 'source' else 800000)) for row in benchmark]
        report = p.evaluate(records)
        self.assertTrue(report['primary_passed'])
        self.assertEqual(set(report['primary']),
                         {'compress_binary_t1', 'build_tbi', 'build_csi', 'query_sparse_t1'})
        self.assertFalse(report['publication'])
        self.assertEqual(len(report['lanes']), 13)
        self.assertEqual(len(report['lanes']['build_tbi']['paired_wall_ratios']), 10)
        changed = copy.deepcopy(records)
        for row in changed:
            if row['engine'] == 'native':
                row['wall_seconds'] = 10.0 if row['lane'] == 'build_tbi' else .01
        self.assertFalse(p.evaluate(changed)['primary_passed'])
        for row in changed:
            if row['engine'] == 'native' and row['lane'] == 'build_tbi':
                row.update(wall_seconds=10.5, peak_rss_bytes=750000)
        self.assertTrue(p.evaluate(changed)['primary_passed'])
        for row in changed:
            if row['engine'] == 'native' and row['lane'] == 'build_tbi':
                row['wall_seconds'] = 10.50001
        self.assertFalse(p.evaluate(changed)['primary_passed'])
        tiny_gap = copy.deepcopy(records)
        for row in tiny_gap:
            row['wall_seconds'] = .10 if row['engine'] == 'source' else .09
        self.assertFalse(p.evaluate(tiny_gap)['primary_passed'])
        for mutation in ('missing', 'source_failed', 'leftover', 'qualification'):
            altered = copy.deepcopy(records)
            if mutation == 'missing':
                altered.pop()
            elif mutation == 'source_failed':
                altered[0]['returncode'] = 1
            elif mutation == 'leftover':
                altered[0]['group_empty'] = False
            else:
                altered[0]['qualification_passed'] = False
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                p.evaluate(altered)
        inputs = root / 'inputs'
        active = root / 'active'
        inputs.mkdir()
        active.mkdir()
        backup = inputs / 'immutable'
        duplicate = active / 'duplicate'
        backup.write_bytes(b'complete\x00body\n')
        backup.chmod(0o444)
        duplicate.write_bytes(b'complete\x00body\n')
        custody = p.duplicate_record(root, duplicate, backup, 'synthetic-repeat')
        self.assertEqual(custody['identity']['bytes'], 14)
        self.assertEqual(custody['backup'], 'inputs/immutable')
        self.assertTrue(custody['full_byte_comparison'])
        self.assertTrue(duplicate.exists(), 'preparation never executes production duplicate removal')
        duplicate.write_bytes(b'novel body\n')
        with self.assertRaisesRegex(ValueError, 'body'):
            p.duplicate_record(root, duplicate, backup, 'synthetic-mismatch')
        self.assertEqual(duplicate.read_bytes(), b'novel body\n')
        frozen = p.freeze_files([backup])
        p.check_frozen(frozen)
        backup.chmod(0o644)
        backup.write_bytes(b'changed\x00body\n')
        with self.assertRaisesRegex(ValueError, 'frozen'):
            p.check_frozen(frozen)
        carrier = root / 'synthetic.tar'
        payload = b'not an executable\n'
        with tarfile.open(carrier, 'w') as archive:
            body = tarfile.TarInfo('runtime/actual-body')
            body.size, body.mode = len(payload), 0o700
            archive.addfile(body, io.BytesIO(payload))
            link = tarfile.TarInfo('runtime/native')
            link.type, link.linkname, link.mode = tarfile.LNKTYPE, 'runtime/actual-body', 0o700
            archive.addfile(link)
        restored = root / 'restored'
        restored.mkdir()
        pin = dict(bytes=len(payload), sha256=hashlib.sha256(payload).hexdigest())
        p.restore_selected(carrier, restored, {'runtime/native': pin})
        self.assertEqual((restored / 'runtime/native').read_bytes(), payload)
        self.assertEqual((restored / 'runtime/native').stat().st_mode & 0o777, 0o700)
        used = {name: 0 for name in ('inputs', 'runtime', 'cas', 'metadata', 'active')}
        p.capacity_decision(used, 41 * 2**30)
        with self.assertRaisesRegex(RuntimeError, 'capacity'):
            p.capacity_decision(used, 39 * 2**30)
        p.reserve_time(0, 1799, 16600)
        with self.assertRaisesRegex(RuntimeError, 'time'):
            p.reserve_time(0, 2001, 16600)
        p.capture_child(root, 'stdlib-ok', [sys.executable, '-B', '-c', 'print("retained")'], timeout=2)
        self.assertEqual((root / 'calls/stdlib-ok/process.stdout').read_bytes(), b'retained\n')
        for name, code, seconds, status in [('nonzero', 'raise SystemExit(7)', 2, 'nonzero'),
                                            ('timeout', 'import time; time.sleep(5)', .05, 'timeout')]:
            with self.assertRaises(p.TrialProcessError):
                p.capture_child(root, 'stdlib-' + name, [sys.executable, '-B', '-c', code], timeout=seconds)
            observation = json.loads((root / f'calls/stdlib-{name}/process.process.json').read_text())
            self.assertEqual(observation['status'], status)
            self.assertTrue(observation['group_empty'])
        p.validate_workflow(ROOT / p.WORKFLOW)
        for block in p.source.shell_blocks(ROOT / p.WORKFLOW):
            checked = subprocess.run(['bash', '-n'], input=block, text=True, capture_output=True)
            self.assertEqual(checked.returncode, 0, checked.stderr)
        with self.assertRaisesRegex(RuntimeError, 'hosted'):
            p.hosted_session()
        self.assertFalse({'numpy', 'pysam', 'allel', 'rpy2'}.intersection(sys.modules))

        previous = os.environ.get('INDEX_CHANGED_NATIVE')
        os.environ['INDEX_CHANGED_NATIVE'] = 'true'
        try:
            changed = p.read_policy()
            self.assertEqual(changed['owner'], 'd57bdaa2a19492d902a5f59c8de3c9f35113053a')
            context = p.verify_context(ROOT, changed)
            self.assertEqual(context['donor_owner'], '030be20895cb7806f78b2923e1c1ac3afa05cd96')
            self.assertEqual(context['native_owner'], changed['owner'])
            self.assertEqual(changed['material'], policy['material'])
            self.assertEqual(changed['fixture'], policy['fixture'])
            selector = json.loads((ROOT / changed['changed_native']['source_selector']['path']).read_text())
            local_owner = os.environ.get('INDEX_PREP_OWNER')
            source_fixture = root / 'native-source'
            source_fixture.mkdir()
            fixture_selector = dict(head_sha='public-stdlib-fixture', tree='9fbac661424a5e1e7fb9c1a6a516b58fc701f2f0', files={
                'greeting': dict(bytes=6, sha256='5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03',
                    git_blob='ce013625030ba8dba906f756967f9e9ca394464a', git_mode='100644')})
            (source_fixture / 'greeting').write_bytes(b'hello\n')
            binding = p.native_source_binding(source_fixture, fixture_selector)
            self.assertEqual(binding['tree'], '9fbac661424a5e1e7fb9c1a6a516b58fc701f2f0')
            if local_owner:
                full_fixture = root / 'native-full-source'
                for name, expected in selector['files'].items():
                    path = full_fixture / name
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_bytes((Path(local_owner) / name).read_bytes())
                    path.chmod(0o700 if expected['git_mode'] == '100755' else 0o600)
                full = p.native_source_binding(full_fixture, selector)
                self.assertEqual(full['tree'], 'b47ad82cd9127030dbdb94dbc85121b99a064d36')
                self.assertEqual(len(full['files']), 116)
                print('actual_local_owner_source_bodies=116; data_only=True')
            else:
                print('actual_local_owner_source_bodies=not_requested; portable_fixture_only=True')
            wrong = copy.deepcopy(fixture_selector)
            wrong['files']['greeting']['sha256'] = '0' * 64
            with self.assertRaisesRegex(ValueError, 'source'):
                p.native_source_binding(source_fixture, wrong)
            self.assertEqual(p.read_policy(changed_native=False)['owner'], policy['owner'])
            mixed = copy.deepcopy(changed)
            mixed['owner'] = policy['owner']
            with self.assertRaisesRegex(ValueError, 'source'):
                p.verify_context(ROOT, mixed)
            changed_workflow = root / 'incorrect-default.yml'
            text = (ROOT / p.WORKFLOW).read_text()
            text = text.replace('default: false', 'default: true', 1)
            changed_workflow.write_text(text)
            with self.assertRaisesRegex(ValueError, 'default-false'):
                p.validate_workflow(changed_workflow)
            spec = p.native_build_spec(root, changed)
            self.assertEqual(spec['build'][1:], ['build', '--offline', '--locked', '--release', '--bin', 'rsomics-index'])
            self.assertIn('runtime/native/target', str(spec['target']))
            self.assertEqual(spec['target_limit_bytes'], 536870912)
            self.assertEqual(spec['environment']['RUSTFLAGS'], '')
            self.assertFalse(spec['target'].exists())
            spec['target'].mkdir(parents=True)
            with self.assertRaisesRegex(ValueError, 'target'):
                p.native_build_spec(root, changed)
            monitor_cases = [('bounded', 1024, 32), ('over-limit', 128, 256)]
            free_bytes = shutil.disk_usage(root).free
            required_free = sum(p.CATEGORIES.values()) + 8 * 2**30
            if free_bytes < required_free:
                print(f'child_monitor_cases=not_run; reason=resource_unavailable; '
                      f'free_bytes={free_bytes}; required_free_bytes={required_free}')
                monitor_cases = []
            for name, limit, payload in monitor_cases:
                observed = root / name
                observed.mkdir()
                target = observed / 'runtime/native/target'
                target.mkdir(parents=True)
                argv = [sys.executable, '-B', '-c',
                    'from pathlib import Path;import sys,time;Path(sys.argv[1]).write_bytes(b"x"*int(sys.argv[2]));time.sleep(.2 if int(sys.argv[2])==32 else 5)',
                    str(target / 'retained'), str(payload)]
                if name == 'bounded':
                    with p.monitor_native_build(observed, target, limit, .01):
                        p.capture_child(observed, 'build-observation', argv, 2)
                else:
                    with self.assertRaises(p.TrialProcessError):
                        with p.monitor_native_build(observed, target, limit, .01):
                            p.capture_child(observed, 'build-observation', argv, 2)
                process = json.loads((observed / 'calls/build-observation/process.process.json').read_text())
                capacity = json.loads((observed / 'metadata/changed-native-build-capacity.json').read_text())
                self.assertTrue(process['group_empty'])
                self.assertEqual(capacity['peak_target_bytes'], payload)
                self.assertFalse(capacity['hard_filesystem_quota'])
                self.assertEqual((target / 'retained').stat().st_size, payload)
                print(f'child_monitor_case={name}; executed=True')
        finally:
            if previous is None:
                os.environ.pop('INDEX_CHANGED_NATIVE', None)
            else:
                os.environ['INDEX_CHANGED_NATIVE'] = previous


if __name__ == '__main__':
    unittest.main()
