#!/usr/bin/env python3
"""Measure the admitted complete supplied-expression lifecycle without rebuilding."""

import argparse
import csv
import importlib.util
import inspect
import json
import math
import os
from pathlib import Path
import platform
import shutil
import sys
import tarfile

from deseq_workflow_performance import (TIME_FORMAT, aggregate_native, check, distribution,
    extract_zip, measured, performance_decision, save, select_native as select_retained_artifact)
from infercnv_source_snapshot import verify_and_extract
from infercnv_trial_process import TrialProcessError, run_child
from verify_infercnv_oracle_artifact import HEX64, _artifact, _load_json, _run, _sha256, _archive_name
from limma_workflow_performance import LOCK_SHA, PACKAGES

ROOT = Path(__file__).resolve().parents[1]
NATIVE_RECEIPT = ROOT / '.autopilot/oracles/limma-native-green2-2026-10-31.json'
ARCHIVE_SHA = 'c8cfb07407405d9ca6a13330e101069f450ca537c55e5be01264daa461532e6e'
MANIFEST_SHA = 'ee600e322850148ee6d1b7216734da1ea9ecfc33f310d5aee654aa20cc55344b'
OWNER_HEAD = '8c26abfb5cde9da1f01dec87ff73432ad747482b'
SOURCE = dict(schema_version=1, repository='omics-rust/rsomics-world', run_attempt=1,
    workflow_path='.github/workflows/limma-logexpression-oracle.yml', run_id=41184814043,
    artifact_id=13036282988, head_sha='06837d86ee46203cb6e5cafc9cef89f08e3f42df',
    artifact_name='limma-logexpression-oracle-41184814043-1', size_in_bytes=54571680,
    artifact_sha256='6460f37c81d0ec62144a38526e05756917702f0629f3e0479bae50d4fef8d7f0', bundle_path='workflow',
    manifest_sha256='c374d2be76ce0d0ee25aae2eeb14548ddb02d9866a0266e814bd39c5dec5c19f',
    capture_witness_sha256='ee4a903ce605ac51d059469d738020fb31e02ac45332156ce30ead5abaece4ab',
    reload_witness_sha256='87ca32ca6d0a199c7b6c729ae99096242fcc223e5ef8eee94bde3e5ae922af5f')
CASES = ('real', 'equalvar', 'constvar', 'two-row', 'one-zero')
BOUNDS = dict(expression=(1e-10, 1e-10), linear=(1e-7, 1e-6), eb=(1e-6, 1e-4),
              df=(1e-3, 1e-4), odds=(1e-5, 5e-4), probability=(1e-12, 5e-4))
OUTPUTS = ('forward.tsv', 'reverse.tsv', 'mixed.tsv', 'expression.tsv', 'fitted.tsv', 'residuals.tsv')
STATES = ('forward.json', 'reverse.json', 'mixed.json')
REQUIRED_STEPS = ('Verify and extract frozen source', 'Install Rust 1.91',
    'Acquire and verify actual current-voom evidence', 'Acquire and verify actual ordinary-expression evidence',
    'Verify frozen dependencies', 'Format and strict Clippy', 'Test debug', 'Test release',
    'Compare complete fitted workflow', 'Check worker determinism', 'Verify source remains unchanged', 'Upload raw evidence')
CONTROLLERS = ('limma_logexpression_performance.py', 'limma_logexpression_performance.R',
    'limma_logexpression_oracle.R', 'limma_workflow_performance.py', 'limma_workflow_oracle.R',
    'edger_workflow_oracle.R', 'deseq_workflow_oracle.R', 'deseq_workflow_performance.py',
    'infercnv_source_snapshot.py', 'infercnv_trial_process.py', 'verify_infercnv_oracle_artifact.py',
    'test_limma_workflow_performance.py')


def load_native_receipt(path=NATIVE_RECEIPT):
    receipt = _load_json(path, 'accepted ordinary native receipt')
    expected = dict(schema_version=1, repository='omics-rust/rsomics-world', run_attempt=1,
        workflow_path='.github/workflows/limma-candidate.yml', snapshot='green-2', product='rsomics-limma',
        owning_commit=OWNER_HEAD,
        source_archive_sha256=ARCHIVE_SHA, source_manifest_sha256=MANIFEST_SHA, lock_sha256=LOCK_SHA,
        source_file_count=31, fit_schema=2, ordinary_workflow_count=5, workers=[1, 4])
    check(all(receipt.get(key) == value for key, value in expected.items()), 'ordinary native admission identity differs')
    check(receipt.get('native_targets') == ['linux-x86_64', 'linux-aarch64', 'macos-x86_64', 'macos-aarch64'] and
          all(isinstance(receipt.get(key), str) and HEX64.fullmatch(receipt[key])
              for key in ('artifact_sha256', 'binary_sha256', 'verification_sha256')) and
          all(type(receipt.get(key)) is int and receipt[key] > 0 for key in ('run_id', 'artifact_id', 'size_in_bytes')) and
          isinstance(receipt.get('head_sha'), str) and len(receipt['head_sha']) == 40 and
          all(c in '0123456789abcdef' for c in receipt['head_sha']), 'ordinary native admission is incomplete')
    check(receipt.get('artifact_name') == f"limma-{receipt['run_id']}-1-linux-x86_64", 'ordinary native artifact name differs')
    return receipt


def select_native(acquisition):
    acquisition = Path(acquisition)
    receipt = load_native_receipt(acquisition / 'receipt.json')
    jobs = _load_json(acquisition / 'jobs.json', 'native required steps')
    check(all(all(any(step['name'] == name and step['status'] == 'completed' and step['conclusion'] == 'success'
                      for step in job['steps']) for name in REQUIRED_STEPS) for job in jobs['jobs']),
          'ordinary native required step is missing or unsuccessful')
    return select_retained_artifact(acquisition, receipt)


def load_checker(path):
    spec = importlib.util.spec_from_file_location('ordinary_complete_output_comparator', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    check(module.BOUNDS == BOUNDS and module.LOG_BOUND == 1e-3 and tuple(module.CASES) == CASES,
          'ordinary fixed source bands/cases changed')
    return module


def check_validation_files(admitted, validation):
    check(set(admitted) == set(validation) and
          all(validation[name] == digest for name, digest in admitted.items() if name != 'scripts/check-logexpression.py'),
          'validation snapshot changed production or other admitted files')
    check(validation['scripts/check-logexpression.py'] != admitted['scripts/check-logexpression.py'],
          'validation snapshot must carry the explicit checker-only reuse extension')


def verify_native_workflow(root, receipt, files):
    digest = receipt['binary_sha256']
    check(_load_json(root / 'source-after.json', 'native source preservation')['unchanged_files'] == 31,
          'admitted native source changed')
    count = _load_json(root / 'worker-determinism.json', 'count regression determinism')
    check(count['passed'] is True and count['compared_workers'] == [1, 4] and count['binary_sha256'] == digest,
          'combined native count regression is incomplete')
    comparison = root / 'ordinary-comparison'
    contract = _load_json(comparison / 'contract.json', 'accepted ordinary contract')
    check(contract['source_run'] == SOURCE['run_id'] and contract['manifest_sha256'] == SOURCE['manifest_sha256'] and
          contract['capture_sha256'] == SOURCE['capture_witness_sha256'] and contract['reload_sha256'] == SOURCE['reload_witness_sha256'] and
          contract['binary_sha256'] == digest and contract['workers'] == [1, 4] and contract['cases'] == list(CASES) and
          contract['bounds'] == {key: list(value) for key, value in BOUNDS.items()} and contract['positive_probability_log_bound'] == 1e-3 and
          contract['controller_sha256'] == files['scripts/check-logexpression.py'] and
          contract['shared_controller_sha256'] == files['scripts/check-workflow.py'], 'accepted ordinary contract differs')
    summary = _load_json(comparison / 'summary.json', 'accepted ordinary coverage')
    check(summary['passed'] is True and summary['errors'] == [] and
          [(row['case'], row['workers'], row['passed']) for row in summary['comparisons']] ==
          [(case, workers, True) for case in CASES for workers in (1, 4)], 'accepted ordinary coverage is incomplete')
    for case in CASES:
        fits = []
        for workers in (1, 4):
            output = comparison / case / f'workers-{workers}'
            report = _load_json(output / 'comparison.json', 'accepted full ordinary comparison')
            commands = json.loads((output / 'commands.json').read_text())
            check(report['passed'] is True and report['source_run'] == SOURCE['run_id'] and
                  json.loads((output / 'failures.json').read_text()) == [] and
                  report['native_bundle_sha256'] == _sha256(output / 'fit.json') and
                  len(commands) == (5 if case in ('two-row', 'one-zero') else 7) and all(row['exit'] == 0 for row in commands),
                  'accepted ordinary full lifecycle failed')
            fit = _load_json(output / 'fit.json', 'accepted ordinary full state')
            check(fit['schema'] == 2 and fit['product'] == 'rsomics-limma' and fit['input_kind'] == 'log-expression' and
                  fit.pop('workers') == workers, 'accepted ordinary state identity differs')
            fits.append(fit)
            check(_load_json(output / 'forward.json', 'accepted initial state') == fit['initial_results'],
                  'accepted initial/reloaded state differs')
        check(fits[0] == fits[1], 'accepted ordinary worker full states differ')
        one, four = comparison / case / 'workers-1', comparison / case / 'workers-4'
        names = ('forward.tsv', 'forward.json', 'expression.tsv', 'fitted.tsv', 'residuals.tsv') if case in ('two-row', 'one-zero') else (*OUTPUTS, *STATES)
        check(all(_sha256(one / name) == _sha256(four / name) for name in names), 'accepted ordinary worker outputs differ')


def prepare_runtime(acquisition, validation_snapshot):
    acquisition = Path(acquisition).resolve(strict=True)
    native, source = acquisition / 'native', acquisition / 'source'
    receipt = load_native_receipt(native / 'receipt.json')
    check(_load_json(native / 'selection.json', 'selected admitted native') == receipt, 'native selection differs')
    _run(native / 'run.json', receipt)
    _artifact(native / 'artifact.json', receipt, native / 'artifact.zip')
    check(_sha256(native / 'artifact.zip') == receipt['artifact_sha256'] and
          (native / 'artifact.zip').stat().st_size == receipt['size_in_bytes'], 'admitted native ZIP differs')
    extract_zip(native / 'artifact.zip', native / 'artifact', 'ordinary-comparison/summary.json')
    root = native / 'artifact'
    for name, digest in {'source.tar.gz': ARCHIVE_SHA, 'files.sha256': MANIFEST_SHA, 'Cargo.lock': LOCK_SHA}.items():
        check(_sha256(root / name) == digest, 'admitted green-2 source differs: ' + name)
    files = verify_and_extract(root, native / 'candidate')
    check(len(files) == 31 and files['Cargo.lock'] == LOCK_SHA, 'admitted production inventory differs')
    binary = root / 'native-binary/rsomics-limma-linux-x86_64'
    digest, name = (binary.parent / 'checksums.sha256').read_text().strip().split('  ', 1)
    check(digest == receipt['binary_sha256'] == _sha256(binary) and Path(name).name == binary.name, 'admitted executable differs')
    check((root / 'host.log').read_text().splitlines()[0] == receipt['head_sha'] and
          'snapshot=green-2' in (root / 'host.log').read_text(), 'admitted native snapshot/head differs')
    verify_native_workflow(root, receipt, files)
    for name in ('check-workflow.py', 'check-logexpression.py'):
        check(_sha256(root / 'controller' / name) == files['scripts/' + name], 'admitted controller differs')
    checker_files = verify_and_extract(Path(validation_snapshot).resolve(strict=True), native / 'validation-candidate')
    check_validation_files(files, checker_files)
    checker = native / 'validation-candidate/scripts/check-logexpression.py'
    module = load_checker(checker)
    check('reuse' in inspect.signature(module.compare_case).parameters, 'validation checker lacks explicit pure reuse')
    check(_load_json(source / 'receipt.json', 'ordinary source receipt') == SOURCE, 'ordinary source receipt differs')
    _run(source / 'run.json', SOURCE)
    _artifact(source / 'artifact.json', SOURCE, source / 'artifact.zip')
    check(_sha256(source / 'artifact.zip') == SOURCE['artifact_sha256'] and
          (source / 'artifact.zip').stat().st_size == SOURCE['size_in_bytes'], 'ordinary source ZIP differs')
    extract_zip(source / 'artifact.zip', source / 'artifact', 'workflow/sha256-manifest.tsv')
    witness = source / 'artifact/workflow'
    module.qualify(witness)
    source_files = {row['path']: row['sha256'] for row in csv.DictReader((witness / 'sha256-manifest.tsv').open(), delimiter='\t')}
    source_tree = acquisition / 'source-tree'
    source_tree.mkdir()
    archive_path = source / 'artifact/limma_3.68.5.tar.gz'
    check(_sha256(archive_path) == PACKAGES['limma']['source_archive_sha256'], 'released limma source differs')
    with tarfile.open(archive_path) as archive:
        seen = set()
        for member in archive.getmembers():
            name = _archive_name(member.name.rstrip('/') + ('/' if member.isdir() else ''), member.isdir())
            check(name not in seen and name.split('/')[0] == 'limma' and (member.isfile() or member.isdir()), 'unsafe limma source archive')
            seen.add(name)
        archive.extractall(source_tree, filter='data')
    binary.chmod(0o755)
    save(acquisition / 'verification.json', dict(native=receipt, owner_head=OWNER_HEAD, binary_sha256=digest,
        native_source_files=files, validation_files=checker_files, validation_snapshot=str(Path(validation_snapshot).resolve()),
        validation_checker_sha256=_sha256(checker), admitted_checker_sha256=files['scripts/check-logexpression.py'],
        checker_only_difference=True, source=SOURCE, source_files=source_files, packages=dict(limma=PACKAGES['limma']),
        bounds=BOUNDS, positive_probability_log_bound=1e-3,
        verification_scope='selected Linux x86_64 ZIP/source/binary/complete ordinary1/4 deeply checked; all four required successful jobs checked; other platform original retention is parent receipt scope'))


def native_commands(witness, output):
    inputs = witness / 'inputs/real'
    forward = ['--contrast=(Intercept)=0', '--contrast=typesingle-read=0', '--contrast=conditiontreated=1']
    bundle = output / 'fit.json'
    commands = [['run', '--expression', inputs / 'expression.tsv', '--samples', inputs / 'samples.tsv',
                 '--design', inputs / 'design.tsv', '--bundle', bundle, *forward]]
    for name, terms in (('forward', forward), ('reverse', forward[:-1] + ['--contrast=conditiontreated=-1']),
                        ('mixed', [forward[0], '--contrast=typesingle-read=-1', forward[2]])):
        commands.append(['results', '--bundle', bundle, '--output', output / f'{name}.tsv', '--state', output / f'{name}.json', *terms])
    commands += [['export', '--bundle', bundle, '--output', output / f'{name}.tsv', '--kind', name]
                 for name in ('expression', 'fitted', 'residuals')]
    return commands


def output_manifest(output, source=False):
    states = ('fit.rds', 'forward.rds', 'reverse.rds', 'mixed.rds') if source else ('fit.json', *STATES)
    return {name: dict(sha256=_sha256(output / name), size_bytes=(output / name).stat().st_size) for name in (*states, *OUTPUTS)}


def native_trial(directory, witness, binary, checker, workers, cores, accepted):
    directory.mkdir()
    output = directory / 'output'
    output.mkdir()
    rows = [measured([binary, '--json', '--threads', str(workers), *command], directory / f'command-{index}', cores)
            for index, command in enumerate(native_commands(witness, output))]
    original = output_manifest(output)
    save(directory / 'timed-output-manifest.json', original)
    fitted = _load_json(output / 'fit.json', 'measured ordinary full state')
    expected = _load_json(accepted / 'fit.json', 'admitted same-binary state')
    check(fitted.pop('workers') == workers and expected.pop('workers') == workers, 'measured worker identity differs')
    save(directory / 'cross-host-binding.json', dict(diagnostic_only=True, full_state_equal=fitted == expected,
        accepted_bundle_sha256=_sha256(accepted / 'fit.json'), measured_bundle_sha256=original['fit.json']['sha256'],
        output_bytes_equal={name: _sha256(accepted / name) == original[name]['sha256'] for name in (*STATES, *OUTPUTS)}))
    validation = directory / 'validation'
    run_child([sys.executable, '-B', str(checker), '--binary', str(binary), '--witness', str(witness), '--output', str(validation),
               '--reuse-from', str(output), '--case', 'real', '--workers', str(workers)], directory / 'validation-process', timeout=1800)
    verified = _load_json(validation / 'summary.json', 'pure measured-output validation')
    check(verified['passed'] is True and verified['native_commands_executed'] == 0, 'measured native validation failed or ran commands')
    check(output_manifest(output) == original, 'pure validation changed measured outputs')
    result = aggregate_native(rows)
    result.update(workers=workers, outputs=original, comparison_passed=True,
        stages=dict(zip(('run', 'forward', 'reverse', 'mixed', 'expression', 'fitted', 'residuals'), rows)))
    save(directory / 'measurement.json', result)
    return fitted, result


def source_trial(directory, witness, qualification, cores):
    directory.mkdir()
    output = directory / 'output'
    driver = ROOT / 'scripts/limma_logexpression_performance.R'
    row = measured(['Rscript', '--vanilla', driver, 'run', witness, output, qualification], directory / 'command', cores)
    original = output_manifest(output, source=True)
    save(directory / 'timed-output-manifest.json', original)
    timing = _load_json(output / 'inner-timing.json', 'source inner timer')
    for name in ('inner_elapsed_seconds', 'inner_user_seconds', 'inner_system_seconds'):
        check(type(timing[name]) in (int, float) and math.isfinite(timing[name]) and timing[name] >= 0, 'invalid source inner timing')
        row[name] = timing[name]
    validation = directory / 'validation'
    try:
        run_child(['Rscript', '--vanilla', str(driver), 'inspect', str(output), str(validation), str(qualification), str(witness)],
                  directory / 'inspection-process', timeout=1800)
    except TrialProcessError as error:
        raise TrialProcessError(f'source trial {directory.name}: untimed source inspection failed ({error}); '
            f'stderr={directory / "inspection-process.stderr"}; failures={validation / "failures.json"}') from error
    report = _load_json(validation / 'comparison.json', 'actual measured source comparison')
    check(report['passed'] is True and report['bounds'] == {key: list(pair) for key, pair in BOUNDS.items()} and
          report['positive_probability_log_bound'] == 1e-3, 'measured source compatibility failed')
    check(output_manifest(output, source=True) == original, 'untimed inspection changed source outputs')
    row.update(outputs=original, comparison_passed=True, discoveries=report['discoveries'])
    save(directory / 'measurement.json', row)
    return row


def run_benchmark(acquisition, source_tree, output):
    acquisition, source_tree = Path(acquisition).resolve(strict=True), Path(source_tree).resolve(strict=True)
    record = _load_json(acquisition / 'verification.json', 'admitted ordinary acquisition')
    check(record['native'] == load_native_receipt(acquisition / 'native/receipt.json') and record['checker_only_difference'] is True,
          'measurement lacks exact accepted native/checker-only admission')
    check(platform.system() == 'Linux' and platform.machine() == 'x86_64', 'measurement requires native Linux x86_64')
    allowed = sorted(os.sched_getaffinity(0))
    check(len(allowed) >= 4, 'four available logical CPUs are required')
    os.sched_setaffinity(0, allowed[:4])
    cores = ','.join(map(str, allowed[:4]))
    for name in ('OMP_NUM_THREADS', 'OPENBLAS_NUM_THREADS', 'MKL_NUM_THREADS'):
        os.environ[name] = '1'
    output = Path(output).absolute()
    output.mkdir()
    witness, native = acquisition / 'source/artifact/workflow', acquisition / 'native/artifact'
    binary = native / 'native-binary/rsomics-limma-linux-x86_64'
    checker = acquisition / 'native/validation-candidate/scripts/check-logexpression.py'
    check(_sha256(binary) == record['binary_sha256'] and _sha256(checker) == record['validation_checker_sha256'], 'admitted executable/validator changed')
    for prefix, files in ((acquisition / 'native/candidate', record['native_source_files']),
                          (acquisition / 'native/validation-candidate', record['validation_files']),
                          (witness, record['source_files'])):
        check(all(_sha256(prefix / name) == digest for name, digest in files.items()), 'admitted source/input bytes changed before timing')
    check_validation_files(record['native_source_files'], record['validation_files'])
    load_checker(checker).qualify(witness)
    qualification = output / 'source-qualification'
    run_child(['Rscript', '--vanilla', str(ROOT / 'scripts/limma_logexpression_performance.R'), 'qualify',
               str(source_tree / 'limma'), str(qualification)], output / 'qualification-process', timeout=1800)
    runtime = _load_json(qualification / 'runtime.json', 'qualified source runtime')
    immutable = {}
    for tree in (acquisition, source_tree, qualification):
        immutable.update({path: _sha256(path) for path in tree.rglob('*') if path.is_file()})
    with (qualification / 'installed-limma-sha256.tsv').open(newline='') as stream:
        immutable.update({Path(runtime['limma']['installed_path']) / row['path']: row['sha256'] for row in csv.DictReader(stream, delimiter='\t')})
    immutable.update({ROOT / 'scripts' / name: _sha256(ROOT / 'scripts' / name) for name in CONTROLLERS})
    workflow = ROOT / '.github/workflows/limma-performance.yml'
    immutable[workflow] = _sha256(workflow)
    def verify_immutable():
        check(all(_sha256(path) == digest for path, digest in immutable.items()), 'source/input/binary/controller bytes changed')
    verify_immutable()
    save(output / 'immutability-before.json', {str(path): digest for path, digest in immutable.items()})
    save(output / 'bounds.json', BOUNDS)
    save(output / 'contract.json', dict(native=record['native'], owner_head=OWNER_HEAD, source=SOURCE, bounds=BOUNDS,
        validation_checker_sha256=record['validation_checker_sha256'], checker_only_difference=True,
        source_warmups=1, source_repetitions=3, native_warmups_per_worker=1, native_repetitions_per_worker=5,
        workers=[1, 4], native_order='alternate [1,4] and [4,1]', features=8066, samples=7, input_kind='supplied unweighted log-expression',
        units='sum wall/CPU/I/O/faults across seven actual native CLI processes; RSS=max process KiB, not live aggregate; filesystem counters=GNUtime blocks',
        encodings='native portable uncompressed JSON; source default gzip full analysis RDS + three derived RDS; each six TSVs',
        source_inner='input TSV read/join and named contrasts, one lmFit/initial forward EB/table, durable complete save/reload, three states/tables, E/fitted/residual TSVs; excludes package loading/identity/inspection',
        validation='already measured complete native outputs only, zero native validation commands; source actual serialized state inspection with no fit',
        throughput_rule='source inner minimum > native complete-path maximum OR source process RSS minimum > native maximum; scaling separate native1 minimum > native4 maximum'))
    save(output / 'host.json', dict(platform=platform.platform(), machine=platform.machine(), cpu_affinity=list(os.sched_getaffinity(0)),
        cpuinfo=Path('/proc/cpuinfo').read_text(), meminfo=Path('/proc/meminfo').read_text(), load_average=os.getloadavg(),
        disk=shutil.disk_usage(output)._asdict(), runtime=runtime, time_format=TIME_FORMAT,
        native_rustc=(native / 'rustc.log').read_text(), native_cargo=(native / 'cargo.log').read_text()))
    save(output / 'input-bytes.json', {path.name: dict(size_bytes=path.stat().st_size, sha256=_sha256(path))
        for path in (witness / 'inputs/real').iterdir() if path.is_file()})
    source, trials, baseline, hashes = [], {1: [], 4: []}, None, None
    source_baseline, source_bytes = None, []
    try:
        source_baseline = source_trial(output / 'source-warmup', witness, qualification, cores)['outputs']
        for workers in (1, 4):
            fit, row = native_trial(output / f'native-warmup-{workers}', witness, binary, checker, workers, cores,
                                    native / f'ordinary-comparison/real/workers-{workers}')
            current = {name: row['outputs'][name]['sha256'] for name in (*STATES, *OUTPUTS)}
            if baseline is None:
                baseline, hashes = fit, current
            check(fit == baseline and current == hashes, 'same-host native warmup complete state/outputs differ')
        for index in range(5):
            if index < 3:
                row = source_trial(output / f'source-{index+1}', witness, qualification, cores)
                source_bytes.append({name: row['outputs'][name] == source_baseline[name] for name in source_baseline})
                source.append(row)
            for workers in ((1, 4) if index % 2 == 0 else (4, 1)):
                fit, row = native_trial(output / f'native-{index+1}-workers-{workers}', witness, binary, checker, workers, cores,
                                       native / f'ordinary-comparison/real/workers-{workers}')
                check(fit == baseline and all(row['outputs'][name]['sha256'] == hashes[name] for name in hashes),
                      'same-host native complete state/outputs differ')
                trials[workers].append(row)
            verify_immutable()
    finally:
        verify_immutable()
        save(output / 'immutability-after.json', dict(unchanged=True, checked_files=len(immutable)))
    statistics = dict(source=distribution(source), native={str(workers): distribution(rows) for workers, rows in trials.items()})
    decision = performance_decision(statistics)
    summary = dict(status='complete' if decision['advantage_passed'] else 'rejected_no_advantage', native=record['native'],
        source=SOURCE, binary_sha256=record['binary_sha256'], statistics=statistics, decision=decision,
        native_stages={str(workers): {name: distribution([row['stages'][name] for row in rows]) for name in rows[0]['stages']}
                       for workers, rows in trials.items()}, source_trials=source, native_trials=trials,
        native_complete_state_and_outputs_equal=True, native_exports_sha256=hashes,
        source_output_bytes_diagnostic=source_bytes, source_outputs=source_baseline,
        scope='one real8066x7 supplied-E complete user lifecycle; serial pinned R versus admitted native1/4; not a pure fit kernel, independent microarray, broad cohort/scaling or release acceptance')
    save(output / 'summary.json', summary)
    check(decision['advantage_passed'], 'complete ordinary lifecycle measured but no strict replacement advantage')
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    for name in ('select-native', 'prepare-runtime', 'run'):
        child = sub.add_parser(name)
        child.add_argument('--acquisition', required=True, type=Path)
        if name == 'prepare-runtime':
            child.add_argument('--validation-snapshot', required=True, type=Path)
        if name == 'run':
            child.add_argument('--source-tree', required=True, type=Path)
            child.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    try:
        if args.command == 'select-native':
            print(select_native(args.acquisition)['artifact_id'])
        elif args.command == 'prepare-runtime':
            prepare_runtime(args.acquisition, args.validation_snapshot)
        else:
            summary = run_benchmark(args.acquisition, args.source_tree, args.output)
            print(json.dumps(dict(status=summary['status'], decision=summary['decision'])))
    except (OSError, ValueError, KeyError, TrialProcessError, tarfile.TarError) as error:
        if args.command == 'run' and args.output.is_dir() and not (args.output / 'failure.json').exists():
            save(args.output / 'failure.json', dict(error=f'{type(error).__name__}: {error}'))
        parser.exit(1, f'complete ordinary workflow measurement failed: {error}\n')


if __name__ == '__main__':
    main()
