import csv
import hashlib
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

from test_infercnv_ingestion_workflow import bash_body, steps


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / '.github/workflows/limma-logexpression-oracle.yml'


class LimmaLogExpressionOracleWorkflowTests(unittest.TestCase):
    def test_ordinary_source_routes_external_inputs_and_always_retains_evidence(self):
        self.assertTrue(WORKFLOW.is_file(), 'missing ordinary log-expression source workflow')
        text = WORKFLOW.read_text()
        routed = steps(text)
        self.assertIn('workflow_dispatch:', text)
        self.assertNotIn('continue-on-error', text)
        self.assertNotIn('edgeR_4.', text)
        self.assertNotIn('DESeq2_1.', text)
        self.assertIn("r-version: '4.6.1'", text)
        capture = bash_body(routed['Capture ordinary expression and independent reload'])
        self.assertIn('scripts/limma_logexpression_oracle.R "$EVIDENCE/limma" "$INPUT_DIR" "$EVIDENCE/workflow"', capture)
        self.assertIn('Rscript --vanilla scripts/limma_logexpression_oracle.R --reload "$EVIDENCE/workflow"', capture)
        self.assertIn('if: always()', routed['Retain complete original source evidence'])
        driver = (ROOT / 'scripts/limma_logexpression_oracle.R').read_text()
        self.assertIn('stats-lm.fit.txt', driver)
        self.assertIn('source_feature_ids=rownames(value)', driver)
        self.assertIn('positional source-row to input-row mapping', driver)
        exporter = driver.split('logexpression_write_derived <- function', 1)[1].split('\nlogexpression_complete <-', 1)[0]
        self.assertLess(exporter.index('saveRDS(derived,'), exporter.index('logexpression_write_fit('))
        self.assertLess(exporter.index('"-returned-table.rds"'), exporter.index('logexpression_write_fit('))
        case = driver.split('logexpression_case <- function', 1)[1].split('\nlogexpression_reload <-', 1)[0]
        self.assertLess(case.index('"analysis-bundle.rds"'), case.index('logexpression_write_derived('))
        for name, step in routed.items():
            if '        run: |\n' in step:
                result = subprocess.run(['bash', '-n'], input=bash_body(step), capture_output=True,
                                        text=True, timeout=15)
                self.assertEqual(result.returncode, 0, f'{name}: {result.stderr}')
        archive = ROOT / 'assets/limma-logexpression-inputs-20261031.tar.gz'
        digest = re.search(r'^      INPUT_SHA256: ([a-f0-9]{64})$', text, re.MULTILINE)
        self.assertIsNotNone(digest)
        self.assertEqual(hashlib.sha256(archive.read_bytes()).hexdigest(), digest[1])
        with tempfile.TemporaryDirectory(prefix='limma-logexpression-route-') as temporary:
            runner = Path(temporary)
            env = dict(os.environ, RUNNER_TEMP=str(runner), GITHUB_ENV=str(runner / 'environment'),
                       GITHUB_WORKSPACE=str(ROOT), INPUT_SHA256=digest[1])
            prepared = subprocess.run(['bash', '-c', bash_body(routed['Prepare hosted source storage'])],
                                      env=env, cwd=ROOT, capture_output=True, text=True, timeout=15)
            self.assertEqual(prepared.returncode, 0, prepared.stderr)
            for line in (runner / 'environment').read_text().splitlines():
                key, value = line.split('=', 1)
                env[key] = value
                if key != 'INPUT_ARCHIVE':
                    self.assertTrue(Path(value).resolve().is_relative_to(runner.resolve()), line)
            unpack = bash_body(routed['Verify and unpack frozen input-only assets'])
            result = subprocess.run(['bash', '-c', unpack], env=env, cwd=ROOT,
                                    capture_output=True, text=True, timeout=15)
            self.assertEqual(result.returncode, 0, result.stderr)
            inputs = Path(env['INPUT_DIR'])
            with (inputs / 'manifest.tsv').open() as stream:
                rows = list(csv.DictReader(stream, delimiter='\t'))
            self.assertEqual(len(rows), 21)
            self.assertEqual({row['path'].split('/')[0] for row in rows},
                             {'real', 'equalvar', 'constvar', 'two-row', 'one-zero', 'LINEAGE.md'})
            for row in rows:
                self.assertEqual(hashlib.sha256((inputs / row['path']).read_bytes()).hexdigest(), row['sha256'])
                self.assertNotIn('expected', row['path'])
            self.assertEqual(next(row['sha256'] for row in rows if row['path'] == 'real/expression.tsv'),
                             '2286b07aafb95da072caff6137dbad28b796fe60785baf1827366d4d1ac127e4')
            self.assertEqual((inputs / 'two-row/expression.tsv').read_text(),
                             'gene\ts1\ts2\ts3\ts4\ng1\t0\t1\t3\t5\ng2\t-3\t-1\t0\t1\n')
            self.assertEqual((inputs / 'one-zero/expression.tsv').read_text(),
                             'gene\ts1\ts2\ts3\ts4\ng1\t0\t0\t0\t0\n')
            changed = runner / 'corrupted.tar.gz'
            changed.write_bytes(archive.read_bytes() + b'corrupt')
            env['INPUT_ARCHIVE'] = str(changed)
            fresh = runner / 'corrupt-evidence'
            fresh.mkdir()
            env['EVIDENCE'] = str(fresh)
            failed = subprocess.run(['bash', '-c', unpack], env=env, cwd=ROOT,
                                    capture_output=True, text=True, timeout=15)
            self.assertNotEqual(failed.returncode, 0, 'corrupted input archive was accepted')


if __name__ == '__main__':
    unittest.main()
