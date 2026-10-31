import importlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest
from unittest import mock

from test_infercnv_ingestion_workflow import bash_body, steps


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / '.github/workflows/signal-first-workflow-oracle.yml'
LOCK = ROOT / 'assets/signal-first-runtime-20261031.json'
INPUTS = ROOT / 'assets/signal-first-inputs-20261031.tar.gz'


class SignalFirstWorkflowOracleTests(unittest.TestCase):
    def test_source_route_freezes_inputs_runtime_and_retains_real_failures(self):
        self.assertTrue(WORKFLOW.is_file(), 'missing signal first-six source workflow')
        oracle = importlib.import_module('signal_first_workflow_oracle')
        text = WORKFLOW.read_text()
        routed = steps(text)
        self.assertNotIn('continue-on-error', text)
        self.assertNotIn('runner.temp', text.split('    steps:', 1)[0])
        self.assertIn('if: always()', routed['Retain complete original evidence'])
        for name, step in routed.items():
            if '        run: |\n' in step:
                body = re.sub(r'\$\{\{.*?\}\}', 'route-only', bash_body(step))
                result = subprocess.run(['bash', '-n'], input=body, text=True,
                                        capture_output=True, timeout=15)
                self.assertEqual(result.returncode, 0, name + ': ' + result.stderr)
                for program in re.findall(r"python3 -B - <<'PY'\n(.*?)\nPY", body, re.DOTALL):
                    compile(program, name, 'exec')
        with tempfile.TemporaryDirectory(prefix='signal-source-route-') as directory:
            root = Path(directory)
            contract = oracle.prepare_inputs(INPUTS, LOCK, root / 'inputs')
            self.assertEqual(contract['upstream']['commit'], 'ea0f68bb4a1587d713dacb3791861308751ef7d0')
            self.assertEqual(set(contract['main_operations']), {'filter', 'track', 'compare', 'average', 'summarize', 'matrix'})
            self.assertEqual(contract['assets']['rsomics-bigwig-compare']['commit'],
                             '5ddcf9db6ecd72920fc0fdd19c61dde752bf4ad7')
            self.assertNotIn('expected', contract)
            self.assertIn('10M40N10M', (root / 'inputs/authored/a.sam').read_text())
            self.assertIn('deepTools_group', (root / 'inputs/authored/regions.gtf').read_text())
            self.assertTrue((root / 'inputs/historical/rsomics-bigwig-compare/tests/golden/a.bw').is_file())
            plan = oracle.workflow_requests(root / 'witness', root / 'inputs', root / 'runtime', LOCK)
            self.assertEqual(len({row['name'] for row in plan}), len(plan))
            self.assertEqual({row['request']['entrypoint'] for row in plan if row['request']['kind'] == 'invoke'},
                             {'alignmentSieve', 'bamCoverage', 'bamCompare', 'bigwigCompare', 'bigwigAverage',
                              'multiBamSummary', 'multiBigwigSummary', 'computeMatrix'})
            self.assertEqual([row['name'] for row in plan if row['request'].get('expected')],
                             ['bed-filtered-out-boundary'])
            requests = {row['name']: row['request'] for row in plan}
            with self.subTest('arithmetic products reach downstream source consumers'):
                derived = [requests[name]['args'][requests[name]['args'].index('-o') + 1]
                           for name in ('compare-bw', 'average')]
                for name, flag, labels in (
                        ('summary-bw-derived-bins', '-b', '--labels'),
                        ('summary-bw-derived-BED-file', '-b', '--labels'),
                        ('matrix-reference', '-S', '--samplesLabel'),
                        ('matrix-scale', '-S', '--samplesLabel')):
                    self.assertIn(name, requests)
                    args = requests[name]['args']
                    self.assertEqual(args[args.index(flag) + 1:args.index(flag) + 3], derived)
                    self.assertEqual(args[args.index(labels) + 1:args.index(labels) + 3],
                                     ['CPM A/B log2 ratio', 'CPM A/B/A scaled average'])
            with self.subTest('main normalization is verbose and bound to source BAMs'):
                main_tracks = ['track-' + sample + '-' + form for sample in ('a', 'b')
                               for form in ('bedgraph', 'bigwig')]
                main_comparisons = ['compare-bam-' + mode for mode in ('readCount', 'SES', 'manual', 'normalized')]
                for name in main_tracks + main_comparisons:
                    self.assertIn('--verbose', requests[name]['args'], name)
                outputs = {request['args'][request['args'].index('-o') + 1]
                           for request in requests.values() if request['kind'] == 'invoke'}
                dummy_outputs = []
                for name, producer, entrypoint in (
                        *[('normalization-main-track-' + sample, 'track-' + sample + '-bigwig', 'bamCoverage')
                          for sample in ('a', 'b')],
                        *[('normalization-main-compare-' + mode, 'compare-bam-' + mode, 'bamCompare')
                          for mode in ('readCount', 'SES', 'manual', 'normalized')]):
                    request = requests[name]
                    self.assertEqual(request['kind'], 'normalization')
                    self.assertEqual((request['producer'], request['entrypoint']), (producer, entrypoint))
                    original = list(requests[producer]['args'])
                    position = original.index('-o') + 1
                    dummy_outputs.append(request['args'][position])
                    self.assertNotIn(dummy_outputs[-1], outputs)
                    original[position] = dummy_outputs[-1]
                    self.assertEqual(request['args'], original)
                self.assertEqual(len(dummy_outputs), len(set(dummy_outputs)))
                self.assertEqual(requests['compare-bam-SES']['args'][-6:],
                                 ['--scaleFactorsMethod', 'SES', '--sampleLength', '50', '--numberOfSamples', '24'])
            with self.subTest('signed asymmetric log declares infinity exposure'):
                self.assertIn('nonfinite-log-asymmetric', requests)
                args = requests['nonfinite-log-asymmetric']['args']
                self.assertEqual(args[args.index('--operation') + 1], 'log2')
                self.assertEqual(args[args.index('--pseudocount') + 1:args.index('--pseudocount') + 3], ['0', '2'])
                self.assertIn('--fixedStep', args)
                self.assertEqual(args[args.index('-b1') + 1], str(root / 'witness/generated-inputs/signed-a.bw'))
                self.assertEqual(args[args.index('-b2') + 1], str(root / 'witness/generated-inputs/signed-b.bw'))
                nonfinite = root / 'retention.bedgraph'
                nonfinite.write_text('chr1\t0\t20\tinf\nchr1\t20\t40\t-inf\nchr1\t40\t60\tnan\n')
                observed = oracle.table_observation(nonfinite)
                self.assertEqual(observed['nonfinite_classes'], ['+inf', '-inf', 'nan'])
                oracle.save(root / 'nonfinite.json', observed)
                self.assertEqual([row['value'] for row in json.loads((root / 'nonfinite.json').read_text())['intervals']],
                                 [{'nonfinite': '+inf'}, {'nonfinite': '-inf'}, {'nonfinite': 'nan'}])
            runtime = json.loads(LOCK.read_text())
            self.assertEqual(runtime['python']['version'], '3.12.10')
            self.assertEqual(runtime['packages']['deeptools']['version'], '3.5.6')
            self.assertTrue(all(re.fullmatch('[0-9a-f]{64}', item['sha256'])
                                for item in runtime['packages'].values()))
            with self.subTest('portable binding precedes separately captured required ensurepip'):
                self.assertTrue(hasattr(oracle, 'bootstrap_venv'), 'missing explicit portable venv bootstrap')

                class FixtureVenv:
                    def __init__(self, with_pip=False, symlinks=False):
                        if with_pip:
                            raise AssertionError('implicit ensurepip would hide child output')
                        self.symlinks = symlinks

                    def create(self, destination):
                        binary = destination / 'bin/python'
                        binary.parent.mkdir(parents=True)
                        original = destination.parent / 'python/bin/python3'
                        if self.symlinks:
                            binary.symlink_to(original)
                        else:
                            shutil.copyfile(original, binary)
                        (destination / 'pyvenv.cfg').write_text('include-system-site-packages = false\n')

                output = root / 'bootstrap'
                original = output / 'python/bin/python3'
                original.parent.mkdir(parents=True)
                original.write_text('#!/bin/sh\nprintf "bootstrap stdout\\n"\nprintf "bootstrap stderr\\n" >&2\nexit 23\n')
                original.chmod(0o755)
                with mock.patch.object(oracle.venv, 'EnvBuilder', FixtureVenv):
                    with self.assertRaisesRegex(RuntimeError, 'required command failed'):
                        oracle.bootstrap_venv(output)
                binding = json.loads((output / 'venv-binding.json').read_text())
                self.assertEqual(binding['resolved_executable'], str(original.resolve()))
                self.assertEqual(binding['original_executable_sha256'], oracle.sha256(original))
                self.assertEqual(binding['configuration_sha256'], oracle.sha256(output / 'venv/pyvenv.cfg'))
                self.assertEqual(json.loads((output / 'ensurepip/request.json').read_text())['argv'],
                                 [str(output / 'venv/bin/python'), '-m', 'ensurepip', '--upgrade', '--default-pip'])
                self.assertEqual(json.loads((output / 'ensurepip/command.json').read_text())['status'], 23)
                self.assertEqual((output / 'ensurepip/stdout.txt').read_text(), 'bootstrap stdout\n')
                self.assertEqual((output / 'ensurepip/stderr.txt').read_text(), 'bootstrap stderr\n')

                class CopiedVenv(FixtureVenv):
                    def create(self, destination):
                        self.symlinks = False
                        super().create(destination)

                copied = root / 'copied-bootstrap'
                (copied / 'python/bin').mkdir(parents=True)
                shutil.copyfile(original, copied / 'python/bin/python3')
                with mock.patch.object(oracle.venv, 'EnvBuilder', CopiedVenv):
                    with self.assertRaisesRegex(RuntimeError, 'virtual environment interpreter'):
                        oracle.bootstrap_venv(copied)
                self.assertFalse((copied / 'ensurepip').exists())
            with (root / 'source-sorted-regions.bed').open('w') as stream:
                oracle.save(root / 'source-defaults.json', {'outFileSortedRegions': stream})
            self.assertEqual(json.loads((root / 'source-defaults.json').read_text())['outFileSortedRegions']['mode'], 'w')
            program = re.findall(r"python3 -B - <<'PY'\n(.*?)\nPY",
                                 bash_body(routed['Prepare hosted storage']), re.DOTALL)[0]
            runner = root / 'runner'
            runner.mkdir()
            env = dict(os.environ, RUNNER_TEMP=str(runner), GITHUB_SHA='route-only',
                       GITHUB_RUN_ID='route-only', GITHUB_RUN_ATTEMPT='1')
            for name in ('EVIDENCE', 'TMPDIR', 'XDG_CACHE_HOME', 'PIP_CACHE_DIR', 'PYTHONUSERBASE', 'MPLCONFIGDIR'):
                path = runner / ('signal-first-workflow-oracle' if name == 'EVIDENCE' else name.lower())
                path.mkdir()
                env[name] = str(path)
            (Path(env['EVIDENCE']) / 'controller').mkdir()
            def storage_check(current):
                return subprocess.run([sys.executable, '-B', '-c', program], env=current,
                                      capture_output=True, text=True, timeout=15)
            result = storage_check(env)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(json.loads((Path(env['EVIDENCE']) / 'storage-host.json').read_text())['controller_commit'], 'route-only')
            self.assertNotEqual(storage_check(dict(env, MPLCONFIGDIR=str(root))).returncode, 0)
            with self.subTest('single original tar preserves case and link metadata after manifest'):
                name = 'Package complete original evidence'
                self.assertTrue(name in routed, 'missing case-preserving original archive step')
                self.assertIn('if: always()', routed[name])
                self.assertLess(list(routed).index('Bind all originals and failed partial evidence'), list(routed).index(name))
                self.assertLess(list(routed).index(name), list(routed).index('Retain complete original evidence'))
                evidence = Path(env['EVIDENCE'])
                (evidence / 'UpperCase').mkdir()
                (evidence / 'UpperCase/Original.txt').write_bytes(b'partial original bytes\n')
                (evidence / 'absolute-link').symlink_to('/host-only/runtime/python/bin/python3')
                oracle.bind_evidence(evidence)
                result = subprocess.run(['bash', '-c', bash_body(routed[name])], env=dict(env, COPYFILE_DISABLE='1'),
                                        capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode, 0, result.stderr)
                archive = runner / 'signal-first-workflow-oracle.tar'
                upload = re.search(r'^\s+path: (.+)$', routed['Retain complete original evidence'], re.MULTILINE)[1]
                self.assertEqual(Path(upload.replace('${{ runner.temp }}', str(runner))), archive)
                with tarfile.open(archive) as retained:
                    prefix = 'signal-first-workflow-oracle/'
                    self.assertEqual(retained.extractfile(prefix + 'UpperCase/Original.txt').read(), b'partial original bytes\n')
                    link = retained.getmember(prefix + 'absolute-link')
                    self.assertTrue(link.issym())
                    self.assertEqual(link.linkname, '/host-only/runtime/python/bin/python3')
                    self.assertIsNotNone(retained.getmember(prefix + 'artifact-manifest.json'))
                self.assertTrue((evidence / 'UpperCase/Original.txt').is_file())
            result = oracle.capture(root / 'command', [sys.executable, '-B', '-c',
                                     'import sys; print("captured input"); sys.stderr.write("original failure\\n"); sys.exit(7)'],
                                    required=False)
            self.assertEqual(result['status'], 7)
            self.assertEqual((root / 'command/stderr.txt').read_text(), 'original failure\n')
            with self.assertRaisesRegex(RuntimeError, 'required command failed'):
                oracle.capture(root / 'required', [sys.executable, '-B', '-c', 'raise SystemExit(9)'])
            damaged = root / 'damaged.tar.gz'
            damaged.write_bytes(INPUTS.read_bytes() + b'changed')
            with self.assertRaisesRegex(ValueError, 'input archive digest'):
                oracle.prepare_inputs(damaged, LOCK, root / 'bad-inputs')
        result = subprocess.run([sys.executable, '-B', str(ROOT / 'scripts/signal_first_workflow_oracle.py'), '--help'],
                                text=True, capture_output=True, timeout=15)
        self.assertEqual(result.returncode, 0, result.stderr)
        for name in ('numpy', 'pysam', 'pyBigWig', 'deeptools'):
            self.assertNotIn(name, sys.modules)


if __name__ == '__main__':
    unittest.main()
