"""Manual independent-metric beta gate; preparation does not earn performance."""
import argparse
from array import array
import json
import math
import os
from pathlib import Path
import platform
import resource
import shutil
import signal
import stat
import statistics
import struct
import subprocess
import sys
import tarfile
import tempfile
import time
import traceback
import urllib.request
import zipfile

import ecology_whole_real as whole
import private_artifact as transport
from ecology_candidate import check, load, save, portable_names, _hash_file, check_tree

ROOT = Path(__file__).resolve().parents[1]
ASSET = 'assets/ecology-beta-real-20261101.json'
WORKFLOW = '.github/workflows/ecology-beta-real.yml'
METRICS = ('braycurtis', 'jaccard', 'euclidean', 'canberra', 'cityblock',
           'unweighted_unifrac', 'weighted_unifrac', 'weighted_normalized_unifrac')
identity = whole.identity


def read_policy(world=ROOT):
    world = Path(world)
    policy = load(world / ASSET)
    old = load(world / 'assets/ecology-whole-real-20261101.json')
    check(policy['schema'] == 'rsomics.ecology.beta-real-policy.v1'
        and policy['owner_commit'] == '111adf5e8a3657a99c50a9832ca5ed9c53fd98d7'
        and policy['owner_tree'] == 'd1963110fbee9b1c18b999a3a6e09d94dd37f977'
        and tuple(policy['policy']['metrics']) == METRICS
        and all(policy[key] == old[key] for key in ('study', 'python', 'image', 'recipient', 'original_source', 'attribution'))
        and policy['source_runtime']['wheels'] == old['source_runtime']['wheels']
        and len(policy['source_runtime']['wheels']) == 27
        and policy['source_profile']['name'] == 'contiguous-public'
        and policy['source_profile']['layout_copy_time_and_rss_charged'] is True
        and policy['source_profile']['upstream_default_memory_claim'] is False,
        'adopted beta source/input/profile differs')
    authority = policy['native_authority']
    check(identity(world / authority['path']) == {key: authority[key] for key in ('bytes', 'sha256')},
          'beta native authority differs')
    source = policy['input_receipt']
    check(identity(world / source['path']) == {key: source[key] for key in ('bytes', 'sha256')},
          'accepted full-input receipt differs')
    p = policy['policy']
    check((p['source_timeout_seconds'], p['native_timeout_seconds'], p['controller_deadline_seconds'],
        p['max_matrix_bytes'], p['memory_cgroup_bytes'], p['abs_tolerance'], p['rel_tolerance']) ==
        (1800, 300, 18600, 64 * 2**20, 12 * 2**30, 1e-10, 1e-10), 'fixed beta limits differ')
    check((p['admission_deadline_seconds'], p['comparison_budget_seconds'], p['finalization_seconds'],
        p['workflow_timeout_seconds'], p['initial_free_bytes'], p['retained_limit_bytes'], p['tar_bound_bytes']) ==
        (1200, 600, 1200, 21600, 22 * 2**30, 6 * 2**30, 7 * 2**30)
        and p['budgets'] == dict(inputs=256 * 2**20, runtime=3072 * 2**20, science=1536 * 2**20, metadata=1280 * 2**20)
        and policy['native_receipt']['path'] == '.autopilot/oracles/ecology-native-beta-2026-11-01.json'
        and policy['native_receipt']['sha256'] == '048089f8ec2814670c75b584f51cc7005533df4df341af18db79a5c9798ece45',
        'fixed admission/retention/native identity differs')
    return policy


def validate_native_receipt(world=ROOT, relative=None, expected_sha=None):
    world, policy = Path(world), read_policy(world)
    pin = policy['native_receipt']
    relative = pin['path'] if relative is None else relative
    check(relative == pin['path'] and expected_sha in (None, pin['sha256'])
          and _hash_file(world / relative) == pin['sha256'], 'actual beta native receipt differs')
    receipt = load(world / relative)
    check(receipt['schema'] == 'rsomics.ecology.native-adoption.v1' and receipt['status'] == 'accepted'
          and receipt['repository'] == 'omics-rust/rsomics-world'
          and receipt['workflow_path'] == '.github/workflows/ecology-candidate.yml'
          and receipt['native_four_qualified'] is True and receipt['root_fresh_review_required'] is False
          and receipt['source_files'] == 120 and receipt['captured_fixture_files'] == 86
          and receipt['tests_passed_each_mode_each_target'] == 6
          and set(receipt['targets']) == whole.TARGETS, 'new actual four-native qualification is absent')
    for key in ('owner_commit', 'owner_tree', 'lock_sha256', 'source_archive_sha256', 'source_manifest_sha256'):
        check(receipt[key] == policy[key], 'native source differs: ' + key)
    for row in receipt['targets'].values():
        check(all(row['test_summaries'][mode]['passed'] == 6
                  and row['test_summaries'][mode]['captured_fixture_files'] == 86
                  for mode in ('debug', 'release')), 'native test/fixture population differs')
    return receipt


def retention_plan():
    plan = transport.finalization_space(6 * 2**30, 7 * 2**30)
    return dict(retained_limit_bytes=6 * 2**30, tar_bound_bytes=7 * 2**30,
                initial_free_bytes=22 * 2**30, **plan)


def schedule():
    base = [('source', 1), ('native', 1), ('native', 4)]
    rows = [dict(phase=phase, pair=0, engine=e, workers=n)
            for phase in ('qualification', 'warmup') for e, n in base]
    for pair in range(1, 6):
        rows += [dict(phase='measured', pair=pair, engine=e, workers=n)
                 for e, n in (base if pair % 2 else list(reversed(base)))]
    return rows


def reserve_call(started, now, timeout, policy):
    p = policy['policy']
    check(now + timeout <= started + p['controller_deadline_seconds']
        and now + timeout + p['finalization_seconds'] <= started + 330 * 60,
        'insufficient full call allowance plus finalization; HOLD')


def evaluate(pairs):
    check(len(pairs) == 5, 'five complete measured rounds required')
    values = {engine: {key: [row[engine][key] for row in pairs]
              for key in ('wall', 'rss_bytes', 'cpu')} for engine in ('source', 'native1', 'native4')}
    check(all(math.isfinite(v) and v > 0 for engine in values.values()
              for group in engine.values() for v in group), 'missing/nonfinite resource measurement')
    med = lambda engine, key: statistics.median(values[engine][key])
    wall, rss, cpu = (med('native1', k) / med('source', k) for k in ('wall', 'rss_bytes', 'cpu'))
    disjoint_wall = max(values['native1']['wall']) < min(values['source']['wall'])
    disjoint_rss = max(values['native1']['rss_bytes']) < min(values['source']['rss_bytes'])
    throughput = wall <= .80 and disjoint_wall
    memory = rss <= .75 and disjoint_rss and wall <= 1.05
    return dict(passed=throughput or memory, throughput=throughput, memory=memory,
        wall_ratio=wall, rss_ratio=rss, cpu_ratio=cpu, disjoint_wall=disjoint_wall,
        disjoint_rss=disjoint_rss, native4_scaling=med('native1', 'wall') / med('native4', 'wall'),
        distributions=values, primary='Native1/Source1 ratio of medians',
        native4_cannot_rescue_primary=True, rss_is_process_peak_not_exact_aggregate=True,
        io_counters_are_not_exact_storage_bytes=True, cold_page_cache_proven=False)


def label_overhead(samples):
    size = len(('\t' + '\t'.join(samples) + '\n').encode()) + sum(len(s.encode()) + 1 for s in samples)
    check(size < 2**20, 'complete matrix ID/header overhead exceeds1MiB')
    return size


def read_matrix(path, samples):
    path, n = Path(path), len(samples)
    check(path.stat().st_size <= 64 * 2**20, 'matrix exceeds64MiB; partial original retained')
    data = array('d')
    with path.open() as stream:
        check(stream.readline().rstrip('\n').split('\t') == ['', *samples], 'matrix header/order differs')
        for i, sample in enumerate(samples):
            fields = stream.readline().rstrip('\n').split('\t')
            check(len(fields) == n + 1 and fields[0] == sample, 'matrix row/shape/order differs')
            row = array('d', map(float, fields[1:]))
            check(all(math.isfinite(v) and v >= 0 for v in row) and row[i] == 0,
                  'matrix is nonfinite/negative/nonhollow; HOLD')
            data.extend(row)
        check(stream.read() == '', 'extra matrix row')
    check(all(data[i * n + j] == data[j * n + i] for i in range(n) for j in range(i + 1, n)),
          'matrix is not symmetric')
    return data


def compare_matrix(reference, actual, samples, native_reference=None):
    source, observed = read_matrix(reference, samples), read_matrix(actual, samples)
    maximum = 0.0
    for expected, value in zip(source, observed, strict=True):
        difference = abs(value - expected)
        check(difference <= 1e-10 + 1e-10 * abs(expected), 'fixed beta numerical band exceeded; HOLD')
        maximum = max(maximum, difference)
    if native_reference is not None:
        check(identity(actual) == identity(native_reference), 'native worker/repetition bytes differ; HOLD')
    return dict(cells=len(source), maximum_absolute_error=maximum, exact_labels_shape=True,
                finite_nonnegative_symmetric_hollow=True)


def capacity(private, policy=None):
    private = Path(private)
    policy = read_policy() if policy is None else policy
    p, buckets, logs = policy['policy'], dict(inputs=0, runtime=0, science=0, metadata=0), 0
    for path in private.rglob('*'):
        try:
            observed = path.lstat()
        except FileNotFoundError:
            continue
        if not stat.S_ISREG(observed.st_mode):
            continue
        first = path.relative_to(private).parts[0]
        raw = path.name in ('stdout.txt', 'stderr.txt') or path.name.startswith('controller.')
        bucket = {'data': 'inputs', 'runtime': 'runtime'}.get(first, 'metadata')
        if first in ('runs', 'bridge') and not raw and path.suffix not in ('.json', '.txt'):
            bucket = 'science'
        size = observed.st_size
        buckets[bucket] += size
        if bucket == 'metadata' and raw:
            logs += size
    check(all(buckets[k] <= p['budgets'][k] for k in buckets)
          and sum(buckets.values()) <= p['retained_limit_bytes']
          and logs <= p['raw_log_budget_bytes']
          and buckets['metadata'] - logs <= p['other_metadata_budget_bytes'], 'retention quota reached; HOLD, no deletion-to-fit')
    check(shutil.disk_usage(private).free >= retention_plan()['required_free_bytes'], 'cipher/headroom reserve reached; HOLD')
    return dict(buckets=buckets, raw_logs=logs)


def capture(root, name, argv, *, workers, timeout, private=None, file_limit=None):
    output = Path(root) / name
    output.mkdir(parents=True, exist_ok=False)
    cpus = sorted(os.sched_getaffinity(0))[:workers] if hasattr(os, 'sched_getaffinity') else None
    check(cpus is None or len(cpus) == workers, 'required CPU allowance is unavailable')
    command, requested = list(map(str, argv)), time.monotonic_ns()
    save(output / 'request.json', dict(argv=command, workers=workers, cpus=cpus, timeout=timeout,
        file_limit=file_limit, requested_monotonic_ns=requested, owns_process_group=True))
    process, status, counters, failure = None, None, None, None
    started, finished = requested, None
    sampled, samples, next_capacity, next_sample = 0, 0, time.monotonic() + 1, time.monotonic()
    events = Path('/sys/fs/cgroup/memory.events')
    before_events = events.read_text() if events.exists() else None
    def setup():
        if cpus is not None:
            os.sched_setaffinity(0, cpus)
        if file_limit is not None:
            resource.setrlimit(resource.RLIMIT_FSIZE, (file_limit, file_limit))
    def reap(flags):
        nonlocal status, counters, finished
        pid, code, usage = os.wait4(process.pid, flags)
        if not pid:
            return False
        process.returncode = status = os.waitstatus_to_exitcode(code)
        finished = time.monotonic_ns()
        counters = dict(user_seconds=usage.ru_utime, system_seconds=usage.ru_stime,
            rss_bytes=whole.rss_bytes(usage), input_blocks=usage.ru_inblock, output_blocks=usage.ru_oublock,
            minor_faults=usage.ru_minflt, major_faults=usage.ru_majflt)
        return True
    with (output / 'stdout.txt').open('xb') as stdout, (output / 'stderr.txt').open('xb') as stderr:
        try:
            started = time.monotonic_ns()
            process = subprocess.Popen(command, stdout=stdout, stderr=stderr, start_new_session=True, preexec_fn=setup)
            try:
                actual_cpus = sorted(os.sched_getaffinity(process.pid)) if cpus is not None else None
            except ProcessLookupError:
                actual_cpus = None
            check(actual_cpus is None or actual_cpus == cpus, 'actual child affinity differs')
            save(output / 'process.json', dict(pid=process.pid, process_group=process.pid, actual_affinity=actual_cpus))
            while not reap(os.WNOHANG):
                check((time.monotonic_ns() - started) / 1e9 <= timeout, 'owned child timeout; HOLD')
                if private is not None and time.monotonic() >= next_capacity:
                    capacity(private)
                    next_capacity = time.monotonic() + 1
                if platform.system() == 'Linux' and time.monotonic() >= next_sample:
                    sampled = max(sampled, sum(row['rss_bytes'] for row in whole.proc_group(process.pid)))
                    samples += 1
                    next_sample = time.monotonic() + .1
                time.sleep(.01)
        except Exception:
            failure = traceback.format_exc()
        finally:
            if process is not None and process.returncode is None:
                try:
                    os.killpg(process.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
                end = time.monotonic() + 10
                while not reap(os.WNOHANG) and time.monotonic() < end:
                    time.sleep(.1)
                if process.returncode is None:
                    try:
                        os.killpg(process.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    reap(0)
    result = dict(returncode=status, signal=-status if status is not None and status < 0 else None,
        wall=((finished or time.monotonic_ns()) - started) / 1e9, raw_wait4=counters, failure=failure,
        started_monotonic_ns=started, finished_observed_monotonic_ns=finished, wait_poll_seconds=.01,
        memory_events_before=before_events, memory_events_after=events.read_text() if events.exists() else None,
        requested_workers=workers, selected_cpus=cpus, sampled_process_group_rss_peak_bytes=sampled,
        sampled_rss_is_not_exact_aggregate=True, rss_sample_count=samples,
        stdout=identity(output / 'stdout.txt'), stderr=identity(output / 'stderr.txt'))
    save(output / 'status.json', result)
    if status != 0 or failure:
        raise RuntimeError('required beta command failed; HOLD: ' + name)
    return result


def source_command(private, python, action, metric, name, timeout, output=None):
    argv = [python, '-B', ROOT / 'scripts/ecology_beta_source.py', action, '--session', private, '--metric', metric]
    if output is not None:
        argv += ['--output', output]
    return capture(private / 'calls', name, argv, workers=1, timeout=timeout, private=private)


def admit_native(private, policy):
    receipt = validate_native_receipt()
    shutil.copy2(ROOT / policy['native_receipt']['path'], private / 'native-receipt.json')
    row = receipt['targets']['linux-x86_64']
    directory = private / 'runtime/native-artifact'
    artifact = dict(artifact_id=row['artifact_id'], artifact_name=row['artifact_name'], **row['artifact'])
    carrier = whole.acquire_action(directory, receipt, artifact)
    jobs = whole.api(f"actions/runs/{receipt['run_id']}/attempts/{receipt['run_attempt']}/jobs?per_page=100",
                     directory / 'jobs.json', 2**20)
    check(jobs['total_count'] == len(jobs['jobs']) == 4 and {j['name'] for j in jobs['jobs']} == whole.TARGETS,
          'complete four-native jobs absent')
    for job in jobs['jobs']:
        check(job['status'] == 'completed' and job['conclusion'] == 'success'
            and job['head_sha'] == receipt['head_sha'] and job['run_attempt'] == receipt['run_attempt']
            and job['id'] == receipt['targets'][job['name']]['job_id'], 'actual native job differs')
    whole.safe_zip_extract(carrier, directory / 'zip', 64 * 2**20)
    archive_path = directory / 'zip/ecology-evidence.tar'
    check(identity(archive_path) == row['tar'], 'actual complete native TAR differs')
    bodies = directory / 'bodies'
    bodies.mkdir()
    with tarfile.open(archive_path) as archive:
        members = archive.getmembers()
        portable_names([m.name for m in members])
        check(all(m.isfile() and m.type == tarfile.REGTYPE for m in members)
              and sum(m.size for m in members) < 64 * 2**20, 'native archive types/size differ')
        model = json.load(archive.extractfile('ecology-evidence/artifact-manifest.json'))
        check(model['schema'] == 'rsomics.ecology.native-four.v1', 'native manifest schema differs')
        expected = {'ecology-evidence/' + name: value for name, value in model['files'].items()}
        check(set(expected) | {'ecology-evidence/artifact-manifest.json'} == {m.name for m in members},
              'complete native member closure differs')
        for member in members:
            target = bodies / member.name
            target.parent.mkdir(parents=True, exist_ok=True)
            with archive.extractfile(member) as source, target.open('xb') as out:
                shutil.copyfileobj(source, out)
            target.chmod(member.mode & 0o777)
            if member.name in expected:
                check(identity(target) == {k: expected[member.name][k] for k in ('bytes', 'sha256')}
                    and stat_mode(target) == expected[member.name]['mode'], 'native original member differs')
    candidate = bodies / 'ecology-evidence/candidate'
    selection = load(candidate / 'selection.json')
    check(_hash_file(candidate / 'selection.json') == row['binary']['selection_sha256'], 'Cargo selection differs')
    for key in ('owner_commit', 'owner_tree', 'lock_sha256', 'source_archive_sha256', 'source_manifest_sha256'):
        check(selection[key] == policy[key], 'native selected source differs: ' + key)
    authority = load(ROOT / policy['native_authority']['path'])
    check_tree(candidate / 'source', {n: r['sha256'] for n, r in authority['files'].items()})
    check(selection['authority_sha256'] == policy['native_authority']['sha256'], 'native beta authority differs')
    binary = candidate / 'native-binary/rsomics-ecology-linux-x86_64'
    check(identity(binary) == {k: row['binary'][k] for k in ('bytes', 'sha256')}
          and load(binary.parent / 'identity.json') == row['binary']
          and stat_mode(binary) == row['binary']['mode'], 'actual Linux release body differs')
    with binary.open('rb') as stream:
        header = stream.read(32)
    check(header[:6] == b'\x7fELF\x02\x01' and struct.unpack_from('<H', header, 18)[0] == 62, 'actual x86_64 ELF absent')
    capture(private / 'calls', 'native-loader', ['ldd', binary], workers=1, timeout=30, private=private)
    capture(private / 'calls', 'native-version', [binary, '--version'], workers=1, timeout=30, private=private)
    check((private / 'calls/native-version/stdout.txt').read_text().strip() == 'rsomics-ecology 0.1.0', 'Bookworm native ABI/version differs')
    save(private / 'native-admission.json', dict(identity=identity(binary), actual_bookworm_execution=True,
        native_rebuilt=False, original_beta_elf=True))
    return binary


def stat_mode(path):
    return path.stat().st_mode & 0o777


def prepare_runtime(private, policy, metric):
    runtime, source = private / 'runtime', policy['original_source']
    artifact = dict(artifact_id=source['artifact_id'], artifact_name=source['artifact_name'],
                    bytes=source['size_in_bytes'], sha256=source['artifact_sha256'])
    carrier = whole.acquire_action(runtime / 'source-artifact', source, artifact)
    whole.safe_zip_extract(carrier, runtime / 'original-source', 2**30)
    old = runtime / 'original-source'
    check(_hash_file(old / 'witness/witness.json') == source['witness_sha256']
        and _hash_file(old / 'witness/sha256-manifest.tsv') == source['manifest_sha256'], 'original source witness differs')
    spec = policy['python']
    original = runtime / spec['filename']
    whole.checked_download(spec, original)
    checksum = runtime / (spec['filename'] + '.sha256')
    whole.download_request(urllib.request.Request(spec['checksum_url']), checksum, 4096)
    check(checksum.read_text().split()[0] == spec['sha256'], 'official Python checksum differs')
    whole.extract_archive(original, runtime / 'interpreter', 2**30)
    capture(private / 'calls', 'create-venv', [runtime / 'interpreter/python/bin/python3.12', '-m', 'venv', runtime / 'venv'],
            workers=1, timeout=120, private=private)
    python = runtime / 'venv/bin/python'
    lock = runtime / 'requirements.lock'
    with lock.open('x') as stream:
        for name, wheel in policy['source_runtime']['wheels'].items():
            path = old / 'wheels' / wheel['filename']
            check(identity(path) == {k: wheel[k] for k in ('bytes', 'sha256')}, 'original wheel differs')
            with zipfile.ZipFile(path) as archive:
                import hashlib
                check(hashlib.sha256(archive.read(wheel['metadata_member'])).hexdigest() == wheel['metadata_sha256'],
                      'original wheel metadata differs')
            stream.write(f'{name}=={wheel["version"]} --hash=sha256:{wheel["sha256"]}\n')
    capture(private / 'calls', 'install-original-wheels', [python, '-m', 'pip', 'install', '--no-index', '--find-links',
        old / 'wheels', '--only-binary=:all:', '--no-deps', '--no-cache-dir', '--require-hashes', '-r', lock],
        workers=1, timeout=600, private=private)
    for name, args in [('pip-check', ['check']), ('pip-inspect', ['inspect', '--local'])]:
        capture(private / 'calls', name, [python, '-m', 'pip', *args], workers=1, timeout=60, private=private)
    source_command(private, python, 'admit', metric, 'admit-source-beta', 120)
    return python


def controller_paths():
    policy = read_policy()
    return sorted(set(whole.controller_paths()) | {'scripts/ecology_beta_real.py', 'scripts/ecology_beta_source.py',
        'scripts/test_ecology_beta_real.py', ASSET, WORKFLOW, policy['native_authority']['path'],
        policy['native_receipt']['path'], policy['input_receipt']['path'],
        'docs/plans/2026-11-01-ecology-beta-real-performance.md'})


def bind_context(private):
    for name in controller_paths():
        destination = private / 'context' / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(ROOT / name, destination)
    save(private / 'context-bindings.json', {name: identity(ROOT / name) for name in controller_paths()})


def hosted():
    policy = read_policy()
    check(os.environ.get('GITHUB_ACTIONS') == 'true' and os.environ.get('GITHUB_REPOSITORY') == 'omics-rust/rsomics-world'
        and os.environ.get('GITHUB_EVENT_NAME') == 'workflow_dispatch'
        and platform.system() == 'Linux' and platform.machine() == 'x86_64'
        and os.environ.get('ECOLOGY_CONTAINER_DIGEST') == policy['image']['manifest_digest']
        and os.environ.get('GCC_VERSION') == '14.3.0', 'actual pinned manual hosted Linux execution required')
    session = Path(os.environ['ECOLOGY_SESSION']).resolve(strict=True)
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    check(session != runner and session.is_relative_to(runner) and session.stat().st_dev == runner.stat().st_dev,
          'private root escapes actual runner device')
    for name in ('TMPDIR', 'XDG_CACHE_HOME', 'PIP_CACHE_DIR', 'CARGO_HOME', 'CARGO_TARGET_DIR', 'RUSTUP_HOME',
                 'PYTHONUSERBASE', 'PYTHONPYCACHEPREFIX', 'MPLCONFIGDIR'):
        path = Path(os.environ[name]).resolve(strict=True)
        check(path.is_relative_to(session) and path.stat().st_dev == session.stat().st_dev, 'cache escapes private root')
    os.sched_setaffinity(0, list(map(int, os.environ['ECOLOGY_CPUS'].split(','))))
    return session, session / 'private', policy


def initialize(metric):
    check(os.environ.get('GITHUB_ACTIONS') == 'true' and platform.system() == 'Linux', 'hosted init required')
    started = time.monotonic()
    def admission_timeout(signum, frame):
        raise TimeoutError('initial admission exceeded20min; HOLD')
    signal.signal(signal.SIGALRM, admission_timeout)
    signal.setitimer(signal.ITIMER_REAL, 1200)
    policy = read_policy()
    check(metric in METRICS and os.environ.get('ECOLOGY_BETA_RECOVERY_ADMITTED') == 'true',
          'fixed metric and prior root recovery attestation required; boolean is not storage proof')
    session = Path(tempfile.mkdtemp(prefix='ecology-beta-' + metric + '-', dir=os.environ['RUNNER_TEMP']))
    for name in ('private/runtime', 'private/metadata', 'cipher'):
        (session / name).mkdir(parents=True, mode=0o700)
    cpus = sorted(os.sched_getaffinity(0))[:4]
    values = dict(ECOLOGY_SESSION=str(session), ECOLOGY_PRIVATE=str(session / 'private'),
                  ECOLOGY_CPUS=','.join(map(str, cpus)), ECOLOGY_BETA_METRIC=metric)
    for key in ('TMPDIR', 'XDG_CACHE_HOME', 'PIP_CACHE_DIR', 'CARGO_HOME', 'CARGO_TARGET_DIR', 'RUSTUP_HOME',
                'PYTHONUSERBASE', 'PYTHONPYCACHEPREFIX', 'MPLCONFIGDIR'):
        path = session / 'private/cache' / key
        path.mkdir(parents=True)
        values[key] = str(path)
    with Path(os.environ['GITHUB_ENV']).open('a') as stream:
        for key, value in values.items():
            stream.write(f'{key}={value}\n')
            os.environ[key] = value
    _, private, _ = hosted()
    save(private / 'request.json', dict(metric=metric, initialized_monotonic=started,
        run_id=int(os.environ['GITHUB_RUN_ID']), run_attempt=int(os.environ['GITHUB_RUN_ATTEMPT']),
        controller_head=os.environ['GITHUB_SHA'], policy=identity(ROOT / ASSET),
        native_receipt=policy['native_receipt'], retention=retention_plan(),
        local_recovery_operator_attestation=True, local_storage_not_verified_by_hosted_job=True))
    check(len(cpus) == 4 and shutil.disk_usage(session).free >= 22 * 2**30, 'actual22GiB/four-CPU admission unavailable')
    maximum = Path('/sys/fs/cgroup/memory.max').read_text().strip()
    swap = Path('/sys/fs/cgroup/memory.swap.max').read_text().strip()
    quota = Path('/sys/fs/cgroup/cpu.max').read_text().split()
    check(maximum == str(12 * 2**30) and swap == '0' and quota[0] != 'max'
          and int(quota[0]) == 4 * int(quota[1]), 'actual12GiB/no-extra-swap/four-CPU cgroup differs')
    check(resource.getrlimit(resource.RLIMIT_AS) == (resource.RLIM_INFINITY, resource.RLIM_INFINITY),
          'inherited address-space limit is not the adopted cgroup-only profile')
    save(private / 'metadata/machine.json', dict(uname=platform.uname()._asdict(), cpus=cpus,
        memory_max=maximum, swap_max=swap, cpu_max=quota, rlimit_as=resource.getrlimit(resource.RLIMIT_AS),
        rlimit_as_not_set_by_controller=True, independent_metric_job=True, cache_cold_proven=False))
    for name in ('cpuinfo', 'meminfo', 'self/cgroup', 'self/mountinfo'):
        (private / 'metadata' / name.replace('/', '-')).write_bytes((Path('/proc') / name).read_bytes())
    bind_context(private)
    validate_native_receipt()
    check(_hash_file(ROOT / policy['recipient']['path']) == policy['recipient']['sha256'], 'public recipient differs')
    transport.prepare_tool('linux-amd64', private / 'runtime/age')


def execute():
    _, private, policy = hosted()
    request = load(private / 'request.json')
    metric, started = request['metric'], request['initialized_monotonic']
    check(metric in METRICS and metric == os.environ['ECOLOGY_BETA_METRIC'], 'actual job metric differs')
    def interrupt(signum, frame):
        raise TimeoutError('owned controller deadline/termination; HOLD')
    for signum in (signal.SIGALRM, signal.SIGTERM, signal.SIGINT):
        signal.signal(signum, interrupt)
    signal.setitimer(signal.ITIMER_REAL, max(.001, started + 1200 - time.monotonic()))
    try:
        whole.registry_image(private / 'metadata', policy)
        binary = admit_native(private, policy)
        capacity(private)
        python = prepare_runtime(private, policy, metric)
        capacity(private)
        whole.acquire_study(private, policy)
        source_command(private, python, 'prepare', metric, 'prepare-complete-data', 600)
        source_command(private, python, 'bridge', metric, 'original-beta-bridge', 120, private / 'bridge')
        check(time.monotonic() - started <= 1200, 'admission/compact bridge exceeded20min; HOLD')
        signal.setitimer(signal.ITIMER_REAL, max(.001, started + 18600 - time.monotonic()))
        admitted = load(private / 'data/admission.json')
        check((len(admitted['features']), len(admitted['samples']), admitted['tree_tips'], admitted['tree_nodes']) ==
              (41328, 1105, 162331, 323372), 'complete original dimensions differ')
        label_overhead(admitted['samples'])
        frozen = {name: identity(private / 'data' / name) for name in ('study.tsv', 'tree.nwk', 'admission.json')}
        save(private / 'timing-inputs.json', frozen)
        outcomes, source_reference, native_reference = [], None, None
        source_body, native_body = None, None
        measurement_started, scientific_seconds = time.monotonic(), 0.0
        for row in schedule():
            timeout = policy['policy']['source_timeout_seconds' if row['engine'] == 'source' else 'native_timeout_seconds']
            reserve_call(started, time.monotonic(), timeout, policy)
            capacity(private)
            check(frozen == {name: identity(private / 'data' / name) for name in frozen}, 'timed original input changed')
            check(source_reference is None or identity(source_reference) == source_body, 'first public reference changed; HOLD')
            check(native_reference is None or identity(native_reference) == native_body, 'first native body changed; HOLD')
            label = f'{row["phase"]}-{row["pair"]}-{row["engine"]}-{row["workers"]}'
            output = private / 'runs' / label / 'matrix.tsv'
            if row['engine'] == 'source':
                argv = [python, '-B', ROOT / 'scripts/ecology_beta_source.py', 'matrix', '--session', private,
                        '--metric', metric, '--output', output]
            else:
                argv = [binary, 'beta', '--counts', private / 'data/study.tsv', '--metric', metric,
                        '--output', output, '--threads', str(row['workers']), '--json']
                if 'unifrac' in metric:
                    argv += ['--tree', private / 'data/tree.nwk']
            call = capture(private / 'runs', label, argv, workers=row['workers'], timeout=timeout,
                           private=private, file_limit=64 * 2**20)
            scientific_seconds += call['wall']
            if row['engine'] == 'native':
                envelope = load(private / 'runs' / label / 'stdout.txt')
                check(envelope['status'] == 'ok' and envelope['result']['workers'] == row['workers']
                    and envelope['result']['beta']['metric'] == metric
                    and envelope['result']['features'] == 41328 and envelope['result']['samples'] == 1105
                    and envelope['result']['nonzeros'] == 314497,
                    'actual native metric/worker/input envelope differs')
            else:
                observation = load(Path(str(output) + '.json'))
                check(observation['metric'] == metric and observation['source_profile'] == 'contiguous-public'
                    and observation['status'] == 'returned' and observation['validate'] is True
                    and observation['feature_count'] == 41328 and observation['input_shape'] == [1105, 41328]
                    and observation['sample_ids'] == admitted['samples'] and observation['shape'] == [1105, 1105],
                    'actual public source request/result differs')
                if 'unifrac' in metric:
                    check(observation['layout']['shape'] == [1105, 323372]
                        and observation['layout']['dtype'] == 'int64' and observation['layout']['result_c'] is True,
                        'actual full prepared public layout differs')
            if source_reference is None:
                check(row['engine'] == 'source' and row['phase'] == 'qualification', 'first reference is not public Source1')
                source_reference = output
                source_body = identity(output)
                save(private / 'full-reference.json', dict(metric=metric, body=source_body,
                    path=str(output.relative_to(private)), observation=load(Path(str(output) + '.json')),
                    source_profile='contiguous-public', unexecuted_default_full_reference_claim=False))
            comparison = compare_matrix(source_reference, output, admitted['samples'],
                native_reference=native_reference if row['engine'] == 'native' else None)
            if row['engine'] == 'native' and native_reference is None:
                native_reference = output
                native_body = identity(output)
            raw = call['raw_wait4']
            result = dict(**row, path=str(output.relative_to(private)), identity=identity(output), comparison=comparison,
                wall=call['wall'], cpu=raw['user_seconds'] + raw['system_seconds'], rss_bytes=raw['rss_bytes'],
                raw_wait4=raw, source_profile='contiguous-public')
            save(private / 'runs' / label / 'comparison.json', result)
            outcomes.append(result)
            capacity(private)
            check(time.monotonic() - measurement_started - scientific_seconds <= 600,
                  'total untimed comparison/metadata allowance exceeded10min; HOLD')
        pairs = []
        for pair in range(1, 6):
            selected = {('source' if r['engine'] == 'source' else 'native' + str(r['workers'])): r
                        for r in outcomes if r['phase'] == 'measured' and r['pair'] == pair}
            check(set(selected) == {'source', 'native1', 'native4'}, 'incomplete measured round; HOLD')
            pairs.append(selected)
        decision = evaluate(pairs)
        save(private / 'performance.json', dict(metric=metric, decision=decision, complete_calls=21,
            source_profile=policy['source_profile'], all_originals_retained=True,
            host=load(private / 'metadata/machine.json'), native_receipt=policy['native_receipt'],
            original_dimensions=dict(features=41328, samples=1105, tree_tips=162331, tree_nodes=323372),
            interpretation='one independent metric/host row; never a summed eight-metric pipeline'))
        check(decision['passed'], 'fixed primary Native1 performance route did not pass; HOLD')
        save(private / 'execution-status.json', dict(status='complete-and-fixed-metric-gate-passed', metric=metric))
    except Exception:
        save(private / 'execution-status.json', dict(status='HOLD', metric=metric, traceback=traceback.format_exc(),
            source_failure_is_not_native_win=True, performance_claim=False, partial_originals_retained=True))
        raise
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)


def seal_manifest(private, policy, metric):
    private = Path(private)
    failures = []
    try:
        bindings = load(private / 'context-bindings.json')
        check(bindings == {name: identity(ROOT / name) for name in bindings}, 'controller/authority changed')
    except (ValueError, OSError) as error:
        failures.append(str(error))
    try:
        capacity(private, policy)
    except (ValueError, OSError) as error:
        failures.append(str(error))
    files = whole.file_manifest(private)
    execution = load(private / 'execution-status.json') if (private / 'execution-status.json').exists() else None
    if failures:
        execution = dict(status='HOLD', original_execution_status=execution, finalization_failures=failures)
    save(private / 'artifact-manifest.json', dict(schema='rsomics.ecology.beta-real-originals.v1',
        metric=metric, files=files, policy=identity(ROOT / ASSET), native_receipt=policy['native_receipt'],
        source_profile=policy['source_profile'], source_failures_are_not_performance_wins=True,
        execution_status=execution, finalization_failures=failures,
        complete_product=False, cross_host_pipeline_total_allowed=False, aggregate_rss_exact=False))
    size = 0
    for name, row in {**files, 'artifact-manifest.json': dict(kind='file', mode=0o600,
                    bytes=(private / 'artifact-manifest.json').stat().st_size)}.items():
        member = tarfile.TarInfo(name)
        member.mode, member.mtime = row['mode'], 0
        if row['kind'] == 'directory':
            member.type = tarfile.DIRTYPE
        elif row['kind'] == 'symlink':
            member.type, member.linkname = tarfile.SYMTYPE, row['target']
        else:
            member.size = row['bytes']
        size += len(member.tobuf(format=tarfile.PAX_FORMAT)) + (member.size + 511) // 512 * 512
    size = (size + 1024 + tarfile.RECORDSIZE - 1) // tarfile.RECORDSIZE * tarfile.RECORDSIZE
    check(size <= policy['policy']['tar_bound_bytes'], 'final deterministic TAR exceeds frozen7GiB; HOLD')
    return failures


def finalize():
    session, private, policy = hosted()
    metric = load(private / 'request.json')['metric']
    failures = seal_manifest(private, policy, metric)
    receipt = transport.encrypt(private / 'runtime/age', ROOT / policy['recipient']['path'],
        session / 'cipher/payload.tar.age', producer=[sys.executable, '-B', __file__, 'tar', '--private', private],
        plaintext_limit=7 * 2**30, context=dict(repository='omics-rust/rsomics-world', product='rsomics-ecology',
            run_id=int(os.environ['GITHUB_RUN_ID']), run_attempt=int(os.environ['GITHUB_RUN_ATTEMPT']),
            head_sha=os.environ['GITHUB_SHA'], workflow_path=WORKFLOW, phase='beta-' + metric.replace('_', '-')))
    save(session / 'cipher/receipt.json', receipt)
    check(not failures, 'original evidence encrypted but finalization integrity/retention remains HOLD')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['init', 'execute', 'finalize', 'tar'])
    parser.add_argument('--metric', choices=METRICS)
    parser.add_argument('--private', type=Path)
    args = parser.parse_args()
    if args.action == 'init':
        try:
            initialize(args.metric)
        finally:
            signal.setitimer(signal.ITIMER_REAL, 0)
    elif args.action == 'execute':
        execute()
    elif args.action == 'finalize':
        finalize()
    else:
        hosted()
        whole.stream_tar(args.private)


if __name__ == '__main__':
    main()
