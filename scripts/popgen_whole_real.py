"""Private same-machine whole-chr22 qualification from an accepted native ELF."""

import argparse
import base64
from collections import Counter
from contextlib import contextmanager
import copy
import gzip
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path, PurePosixPath
import platform
import re
import resource
import shutil
import signal
import stat
import statistics
import subprocess
import sys
import tarfile
import time
import traceback
import urllib.error
import urllib.parse
import urllib.request
import warnings
import zipfile

import popgen_first_workflow_inputs as inputs
import popgen_first_workflow_oracle as oracle
import popgen_hosted_source as hosted
import private_artifact as transport


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = '.github/workflows/popgen-whole-real.yml'
POLICY = 'assets/popgen-whole-real-20261101.json'
TARGETS = {'linux-x86_64', 'linux-aarch64', 'macos-x86_64', 'macos-aarch64'}
SOURCE_ASSET = 'assets/popgen-first-input-20261101.json'
NATIVE_ASSET = 'assets/popgen-native-20261101.json'
COMPARATOR_SHA = 'd14f08cee17b395ff6460692c166d4740f352ea9ee6c33115449ce778d0177d9'


def check(condition, message):
    if not condition:
        raise ValueError(message)


def save(path, value):
    hosted.save(Path(path), value)
    Path(path).chmod(0o600)


def load(path):
    path = Path(path)
    check(path.is_file() and not path.is_symlink(), 'regular JSON input required')
    return json.loads(path.read_bytes())


def identity(path):
    path = Path(path)
    check(path.is_file() and not path.is_symlink(), 'regular body required')
    return dict(bytes=path.stat().st_size, sha256=inputs.sha256(path))


def load_policy():
    value = load(ROOT / POLICY)
    check(value['schema'] == 'rsomics.popgen.whole-real.v1', 'whole-real policy schema differs')
    check(identity(ROOT / value['comparator']['path'])['sha256'] == COMPARATOR_SHA == value['comparator']['sha256'],
          'frozen complete comparison body differs')
    check(value['comparator']['root_authenticated_original_source_required'] is True,
          'original-source root adoption requirement absent')
    check(retention_partition(value) <= value['retention']['retained_bytes'], 'complete retention partition exceeds cap')
    return value


def schedule(policy):
    check(policy['schedule'] == dict(allowances=[1, 4], warmup_pairs_each=1, measured_pairs_each=5),
          'fixed whole-workflow schedule differs')
    result = []
    for workers in (1, 4):
        first = ('native', 'source') if workers == 1 else ('source', 'native')
        for engine in first:
            result.append(dict(workers=workers, warmup=True, pair=0, engine=engine,
                               name=f'b{workers}-warmup-{engine}'))
        for pair in range(1, 6):
            for engine in (first if pair % 2 else first[::-1]):
                result.append(dict(workers=workers, warmup=False, pair=pair, engine=engine,
                                   name=f'b{workers}-p{pair}-{engine}'))
    return result


def retention_partition(policy):
    limit = policy['retention']
    return limit['base_bytes'] + 12 * (limit['source_run_bytes'] + limit['native_run_bytes']) + limit['overhead_bytes']


def read_native_receipt(path):
    path = Path(path)
    check(path.is_file() and not path.is_symlink(), 'actual tracked native receipt is required')
    resolved = path.resolve(strict=True)
    check(resolved.is_relative_to(ROOT / '.autopilot/oracles'), 'native receipt must be a tracked oracle path')
    value = load(path)
    binding = load_policy()['native_receipt']
    check(resolved == ROOT / binding['path'] and identity(path)['sha256'] == binding['sha256'],
          'native receipt is not the pinned actual adopted body')
    check(value.get('schema') == 'rsomics.popgen.native-adoption.v1' and value.get('product') == 'rsomics-popgen'
          and value.get('repository') == 'omics-rust/rsomics-world', 'native receipt product/schema differs')
    check(value.get('status') == 'accepted' and value.get('root_fresh_review_required') is False
          and value.get('native_four_qualified') is True and set(value.get('native_targets', [])) == TARGETS,
          'native receipt has no accepted four-native qualification')
    check(set(value['targets']) == TARGETS and value['source_files'] == 48 and value['recorded_registry_archive_checksums'] == 69,
          'native receipt complete target/source/lock population differs')
    linux = value['targets']['linux-x86_64']
    normalized = dict(value, artifact_id=linux['artifact_id'], size_in_bytes=linux['artifact']['bytes'],
        artifact_sha256=linux['artifact']['sha256'], native_tar_sha256=linux['tar']['sha256'],
        binary_sha256=linux['binary']['sha256'])
    for key in ('run_id', 'run_attempt', 'artifact_id', 'size_in_bytes'):
        check(type(normalized.get(key)) is int and normalized[key] > 0, 'native receipt requires actual ' + key)
    for key, count in (('head_sha', 40), ('artifact_sha256', 64), ('native_tar_sha256', 64), ('binary_sha256', 64)):
        check(re.fullmatch('[0-9a-f]{' + str(count) + '}', normalized.get(key, '')) is not None
              and set(normalized[key]) != {'0'}, 'native receipt requires actual ' + key)
    expected = load_policy()['owner']
    for key, wanted in (('owner_commit', expected['commit']), ('owner_tree', expected['tree']),
                        ('source_archive_sha256', expected['source_archive_sha256']),
                        ('source_manifest_sha256', expected['source_manifest_sha256']), ('lock_sha256', expected['lock_sha256'])):
        check(value.get(key) == wanted, 'accepted native receipt source binding differs: ' + key)
    return normalized


def validate_workflow(path):
    text = Path(path).read_text()
    events = re.search(r'^on:\n((?:  .*\n)+)', text, re.MULTILINE)
    check(events is not None and events.group(1).startswith('  workflow_dispatch:\n')
          and '\n  push:' not in text and '\n  schedule:' not in text, 'manual-only whole-real workflow required')
    check('continue-on-error:' not in text and 'secrets.' not in text, 'masked failures/private keys are forbidden')
    images = [line.strip() for line in text.splitlines() if line.strip().startswith('image:')]
    check(len(images) == 1 and oracle.IMAGE in images[0] and 'runs-on: ubuntu-24.04' in text,
          'whole-real image/provider class differs')
    upload = text.split('uses: actions/upload-artifact@v4', 1)
    check(len(upload) == 2, 'one ciphertext upload required')
    block = re.search(r'          path: \|\n((?:            .*\n)+)', upload[1])
    expected = ['${{ env.POPGEN_SESSION }}/cipher/payload.tar.age', '${{ env.POPGEN_SESSION }}/cipher/receipt.json']
    check(block is not None and [line.strip() for line in block.group(1).splitlines()] == expected,
          'only ciphertext and allowlisted receipt may be uploaded')
    check('native_receipt:' in text and 'required: true' in text, 'actual native receipt workflow input absent')


def paths():
    return hosted.hosted_session()


def cgroup():
    result = hosted.cgroup_observation()
    path = Path('/sys/fs/cgroup/io.stat')
    result['io.stat'] = path.read_text() if path.is_file() else None
    path = Path('/sys/fs/cgroup/cpu.stat')
    result['cpu.stat'] = path.read_text() if path.is_file() else None
    return result


def machine_admission(private, policy):
    group = cgroup()
    quota, period = group.get('cpu.max', '').split()
    check(quota != 'max' and int(quota) / int(period) == policy['limits']['container_cpus'],
          'actual four-CPU quota absent')
    affinity = sorted(os.sched_getaffinity(0))
    check(len(affinity) >= 4 and int(group['memory.max']) == policy['limits']['container_memory_bytes'],
          'actual four-CPU/12GiB provider admission failed')
    check(resource.getrlimit(resource.RLIMIT_AS) == (resource.RLIM_INFINITY, resource.RLIM_INFINITY),
          'an inherited virtual-address restriction prevents a fair benchmark')
    save(private / 'machine-admission.json', dict(affinity=affinity, cgroup=group,
        uname=list(platform.uname()), cpuinfo=Path('/proc/cpuinfo').read_text(),
        meminfo=Path('/proc/meminfo').read_text(), mounts=Path('/proc/self/mountinfo').read_text(),
        load=list(os.getloadavg()), address_space_limit='unrestricted; no RLIMIT_AS set',
        rss_guard=dict(bytes=policy['limits']['sampled_tree_rss_bytes'], sampled=True, hard_peak=False)))
    return affinity[:4]


def live_tree(pid):
    processes = {}
    for directory in Path('/proc').iterdir():
        if not directory.name.isdigit():
            continue
        try:
            fields = (directory / 'stat').read_text().rsplit(')', 1)[1].split()
            processes[int(directory.name)] = (int(fields[1]), int(fields[21]) * os.sysconf('SC_PAGE_SIZE'))
        except (FileNotFoundError, PermissionError, ProcessLookupError):
            continue
    selected = {pid}
    while True:
        more = {child for child, (parent, _) in processes.items() if parent in selected}
        if more <= selected:
            break
        selected |= more
    counters = Counter()
    for child in selected:
        try:
            for line in (Path('/proc') / str(child) / 'io').read_text().splitlines():
                key, value = line.split(':', 1)
                counters[key] += int(value)
        except (FileNotFoundError, PermissionError, ProcessLookupError):
            continue
    return dict(sampled_live_tree_rss_bytes=sum(processes.get(child, (0, 0))[1] for child in selected),
                live_processes=len(selected & processes.keys()), live_io_counters=dict(counters))


def child_setup(affinity):
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    if affinity is not None:
        os.sched_setaffinity(0, affinity)


def rusage(value):
    names = ('ru_utime', 'ru_stime', 'ru_maxrss', 'ru_ixrss', 'ru_idrss', 'ru_isrss',
             'ru_minflt', 'ru_majflt', 'ru_nswap', 'ru_inblock', 'ru_oublock',
             'ru_msgsnd', 'ru_msgrcv', 'ru_nsignals', 'ru_nvcsw', 'ru_nivcsw')
    return {name: getattr(value, name) for name in names}


def stop_child(process):
    if process is None or process.returncode is not None:
        return None
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        child, status, usage = os.wait4(process.pid, os.WNOHANG)
        if child:
            process.returncode = os.waitstatus_to_exitcode(status)
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            return usage
        time.sleep(0.05)
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    _, status, usage = os.wait4(process.pid, 0)
    process.returncode = os.waitstatus_to_exitcode(status)
    return usage


def capture(root, name, argv, *, cwd=None, timeout=1800, allowed=(0,), affinity=None, environment=None):
    root = Path(root)
    call = root / 'calls' / name
    call.mkdir(parents=True, exist_ok=False, mode=0o700)
    policy = load_policy()
    env = dict(os.environ if environment is None else environment)
    save(call / 'request.json', dict(argv=list(map(str, argv)), cwd=str(cwd or root),
        timeout_seconds=timeout, affinity=affinity, virtual_address_cap=None,
        sampled_tree_rss_guard=policy['limits']['sampled_tree_rss_bytes'],
        environment={key: env.get(key) for key in (*hosted.CACHE_NAMES, 'OMP_NUM_THREADS',
            'OPENBLAS_NUM_THREADS', 'MKL_NUM_THREADS', 'BLIS_NUM_THREADS', 'NUMEXPR_NUM_THREADS')}))
    process = None
    usage = None
    peak = 0
    before = cgroup()
    started = time.monotonic()
    failure = None
    try:
        with (call / 'stdout.txt').open('xb') as stdout, (call / 'stderr.txt').open('xb') as stderr, (call / 'observations.jsonl').open('x') as samples:
            started = time.monotonic()
            process = subprocess.Popen(list(map(str, argv)), cwd=cwd or root, env=env, stdout=stdout, stderr=stderr,
                start_new_session=True, preexec_fn=lambda: child_setup(affinity))
            while True:
                child, status, observed = os.wait4(process.pid, os.WNOHANG)
                if child:
                    process.returncode = os.waitstatus_to_exitcode(status)
                    usage = observed
                    break
                elapsed = time.monotonic() - started
                sample = live_tree(process.pid)
                peak = max(peak, sample['sampled_live_tree_rss_bytes'])
                samples.write(json.dumps(dict(seconds=elapsed, **sample)) + '\n')
                samples.flush()
                if peak > policy['limits']['sampled_tree_rss_bytes']:
                    raise RuntimeError('sampled live process-tree RSS guard exceeded; qualification held')
                if elapsed > timeout:
                    raise RuntimeError('whole-workflow stage timeout; qualification held')
                time.sleep(policy['limits']['sample_interval_seconds'])
        if process.returncode not in allowed:
            raise RuntimeError('private whole-workflow command failed: ' + name)
    except BaseException:
        failure = traceback.format_exc()
        (call / 'failure.txt').write_text(failure)
        raise
    finally:
        if process is not None and process.returncode is None:
            usage = stop_child(process)
        save(call / 'status.json', dict(returncode=process.returncode if process else None,
            elapsed_seconds=time.monotonic() - started, kernel_rusage=rusage(usage) if usage else None,
            completion_observation_interval_seconds=policy['limits']['sample_interval_seconds'],
            sampled_descendant_rss_peak_bytes=peak, sampled_rss_is_hard_peak=False,
            io_scope='raw live process counters may lose exited children; cgroup deltas are job-scoped, not exclusive device bytes',
            cgroup_before=before, cgroup_after=cgroup(), failure=failure is not None))
    return call


@contextmanager
def source_runtime_capture():
    original = hosted.capture
    hosted.capture = capture
    try:
        yield
    finally:
        hosted.capture = original


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, response, code, message, headers, location):
        return None


def api(route, target, *, binary=False, maximum=64 * 2**20):
    target = Path(target)
    check(not target.exists(), 'fresh original API target required')
    target.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    request = urllib.request.Request('https://api.github.com/repos/omics-rust/rsomics-world/' + route,
        headers={'Authorization': 'Bearer ' + os.environ['GH_TOKEN'], 'Accept': 'application/vnd.github+json',
                 'X-GitHub-Api-Version': '2022-11-28'})
    try:
        response = urllib.request.build_opener(NoRedirect).open(request, timeout=120)
    except urllib.error.HTTPError as error:
        if not binary or error.code != 302:
            save(str(target) + '.error.json', dict(status=error.code, route=route))
            raise RuntimeError('native original API request failed') from None
        location = urllib.parse.urlsplit(error.headers['Location'])
        check(location.scheme == 'https' and location.hostname and any(location.hostname.endswith(suffix)
            for suffix in ('.blob.core.windows.net', '.actions.githubusercontent.com', '.githubusercontent.com')),
            'untrusted native artifact redirect')
        response = urllib.request.urlopen(error.headers['Location'], timeout=120)
    count = 0
    with response, target.open('xb') as stream:
        for block in iter(lambda: response.read(2**20), b''):
            count += len(block)
            check(count <= maximum, 'native original API response exceeds admitted bound')
            stream.write(block)
    target.chmod(0o600)
    return None if binary else load(target)


def portable(name):
    check(not PurePosixPath(name).is_absolute() and all(part not in ('', '.', '..') for part in name.split('/')),
          'unsafe native original member')


def admit_native(private, receipt_path):
    policy = load_policy()
    accepted = read_native_receipt(receipt_path)
    root = private / 'native-original'
    root.mkdir(mode=0o700)
    relative = Path(receipt_path).resolve(strict=True).relative_to(ROOT).as_posix()
    entry = api('contents/' + relative + '?ref=' + os.environ['GITHUB_SHA'], root / 'tracked-receipt-api.json')
    check(entry.get('encoding') == 'base64' and entry['path'] == relative, 'native receipt is not tracked at this head')
    body = base64.b64decode(''.join(entry['content'].splitlines()), validate=True)
    check(body == Path(receipt_path).read_bytes(), 'tracked actual-head native receipt body differs')
    (root / 'accepted-receipt.json').write_bytes(body)
    run_id, attempt = accepted['run_id'], accepted['run_attempt']
    run = api(f'actions/runs/{run_id}/attempts/{attempt}', root / 'run.json')
    check((run['id'], run['run_attempt'], run['head_sha'], run['status'], run['conclusion'], run['path']) ==
          (run_id, attempt, accepted['head_sha'], 'completed', 'success', '.github/workflows/popgen-candidate.yml'),
          'actual native-four run does not match accepted receipt')
    check(run['repository']['full_name'] == 'omics-rust/rsomics-world', 'native repository differs')
    jobs = api(f'actions/runs/{run_id}/attempts/{attempt}/jobs?per_page=100', root / 'jobs.json')
    check(jobs['total_count'] == len(jobs['jobs']) == 4 and
          {row['name'] for row in jobs['jobs']} == TARGETS and all(row['conclusion'] == 'success' for row in jobs['jobs']),
          'actual complete four native jobs absent')
    check(all(row['id'] == accepted['targets'][row['name']]['job_id'] for row in jobs['jobs']),
          'actual four-native job identities differ')
    listing = api(f'actions/runs/{run_id}/artifacts?per_page=100', root / 'artifacts.json')
    check(listing['total_count'] == len(listing['artifacts']) == 4, 'complete native artifact listing differs')
    for target, recorded in accepted['targets'].items():
        rows = [entry for entry in listing['artifacts'] if entry['id'] == recorded['artifact_id']]
        check(len(rows) == 1 and rows[0]['name'] == recorded['artifact_name'] and not rows[0]['expired']
              and rows[0]['size_in_bytes'] == recorded['artifact']['bytes']
              and rows[0]['digest'] == 'sha256:' + recorded['artifact']['sha256']
              and rows[0]['workflow_run']['head_sha'] == accepted['head_sha'], 'actual four-native artifact identities differ')
    name = f'popgen-native-{run_id}-{attempt}-linux-x86_64'
    found = [row for row in listing['artifacts'] if row['id'] == accepted['artifact_id'] and row['name'] == name]
    check(len(found) == 1 and not found[0]['expired'], 'actual accepted Linux native artifact absent')
    row = api('actions/artifacts/' + str(accepted['artifact_id']), root / 'artifact-api.json')
    check(all(row[key] == found[0][key] for key in ('id', 'name', 'size_in_bytes', 'digest', 'expired'))
          and row['workflow_run']['head_sha'] == accepted['head_sha'], 'native artifact API identities differ')
    check(row['size_in_bytes'] == accepted['size_in_bytes'] and row['digest'] == 'sha256:' + accepted['artifact_sha256'],
          'native artifact receipt digest differs')
    api(f"actions/artifacts/{row['id']}/zip", root / 'original.zip', binary=True, maximum=row['size_in_bytes'])
    check(identity(root / 'original.zip') == dict(bytes=row['size_in_bytes'], sha256=accepted['artifact_sha256']),
          'native original ZIP differs')
    with zipfile.ZipFile(root / 'original.zip') as archive:
        check(len(archive.infolist()) == 1 and archive.testzip() is None, 'native ZIP membership/CRC differs')
        member = archive.infolist()[0]
        portable(member.filename)
        check(member.filename == 'popgen-evidence.tar' and member.file_size <= 512 * 2**20,
              'native original TAR carrier differs')
        with archive.open(member) as source, (root / 'original.tar').open('xb') as target:
            shutil.copyfileobj(source, target)
    check(identity(root / 'original.tar') == accepted['targets']['linux-x86_64']['tar'], 'native original TAR differs')
    prefix = 'popgen-evidence/'
    selected = {'candidate/selection.json', 'candidate/native-binary/identity.json',
                'candidate/native-binary/rsomics-popgen-linux-x86_64', 'candidate/integrity-after.json',
                'candidate/build-release/request.json', 'candidate/build-release/stdout.txt',
                'candidate/build-release/stderr.txt', 'candidate/build-release/status.json',
                'candidate/files.sha256', 'candidate/source.tar.gz', 'controller/checksums.json'}
    controllers = ('scripts/popgen_candidate.py', 'scripts/test_popgen_candidate_workflow.py',
        'scripts/infercnv_source_snapshot.py', 'scripts/signal_candidate.py', '.github/workflows/popgen-candidate.yml',
        NATIVE_ASSET, '.autopilot/oracles/popgen-source-2026-11-01.json')
    selected.update('controller/' + name for name in controllers)
    authority = load(ROOT / NATIVE_ASSET)
    owner_seen = set()
    with tarfile.open(root / 'original.tar', 'r:') as archive:
        members = archive.getmembers()
        names = [member.name for member in members]
        check(len(names) == len(set(names)) == len({name.casefold() for name in names}), 'native member collisions')
        for member in members:
            portable(member.name)
            check(member.isfile() and member.type == tarfile.REGTYPE and member.name.startswith(prefix),
                  'nonregular or unexpected native member')
        manifest = json.load(archive.extractfile(prefix + 'artifact-manifest.json'))
        check(manifest['schema'] == 'rsomics.popgen.native-four.v1' and
              set(names) == {prefix + name for name in manifest['files']} | {prefix + 'artifact-manifest.json'},
              'native exact manifest membership differs')
        check(len(manifest['files']) == accepted['targets']['linux-x86_64']['manifest_entries'],
              'native original manifest population differs')
        for member in members:
            relative = member.name.removeprefix(prefix)
            digest = hashlib.sha256()
            git_digest = hashlib.sha1(b'blob ' + str(member.size).encode() + b'\0') if relative.startswith('candidate/source/') else None
            target = None
            if relative in selected:
                path = root / 'regular' / relative
                path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
                target = path.open('xb')
            try:
                with archive.extractfile(member) as source:
                    for block in iter(lambda: source.read(2**20), b''):
                        digest.update(block)
                        if git_digest is not None:
                            git_digest.update(block)
                        if target is not None:
                            target.write(block)
            finally:
                if target is not None:
                    target.close()
            actual = dict(bytes=member.size, sha256=digest.hexdigest(), mode=member.mode)
            if relative != 'artifact-manifest.json':
                check(manifest['files'][relative] == actual, 'native original manifest body differs')
            if relative.startswith('candidate/source/'):
                owner_name = relative.removeprefix('candidate/source/')
                expected_owner = authority['files'].get(owner_name)
                check(expected_owner is not None and {key: expected_owner[key] for key in ('bytes', 'sha256', 'mode')} == actual,
                      'native original owner body differs')
                check(git_digest.hexdigest() == expected_owner['git_blob'], 'native original Git blob differs')
                owner_seen.add(owner_name)
        save(root / 'artifact-manifest.json', manifest)
    check(owner_seen == set(authority['files']) and len(owner_seen) == 48, 'complete actual owner source closure absent')
    binding = load(root / 'regular/controller/checksums.json')
    check(set(binding) == set(controllers) and len(binding) == 7, 'native exact-head controller population differs')
    for index, name in enumerate(controllers):
        captured = root / 'regular/controller' / name
        observed = identity(captured)
        check({key: binding[name][key] for key in ('bytes', 'sha256')} == observed,
              'retained native controller body differs')
        response = api('contents/' + name + '?ref=' + accepted['head_sha'], root / 'head-controller-api' / (str(index) + '.json'))
        check(response.get('encoding') == 'base64', 'exact-head native controller body unavailable')
        body = base64.b64decode(''.join(response['content'].splitlines()), validate=True)
        check(body == captured.read_bytes() and hashlib.sha1(b'blob ' + str(len(body)).encode() + b'\0' + body).hexdigest() == response['sha'],
              'actual native-head controller Git body differs')
    base = root / 'regular/candidate'
    selection = load(base / 'selection.json')
    expected = policy['owner']
    source_receipt_path = root / 'regular/controller/.autopilot/oracles/popgen-source-2026-11-01.json'
    source_receipt = load(source_receipt_path)
    source_binding = authority['source_run']
    check(set(source_binding) == {'artifact_id', 'artifact_sha256', 'authenticated_tar_sha256', 'head_sha', 'run_attempt', 'run_id'}
          and source_binding['run_id'] == policy['source_run']
          and source_binding == {key: source_receipt[key] for key in source_binding}
          and identity(source_receipt_path)['sha256'] == authority['source_receipt_sha256'] == policy['source_receipt_sha256'],
          'native source authority/adopted source receipt binding differs')
    for key, wanted in (('owner_commit', expected['commit']), ('owner_tree', expected['tree']),
                        ('snapshot', expected['snapshot']), ('source_archive_sha256', expected['source_archive_sha256']),
                        ('source_manifest_sha256', expected['source_manifest_sha256']), ('lock_sha256', expected['lock_sha256']),
                        ('controller_commit', accepted['head_sha']), ('target', 'linux-x86_64'), ('source_run', source_binding)):
        check(selection[key] == wanted, 'actual native selection differs: ' + key)
    check(load(base / 'integrity-after.json')['failures'] == [], 'actual native integrity failed')
    check(selection['authority_sha256'] == identity(root / 'regular/controller' / NATIVE_ASSET)['sha256'],
          'native actual authority asset binding differs')
    binary = base / 'native-binary/rsomics-popgen-linux-x86_64'
    declared = load(base / 'native-binary/identity.json')
    check(declared['selection_sha256'] == identity(base / 'selection.json')['sha256']
          and declared['target'] == 'linux-x86_64' and declared['format'] == 'ELF64'
          and declared['compiler_host'] == 'x86_64-unknown-linux-gnu'
          and {key: declared[key] for key in ('bytes', 'sha256')} == identity(binary)
          and declared['sha256'] == accepted['binary_sha256'], 'actual retained ELF identity differs')
    check(declared == accepted['targets']['linux-x86_64']['binary']
          and manifest['files']['candidate/native-binary/rsomics-popgen-linux-x86_64']['mode'] == declared['mode'],
          'accepted actual binary metadata/mode differs')
    check(binary.read_bytes()[:6] == b'\x7fELF\x02\x01' and int.from_bytes(binary.read_bytes()[18:20], 'little') == 62,
          'retained native binary is not actual Linux x86_64 ELF')
    check(load(base / 'build-release/status.json')['status'] == 0, 'actual release build failed')
    messages = [json.loads(line) for line in (base / 'build-release/stdout.txt').read_text().splitlines()]
    actual_binaries = [message for message in messages if message.get('reason') == 'compiler-artifact'
        and message.get('executable') and message['target']['name'] == 'rsomics-popgen'
        and 'bin' in message['target']['kind'] and not message['profile']['test']]
    check(len(actual_binaries) == 1 and actual_binaries[0]['executable'].endswith('/release/rsomics-popgen'),
          'actual Cargo executable provenance differs')
    check(identity(base / 'source.tar.gz')['sha256'] == expected['source_archive_sha256']
          and identity(base / 'files.sha256')['sha256'] == expected['source_manifest_sha256'], 'native source carrier differs')
    binary.chmod(0o700)
    capture(private, 'native-readelf', ['readelf', '--file-header', '--program-headers', '--dynamic', '--version-info', binary])
    call = capture(private, 'native-providers', ['ldd', binary])
    text = (call / 'stdout.txt').read_text() + (call / 'stderr.txt').read_text()
    check('not found' not in text, 'accepted native ELF cannot resolve on pinned Bookworm providers')
    providers = {}
    for token in sorted(set(re.findall(r'(?:=>\s*)?(/[^\s]+)', text))):
        path = Path(token)
        if path.is_file():
            actual = identity(path.resolve())
            retained = root / 'providers' / (actual['sha256'] + '-' + path.name)
            retained.parent.mkdir(exist_ok=True, mode=0o700)
            if not retained.exists():
                shutil.copy2(path.resolve(), retained)
            check(identity(retained) == actual, 'native provider retained body differs')
            providers[token] = dict(realpath=str(path.resolve()), retained=str(retained.relative_to(private)), **actual)
    bind_now = dict(os.environ, LD_BIND_NOW='1')
    capture(private, 'native-version', [binary, '--version'], environment=bind_now)
    capture(private, 'native-help', [binary, '--help'], environment=bind_now)
    result = dict(accepted_receipt=str(relative), accepted_receipt_identity=identity(receipt_path),
        run_id=run_id, run_attempt=attempt, head_sha=accepted['head_sha'], artifact_id=row['id'],
        original_zip=identity(root / 'original.zip'), original_tar=identity(root / 'original.tar'),
        selection=selection, binary=str(binary), binary_identity=identity(binary), providers=providers,
        all_native_manifest_bodies_verified=len(manifest['files']), native_four_qualified=True,
        performance_qualified=False, root_original_source_adoption_required=True)
    save(private / 'native-admission.json', result)
    return result

def write_legacy(root: Path, spec: dict, admission: dict, ordinals: set[int],
                 destination: Path) -> dict:
    paths = inputs.input_paths(root, spec)
    meta, samples = inputs.vcf_header(paths['vcf'])
    selected = admission['sample_indices']
    if [samples[index] for index in selected] != admission['sample_ids']:
        raise ValueError('legacy sample order/membership differs from admission')
    os.chmod(destination, 0o700)
    identities = {}
    count = total = 0
    path = destination / 'eligible.vcf'
    digest = hashlib.sha256()
    size = 0
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, 'wb') as stream:
        def emit(body: bytes) -> None:
            nonlocal size
            stream.write(body)
            digest.update(body)
            size += len(body)
        emit(('\n'.join(meta) + '\n').encode('utf-8'))
        emit(('\t'.join(['#CHROM', 'POS', 'ID', 'REF', 'ALT', 'QUAL', 'FILTER', 'INFO', 'FORMAT',
                         *admission['sample_ids']]) + '\n').encode('utf-8'))
        with inputs.text_stream(paths['vcf']) as original:
            for line in original:
                if line.startswith('#'):
                    continue
                if total in ordinals:
                    fields = line.rstrip('\r\n').split('\t')
                    if len(fields) != len(samples) + 9:
                        raise ValueError('legacy original record cardinality differs')
                    emit(('\t'.join(fields[:9] + [fields[9 + index] for index in selected]) + '\n').encode('utf-8'))
                    count += 1
                total += 1
    if total != admission['gt_shape'][0] or count != len(ordinals):
        raise ValueError('legacy original ordinal membership differs')
    identities[path.name] = dict(size=size, sha256=digest.hexdigest())
    for pop, indices in admission['population_indices'].items():
        path = destination / (pop + '.txt')
        body = ''.join(admission['sample_ids'][index] + '\n' for index in indices).encode('utf-8')
        descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(descriptor, 'wb') as stream:
            stream.write(body)
        identities[path.name] = dict(size=len(body), sha256=hashlib.sha256(body).hexdigest())
    return dict(root=str(destination.resolve()), records=count,
                input_gt_stream_sha256=admission['selected_gt_stream_sha256'],
                original_record_profile='PASS P-accessible literal biallelic SNV; both populations complete; original fields and GT preserved',
                files=identities)


def _source_fsync_directory(path: Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def source_statistics(np, allel, chrom: str, selected: dict, mask: bytearray,
                      admission: dict, spec: dict, output: Path) -> dict:
    """Export the source oracle contract using shared pinned FST components."""
    output.mkdir(mode=0o700)
    pos = selected['variants/POS']
    g = allel.GenotypeArray(selected['calldata/GT'], copy=False)
    indices = admission['population_indices']
    populations = spec['population_order']
    first, second = populations
    width = admission['max_alt_number']
    ac = {pop: g.count_alleles(subpop=indices[pop], max_allele=width) for pop in populations}
    an = {pop: ac[pop].sum(axis=1) for pop in populations}
    accessible = np.frombuffer(mask, dtype='u1') == ord('P')
    region = dict(start=1, stop=len(mask), is_accessible=accessible)
    windows = allel.position_windows(pos, size=spec['window_bp'], start=1, stop=len(mask), step=spec['window_bp'])
    mpd = {pop: allel.mean_pairwise_difference(ac[pop], fill=np.nan) for pop in populations}
    dxy = allel.mean_pairwise_difference_between(ac[first], ac[second], fill=np.nan)
    fixed = allel.locate_fixed_differences(ac[first], ac[second])
    num, den = allel.hudson_fst(ac[first], ac[second], fill=np.nan)
    a, b, c = allel.weir_cockerham_fst(g, [indices[pop] for pop in populations], max_allele=width)
    ancestor = [oracle.resolve_ancestral(str(raw), [str(ref), *[str(alt) for alt in alts if alt != '']])
                for raw, ref, alts in zip(selected['variants/AA'], selected['variants/REF'], selected['variants/ALT'])]
    ancestor_indices = np.array([value[0] for value in ancestor], dtype=int)
    header = ['ordinal0', 'chrom', 'pos1', 'ref', 'alts', 'raw_AA', 'ancestral_allele_index', 'ancestral_status']
    header += [pop + '_' + term for pop in populations for term in ['AN', 'AC', 'mpd', 'no_pair']]
    header += ['dxy_mpd', 'fixed_difference', 'hudson_num', 'hudson_den', 'wc_a', 'wc_b', 'wc_c']
    def sites():
        for i in range(len(pos)):
            row = [int(selected['ordinal'][i]), chrom, int(pos[i]), str(selected['variants/REF'][i]),
                   [str(alt) for alt in selected['variants/ALT'][i] if alt != ''],
                   str(selected['variants/AA'][i]), *ancestor[i]]
            for pop in populations:
                row.extend([an[pop][i], ac[pop][i].tolist(), mpd[pop][i], bool(an[pop][i] < 2)])
            yield row + [dxy[i], bool(fixed[i]), num[i], den[i], a[i].tolist(), b[i].tolist(), c[i].tolist()]
    oracle.table(output / 'sites.tsv.gz', header, sites())
    region_results = dict(chrom=chrom, start=1, stop=len(mask), accessible_bp=int(accessible.sum()),
                          records=len(pos), populations={}, between={},
                          ancestral_status_counts=dict(Counter(value[1] for value in ancestor)),
                          denominator='project accessibility, not individual callability')
    columns = {'start1': windows[:, 0], 'stop1': windows[:, 1]}
    def add_windows(prefix: str, value: tuple, per_base: bool) -> None:
        values, actual, *terms = value
        if not np.array_equal(actual, windows):
            raise ValueError('upstream window identity differs: ' + prefix)
        columns[prefix] = values
        if per_base:
            columns[prefix + '_accessible_bp'] = terms[0]
        columns[prefix + '_records'] = terms[-1]
    for pop in populations:
        region_results['populations'][pop] = dict(
            pi=allel.sequence_diversity(pos, ac[pop], **region),
            theta=allel.watterson_theta(pos, ac[pop], **region),
            tajima_d=allel.tajima_d(ac[pop], pos=pos, start=1, stop=len(mask), min_sites=3),
            segregating_sites=int(ac[pop].count_segregating()), n_max=int(an[pop].max()),
            called_AN_min=int(an[pop].min()), no_pair_sites=int(np.count_nonzero(an[pop] < 2)),
            mpd_sum_fill0=np.sum(allel.mean_pairwise_difference(ac[pop], fill=0)),
            theta_tajima_n_policy='source n=max(called AN); no imputed invariant genotypes')
        add_windows(pop + '_pi', allel.windowed_diversity(pos, ac[pop], windows=windows, is_accessible=accessible), True)
        add_windows(pop + '_theta', allel.windowed_watterson_theta(pos, ac[pop], windows=windows, is_accessible=accessible), True)
        add_windows(pop + '_tajima_d', allel.windowed_tajima_d(pos, ac[pop], windows=windows, min_sites=3), False)
    region_results['between'] = dict(dxy=allel.sequence_divergence(pos, ac[first], ac[second], **region),
                                    fixed_differences=int(np.count_nonzero(fixed)),
                                    fixed_density=np.count_nonzero(fixed) / accessible.sum(),
                                    hudson_num_sum=np.nansum(num), hudson_den_sum=np.nansum(den),
                                    wc_a_sum=np.nansum(a), wc_b_sum=np.nansum(b), wc_c_sum=np.nansum(c))
    add_windows('dxy', allel.windowed_divergence(pos, ac[first], ac[second], windows=windows, is_accessible=accessible), True)
    add_windows('fixed_density', allel.windowed_df(pos, ac[first], ac[second], windows=windows, is_accessible=accessible), True)
    # scikit-allel1.3.13 fst.py windowed-FST reductions; only per-site arrays are shared.
    def hudson_window(wn, wd):
        return np.nansum(wn) / np.nansum(wd)
    def wc_window(wa, wb, wc):
        return np.nansum(wa) / (np.nansum(wa) + np.nansum(wb) + np.nansum(wc))
    add_windows('hudson_fst', allel.windowed_statistic(pos, values=(num, den), statistic=hudson_window,
                                                    windows=windows, fill=np.nan), False)
    add_windows('wc_fst', allel.windowed_statistic(pos, values=(a, b, c), statistic=wc_window,
                                                windows=windows, fill=np.nan), False)
    for name, values in {'hudson_num_sum': num, 'hudson_den_sum': den,
                         'wc_a_sum': a, 'wc_b_sum': b, 'wc_c_sum': c}.items():
        sums, actual, _ = allel.windowed_statistic(pos, values, statistic=np.nansum, windows=windows, fill=0)
        if not np.array_equal(actual, windows):
            raise ValueError('component window identity differs')
        columns[name] = sums
    oracle.table(output / 'windows.tsv.gz', list(columns), zip(*columns.values()))
    blen = spec['jackknife_variants']
    # scikit-allel1.3.13 fst.py:716-759 and :666-713; jackknife keeps its masked sums.
    def average_components(numerator, denominator, point):
        num_bsum = allel.moving_statistic(numerator, statistic=np.nansum, size=blen)
        den_bsum = allel.moving_statistic(denominator, statistic=np.nansum, size=blen)
        vb = num_bsum / den_bsum
        _, se, vj = allel.jackknife((num_bsum, den_bsum),
                                   statistic=lambda n, d: np.sum(n) / np.sum(d))
        return point, se, vb, vj
    hudson = average_components(num, den, np.nansum(num) / np.nansum(den))
    a_sum = np.nansum(a)
    b_sum = np.nansum(b)
    c_sum = np.nansum(c)
    weir_cockerham = average_components(a, a + b + c, a_sum / (a_sum + b_sum + c_sum))
    fst = {}
    calls = {'hudson': hudson, 'weir_cockerham': weir_cockerham}
    blocks = list(allel.index_windows(pos, size=blen, start=0, stop=len(pos), step=blen))
    for name, (point, se, vb, vj) in calls.items():
        fst[name] = dict(point=point, se=se, vb=vb, vj=vj)
    fst.update(profile='source variant-count block jackknife; all-row point estimate; incomplete residual tail excluded from resampling; not demographic confidence interval',
               block_size=blen, point_rows=len(pos), jackknife_rows=len(blocks) * blen,
               residual_tail_rows=len(pos) - len(blocks) * blen,
               residual_tail_index_range=[len(blocks) * blen, len(pos)],
               residual_tail_ordinals=selected['ordinal'][len(blocks) * blen:])
    oracle.table(output / 'jackknife-membership.tsv.gz', ['block0', 'start_index0', 'stop_index_exclusive', 'first_ordinal0', 'last_ordinal0', 'first_pos1', 'last_pos1'],
                 ((i, start, stop, selected['ordinal'][start], selected['ordinal'][stop - 1], pos[start], pos[stop - 1])
                  for i, (start, stop) in enumerate(blocks)))
    oracle.save(output / 'region.json', region_results)
    oracle.save(output / 'fst-jackknife.json', fst)
    oracle.save(output / 'sfs.json', oracle.spectra(np, allel, ac, selected['n_alt'], ancestor_indices, populations, indices))
    count_digest = hashlib.sha256()
    for pop in populations:
        count_digest.update(pop.encode() + b'\0')
        count_digest.update(np.asarray(ac[pop], dtype='<i8').tobytes(order='C'))
    return dict(chrom=chrom, retained_rows=len(pos), window_rows=len(windows), allele_counts_sha256=count_digest.hexdigest(),
                allele_counts_identity='population insertion order, UTF8 name NUL, full-width int64 little-endian variant/allele counts',
                output_files={path.name: dict(size=path.stat().st_size, sha256=inputs.sha256(path))
                              for path in output.iterdir() if path.is_file()})


def run_legacy(output, executable, spec, prepared, workers):
    output.mkdir(mode=0o700)
    commands = oracle.legacy_commands(output, str(executable), spec['window_bp'], prepared)
    pending = list(enumerate(commands))
    active = {}
    completed = {}
    try:
        while pending or active:
            while pending and len(active) < workers:
                index, command = pending.pop(0)
                call = output.parent / 'legacy-calls' / command['name']
                call.mkdir(parents=True, mode=0o700)
                save(call / 'request.json', command)
                stdout, stderr = (call / 'stdout.txt').open('xb'), (call / 'stderr.txt').open('xb')
                process = subprocess.Popen(command['argv'], stdout=stdout, stderr=stderr)
                active[process.pid] = (index, command, process, stdout, stderr, time.monotonic(), call)
            for pid, (index, command, process, stdout, stderr, started, call) in list(active.items()):
                child, status, usage = os.wait4(pid, os.WNOHANG)
                if not child:
                    continue
                process.returncode = os.waitstatus_to_exitcode(status)
                stdout.close()
                stderr.close()
                record = dict(**command, returncode=process.returncode, elapsed_seconds=time.monotonic() - started,
                              kernel_rusage=rusage(usage), inherited_affinity=sorted(os.sched_getaffinity(0)))
                save(call / 'status.json', record)
                completed[index] = record
                del active[pid]
                check(process.returncode == 0, 'real vcftools legacy command failed: ' + command['name'])
            if active:
                time.sleep(0.02)
    finally:
        for _, _, process, stdout, stderr, _, _ in active.values():
            if process.returncode is None:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
            stdout.close()
            stderr.close()
    calls = [completed[index] for index in range(5)]
    save(output / 'commands.json', calls)
    return calls


def finish_source(output):
    manifest = {path.relative_to(output).as_posix(): dict(size=path.stat().st_size, sha256=inputs.sha256(path))
                for path in sorted(output.rglob('*')) if path.is_file()}
    save(output / 'output-manifest.json', manifest)
    for path in sorted(output.rglob('*')):
        if path.is_file():
            path.chmod(0o600)
            with path.open('rb') as stream:
                os.fsync(stream.fileno())
    for path in sorted((path for path in output.rglob('*') if path.is_dir()), reverse=True):
        _source_fsync_directory(path)
    _source_fsync_directory(output)
    _source_fsync_directory(output.parent)


def source_worker(options_path):
    private = paths()['private']
    check(Path(options_path).resolve(strict=True).is_relative_to(private), 'source request escapes private root')
    options = load(options_path)
    spec = load(options['specification'])
    admission = copy.deepcopy(load(options['admission']))
    cached = load(options['runtime'])
    output = Path(options['output'])
    check(output.resolve().is_relative_to(private) and not output.exists(), 'fresh private source output required')
    output.mkdir(mode=0o700)
    import numpy as np
    import allel
    check(np.__version__ == '1.26.4' and allel.__version__ == '1.3.13', 'timed imported source versions differ')
    source_paths = oracle.bind_admission(Path(options['inputs']), spec, admission)
    phases = {}
    started = time.monotonic()
    captured = []
    try:
        with warnings.catch_warnings(record=True) as captured:
            warnings.simplefilter('always')
            masks = inputs.load_masks(source_paths, spec)
            data = allel.read_vcf(str(source_paths['vcf']), **oracle.read_options(admission))
            check(data is not None and data['calldata/GT'].shape == tuple(admission['gt_shape'])
                  and data['samples'].tolist() == admission['sample_ids'], 'fresh scientific parse shape/sample order differs')
            selected, counts = oracle.eligible_data(np, data, masks)
            del data
            check(counts == admission['filter_counts'], 'fresh source filters differ from shared admission')
            eligible = set()
            for begin in range(0, len(selected['ordinal']), 8192):
                end = begin + 8192
                complete = np.all(selected['calldata/GT'][begin:end] >= 0, axis=(1, 2))
                keep = complete & (selected['n_alt'][begin:end] == 1)
                eligible.update(map(int, selected['ordinal'][begin:end][keep]))
            phases['fresh_parse_filter_eligibility_seconds'] = time.monotonic() - started
            point = time.monotonic()
            results = []
            for chrom in spec['contigs']:
                rows = selected['variants/CHROM'] == chrom
                contig = selected if rows.all() else {key: values[rows] for key, values in selected.items()}
                results.append(source_statistics(np, allel, chrom, contig, masks[chrom], admission, spec, output / chrom))
            phases['shared_components_complete_canonical_exports_seconds'] = time.monotonic() - point
            del selected
            point = time.monotonic()
            prepared = Path(options['derivative'])
            prepared.mkdir(mode=0o700)
            admission['legacy'] = write_legacy(Path(options['inputs']), spec, admission, eligible, prepared)
            phases['lightweight_derivative_serialization_seconds'] = time.monotonic() - point
            point = time.monotonic()
            calls = run_legacy(output / 'vcftools', Path(cached['vcftools']), spec, prepared, options['workers'])
            phases['five_real_legacy_commands_seconds'] = time.monotonic() - point
            if 'independent_truth' in spec:
                oracle.check_casebook_results(output, spec['independent_truth'])
            oracle.save(output / 'result.json', dict(contigs=results,
                selected_gt_stream_sha256=admission['selected_gt_stream_sha256'],
                record_stream_sha256=admission['record_stream_sha256'],
                exclusions='selection/admixture/PBS/LD; individual callability; genome-wide invariant SFS; demographic/clinical ancestry claims',
                source_calls='pinned scikit-allel1.3.13 canonical source; vcftools0.1.17 named legacy lane',
                vcftools=dict(executable_sha256=cached['vcftools_identity']['sha256'], derivative=admission['legacy'],
                    profile='named vcftools legacy bins/denominators, not canonical project-accessibility estimates', calls=calls)))
            save(output / 'runtime.json', dict(implementation='pinned-source-shared-workflow',
                admitted_runtime=dict(python_identity=cached['python_identity'], vcftools_identity=cached['vcftools_identity'],
                    runtime_lock=cached['runtime_lock'], versions=cached['imported']['versions']),
                python=sys.version, executable=str(Path(sys.executable).resolve()),
                numpy_file=np.__file__, scikit_allel_file=allel.__file__, workers=options['workers'],
                thread_libraries={name: os.environ.get(name) for name in ('OMP_NUM_THREADS', 'OPENBLAS_NUM_THREADS',
                    'MKL_NUM_THREADS', 'BLIS_NUM_THREADS', 'NUMEXPR_NUM_THREADS')}))
            save(output / 'input-admission.json', admission)
            save(output / 'runtime-lock.json', load(hosted.RUNTIME))
            save(output / 'filter-counts.json', counts)
            save(output / 'request.json', dict(argv=sys.argv, cwd=str(Path.cwd()), source_workflow='shared-components-v1'))
            save(output / 'warnings.json', [dict(category=row.category.__name__, message=str(row.message),
                filename=row.filename, lineno=row.lineno) for row in captured])
            finish_source(output)
    except BaseException:
        (output.parent / 'source-failure.txt').write_text(traceback.format_exc())
        raise
    finally:
        save(output.parent / 'source-phases.json', dict(phases=phases,
            scope='descriptive timed subphases; not substituted for external whole-process wall',
            self_kernel_rusage=rusage(resource.getrusage(resource.RUSAGE_SELF)),
            children_kernel_rusage=rusage(resource.getrusage(resource.RUSAGE_CHILDREN))))


def scientific_payloads(output):
    result = {}
    for path in sorted(Path(output).rglob('*')):
        name = path.relative_to(output).as_posix()
        scientific = path.parent.name == 'vcftools' and name.endswith(('.sites.pi', '.windowed.pi', '.Tajima.D', '.weir.fst'))
        scientific |= path.parent.name != 'vcftools' and path.name in (
            'sites.tsv.gz', 'windows.tsv.gz', 'region.json', 'sfs.json', 'fst-jackknife.json', 'jackknife-membership.tsv.gz')
        if scientific:
            check(path.is_file() and not path.is_symlink(), 'regular scientific payload required')
            with (gzip.open(path, 'rb') if name.endswith('.gz') else path.open('rb')) as stream:
                result[name] = hashlib.file_digest(stream, 'sha256').hexdigest()
    return result


def source_view(destination, output, specification, *, casebook=False):
    destination.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    destination.mkdir(mode=0o700)
    prefix = 'casebook-source' if casebook else 'real-source'
    manifest = load(output / 'output-manifest.json')
    actual = {path.relative_to(output).as_posix() for path in output.rglob('*') if path.is_file()}
    check(actual == set(manifest) | {'output-manifest.json'}, 'fresh source manifest closure differs')
    files = {}
    for name in sorted(actual):
        path = output / name
        observed = identity(path)
        if name != 'output-manifest.json':
            check(observed == dict(bytes=manifest[name]['size'], sha256=manifest[name]['sha256']),
                  'fresh source output body differs')
        files[prefix + '/' + name] = dict(**observed, extracted_path=str(path.resolve()))
    key = 'casebook/specification.json' if casebook else 'controller/assets/popgen-first-input-20261101.json'
    files[key] = dict(**identity(specification), extracted_path=str(Path(specification).resolve()))
    save(destination / 'tar-verification.json', dict(kind='fresh-source-output-view',
        archive_authentication_claim=False, files=files))
    return destination


def comparison_module():
    policy = load_policy()
    spec = importlib.util.spec_from_file_location('popgen_frozen_comparison', ROOT / policy['comparator']['path'])
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def compare_pair(private, name, source, native, specification, *, casebook=False, twin=None):
    view = source_view(private / 'source-views' / name, source, specification, casebook=casebook)
    report = comparison_module().compare(view, 'casebook-source' if casebook else 'real-source', native, twin)
    check(report['status'] == 'passed', 'full fresh source/native comparison failed')
    if not casebook:
        check(report['compared_cells'] == load_policy()['population']['comparison_cells'], 'full comparison population differs')
    save(private / 'comparisons' / (name + '.json'), report)
    return report


def advantage(records, policy):
    evaluated = {}
    material = policy['material']
    for workers in (1, 4):
        ratios = {'wall': [], 'cpu': [], 'rss': []}
        for pair in range(1, 6):
            rows = [row for row in records if row['workers'] == workers and row['pair'] == pair and not row['warmup']]
            check(len(rows) == 2 and {row['engine'] for row in rows} == {'source', 'native'}, 'five complete paired records required')
            native = next(row for row in rows if row['engine'] == 'native')
            source = next(row for row in rows if row['engine'] == 'source')
            for key, field in (('wall', 'wall_seconds'), ('cpu', 'total_cpu_seconds'), ('rss', 'kernel_maxrss_kib')):
                check(source[field] > 0 and math.isfinite(source[field]) and math.isfinite(native[field]), 'invalid actual resource observation')
                ratios[key].append(native[field] / source[field])
        medians = {key: statistics.median(values) for key, values in ratios.items()}
        throughput = (medians['wall'] <= material['throughput_median_wall_ratio_max']
            and sum(ratio <= material['throughput_pair_wall_ratio_max'] for ratio in ratios['wall']) >= material['throughput_pairs_required']
            and medians['cpu'] <= material['cpu_ratio_max'] and medians['rss'] <= material['throughput_rss_ratio_max'])
        memory = (medians['rss'] <= material['resource_median_rss_ratio_max']
            and all(ratio < material['resource_all_memory_ratios_below'] for ratio in ratios['rss'])
            and medians['wall'] <= material['resource_wall_ratio_max'] and medians['cpu'] <= material['cpu_ratio_max'])
        evaluated[str(workers)] = dict(paired_native_over_source=ratios, medians=medians,
            throughput_material=throughput, conservative_memory_material=memory, material=throughput or memory,
            rss_scope='native single-process kernel peak / source largest-process kernel peak lower bound')
    return dict(status='observations-await-root-original-source-adoption', allowances=evaluated,
        material_advantage_observed=all(row['material'] for row in evaluated.values()),
        root_original_source_adoption_required=True, performance_qualified=False, release=False)


def native_command(binary, input_root, spec, output, workers):
    original = inputs.input_paths(input_root, spec)
    argv = [str(binary), 'analyze', '--vcf', str(original['vcf']), '--panel', str(original['panel'])]
    for population in spec['population_order']:
        argv.extend(['--population', population])
    argv.extend(['--reference-fai', str(original['reference_fai'])])
    for chrom, contig in spec['contigs'].items():
        argv.extend(['--mask', chrom + '=' + str(original[contig['mask_file']])])
    if 'tbi' in original:
        argv.extend(['--index', str(original['tbi'])])
    argv.extend(['--window-bp', str(spec['window_bp']), '--jackknife-variants', str(spec['jackknife_variants']),
        '--legacy-vcftools', '--out', str(output), '--threads', str(workers), '--memory-mib', '512'])
    return argv


def run_one(private, request, python, native, input_root, specification, admission, runtime, affinity):
    run = private / 'runs' / request['name']
    run.mkdir(parents=True, mode=0o700)
    output = run / 'output'
    env = dict(os.environ)
    for name in ('OMP_NUM_THREADS', 'OPENBLAS_NUM_THREADS', 'MKL_NUM_THREADS', 'BLIS_NUM_THREADS', 'NUMEXPR_NUM_THREADS'):
        env[name] = str(request['workers'])
    if request['engine'] == 'native':
        argv = native_command(native, input_root, load(specification), output, request['workers'])
    else:
        options = dict(inputs=str(input_root), specification=str(specification), admission=str(admission),
            runtime=str(runtime), output=str(output), derivative=str(run / 'legacy-inputs'), workers=request['workers'])
        save(run / 'source-options.json', options)
        argv = [python, '-B', Path(__file__), 'source-worker', '--options', run / 'source-options.json']
    call = capture(run, 'process', argv, cwd=ROOT, affinity=affinity[:request['workers']], environment=env,
                   timeout=load_policy()['limits']['call_timeout_seconds'])
    status = load(call / 'status.json')
    usage = status['kernel_rusage']
    result = dict(**request, output=str(output.relative_to(private)), returncode=status['returncode'],
        wall_seconds=status['elapsed_seconds'], total_cpu_seconds=usage['ru_utime'] + usage['ru_stime'],
        user_seconds=usage['ru_utime'], system_seconds=usage['ru_stime'], kernel_maxrss_kib=usage['ru_maxrss'],
        sampled_tree_rss_peak_bytes=status['sampled_descendant_rss_peak_bytes'],
        kernel_rusage=usage, cgroup_before=status['cgroup_before'], cgroup_after=status['cgroup_after'],
        rss_scope='kernel maximum of a process; for source, a lower bound on simultaneous tree peak',
        scientific_payloads=scientific_payloads(output))
    check(len(result['scientific_payloads']) == (17 if len(load(specification)['contigs']) == 2 else 11),
          'complete scientific payload membership differs')
    save(run / 'execution.json', result)
    return result


def capacity_check(private, policy):
    observation = hosted.capacity(private)
    base = sum(hosted.retained_size(path) for path in private.iterdir() if path.name not in ('runs', 'calls', 'comparisons', 'source-views') and path.is_dir())
    check(base <= policy['retention']['base_bytes'], 'base runtime/data/native envelope exceeded')
    runs = private / 'runs'
    for run in runs.iterdir() if runs.exists() else []:
        if run.name.startswith('casebook-'):
            continue
        engine = 'source' if run.name.endswith('-source') else 'native'
        check(hosted.retained_size(run) <= policy['retention'][engine + '_run_bytes'],
              'complete per-run retention slot exceeded')
    return observation


def authority_bindings(private, input_root, spec):
    runtime = load(private / 'runtime/admitted-runtime.json')
    candidates = {Path(runtime['python']).resolve(), Path(runtime['vcftools']).resolve(),
                  Path(load(private / 'native-admission.json')['binary']).resolve()}
    for name in runtime['imported']['loaded_file_identities']:
        candidates.add(Path(name).resolve())
    for relative in hosted.SOURCE_ANCHORS:
        candidates.add(Path(runtime['imported']['imported']['allel']).parent.parent / relative)
    candidates.update(inputs.input_paths(input_root, spec).values())
    candidates.update(ROOT / name for name in (POLICY, WORKFLOW, 'scripts/popgen_whole_real.py',
        'scripts/popgen_hosted_source.py', 'scripts/popgen_first_workflow_oracle.py',
        'scripts/popgen_first_workflow_inputs.py', 'scripts/private_artifact.py',
        load_policy()['comparator']['path'], SOURCE_ASSET, str(hosted.RUNTIME.relative_to(ROOT)), NATIVE_ASSET,
        load_policy()['native_receipt']['path'], '.autopilot/oracles/popgen-source-2026-11-01.json',
        str(hosted.RECIPIENT.relative_to(ROOT)), str(transport.TOOLS.relative_to(ROOT))))
    return {str(path): identity(path) for path in sorted(candidates)}


def verify_casebook_goldens(output):
    authority = load(ROOT / NATIVE_ASSET)
    snapshot = ROOT / '.autopilot/snapshots/popgen-native-2026-11-01/green-1/source.tar.gz'
    check(identity(snapshot)['sha256'] == load_policy()['owner']['source_archive_sha256'], 'casebook source carrier differs')
    with tarfile.open(snapshot, 'r:gz') as archive:
        for name, expected in authority['goldens'].items():
            member = archive.getmember('tests/fixtures/neutral/expected/' + name)
            check(member.isfile() and member.type == tarfile.REGTYPE, 'regular original synthetic golden required')
            body = archive.extractfile(member).read()
            check(hashlib.sha256(body).hexdigest() == expected['sha256'], 'actual source golden differs')
            actual = (output / name).read_bytes()
            if name.endswith('.gz'):
                body, actual = gzip.decompress(body), gzip.decompress(actual)
            check(actual == body, 'fresh shared-source casebook differs from original golden: ' + name)


def execute(private):
    policy = load_policy()
    check(not os.environ.get('GH_TOKEN') and not os.environ.get('GITHUB_TOKEN'),
          'drop the API token before source builds or timed execution')
    validate_workflow(ROOT / WORKFLOW)
    native = load(private / 'native-admission.json')
    check(native['native_four_qualified'] is True and native['binary_identity'] == identity(native['binary']),
          'actual native original admission absent or changed')
    affinity = machine_admission(private, policy)
    started = time.monotonic()
    with source_runtime_capture():
        python, vcftools = hosted.prepare_runtime(private)
    runtime = private / 'runtime/admitted-runtime.json'
    check(identity(ROOT / '.autopilot/oracles/popgen-source-2026-11-01.json')['sha256'] == policy['source_receipt_sha256'],
          'adopted original source receipt differs')
    case = inputs.casebook(private / 'casebook')
    case_admission = inputs.admit(private / 'casebook', case)
    save(private / 'casebook-admission.json', case_admission)
    case_records = {}
    for engine in ('source', 'native'):
        request = dict(name='casebook-' + engine, workers=1, warmup=True, pair=0, engine=engine)
        case_records[engine] = run_one(private, request, python, native['binary'], private / 'casebook',
            private / 'casebook/specification.json', private / 'casebook-admission.json', runtime, affinity)
    source_case = private / case_records['source']['output']
    verify_casebook_goldens(source_case)
    compare_pair(private, 'casebook', source_case, private / case_records['native']['output'],
                 private / 'casebook/specification.json', casebook=True)
    save(private / 'casebook-qualified.json', dict(original_goldens=17, fresh_source_and_actual_native=True,
        scientific_imports_executed_here=True, runtime=identity(runtime)))
    specification = ROOT / SOURCE_ASSET
    spec = load(specification)
    real = private / 'real'
    real.mkdir(mode=0o700)
    for name, entry in spec['files'].items():
        expected = dict(entry, **policy['inputs'][name])
        hosted.acquire(private, expected, real / entry['filename'])
    capture(private, 'real-independent-admission', [python, '-B', '-c',
        'import json,sys;from pathlib import Path;sys.path.insert(0,sys.argv[1]);'
        'import popgen_first_workflow_inputs as i;'
        's=json.loads(Path(sys.argv[3]).read_text());i.save(Path(sys.argv[4]),i.admit(Path(sys.argv[2]),s))',
        ROOT / 'scripts', real, specification, private / 'real-admission.json'],
        timeout=policy['limits']['admission_timeout_seconds'])
    admission = load(private / 'real-admission.json')
    check(admission['gt_shape'] == [1103547, 207, 2] and admission['max_alt_number'] == 8
          and admission['filter_counts']['retained'] == 723783, 'unchanged original whole-input population differs')
    for name, path in inputs.input_paths(real, spec).items():
        check(identity(path) == dict(bytes=policy['inputs'][name]['size'], sha256=policy['inputs'][name]['sha256']),
              'complete provider original identity differs')
        path.chmod(0o400)
    frozen = authority_bindings(private, real, spec)
    save(private / 'authorities-before.json', frozen)
    save(private / 'frozen-execution-plan.json', dict(policy=policy, policy_identity=identity(ROOT / POLICY),
        native=native, runtime=load(runtime), original_provider_files=admission['files'],
        schedule=schedule(policy), comparator_identity=identity(ROOT / policy['comparator']['path']),
        timer='fresh exec through whole tree completion, no install/admission/checker/cipher inside',
        address_space_limit=None, rss_guard='same sampled8GiB live tree, not a hard/exact peak'))
    records = []
    outputs = {}
    first_native = None
    first_source_payloads = None
    root_required = {}
    (private / 'source-views').mkdir(mode=0o700, exist_ok=True)
    for request in schedule(policy):
        check(time.monotonic() - started < policy['limits']['controller_compute_seconds'],
              'controller computation deadline reached; full gate held, no dropped repetitions')
        capacity_check(private, policy)
        record = run_one(private, request, python, native['binary'], real, specification,
                         private / 'real-admission.json', runtime, affinity)
        output = private / record['output']
        if request['engine'] == 'native':
            if first_native is None:
                first_native = output
            else:
                check(record['scientific_payloads'] == scientific_payloads(first_native),
                      'native repetition/worker scientific payloads differ')
        else:
            if first_source_payloads is None:
                first_source_payloads = record['scientific_payloads']
            else:
                check(record['scientific_payloads'] == first_source_payloads, 'fresh source repetition/worker payloads differ')
            derivative = load(output / 'input-admission.json')['legacy']
            check(derivative['records'] == 721170 and derivative['files']['eligible.vcf'] ==
                  dict(size=policy['population']['derivative_bytes'], sha256=policy['population']['derivative_sha256']),
                  'actual timed full legacy derivative differs')
            for name, expected in derivative['files'].items():
                check(identity(Path(derivative['root']) / name) == dict(bytes=expected['size'], sha256=expected['sha256']),
                      'retained original legacy derivative/list body differs after timing')
        records.append(record)
        outputs[request['workers'], request['warmup'], request['pair'], request['engine']] = output
        if request['warmup']:
            root_required.setdefault(str(request['workers']), {})[request['engine']] = record['output']
        opposite = 'source' if request['engine'] == 'native' else 'native'
        key = request['workers'], request['warmup'], request['pair'], opposite
        if key in outputs:
            source = output if request['engine'] == 'source' else outputs[key]
            product = output if request['engine'] == 'native' else outputs[key]
            compare_pair(private, request['name'], source, product, specification, twin=first_native)
        save(private / 'execution-records.json', records)
        capacity_check(private, policy)
    check(len(records) == 24 and authority_bindings(private, real, spec) == frozen,
          'full execution population or source/native/provider/controller integrity differs')
    save(private / 'authorities-after.json', frozen)
    save(private / 'root-original-source-adoption-required.json', dict(required=True, performed=False,
        authenticated_original_source_receipt=load(ROOT / '.autopilot/oracles/popgen-source-2026-11-01.json'),
        comparator_identity=identity(ROOT / policy['comparator']['path']), first_by_allowance=root_required,
        action='After local authenticated decryption, root must compare first fresh source AND first native at EACH allowance against original real-source under the frozen comparator, then review exact native payload repetition/worker identities.',
        no_ci_private_key=True, no_old_plaintext_upload=True, performance_qualified=False))
    result = advantage(records, policy)
    save(private / 'provisional-resource-result.json', result)
    save(private / 'complete-observations.json', dict(status='complete-full-chr22-observations-await-root-adoption',
        native_run=native['run_id'], native_binary=native['binary_identity'], full_processes=24,
        full_comparisons=12, comparison_cells_each=50010408, real_casebook_goldens=17,
        material_advantage_observed=result['material_advantage_observed'],
        root_original_source_adoption_required=True, performance_qualified=False, release=False))
    check(result['material_advantage_observed'], 'fixed material advantage criteria not met; complete observations retained, qualification held')


def finalize(session):
    private = session['private']
    hosted.recipient()
    tool = transport.prepare_tool('linux-amd64', private / 'age-tool')
    members = hosted.archive_inventory(private)
    save(private / 'archive-members.json', dict(members=members,
        scope='all available quiescent private originals/runtime/logs/24 outputs/intermediates; self-index excluded once',
        limitation='host loss, cancellation, encryption failure and finalization diagnostics are not zero-loss retention'))
    members = hosted.archive_inventory(private)
    manifest = session['session'] / 'sealed-members.json'
    save(manifest, members)
    bound = sum(row['bytes'] for row in members) + len(members) * 8192 + 10240
    check(bound <= load_policy()['retention']['tar_bound_bytes'], 'complete private TAR exceeds frozen bound')
    space = transport.finalization_space(sum(row['bytes'] for row in members), bound)
    check(shutil.disk_usage(private).free >= space['required_free_bytes'], 'cipher/full-original headroom absent')
    outputs = hosted.public_outputs(session['session'])
    receipt = transport.encrypt(tool, hosted.RECIPIENT, outputs['payload.tar.age'],
        producer=[sys.executable, '-B', Path(__file__), 'tar', '--root', private, '--members', manifest],
        plaintext_limit=bound, context=dict(repository='omics-rust/rsomics-world', product='rsomics-popgen',
            head_sha=os.environ['GITHUB_SHA'], run_id=int(os.environ['GITHUB_RUN_ID']),
            run_attempt=int(os.environ['GITHUB_RUN_ATTEMPT']), workflow_path=WORKFLOW, phase='whole-real-performance'))
    save(outputs['receipt.json'], receipt)


def recovered_body(packet, mapping, name):
    portable(name)
    entry = mapping['files'][name]
    path = packet / entry['extracted_path']
    check(path.resolve(strict=True).is_relative_to(packet.resolve(strict=True)) and not path.is_symlink(),
          'recovered body escapes authenticated regular CAS packet')
    check(identity(path) == dict(bytes=entry['bytes'], sha256=entry['sha256']), 'recovered original body identity differs')
    return path


def compare_original_source(original, source_file):
    frozen = comparison_module()
    mapping = load(original / 'tar-verification.json')
    expected = lambda name: recovered_body(original, mapping, 'real-source/' + name)
    read_expected = lambda name: load(expected(name))
    science = frozen.Comparison()
    old_admission, new_admission = read_expected('input-admission.json'), load(source_file('input-admission.json'))
    check(set(old_admission) == set(new_admission), 'fresh/original source admission key population differs')
    for key in set(old_admission) - {'files', 'legacy'}:
        science.value(new_admission[key], old_admission[key], 'source-admission/' + key)
    body_ids = lambda records: sorted((row['sha256'], row.get('size', row.get('bytes'))) for row in records.values())
    check(body_ids(old_admission['files']) == body_ids(new_admission['files']), 'source original input identities differ')
    for key in ('files', 'records', 'input_gt_stream_sha256', 'original_record_profile'):
        science.value(new_admission['legacy'][key], old_admission['legacy'][key], 'source-legacy/' + key)
    science.value(load(source_file('filter-counts.json')), read_expected('filter-counts.json'), 'source-filters')
    old_result, new_result = read_expected('result.json'), load(source_file('result.json'))
    source_manifest = load(source_file('output-manifest.json'))
    check(set(old_result) == set(new_result), 'fresh/original source result field population differs')
    for key in ('selected_gt_stream_sha256', 'record_stream_sha256', 'exclusions', 'source_calls'):
        science.value(new_result[key], old_result[key], 'source-result/' + key)
    check(len(old_result['contigs']) == len(new_result['contigs']), 'source contig population differs')
    rows = {}
    payloads = set()
    for current, old in zip(new_result['contigs'], old_result['contigs']):
        check(set(current) == set(old), 'source contig result fields differ')
        for key in set(old) - {'output_files'}:
            science.value(current[key], old[key], 'source-result/' + old['chrom'] + '/' + key)
        check(set(current['output_files']) == set(old['output_files']) == set(frozen.SCIENCE_NAMES),
              'source contig scientific member population differs')
        check(current['output_files'] == {name: source_manifest[old['chrom'] + '/' + name] for name in frozen.SCIENCE_NAMES},
              'fresh source result carrier identities differ from own manifest')
        for suffix in frozen.SCIENCE_NAMES:
            name = old['chrom'] + '/' + suffix
            payloads.add(name)
            if suffix.endswith('.json'):
                science.value(load(source_file(name)), read_expected(name), name)
            else:
                rows[name] = science.table(source_file(name), expected(name), name)
    current_legacy, old_legacy = new_result['vcftools'], old_result['vcftools']
    check(set(current_legacy) == {'executable_sha256', 'profile', 'derivative', 'calls'}
          and set(old_legacy) == {'version_stdout', 'version_stderr', 'executable_sha256', 'profile', 'derivative', 'calls'},
          'unexpected source legacy producer/scientific fields')
    science.value(current_legacy['profile'], old_legacy['profile'], 'source-legacy-profile')
    for key in ('files', 'records', 'input_gt_stream_sha256', 'original_record_profile'):
        science.value(current_legacy['derivative'][key], old_legacy['derivative'][key], 'source-result-legacy/' + key)
    operations = ['site-pi', 'window-pi', 'tajima-d', 'weir-fst', 'window-weir-fst']
    check([row['name'] for row in current_legacy['calls']] == [row['name'] for row in old_legacy['calls']] == operations
          and all(row['returncode'] == 0 for row in current_legacy['calls']), 'all five actual source legacy commands required')
    def flags(command):
        argv = command['argv'][1:]
        result = []
        for index, token in enumerate(argv):
            result.append('<producer-path>' if index and argv[index - 1] in ('--vcf', '--out', '--weir-fst-pop') else token)
        return result
    check([flags(row) for row in current_legacy['calls']] == [flags(row) for row in old_legacy['calls']],
          'actual original legacy operation flags differ')
    for suffix in frozen.LEGACY_NAMES:
        name = 'vcftools/' + suffix
        payloads.add(name)
        rows[name] = science.table(source_file(name), expected(name), name, legacy=True)
    check(len(payloads) == 11, 'complete original-source scientific population absent')
    return dict(status='passed', policy=frozen.POLICY, table_rows=rows, scientific_payloads=sorted(payloads),
        compared_cells=science.cells, max_finite_absolute_difference=science.max_abs,
        max_canonical_budget_fraction=science.max_scaled,
        producer_exceptions='actual runtime/executable/paths/timing/resource records; scientific and discrete identity fields checked')


def root_original_adoption(original, recovered, report):
    check(os.environ.get('GITHUB_ACTIONS') != 'true', 'original-source adoption runs locally after authenticated decryption only')
    check(local_report_allowed(report), 'root comparison report must stay on the allowed external scratch/fixture roots')
    source_receipt = load(ROOT / '.autopilot/oracles/popgen-source-2026-11-01.json')
    check(not report.exists(), 'fresh external root comparison report required')
    check(identity(original / 'COMPLETED.json')['sha256'] == source_receipt['completed_sha256'],
          'original authenticated source packet seal differs')
    mapping = load(recovered / 'tar-verification.json')
    check(mapping.get('kind') != 'fresh-source-output-view', 'authenticated artifact recovery, not a CI view, is required')
    fetch = lambda name: recovered_body(recovered, mapping, name)
    required = load(fetch('root-original-source-adoption-required.json'))
    records = load(fetch('execution-records.json'))
    check(required['required'] is True and required['performed'] is False and len(records) == 24,
          'complete unapplied root-adoption request required')
    check(set(required['first_by_allowance']) == {'1', '4'}, 'both original CPU allowances required')
    report.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    views = report.parent / (report.stem + '-regular-native-views')
    views.mkdir(mode=0o700)
    results = {}
    first_native = None
    reference_payloads = None
    for workers in ('1', '4'):
        pair = required['first_by_allowance'][workers]
        check(set(pair) == {'source', 'native'}, 'first source AND native original comparisons required')
        source_file = lambda name: fetch(pair['source'] + '/' + name)
        source_manifest = load(source_file('output-manifest.json'))
        for name, expected in source_manifest.items():
            check(identity(source_file(name)) == dict(bytes=expected['size'], sha256=expected['sha256']),
                  'fresh source own complete manifest differs')
        derived = load(source_file('input-admission.json'))['legacy']
        derivative_prefix = pair['source'].rsplit('/', 1)[0] + '/legacy-inputs/'
        for name, expected in derived['files'].items():
            check(identity(fetch(derivative_prefix + name)) == dict(bytes=expected['size'], sha256=expected['sha256']),
                  'recovered actual source derivative/list body differs')
        results['source-' + workers] = compare_original_source(original, source_file)
        native_view = views / ('b' + workers)
        native_view.mkdir(mode=0o700)
        native_manifest = load(fetch(pair['native'] + '/output-manifest.json'))
        for name in [*native_manifest, 'output-manifest.json']:
            portable(name)
            source = fetch(pair['native'] + '/' + name)
            target = native_view / name
            target.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
            with source.open('rb') as stream, target.open('xb') as output:
                shutil.copyfileobj(stream, output)
            target.chmod(0o600)
        results['native-' + workers] = comparison_module().compare(original, 'real-source', native_view, first_native)
        if first_native is None:
            first_native = native_view
            reference_payloads = scientific_payloads(native_view)
    for row in records:
        if row['engine'] != 'native':
            continue
        actual = {}
        for name in reference_payloads:
            path = fetch(row['output'] + '/' + name)
            with (gzip.open(path, 'rb') if name.endswith('.gz') else path.open('rb')) as stream:
                actual[name] = hashlib.file_digest(stream, 'sha256').hexdigest()
        check(actual == reference_payloads, 'actual recovered native repetition/worker payloads differ')
    save(report, dict(status='original-source-comparisons-passed-await-root-custody-and-performance-decision',
        comparisons=results, native_payload_repeat_worker_exact=True,
        scientific_checker_identity=identity(ROOT / load_policy()['comparator']['path']),
        custody_must_be_independently_adopted=True, no_private_key_read=True, performance_qualified=False, release=False))


def local_report_allowed(path):
    roots = (Path('/Volumes/KIOXIA/Developments/tmp'), Path("/Volumes/Zane's HDD/rsomics-fixtures"))
    return any(root.is_dir() and path.resolve().is_relative_to(root.resolve(strict=True)) for root in roots)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['init', 'transport', 'admit-native', 'execute', 'source-worker', 'finalize', 'tar', 'adopt-original'])
    parser.add_argument('--native-receipt', type=Path)
    parser.add_argument('--options', type=Path)
    parser.add_argument('--root', type=Path)
    parser.add_argument('--members', type=Path)
    parser.add_argument('--original-packet', type=Path)
    parser.add_argument('--recovered-packet', type=Path)
    parser.add_argument('--report', type=Path)
    args = parser.parse_args()
    session = None
    try:
        if args.action != 'source-worker':
            load_policy()
        if args.action == 'adopt-original':
            check(args.original_packet is not None and args.recovered_packet is not None and args.report is not None,
                  'root adoption requires original and authenticated recovered packets plus a fresh report')
            root_original_adoption(args.original_packet, args.recovered_packet, args.report)
            return 0
        if args.action == 'init':
            check(args.native_receipt is not None, 'actual tracked native receipt required before acquisition')
            read_native_receipt(args.native_receipt)
            hosted.initialize()
            session = paths()
            validate_workflow(ROOT / WORKFLOW)
            controller = session['private'] / 'controller'
            names = (WORKFLOW, POLICY, 'scripts/popgen_whole_real.py', 'scripts/popgen_hosted_source.py',
                'scripts/popgen_first_workflow_oracle.py', 'scripts/popgen_first_workflow_inputs.py',
                'scripts/private_artifact.py', load_policy()['comparator']['path'], SOURCE_ASSET,
                str(hosted.RUNTIME.relative_to(ROOT)), str(args.native_receipt.resolve().relative_to(ROOT)))
            for name in names:
                target = controller / name
                target.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
                shutil.copy2(ROOT / name, target)
            save(controller / 'identities.json', {name: identity(controller / name) for name in names})
            return 0
        session = paths()
        if args.action == 'transport':
            hosted.prepare_transport(session)
        elif args.action == 'admit-native':
            check(args.native_receipt is not None, 'actual native receipt required')
            admit_native(session['private'], args.native_receipt)
        elif args.action == 'execute':
            execute(session['private'])
        elif args.action == 'source-worker':
            check(args.options is not None, 'source request required')
            source_worker(args.options)
        elif args.action == 'finalize':
            finalize(session)
        else:
            check(args.root is not None and args.root.resolve() == session['private']
                  and args.members is not None and args.members.resolve().is_relative_to(session['session']),
                  'private streamed TAR routes differ')
            hosted.write_tar(args.root, load(args.members), sys.stdout.buffer)
        return 0
    except BaseException:
        if args.action == 'adopt-original' and args.report is not None and local_report_allowed(args.report):
            args.report.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
            failure = args.report.with_suffix('.failure.txt')
            if not failure.exists():
                with failure.open('x') as stream:
                    stream.write(traceback.format_exc())
                failure.chmod(0o600)
        if session is None and os.environ.get('POPGEN_SESSION'):
            session = paths()
        if session is not None:
            target = session['session'] if args.action in ('finalize', 'tar') else session['private']
            (target / (args.action + '-failure.txt')).write_text(traceback.format_exc())
        print('private whole-workflow stage failed; scientific diagnostics remain private', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
