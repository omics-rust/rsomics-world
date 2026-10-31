import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

from test_infercnv_ingestion_workflow import bash_body, steps


ROOT = Path(__file__).resolve().parents[1]


class LimmaWorkflowPerformanceTests(unittest.TestCase):
    def test_ordinary_seven_process_admission_reuse_and_source_symmetry(self):
        from limma_logexpression_performance import native_commands, load_native_receipt, check_validation_files, OUTPUTS, STATES
        commands = native_commands(Path('/source'), Path('/output'))
        self.assertEqual([row[0] for row in commands], ['run', 'results', 'results', 'results', 'export', 'export', 'export'])
        self.assertEqual(commands[0][commands[0].index('--expression') + 1], Path('/source/inputs/real/expression.tsv'))
        self.assertEqual(len(OUTPUTS), 6)
        self.assertEqual(STATES, ('forward.json', 'reverse.json', 'mixed.json'))
        for command, state in zip(commands[1:4], STATES):
            self.assertEqual(command[command.index('--state') + 1], Path('/output') / state)
        with tempfile.TemporaryDirectory(prefix='limma-ordinary-performance-') as temporary:
            with self.assertRaises(OSError):
                load_native_receipt(Path(temporary) / 'missing-admission.json')
        admitted = {'src/model.rs': 'a', 'Cargo.lock': 'b', 'scripts/check-logexpression.py': 'c'}
        changed = dict(admitted, **{'scripts/check-logexpression.py': 'd'})
        check_validation_files(admitted, changed)
        changed['src/model.rs'] = 'e'
        with self.assertRaisesRegex(ValueError, 'production'):
            check_validation_files(admitted, changed)
        text = (ROOT / '.github/workflows/limma-performance.yml').read_text()
        self.assertRegex(text, r'(?m)^      ordinary:\n        description: [^\n]+\n        type: boolean\n        required: false\n        default: false$')
        self.assertIn('validation_snapshot:', text)
        self.assertIn('limma_logexpression_performance.py', text)
        self.assertIn('VALIDATION_SNAPSHOT', text)
        self.assertNotIn('cargo build', text)
        for name, step in steps(text).items():
            if '        run: |\n' in step:
                body = re.sub(r'\$\{\{.*?\}\}', 'stub', bash_body(step))
                result = subprocess.run(['bash', '-n'], input=body, text=True, capture_output=True, timeout=15)
                self.assertEqual(result.returncode, 0, name + ': ' + result.stderr)
                for program in re.findall(r"python3 -B - <<'PY'\n(.*?)\nPY", body, re.DOTALL):
                    compile(program, name, 'exec')
        selector = 'controller=' + bash_body(steps(text)['Prepare hosted source and measurement storage']).split('controller=', 1)[1]
        with tempfile.TemporaryDirectory(prefix='limma-ordinary-selector-') as temporary:
            for ordinary, snapshot, valid in [('false', '', True), ('true', '', False), ('true', '../green-2', False), ('false', 'green-2', False)]:
                env_file = Path(temporary) / (ordinary + '-' + str(valid) + '-environment')
                result = subprocess.run(['bash', '-euc', selector], cwd=ROOT,
                    env=dict(os.environ, ORDINARY=ordinary, VALIDATION_SNAPSHOT=snapshot,
                             GITHUB_WORKSPACE=str(ROOT), GITHUB_ENV=str(env_file)), text=True, capture_output=True, timeout=15)
                self.assertEqual(result.returncode == 0, valid, f'{ordinary=} {snapshot=}: {result.stderr}')
        driver = (ROOT / 'scripts/limma_logexpression_performance.R').read_text()
        timed = driver.split('ordinary_performance_run <- function', 1)[1].split('ordinary_performance_inspect <- function', 1)[0]
        self.assertEqual(timed.count('limma::lmFit('), 1)
        self.assertLess(timed.index('ordinary_test(bundle, "forward")'), timed.index('saveRDS(bundle'))
        self.assertLess(timed.index('saveRDS(bundle'), timed.index('reloaded <- readRDS('))
        self.assertNotIn('logexpression_derive(', timed)
        self.assertNotIn('logexpression_write_fit(', timed)
        self.assertNotIn('squeezeVar(', timed)
        self.assertIn('ordinary_no_refit()', timed)
        inspector = driver.split('ordinary_performance_inspect <- function', 1)[1]
        self.assertNotIn('ordinary_test(', inspector)
        self.assertNotIn('limma::lmFit(', inspector)

    def test_complete_trial_routing_and_hosted_shells(self):
        workflow = ROOT / '.github/workflows/limma-performance.yml'
        self.assertTrue(workflow.is_file(), 'missing complete limma performance workflow')
        from limma_workflow_performance import native_commands, OUTPUTS, STATES
        commands = native_commands(Path('/source'), Path('/output'))
        self.assertEqual([row[0] for row in commands], ['run', 'results', 'results', 'results',
                                                     'export', 'export', 'export', 'export'])
        self.assertEqual(commands[0][commands[0].index('--counts') + 1], Path('/source/inputs/counts.tsv'))
        self.assertEqual(commands[0][commands[0].index('--design') + 1], Path('/source/inputs/model-matrix.tsv'))
        self.assertEqual(len(OUTPUTS), 7)
        self.assertEqual(STATES, ('forward.json', 'reverse.json', 'mixed.json'))
        for command, name in zip(commands[1:4], STATES):
            self.assertEqual(command[command.index('--state') + 1], Path('/output') / name)
        self.assertIn('--contrast=typesingle-read=-1', commands[3])
        text = workflow.read_text()
        self.assertIn('limma-native-green1-2026-10-28.json', text)
        self.assertIn('load_native_receipt', text)
        self.assertIn("r-version: '4.6.1'", text)
        self.assertIn('if: always()', text)
        self.assertNotIn('cargo build', text)
        self.assertNotIn('continue-on-error', text)
        self.assertNotIn('R CMD INSTALL --preclean --clean --library="$R_LIBS_USER" "$EVIDENCE/runtime/DESeq2"', text)
        for name, step in steps(text).items():
            if '        run: |\n' in step:
                body = re.sub(r'\$\{\{.*?\}\}', 'stub', bash_body(step))
                result = subprocess.run(['bash', '-n'], input=body, text=True, capture_output=True, timeout=15)
                self.assertEqual(result.returncode, 0, name + ': ' + result.stderr)
                for program in re.findall(r"python3 -B - <<'PY'\n(.*?)\nPY", body, re.DOTALL):
                    compile(program, name, 'exec')
        driver = (ROOT / 'scripts/limma_workflow_performance.R').read_text()
        timed = driver.split('limma_performance_run <- function', 1)[1].split('limma_performance_inspect <- function', 1)[0]
        self.assertLess(timed.index('saveRDS(bundle'), timed.index('limma_derive(reloaded'))
        self.assertEqual(timed.count('limma::contrasts.fit('), 1)
        self.assertNotIn('limma_write_derived(', timed)
        self.assertNotIn('limma_write_fit(', timed)
        self.assertIn('saveRDS(derived$tests[[name]]', timed)
        inspection = driver.split('limma_performance_inspect <- function', 1)[1]
        self.assertIn('colnames(stored) <- colnames(bundle$normalized$counts)', inspection)
        self.assertIn('deseq_write_table(stored, copy, rownames(bundle$base$coefficients))', inspection)
        self.assertNotIn('all.equal(as.matrix(actual), stored', inspection)

    def test_retained_admission_rejects_missing_or_partial_native_gate(self):
        from limma_workflow_performance import (NATIVE, SOURCE, BOUNDS, OUTPUTS, STATES,
            load_native_receipt, select_native, verify_native_workflow)
        with tempfile.TemporaryDirectory(prefix='limma-performance-gate-') as temporary:
            root = Path(temporary)
            with self.assertRaises(OSError):
                load_native_receipt(root / 'absent.json')
            (root / 'incomplete.json').write_text(json.dumps(NATIVE))
            with self.assertRaisesRegex(ValueError, 'receipt'):
                load_native_receipt(root / 'incomplete.json')
            receipt = dict(NATIVE, run_id=123, head_sha='a' * 40, artifact_id=456,
                           artifact_name='unit-fixture', size_in_bytes=1, artifact_sha256='0' * 64)
            run = dict(id=123, run_attempt=1, head_sha=receipt['head_sha'], path=NATIVE['workflow_path'],
                       status='completed', conclusion='success', event='workflow_dispatch',
                       repository=dict(full_name=NATIVE['repository']), head_repository=dict(full_name=NATIVE['repository']))
            (root / 'run.json').write_text(json.dumps(run))
            jobs = [dict(name=name, run_id=123, run_attempt=1, head_sha=receipt['head_sha'],
                         status='completed', conclusion='success') for name in
                    ('linux-x86_64', 'linux-aarch64', 'macos-x86_64', 'macos-aarch64')]
            (root / 'jobs.json').write_text(json.dumps(dict(total_count=3, jobs=jobs[:3])))
            with self.assertRaisesRegex(ValueError, 'all four'):
                select_native(root, receipt)
            jobs[-1]['status'] = 'in_progress'
            (root / 'jobs.json').write_text(json.dumps(dict(total_count=4, jobs=jobs)))
            with self.assertRaisesRegex(ValueError, 'unfinished'):
                select_native(root, receipt)
            self.assertFalse((root / 'selection.json').exists())
            import hashlib
            evidence = root / 'native-evidence'
            binary = '0' * 64

            def write(name, value):
                path = evidence / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(json.dumps(value))

            hashes = {name: hashlib.sha256(b'unit-admission-fixture\n').hexdigest() for name in (*OUTPUTS, *STATES)}
            write('worker-determinism.json', dict(passed=True, compared_workers=[1, 4], compared_workflows=1,
                fitted_state_equal=True, complete_contrast_states=3, export_tables=7, binary_sha256=binary, exports_sha256=hashes))
            write('source-after.json', dict(unchanged_files=29))
            for workers in (1, 4):
                prefix = f'comparison/workers-{workers}'
                write(prefix + '/fit.json', dict(schema=1, product='rsomics-limma', workers=workers))
                fit_sha = hashlib.sha256((evidence / prefix / 'fit.json').read_bytes()).hexdigest()
                write(prefix + '/comparison-contract.json', dict(workers=workers, binary_sha256=binary,
                    source_run=SOURCE['run_id'], manifest_sha256=SOURCE['manifest_sha256'], witness_sha256=SOURCE['witness_sha256'],
                    bounds=BOUNDS, positive_probability_log_bound=1e-3))
                write(prefix + '/comparison.json', dict(passed=True, failure_count=0, raw_genes=14599, kept_genes=8066,
                    residual_df_distribution={'2': 7, '4': 8059}, bounds=BOUNDS, native_bundle_sha256=fit_sha))
                write(prefix + '/failures.json', [])
                write(prefix + '/commands.json', [dict(exit=0)] * 8)
                for name in hashes:
                    (evidence / prefix / name).write_text('unit-admission-fixture\n')
            self.assertEqual(verify_native_workflow(evidence, binary), dict(schema=1, product='rsomics-limma',
                workflow_count=1, workers=[1, 4], complete_contrast_states=3, export_tables=7))
            (evidence / 'comparison/workers-4/mixed.json').write_text('changed')
            with self.assertRaisesRegex(ValueError, 'state/table determinism'):
                verify_native_workflow(evidence, binary)

    def test_eight_process_units_and_fixed_advantage_decision(self):
        from limma_workflow_performance import (aggregate_native, performance_decision, output_manifest,
                                               verify_reused_outputs, OUTPUTS, STATES)
        with tempfile.TemporaryDirectory(prefix='limma-actual-state-') as temporary:
            output, validation = [Path(temporary) / name for name in ('output', 'validation')]
            output.mkdir()
            validation.mkdir()
            for name in ('fit.json', *OUTPUTS):
                for directory in (output, validation):
                    (directory / name).write_text('unchanged stored bytes\n')
            for name in STATES:
                (output / name).write_text('{"contrast":[0.0,0.0,-1.0],"values":[1.25]}')
                (validation / name).write_text('{"contrast":[-0.0,-0.0,-1.0],"values":[1.25]}')
            original = output_manifest(output)
            verify_reused_outputs(output, validation, original)
            (validation / 'reverse.json').write_text('{"contrast":[0.0,0.0,-1.0],"values":[1.250001]}')
            with self.assertRaisesRegex(ValueError, 'complete contrast values: reverse.json'):
                verify_reused_outputs(output, validation, original)
        row = dict(wall_seconds=1, user_seconds=.5, system_seconds=.2, max_rss_kib=100,
                   filesystem_inputs=2, filesystem_outputs=3, major_faults=4, minor_faults=5,
                   owner_wall_seconds=1.1, exit_status=0)
        total = aggregate_native([row] * 8)
        self.assertEqual((total['wall_seconds'], total['max_rss_kib'], total['filesystem_outputs']), (8, 100, 24))
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


if __name__ == '__main__':
    unittest.main()
