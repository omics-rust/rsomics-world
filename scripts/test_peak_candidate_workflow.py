"""Stdlib preparation only; never native or upstream scientific admission."""

import ast
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
WORKFLOW = ROOT / '.github/workflows/peak-candidate.yml'


class PeakCandidatePreparationTest(unittest.TestCase):
    def test_native_four_transport_and_same_native_consumer_contract(self):
        self.assertTrue(WORKFLOW.is_file(), 'complete peak native-four/consumer gate is absent')
        candidate = importlib.import_module('peak_candidate')
        consumer = importlib.import_module('peak_source_on_native')
        self.assertEqual(candidate.TEST_POPULATIONS,
                         {'green-1': (6, 3, 1), 'green-2': (7, 3, 1), 'green-3': (7, 3, 1)})
        text = WORKFLOW.read_text()
        routes = json.loads(re.search(r"fromJSON\('([^']+)'\)", text)[1])
        self.assertEqual([(r['target'], r['runner'], r['system'], r['arch']) for r in routes], [
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
        for name in ('peak_candidate.py', 'peak_source_on_native.py'):
            ast.parse((ROOT / 'scripts' / name).read_text())
        self.assertTrue((ROOT / 'scripts/peak_source_on_native.R').is_file())
        with tempfile.TemporaryDirectory(prefix='peak-gate-preparation-') as directory:
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
            self.assertTrue(all(Path(path).is_relative_to(runner.resolve())
                                for path in candidate.storage(env, require_space=0)['paths'].values()))
            with self.assertRaises(ValueError):
                candidate.storage(dict(env, TMPDIR=str(root)), require_space=0)
            for selector in ('missing', '../green-1', 'green-0', 'green-1/extra'):
                with self.assertRaises((ValueError, OSError)):
                    candidate.prepare_source(ROOT, root / selector.replace('/', '_'), selector)

            body = b'literal transport fixture, never a scientific answer\n'
            manifest = {'inputs/literal.txt': dict(bytes=len(body), sha256=hashlib.sha256(body).hexdigest())}
            archive = root / 'regular.tar.gz'
            with tarfile.open(archive, 'w:gz') as stream:
                item = tarfile.TarInfo('inputs/literal.txt'); item.size = len(body)
                stream.addfile(item, io.BytesIO(body))
            candidate.extract_regular(archive, root / 'regular', manifest)
            self.assertEqual((root / 'regular/inputs/literal.txt').read_bytes(), body)
            for name in ('../escape', 'inputs/literal.txt'):
                bad = root / ('bad-' + str(len(name)) + '.tar.gz')
                with tarfile.open(bad, 'w:gz') as stream:
                    item = tarfile.TarInfo(name); item.size = 1
                    stream.addfile(item, io.BytesIO(b'x'))
                with self.assertRaises(ValueError):
                    candidate.extract_regular(bad, root / ('bad-' + str(len(name))), manifest)

            snap = root / 'snapshots/green-1'; snap.mkdir(parents=True)
            source = {'Cargo.toml': b'[package]\nname="transport-only"\n'}
            with tarfile.open(snap / 'source.tar.gz', 'w:gz') as stream:
                item = tarfile.TarInfo('Cargo.toml'); item.size = len(source['Cargo.toml'])
                stream.addfile(item, io.BytesIO(source['Cargo.toml']))
            (snap / 'README.md').write_text('Transport-only source fixture.\n')
            (snap / 'files.sha256').write_text(hashlib.sha256(source['Cargo.toml']).hexdigest() + '  Cargo.toml\n')
            (snap / 'checksums.sha256').write_text(''.join(
                hashlib.sha256((snap / name).read_bytes()).hexdigest() + '  ' + name + '\n'
                for name in ['README.md', 'files.sha256', 'source.tar.gz']))
            candidate.verify_and_extract(snap, root / 'unpacked-source')
            self.assertEqual((root / 'unpacked-source/Cargo.toml').read_bytes(), source['Cargo.toml'])

            test_env = candidate.fixture_environment(root / 'candidate', root / 'scratch')
            self.assertEqual({test_env[name] for name in ['RSOMICS_PEAK_FIXTURES',
                'RSOMICS_PEAK_ORACLE', 'RSOMICS_PEAK_SOURCE_FIXTURE']}, {str(root / 'candidate/fixtures')})
            self.assertEqual(test_env['RSOMICS_PEAK_SOURCE_CALLS'], str(root / 'candidate/fixtures/witness/calls'))
            for mode in ('debug', 'release'):
                argv = candidate.test_command(mode)
                self.assertIn('--include-ignored', argv)
                self.assertIn('--test-threads=1', argv)
                self.assertEqual('--release' in argv, mode == 'release')

            called, sparse = root / 'called.narrowPeak', root / 'sparse.bed'
            called.write_text('chrA\t100\t120\tdisplay\t12\t.\t1\t2\t3\t4\n')
            sparse.write_text('chrA\t200\t220\t40\t2\tchrA:200-220\n')
            candidate.native_rows(called, sparse, root / 'rows')
            rows = (root / 'rows/workflow.bed').read_text().splitlines()
            self.assertEqual(rows, ['chrA\t100\t120\tcall-0\t0\t.',
                                    'chrA\t200\t220\tsparse-0\t0\t.',
                                    'chrA\t100\t120\tcall-0\t0\t.'])
            for workers in (1, 4):
                plan = candidate.worker_plan(root / 'fixture', root / str(workers), workers)
                self.assertEqual([row['name'] for row in plan],
                    ['call', 'sparse-seacr-1.3', 'sparse-corrected', 'annotate', 'count'])
                self.assertTrue(all(row['args'][-2:] == ['--threads', str(workers)] for row in plan))
                self.assertIn('seacr-1.3', plan[1]['args'])
                self.assertIn('corrected', plan[2]['args'])
                self.assertIn(str(root / str(workers) / 'rows/workflow.bed'), plan[-1]['args'])

            source_sparse = root / 'source-sparse.bed'
            source_sparse.write_text('chrA\t200\t220\t4e1\t2.0\tchrA:200-220\n')
            self.assertEqual(consumer.compare_sparse(sparse, source_sparse), 1)
            source_sparse.write_text('chrA\t200\t221\t40\t2\tchrA:200-220\n')
            with self.assertRaises(ValueError): consumer.compare_sparse(sparse, source_sparse)
            native_count, source_count = root / 'native.tsv', root / 'source.tsv'
            native_count.write_text('#chrom\tstart\tend\tfield4\tfield5\tfield6\tT1\tT2\n' +
                                    '\n'.join(row + '\t3\t4' for row in rows) + '\n')
            source_count.write_text('\n'.join(row + '\t3\t4' for row in rows) + '\n')
            self.assertEqual(consumer.compare_counts(native_count, source_count, root / 'rows/workflow.bed'), 3)
            source_count.write_text(rows[0] + '\t3\t5\n')
            with self.assertRaises(ValueError):
                consumer.compare_counts(native_count, source_count, root / 'rows/workflow.bed')

            annotation, core, detail = root / 'annotation.tsv', root / 'core.tsv', root / 'detail.tsv'
            annotation.write_text('row_id\tannotation\tgeneChr\tgeneStart\tgeneEnd\tgeneLength\tgeneStrand\tgeneId\ttranscriptId\tdistanceToTSS\tdetail\n' +
                ''.join(str(number) + '\tPromoter\tchrA\t1\t1000\t1000\t+\tg\tt\t0\t' +
                    '{"genic":true,"intergenic":false,"promoter":true,"five_utr":false,"three_utr":false,"exon":true,"intron":false,"downstream":false,"distal_intergenic":false}\n'
                    for number in range(3)))
            core.write_text('row_key\trow_ordinal\tannotation\tgeneChr\tgeneStart\tgeneEnd\tgeneLength\tgeneStrand\tgeneId\ttranscriptId\tdistanceToTSS\n' +
                ''.join(key + '\t' + str(number) + '\tPromoter\tchrA\t1\t1000\t1000\t+\tg\tt\t0\n'
                        for number, key in enumerate(['call:0', 'sparse:0', 'duplicate:0'])))
            detail.write_text('row_key\trow_ordinal\tgenic\tintergenic\tpromoter\tfive_utr\tthree_utr\texon\tintron\tdownstream\tdistal_intergenic\n' +
                ''.join(key + '\t' + str(number) + '\tTRUE\tFALSE\tTRUE\tFALSE\tFALSE\tTRUE\tFALSE\tFALSE\tFALSE\n'
                        for number, key in enumerate(['call:0', 'sparse:0', 'duplicate:0'])))
            self.assertEqual(consumer.compare_annotation(annotation, core, detail,
                root / 'rows/workflow-rows.tsv')['rows'], 3)
            detail.write_text(detail.read_text().replace('TRUE\tFALSE\tTRUE', 'FALSE\tFALSE\tTRUE'))
            with self.assertRaises(ValueError):
                consumer.compare_annotation(annotation, core, detail, root / 'rows/workflow-rows.tsv')

            blocked = subprocess.run([sys.executable, '-B', str(ROOT / 'scripts/peak_source_on_native.py'),
                '--native-archive', str(archive), '--native-output', str(root / 'blocked-native'),
                '--work', str(root / 'blocked-work'), '--output', str(root / 'blocked-source')],
                env=dict(os.environ, GITHUB_ACTIONS='false'), text=True, capture_output=True)
            self.assertNotEqual(blocked.returncode, 0)
            self.assertIn('hosted Linux', blocked.stderr)
            self.assertFalse((root / 'blocked-work').exists())
            self.assertFalse((root / 'blocked-source').exists())

            failed = root / 'failed'
            with self.assertRaises(RuntimeError):
                candidate.capture(failed, 'process', [sys.executable, '-B', '-c', 'print("original failure"); raise SystemExit(7)'])
            self.assertEqual(json.loads((failed / 'process/status.json').read_text())['status'], 7)
            self.assertIn('original failure', (failed / 'process/stdout.txt').read_text())
            evidence = root / 'evidence'; (evidence / 'candidate/fixtures').mkdir(parents=True)
            (evidence / 'candidate/fixtures/do-not-duplicate').write_bytes(body)
            (evidence / 'useful.txt').write_text('retained new useful output\n')
            candidate.package(evidence, root / 'evidence.tar')
            with tarfile.open(root / 'evidence.tar') as stream:
                self.assertEqual(sorted(member.name for member in stream),
                    ['evidence/artifact-manifest.json', 'evidence/useful.txt'])


if __name__ == '__main__':
    unittest.main()
