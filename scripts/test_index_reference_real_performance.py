import importlib
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))


class IndexReferenceRealPerformancePreparationTest(unittest.TestCase):
    def test_complete_reference_graph_retains_every_round_and_requires_a_real_advantage(self):
        self.assertTrue((ROOT / 'scripts/index_reference_real_performance.py').is_file(),
                        'complete reference performance controller is absent')
        controller = importlib.import_module('index_reference_real_performance')
        root = Path(tempfile.mkdtemp(prefix='index-real-performance-preparation-', dir=os.environ['TMPDIR']))
        data = root / 'data'
        data.mkdir()
        reference = data / 'reference.fa'
        reference.write_bytes(b'>1\n' + (b'A' * 60 + b'\n') * 33 + b'A' * 20 + b'\n'
                              + b'>2\n' + (b'c' * 60 + b'\n') * 25)
        fai = data / 'provider.fai'
        fai.write_bytes(b'1\t2000\t3\t60\t61\n2\t1500\t2040\t60\t61\n')
        rows = controller.parse_fai(fai)
        controller.verify_plain(reference, rows)
        reference.write_bytes(reference.read_bytes() + b'\n\n')
        controller.verify_plain(reference, rows)
        plan = controller.prepare_plan(root, rows,
            dict(plain=reference, bgzf=data / 'reference.fa.gz'),
            dict(source=Path('/oracle/samtools'), native=Path('/native/rsomics-index')))
        regions = (data / 'regions.txt').read_text().splitlines()
        self.assertEqual(len(regions), 10013)
        self.assertEqual(regions[0], '1:1-1000')
        self.assertEqual(regions[5000], '1:1001-2000')
        self.assertEqual(regions[-3:], ['1:1-1000'] * 3)
        self.assertEqual(len(plan), 168)
        self.assertEqual([p['engine'] for p in plan[48:72]], ['source'] * 12 + ['native'] * 12)
        self.assertEqual([p['engine'] for p in plan[72:96]], ['native'] * 12 + ['source'] * 12)
        self.assertEqual(len([p for p in plan if p['stage'].startswith('chr1')]), 56)
        outputs = [path for job in plan for path in job['outputs'].values()]
        self.assertEqual(len(outputs), len(set(outputs)))
        for job in plan:
            argv = list(map(str, job['argv']))
            if job['stage'] == 'build' or job['stage'] == 'dict':
                self.assertNotIn('--threads', argv)
                self.assertNotIn('-@', argv)
            else:
                flag = '-@' if job['engine'] == 'source' else '--threads'
                expected = job['workers'] - 1 if job['engine'] == 'source' else job['workers']
                self.assertEqual(argv[argv.index(flag) + 1], str(expected))
                self.assertIn('--fai-idx', argv)
                self.assertEqual('--gzi-idx' in argv, job['representation'] == 'bgzf')
        records = []
        for job in plan:
            for kind, path in job['outputs'].items():
                path.write_bytes(fai.read_bytes() if kind == 'fai' else (kind + '\n').encode())
            records.append(dict(**job, wall_seconds=10.0 if job['engine'] == 'source' else 5.0,
                user_seconds=1.0, system_seconds=0.1, peak_rss_bytes=64 * 2**20 if job['engine'] == 'source' else 8 * 2**20,
                status='returned', returncode=0))
        controller.verify_outputs(plan, controller.identity(fai))
        report = controller.reduce_measurements(records)
        self.assertEqual(report['primary']['source']['values'], [80.0] * 5)
        self.assertEqual(report['primary']['native']['values'], [40.0] * 5)
        self.assertTrue(report['decision']['throughput_accepted'])
        self.assertTrue(report['decision']['bounded_memory_accepted'])
        self.assertIsNone(report['exact_aggregate_memory_bytes'])
        varied = [dict(row, wall_seconds=(100.0 if row['round'] == 'trial-1' and row['engine'] == 'native'
                                         else row['wall_seconds'])) for row in records]
        self.assertFalse(controller.reduce_measurements(varied)['decision']['throughput_accepted'])
        plan[-1]['outputs']['dict'].write_bytes(b'different\n')
        with self.assertRaisesRegex(ValueError, 'output'):
            controller.verify_outputs(plan, controller.identity(fai))
        cpus = sorted(os.sched_getaffinity(0))[:1] if hasattr(os, 'sched_getaffinity') else None
        child = ('import os,sys; print("retained"); print("diagnostic", file=sys.stderr); '
                 'print("affinity="+",".join(map(str,sorted(os.sched_getaffinity(0))))) '
                 'if hasattr(os,"sched_getaffinity") else None')
        call = controller.timed_call(root, 'stdlib-child',
            [sys.executable, '-B', '-c', child], cpus=cpus)
        self.assertEqual(call['returncode'], 0)
        self.assertGreater(call['peak_rss_bytes'], 0)
        self.assertGreaterEqual(call['user_seconds'], 0)
        stdout = (root / 'calls/stdlib-child/stdout.txt').read_bytes()
        self.assertEqual(stdout, b'retained\n' +
            (('affinity=' + ','.join(map(str, cpus)) + '\n').encode() if cpus is not None else b''))
        self.assertEqual(call['requested_cpus'], cpus)
        self.assertEqual((root / 'calls/stdlib-child/stderr.txt').read_bytes(), b'diagnostic\n')
        runtime = root / 'runtime'
        runtime.mkdir()
        program = runtime / 'program'
        program.write_bytes(b'retained runtime\n')
        program.chmod(0o755)
        (runtime / 'link').symlink_to('program')
        controller.package_runtime(runtime, root / 'runtime.tar')
        with tarfile.open(root / 'runtime.tar') as archive:
            self.assertEqual(archive.getmember('runtime/program').mode, 0o755)
            self.assertTrue(archive.getmember('runtime/link').issym())
            self.assertEqual(archive.extractfile('runtime/program').read(), b'retained runtime\n')
        controller.admit_capacity(dict(inputs=0, outputs=0, runtime=0, metadata=0), 32 * 2**30)
        with self.assertRaisesRegex(RuntimeError, 'capacity'):
            controller.admit_capacity(dict(inputs=0, outputs=0, runtime=0, metadata=0), 31 * 2**30)
        workflow = ROOT / '.github/workflows/index-reference-real-performance.yml'
        controller.validate_workflow(workflow)
        for block in controller.source.shell_blocks(workflow):
            checked = subprocess.run(['bash', '-n'], input=block, text=True, capture_output=True)
            self.assertEqual(checked.returncode, 0, checked.stderr)
        with self.assertRaisesRegex(RuntimeError, 'hosted'):
            controller.source.hosted_root()
        self.assertFalse({'numpy', 'pysam', 'allel'}.intersection(sys.modules))


if __name__ == '__main__':
    unittest.main()
