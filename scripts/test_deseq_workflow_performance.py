import json
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

from test_infercnv_ingestion_workflow import bash_body, steps


ROOT = Path(__file__).resolve().parents[1]


class DeseqWorkflowPerformanceTests(unittest.TestCase):
    def test_hosted_workflow_reuses_green5_and_keeps_failed_measurements(self):
        workflow = ROOT / '.github/workflows/deseq-performance.yml'
        self.assertTrue(workflow.is_file(), 'missing whole DESeq performance workflow')
        self.assertTrue((ROOT / 'scripts/deseq_workflow_performance.R').is_file())
        text = workflow.read_text()
        self.assertIn('40611588271', text)
        self.assertIn('if: always()', text)
        self.assertNotIn('continue-on-error', text)
        self.assertNotIn('cargo build', text)
        routed = steps(text)
        self.assertIn('select-native', bash_body(routed['Acquire the exact accepted four-native executable']))
        self.assertIn('prepare-runtime', bash_body(routed['Acquire and verify the original source witness']))
        self.assertIn("r-version: '4.6.1'", text)
        for name, step in routed.items():
            if '        run: |\n' in step:
                with self.subTest(step=name):
                    body = re.sub(r'\$\{\{.*?\}\}', 'stub', bash_body(step))
                    result = subprocess.run(['bash', '-n'], input=body, capture_output=True,
                                            text=True, timeout=15)
                    self.assertEqual(result.returncode, 0, result.stderr)

    def test_multicommand_cost_and_startup_only_or_overlapping_advantages(self):
        self.assertTrue((ROOT / 'scripts/deseq_workflow_performance.py').is_file(), 'missing measurement harness')
        from deseq_workflow_performance import TIME_FORMAT, aggregate_native, performance_decision

        row = dict(wall_seconds=1, user_seconds=0.5, system_seconds=0.2, max_rss_kib=100,
                   filesystem_inputs=2, filesystem_outputs=3, major_faults=4, minor_faults=5,
                   owner_wall_seconds=1.1, exit_status=0)
        aggregate = aggregate_native([row, dict(row, wall_seconds=2, max_rss_kib=200)])
        self.assertEqual((aggregate['wall_seconds'], aggregate['max_rss_kib'], aggregate['filesystem_outputs']),
                         (3, 200, 6))
        self.assertEqual(json.loads(re.sub(r'%[a-zA-Z]', '1', TIME_FORMAT))['exit_status'], 1)
        source = dict(wall_seconds=dict(minimum=100, maximum=110, median=105),
                      inner_elapsed_seconds=dict(minimum=2, maximum=4, median=3),
                      max_rss_kib=dict(minimum=100, maximum=200, median=150))
        native = dict(wall_seconds=dict(minimum=2.5, maximum=3.5, median=3),
                      max_rss_kib=dict(minimum=100, maximum=200, median=150))
        decision = performance_decision(dict(source=source, native={'1': native, '4': native}))
        self.assertFalse(decision['advantage_passed'])
        self.assertFalse(decision['strict_worker_scaling'])
        faster = dict(native, wall_seconds=dict(minimum=1, maximum=1.5, median=1.25))
        decision = performance_decision(dict(source=source, native={'1': native, '4': faster}))
        self.assertTrue(decision['advantage_passed'])
        self.assertTrue(decision['strict_worker_scaling'])

    def test_native_gate_rejects_incomplete_platform_and_schema4_workflow_evidence(self):
        from deseq_workflow_performance import NATIVE, select_native, verify_native_workflows

        with tempfile.TemporaryDirectory(prefix='deseq-performance-gate-') as temporary:
            root = Path(temporary)
            run = dict(id=NATIVE['run_id'], run_attempt=1, head_sha=NATIVE['head_sha'],
                       path=NATIVE['workflow_path'], status='completed', conclusion='success',
                       event='workflow_dispatch', repository=dict(full_name=NATIVE['repository']),
                       head_repository=dict(full_name=NATIVE['repository']))
            jobs = [dict(name=name, run_id=NATIVE['run_id'], run_attempt=1, head_sha=NATIVE['head_sha'],
                         status='completed', conclusion='success')
                    for name in ('linux-x86_64', 'linux-aarch64', 'macos-x86_64', 'macos-aarch64')]
            (root / 'run.json').write_text(json.dumps(run))
            (root / 'jobs.json').write_text(json.dumps(dict(total_count=3, jobs=jobs[:3])))
            with self.assertRaisesRegex(ValueError, 'all four'):
                select_native(root)
            self.assertFalse((root / 'selection.json').exists())
            jobs[-1]['status'] = 'in_progress'
            (root / 'jobs.json').write_text(json.dumps(dict(total_count=4, jobs=jobs)))
            with self.assertRaisesRegex(ValueError, 'unfinished'):
                select_native(root)
            self.assertFalse((root / 'selection.json').exists())

            evidence = root / 'native-evidence'
            binary = '0' * 64
            general = ['interaction-ratio-formula', 'interaction-ratio-matrix', 'interaction-sf-formula',
                       'interaction-nf-formula', 'interaction-sf-nf-formula', 'nointercept-sf-formula',
                       'intercept-sf-formula']
            design = general + ['cook-7x7-default-formula', 'cook-7x7-default-matrix',
                                'cook-7x7-disabled-formula', 'cook-7x7-disabled-matrix',
                                'cook-7x3-default-formula', 'cook-7x3-default-matrix',
                                'cook-7x3-disabled-formula', 'cook-7x3-disabled-matrix']
            low_df = ['df-1', 'df-2', 'df-3', 'df-4']

            def write(name, value):
                path = evidence / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(json.dumps(value))

            write('worker-determinism.json', dict(passed=True, compared_workers=[1, 4], compared_workflows=20,
                pasilla=dict(fitted_state_equal=True),
                design_cases={case: dict(fitted_state_equal=True) for case in design},
                low_df_cases={case: dict(fitted_state_equal=True) for case in low_df}))
            for workers in (1, 4):
                prefix = f'comparison/workers-{workers}'
                write(prefix + '/comparison-contract.json', dict(binary_sha256=binary, workers=workers))
                write(prefix + '/comparison.json', dict(passed=True, failure_count=0))
                write(prefix + '/fit.json', dict(schema=4))
                for group, cases in [('design', design), ('low-df', low_df)]:
                    prefix = f'{group}-comparison/workers-{workers}'
                    write(prefix + '/comparison-contract.json', dict(binary_sha256=binary, workers=workers,
                        cases=cases, excluded_cases=[]))
                    write(prefix + '/summary.json', dict(passed=True,
                        cases=[dict(case=case, passed=True, failure_count=0) for case in cases]))
                    for case in cases:
                        write(prefix + '/' + case + '/fit.json', dict(schema=4))
            self.assertEqual(verify_native_workflows(evidence, binary),
                             dict(schema=4, workflow_count=20, workers=[1, 4]))
            write('low-df-comparison/workers-4/summary.json', dict(passed=True,
                cases=[dict(case=case, passed=True, failure_count=0) for case in low_df[:3]]))
            with self.assertRaisesRegex(ValueError, 'incomplete'):
                verify_native_workflows(evidence, binary)
            write('low-df-comparison/workers-4/summary.json', dict(passed=True,
                cases=[dict(case=case, passed=True, failure_count=0) for case in low_df]))
            write('low-df-comparison/workers-4/df-4/fit.json', dict(schema=3))
            with self.assertRaisesRegex(ValueError, 'schema4'):
                verify_native_workflows(evidence, binary)


if __name__ == '__main__':
    unittest.main()
