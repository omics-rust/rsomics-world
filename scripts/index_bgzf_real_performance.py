#!/usr/bin/env python3
"""Manual Linux-only complete thirteen-lane Index BGZF/tabix campaign."""

import argparse
import ast
from contextlib import contextmanager
from decimal import Decimal
from functools import lru_cache
import gzip
import hashlib
import json
import math
import os
from pathlib import Path, PurePosixPath
import platform
import posixpath
import re
import resource
import shutil
import signal
import stat
import statistics
import sys
import tarfile
import time
import traceback
import urllib.error
import urllib.parse
import urllib.request
import zipfile

import index_reference_source as source
from infercnv_trial_process import TrialProcessError, run_child
from plink_first_workflow_oracle import download, save, sha256


ROOT = Path(__file__).resolve().parents[1]
ASSET = 'assets/index-bgzf-real-20261101.json'
WORKFLOW = '.github/workflows/index-bgzf-real-performance.yml'
PLAN = 'docs/plans/2026-11-01-index-bgzf-real-performance.md'
OWNER = '030be20895cb7806f78b2923e1c1ac3afa05cd96'
LANES = (
    ('compress_text_t1', 1, 45), ('compress_text_t4', 4, 45),
    ('compress_binary_t1', 1, 60), ('compress_binary_t4', 4, 60),
    ('decompress_text_t1', 1, 10), ('decompress_binary_t1', 1, 45),
    ('partial_binary_512m', 1, 10), ('build_tbi', 1, 30), ('build_csi', 1, 30),
    ('query_sparse_t1', 1, 120), ('query_dense_t4', 4, 10),
    ('query_overlap_unique_t4', 4, 30), ('query_targets_t4', 4, 30))
PRIMARY = ('compress_binary_t1', 'build_tbi', 'build_csi', 'query_sparse_t1')
CATEGORIES = dict(inputs=8 * 2**30, runtime=4 * 2**30, cas=15 * 2**30,
                  metadata=2 * 2**30, active=3 * 2**30)
CACHE_KEYS = source.CACHE_KEYS + ('CARGO_HOME', 'CARGO_TARGET_DIR', 'RUSTUP_HOME', 'PIP_CACHE_DIR')
CONTEXT = (ASSET, WORKFLOW, PLAN, 'scripts/index_bgzf_real_performance.py',
           'scripts/test_index_bgzf_real_performance.py', 'scripts/index_reference_source.py',
           'scripts/plink_first_workflow_oracle.py', 'scripts/plink_first_workflow_inputs.py',
           'scripts/infercnv_trial_process.py', 'scripts/rnaseq_qc_performance.py')
EXEC_REDIRECT = 'set -euo pipefail; ulimit -c 0; ulimit -f "$1"; output=$2; shift 2; exec > "$output"; exec "$@"'
VCF_AWK = '''BEGIN {
    print "##fileformat=VCFv4.3"
    print "##INFO=<ID=PAYLOAD,Number=1,Type=String,Description=\\"Deterministic benchmark payload\\">"
    print "##INFO=<ID=N,Number=1,Type=Integer,Description=\\"Record ordinal\\">"
    for (chromosome = 1; chromosome <= 24; chromosome++)
        print "##contig=<ID=chr" chromosome ",length=" per ">"
    print "#CHROM\\tPOS\\tID\\tREF\\tALT\\tQUAL\\tFILTER\\tINFO"
    for (i = 0; i < records; i++) {
        chromosome = int(i / per) + 1
        position = i % per + 1
        print "chr" chromosome "\\t" position "\\t.\\tA\\tC\\t50\\tPASS\\tPAYLOAD=" filler ";N=" i
    }
}'''


def load(path):
    return json.loads(Path(path).read_text())


def check(ok, message):
    if not ok:
        raise ValueError(message)


def identity(path):
    return source.identity(Path(path))


def matches(path, pin):
    check(identity(path) == {k: pin[k] for k in ('bytes', 'sha256')},
          'complete body identity differs: ' + str(path))


def read_policy(world=ROOT):
    p = load(Path(world) / ASSET)
    check(p['schema'] == 'rsomics.index.bgzf-real.v1' and p['owner'] == OWNER
          and p['owner_ci'] == 41272452578 and p['image']['manifest'] == source.IMAGE
          and p['image']['config'] == source.IMAGE_CONFIG, 'frozen owner/image policy differs')
    check(p['limits']['categories'] == CATEGORIES and p['limits']['controller_seconds'] == 18600
          and p['limits']['initial_free_bytes'] == 41 * 2**30
          and p['limits']['headroom_bytes'] == 8 * 2**30
          and p['limits']['retained_bytes'] == 32 * 2**30
          and p['recovery']['bytes'] == 76 * 2**30 and p['recovery']['allocated'] is False,
          'frozen retention/time/recovery policy differs')
    check(p['material'] == dict(primary=list(PRIMARY), throughput_median_ratio=.9,
          rss_median_ratio=.75, rss_wall_ratio=1.05, disjoint_wall_gap_exceeds_seconds=.01,
          all_primary_each_qualify=True, aggregate_gate=False, publication_automatic=False),
          'pre-outcome per-primary material policy differs')
    check(p['timer']['deb']['sha256'] == '10257c6fe888e6578550b1f20705d282dd630b76f8df0c006aec3031fefe5e23'
          and p['timer']['deb']['bytes'] == 50776, 'exact Debian timer package differs')
    return p


def verify_context(world, policy):
    world = Path(world)
    pin = policy['accepted_reference']
    matches(world / pin['path'], pin)
    receipt = load(world / pin['path'])
    mapping = receipt['owner_source_git_mapping']
    check(receipt['root_adopted'] is True and receipt['owner_commit'] == OWNER
          and receipt['run_id'] == pin['run_id'] and receipt['controller'] == pin['controller']
          and mapping['commit'] == OWNER and mapping['tree'] == pin['source_tree']
          and len(mapping['files']) == pin['source_files'] == 116,
          'accepted runtime/source/owner binding differs')
    check(receipt['actual_native']['sha256'] == policy['runtime']['binaries']['native']['sha256']
          and receipt['carriers']['13070438494']['archive'] == policy['runtime']['zip'],
          'accepted exact runtime body binding differs')
    return dict(owner_files=len(mapping['files']), receipt=identity(world / pin['path']),
                controllers={name: identity(world / name) for name in CONTEXT})


@lru_cache(maxsize=1)
def time_format(world=ROOT):
    tree = ast.parse((Path(world) / 'scripts/rnaseq_qc_performance.py').read_text())
    return next(ast.literal_eval(node.value) for node in tree.body if isinstance(node, ast.Assign)
                and any(isinstance(t, ast.Name) and t.id == 'TIME_FORMAT' for t in node.targets))


def require_recovery(admitted, record):
    if admitted is not True or not isinstance(record, str) or not record.strip():
        raise RuntimeError('independent76GiB recovery is UNADMITTED; root must provide a fresh capacity record')


def hosted_session():
    root = source.hosted_root()
    check(root == Path(read_policy()['runtime']['session']), 'session differs from the exact original runtime prefix')
    for key in CACHE_KEYS:
        path = Path(os.environ[key]).resolve(strict=True)
        check(path.is_relative_to(root) and path.stat().st_dev == root.stat().st_dev,
              'actual tool cache escapes the task session: ' + key)
    return root


def schedule():
    jobs = []
    for lane, workers, timeout in LANES:
        for phase, count in (('warmup', 3), ('trial', 10)):
            for pair in range(1, count + 1):
                engines = ('source', 'native') if pair % 2 else ('native', 'source')
                for engine in engines:
                    jobs.append(dict(id=f'{phase}/{lane}/{pair:02d}/{engine}', lane=lane,
                        phase=phase, pair=pair, engine=engine, workers=workers, timeout=timeout))
    return jobs


def qualification_schedule():
    jobs = [dict(id=f'qualification/{lane}/{engine}', kind='lane', lane=lane,
                 engine=engine, workers=workers, timeout=timeout)
            for lane, workers, timeout in LANES for engine in ('source', 'native')]
    for lane, _, _ in LANES[:4]:
        for producer in ('source', 'native'):
            engine = 'native' if producer == 'source' else 'source'
            jobs.append(dict(id=f'cross-decode/{lane}/{producer}', kind='cross-decode',
                lane=lane, producer=producer, engine=engine, workers=1,
                timeout=45 if 'binary' in lane else 10))
    for extension in ('tbi', 'csi'):
        for producer in ('source', 'native'):
            for engine in ('source', 'native'):
                jobs.append(dict(id=f'cross-index/{extension}/{producer}/{engine}', kind='cross-index',
                    lane='build_' + extension, producer=producer, engine=engine, workers=1, timeout=60))
    return jobs


def command_for(root, programs, row):
    root = Path(root)
    lane, engine, workers = row['lane'], row['engine'], row['workers']
    data, active = root / 'inputs', root / 'active'
    native, bgzip, tabix = map(str, (programs['native'], programs['bgzip'], programs['tabix']))
    vcf, binary = str(data / 'records.vcf'), str(data / 'incompressible.bin')
    compressed = str(data / ('incompressible.bin.bgz' if 'binary' in lane else 'records.vcf.gz'))
    out = active / (row['id'].replace('/', '-') + '.bgz')
    if lane.startswith('compress_'):
        flags = ['--binary'] if 'binary' in lane else []
        if engine == 'native':
            argv = [native, 'bgzip', binary if flags else vcf, *flags,
                    '--output', str(out), '--threads', str(workers), '--compress-level', '6']
        else:
            argv = [bgzip, *flags, '--output', str(out), '--threads', str(workers),
                    '--compress-level', '6', binary if flags else vcf]
        return argv, out
    if lane.startswith('decompress_'):
        return ([native, 'bgzip', '--decompress', compressed] if engine == 'native' else
                [bgzip, '--decompress', '--stdout', '--threads', '1', compressed]), None
    if lane == 'partial_binary_512m':
        gzi = str(data / 'incompressible.bin.bgz.gzi')
        return ([native, 'bgzip', '--decompress', compressed, '--index-input', gzi,
                 '--offset', '536870912', '--size', '536870912'] if engine == 'native' else
                [bgzip, '--decompress', '--stdout', '--index-name', gzi,
                 '--offset', '536870912', '--size', '536870912', compressed]), None
    if lane.startswith('build_'):
        extension = lane.removeprefix('build_')
        flags = ['--csi'] if extension == 'csi' else []
        build = data / (engine + '-build.vcf.gz')
        if engine == 'native':
            out = active / (row['id'].replace('/', '-') + '.' + extension)
            return [native, 'tabix', 'build', '--preset', 'vcf', *flags, '--output', str(out), str(build)], out
        out = Path(str(build) + '.' + extension)
        return [tabix, '--force', '--preset', 'vcf', *flags, str(build)], out
    files = dict(query_sparse_t1=('regions-file', 'regions-sparse.tsv', 'regions'),
                 query_dense_t4=('regions-file', 'regions-dense.tsv', 'regions'),
                 query_overlap_unique_t4=('regions-file', 'regions-overlap.tsv', 'regions'),
                 query_targets_t4=('targets-file', 'targets.bed', 'targets'))
    flag, filename, source_flag = files[lane]
    unique = ['--unique'] if lane == 'query_overlap_unique_t4' else []
    if engine == 'native':
        return [native, 'tabix', 'query', str(data / 'records.vcf.gz'), '--index',
                str(data / 'records.vcf.gz.tbi'), '--' + flag, str(data / filename), *unique,
                *(['--threads', str(workers)] if workers == 4 else [])], None
    return [tabix, '--' + source_flag, str(data / filename), *unique,
            *(['--threads', str(workers - 1)] if workers == 4 else []), str(data / 'records.vcf.gz')], None


def redirect_argv(argv, stdout, file_cap):
    return ['bash', '-c', EXEC_REDIRECT, 'index-bgzf', str((file_cap + 1023) // 1024),
            str(stdout), *map(str, argv)]


def timed_argv(timer, argv, cpus, stdout, usage, file_cap):
    return redirect_argv([str(timer), '-f', time_format(), '-o', str(usage),
                          'taskset', '-c', ','.join(map(str, cpus)), *map(str, argv)], stdout, file_cap)


def parse_usage(path):
    raw = load(path)
    check(set(raw) == {'wall_seconds', 'user_seconds', 'system_seconds', 'max_rss_kib',
        'filesystem_inputs', 'filesystem_outputs', 'major_faults', 'minor_faults', 'exit_status'},
        'GNU time resource fields differ')
    check(all(type(v) in (int, float) and math.isfinite(v) and v >= 0 for v in raw.values())
          and raw['exit_status'] == 0, 'GNU time recorded a failed/invalid target')
    check(all(type(raw[k]) is int for k in ('max_rss_kib', 'filesystem_inputs',
        'filesystem_outputs', 'major_faults', 'minor_faults', 'exit_status')), 'GNU time integer fields differ')
    return dict(wall_seconds=raw['wall_seconds'], user_seconds=raw['user_seconds'],
        system_seconds=raw['system_seconds'], total_cpu_seconds=raw['user_seconds'] + raw['system_seconds'],
        peak_rss_bytes=raw['max_rss_kib'] * 1024, raw_gnu_time=raw,
        filesystem_inputs=raw['filesystem_inputs'], filesystem_outputs=raw['filesystem_outputs'],
        major_faults=raw['major_faults'], minor_faults=raw['minor_faults'])


def distribution(values):
    return dict(values=values, minimum=min(values), median=statistics.median(values), maximum=max(values))


def evaluate(records):
    expected = {row['id']: row for row in schedule()}
    check(len(records) == len(expected) and {r['id'] for r in records} == set(expected),
          'complete338-call graph is required; missing/duplicate calls are HOLD')
    for row in records:
        check(all(row[k] == v for k, v in expected[row['id']].items())
              and row['status'] == 'returned' and row['returncode'] == 0
              and row['group_empty'] is True and row['qualification_passed'] is True,
              'failed/source/leftover/unqualified call cannot earn a performance win')
        check(row['wall_seconds'] > 0 and row['peak_rss_bytes'] > 0,
              'zero or absent scientific resource observation is HOLD')
    lanes = {}
    metrics = ('wall_seconds', 'user_seconds', 'system_seconds', 'total_cpu_seconds',
               'peak_rss_bytes', 'filesystem_inputs', 'filesystem_outputs', 'major_faults', 'minor_faults')
    for lane, _, _ in LANES:
        roles = {}
        for engine in ('source', 'native'):
            values = sorted((r for r in records if r['phase'] == 'trial' and r['lane'] == lane
                             and r['engine'] == engine), key=lambda r: r['pair'])
            roles[engine] = {metric: distribution([r[metric] for r in values]) for metric in metrics}
        native, original = roles['native'], roles['source']
        wall = native['wall_seconds']['median'] / original['wall_seconds']['median']
        rss = native['peak_rss_bytes']['median'] / original['peak_rss_bytes']['median']
        gap = Decimal(str(original['wall_seconds']['minimum'])) - Decimal(str(native['wall_seconds']['maximum']))
        throughput = wall <= .90 and gap > Decimal('0.01')
        memory = rss <= .75 and native['peak_rss_bytes']['maximum'] < original['peak_rss_bytes']['minimum'] and wall <= 1.05
        lanes[lane] = dict(**roles, median_wall_ratio=wall, median_process_rss_ratio=rss,
            paired_wall_ratios=[n / s for n, s in zip(native['wall_seconds']['values'], original['wall_seconds']['values'])],
            wall_disjoint_gap_seconds=str(gap), throughput=throughput, resource=memory,
            material=throughput or memory, wall_loss=wall > 1, rss_loss=rss > 1)
    scaling = {family: {engine: lanes[family + '_t1'][engine]['wall_seconds']['median'] /
               lanes[family + '_t4'][engine]['wall_seconds']['median'] for engine in ('source', 'native')}
               for family in ('compress_text', 'compress_binary')}
    primary = {name: lanes[name]['material'] for name in PRIMARY}
    return dict(lanes=lanes, primary=primary, primary_passed=all(primary.values()), scaling=scaling,
        nonprimary_losses={name: dict(wall=row['median_wall_ratio'], rss=row['median_process_rss_ratio'])
            for name, row in lanes.items() if name not in PRIMARY and (row['wall_loss'] or row['rss_loss'])},
        all_losses={name: dict(wall=row['median_wall_ratio'], rss=row['median_process_rss_ratio'])
            for name, row in lanes.items() if row['wall_loss'] or row['rss_loss']},
        publication=False, aggregate_gate=False, cold_cache_proven=False,
        process_rss_scope='GNU time Linux process maximum, not aggregate or heap; raw KiB retained',
        io_scope='raw GNU time/kernel block-operation counters, not logical application bytes',
        timing_resolution_seconds=.01, outer_monotonic_is_gnu_wall=False,
        final_nonprimary_tradeoff_review_required=True)


def same_bytes(first, second, offset=0):
    with Path(first).open('rb') as a, Path(second).open('rb') as b:
        b.seek(offset)
        while True:
            left, right = a.read(2**20), b.read(2**20)
            if left != right:
                return False
            if not left:
                return True


def owned(root, path):
    root, path = Path(root).resolve(strict=True), Path(path)
    check(not path.is_symlink() and path.resolve(strict=True).is_relative_to(root)
          and path.stat().st_dev == root.stat().st_dev and path.is_file(),
          'body must be a regular task-owned file: ' + str(path))
    return path


def duplicate_record(root, candidate, backup, logical):
    root = Path(root).resolve(strict=True)
    candidate, backup = owned(root, candidate), owned(root, backup)
    check(candidate != backup and candidate.parent.resolve().is_relative_to(root), 'duplicate body aliases its backup')
    check(candidate.stat().st_size == backup.stat().st_size, 'novel/mismatched body retained; HOLD')
    left_hash, right_hash = hashlib.sha256(), hashlib.sha256()
    count = 0
    with candidate.open('rb') as left, backup.open('rb') as right:
        while True:
            a, b = left.read(2**20), right.read(2**20)
            check(a == b, 'novel/mismatched body retained; HOLD')
            if not a:
                break
            count += len(a)
            left_hash.update(a)
            right_hash.update(b)
    observed = dict(bytes=count, sha256=left_hash.hexdigest())
    preserved = dict(bytes=count, sha256=right_hash.hexdigest())
    check(observed == preserved, 'complete body hash differs; HOLD')
    if backup.parent == root / 'cas':
        check(backup.name == preserved['sha256'], 'immutable CAS backup body differs; HOLD')
    return dict(logical=logical, path=candidate.relative_to(root).as_posix(),
        mode=stat.S_IMODE(candidate.stat().st_mode), identity=observed,
        backup=backup.relative_to(root).as_posix(), backup_identity=preserved,
        full_byte_comparison=True, duplicate_removed=False)


def retain_body(root, path, logical, backup=None):
    root, path = Path(root).resolve(strict=True), owned(root, path)
    if backup is None:
        observed = identity(path)
        backup = root / 'cas' / observed['sha256']
        backup.parent.mkdir(exist_ok=True)
        if not backup.exists():
            mode = stat.S_IMODE(path.stat().st_mode)
            path.rename(backup)
            backup.chmod(0o444)
            record = dict(logical=logical, path=path.relative_to(root).as_posix(), mode=mode,
                identity=observed, backup=backup.relative_to(root).as_posix(), backup_identity=observed,
                full_byte_comparison=None, duplicate_removed=False, disposition='first-unique-body-moved-to-CAS')
            save(root / 'metadata/custody' / (logical + '.json'), record)
            return backup
    record = duplicate_record(root, path, backup, logical)
    check(hosted_session() == root and platform.system() == 'Linux',
          'actual duplicate retirement is hosted Linux only')
    check(path.is_relative_to(root / 'active') or path in (
        root / 'inputs/source-build.vcf.gz.tbi', root / 'inputs/source-build.vcf.gz.csi'),
        'only the task-owned transient output may be retired')
    save(root / 'metadata/custody' / (logical + '.json'), record)
    path.unlink()
    record.update(duplicate_removed=True, disposition='exact-backed-duplicate')
    save(root / 'metadata/custody' / (logical + '.json'), record)
    return Path(backup)


def file_stat(path):
    s = Path(path).stat()
    return dict(device=s.st_dev, inode=s.st_ino, bytes=s.st_size,
                mode=stat.S_IMODE(s.st_mode), mtime_ns=s.st_mtime_ns, ctime_ns=s.st_ctime_ns)


def freeze_files(paths):
    return {str(path): dict(stat=file_stat(path), identity=identity(path)) for path in paths}


def check_frozen(frozen, hash_bodies=False):
    for name, row in frozen.items():
        check(file_stat(name) == row['stat'], 'frozen input/runtime stat changed: ' + name)
        if hash_bodies:
            check(identity(name) == row['identity'], 'frozen input/runtime body changed: ' + name)


def capacity_decision(used, free_bytes):
    if any(used[k] > limit for k, limit in CATEGORIES.items()):
        raise RuntimeError('capacity: frozen retained category exceeded; retain failure and stop')
    remaining = sum(CATEGORIES.values()) - sum(used.values())
    if free_bytes < remaining + 8 * 2**30:
        raise RuntimeError('capacity: complete remaining32GiB graph plus independent8GiB headroom does not fit')
    return dict(used=used, free_bytes=free_bytes, remaining_reservation=remaining,
                independent_headroom_bytes=8 * 2**30, retained_cap_bytes=sum(CATEGORIES.values()))


def capacity(root, label):
    used = {k: 0 for k in CATEGORIES}
    for path in Path(root).rglob('*'):
        if not path.is_file() or path.is_symlink():
            continue
        relative = path.relative_to(root)
        first = relative.parts[0]
        category = first if first in CATEGORIES else 'metadata'
        if first == 'inputs' and path.name in ('source-build.vcf.gz.tbi', 'source-build.vcf.gz.csi'):
            category = 'active'
        used[category] += path.stat().st_size
    free = shutil.disk_usage(root).free
    save(Path(root) / 'metadata/capacity' / (label + '.json'), dict(used=used, free_bytes=free, categories=CATEGORIES))
    return capacity_decision(used, free)


def reserve_time(started, now, remaining):
    if now - started + remaining > 18600:
        raise RuntimeError('time: complete remaining fixed graph does not fit; no deadline/data reduction')


def capture_child(root, name, argv, timeout):
    directory = Path(root) / 'calls' / name
    directory.mkdir(parents=True)
    save(directory / 'request.json', dict(argv=list(map(str, argv)), cwd=os.getcwd(), timeout=timeout,
         environment={key: os.environ.get(key) for key in CACHE_KEYS +
             ('PATH', 'LD_LIBRARY_PATH', 'INDEX_CONTAINER_DIGEST', 'GCC_VERSION', 'GITHUB_SHA')},
         started_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())))
    child = run_child(list(map(str, argv)), directory / 'process', timeout=timeout, cleanup_timeout=5)
    return dict(status='returned', returncode=child.returncode, group_empty=child.group_empty,
                owner_monotonic_seconds=(child.end_ns - child.start_ns) / 1e9,
                raw_process=str((directory / 'process.process.json').relative_to(root)))


def validate_workflow(path):
    text = Path(path).read_text()
    event = re.search(r'^on:\n(.*?)(?=^\S)', text, re.M | re.S)
    check(event is not None and event.group(1).lstrip().startswith('workflow_dispatch:')
          and not re.search(r'^  (?:push|pull_request|schedule|workflow_run):', event.group(1), re.M),
          'BGZF workflow must be manual only')
    check(all(value in text for value in ('gcc:14.3.0-bookworm@' + source.IMAGE,
        '--cpus 4 --memory 8g --memory-swap 8g', 'timeout-minutes: 360', 'default: false',
        '${{ env.INDEX_SESSION }}', 'actions: read', 'retention-days: 90', 'if: always()')),
        'frozen workflow runtime/recovery/retention routes differ')


def restore_selected(carrier, destination, selected):
    destination = Path(destination).resolve(strict=True)
    with tarfile.open(carrier, 'r:') as archive:
        members = archive.getmembers()
        names = [m.name.rstrip('/') for m in members]
        check(len(names) == len(set(names)), 'duplicate runtime TAR member')
        for member in members:
            path = PurePosixPath(member.name)
            check(not path.is_absolute() and '..' not in path.parts and path.parts[0] == 'runtime',
                  'unsafe runtime TAR member')
            if member.issym():
                normalized = posixpath.normpath(posixpath.join(str(path.parent), member.linkname))
                check(not member.linkname.startswith('/') and normalized.startswith('runtime/'), 'runtime symlink escapes')
            elif member.islnk():
                check(member.linkname in names and not member.linkname.startswith('/')
                      and '..' not in PurePosixPath(member.linkname).parts, 'runtime hardlink escapes')
            else:
                check(member.isdir() or member.isfile(), 'special runtime TAR member')
        links = {m.name.rstrip('/') for m in members if m.issym()}
        for name, pin in selected.items():
            member = archive.getmember(name)
            path = PurePosixPath(name)
            check(not any('/'.join(path.parts[:n]) in links for n in range(1, len(path.parts))),
                  'selected runtime member traverses a symlink')
            output = destination / name
            output.parent.mkdir(parents=True, exist_ok=True)
            if member.isdir():
                output.mkdir(exist_ok=True)
            elif member.issym():
                output.symlink_to(member.linkname)
            else:
                with archive.extractfile(member) as body, output.open('xb') as stream:
                    shutil.copyfileobj(body, stream, 2**20)
                output.chmod(member.mode & 0o777)
                os.utime(output, (member.mtime, member.mtime))
                if pin is not None:
                    matches(output, pin)


def api(root, route, name):
    output = Path(root) / 'metadata/acquisition' / (name + '.json')
    request = urllib.request.Request('https://api.github.com/repos/omics-rust/' + route,
        headers={'Authorization': 'Bearer ' + os.environ['GH_TOKEN'],
                 'Accept': 'application/vnd.github+json', 'User-Agent': 'rsomics-index-bgzf-gate'})
    with urllib.request.urlopen(request, timeout=60) as response:
        body = response.read(4 * 2**20 + 1)
    check(len(body) <= 4 * 2**20, 'bounded GitHub metadata response exceeded')
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open('xb') as stream:
        stream.write(body)
    return load(output)


class ArtifactRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, message, headers, newurl):
        return None


def acquire_carrier(root, policy):
    pin, reference = policy['runtime'], policy['accepted_reference']
    run = api(root, f"rsomics-world/actions/runs/{reference['run_id']}/attempts/{reference['run_attempt']}", 'reference-run')
    check((run['id'], run['run_attempt'], run['head_sha'], run['path'], run['status'], run['conclusion']) ==
          (reference['run_id'], reference['run_attempt'], reference['controller'], reference['workflow'], 'completed', 'success'),
          'accepted exact reference runtime run is no longer available/matching')
    item = api(root, f"rsomics-world/actions/artifacts/{pin['artifact_id']}", 'runtime-artifact')
    check(item['id'] == pin['artifact_id'] and item['name'] == pin['artifact_name']
          and item['expired'] is False and item['size_in_bytes'] == pin['zip']['bytes']
          and item['digest'] == 'sha256:' + pin['zip']['sha256']
          and item['workflow_run']['id'] == reference['run_id']
          and item['workflow_run']['head_sha'] == reference['controller'], 'runtime artifact metadata differs')
    request = urllib.request.Request(f"https://api.github.com/repos/omics-rust/rsomics-world/actions/artifacts/{pin['artifact_id']}/zip",
        headers={'Authorization': 'Bearer ' + os.environ['GH_TOKEN'], 'Accept': 'application/vnd.github+json'})
    try:
        urllib.request.build_opener(ArtifactRedirect).open(request, timeout=60)
    except urllib.error.HTTPError as error:
        check(error.code == 302, 'runtime artifact redirect failed')
        location = error.headers['Location']
    else:
        raise ValueError('expected authenticated artifact redirect is absent')
    parsed = urllib.parse.urlsplit(location)
    check(parsed.scheme == 'https' and parsed.hostname and any(parsed.hostname.endswith(suffix)
        for suffix in ('.blob.core.windows.net', '.actions.githubusercontent.com', '.githubusercontent.com')),
        'unadmitted runtime carrier destination')
    directory = Path(root) / 'runtime/carriers'
    directory.mkdir(parents=True)
    archive = directory / 'original-runtime.zip'
    count = 0
    with urllib.request.urlopen(location, timeout=120) as response, archive.open('xb') as output:
        for block in iter(lambda: response.read(2**20), b''):
            count += len(block)
            check(count <= pin['zip']['bytes'], 'runtime carrier exceeds exact original size')
            output.write(block)
    matches(archive, pin['zip'])
    carrier = directory / 'runtime.tar'
    with zipfile.ZipFile(archive) as zipped:
        check(zipped.namelist() == ['runtime.tar'] and zipped.getinfo('runtime.tar').file_size == pin['tar']['bytes']
              and not zipped.getinfo('runtime.tar').flag_bits & 1, 'original runtime ZIP layout differs')
        with zipped.open('runtime.tar') as body, carrier.open('xb') as output:
            shutil.copyfileobj(body, output, 2**20)
    matches(carrier, pin['tar'])
    save(directory / 'carrier.json', dict(artifact=pin['artifact_id'], zip=pin['zip'], tar=pin['tar'],
         complete_zip_crc_read=True, token_or_presigned_query_retained=False))
    return carrier


def restore_runtime(root, carrier, policy, receipt):
    pin = policy['runtime']
    selected = {row['path']: row for row in pin['binaries'].values()}
    selected.update(pin['anchors'])
    mapping = receipt['owner_source_git_mapping']['files']
    selected.update({'runtime/native/owner/' + name: row for name, row in mapping.items()})
    selected['runtime/native/admission.json'] = None
    for name, spec in source.SOURCES.items():
        if name in ('htslib', 'libdeflate'):
            selected['runtime/downloads/' + spec['filename']] = dict(bytes=spec['size'], sha256=spec['sha256'])
    with tarfile.open(carrier, 'r:') as archive:
        check(len(archive.getmembers()) == pin['members'], 'complete accepted runtime TAR membership differs')
        selected.update({m.name.rstrip('/'): None for m in archive.getmembers()
            if m.name.startswith('runtime/prefix/') or m.name in (
                'runtime/htslib-original/htslib-1.24/LICENSE',
                'runtime/libdeflate-original/libdeflate-1.25/COPYING')})
    restore_selected(carrier, root, selected)
    for name, expected in mapping.items():
        path = root / 'runtime/native/owner' / name
        body = path.read_bytes()
        blob = hashlib.sha1(b'blob ' + str(len(body)).encode() + b'\0' + body).hexdigest()
        check(blob == expected['git_blob'] and ('100755' if path.stat().st_mode & 0o111 else '100644') == expected['git_mode'],
              'retained current-owner Git body/mode differs: ' + name)
    check(sha256(root / 'runtime/native/owner/benchmarks/index-vs-htslib.sh') ==
          policy['fixture']['historical_harness_sha256'], 'literal original harness changed')
    matches(root / pin['libdeflate']['path'], pin['libdeflate'])
    save(root / 'metadata/restored-runtime.json', dict(selected=source.tree_files(root / 'runtime',
         ('carriers/original-runtime.zip', 'carriers/runtime.tar')),
         source_files=len(mapping), provider_graph=pin['provider_graph'], whole_original_zip_tar_retained=True))
    return {name: root / row['path'] for name, row in pin['binaries'].items()}


def command_text(root, name, argv, timeout=30):
    capture_child(root, name, argv, timeout)
    return (Path(root) / 'calls' / name / 'process.stdout').read_text()


def prepare_timer(root, policy):
    spec = policy['timer']
    directory = Path(root) / 'runtime/timer'
    directory.mkdir()
    bodies = {}
    for name in ('deb', 'dsc', 'orig', 'debian', 'copyright'):
        pin = spec[name]
        path = directory / urllib.parse.urlsplit(pin['url']).path.rsplit('/', 1)[-1]
        download(pin['url'], path, pin['bytes'])
        matches(path, pin)
        bodies[name] = path
    fields = command_text(root, 'timer-package-fields', ['dpkg-deb', '--field', bodies['deb'], 'Package', 'Version', 'Architecture'])
    check(fields.splitlines() == ['Package: time', 'Version: 1.9-0.2', 'Architecture: amd64'],
          'exact timer package control identity differs')
    command_text(root, 'timer-package-extract', ['dpkg-deb', '--extract', bodies['deb'], directory / 'prefix'])
    original = source.extract_release(bodies['orig'], directory / 'upstream-source',
        dict(size=spec['orig']['bytes'], sha256=spec['orig']['sha256'], prefix='time-1.9', unpacked_limit=16 * 2**20))
    packaging = source.extract_release(bodies['debian'], directory / 'packaging-source',
        dict(size=spec['debian']['bytes'], sha256=spec['debian']['sha256'], prefix='debian', unpacked_limit=2 * 2**20))
    matches(packaging / 'copyright', spec['copyright'])
    matches(directory / 'prefix/usr/share/doc/time/copyright', spec['copyright'])
    licenses = directory / 'licenses'
    licenses.mkdir()
    for name in ('GPL-2', 'GPL-3'):
        shutil.copy2(Path('/usr/share/common-licenses') / name, licenses / name)
    binary = directory / 'prefix/usr/bin/time'
    banner = command_text(root, 'timer-version', [binary, '--version']).splitlines()[0]
    check(banner in ('time (GNU Time) 1.9', 'time (GNU Time) UNKNOWN'), 'actual GNU Time banner differs')
    save(directory / 'source-and-version.json', dict(package='time', package_version='1.9-0.2',
         architecture='amd64', bodies={name: identity(path) for name, path in bodies.items()},
         executable=identity(binary), actual_banner=banner, upstream_version_observed='UNKNOWN' if banner.endswith('UNKNOWN') else '1.9',
         unknown_is_not_observed_1_9=True, source=str(original.relative_to(root)), packaging=str(packaging.relative_to(root)),
         licenses=source.tree_files(licenses), signature_verified=False, release_chain_verified=False,
         role='standalone benchmark recorder only; not linked/copied into Rust products'))
    return binary


def admit_runtime(root, programs, policy, receipt, cpus):
    source.admit_image(root)
    check(command_text(root, 'compiler-version', ['/usr/local/bin/gcc', '-dumpfullversion', '-dumpversion']).strip() == '14.3.0',
          'actual pinned-image compiler version differs')
    for name, argv in {'packages': ['dpkg-query', '-W', '-f=${Package}\t${Version}\n'],
        'openssl-version': ['openssl', 'version', '-a'], 'taskset-version': ['taskset', '--version'],
        'awk-version': ['awk', '-W', 'version'], 'dpkg-deb-version': ['dpkg-deb', '--version']}.items():
        command_text(root, name, argv)
    for name, first in (('native', 'rsomics-index 0.1.0'), ('bgzip', 'bgzip (htslib) 1.24'), ('tabix', 'tabix (htslib) 1.24')):
        check(command_text(root, name + '-version', [programs[name], '--version']).splitlines()[0] == first,
              'actual restored tool version differs: ' + name)
    provider = (Path(root) / policy['runtime']['libdeflate']['path']).resolve(strict=True)
    elf = source.elf_admission(root, programs, provider)
    for name in ('bgzip', 'tabix'):
        check(str(provider) in elf[name]['libraries'], 'actual source tool did not load exact shared libdeflate: ' + name)
    for name in ('native', 'bgzip'):
        wanted = receipt['actual_native'] if name == 'native' else receipt['original_runtime_executables_and_libraries']['bgzip']
        observed = {path: {k: row[k] for k in ('bytes', 'sha256')} for path, row in elf[name]['libraries'].items()}
        expected = {path: {k: row[k] for k in ('bytes', 'sha256')} for path, row in wanted['libraries'].items()}
        check(observed == expected, 'fresh loaded-provider bodies differ from accepted runtime: ' + name)
    usage = Path(root) / 'metadata/timer-small-child.json'
    capture_child(root, 'timer-small-child', timed_argv(programs['timer'], ['/bin/true'], cpus[:1],
                  '/dev/null', usage, 2**20), timeout=10)
    floor = parse_usage(usage)
    check(floor['peak_rss_bytes'] < policy['timer']['small_child_max_rss_exclusive_bytes'],
          'actual small-C-child RSS floor can obscure the3MiB scientific process range')
    save(Path(root) / 'metadata/runtime-admission.json', dict(elf=elf, small_child=floor,
        controller_process_peak_rss_bytes=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss * 1024,
        floor_scope='actual GNU time C child; no Python between recorder and target',
        time_format=time_format(), providers=policy['runtime']['provider_graph'],
        tabix_freshly_admitted=True, timer_freshly_admitted=True))


def admit_owner(root, policy):
    run = api(root, f"rsomics-index/actions/runs/{policy['owner_ci']}", 'owner-native-four')
    jobs = api(root, f"rsomics-index/actions/runs/{policy['owner_ci']}/jobs?per_page=100", 'owner-native-four-jobs')
    check(run['head_sha'] == OWNER and run['status'] == 'completed' and run['conclusion'] == 'success'
          and run['repository']['full_name'] == policy['owner_repository'], 'fresh exact owner native CI differs')
    names = {'linux-x86_64', 'linux-aarch64', 'macos-x86_64', 'macos-aarch64'}
    selected = [r for r in jobs['jobs'] if r['name'] in names]
    check(len(selected) == 4 and {r['name'] for r in selected} == names
          and all(r['status'] == 'completed' and r['conclusion'] == 'success' for r in selected),
          'fresh actual four-native owner CI differs')


def generate_fixture(root, programs, policy, cpus):
    root = Path(root)
    data = root / 'inputs'
    data.mkdir()
    fixture = policy['fixture']
    vcf = data / 'records.vcf'
    argv = ['awk', '-v', 'records=6000000', '-v', 'per=250000', '-v', 'filler=' + 'ACGT' * 80, VCF_AWK]
    capture_child(root, 'generate-vcf', redirect_argv(argv, vcf, 3 * 2**30), timeout=300)
    matches(vcf, fixture['vcf'])
    binary = data / 'incompressible.bin'
    script = 'set -euo pipefail; dd if=/dev/zero bs=1048576 count=2048 2>/dev/null | openssl enc -aes-256-ctr -nosalt -K "$1" -iv "$2"'
    capture_child(root, 'generate-aes-ctr', redirect_argv(['bash', '-c', script, 'synthetic-aes',
        fixture['aes_key'], fixture['aes_iv']], binary, fixture['binary']['bytes']), timeout=300)
    matches(binary, fixture['binary'])
    partial = data / 'expected-slice.bin'
    with binary.open('rb') as original, partial.open('xb') as output:
        original.seek(fixture['slice']['offset'])
        remaining = fixture['slice']['bytes']
        while remaining:
            block = original.read(min(2**20, remaining))
            check(bool(block), 'complete expected indexed slice is absent')
            output.write(block)
            remaining -= len(block)
    matches(partial, fixture['slice'])
    sparse = ''.join(f'chr{(i * 600 // 250000) + 1}\t{i * 600 % 250000 + 1}\t{i * 600 % 250000 + 1}\n' for i in range(10000))
    dense = ''.join(f'chr{c}\t1\t25000\n' for c in range(1, 25))
    overlap = ''.join(f'chr{c}\t{1 + r * 10000}\t{50000 + r * 10000}\n' for c in range(1, 25) for r in range(8))
    targets = ''.join(f'chr{c}\t83332\t108333\n' for c in range(1, 25))
    for name, body in (('regions-sparse.tsv', sparse), ('regions-dense.tsv', dense),
                       ('regions-overlap.tsv', overlap), ('targets.bed', targets)):
        with (data / name).open('x') as stream:
            stream.write(body)
    canonical = [
        ('canonical-text', [programs['bgzip'], '--force', '--threads', '4', '--compress-level', '6', '--output', data / 'records.vcf.gz', vcf]),
        ('canonical-tbi', [programs['tabix'], '--force', '--preset', 'vcf', data / 'records.vcf.gz']),
        ('canonical-csi', [programs['tabix'], '--force', '--preset', 'vcf', '--csi', data / 'records.vcf.gz']),
        ('canonical-binary', [programs['bgzip'], '--force', '--binary', '--threads', '4', '--compress-level', '6', '--index',
            '--index-name', data / 'incompressible.bin.bgz.gzi', '--output', data / 'incompressible.bin.bgz', binary])]
    save(root / 'canonical-graph.json', [dict(name=name, argv=list(map(str, argv))) for name, argv in canonical])
    for name, argv in canonical:
        capture_child(root, name, redirect_argv(['taskset', '-c', ','.join(map(str, cpus)), *map(str, argv)],
                      '/dev/null', 3 * 2**30), timeout=180)
        capacity(root, name)
    for name, limit in (('records.vcf.gz', policy['limits']['text_bgzf_bytes']),
        ('incompressible.bin.bgz', policy['limits']['binary_bgzf_bytes']),
        ('records.vcf.gz.tbi', policy['limits']['index_bytes']), ('records.vcf.gz.csi', policy['limits']['index_bytes']),
        ('incompressible.bin.bgz.gzi', policy['limits']['index_bytes'])):
        check(0 < (data / name).stat().st_size <= limit, 'canonical complete output exceeds frozen size: ' + name)
    for engine in ('source', 'native'):
        shutil.copy2(data / 'records.vcf.gz', data / (engine + '-build.vcf.gz'))
    for path in data.iterdir():
        path.chmod(0o444)
    frozen = freeze_files(sorted(data.iterdir()))
    save(root / 'frozen-inputs.json', frozen)
    save(root / 'metadata/generation.json', dict(generator='literal retained awk and fixed public AES256-CTR parameters',
         input_identities=frozen, requests=dict(sparse=10000, dense=24, overlap_unique=192, targets=24),
         canonical_calls=4, canonical_encodings_fresh_linux=True, old_mac_compressed_hashes_required=False))
    return frozen


def output_limit(row, policy):
    lane = row['lane']
    if row.get('kind') in ('cross-index', 'cross-decode'):
        return policy['limits']['read_output_bytes']
    if lane.startswith('compress_'):
        return policy['limits']['binary_bgzf_bytes' if 'binary' in lane else 'text_bgzf_bytes']
    return policy['limits']['index_bytes'] if lane.startswith('build_') else policy['limits']['read_output_bytes']


def row_count(path):
    with Path(path).open('rb') as body:
        return sum(block.count(b'\n') for block in iter(lambda: body.read(2**20), b''))


def arm_deadline(deadline):
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        raise TimeoutError('fixed controller/phase deadline reached')
    signal.setitimer(signal.ITIMER_REAL, remaining)


def deadline_expired(_signum, _frame):
    raise TimeoutError('fixed Index BGZF controller/phase deadline reached')


@contextmanager
def comparisons(state):
    started = time.monotonic()
    remaining = 1200 - state['comparison_spent']
    arm_deadline(min(state['deadline'], started + remaining))
    try:
        yield
    finally:
        state['comparison_spent'] += time.monotonic() - started
        arm_deadline(state['deadline'])
        if state['comparison_spent'] > 1200:
            raise TimeoutError('fixed1200s full-comparison budget exceeded')


def science_call(root, programs, row, argv, output, cpus, policy, measured=False):
    root = Path(root)
    if output is not None:
        check(not os.path.lexists(output), 'refusing to overwrite an existing scientific output: ' + str(output))
    stdout = '/dev/null' if measured else str(root / 'active' / (row['id'].replace('/', '-') + '.stdout'))
    usage = root / 'calls' / row['id'] / 'gnu-time.json'
    chosen = cpus[:row['workers']]
    wrapper = (timed_argv(programs['timer'], argv, chosen, stdout, usage, output_limit(row, policy)) if measured else
               redirect_argv(['taskset', '-c', ','.join(map(str, chosen)), *map(str, argv)], stdout, output_limit(row, policy)))
    save(root / 'metadata/requests' / (row['id'] + '.json'), dict(**row, argv=list(map(str, argv)),
        output_path=str(output) if output is not None else stdout, requested_cpus=chosen,
        stdout_profile='literal old /dev/null timed-read profile' if measured else 'complete qualification body',
        outer_wrapper=wrapper, file_size_cap=output_limit(row, policy)))
    try:
        observation = capture_child(root, row['id'], wrapper, row['timeout'])
        if measured:
            observation.update(parse_usage(usage))
        else:
            if output is not None:
                check(Path(stdout).stat().st_size == 0, 'file-producing tool emitted unexpected qualification stdout')
                retain_body(root, Path(stdout), row['id'] + '/empty-stdout')
        return observation, output if output is not None else (None if measured else Path(stdout))
    except BaseException:
        save(root / 'metadata/failed-node.json', dict(**row, argv=list(map(str, argv)),
             exception=traceback.format_exc(), stdout=stdout, output=str(output) if output is not None else None,
             failures_and_novel_bodies_retained=True, performance_accepted=False))
        raise


def qualification(root, programs, policy, cpus, frozen, state):
    root = Path(root)
    qualified, records = {}, []
    remaining = 1750 + 13650
    full_data = None
    for row in qualification_schedule():
        reserve_time(state['started'], time.monotonic(), remaining + 1200 - state['comparison_spent'])
        capacity(root, row['id'].replace('/', '-'))
        check_frozen(frozen)
        transient_index = None
        if row['kind'] == 'lane':
            argv, output = command_for(root, programs, row)
        elif row['kind'] == 'cross-decode':
            compressed = qualified[row['producer'] + '/' + row['lane']]
            argv = ([str(programs['native']), 'bgzip', '--decompress', str(compressed)] if row['engine'] == 'native' else
                    [str(programs['bgzip']), '--decompress', '--stdout', '--threads', '1', str(compressed)])
            output = None
        else:
            index = qualified[row['producer'] + '/' + row['lane']]
            data = root / 'inputs/records.vcf.gz'
            regions = [f'chr{c}:1-250000' for c in range(1, 25)]
            if row['engine'] == 'native':
                argv = [str(programs['native']), 'tabix', 'query', str(data), '--index', str(index), *regions]
            else:
                data = root / 'inputs/source-build.vcf.gz'
                transient_index = Path(str(data) + '.' + row['lane'].removeprefix('build_'))
                check(not transient_index.exists(), 'cross-read index destination already exists')
                shutil.copy2(index, transient_index)
                argv = [str(programs['tabix']), '--verbosity', '0', str(data), *regions]
            output = None
        observation, body = science_call(root, programs, row, argv, output, cpus, policy)
        check(0 <= body.stat().st_size <= output_limit(row, policy), 'qualification output exceeds frozen bound')
        with comparisons(state):
            backup, rows = None, None
            if row['kind'] == 'cross-index':
                check(body.stat().st_size == policy['fixture']['vcf_data_bytes']
                      and same_bytes(body, root / 'inputs/records.vcf', policy['fixture']['vcf_header_bytes']),
                      'full24-contig producer/reader cross-index body differs from literal VCF data')
                rows = row_count(body)
                check(rows == 6000000, 'full cross-index ordered record count differs')
                backup = full_data
            elif row['kind'] == 'cross-decode' or row['lane'].startswith('decompress_'):
                backup = root / 'inputs' / ('incompressible.bin' if 'binary' in row['lane'] else 'records.vcf')
            elif row['lane'] == 'partial_binary_512m':
                backup = root / 'inputs/expected-slice.bin'
            elif row['lane'].startswith('query_'):
                rows = row_count(body)
                check(rows == policy['fixture']['query_rows'][row['lane']], 'qualification ordered query record count differs')
                if row['engine'] == 'native':
                    backup = qualified['source/' + row['lane']]
            backing = retain_body(root, body, row['id'], backup)
            if row['kind'] == 'lane':
                qualified[row['engine'] + '/' + row['lane']] = backing
            elif row['kind'] == 'cross-index':
                full_data = backing
            if transient_index is not None:
                retain_body(root, transient_index, row['id'] + '/index-view', index)
            custody = load(root / 'metadata/custody' / (row['id'] + '.json'))
            result = dict(**row, **observation, complete_output=dict(backup=backing.relative_to(root).as_posix(),
                **custody['backup_identity']), ordered_records=rows, complete_body_qualified=True)
            records.append(result)
            save(root / 'qualification-records.json', records)
        remaining -= row['timeout']
        capacity(root, 'after-' + row['id'].replace('/', '-'))
    with comparisons(state):
        a, b = qualified['source/build_tbi'], qualified['native/build_tbi']
        payloads = []
        for path in (a, b):
            with gzip.open(path, 'rb') as stream:
                payload = stream.read(64 * 2**20 + 1)
            check(len(payload) <= 64 * 2**20 and payload.startswith(b'TBI\1'), 'complete TBI payload diagnostic differs')
            payloads.append(payload)
        check(payloads[0] == payloads[1], 'native/source complete decompressed TBI payload differs')
        save(root / 'metadata/index-diagnostics.json', dict(tbi_raw_byte_identical=same_bytes(a, b),
             tbi_payload_equal=True, tbi_payload_bytes=len(payloads[0]),
             tbi_payload_sha256=hashlib.sha256(payloads[0]).hexdigest(),
             csi_serialization_identity_required=False, full_tbi_csi_cross_reader_calls=8,
             typed_compact_oracle_evidence=dict(owner=OWNER, native_four=policy['owner_ci'],
                 tests='all current owner116 Git bodies retained, including tests/tabix_compat.rs'),
             no_new_biological_or_index_parser=True))
        check_frozen(frozen, hash_bodies=True)
    check(len(records) == 42 and all(r['complete_body_qualified'] for r in records), 'complete42 qualifications are required')
    save(root / 'qualified-outputs.json', {k: str(v.relative_to(root)) for k, v in qualified.items()})
    return qualified


def benchmark(root, programs, policy, cpus, qualified, frozen, state):
    root = Path(root)
    records = []
    remaining = 13650
    for row in schedule():
        reserve_time(state['started'], time.monotonic(), remaining + 1200 - state['comparison_spent'])
        check_frozen(frozen)
        capacity(root, row['id'].replace('/', '-'))
        argv, output = command_for(root, programs, row)
        observation, body = science_call(root, programs, row, argv, output, cpus, policy, measured=True)
        record = dict(**row, **observation, qualification_passed=True,
            timed_read_stdout_discarded=output is None, per_trial_read_body_retained=False,
            output_contract='prior complete qualification only; stdout discarded' if output is None else 'complete own-qualified-body equality')
        if body is not None:
            check(body.stat().st_size <= output_limit(row, policy), 'repeat output exceeds frozen bound; retained HOLD')
            with comparisons(state):
                backing = retain_body(root, body, row['id'], qualified[row['engine'] + '/' + row['lane']])
                custody = load(root / 'metadata/custody' / (row['id'] + '.json'))
                record['complete_output'] = dict(backup=backing.relative_to(root).as_posix(), **custody['backup_identity'])
        records.append(record)
        save(root / 'measurements.json', records)
        remaining -= row['timeout']
        capacity(root, 'after-' + row['id'].replace('/', '-'))
    with comparisons(state):
        check_frozen(frozen, hash_bodies=True)
    return records


def initialize():
    if (os.environ.get('GITHUB_ACTIONS') != 'true' or platform.system() != 'Linux'
            or platform.machine() != 'x86_64' or os.environ.get('GITHUB_EVENT_NAME') != 'workflow_dispatch'):
        raise RuntimeError('real manual hosted Linux x86_64 initialization required')
    policy = read_policy()
    record = os.environ.get('INDEX_RECOVERY_RECORD', '')
    require_recovery(os.environ.get('INDEX_RECOVERY_ADMITTED') == 'true', record)
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    root = Path(policy['runtime']['session'])
    check(root.parent == runner and root.parent.stat().st_dev == runner.stat().st_dev,
          'original absolute runtime prefix does not belong to actual RUNNER_TEMP')
    check(shutil.disk_usage(runner).free >= policy['limits']['initial_free_bytes'],
          'capacity: fresh41GiB after image is required before acquisition')
    root.mkdir(mode=0o700)
    (root / 'active').mkdir()
    (root / 'cas').mkdir()
    values = dict(INDEX_SESSION=str(root))
    for key in CACHE_KEYS:
        path = root / 'cache' / key
        path.mkdir(parents=True)
        values[key] = str(path)
    with Path(os.environ['GITHUB_ENV']).open('a') as stream:
        for key, value in values.items():
            stream.write(f'{key}={value}\n')
    save(root / 'request.json', dict(schema=policy['schema'], controller=os.environ['GITHUB_SHA'], owner=OWNER,
         run_id=os.environ['GITHUB_RUN_ID'], run_attempt=os.environ['GITHUB_RUN_ATTEMPT'],
         workflow_ref=os.environ.get('GITHUB_WORKFLOW_REF'), recovery_manually_admitted=True,
         recovery_record=record, recovery_bytes=76 * 2**30, historical_13_lane_hold_reclassified=False,
         scientific_calls=380, canonical_setup_calls=4, publication=False))


def machine(root, policy):
    groups = {name: (Path('/sys/fs/cgroup') / name).read_text().strip()
              for name in ('cpu.max', 'memory.max', 'memory.current', 'memory.peak', 'memory.swap.max')}
    quota, period = groups['cpu.max'].split()
    check(quota != 'max' and int(quota) == 4 * int(period) and int(groups['memory.max']) == 8 * 2**30
          and int(groups['memory.swap.max']) == 0, 'actual fourCPU/eightGiB/no-extra-swap cgroup differs')
    check(resource.getrlimit(resource.RLIMIT_AS) == (resource.RLIM_INFINITY, resource.RLIM_INFINITY),
          'RLIMIT_AS is not an allowed execution substitute')
    cpus = sorted(os.sched_getaffinity(0))[:4]
    check(len(cpus) == 4, 'four eligible logical CPUs are required')
    save(Path(root) / 'machine.json', dict(platform=platform.platform(), python=sys.version,
        python_executable=identity(Path(sys.executable).resolve()), cgroup=groups,
        cpuinfo=Path('/proc/cpuinfo').read_text(), host_visible_meminfo=Path('/proc/meminfo').read_text(),
        host_visible_memory_is_not_cgroup_entitlement=True, load_average=os.getloadavg(),
        affinity=sorted(os.sched_getaffinity(0)), selected_cpus=cpus,
        disk=shutil.disk_usage(root)._asdict(), device=Path(root).stat().st_dev,
        resource_limits={name: resource.getrlimit(getattr(resource, name))
            for name in ('RLIMIT_AS', 'RLIMIT_CORE', 'RLIMIT_FSIZE')},
        cache='Uncontrolled; preparation and complete hashing precede timings. No cold-cache or exclusive-host claim.',
        environment={key: os.environ.get(key) for key in CACHE_KEYS +
            ('INDEX_CONTAINER_DIGEST', 'GCC_VERSION', 'PATH', 'GITHUB_SHA', 'RUNNER_TEMP')}))
    return cpus


def execute():
    started = time.monotonic()
    state = dict(started=started, deadline=started + 18600, comparison_spent=0.)
    signal.signal(signal.SIGALRM, deadline_expired)
    arm_deadline(started + 1800)
    root = hosted_session()
    policy = read_policy()
    require_recovery(load(root / 'request.json')['recovery_manually_admitted'],
                     load(root / 'request.json')['recovery_record'])
    validate_workflow(ROOT / WORKFLOW)
    context = verify_context(ROOT, policy)
    check(command_text(root, 'controller-head', ['git', '-c', 'safe.directory=' + str(ROOT), '-C', ROOT,
          'rev-parse', 'HEAD']).strip() == os.environ['GITHUB_SHA'], 'actual controller checkout differs from dispatch')
    for name in (*CONTEXT, policy['accepted_reference']['path']):
        destination = root / 'controller' / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(ROOT / name, destination)
    save(root / 'controller-bindings.json', context)
    save(root / 'graph.json', dict(qualification=qualification_schedule(), benchmark=schedule(),
        qualification_excluded=True, warmup_excluded=True, primary=list(PRIMARY), aggregate_gate=False,
        complete_qualification_bodies=True, timed_read_stdout='/dev/null', per_trial_read_body_retained=False))
    cpus = machine(root, policy)
    check(shutil.disk_usage(root).free >= 41 * 2**30, 'capacity: fresh41GiB after image admission is required')
    capacity(root, 'initial')
    admit_owner(root, policy)
    carrier = acquire_carrier(root, policy)
    receipt = load(ROOT / policy['accepted_reference']['path'])
    programs = restore_runtime(root, carrier, policy, receipt)
    os.environ['LD_LIBRARY_PATH'] = str(root / 'runtime/prefix/lib')
    programs['timer'] = prepare_timer(root, policy)
    admit_runtime(root, programs, policy, receipt, cpus)
    os.environ.pop('GH_TOKEN', None)
    capacity(root, 'runtime-admitted')
    runtime_files = [p for p in (root / 'runtime').rglob('*') if p.is_file() and not p.is_symlink()
                     and not p.is_relative_to(root / 'runtime/carriers')]
    runtime_frozen = freeze_files(runtime_files)
    for filename, pin in (('original-runtime.zip', policy['runtime']['zip']), ('runtime.tar', policy['runtime']['tar'])):
        path = root / 'runtime/carriers' / filename
        runtime_frozen[str(path)] = dict(stat=file_stat(path), identity=pin)
    frozen = generate_fixture(root, programs, policy, cpus)
    check(time.monotonic() - started <= 1800, 'fixed admission/generation bound exceeded')
    arm_deadline(state['deadline'])
    save(root / 'frozen-runtime.json', runtime_frozen)
    qualified = qualification(root, programs, policy, cpus, frozen, state)
    with comparisons(state):
        check_frozen(runtime_frozen, hash_bodies=True)
        qualified_frozen = freeze_files(sorted(set(qualified.values())))
    records = benchmark(root, programs, policy, cpus, qualified, {**frozen, **runtime_frozen, **qualified_frozen}, state)
    save(root / 'time-budget.json', dict(elapsed_seconds=time.monotonic() - started,
         comparison_seconds=state['comparison_spent'], controller_limit_seconds=18600,
         qualification_limit_seconds=1750, benchmark_limit_seconds=13650))
    check(verify_context(ROOT, policy) == context, 'controller/authority context changed during campaign')
    report = evaluate(records)
    save(root / 'performance.json', report)
    capacity(root, 'complete')
    save(root / 'outcome.json', dict(schema=policy['schema'], status='complete-correct-13-lane-campaign',
         controller=os.environ['GITHUB_SHA'], owner=OWNER, qualification_calls=42, warmup_calls=78, measured_calls=260,
         primary_passed=report['primary_passed'], performance_accepted_by_root=False,
         manual_full_original_adoption_required=True, final_tradeoff_review_required=True, publication=False))
    signal.setitimer(signal.ITIMER_REAL, 0)
    if not report['primary_passed']:
        raise RuntimeError('complete correct measurements did not qualify EACH of the four named serial primary hot paths')


def finalize():
    root = hosted_session()
    started = time.monotonic()
    signal.signal(signal.SIGALRM, deadline_expired)
    signal.setitimer(signal.ITIMER_REAL, 600)
    logical = {path.relative_to(root).as_posix(): load(path) for path in sorted((root / 'metadata/custody').rglob('*.json'))}
    files = source.tree_files(root, ('artifact-manifest.json',))
    for record in logical.values():
        owned(root, root / record['backup'])
        backing = files[record['backup']]
        check(backing['type'] == 'file' and {k: backing[k] for k in ('bytes', 'sha256')} == record['backup_identity'],
              'final unique backup body differs from its logical output identity')
    total = sum(row['bytes'] for row in files.values() if row['type'] == 'file')
    save(root / 'artifact-manifest.json', dict(schema='rsomics.index.bgzf-real.manifest.v1',
         controller=os.environ['GITHUB_SHA'], owner=OWNER, files=files, logical_outputs=logical,
         retained_regular_bytes=total, retained_limit_bytes=32 * 2**30,
         artifact_zip_limit_bytes=34 * 2**30, complete_recovery_bytes=76 * 2**30,
         original_runtime_zip_tar_preserved=True, direct_modes_recorded=True,
         data='public deterministic synthetic VCF/AES-CTR fixtures; no personal genomes',
         outcome=load(root / 'outcome.json') if (root / 'outcome.json').exists() else None,
         runtime_route='whole original mode/link-preserving TAR plus selective exact runtime; no new project TAR',
         timed_reads='stdout /dev/null; complete preceding qualification only, not fictitious per-trial read outputs',
         failed_and_novel_bodies_retained=True, publication=False,
         finalization_elapsed_seconds=time.monotonic() - started))
    signal.setitimer(signal.ITIMER_REAL, 0)
    check(total <= 32 * 2**30, 'capacity: available originals exceed frozen retained cap; no cleanup authorized')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=('init', 'execute', 'finalize'))
    action = parser.parse_args().action
    try:
        {'init': initialize, 'execute': execute, 'finalize': finalize}[action]()
    except BaseException:
        signal.setitimer(signal.ITIMER_REAL, 0)
        root = os.environ.get('INDEX_SESSION')
        if root and Path(root).is_dir():
            failure = dict(action=action, exception=traceback.format_exc(), publication=False,
                           performance_accepted=False, failures_and_novel_bodies_retained=True)
            save(Path(root) / ('failure-' + action + '.json'), failure)
            if action == 'execute' and not (Path(root) / 'outcome.json').exists():
                save(Path(root) / 'outcome.json', dict(status='HOLD', **failure))
        traceback.print_exc()
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())

