import importlib
from contextlib import redirect_stderr
import io
import json
import os
from pathlib import Path
import re
import sys
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from test_infercnv_ingestion_workflow import bash_body, steps


ROOT = Path(__file__).resolve().parents[1]


class PeakRealPreparation(unittest.TestCase):
    def test_complete_real_workflow_preserves_own_rows_cpu_budgets_and_exact_admission(self):
        self.assertTrue((ROOT / 'scripts/peak_real_performance.py').is_file(),
                        'adopted integrated real-workflow controller is not implemented')
        real = importlib.import_module('peak_real_performance')
        self.assertFalse({'pysam', 'numpy', 'scipy', 'MACS3'}.intersection(sys.modules))
        with tempfile.TemporaryDirectory(prefix='peak-real-preparation-', dir=os.environ['TMPDIR']) as name:
            root = Path(name)
            inputs, runtime = root / 'inputs', root / 'old/runtime'
            for tool in ('source', 'native'):
                for workers in (1, 4):
                    graph = real.graph(tool, workers, inputs, root / tool / str(workers),
                                       root / 'binary', runtime)
                    self.assertEqual([stage['name'] for stage in graph],
                                     ['call', 'sparse', 'rows', 'annotate', 'count'])
                    call = graph[0]['argv']
                    self.assertEqual(call[call.index('--threads') + 1], str(workers))
                    self.assertNotIn('--nomodel', call)
                    self.assertNotIn('--fallback-extension', call)
                    if tool == 'source':
                        self.assertEqual(call[call.index('--process-backend') + 1],
                                         'serial' if workers == 1 else 'auto')
                        self.assertIn(str(runtime / 'venv/bin/python'), call)
                        self.assertIn(str(runtime / 'venv/bin/macs3'), call)
                        self.assertIn(str(inputs / 'C.bam'), graph[-1]['argv'])
                    else:
                        self.assertEqual(call[call.index('--duplicates') + 1], 'auto')
                        sparse = graph[1]['argv']
                        self.assertEqual(sparse[sparse.index('--compatibility') + 1], 'seacr-1.3')
                        self.assertIn('C=' + str(inputs / 'C.bam'), graph[-1]['argv'])

            called, sparse = root / 'peaks.narrowPeak', root / 'sparse.bed'
            called.write_text('chr19\t10000000\t10000020\tsame\t1\t.\t1\t2\t3\t4\n'
                              'chr19\t10000000\t10000020\tsame\t1\t.\t1\t2\t3\t4\n')
            sparse.write_text('chr20\t10000030\t10000050\t40\t2\tchr20:10000030-10000050\n')
            real.write_rows(called, sparse, root / 'rows')
            bed = root / 'rows/workflow.bed'
            self.assertEqual(bed.read_text().splitlines(), [
                'chr19\t10000000\t10000020\tcall-0\t0\t.',
                'chr19\t10000000\t10000020\tcall-1\t0\t.',
                'chr20\t10000030\t10000050\tsparse-0\t0\t.'])
            native, source = root / 'counts.tsv', root / 'source-counts.tsv'
            records = bed.read_text().splitlines()
            native.write_text('chrom\tstart\tend\tname\tscore\tstrand\tT1\tT2\tC\n' +
                              ''.join(line + '\t2\t3\t4\n' for line in records))
            source.write_text(''.join(line + '\t2\t3\t4\n' for line in records))
            self.assertEqual(real.compare_counts(native, source, bed), 3)
            source.write_text(source.read_text().replace('\t2\t3\t4\n', '\t2\t3\t5\n', 1))
            with self.assertRaisesRegex(ValueError, 'count'):
                real.compare_counts(native, source, bed)
            source.write_text(''.join(line + '\t2\t3\t4\n' for line in reversed(records)))
            with self.assertRaisesRegex(ValueError, 'identity'):
                real.compare_counts(native, source, bed)

            needed = 'Version needs section:\n  File: libc.so.6 Cnt: 1\n  Name: GLIBC_2.34 Flags: none\n'
            provided = 'Version definition section:\n  Name: GLIBC_2.31\n\nVersion needs section:\n  Name: GLIBC_2.34\n'
            self.assertEqual(real.version_needs(needed), {'libc.so.6': {'GLIBC_2.34'}})
            with self.assertRaisesRegex(ValueError, 'GLIBC_2.34'):
                real.check_version_closure(real.version_needs(needed), {'libc.so.6': provided})
            real.check_version_closure(real.version_needs(needed), {
                'libc.so.6': 'Version definition section:\n  Name: GLIBC_2.31\n  Name: GLIBC_2.34\n'})
            prefix = root / 'runner/peak-first-workflow-oracle'
            prefix.parent.mkdir()
            self.assertEqual(real.admit_old_prefix(prefix, prefix.parent), prefix)
            prefix.mkdir()
            with self.assertRaisesRegex(ValueError, 'already'):
                real.admit_old_prefix(prefix, prefix.parent)
            with self.assertRaisesRegex(ValueError, 'runner'):
                real.admit_old_prefix(root / 'elsewhere', prefix.parent)

            schedule = real.pair_schedule()
            self.assertEqual(len(schedule), 10)
            for workers in (1, 4):
                self.assertEqual([row['order'] for row in schedule if row['workers'] == workers],
                                 [['source', 'native'], ['native', 'source'], ['source', 'native'],
                                  ['native', 'source'], ['source', 'native']])
            source = [dict(tool='source', wall_seconds=x) for x in (10, 11, 12, 13, 14)]
            native = [dict(tool='native', wall_seconds=x) for x in (5, 6, 7, 8, 9)]
            self.assertTrue(real.strict_decision(source + native)['strict_wall'])
            native[-1]['wall_seconds'] = 10
            decision = real.strict_decision(source + native)
            self.assertFalse(decision['strict_wall'])
            self.assertIsNone(decision['exact_tree_memory_advantage'])
            self.assertFalse(decision['passed'])

            self.assertTrue(callable(getattr(real, 'source_model', None)),
                            'fit admission must parse the actual captured MACS paired-peak log')
            self.assertEqual(real.source_model(
                'INFO #2 Total number of paired peaks: 344 \n'
                'INFO #2 predicted fragment length is 211 bps \n'), {'d': 211, 'paired_peaks': 344})
            self.assertTrue(callable(getattr(real, 'downstream_graph', None)),
                            'corrected source/native timings must consume the same fixed native ledger')
            ledger = root / 'corrected-rows'
            for tool in ('source', 'native'):
                graph = real.downstream_graph(tool, 4, inputs, root / ('C-' + tool), ledger,
                                              root / 'binary', runtime)
                self.assertEqual([row['name'] for row in graph], ['annotate', 'count'])
                self.assertTrue(all(str(ledger) in ' '.join(row['argv']) for row in graph))

            with patch.dict(os.environ, GITHUB_ACTIONS='false'), redirect_stderr(io.StringIO()) as error:
                blocked = real.main(['campaign', '--root', str(root / 'not-hosted'),
                                     '--native-receipt', 'missing'])
            self.assertNotEqual(blocked, 0)
            self.assertIn('hosted Linux', error.getvalue())
            self.assertFalse((root / 'not-hosted').exists())
            source = (ROOT / 'scripts/peak_real_performance.R').read_text()
            self.assertIn('txdbmaker::makeTxDbFromGFF', source,
                          'timed annotation must invoke the actual TxDb API, not the diagnostic capture wrapper')
            self.assertIn('ChIPseeker::annotatePeak', source)
            self.assertNotRegex(source, r'\b(?:saveRDS|saveDb|snapshot|capture\.output|exonsBy)\s*\(')
            self.assertNotIn('body(environment$annotate)', source)
            self.assertRegex(source, r'level\s*=\s*"transcript"')
            self.assertRegex(source, r'annoDb\s*=\s*NULL')
            self.assertIn('detailGenomicAnnotation', source)
            workflow = (ROOT / '.github/workflows/peak-real-performance.yml').read_text()
            for name, step in steps(workflow).items():
                if '        run: |\n' not in step:
                    continue
                result = subprocess.run(['bash', '-n'], input=bash_body(step), text=True, capture_output=True)
                self.assertEqual(result.returncode, 0, name + result.stderr)


if __name__ == '__main__':
    unittest.main()
