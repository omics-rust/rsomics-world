"""One connected preparation contract; no native/source scientific execution."""

import copy
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
import zipfile

from test_infercnv_ingestion_workflow import bash_body, steps


ROOT = Path(__file__).resolve().parents[1]


class CompleteJunctionPerformancePreparation(unittest.TestCase):
    def test_connected_adopted_whole_campaign_and_failure_contract(self):
        try:
            perf = importlib.import_module('rnaseq_qc_junction_performance')
        except ModuleNotFoundError:
            self.fail('the complete junction performance controller is missing')
        policy = perf.read_policy()
        graph = perf.schedule()
        self.assertEqual(len(graph), 21)
        triples = [[(r['engine'], r['workers']) for r in graph[i:i + 3]] for i in range(0, 21, 3)]
        first = [('source', 1), ('native', 1), ('native', 4)]
        reverse = list(reversed(first))
        self.assertEqual(triples, [first, reverse, first, reverse, first, reverse, first])
        self.assertEqual([row['phase'] for row in graph[:6]], ['qualification'] * 3 + ['warmup'] * 3)
        self.assertEqual([row['pair'] for row in graph[6:]], sum(([n] * 3 for n in range(1, 6)), []))
        self.assertEqual(policy['input']['artifact_id'], 12781035705)
        self.assertEqual(policy['input']['uncompressed_bytes'], 758470501)
        self.assertEqual(len(policy['input']['files']), 521)
        self.assertEqual(policy['native']['artifact_id'], 13074285515)
        self.assertEqual(policy['native']['binary_sha256'], 'eb472af80dfa26a00819448f895ccbcceb570de3ad629ac81f9507c7534282ae')
        self.assertEqual(len(policy['native']['files']), 226)
        self.assertEqual(perf.budget_plan()['science_bytes'], 1145336745)
        self.assertEqual(perf.budget_plan()['allocated_bytes'], 5086763584)
        self.assertEqual(perf.budget_plan()['peak_project_bytes'], 26848002080)
        self.assertLess(perf.budget_plan()['peak_project_bytes'], policy['limits']['initial_free_bytes'])
        self.assertEqual(policy['limits']['source_seconds'] * 7 + policy['limits']['native_seconds'] * 14, 8400)
        perf.verify_context(ROOT, policy)
        self.assertEqual(perf.capture.__module__, 'ecology_beta_real')
        commands = perf.command_plan(Path('/bounded/private'), Path('/exact/accepted/elf'), Path('/admitted/venv/bin/python3'))
        self.assertEqual(len(commands), 21)
        for row in commands:
            argv = row['argv']
            if row['engine'] == 'source':
                self.assertEqual(row['timeout'], 600)
                self.assertTrue({'--skip-plot', '--skip-bed', '--skip-interact'} <= set(argv))
                self.assertIn('junction_annotation.py', ' '.join(argv))
            else:
                self.assertEqual(row['timeout'], 300)
                self.assertEqual(argv[0], '/exact/accepted/elf')
                self.assertEqual(argv[1], 'junctions')
                self.assertEqual(argv[argv.index('--threads') + 1], str(row['workers']))
            self.assertNotIn('report', argv)
            self.assertNotIn('trace_eligibility', argv)
        outcomes = [dict(row, wall=10 if row['engine'] == 'source' else 7 if row['workers'] == 1 else 6,
            rss_bytes=1000, cpu=9, returncode=0, compatible=True, group_empty=True,
            raw_wait4=dict(user_seconds=8, system_seconds=1, rss_bytes=1000, input_blocks=0,
                           output_blocks=2, minor_faults=3, major_faults=0)) for row in graph]
        self.assertTrue(perf.evaluate(outcomes)['passed'])
        changed = copy.deepcopy(outcomes)
        for row in changed:
            if row['engine'] == 'native':
                row['wall'] = 9 if row['workers'] == 1 else .01
        self.assertFalse(perf.evaluate(changed)['passed'])
        for row in changed:
            if row['engine'] == 'native' and row['workers'] == 1:
                row.update(wall=10.5, rss_bytes=750)
        self.assertTrue(perf.evaluate(changed)['memory'])
        for row in changed:
            if row['engine'] == 'native' and row['workers'] == 1:
                row['wall'] = 10.50001
        self.assertFalse(perf.evaluate(changed)['passed'])
        for mutation in ('missing', 'source_failure', 'correctness', 'leftover'):
            altered = copy.deepcopy(outcomes)
            if mutation == 'missing':
                altered.pop()
            elif mutation == 'source_failure':
                altered[0]['returncode'] = 1
            elif mutation == 'correctness':
                altered[0]['compatible'] = False
            else:
                altered[0]['group_empty'] = False
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                perf.evaluate(altered)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture = root / 'source-fixture'
            fixture.mkdir()
            snapshot = ROOT / '.autopilot/snapshots/rnaseq-qc-junctions-2026-11-01/green-1/source.tar.gz'
            with tarfile.open(snapshot) as archive:
                for old, new in [('observed.junction.xls', 'observed.junction.xls'),
                                 ('process.stdout', 'stdout.txt'), ('process.stderr', 'stderr.txt')]:
                    member = 'tests/fixtures/junctions/informative-single/' + old
                    (fixture / new).write_bytes(archive.extractfile(member).read())
            (fixture / 'observed.junction_plot.r').write_text('events=c(1,2,3)\njunction=c(1,2,3)\n')
            expected = perf.source_result(fixture)
            self.assertEqual(expected['counts'], dict(total_events=11, known_events=6, partial_novel_events=1,
                novel_events=3, filtered_events=1, total_junctions=6, known_junctions=2,
                partial_novel_junctions=1, novel_junctions=3))
            self.assertEqual([(r['start'], r['end'], r['read_count']) for r in expected['rows']],
                [(120,170,5),(120,270,1),(120,171,1),(190,240,1),(320,370,1),(522,572,1)])
            native = root / 'native'
            native.mkdir()
            report = dict(schema_version='1.0', profile='rseqc-5.0.5', sample='paired', metrics=['junctions'],
                          junctions={k: copy.deepcopy(expected[k]) for k in ('rows', 'counts')})
            def write_native(value):
                (native / 'report.json').write_text(json.dumps(value))
                (native / 'junctions.tsv').write_text('chrom\tintron_start\tintron_end\tread_count\tannotation\n' + ''.join(
                    '\t'.join(str(row[k]) for k in ('chrom','start','end','read_count','annotation')) + '\n'
                    for row in value['junctions']['rows']))
                (native / 'junctions-summary.tsv').write_text('metric\tvalue\n' + ''.join(
                    f'{key}\t{number}\n' for key, number in value['junctions']['counts'].items()))
                outputs = [dict(path=n, bytes=(native/n).stat().st_size, sha256=hashlib.sha256((native/n).read_bytes()).hexdigest())
                           for n in ('report.json','junctions.tsv','junctions-summary.tsv')]
                (native / 'manifest.json').write_text(json.dumps(dict(status='complete', profile='rseqc-5.0.5',
                    selected_metrics=['junctions'], omitted_metrics=['mapping'], bam_traversals=1,
                    original_records=9810963, inflate_workers=1, outputs=outputs,
                    policy=dict(junctions=dict(excluded_flags=0x704, minimum_mapq=30, minimum_intron=50,
                        omitted_outputs=['R','PDF','BED12','Interact','junction-saturation'])))))
            write_native(report)
            perf.native_result(native, expected, 1)
            for mutation in ('order','support','class','counter'):
                value = copy.deepcopy(report)
                if mutation == 'order': value['junctions']['rows'].reverse()
                elif mutation == 'support': value['junctions']['rows'][0]['read_count'] += 1
                elif mutation == 'class': value['junctions']['rows'][0]['annotation'] = 'complete_novel'
                else: value['junctions']['counts']['filtered_events'] = 0
                write_native(value)
                with self.subTest(output_mutation=mutation), self.assertRaises(ValueError):
                    perf.native_result(native, expected, 1)
            payload = root / 'carrier.zip'
            with zipfile.ZipFile(payload, 'w') as archive:
                archive.writestr('original/input.bin', b'full test input\n')
            pin = dict(bytes=payload.stat().st_size, sha256=hashlib.sha256(payload.read_bytes()).hexdigest(),
                uncompressed_bytes=16, files={'original/input.bin': dict(bytes=16, sha256=hashlib.sha256(b'full test input\n').hexdigest())})
            perf.extract_original(payload, root / 'extracted', pin)
            self.assertEqual((root / 'extracted/original/input.bin').read_bytes(), b'full test input\n')
            with zipfile.ZipFile(root / 'unsafe.zip', 'w') as archive:
                archive.writestr('../outside', b'x')
            unsafe = root / 'unsafe.zip'
            bad_pin = dict(pin, bytes=unsafe.stat().st_size, sha256=hashlib.sha256(unsafe.read_bytes()).hexdigest())
            with self.assertRaises(ValueError): perf.extract_original(unsafe, root / 'bad', bad_pin)
            self.assertFalse((root / 'outside').exists())
            calls = root / 'recorder'
            calls.mkdir()
            captured = perf.capture(calls, 'stdlib-only', [sys.executable, '-B', '-c', 'print("retained recorder contract")'],
                                    workers=1, timeout=10, private=None, file_limit=4096)
            self.assertEqual(captured['returncode'], 0)
            self.assertGreater(captured['wall'], 0)
            self.assertIn('user_seconds', captured['raw_wait4'])
            self.assertIn('rss_bytes', captured['raw_wait4'])
            with self.assertRaises(RuntimeError):
                perf.capture(calls, 'failed-stdlib-only', [sys.executable, '-B', '-c', 'print("original failure");raise SystemExit(7)'],
                             workers=1, timeout=10, private=None, file_limit=4096)
            failure = json.loads((calls / 'failed-stdlib-only/status.json').read_text())
            self.assertEqual(failure['returncode'], 7)
            self.assertIn(b'original failure', (calls / 'failed-stdlib-only/stdout.txt').read_bytes())
            private = root / 'private'
            private.mkdir()
            (private / 'execution-status.json').write_text('{"status":"HOLD","source_failure_is_not_native_win":true}\n')
            members = perf.seal_manifest(private, policy, bindings={})
            archive_manifest = json.loads((private / 'artifact-manifest.json').read_text())
            self.assertEqual(archive_manifest['execution']['status'], 'HOLD')
            self.assertTrue({'artifact-manifest.json','execution-status.json'} <= {r['name'] for r in members})
            self.assertLess(perf.tar_bound(members), policy['limits']['tar_bytes'])
        workflow = (ROOT / '.github/workflows/rnaseq-qc-junction-performance.yml').read_text()
        self.assertRegex(workflow, r'recovery_admitted:\n(?:        .*\n)*?        default: false\n')
        self.assertIn('timeout-minutes: 240', workflow)
        self.assertIn('--cpus=4 --memory=8g --memory-swap=8g', workflow)
        self.assertNotIn('cargo build', workflow)
        self.assertNotIn('actions/setup-python', workflow)
        uploaded = steps(workflow)['Upload ciphertext and allowlisted transport receipt only']
        self.assertIn('/cipher/evidence.tar.age', uploaded)
        self.assertIn('/cipher/transport.json', uploaded)
        self.assertNotIn('/private', uploaded)
        for name, step in steps(workflow).items():
            if '        run: |\n' in step:
                body = re.sub(r'\$\{\{.*?\}\}', 'stub', bash_body(step))
                parsed = subprocess.run(['bash', '-n'], input=body, capture_output=True, text=True)
                self.assertEqual(parsed.returncode, 0, (name, parsed.stderr))


if __name__ == '__main__':
    unittest.main()
