"""One transport/preparation check; no native or upstream scientific execution."""

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
WORKFLOW = ROOT / '.github/workflows/plink-candidate.yml'


class PlinkCandidatePreparationTest(unittest.TestCase):
    def test_plink_candidate_workflow(self):
        self.assertTrue(WORKFLOW.is_file(), 'complete PLINK native-four/source gate is absent')
        gate = importlib.import_module('plink_candidate')
        source = importlib.import_module('plink_source_on_native')
        text = WORKFLOW.read_text()
        routes = json.loads(re.search(r"fromJSON\('([^']+)'\)", text)[1])
        self.assertEqual([(row['target'], row['runner'], row['system'], row['arch']) for row in routes], [
            ('linux-x86_64', 'ubuntu-24.04', 'Linux', 'x86_64'),
            ('linux-aarch64', 'ubuntu-24.04-arm', 'Linux', 'aarch64'),
            ('macos-x86_64', 'macos-15-intel', 'Darwin', 'x86_64'),
            ('macos-aarch64', 'macos-15', 'Darwin', 'arm64')])
        bodies = {}
        for name, step in steps(text).items():
            if '        run: |\n' not in step:
                continue
            body = re.sub(r'\$\{\{.*?\}\}', 'route-only', bash_body(step))
            bodies[name] = body
            checked = subprocess.run(['bash', '-n'], input=body, text=True, capture_output=True)
            self.assertEqual(checked.returncode, 0, name + checked.stderr)
        self.assertNotIn('continue-on-error', text)
        self.assertNotRegex(text.split('permissions:', 1)[0], r'\n  (push|pull_request|schedule):')
        with tempfile.TemporaryDirectory(prefix='plink-gate-preparation-') as directory:
            root = Path(directory)
            runner = root / 'runner'; runner.mkdir()
            envfile = root / 'environment'
            env = dict(os.environ, RUNNER_TEMP=str(runner), GITHUB_WORKSPACE=str(ROOT),
                       GITHUB_ENV=str(envfile), GITHUB_ACTIONS='true',
                       GITHUB_REPOSITORY='omics-rust/rsomics-world',
                       CONTROLLER_COMMIT='route-only', SNAPSHOT_ID='missing', TARGET='route-only')
            actual = subprocess.run(['bash', '-c', bodies['Prepare native storage']], cwd=ROOT,
                                    env=env, text=True, capture_output=True)
            self.assertEqual(actual.returncode, 0, actual.stderr)
            env.update(line.split('=', 1) for line in envfile.read_text().splitlines())
            self.assertTrue(all(Path(value).is_relative_to(runner.resolve())
                                for value in gate.storage(env, require_space=0)['paths'].values()))
            with self.assertRaises(ValueError):
                gate.storage(dict(env, TMPDIR=str(root)), require_space=0)
            for selector in ('missing', '../green-1', 'green-0', 'green-1/extra', 'green-99'):
                with self.assertRaises((ValueError, OSError)):
                    gate.prepare_source(ROOT, root / selector.replace('/', '_'), selector)
            for selector in ('green-1', 'green-2', 'green-3'):
                prepared = root / ('prepared-' + selector)
                manifest = gate.prepare_source(ROOT, prepared, selector)
                self.assertEqual('src/numeric.rs' in manifest, selector == 'green-3')
                self.assertEqual(hashlib.sha256((prepared / 'source/Cargo.lock').read_bytes()).hexdigest(),
                                 manifest['Cargo.lock'])

            snap = root / 'snapshot'; snap.mkdir()
            payload = b'[package]\nname="transport-only"\n'
            with tarfile.open(snap / 'source.tar.gz', 'w:gz') as archive:
                member = tarfile.TarInfo('Cargo.toml'); member.size = len(payload)
                archive.addfile(member, io.BytesIO(payload))
            (snap / 'README.md').write_text('Transport only, never scientific admission.\n')
            (snap / 'files.sha256').write_text(hashlib.sha256(payload).hexdigest() + '  Cargo.toml\n')
            (snap / 'checksums.sha256').write_text(''.join(
                hashlib.sha256((snap / name).read_bytes()).hexdigest() + '  ' + name + '\n'
                for name in ('README.md', 'files.sha256', 'source.tar.gz')))
            gate.verify_and_extract(snap, root / 'unpacked')
            self.assertEqual((root / 'unpacked/Cargo.toml').read_bytes(), payload)
            (snap / 'source.tar.gz').write_bytes(b'changed')
            with self.assertRaises(ValueError): gate.verify_and_extract(snap, root / 'tampered')

            for mode in ('debug', 'release'):
                argv = gate.test_command(mode)
                self.assertIn('--include-ignored', argv)
                self.assertIn('--test-threads=1', argv)
                self.assertEqual('--release' in argv, mode == 'release')

            graph = root / 'graph'; graph.mkdir()
            generations = []
            shapes = {'gt': ((72, 64), (69, 58), (69, 5)), 'bed': ((72, 64), (69, 58), (69, 5)),
                'pgen': ((72, 64), (69, 58), (69, 5)), 'ds': ((72, 24), (70, 24), (70, 2)),
                'hds': ((72, 24), (70, 24), (70, 2)), 'rich': ((72, 24), (70, 24), (70, 1))}
            for workers in (1, 4):
                (graph / str(workers)).mkdir()
                for lane, stages in shapes.items():
                    for stage, shape in zip(('I', 'F', 'P'), stages):
                        prefix = graph / str(workers) / f'{lane}.{stage}'
                        for extension in ('pgen', 'pvar', 'psam', 'rsomics.json', 'raw.json'):
                            Path(str(prefix) + '.' + extension).write_text('transport-only\n')
                        generations.append(dict(workers=workers, lane=lane, stage=stage,
                            prefix=str(prefix), shape=list(shape), expected='/original/answer-only'))
                    for stage in ('F', 'P'):
                        for extension in ('acount', 'gcount', 'scount', 'smiss', 'vmiss', 'het', 'hwe-log.hardy') + (() if lane == 'rich' else ('hardy',)):
                            (graph / str(workers) / f'{lane}.{stage}.qc.{extension}').write_text('transport-only\n')
                    for extension in ('ld.vcor', 'prune.prune.in', 'prune.prune.out'):
                        (graph / str(workers) / f'{lane}.F.{extension}').write_text('transport-only\n')
                for extension in ('hardy', 'hwe-log.hardy'):
                    (graph / str(workers) / f'tail.{extension}').write_text('transport-only\n')
                for call in [*(f'{lane}.{name}' for lane in shapes
                                for name in ('F.stats', 'P.stats', 'F.pairs', 'F.prune')), 'tail.stats']:
                    directory = graph / str(workers) / 'calls' / call
                    directory.mkdir(parents=True)
                    field = 'effective_worker_limit' if call.endswith('.stats') else 'effective_workers'
                    (directory / 'stdout.txt').write_text(json.dumps(dict(result={
                        'requested_workers': workers, 'writer_workers': 1, field: workers})))
            record = dict(status='complete_compact_native_graph_passed', generations=generations,
                principal_report_files=114, raw_cells_checked=74878)
            (graph / 'native-graph.json').write_text(json.dumps(record))
            mapped = gate.bind_graph(graph)
            self.assertEqual(len(mapped['generations']), 36)
            self.assertEqual(mapped['generations'][0]['prefix'], '1/gt.I')
            self.assertEqual(mapped['generations'][-1]['prefix'], '4/rich.P')
            self.assertEqual(len(gate.worker_provenance(graph, 'green-2')), 50)
            for workers in (1, 4):
                for extension in ('hardy', 'hwe-log.hardy'):
                    (graph / str(workers) / f'rounding.qc.{extension}').write_text('transport-only\n')
                directory = graph / str(workers) / 'calls/rounding.stats'
                directory.mkdir(parents=True)
                (directory / 'stdout.txt').write_text(json.dumps(dict(result={
                    'requested_workers': workers, 'writer_workers': 1, 'effective_worker_limit': workers})))
            record['principal_report_files'] = 116
            (graph / 'native-graph.json').write_text(json.dumps(record))
            mapped = gate.bind_graph(graph, 'green-3')
            self.assertEqual(mapped['principal_report_files_per_worker'], 116)
            self.assertEqual(len(mapped['reports']), 232)
            self.assertEqual(len(gate.worker_provenance(graph, 'green-3')), 52)
            for selector in ('green-1', 'green-2', 'green-99'):
                with self.assertRaises(ValueError): gate.bind_graph(graph, selector)
                with self.assertRaises(ValueError): gate.worker_provenance(graph, selector)
            record['generations'][0]['prefix'] = str(root / 'outside')
            (graph / 'native-graph.json').write_text(json.dumps(record))
            with self.assertRaises(ValueError): gate.bind_graph(graph, 'green-3')

            bodies_by_name = {'runtime/source/plink2': b'not an executable\n',
                              'case/A': b'upper\n', 'case/a': b'lower\n'}
            inventory = {name: dict(kind='file', bytes=len(body), sha256=hashlib.sha256(body).hexdigest())
                         for name, body in bodies_by_name.items()}
            inventory['case/link'] = dict(kind='symlink', target='A')
            capture = json.dumps(dict(schema=1, files=inventory)).encode()
            carrier = root / 'capture.tar'
            with tarfile.open(carrier, 'w') as archive:
                for name, body in {**bodies_by_name, 'CAPTURE.json': capture}.items():
                    member = tarfile.TarInfo('plink-first-workflow-oracle/' + name)
                    member.size = len(body); archive.addfile(member, io.BytesIO(body))
                member = tarfile.TarInfo('plink-first-workflow-oracle/case/link')
                member.type = tarfile.SYMTYPE; member.linkname = 'A'; archive.addfile(member)
            restored = root / 'restored'
            proof = source.verify_restore(carrier, restored, hashlib.sha256(capture).hexdigest(), 4,
                                          lambda name: name == 'runtime/source/plink2')
            self.assertEqual(proof['verified_entries'], 4)
            self.assertEqual((restored / 'runtime/source/plink2').read_bytes(), b'not an executable\n')
            self.assertFalse((restored / 'case').exists())
            with self.assertRaises(ValueError):
                source.verify_restore(carrier, root / 'wrong-capture', '0' * 64, 4, lambda name: False)

            native_raw, bridge_raw = root / 'native.json', root / 'bridge.jsonl'
            raw_rows = [dict(variant=0, provisional_ref=True, calls=[[0, 1, 1, 0, 17000, -12]])]
            native_raw.write_text(json.dumps(raw_rows))
            header = dict(schema=1, samples=1, variants=1, source='cc2c95e61c509d49aaf884d2ec0956a921770745',
                          semantics='raw codec; chromosome correction belongs to caller')
            bridge_raw.write_text(json.dumps(header) + '\n' + json.dumps(raw_rows[0]) + '\n')
            self.assertEqual(source.compare_raw(native_raw, bridge_raw, [1, 1]), 1)
            raw_rows[0]['calls'][0][-1] = None
            native_raw.write_text(json.dumps(raw_rows))
            with self.assertRaises(ValueError): source.compare_raw(native_raw, bridge_raw, [1, 1])

            plan = source.report_plan(root / 'native', root / 'owner', root / 'observed', 4)
            self.assertEqual(len(plan), 56)
            rich_qc = next(item for item in plan if item['name'] == '4/rich/F/qc')
            self.assertEqual(rich_qc['args'][0:2], ['--pfile', str(root / 'native/4/rich.F')])
            self.assertNotIn('--hardy', rich_qc['args'])
            rich_hwe = next(item for item in plan if item['name'] == '4/rich/F/hwe-log')
            self.assertIn('--autosome', rich_hwe['args'])
            self.assertIn('--max-alleles', rich_hwe['args'])
            tail = next(item for item in plan if item['name'] == '4/tail/raw')
            self.assertIn(str(root / 'native/4/gt.I'), tail['args'])
            self.assertIn(str(root / 'owner/tests/fixtures/inputs/founders64.id'), tail['args'])

            evidence = root / 'evidence'; evidence.mkdir()
            (evidence / 'useful.txt').write_text('retained transport output\n')
            gate.package(evidence, root / 'evidence.tar')
            delivered = root / 'delivered'
            source.extract_artifact(root / 'evidence.tar', delivered)
            self.assertEqual((delivered / 'useful.txt').read_text(), 'retained transport output\n')
            blocked = subprocess.run([sys.executable, '-B', str(ROOT / 'scripts/plink_source_on_native.py'),
                '--native-archive', str(carrier), '--native-output', str(root / 'blocked-native'),
                '--work', str(root / 'blocked-work'), '--output', str(root / 'blocked-source')],
                env=dict(os.environ, GITHUB_ACTIONS='false'), text=True, capture_output=True)
            self.assertNotEqual(blocked.returncode, 0)
            self.assertIn('hosted Linux', blocked.stderr)
            self.assertFalse((root / 'blocked-work').exists())
            self.assertFalse((root / 'blocked-source').exists())


if __name__ == '__main__':
    unittest.main()
