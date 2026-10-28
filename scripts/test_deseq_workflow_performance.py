import json
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

from test_infercnv_ingestion_workflow import bash_body, steps


ROOT = Path(__file__).resolve().parents[1]


class DeseqWorkflowPerformanceTests(unittest.TestCase):
    def test_hosted_workflow_reuses_green7_and_keeps_both_complete_paths(self):
        workflow = ROOT / '.github/workflows/deseq-performance.yml'
        self.assertTrue(workflow.is_file(), 'missing whole DESeq performance workflow')
        self.assertTrue((ROOT / 'scripts/deseq_workflow_performance.R').is_file())
        text = workflow.read_text()
        self.assertIn('load_native_receipt', text)
        self.assertIn('deseq-native-green7-2026-10-28.json', text)
        self.assertIn('deseq-local-trend-2026-10-28.json', text)
        self.assertIn('locfit_1.5-9.12.tar.gz', text)
        self.assertIn('if: always()', text)
        self.assertNotIn('continue-on-error', text)
        self.assertNotIn('cargo build', text)
        routed = steps(text)
        self.assertIn('select-native', bash_body(routed['Acquire the exact accepted four-native executable']))
        self.assertIn('prepare-runtime', bash_body(routed['Acquire and verify the original source witness']))
        self.assertIn("r-version: '4.6.1'", text)
        from deseq_workflow_performance import native_commands
        for mode in ('parametric', 'local'):
            commands = native_commands(Path('/source'), Path('/output'), mode)
            self.assertEqual([command[0] for command in commands], ['run', 'results', 'results', 'transform', 'transform'])
            self.assertNotIn('--no-replace-outliers', commands[0])
            self.assertEqual(commands[-1][-2:], ['--kind', 'vst'])
            if mode == 'local':
                self.assertEqual(commands[0][commands[0].index('--fit-type') + 1], 'local')
                self.assertEqual(commands[0][commands[0].index('--formula') + 1], '~type+condition')
                self.assertIn('type=paired-end,single-read', commands[0])
                self.assertIn('condition=untreated,treated', commands[0])
                self.assertEqual(commands[0][commands[0].index('--counts') + 1], Path('/source/inputs/pasilla/counts.tsv'))
            else:
                self.assertEqual(commands[0][commands[0].index('--design') + 1], Path('/source/inputs/model-matrix.tsv'))
        for name, step in routed.items():
            if '        run: |\n' in step:
                with self.subTest(step=name):
                    body = re.sub(r'\$\{\{.*?\}\}', 'stub', bash_body(step))
                    result = subprocess.run(['bash', '-n'], input=body, capture_output=True,
                                            text=True, timeout=15)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    for program in re.findall(r"python3 -B - <<'PY'\n(.*?)\nPY", body, re.DOTALL):
                        compile(program, name, 'exec')

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

    def test_native_gate_rejects_missing_receipt_and_incomplete_schema5_workflow_evidence(self):
        from deseq_workflow_performance import NATIVE, select_native, verify_native_workflows, load_native_receipt

        with tempfile.TemporaryDirectory(prefix='deseq-performance-gate-') as temporary:
            root = Path(temporary)
            with self.assertRaises(OSError):
                load_native_receipt(root / 'absent.json')
            (root / 'incomplete.json').write_text(json.dumps(NATIVE))
            with self.assertRaisesRegex(ValueError, 'receipt'):
                load_native_receipt(root / 'incomplete.json')
            fixture_receipt = dict(NATIVE, run_id=123, head_sha='a' * 40, artifact_id=456,
                                   artifact_name='unit-fixture', size_in_bytes=1, artifact_sha256='0' * 64)
            run = dict(id=fixture_receipt['run_id'], run_attempt=1, head_sha=fixture_receipt['head_sha'],
                       path=NATIVE['workflow_path'], status='completed', conclusion='success',
                       event='workflow_dispatch', repository=dict(full_name=NATIVE['repository']),
                       head_repository=dict(full_name=NATIVE['repository']))
            jobs = [dict(name=name, run_id=fixture_receipt['run_id'], run_attempt=1, head_sha=fixture_receipt['head_sha'],
                         status='completed', conclusion='success')
                    for name in ('linux-x86_64', 'linux-aarch64', 'macos-x86_64', 'macos-aarch64')]
            (root / 'run.json').write_text(json.dumps(run))
            (root / 'jobs.json').write_text(json.dumps(dict(total_count=3, jobs=jobs[:3])))
            with self.assertRaisesRegex(ValueError, 'all four'):
                select_native(root, fixture_receipt)
            self.assertFalse((root / 'selection.json').exists())
            jobs[-1]['status'] = 'in_progress'
            (root / 'jobs.json').write_text(json.dumps(dict(total_count=4, jobs=jobs)))
            with self.assertRaisesRegex(ValueError, 'unfinished'):
                select_native(root, fixture_receipt)
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
            local = ['pasilla-local-ratio', 'increasing-default-ratio', 'increasing-local-nf', 'increasing-local-sf-nf']

            def write(name, value):
                path = evidence / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(json.dumps(value))

            write('worker-determinism.json', dict(passed=True, compared_workers=[1, 4], compared_workflows=24,
                pasilla=dict(fitted_state_equal=True),
                design_cases={case: dict(fitted_state_equal=True) for case in design},
                low_df_cases={case: dict(fitted_state_equal=True) for case in low_df},
                local_cases={case: dict(fitted_state_equal=True) for case in local}))
            for workers in (1, 4):
                prefix = f'comparison/workers-{workers}'
                write(prefix + '/comparison-contract.json', dict(binary_sha256=binary, workers=workers))
                write(prefix + '/comparison.json', dict(passed=True, failure_count=0))
                write(prefix + '/fit.json', dict(schema=5, workers=workers))
                for group, cases in [('design', design), ('low-df', low_df), ('local', local)]:
                    prefix = f'{group}-comparison/workers-{workers}'
                    write(prefix + '/comparison-contract.json', dict(binary_sha256=binary, workers=workers,
                        cases=cases, excluded_cases=[]))
                    write(prefix + '/summary.json', dict(passed=True,
                        cases=[dict(case=case, passed=True, failure_count=0) for case in cases]))
                    for case in cases:
                        write(prefix + '/' + case + '/fit.json', dict(schema=5, workers=workers))
            import hashlib
            determinism = json.loads((evidence / 'worker-determinism.json').read_text())
            for family, cases in [('comparison', [None]), ('design-comparison', design),
                                  ('low-df-comparison', low_df), ('local-comparison', local)]:
                for case in cases:
                    names = ['forward.tsv', 'reverse.tsv', 'normalized-counts.tsv', 'vst.tsv']
                    if case is not None and case != 'intercept-sf-formula':
                        names.append('mixed.tsv')
                    if family == 'local-comparison':
                        names += ['queries.tsv', 'trend-first.tsv', 'trend-reload.tsv']
                    for workers in (1, 4):
                        target = evidence / family / f'workers-{workers}'
                        if case is not None:
                            target /= case
                        (target / 'failures.json').write_text('[]')
                        for name in names:
                            (target / name).write_text('gene\tvalue\ng1\t0.5\n')
                    record = determinism['pasilla'] if case is None else determinism[family.removesuffix('-comparison').replace('-', '_') + '_cases'][case]
                    record['exports_sha256'] = {name: hashlib.sha256(b'gene\tvalue\ng1\t0.5\n').hexdigest() for name in names}
            write('worker-determinism.json', determinism)
            self.assertEqual(verify_native_workflows(evidence, binary),
                             dict(schema=5, workflow_count=24, workers=[1, 4]))
            write('low-df-comparison/workers-4/summary.json', dict(passed=True,
                cases=[dict(case=case, passed=True, failure_count=0) for case in low_df[:3]]))
            with self.assertRaisesRegex(ValueError, 'incomplete'):
                verify_native_workflows(evidence, binary)
            write('low-df-comparison/workers-4/summary.json', dict(passed=True,
                cases=[dict(case=case, passed=True, failure_count=0) for case in low_df]))
            write('low-df-comparison/workers-4/df-4/fit.json', dict(schema=3))
            with self.assertRaisesRegex(ValueError, 'schema5'):
                verify_native_workflows(evidence, binary)
            write('low-df-comparison/workers-4/df-4/fit.json', dict(schema=5, workers=4))
            (evidence / 'local-comparison/workers-4/pasilla-local-ratio/trend-reload.tsv').write_text('changed')
            with self.assertRaisesRegex(ValueError, 'export'):
                verify_native_workflows(evidence, binary)


if __name__ == '__main__':
    unittest.main()
