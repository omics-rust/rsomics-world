"""Manual complete-study qualification and fixed paired Ecology performance gate."""

import argparse
import ast
import csv
import hashlib
import io
import json
import math
import os
from pathlib import Path
import platform
import re
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

import private_artifact as transport
from ecology_candidate import check, load, save, portable_names, _hash_file, _regular
from popgen_hosted_source import extract_archive, registry_image
from signal_real_performance import zip_members, proc_group


ROOT = Path(__file__).resolve().parents[1]
ASSET = 'assets/ecology-whole-real-20261101.json'
WORKFLOW = '.github/workflows/ecology-whole-real.yml'
TARGETS = {'linux-x86_64', 'linux-aarch64', 'macos-x86_64', 'macos-aarch64'}
FILES = ('original-alpha.tsv', 'rarefied.tsv', 'rarefied-alpha.tsv')


def identity(path):
    path = Path(path)
    _regular(path)
    return dict(bytes=path.stat().st_size, sha256=_hash_file(path))


def read_policy(world=ROOT):
    policy = load(Path(world) / ASSET)
    check(policy['schema'] == 'rsomics.ecology.whole-real-policy.v1'
          and policy['product'] == 'rsomics-ecology'
          and policy['owner_commit'] == 'e1b03cb68876be2dd8bb9fbc520b5194be055d9e'
          and policy['study']['features'] == 41328 and policy['study']['samples'] == 1105
          and len(policy['source_runtime']['wheels']) == 27
          and identity(Path(world) / 'assets/ecology-native-20261101.json') == policy['native_authority'],
          'adopted complete-study policy differs')
    return policy


def retention_plan():
    result = transport.finalization_space(6 * 2**30, 7 * 2**30)
    return dict(retained_limit_bytes=6 * 2**30, tar_bound_bytes=7 * 2**30,
                initial_free_bytes=6 * 2**30 + result['required_free_bytes'], **result)


def schedule():
    rows = [dict(phase='qualification', workers=workers, pair=0, engine=engine)
            for engine in ('source', 'native') for workers in (1, 4)]
    for workers in (1, 4):
        rows += [dict(phase='warmup', workers=workers, pair=0, engine=engine) for engine in ('source', 'native')]
        for pair in range(1, 6):
            for engine in (('source', 'native') if pair % 2 else ('native', 'source')):
                rows.append(dict(phase='measured', workers=workers, pair=pair, engine=engine))
    return rows


def compare_alpha(expected, actual, samples, metrics):
    maximum, cells = 0.0, 0
    with Path(expected).open(newline='') as left, Path(actual).open(newline='') as right:
        lrows, rrows = csv.reader(left, delimiter='\t'), csv.reader(right, delimiter='\t')
        check(next(lrows) == next(rrows) == ['sample', *metrics], 'alpha metric header differs')
        for sample in samples:
            source, native = next(lrows), next(rrows)
            check(len(source) == len(native) == len(metrics) + 1 and source[0] == native[0] == sample,
                  'alpha sample/shape/order differs')
            for metric, text_s, text_n in zip(metrics, source[1:], native[1:]):
                s, n = float(text_s), float(text_n)
                if not math.isfinite(s) or not math.isfinite(n):
                    check((math.isnan(s) and math.isnan(n)) or s == n, 'alpha unavailable class differs')
                elif metric == 'observed_features':
                    check(s.is_integer() and s == n, 'observed-feature count differs')
                else:
                    error = abs(n - s)
                    check(error <= 1e-10 + 1e-10 * abs(s), 'finite alpha value differs')
                    maximum = max(maximum, error)
                cells += 1
        check(next(lrows, None) is None and next(rrows, None) is None, 'extra alpha sample')
    return dict(cells=cells, maximum_absolute_error=maximum)


def count_rows(path, features, samples):
    with Path(path).open(newline='') as stream:
        rows = csv.reader(stream, delimiter='\t')
        header = next(rows)
        check(header[0] in ('feature', '#OTU ID') and header[1:] == samples, 'count sample header differs')
        for feature in features:
            row = next(rows)
            check(len(row) == len(samples) + 1 and row[0] == feature
                  and all(re.fullmatch('[0-9]+', value) for value in row[1:]), 'count feature/domain differs')
            yield list(map(int, row[1:]))
        check(next(rows, None) is None, 'extra count feature')


def compare_counts(original, expected, actual, features, samples, depth):
    totals = [0] * len(samples)
    cells = 0
    for before, source, native in zip(count_rows(original, features, samples),
            count_rows(expected, features, samples), count_rows(actual, features, samples), strict=True):
        check(source == native and all(0 <= value <= maximum for value, maximum in zip(native, before)),
              'rarefied count value/conservation differs')
        for column, value in enumerate(native):
            totals[column] += value
        cells += len(native)
    check(totals == [depth] * len(samples), 'rarefied sample total differs')
    return dict(cells=cells, totals=totals, output=identity(actual))


def evaluate(pairs):
    check(len(pairs) == 5, 'five complete measured pairs required')
    endpoints = {}
    for endpoint in ('a', 'w'):
        ratios = {}
        for field in ('wall', 'cpu', 'rss'):
            s = [row['source'][endpoint][field] for row in pairs]
            n = [row['native'][endpoint][field] for row in pairs]
            check(all(math.isfinite(value) and value > 0 for value in s + n), 'missing/nonfinite resource observation')
            ratios[field] = [native / source for native, source in zip(n, s)]
        medians = {key: statistics.median(values) for key, values in ratios.items()}
        throughput = (medians['wall'] <= .8 and sum(value <= .9 for value in ratios['wall']) >= 4
                      and medians['cpu'] <= 1.05 and medians['rss'] <= 1.10)
        memory = (medians['rss'] <= .75 and all(value < 1 for value in ratios['rss'])
                  and medians['wall'] <= 1.05 and medians['cpu'] <= 1.05)
        endpoints[endpoint] = dict(passed=throughput or memory, throughput=throughput,
                                   conservative_process_memory=memory, ratios=ratios, median_ratios=medians)
    later = {}
    for stage in ('rarefy', 'rarefied-alpha'):
        values = [row['native']['stages'][stage] / row['source']['stages'][stage] for row in pairs]
        check(all(math.isfinite(value) and value > 0 for value in values), 'invalid stage timing')
        later[stage] = dict(ratios=values, passed=statistics.median(values) <= 1.10)
    return dict(passed=all(row['passed'] for row in [*endpoints.values(), *later.values()]),
                endpoints=endpoints, later_stages=later, aggregate_rss_exact=False)


def validate_native_receipt(world, relative, expected_sha=None):
    world = Path(world).resolve(strict=True)
    check(re.fullmatch(r'\.autopilot/oracles/ecology-native-[A-Za-z0-9.-]+\.json', relative) is not None,
          'tracked accepted Ecology native receipt path required')
    path = world / relative
    _regular(path)
    check(path.resolve(strict=True).is_relative_to(world), 'native receipt escapes checkout')
    receipt, policy = load(path), read_policy(world)
    pin = policy['native_receipt']
    check(relative == pin['path'] and _hash_file(path) == pin['sha256']
          and expected_sha in (None, pin['sha256']), 'actual pinned native receipt SHA/path differs')
    check(receipt['schema'] == 'rsomics.ecology.native-adoption.v1' and receipt['status'] == 'accepted'
          and receipt['product'] == policy['product'] and receipt['repository'] == 'omics-rust/rsomics-world'
          and receipt['workflow_path'] == '.github/workflows/ecology-candidate.yml'
          and receipt['native_four_qualified'] is True and receipt['root_fresh_review_required'] is False,
          'native four acceptance is absent')
    for key in ('owner_commit', 'owner_tree', 'lock_sha256', 'source_archive_sha256', 'source_manifest_sha256'):
        check(receipt[key] == policy[key], 'accepted native source differs: ' + key)
    check(set(receipt['targets']) == TARGETS and all(isinstance(receipt[name], int) and receipt[name] > 0
          for name in ('run_id', 'run_attempt')), 'native target/run authority differs')
    return receipt


def rss_bytes(usage):
    return usage.ru_maxrss if platform.system() == 'Darwin' else usage.ru_maxrss * 1024


def capture(root, name, argv, *, workers, timeout, started_arg=False, own_group=True):
    output = Path(root) / name
    output.mkdir(parents=True, exist_ok=False)
    cpus = sorted(os.sched_getaffinity(0))[:workers] if hasattr(os, 'sched_getaffinity') else None
    check(cpus is None or len(cpus) == workers, 'requested CPU allowance is unavailable')
    command = list(map(str, argv))
    started = time.monotonic_ns()
    if started_arg:
        command += ['--started-ns', str(started)]
    save(output / 'request.json', dict(argv=command, workers=workers, selected_cpus=cpus,
                                      started_monotonic_ns=started, own_process_group=own_group))
    process, status, counters = None, None, None
    maximum_sampled, observations, failure = None, 0, None
    def affinity():
        if cpus is not None:
            os.sched_setaffinity(0, cpus)
    with (output / 'stdout.txt').open('xb') as stdout, (output / 'stderr.txt').open('xb') as stderr:
        try:
            process = subprocess.Popen(command, stdout=stdout, stderr=stderr, start_new_session=own_group,
                                       preexec_fn=affinity)
            while True:
                pid, wait_status, usage = os.wait4(process.pid, os.WNOHANG)
                if pid:
                    process.returncode = status = os.waitstatus_to_exitcode(wait_status)
                    counters = dict(user_seconds=usage.ru_utime, system_seconds=usage.ru_stime,
                                    rss_bytes=rss_bytes(usage), input_blocks=usage.ru_inblock,
                                    output_blocks=usage.ru_oublock, minor_faults=usage.ru_minflt,
                                    major_faults=usage.ru_majflt)
                    break
                elapsed = (time.monotonic_ns() - started) / 1e9
                if platform.system() == 'Linux' and own_group:
                    group = proc_group(process.pid)
                    sampled = sum(row['rss_bytes'] for row in group)
                    maximum_sampled = max(maximum_sampled or 0, sampled)
                    observations += 1
                check(elapsed <= timeout, 'workflow deadline exceeded')
                private = os.environ.get('ECOLOGY_PRIVATE')
                if private:
                    check(shutil.disk_usage(private).free >= retention_plan()['required_free_bytes'],
                          'finalization headroom reached; partial originals retained')
                time.sleep(.1)
        except Exception:
            failure = traceback.format_exc()
        finally:
            if process is not None and process.returncode is None:
                stop = (lambda sig: os.killpg(process.pid, sig)) if own_group else (lambda sig: process.send_signal(sig))
                try:
                    stop(signal.SIGTERM)
                    process.wait(timeout=10)
                except ProcessLookupError:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    stop(signal.SIGKILL)
                    process.wait(timeout=10)
                status = process.returncode
    result = dict(returncode=status, wall=(time.monotonic_ns() - started) / 1e9, raw_wait4=counters,
                  requested_workers=workers, selected_cpus=cpus, affinity_applied=cpus is not None,
                  sampled_process_group_rss_peak_bytes=maximum_sampled, rss_sample_count=observations,
                  sampled_rss_is_not_exact_aggregate=True, failure=failure,
                  stdout=identity(output / 'stdout.txt'), stderr=identity(output / 'stderr.txt'))
    save(output / 'status.json', result)
    if status != 0 or failure:
        raise RuntimeError('required Ecology command failed: ' + name)
    return result


def api(endpoint, destination, limit, binary=False):
    token = os.environ.get('GH_TOKEN') or os.environ.get('GITHUB_TOKEN')
    check(token, 'read-only Actions API token required')
    request = urllib.request.Request('https://api.github.com/repos/omics-rust/rsomics-world/' + endpoint,
        headers={'Accept': 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28',
                 'User-Agent': 'rsomics-ecology-qualification'})
    request.add_unredirected_header('Authorization', 'Bearer ' + token)
    download_request(request, destination, limit)
    return None if binary else load(destination)


def download_request(request, destination, limit, *, status=200, content_range=None):
    destination = Path(destination)
    destination.parent.mkdir(parents=True, exist_ok=True)
    record = dict(url=request.full_url, expected_status=status, byte_limit=limit)
    try:
        with urllib.request.urlopen(request, timeout=120) as response:
            record.update(status=response.status, headers={name: response.headers.get(name) for name in
                ('Content-Length', 'Content-Range', 'Content-Encoding', 'ETag', 'Last-Modified')})
            check(response.status == status and (content_range is None or response.headers.get('Content-Range') == content_range),
                  'provider HTTP status/range differs before body acquisition')
            size = 0
            with destination.open('xb') as output:
                while block := response.read(2**20):
                    size += len(block)
                    check(size <= limit, 'original download exceeds frozen bound')
                    output.write(block)
        record['body'] = identity(destination)
    except Exception:
        record['failure'] = traceback.format_exc()
        raise
    finally:
        save(Path(str(destination) + '.receipt.json'), record)
    return record['body']


def checked_download(spec, destination):
    request = urllib.request.Request(spec['url'], headers={'User-Agent': 'rsomics-ecology-real-workflow'})
    actual = download_request(request, destination, spec.get('bytes', spec.get('size')))
    check(actual == dict(bytes=spec.get('bytes', spec.get('size')), sha256=spec['sha256']), 'pinned original body differs')
    return actual


def safe_zip_extract(archive_path, destination, limit):
    destination.mkdir(parents=True, exist_ok=False)
    with zipfile.ZipFile(archive_path) as archive:
        rows = zip_members(archive)
        check(sum(row.file_size for row in rows) <= limit and archive.testzip() is None,
              'original ZIP size/CRC differs')
        for row in rows:
            output = destination / row.filename
            if row.is_dir():
                output.mkdir(parents=True, exist_ok=True)
            else:
                output.parent.mkdir(parents=True, exist_ok=True)
                with archive.open(row) as source, output.open('xb') as target:
                    shutil.copyfileobj(source, target)


def acquire_action(root, run_receipt, artifact):
    root.mkdir(parents=True)
    run = api(f"actions/runs/{run_receipt['run_id']}/attempts/{run_receipt['run_attempt']}", root / 'run.json', 2**20)
    check(run['head_sha'] == run_receipt['head_sha'] and run['status'] == 'completed'
          and run['conclusion'] == 'success' and run['repository']['full_name'] == 'omics-rust/rsomics-world',
          'actual successful source/native run authority differs')
    row = api(f"actions/artifacts/{artifact['artifact_id']}", root / 'artifact.json', 2**20)
    check(row['name'] == artifact['artifact_name'] and row['size_in_bytes'] == artifact['bytes']
          and row.get('digest') == 'sha256:' + artifact['sha256'] and row['expired'] is False
          and row['workflow_run']['id'] == run_receipt['run_id'], 'original Actions artifact authority differs')
    api(f"actions/artifacts/{artifact['artifact_id']}/zip", root / 'artifact.zip', artifact['bytes'], binary=True)
    check(identity(root / 'artifact.zip') == {key: artifact[key] for key in ('bytes', 'sha256')},
          'complete original Actions ZIP differs')
    return root / 'artifact.zip'


def admit_native(private, relative, expected_sha, policy):
    receipt = validate_native_receipt(ROOT, relative, expected_sha)
    tracked = subprocess.run(['git', 'ls-files', '--error-unmatch', relative], cwd=ROOT,
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    check(tracked.returncode == 0, 'accepted native receipt is not tracked')
    retained = private / 'native-receipt.json'
    shutil.copy2(ROOT / relative, retained)
    row = receipt['targets']['linux-x86_64']
    artifact = dict(artifact_id=row['artifact_id'], artifact_name=row['artifact_name'], **row['artifact'])
    directory = private / 'runtime/native-artifact'
    carrier = acquire_action(directory, receipt, artifact)
    jobs = api(f"actions/runs/{receipt['run_id']}/attempts/{receipt['run_attempt']}/jobs?per_page=100",
               directory / 'jobs.json', 2**20)
    check(jobs['total_count'] == len(jobs['jobs']) == 4 and {job['name'] for job in jobs['jobs']} == TARGETS,
          'actual complete four-native jobs are absent')
    for job in jobs['jobs']:
        check(job['conclusion'] == 'success' and job['status'] == 'completed'
              and job['head_sha'] == receipt['head_sha'] and job['run_attempt'] == receipt['run_attempt']
              and job['id'] == receipt['targets'][job['name']]['job_id'], 'native job identity/outcome differs')
    safe_zip_extract(carrier, directory / 'zip', 64 * 2**20)
    tar = directory / 'zip/ecology-evidence.tar'
    check(identity(tar) == row['tar'], 'original native TAR differs')
    destination = directory / 'bodies'
    destination.mkdir()
    with tarfile.open(tar) as archive:
        members = archive.getmembers()
        portable_names([member.name for member in members])
        check(all(member.isfile() and member.type == tarfile.REGTYPE for member in members)
              and sum(member.size for member in members) < 64 * 2**20, 'native TAR membership differs')
        manifest = json.load(archive.extractfile('ecology-evidence/artifact-manifest.json'))
        check(manifest['schema'] == 'rsomics.ecology.native-four.v1', 'native carrier schema differs')
        expected = {'ecology-evidence/' + name: value for name, value in manifest['files'].items()}
        check(set(expected) | {'ecology-evidence/artifact-manifest.json'} == {member.name for member in members},
              'complete native manifest membership differs')
        for member in members:
            output = destination / member.name
            output.parent.mkdir(parents=True, exist_ok=True)
            with archive.extractfile(member) as source, output.open('xb') as target:
                shutil.copyfileobj(source, target)
            output.chmod(member.mode & 0o777)
            if member.name in expected:
                check(identity(output) == {key: expected[member.name][key] for key in ('bytes', 'sha256')}
                      and stat.S_IMODE(output.stat().st_mode) == expected[member.name]['mode'], 'native manifest body differs')
    candidate = destination / 'ecology-evidence/candidate'
    selection = load(candidate / 'selection.json')
    check(_hash_file(candidate / 'selection.json') == row['binary']['selection_sha256'], 'actual Cargo selection differs')
    for key in ('owner_commit', 'owner_tree', 'source_archive_sha256', 'source_manifest_sha256', 'lock_sha256'):
        check(selection[key] == policy[key], 'actual native source/selection differs: ' + key)
    binary = candidate / 'native-binary/rsomics-ecology-linux-x86_64'
    check(load(binary.parent / 'identity.json') == row['binary']
          and identity(binary) == {key: row['binary'][key] for key in ('bytes', 'sha256')}
          and stat.S_IMODE(binary.stat().st_mode) == row['binary']['mode'], 'retained native binary differs')
    with binary.open('rb') as stream:
        header = stream.read(32)
    check(header[:6] == b'\x7fELF\x02\x01' and struct.unpack_from('<H', header, 18)[0] == 62,
          'actual Linux x86_64 ELF is absent')
    capture(private / 'calls', 'native-loader', ['ldd', binary], workers=1, timeout=30)
    capture(private / 'calls', 'native-version', [binary, '--version'], workers=1, timeout=30)
    check((private / 'calls/native-version/stdout.txt').read_text().strip() == 'rsomics-ecology 0.1.0',
          'actual native ELF cannot execute with its expected version in Bookworm')
    save(private / 'native-admission.json', dict(binary=str(binary), identity=identity(binary),
         receipt=identity(retained), actual_bookworm_execution=True, native_rebuilt=False))
    return binary


def prepare_runtime(private, policy):
    runtime = private / 'runtime'
    source = policy['original_source']
    row = dict(artifact_id=source['artifact_id'], artifact_name=source['artifact_name'],
               bytes=source['size_in_bytes'], sha256=source['artifact_sha256'])
    archive = acquire_action(runtime / 'source-artifact', source, row)
    safe_zip_extract(archive, runtime / 'original-source', 2**30)
    old = runtime / 'original-source'
    check(_hash_file(old / 'witness/witness.json') == source['witness_sha256']
          and _hash_file(old / 'witness/sha256-manifest.tsv') == source['manifest_sha256'], 'old immutable source witness differs')
    python_spec = policy['python']
    original = runtime / python_spec['filename']
    checked_download(python_spec, original)
    checksum = runtime / (python_spec['filename'] + '.sha256')
    download_request(urllib.request.Request(python_spec['checksum_url']), checksum, 4096)
    check(checksum.read_text().split()[0] == python_spec['sha256'], 'official interpreter checksum differs')
    extract_archive(original, runtime / 'interpreter', 2**30)
    base = runtime / 'interpreter/python/bin/python3.12'
    capture(private / 'calls', 'create-venv', [base, '-m', 'venv', runtime / 'venv'], workers=1, timeout=120)
    python = runtime / 'venv/bin/python'
    lock = runtime / 'requirements.lock'
    with lock.open('x') as stream:
        for name, row in policy['source_runtime']['wheels'].items():
            wheel = old / 'wheels' / row['filename']
            check(identity(wheel) == {key: row[key] for key in ('bytes', 'sha256')}, 'original wheel body differs')
            with zipfile.ZipFile(wheel) as archive:
                check(hashlib.sha256(archive.read(row['metadata_member'])).hexdigest() == row['metadata_sha256'],
                      'original wheel metadata differs')
            stream.write(f'{name}=={row["version"]} --hash=sha256:{row["sha256"]}\n')
    capture(private / 'calls', 'install-source-wheels', [python, '-m', 'pip', 'install', '--no-index',
        '--find-links', old / 'wheels', '--only-binary=:all:', '--no-deps', '--no-cache-dir', '--require-hashes',
        '-r', lock], workers=1, timeout=600)
    capture(private / 'calls', 'source-pip-check', [python, '-m', 'pip', 'check'], workers=1, timeout=60)
    capture(private / 'calls', 'source-pip-inspect', [python, '-m', 'pip', 'inspect', '--local'], workers=1, timeout=60)
    source_command(private, python, 'admit', 'source-admission', workers=1, timeout=120)
    return python


class CachedRanges(io.RawIOBase):
    def __init__(self, size, rows):
        self.size, self.rows, self.position = size, rows, 0

    def seekable(self):
        return True

    def tell(self):
        return self.position

    def seek(self, offset, whence=0):
        self.position = offset + (0 if whence == 0 else self.position if whence == 1 else self.size)
        check(0 <= self.position <= self.size, 'selected archive seek differs')
        return self.position

    def read(self, size=-1):
        size = self.size - self.position if size < 0 else min(size, self.size - self.position)
        output = bytearray()
        while size:
            matches = [row for row in self.rows if row[0] <= self.position <= row[1]]
            check(len(matches) == 1, 'ZIP requires bytes outside the fixed selected ranges')
            start, end, body = matches[0]
            count = min(size, end - self.position + 1)
            output.extend(body[self.position - start:self.position - start + count])
            self.position += count
            size -= count
        return bytes(output)


def acquire_study(private, policy):
    data = private / 'data'
    data.mkdir()
    study = policy['study']
    metadata = data / 'zenodo-record.json'
    download_request(urllib.request.Request(study['api_url']), metadata, 2**20)
    provider = load(metadata)
    check(provider['id'] == study['record_id'] and provider['doi'] == study['doi']
          and provider['metadata']['license']['id'] == 'cc-by-4.0'
          and provider['metadata']['access_right'] == 'open', 'fixed public study/rights differ')
    names = {row['key']: row for row in provider['files']}
    for name in ('asv-table.qza', 'sample-metadata.tsv'):
        spec = study['originals'][name]
        check(names[name]['size'] == spec['bytes'] and names[name]['checksum'] == 'md5:' + spec['md5'],
              'publisher original identity differs')
        checked_download(spec, data / name)
    specification = study['tree_ranges']
    outer = names['gut-to-soil-qiime2.zip']
    check(outer['size'] == specification['total_bytes'] and outer['checksum'] == 'md5:' + specification['publisher_md5'],
          'selected original archive directory authority differs')
    rows = []
    for number, row in enumerate(specification['requests']):
        target = data / 'ranges' / f'{number:02d}-{row["start"]}-{row["end"]}.bin'
        request = urllib.request.Request(specification['url'], headers={
            'Range': f'bytes={row["start"]}-{row["end"]}', 'Accept-Encoding': 'identity'})
        actual = download_request(request, target, row['requested_bytes'], status=206,
            content_range=f'bytes {row["start"]}-{row["end"]}/{specification["total_bytes"]}')
        check(actual == dict(bytes=row['requested_bytes'], sha256=row['sha256']), 'original selected range bytes differ')
        rows.append((row['start'], row['end'], target.read_bytes()))
    with zipfile.ZipFile(CachedRanges(specification['total_bytes'], rows)) as archive:
        spec = study['originals']['rooted-tree.qza']
        item = archive.getinfo(spec['outer_member'])
        check(item.file_size == spec['bytes'] and f'{item.CRC:08x}' == spec['outer_crc32'], 'outer tree-QZA member differs')
        with archive.open(item) as source, (data / 'rooted-tree.qza').open('xb') as target:
            shutil.copyfileobj(source, target)
    for name in ('asv-table.qza', 'rooted-tree.qza'):
        spec = study['originals'][name]
        check(identity(data / name) == {key: spec[key] for key in ('bytes', 'sha256')}, 'complete original QZA differs')
        with zipfile.ZipFile(data / name) as archive:
            members = zip_members(archive)
            check(archive.testzip() is None and {row.filename.split('/')[0] for row in members} == {spec['uuid']},
                  'complete original QZA membership/CRC differs')
            payload = spec['payload']
            target = data / ('feature-table.biom' if name == 'asv-table.qza' else 'tree.nwk')
            with archive.open(payload['path']) as source, target.open('xb') as output:
                shutil.copyfileobj(source, output)
            check(identity(target) == {key: payload[key] for key in ('bytes', 'sha256')}, 'original QZA scientific payload differs')
            save(data / (name + '.members.json'), [dict(name=row.filename, bytes=row.file_size, crc32=f'{row.CRC:08x}')
                                                  for row in members])
    save(data / 'range-scope.json', dict(whole_archive_retained=False, whole_archive_checksum_verified=False,
         original_member_crc_verified=True, exact_ranges=specification['requests'], attribution=policy['attribution']))


def source_command(private, python, action, name, *, workers, timeout, output=None, started_arg=False):
    command = [python, '-B', ROOT / 'scripts/ecology_source_workflow.py', action, '--session', private,
               '--workers', str(workers)]
    if output is not None:
        command += ['--output', output]
    return capture(private / 'calls', name, command, workers=workers, timeout=timeout, started_arg=started_arg)


def bridge_check(private, output, metrics):
    witness = private / 'runtime/original-source/witness'
    request = load(witness / 'cases/original/request.json')
    expected = private / 'metadata/old-bridge-reference'
    expected.mkdir(parents=True)
    results = {}
    for name, case in (('original-alpha.tsv', 'cases/original'),
                       ('rarefied-alpha.tsv', 'cases/rarefied/diversity')):
        columns = []
        for metric in metrics:
            source_name = 'phydiv-rooted-quarter' if metric == 'phydiv' else metric
            observation = load(witness / case / 'alpha' / source_name / 'observation.json')
            check(observation['status'] == 'returned' and observation['result']['ids'] == request['sample_ids'],
                  'old accepted bridge series differs')
            columns.append(observation['result']['values'])
        target = expected / name
        with target.open('x') as stream:
            stream.write('\t'.join(['sample', *metrics]) + '\n')
            for sample, values in zip(request['sample_ids'], zip(*columns)):
                stream.write(sample + '\t' + '\t'.join(str(value['nonfinite'] if isinstance(value, dict) else value)
                                                     for value in values) + '\n')
        results[name] = compare_alpha(target, output / name, request['sample_ids'], metrics)
    results['rarefied.tsv'] = compare_counts(witness / 'inputs/primary-counts.tsv',
        witness / 'cases/rarefied/counts.tsv', output / 'rarefied.tsv', request['feature_ids'], request['sample_ids'], 8)
    save(private / 'metadata/old-case-bridge.json', dict(status='qualified', selected_metrics=metrics,
         source_phydiv='phydiv-rooted-quarter', comparisons=results, new_source_casebook=False))


def native_workflow(private, binary, output, workers, started_ns):
    policy = read_policy()
    output.mkdir(parents=True, exist_ok=False)
    commands = []
    metrics = ','.join(policy['policy']['metrics'])
    def alpha(input_path, destination):
        return [binary, 'alpha', '--counts', input_path, '--tree', private / 'data/tree.nwk',
                '--output', destination, '--metrics', metrics, '--pd-rooted', 'auto', '--pd-weight', '.25',
                '--threads', str(workers), '--json']
    def measured(name, argv):
        row = capture(output / 'calls', name, argv, workers=workers, timeout=1200, own_group=False)
        commands.append(row)
        own = resource.getrusage(resource.RUSAGE_SELF)
        return dict(wall=(time.monotonic_ns() - started_ns) / 1e9,
                    cpu=own.ru_utime + own.ru_stime + sum(
                        entry['raw_wait4']['user_seconds'] + entry['raw_wait4']['system_seconds'] for entry in commands),
                    rss=max([rss_bytes(own), *[entry['raw_wait4']['rss_bytes'] for entry in commands]]),
                    aggregate_rss_exact=False)
    original_end = measured('original-alpha', alpha(private / 'data/study.tsv', output / 'original-alpha.tsv'))
    first = commands[0]
    original = dict(wall=first['wall'],
        cpu=first['raw_wait4']['user_seconds'] + first['raw_wait4']['system_seconds'],
        rss=first['raw_wait4']['rss_bytes'], scope='first real native CLI; excludes composition-wrapper startup')
    rarefaction = measured('rarefy', [binary, 'rarefy', '--counts', private / 'data/study.tsv', '--output', output / 'rarefied.tsv',
        '--depth', '3', '--seed', '42', '--threads', str(workers), '--json'])
    entire = measured('rarefied-alpha', alpha(output / 'rarefied.tsv', output / 'rarefied-alpha.tsv'))
    report = dict(a=original, w=entire, stages={'rarefy': rarefaction['wall'] - original_end['wall'],
        'rarefied-alpha': entire['wall'] - rarefaction['wall']}, native_cli_processes=3,
        workers=workers, counts_reused_between_stages=False,
        commands=commands)
    save(output / 'workflow.json', report)


def file_manifest(private):
    result = {}
    for path in sorted(private.rglob('*')):
        relative = path.relative_to(private).as_posix()
        if relative == 'artifact-manifest.json':
            continue
        row = dict(mode=stat.S_IMODE(path.lstat().st_mode))
        if path.is_symlink():
            check(path.resolve(strict=True).is_relative_to(private.resolve(strict=True)), 'runtime symlink escapes private root')
            row.update(kind='symlink', target=os.readlink(path))
        elif path.is_dir():
            row['kind'] = 'directory'
        else:
            row.update(kind='file', **identity(path))
        result[relative] = row
    return result


def capacity(private):
    policy = read_policy()['policy']
    buckets = {name: 0 for name in policy['budgets']}
    for path in private.rglob('*'):
        if not path.is_file() or path.is_symlink():
            continue
        first = path.relative_to(private).parts[0]
        bucket = {'data': 'inputs', 'runtime': 'runtime', 'runs': 'science', 'reference': 'science'}.get(first, 'metadata')
        buckets[bucket] += path.stat().st_size
    check(all(buckets[name] <= maximum for name, maximum in policy['budgets'].items()),
          'retained capacity envelope reached; no deletion-to-fit')
    check(shutil.disk_usage(private).free >= retention_plan()['required_free_bytes'], 'cipher/finalization headroom unavailable')
    return buckets


def controller_paths():
    queue = ['scripts/ecology_whole_real.py', 'scripts/ecology_source_workflow.py', 'scripts/test_ecology_workflows.py']
    seen = set()
    while queue:
        name = queue.pop()
        if name in seen:
            continue
        seen.add(name)
        tree = ast.parse((ROOT / name).read_text())
        for node in ast.walk(tree):
            modules = [row.name for row in node.names] if isinstance(node, ast.Import) else (
                [node.module] if isinstance(node, ast.ImportFrom) and node.module else [])
            for module in modules:
                candidate = 'scripts/' + module.replace('.', '/') + '.py'
                if (ROOT / candidate).is_file():
                    queue.append(candidate)
    return sorted([*seen, ASSET, WORKFLOW, 'assets/ecology-native-20261101.json',
                   'assets/private-artifact-tools-20261101.json', 'assets/private-artifacts-20261101.recipient',
                   '.autopilot/oracles/ecology-diversity-2026-10-28.json'])


def hosted():
    policy = read_policy()
    check(os.environ.get('GITHUB_ACTIONS') == 'true' and os.environ.get('GITHUB_REPOSITORY') == 'omics-rust/rsomics-world'
          and os.environ.get('GITHUB_EVENT_NAME') == 'workflow_dispatch'
          and platform.system() == 'Linux' and platform.machine() == 'x86_64'
          and os.environ.get('ECOLOGY_CONTAINER_DIGEST') == policy['image']['manifest_digest']
          and os.environ.get('GCC_VERSION') == '14.3.0', 'actual pinned manual hosted execution required')
    session = Path(os.environ['ECOLOGY_SESSION']).resolve(strict=True)
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    check(session != runner and session.is_relative_to(runner) and session.stat().st_dev == runner.stat().st_dev,
          'task root escapes runner device')
    for name in ('TMPDIR', 'XDG_CACHE_HOME', 'PIP_CACHE_DIR', 'CARGO_HOME', 'CARGO_TARGET_DIR', 'RUSTUP_HOME',
                 'PYTHONUSERBASE', 'PYTHONPYCACHEPREFIX', 'MPLCONFIGDIR'):
        path = Path(os.environ[name]).resolve(strict=True)
        check(path.is_relative_to(session) and path.stat().st_dev == session.stat().st_dev,
              'private cache/output escapes admitted task root: ' + name)
    os.sched_setaffinity(0, list(map(int, os.environ['ECOLOGY_CPUS'].split(','))))
    return session, session / 'private', policy


def initialize(receipt, sha):
    check(os.environ.get('GITHUB_ACTIONS') == 'true' and platform.system() == 'Linux', 'hosted initialization required')
    parent = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    session = Path(tempfile.mkdtemp(prefix='ecology-whole-real-', dir=parent))
    for name in ('private', 'cipher', 'private/runtime', 'private/metadata'):
        (session / name).mkdir(parents=True, exist_ok=True, mode=0o700)
    cpus = sorted(os.sched_getaffinity(0))[:4]
    values = dict(ECOLOGY_SESSION=str(session), ECOLOGY_PRIVATE=str(session / 'private'),
                  ECOLOGY_CPUS=','.join(map(str, cpus)))
    for key in ('TMPDIR', 'XDG_CACHE_HOME', 'PIP_CACHE_DIR', 'CARGO_HOME', 'CARGO_TARGET_DIR', 'RUSTUP_HOME',
                'PYTHONUSERBASE', 'PYTHONPYCACHEPREFIX', 'MPLCONFIGDIR'):
        path = session / 'private/cache' / key
        path.mkdir(parents=True)
        values[key] = str(path)
    with Path(os.environ['GITHUB_ENV']).open('a') as stream:
        for key, value in values.items():
            stream.write(f'{key}={value}\n')
            os.environ[key] = value
    _, private, policy = hosted()
    save(private / 'request.json', dict(run_id=int(os.environ['GITHUB_RUN_ID']),
        run_attempt=int(os.environ['GITHUB_RUN_ATTEMPT']), head_sha=os.environ['GITHUB_SHA'],
        native_receipt=receipt, native_receipt_sha256=sha, plan=retention_plan(), owner=policy['owner_commit']))
    check(len(cpus) == 4 and shutil.disk_usage(session).free >= retention_plan()['initial_free_bytes'],
          'complete CPU/storage reservation is unavailable')
    maximum = Path('/sys/fs/cgroup/memory.max').read_text().strip()
    swap = Path('/sys/fs/cgroup/memory.swap.max').read_text().strip()
    cpu_max = Path('/sys/fs/cgroup/cpu.max').read_text().split()
    check(maximum == str(12 * 2**30) and swap == '0', 'actual12GiB no-extra-swap cgroup is absent')
    check(cpu_max[0] != 'max' and int(cpu_max[0]) == 4 * int(cpu_max[1]), 'actual four-CPU cgroup quota differs')
    machine = dict(cpus=cpus, uname=platform.uname()._asdict(), memory_max=maximum, memory_swap_max=swap,
                   cpu_max=Path('/sys/fs/cgroup/cpu.max').read_text(), rlimit_as=resource.getrlimit(resource.RLIMIT_AS),
                   no_address_space_limit_set=True)
    for name in ('cpuinfo', 'meminfo', 'self/cgroup', 'self/mountinfo'):
        target = private / 'metadata' / name.replace('/', '-')
        target.write_bytes((Path('/proc') / name).read_bytes())
    save(private / 'metadata/machine.json', machine)
    context = private / 'context'
    for name in controller_paths():
        output = context / name
        output.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(ROOT / name, output)
    save(private / 'context-bindings.json', {name: identity(context / name) for name in controller_paths()})
    check(re.fullmatch('[0-9a-f]{64}', sha) is not None, 'actual native receipt hash is required')
    validate_native_receipt(ROOT, receipt, sha)
    check(_hash_file(ROOT / policy['recipient']['path']) == policy['recipient']['sha256'], 'public recipient differs')
    transport.prepare_tool('linux-amd64', private / 'runtime/age')


def compare_workflow(reference, actual, samples, metrics):
    report = {name: compare_alpha(reference / name, actual / name, samples, metrics)
              for name in ('original-alpha.tsv', 'rarefied-alpha.tsv')}
    check(identity(reference / 'rarefied.tsv') == identity(actual / 'rarefied.tsv'), 'complete canonical count bytes differ')
    report['rarefied.tsv'] = dict(exact_canonical_body=True, cells=41328 * 1105, body=identity(actual / 'rarefied.tsv'))
    return report


def execute():
    session, private, policy = hosted()
    started = time.monotonic()
    try:
        registry_image(private / 'metadata', policy)
        request = load(private / 'request.json')
        binary = admit_native(private, request['native_receipt'], request['native_receipt_sha256'], policy)
        python = prepare_runtime(private, policy)
        acquire_study(private, policy)
        source_command(private, python, 'prepare', 'prepare-complete-data', workers=1, timeout=600)
        admitted = load(private / 'data/admission.json')
        source_command(private, python, 'bridge', 'old-case-bridge', workers=1, timeout=120,
                       output=private / 'runtime/bridge')
        bridge_check(private, private / 'runtime/bridge', policy['policy']['metrics'])
        reference = private / 'reference/science'
        source_command(private, python, 'reference', 'first-full-public-reference', workers=1, timeout=1800,
                       output=reference, started_arg=True)
        for name in FILES:
            maximum = policy['policy']['max_rarefied_bytes' if name == 'rarefied.tsv' else 'max_alpha_bytes']
            check((reference / name).stat().st_size <= maximum, 'reference output exceeds frozen per-file envelope')
        conservation = compare_counts(private / 'data/study.tsv', reference / 'rarefied.tsv', reference / 'rarefied.tsv',
                                      admitted['features'], admitted['samples'], 3)
        save(private / 'reference/conservation.json', conservation)
        frozen_inputs = {name: identity(private / 'data' / name) for name in ('study.tsv', 'tree.nwk', 'admission.json')}
        save(private / 'timing-inputs.json', frozen_inputs)
        outcomes, native_payloads, decisions = [], None, {}
        for row in schedule():
            check(time.monotonic() - started < policy['policy']['controller_deadline_seconds'], 'controller deadline reached')
            capacity(private)
            check(frozen_inputs == {name: identity(private / 'data' / name) for name in frozen_inputs},
                  'frozen timing inputs changed')
            label = f'{row["phase"]}-{row["workers"]}-{row["pair"]}-{row["engine"]}'
            output = private / 'runs' / label / 'science'
            if row['engine'] == 'source':
                argv = [python, '-B', ROOT / 'scripts/ecology_source_workflow.py', 'workflow', '--session', private,
                        '--output', output, '--workers', str(row['workers'])]
            else:
                argv = [python, '-B', ROOT / 'scripts/ecology_whole_real.py', 'native-workflow', '--private', private,
                        '--binary', binary, '--output', output, '--workers', str(row['workers'])]
            call = capture(private / 'runs', label + '-process', argv, workers=row['workers'], timeout=1200, started_arg=True)
            metrics = load(output / 'workflow.json')
            metrics['reported_export_resources'] = metrics['w']
            raw = call['raw_wait4']
            metrics['w'] = dict(wall=call['wall'], cpu=raw['user_seconds'] + raw['system_seconds'],
                rss=raw['rss_bytes'], method='Linux wait4 RUSAGE_BOTH; waited descendant CPU counted once',
                aggregate_rss_exact=False)
            comparison = compare_workflow(reference, output, admitted['samples'], policy['policy']['metrics'])
            payloads = {name: identity(output / name) for name in FILES}
            if row['engine'] == 'native':
                if native_payloads is None:
                    native_payloads = payloads
                check(payloads == native_payloads, 'native scientific worker/repetition bytes differ')
            record = dict(**row, label=label, metrics=metrics, comparison=comparison, payloads=payloads)
            save(private / 'runs' / label / 'comparison.json', record)
            outcomes.append(record)
            if row['phase'] == 'qualification' and len(outcomes) == 4:
                save(private / 'qualification.json', dict(status='first-complete-study-qualified',
                     full_reference=True, source_native_workers=[1, 4], workflows=outcomes,
                     selected_metrics=policy['policy']['metrics'], all15_profiles_measured=False))
        for workers in (1, 4):
            pairs = []
            for pair in range(1, 6):
                selected = {row['engine']: row['metrics'] for row in outcomes
                            if row['phase'] == 'measured' and row['workers'] == workers and row['pair'] == pair}
                check(set(selected) == {'source', 'native'}, 'incomplete measured pair')
                pairs.append(selected)
            decisions[str(workers)] = evaluate(pairs)
        save(private / 'performance.json', dict(decisions=decisions, passed=all(row['passed'] for row in decisions.values()),
             complete_workflows=len(outcomes), fixed_input=True, all_outcomes_retained=True,
             source_failure_count=0, selected_metrics=policy['policy']['metrics'], full_product=False))
        capacity(private)
        check(all(row['passed'] for row in decisions.values()), 'fixed complete-study performance gate did not pass')
        save(private / 'execution-status.json', dict(status='complete-and-fixed-gates-passed'))
    except Exception:
        save(private / 'execution-status.json', dict(status='failed-originals-retained', traceback=traceback.format_exc()))
        raise


def stream_tar(private):
    manifest = load(private / 'artifact-manifest.json')
    with tarfile.open(fileobj=sys.stdout.buffer, mode='w|', format=tarfile.PAX_FORMAT) as archive:
        for name, row in sorted(manifest['files'].items()):
            path = private / name
            member = tarfile.TarInfo(name)
            member.mode, member.mtime = row['mode'], 0
            if row['kind'] == 'directory':
                member.type = tarfile.DIRTYPE
                archive.addfile(member)
            elif row['kind'] == 'symlink':
                check(os.readlink(path) == row['target'], 'finalized symlink changed')
                member.type, member.linkname = tarfile.SYMTYPE, row['target']
                archive.addfile(member)
            else:
                check(path.stat().st_size == row['bytes'], 'finalized body size changed')
                member.size = row['bytes']
                with path.open('rb') as source:
                    archive.addfile(member, source)
        path = private / 'artifact-manifest.json'
        member = tarfile.TarInfo('artifact-manifest.json')
        member.size, member.mode = path.stat().st_size, 0o600
        with path.open('rb') as source:
            archive.addfile(member, source)


def finalize():
    session, private, policy = hosted()
    bindings = load(private / 'context-bindings.json')
    check(bindings == {name: identity(ROOT / name) for name in bindings}, 'controller/source authority changed')
    capacity(private)
    files = file_manifest(private)
    save(private / 'artifact-manifest.json', dict(schema='rsomics.ecology.whole-real-originals.v1', files=files,
         native_receipt=load(private / 'request.json')['native_receipt'], policy=identity(ROOT / ASSET),
         source_failures_are_not_performance_wins=True, exact_aggregate_rss=False, complete_product=False,
         execution_status=load(private / 'execution-status.json') if (private / 'execution-status.json').exists() else None))
    context = dict(repository='omics-rust/rsomics-world', product='rsomics-ecology',
        run_id=int(os.environ['GITHUB_RUN_ID']), run_attempt=int(os.environ['GITHUB_RUN_ATTEMPT']),
        head_sha=os.environ['GITHUB_SHA'], workflow_path=WORKFLOW, phase='complete-study')
    receipt = transport.encrypt(private / 'runtime/age', ROOT / policy['recipient']['path'],
        session / 'cipher/payload.tar.age', producer=[sys.executable, '-B', __file__, 'tar', '--private', private],
        plaintext_limit=policy['policy']['tar_bound_bytes'], context=context)
    save(session / 'cipher/receipt.json', receipt)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['init', 'execute', 'finalize', 'native-workflow', 'tar'])
    parser.add_argument('--native-receipt')
    parser.add_argument('--native-receipt-sha')
    parser.add_argument('--private', type=Path)
    parser.add_argument('--binary', type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--workers', type=int, choices=[1, 4])
    parser.add_argument('--started-ns', type=int)
    args = parser.parse_args()
    if args.action == 'init':
        initialize(args.native_receipt, args.native_receipt_sha)
    elif args.action == 'execute':
        execute()
    elif args.action == 'finalize':
        finalize()
    elif args.action == 'native-workflow':
        hosted()
        native_workflow(args.private, args.binary, args.output, args.workers, args.started_ns)
    else:
        hosted()
        stream_tar(args.private)


if __name__ == '__main__':
    main()
