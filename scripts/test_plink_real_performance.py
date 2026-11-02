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
    def test_qualified_repeat_reuses_only_exact_own_role_content_and_retains_failures(self):
        real = importlib.import_module('plink_real_performance')
        self.assertTrue(callable(getattr(real, 'freeze_qualification', None)),
                        'own-role qualification certificates are not implemented')
        with tempfile.TemporaryDirectory(prefix='plink-qualified-repeat-', dir=os.environ['TMPDIR']) as name:
            root = Path(name)
            def put(path, value):
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(json.dumps(value))
            def inert(path):
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b'inert transport body, never executed\n')
                return path
            runtime = dict(native=str(root / 'native'), restored=str(root / 'restored'),
                           binary=str(inert(root / 'native/binary-release')),
                           observer=str(inert(root / 'observer')))
            for tool in ('source', 'official'):
                inert(Path(runtime['restored']) / real.source.ENGINES[tool][0])
            library = inert(root / 'image/library')
            alias = root / 'image/library-alias'; alias.symlink_to(library.name)
            library_identity = real.file_identity(library)
            put(Path(runtime['restored']) / 'runtime/loaded-libraries.json',
                {str(alias): dict(realpath=str(library), retained='library', sha256=library_identity['sha256'])})
            put(root / 'evidence/native-elf/admission.json', dict(interpreter=str(alias),
                binary_sha256=real.file_identity(runtime['binary'])['sha256'],
                providers={'fixture': dict(path=str(library), sha256=library_identity['sha256'])}))
            put(root / 'admitted-runtime.json', runtime)
            put(root / 'evidence/controller.json', dict(commit=os.environ.get('GITHUB_SHA'), image=real.oracle.IMAGE,
                files={path: real.oracle.sha256(real.ROOT / path) for path in set(real.CONTROLLERS)}))
            inputs = root / 'prepared/INPUT'
            for extension in ('pgen', 'pvar', 'psam'):
                inert(Path(str(inputs) + '.' + extension))
            put(root / 'prepared-input.json', dict(prefix=str(inputs), shape=[64, 64],
                files={ext: real.file_identity(str(inputs) + '.' + ext) for ext in ('pgen', 'pvar', 'psam')}))
            put(root / 'evidence/input.raw.jsonl.gz.json', dict(fixture='completed input capture record'))
            bodies = [stage + '.' + ext for stage in ('I', 'F', 'P') for ext in ('pgen', 'pvar', 'psam')]
            bodies += [stage + '.qc.' + ext for stage in ('F', 'P') for ext in
                       ('acount', 'gcount', 'scount', 'smiss', 'vmiss', 'het', 'hardy', 'hwe-log.hardy')]
            bodies += ['pairs.vcor', 'prune.prune.in', 'prune.prune.out']
            def full_check_records(directory):
                put(directory / 'whole-check.json', dict(original_input=str(inputs), own_projection=True,
                    fresh_P_validated=True, exact_raw={stage: dict(fixture='original full raw proof') for stage in ('I', 'F', 'P')}))
                for stage in ('I', 'F', 'P'):
                    put(directory / (stage + '.raw.jsonl.gz.json'), dict(fixture='completed capture record'))
                for engine in ('source', 'official'):
                    put(directory / ('fresh-P-' + engine) / 'result.json', dict(status='returned', returncode=0))
            def generation(directory, tool, workers, qualification=False):
                products = directory / 'products'; products.mkdir(parents=True)
                for relative in bodies + ([stage + '.rsomics.json' for stage in ('I', 'F', 'P')] if tool == 'native' else []):
                    (products / relative).write_bytes(('unchanged synthetic payload ' + relative).encode())
                binary = runtime['binary'] if tool == 'native' else str(Path(runtime['restored']) / real.source.ENGINES[tool][0])
                stages = real.graph(tool, workers, inputs, products, binary)
                put(directory / 'plan.json', dict(tool=tool, workers=workers, stages=stages))
                measured = []
                for command in stages:
                    row = dict(name=command['name'], group=command['group'], argv=command['argv'], exit=0,
                               wall_seconds=1.0, user_seconds=0.5, system_seconds=0.1)
                    measured.append(row)
                    put(directory / 'commands' / command['name'] / 'measurement.json', row)
                    (directory / 'commands' / command['name'] / 'stderr.txt').write_text('')
                    put(directory / 'commands' / command['name'] / 'stdout.txt', dict(result=dict(
                        samples=64, variants=64, founders=64, requested_workers=workers, writer_workers=1,
                        effective_worker_limit=workers, effective_workers=workers)))
                files = {stage: {str(products / (stage + '.' + ext)): real.file_identity(products / (stage + '.' + ext))
                         for ext in ('pgen', 'pvar', 'psam')} for stage in ('I', 'F', 'P')}
                custody = dict(creation_files=files, unchanged_after_descendants=True,
                               audit_included_in_outer_wall=True, audit_excluded_from_scientific_cli_times=True)
                put(directory / 'generation-custody.json', custody)
                put(directory / 'complete-measurement.json', dict(tool=tool, workers=workers, exit=0,
                    failure=None, stages=measured, generation_custody=custody,
                    wall_seconds=8.0, scientific_cli_wall_seconds=7.0))
                if qualification:
                    full_check_records(directory)
                return directory
            certificates = {}
            for workers in (1, 4):
                for tool in ('source', 'native', 'official'):
                    directory = generation(root / 'evidence/qualification' / str(workers) / tool, tool, workers, True)
                    certificates[workers, tool] = real.freeze_qualification(root, directory, runtime, inputs, [0, 1, 2, 3])
            for workers in (1, 4):
                for tool in ('source', 'native'):
                    for pair in range(5):
                        directory = generation(root / 'evidence/trials' / str(workers) / str(pair) / tool, tool, workers)
                        proof = real.check_qualified_repeat(root, directory, certificates[workers, tool], runtime, inputs, [0, 1, 2, 3])
                        self.assertEqual(proof['validation_kind'], 'qualified-content-reused')
                        self.assertEqual(len(proof['scientific_payload']), 31 if tool == 'native' else 28)
                        self.assertEqual(proof['fresh_observer_captures'], 0)
                        self.assertEqual(proof['fresh_P_validations'], 0)
                        self.assertFalse((directory / 'whole-check.json').exists())
                        self.assertFalse((directory / 'I.raw.jsonl.gz').exists())
            counts = real.validation_counts(root)
            self.assertEqual(counts['observer_captures_completed'], 19)
            self.assertEqual(counts['fresh_P_validations_completed'], 12)
            self.assertEqual(counts['qualified_content_reused'], 20)
            route = bash_body(steps((ROOT / '.github/workflows/plink-real-performance.yml').read_text())[
                'Run fixed real whole workflow with private scientific logs'])
            for enabled in ('false', 'true'):
                directory = root / ('workflow-' + enabled); directory.mkdir()
                record = directory / 'argv.txt'
                result = subprocess.run(['bash', '-c', 'python3() { printf "%s\\n" "$@" > "$ARG_RECORD"; };\n' + route],
                    env=dict(os.environ, REUSE_QUALIFIED_BODIES=enabled, ARG_RECORD=str(record),
                        PRIVATE_CAMPAIGN=str(directory), GITHUB_WORKSPACE=str(ROOT), NATIVE_RECEIPT=real.DEFAULT_RECEIPT),
                    capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(record.read_text().splitlines().count('--reuse-qualified-bodies'), int(enabled == 'true'))
            wrong = generation(root / 'wrong-role', 'native', 4)
            with self.assertRaisesRegex(ValueError, 'role|context'):
                real.check_qualified_repeat(root, wrong, certificates[1, 'native'], runtime, inputs, [0, 1, 2, 3])
            changed = generation(root / 'changed-body', 'native', 1)
            (changed / 'products/F.qc.het').write_bytes(b'different exact body\n')
            self.assertIsNone(real.check_qualified_repeat(root, changed, certificates[1, 'native'], runtime, inputs, [0, 1, 2, 3]))
            difference = real.load(changed / 'qualified-content-difference.json')
            self.assertEqual(set(difference['differences']), {'F.qc.het'})
            self.assertFalse((changed / 'qualified-content-reused.json').exists())
            with self.assertRaises((OSError, ValueError)):
                real.record_qualified_fallback(changed)
            full_check_records(changed)
            fallback = real.record_qualified_fallback(changed)
            self.assertEqual(fallback['validation_kind'], 'full-check-after-body-difference')
            self.assertEqual(fallback['fresh_observer_captures'], 3)
            self.assertEqual(fallback['fresh_P_validations'], 2)
            self.assertFalse(fallback['content_proof_reused'])
            self.assertFalse((changed / 'qualified-content-reused.json').exists())
            drift = generation(root / 'changed-context', 'native', 1)
            plan = real.load(drift / 'plan.json'); plan['stages'][0]['argv'][5] = '2049'
            put(drift / 'plan.json', plan)
            with self.assertRaisesRegex(ValueError, 'context|plan|execution'):
                real.check_qualified_repeat(root, drift, certificates[1, 'native'], runtime, inputs, [0, 1, 2, 3])
            missing = generation(root / 'missing-body', 'native', 1)
            (missing / 'products/P.rsomics.json').rename(missing / 'products/retained-sidecar')
            with self.assertRaises((OSError, ValueError)):
                real.check_qualified_repeat(root, missing, certificates[1, 'native'], runtime, inputs, [0, 1, 2, 3])
            library.write_bytes(b'changed runtime provider\n')
            with self.assertRaisesRegex(ValueError, 'runtime|provider|context'):
                real.check_qualified_repeat(root, wrong, certificates[4, 'native'], runtime, inputs, [0, 1, 2, 3])

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
            executable_body = b'transport permission fixture, not an executable program\n'
            files = {'candidate/binary-release': dict(bytes=len(executable_body),
                     sha256=hashlib.sha256(executable_body).hexdigest())}
            native_archive = root / 'native.tar'
            with tarfile.open(native_archive, 'w') as archive:
                for relative, body, mode in [('candidate/binary-release', executable_body, 0o755),
                        ('artifact-manifest.json', json.dumps(dict(schema=real.source.SCHEMA, files=files)).encode(), 0o644)]:
                    entry = tarfile.TarInfo('plink-evidence/' + relative)
                    entry.size, entry.mode = len(body), mode
                    archive.addfile(entry, io.BytesIO(body))
            real.source.extract_artifact(native_archive, root / 'restored-native')
            self.assertEqual((root / 'restored-native/candidate/binary-release').stat().st_mode & 0o777, 0o755)
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
            updated, _ = real.committed_receipt(ROOT, '.autopilot/oracles/plink-native-2026-11-01-green-3.json')
            self.assertEqual(updated['snapshot'], 'green-3')
            self.assertEqual(updated['source_file_count'], 303)
            self.assertEqual(updated['principal_reports_per_worker'], 116)
            packed, _ = real.committed_receipt(ROOT, '.autopilot/oracles/plink-native-2026-11-01-green-4.json')
            self.assertEqual(packed['snapshot'], 'green-4')
            self.assertEqual(packed['owner_commit'], 'e03ada173d8dabb1c7894e2a0e53479d1cd41fa6')
            self.assertEqual(packed['binary_sha256'], '032ea39a28846dbe145445662e8534717a0a36035e2cb64a0bbf0626494b9283')
            self.assertEqual(packed['source_file_count'], 303)
            self.assertEqual(packed['principal_reports_per_worker'], 116)
            prepared, _ = real.committed_receipt(ROOT, '.autopilot/oracles/plink-native-2026-11-02-green-7.json')
            self.assertEqual(prepared['snapshot'], 'green-7')
            self.assertEqual(prepared['owner_commit'], '7b21443622321b3bd9adeb5059e3035de69c4ea4')
            self.assertEqual(prepared['binary_sha256'], '76a81eee24c7425615473d542413080d790a907f438100723a092577c0cfbc41')
            self.assertEqual(prepared['principal_reports_per_worker'], 123)
            self.assertEqual(prepared['worker_provenance_records_per_mode'], 66)
            qc, _ = real.committed_receipt(ROOT, '.autopilot/oracles/plink-native-2026-11-02-green-8.json')
            self.assertEqual(qc['snapshot'], 'green-8')
            self.assertEqual(qc['owner_commit'], 'f49f3675184801bb1e2cbf935f0ecadbc6a22e84')
            self.assertEqual(qc['binary_sha256'], 'd9c448e30f988b23f5dd3df025674483d469f9812d37e58712c8c55cd19ec882')
            self.assertEqual(qc['library_tests_per_mode'], 5)
            self.assertEqual(qc['principal_reports_per_worker'], 123)
            self.assertEqual(qc['worker_provenance_records_per_mode'], 66)
            ordered, _ = real.committed_receipt(ROOT, '.autopilot/oracles/plink-native-2026-11-02-green-9.json')
            self.assertEqual(ordered['snapshot'], 'green-9')
            self.assertEqual(ordered['owner_commit'], '80fafbf8eaed950f03d7fed88dd641d58ca99882')
            self.assertEqual(ordered['binary_sha256'], '9a5a427d1881103be4577434c8306c43bfa04817dbf1c954d9de2c557ee7a22a')
            self.assertEqual(ordered['source_file_count'], 305)
            self.assertEqual(ordered['library_tests_per_mode'], 7)
            self.assertEqual(ordered['principal_reports_per_worker'], 123)
            self.assertEqual(ordered['worker_provenance_records_per_mode'], 66)
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
            self.assertIn('${{ env.PRIVATE_DELIVERY }}/hgdp.tar.age', paths)
            self.assertIn('${{ env.PRIVATE_DELIVERY }}/receipt.json', paths)
            self.assertNotIn('runner.temp', paths)
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
