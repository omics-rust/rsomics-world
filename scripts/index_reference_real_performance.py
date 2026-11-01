#!/usr/bin/env python3
"""Manual hosted complete-reference comparison of native Index and samtools1.24."""

import argparse
import bisect
import gzip
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import signal
import statistics
import struct
import subprocess
import sys
import tarfile
import tempfile
import time
import traceback
import zlib

import index_reference_source as source
from plink_first_workflow_oracle import download, save, sha256


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = '.github/workflows/index-reference-real-performance.yml'
OWNER = '7f2d4fdeda9d0062d10941258b40eee16fdb9042'
OWNER_CI = 41259142238
MANIFEST = '33741a1f3dc1a89037aac0cf37512e0ef3cb3b9167e55178b8e797fb1b46a671'
LOCK = '469601b48976964ef31f1a1bbaa54f63583239e94e73129ceb8f03961d29ccb6'
BASE = 'https://ftp.1000genomes.ebi.ac.uk/vol1/ftp/technical/reference/'
URI = BASE + 'human_g1k_v37.fasta.gz'
PROVIDER_BYTES = 892331003
PROVIDER_ETAG = '"352fe3fb-4a7e0737e1bc0"'
PROVIDER_SHA = '8b6c538abf0dd92d3f3020f36cc1dd67ce004ffa421c2781205f1eb690bdb442'
PLAIN_BYTES = 3153506519
PLAIN_SHA = '2f9cd9e853a9284c53884e6a551b1c7284795dd053f255d630aeeb114d1fa81f'
FAI_SHA = '04aa12dcb4e7131d9c7361082bfd02f8415ee11e956bf358a339f0d3c54180ab'
BGZF_CAP = 3 * 2**29
BUDGET = dict(inputs=11 * 2**29, outputs=14 * 2**30, runtime=4 * 2**30, metadata=2**29)
HEADROOM = 8 * 2**30
TERMS = {
    'readme': (BASE + 'README.human_g1k_v37.fasta.txt', '6b33916bb2f3b179c28c8d5680a6c7306779ab2fd31f5e638f4ee516a64a5578'),
    'ebi': ('https://www.ebi.ac.uk/about/terms-of-use/', 'f0e32be0b599c81273f2197f5c2cf7a7d18ec2e8587e87ddd246b36b89100ab2'),
    'igsr': ('https://www.internationalgenome.org/IGSR_disclaimer/', 'cbecc1de1c4cc5075f19c2bcce19aa5e909532d1e3783e04a51f707e21789bed'),
}
RUST_MANIFEST_SHA = 'd9e59cdf15532aa839c0a4871b0b755e833f914042eb2dcc59f2a0ef8667f9f6'
RUST_COMPONENTS = {
    'rustc': 'a7169e8cb6174af2f45717703370363d8de82ce55f6ccba185893045b9370874',
    'cargo': '7103c03fb8abe85b23307005a9dfe4f01c826a89945d84b96fa2d03fd4d2d138',
    'rust-std': '89e6520b16c12b43526440298d2da0dcb70747c5cc2d0b8e47d39b5da9aeef49',
}
ENV_KEYS = source.CACHE_KEYS + ('PATH', 'CC', 'CXX', 'CPPFLAGS', 'CFLAGS', 'CXXFLAGS',
    'LDFLAGS', 'LD_LIBRARY_PATH', 'PKG_CONFIG_PATH', 'RUSTFLAGS', 'RUSTC', 'CARGO_HOME',
    'CARGO_TARGET_DIR', 'RUSTUP_HOME', 'CARGO_BUILD_JOBS', 'GCC_VERSION',
    'INDEX_CONTAINER_DIGEST', 'OMP_NUM_THREADS', 'OPENBLAS_NUM_THREADS',
    'GITHUB_SHA', 'GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT', 'RUNNER_TEMP')


def identity(path: Path) -> dict:
    return source.identity(path)


def parse_fai(path: Path) -> list[dict]:
    rows = []
    for line in path.read_text().splitlines():
        name, bases, offset, line_bases, line_width = line.split('\t')
        rows.append(dict(name=name, bases=int(bases), offset=int(offset),
                         line_bases=int(line_bases), line_width=int(line_width)))
    if not rows or len({row['name'] for row in rows}) != len(rows):
        raise ValueError('reference FAI names are empty or duplicated')
    if any(row['bases'] < 1000 or row['line_bases'] < 2 or row['line_width'] <= row['line_bases'] for row in rows):
        raise ValueError('reference FAI shape differs from the complete-reference profile')
    return rows


def verify_plain(path: Path, rows: list[dict]) -> None:
    index = -1
    count = 0
    with path.open('rb') as stream:
        while True:
            offset = stream.tell()
            line = stream.readline()
            if not line:
                break
            payload = line.rstrip(b'\r\n')
            if payload.startswith(b'>'):
                if index >= 0 and count != rows[index]['bases']:
                    raise ValueError('full reference sequence length differs from provider FAI')
                index += 1
                if index >= len(rows) or payload[1:].split()[0].decode('ascii') != rows[index]['name']:
                    raise ValueError('full reference names/order differ from provider FAI')
                count = 0
            elif payload:
                if index < 0:
                    raise ValueError('full reference sequence precedes a header')
                row = rows[index]
                expected = row['offset'] + count // row['line_bases'] * row['line_width'] + count % row['line_bases']
                if offset != expected or len(payload) > row['line_bases']:
                    raise ValueError('full reference byte layout differs from provider FAI')
                count += len(payload)
                if count > row['bases']:
                    raise ValueError('full reference sequence exceeds provider FAI')
    if index != len(rows) - 1 or count != rows[-1]['bases']:
        raise ValueError('full reference body is incomplete')


def make_regions(rows: list[dict]) -> list[str]:
    ends = []
    for row in rows:
        ends.append((ends[-1] if ends else 0) + row['bases'])
    regions = []
    for index in range(10000):
        position = (2 * index + 1) * ends[-1] // 20000
        contig = bisect.bisect_right(ends, position)
        row = rows[contig]
        start = min(position - (ends[contig - 1] if contig else 0), row['bases'] - 1000)
        regions.append(f'{row["name"]}:{start + 1}-{start + 1000}')
    for row in rows:
        length, width = row['bases'], row['line_bases']
        for start, end in ((1, 1), (1, 1000), (length - 999, length), (length, length), (width - 1, width + 2)):
            regions.append(f'{row["name"]}:{start}-{end}')
    return regions + [regions[0]] * 3


def prepare_plan(root: Path, rows: list[dict], references: dict, binaries: dict) -> list[dict]:
    region_file = root / 'data/regions.txt'
    region_file.write_text('\n'.join(make_regions(rows)) + '\n')
    rounds = [('qualification', ('source', 'native')), ('warmup', ('source', 'native'))]
    rounds += [(f'trial-{index + 1}', ('source', 'native') if index % 2 == 0 else ('native', 'source')) for index in range(5)]
    jobs = []
    for round_name, engines in rounds:
        for engine in engines:
            binary = binaries[engine]
            for representation, reference in references.items():
                target = root / 'outputs' / round_name / engine / representation
                target.mkdir(parents=True)
                fai = target / 'reference.fai'
                gzi = target / 'reference.gzi'
                indexes = ['--fai-idx', fai] + (['--gzi-idx', gzi] if representation == 'bgzf' else [])
                outputs = dict(fai=fai, **(dict(gzi=gzi) if representation == 'bgzf' else {}))
                build = [binary, 'faidx', *indexes, reference] if engine == 'source' else [binary, 'faidx', 'build', reference, *indexes]
                nodes = [('build', 1, build, outputs)]
                for family in ('bulk', 'chr1'):
                    for workers in (1, 4):
                        output = target / f'{family}-{workers}.fa'
                        if engine == 'source':
                            argv = [binary, 'faidx', '-@', str(workers - 1), *indexes,
                                    *(['-r', region_file] if family == 'bulk' else []), '-o', output, reference,
                                    *([] if family == 'bulk' else ['1'])]
                        else:
                            argv = [binary, 'faidx', 'query', reference, *indexes, '--threads', str(workers),
                                    *(['--region-file', region_file] if family == 'bulk' else ['1']), '--output', output]
                        nodes.append((f'{family}-{workers}', workers, argv, {family: output}))
                dictionary = target / 'reference.dict'
                nodes.append(('dict', 1, [binary, 'dict', '-u', URI, '-o', dictionary, reference], dict(dict=dictionary)))
                for stage, workers, argv, outputs in nodes:
                    jobs.append(dict(round=round_name, engine=engine, representation=representation,
                        stage=stage, workers=workers, primary=workers == 1,
                        argv=list(map(str, argv)), outputs=outputs))
    save(root / 'graph.json', [{**job, 'outputs': {key: str(path) for key, path in job['outputs'].items()}} for job in jobs])
    return jobs


def timed_call(root: Path, name: str, argv: list, *, cpus: list[int] | None = None, timeout: int = 1800) -> dict:
    target = root / 'calls' / name
    target.mkdir(parents=True)
    save(target / 'request.json', dict(argv=list(map(str, argv)), cwd=str(target),
         environment={key: os.environ.get(key) for key in ENV_KEYS},
         requested_cpus=cpus,
         started_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())))
    result = dict(status='failed', returncode=None, user_seconds=None, system_seconds=None, peak_rss_bytes=None)
    started = time.monotonic()
    def bind_cpus():
        os.sched_setaffinity(0, cpus)
    with (target / 'stdout.txt').open('xb') as stdout, (target / 'stderr.txt').open('xb') as stderr:
        try:
            process = subprocess.Popen(list(map(str, argv)), cwd=target, stdout=stdout, stderr=stderr,
                start_new_session=True, preexec_fn=bind_cpus if cpus is not None else None)
            deadline = started + timeout
            terminated = False
            while True:
                child, status, usage = os.wait4(process.pid, os.WNOHANG)
                if child:
                    process.returncode = os.waitstatus_to_exitcode(status)
                    result.update(status='timed_out' if terminated else ('returned' if process.returncode == 0 else 'failed'),
                        returncode=process.returncode, user_seconds=usage.ru_utime, system_seconds=usage.ru_stime,
                        peak_rss_bytes=usage.ru_maxrss * (1 if platform.system() == 'Darwin' else 1024))
                    break
                if time.monotonic() >= deadline:
                    try:
                        os.killpg(process.pid, signal.SIGKILL if terminated else signal.SIGTERM)
                    except ProcessLookupError:
                        pass
                    terminated = True
                    deadline = time.monotonic() + 10
                time.sleep(0.001)
        except OSError as error:
            result['error'] = str(error)
    result.update(wall_seconds=time.monotonic() - started, requested_cpus=cpus,
                  stdout=identity(target / 'stdout.txt'), stderr=identity(target / 'stderr.txt'),
                  finished_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
                  rss_scope='wait4 process peak; not controller/process-tree aggregate', poll_seconds=0.001)
    save(target / 'result.json', result)
    return result


def verify_outputs(jobs: list[dict], expected_fai: dict) -> dict:
    baseline = dict(fai=expected_fai)
    observed = {}
    for job in jobs:
        for kind, path in job['outputs'].items():
            actual = identity(path)
            if actual['bytes'] == 0:
                raise ValueError('empty required output: ' + str(path))
            if kind not in baseline:
                if job['round'] != 'qualification' or job['engine'] != 'source':
                    raise ValueError('missing original source qualification output')
                baseline[kind] = actual
            if actual != baseline[kind]:
                raise ValueError('complete output differs from source/provider: ' + str(path))
            observed[str(path)] = actual
    return dict(source_qualification=baseline, outputs=observed)


def distribution(values: list) -> dict:
    return dict(values=values, median=statistics.median(values), minimum=min(values), maximum=max(values))


def reduce_measurements(records: list[dict]) -> dict:
    measured = [row for row in records if row['round'].startswith('trial-')]
    if len(records) != 168 or len(measured) != 120 or any(row['status'] != 'returned' for row in records):
        raise ValueError('complete qualification/warmup/five-pair measurements are required')
    primary = {}
    nodes = {}
    for engine in ('source', 'native'):
        primary[engine] = distribution([sum(row['wall_seconds'] for row in measured
            if row['engine'] == engine and row['round'] == f'trial-{trial}' and row['primary']) for trial in range(1, 6)])
        for representation in ('plain', 'bgzf'):
            for stage in ('build', 'bulk-1', 'bulk-4', 'chr1-1', 'chr1-4', 'dict'):
                group = [row for row in measured if row['engine'] == engine and row['representation'] == representation and row['stage'] == stage]
                key = representation + '/' + stage
                nodes.setdefault(key, {})[engine] = {metric: distribution([row[metric] for row in group])
                    for metric in ('wall_seconds', 'user_seconds', 'system_seconds', 'peak_rss_bytes')}
    primary_ratio = primary['native']['median'] / primary['source']['median']
    regressions = {key: item['native']['wall_seconds']['median'] / item['source']['wall_seconds']['median']
                   for key, item in nodes.items() if not key.endswith('-4')}
    throughput = (primary_ratio <= 0.90 and primary['native']['maximum'] < primary['source']['minimum']
        and all(n < s for n, s in zip(primary['native']['values'], primary['source']['values']))
        and all(ratio <= 1.10 for ratio in regressions.values()))
    memory = primary_ratio <= 1.05
    for representation in ('plain', 'bgzf'):
        item = nodes[representation + '/chr1-1']
        native, original = item['native'], item['source']
        memory = (memory and native['peak_rss_bytes']['median'] <= 0.5 * original['peak_rss_bytes']['median']
            and native['peak_rss_bytes']['maximum'] < original['peak_rss_bytes']['minimum']
            and native['wall_seconds']['median'] <= 1.05 * original['wall_seconds']['median'])
    return dict(primary=primary, primary_wall_ratio=primary_ratio, primary_node_wall_ratios=regressions,
        nodes=nodes, exact_aggregate_memory_bytes=None,
        limitations='Process RSS only; no proven cold cache; qualification and warmup excluded; no prior13-lane evidence repair.',
        decision=dict(throughput_accepted=throughput, bounded_memory_accepted=memory,
                      performance_accepted=throughput or memory))


def admit_capacity(used: dict, free_bytes: int) -> dict:
    if any(used[key] > limit for key, limit in BUDGET.items()):
        raise RuntimeError('capacity: retained category exceeds its frozen envelope')
    remaining = sum(BUDGET.values()) - sum(used.values())
    if free_bytes < remaining + HEADROOM:
        raise RuntimeError('capacity: full remaining graph plus independent8GiB headroom does not fit')
    return dict(used=used, free_bytes=free_bytes, remaining_reservation=remaining,
                retained_cap=sum(BUDGET.values()), independent_headroom=HEADROOM)


def capacity(root: Path, name: str) -> None:
    used = dict(inputs=0, outputs=0, runtime=0, metadata=0)
    for path in root.rglob('*'):
        if not path.is_file() or path.is_symlink():
            continue
        first = path.relative_to(root).parts[0]
        category = {'data': 'inputs', 'outputs': 'outputs', 'runtime': 'runtime', 'runtime.tar': 'runtime'}.get(first, 'metadata')
        used[category] += path.stat().st_size
    free = shutil.disk_usage(root).free
    try:
        record = admit_capacity(used, free)
    except RuntimeError:
        save(root / 'metadata/capacity' / (name + '.json'),
             dict(status='capacity-hold', used=used, free_bytes=free, envelopes=BUDGET, independent_headroom=HEADROOM))
        raise
    save(root / 'metadata/capacity' / (name + '.json'), record)


def package_runtime(runtime: Path, carrier: Path) -> None:
    if carrier.parent.resolve() != runtime.parent.resolve():
        raise ValueError('runtime TAR must share the session parent')
    with tarfile.open(carrier, 'x', format=tarfile.PAX_FORMAT) as archive:
        archive.add(runtime, arcname='runtime', recursive=True)


def validate_workflow(path: Path) -> None:
    text = path.read_text()
    event = re.search(r'^on:\n(.*?)(?=^\S)', text, re.M | re.S)
    if event is None or event.group(1).strip() != 'workflow_dispatch:':
        raise ValueError('complete-reference workflow must be manual only')
    required = ['gcc:14.3.0-bookworm@' + source.IMAGE, 'runs-on: ubuntu-24.04',
        '--cpus 4 --memory 8g --memory-swap 8g', '${{ env.INDEX_RUNTIME_CARRIER }}',
        '${{ env.INDEX_SESSION }}/outputs', 'retention-days: 90', 'if: always()']
    if any(value not in text for value in required):
        raise ValueError('complete-reference workflow execution/retention routes differ')


def initialize() -> None:
    if (os.environ.get('GITHUB_ACTIONS') != 'true' or platform.system() != 'Linux'
            or os.environ.get('GITHUB_EVENT_NAME') != 'workflow_dispatch'):
        raise RuntimeError('real manual hosted initialization required')
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix='index-reference-real-', dir=runner))
    values = dict(INDEX_SESSION=str(root), INDEX_RUNTIME_CARRIER=str(root / 'runtime.tar'))
    for key in source.CACHE_KEYS:
        path = root / 'cache' / key
        path.mkdir(parents=True)
        values[key] = str(path)
    with Path(os.environ['GITHUB_ENV']).open('a') as stream:
        for key, value in values.items():
            stream.write(f'{key}={value}\n')
    save(root / 'request.json', dict(controller=os.environ['GITHUB_SHA'], owner=OWNER, owner_ci=OWNER_CI,
         run_id=os.environ['GITHUB_RUN_ID'], run_attempt=os.environ['GITHUB_RUN_ATTEMPT'],
         workflow_ref=os.environ.get('GITHUB_WORKFLOW_REF'), schema='rsomics.index.reference-real.v1',
         graph_rounds=['qualification', 'warmup', 'trial-1', 'trial-2', 'trial-3', 'trial-4', 'trial-5'],
         science_nodes=168, retained_envelopes=BUDGET, independent_headroom=HEADROOM))


def admit_owner_ci(root: Path) -> None:
    url = f'https://api.github.com/repos/omics-rust/rsomics-index/actions/runs/{OWNER_CI}'
    run_path, jobs_path = root / 'metadata/owner-ci.json', root / 'metadata/owner-ci-jobs.json'
    download(url, run_path, 2**20)
    download(url + '/jobs?per_page=100', jobs_path, 2**20)
    run, jobs = source.load(run_path), source.load(jobs_path)
    if (run.get('head_sha') != OWNER or run.get('conclusion') != 'success' or run.get('status') != 'completed'
            or run.get('repository', {}).get('full_name') != 'omics-rust/rsomics-index'):
        raise ValueError('exact-head successful native owner CI is absent')
    expected = {'linux-x86_64', 'linux-aarch64', 'macos-x86_64', 'macos-aarch64'}
    selected = {job['name']: job for job in jobs['jobs'] if job['name'] in expected}
    if set(selected) != expected or any(job['conclusion'] != 'success' or job['status'] != 'completed' for job in selected.values()):
        raise ValueError('actual four-native owner CI gate differs')


def prepare_native(root: Path) -> Path:
    native = root / 'runtime/native'
    native.mkdir()
    rust = native / 'rust'
    manifest = native / 'channel-rust-1.91.0.toml'
    download('https://static.rust-lang.org/dist/channel-rust-1.91.0.toml', manifest, 2**20)
    if sha256(manifest) != RUST_MANIFEST_SHA:
        raise ValueError('official pinned Rust1.91 manifest differs')
    for component, expected in RUST_COMPONENTS.items():
        prefix = component + '-1.91.0-x86_64-unknown-linux-gnu'
        archive = native / (prefix + '.tar.xz')
        download('https://static.rust-lang.org/dist/2025-10-30/' + archive.name, archive, 200 * 2**20)
        extracted = source.extract_release(archive, native / (component + '-original'), dict(
            size=archive.stat().st_size, sha256=expected, prefix=prefix, unpacked_limit=2**30))
        source.command(root, 'native-install-' + component,
            ['bash', extracted / 'install.sh', '--prefix=' + str(rust), '--disable-ldconfig'], cwd=extracted)
    for key, relative in (('CARGO_HOME', 'cargo-home'), ('CARGO_TARGET_DIR', 'target'), ('RUSTUP_HOME', 'rustup-home')):
        path = native / relative
        path.mkdir()
        os.environ[key] = str(path)
    os.environ.update(PATH=str(rust / 'bin') + ':' + os.environ['PATH'], RUSTC=str(rust / 'bin/rustc'),
        CARGO_BUILD_JOBS='2', RUSTFLAGS='', CPPFLAGS='', LDFLAGS='', PKG_CONFIG_PATH='',
        GIT_CONFIG_NOSYSTEM='1', GIT_CONFIG_GLOBAL='/dev/null', GIT_TERMINAL_PROMPT='0')
    owner = native / 'owner'
    source.command(root, 'native-clone', ['git', '-c', 'credential.helper=', 'clone', '--no-checkout',
        'https://github.com/omics-rust/rsomics-index.git', owner])
    source.command(root, 'native-checkout', ['git', 'checkout', '--detach', OWNER], cwd=owner)
    source.command(root, 'native-head', ['git', 'rev-parse', 'HEAD'], cwd=owner)
    source.command(root, 'native-clean-before', ['git', 'status', '--porcelain', '--untracked-files=all'], cwd=owner)
    if ((root / 'calls/native-head/stdout.txt').read_text().strip() != OWNER
            or (root / 'calls/native-clean-before/stdout.txt').read_bytes()
            or sha256(owner / 'Cargo.toml') != MANIFEST or sha256(owner / 'Cargo.lock') != LOCK):
        raise ValueError('native exact clean source/manifest/lock admission differs')
    source.command(root, 'native-files', ['git', 'ls-files', '-z'], cwd=owner)
    files = [os.fsdecode(name) for name in (root / 'calls/native-files/stdout.txt').read_bytes().split(b'\0') if name]
    before = {name: identity(owner / name) for name in files}
    save(native / 'owner-files.json', before)
    source.command(root, 'native-rustc-version', [rust / 'bin/rustc', '-vV'])
    source.command(root, 'native-cargo-version', [rust / 'bin/cargo', '-vV'])
    if not (root / 'calls/native-rustc-version/stdout.txt').read_text().startswith('rustc 1.91.0 '):
        raise ValueError('actual native Rust compiler differs')
    source.command(root, 'native-fetch', [rust / 'bin/cargo', 'fetch', '--locked'], cwd=owner)
    source.command(root, 'native-build', [rust / 'bin/cargo', 'build', '--locked', '--release', '--bin', 'rsomics-index'], cwd=owner)
    source.command(root, 'native-feature-tree', [rust / 'bin/cargo', 'tree', '--locked', '--offline', '-e', 'features'], cwd=owner)
    source.command(root, 'native-cargo-metadata', [rust / 'bin/cargo', 'metadata', '--locked', '--offline', '--format-version', '1'], cwd=owner)
    source.command(root, 'native-clean-after', ['git', 'status', '--porcelain', '--untracked-files=all'], cwd=owner)
    if before != {name: identity(owner / name) for name in files} or (root / 'calls/native-clean-after/stdout.txt').read_bytes():
        raise ValueError('native source changed during build')
    binary = Path(os.environ['CARGO_TARGET_DIR']) / 'release/rsomics-index'
    source.command(root, 'native-version', [binary, '--version'])
    if (root / 'calls/native-version/stdout.txt').read_text().strip() != 'rsomics-index 0.1.0':
        raise ValueError('actual native product version differs')
    provider = (root / 'runtime/prefix/lib/libdeflate.so.0').resolve(strict=True)
    elf = source.elf_admission(root, {'native': binary}, provider)
    metadata = source.load(root / 'calls/native-cargo-metadata/stdout.txt')
    packages = {package['name']: package for package in metadata['packages']}
    if (packages['libdeflate-sys']['version'] != '1.25.2' or packages['zlib-rs']['version'] != '0.6.7'
            or packages['flate2']['version'] != '1.1.9'):
        raise ValueError('native compression provider graph differs')
    provider_id = packages['libdeflate-sys']['id']
    provider_node = next(node for node in metadata['resolve']['nodes'] if node['id'] == provider_id)
    static_archives = list(Path(os.environ['CARGO_TARGET_DIR']).glob('release/build/libdeflate-sys-*/out/lib/libdeflate.a'))
    if 'dynamic' in provider_node['features'] or len(static_archives) != 1:
        raise ValueError('actual native static libdeflate provider admission differs')
    save(native / 'admission.json', dict(owner=OWNER, manifest=identity(owner / 'Cargo.toml'), lock=identity(owner / 'Cargo.lock'),
         executable=elf, compiler=identity(rust / 'bin/rustc'), cargo=identity(rust / 'bin/cargo'),
         providers='native static libdeflate-sys1.25.2 and Rust zlib-rs0.6.7; source dynamically loaded libdeflate1.25, not one shared DSO',
         native_static_archive=identity(static_archives[0]), native_provider_features=provider_node['features'],
         package_manifests={name: identity(Path(packages[name]['manifest_path'])) for name in ('libdeflate-sys', 'libdeflater', 'zlib-rs', 'flate2')}))
    return binary


def _decode_razf(original: Path, plain: Path, plain_cap: int) -> dict:
    size = original.stat().st_size
    header = bytes.fromhex('1f8b0804000000000003070052415a46018000')
    with original.open('rb') as stream:
        if stream.read(len(header)) != header or size < 47:
            raise ValueError('RAZF provider header differs')
        stream.seek(-16, 2)
        footer = stream.read(16)
        plain_bytes, member_end = struct.unpack('>QQ', footer)
        if plain_bytes > plain_cap:
            raise RuntimeError('capacity: complete decompression exceeds input envelope, not a FAI-derived upper bound')
        entries = plain_bytes // 32768
        bins = entries // 131072 + 1
        index_bytes = 4 + 8 * bins + 4 * entries
        if member_end <= len(header) + 8 or member_end + index_bytes + 16 != size:
            raise ValueError('RAZF index/footer geometry differs')
        stream.seek(member_end)
        index = stream.read(index_bytes)
        if struct.unpack_from('>I', index)[0] != entries:
            raise ValueError('RAZF index count differs from complete plain length')
        bin_offsets = struct.unpack_from(f'>{bins}Q', index, 4)
        previous = len(header)
        for number, (cell,) in enumerate(struct.iter_unpack('>I', index[4 + 8 * bins:])):
            offset = bin_offsets[number // 131072] + cell
            if (number % 131072 == 0 and cell != 0) or not previous < offset < member_end - 8:
                raise ValueError('RAZF index offsets differ')
            stream.seek(offset - 4)
            if stream.read(4) != b'\x00\x00\xff\xff':
                raise ValueError('RAZF index full-flush marker differs')
            previous = offset
        stream.seek(member_end - 8)
        trailer_crc, trailer_size = struct.unpack('<II', stream.read(8))
        stream.seek(len(header))
        decoder = zlib.decompressobj(-15)
        digest = hashlib.sha256()
        crc = count = 0
        remaining = member_end - 8 - len(header)
        block = b''
        with plain.open('xb') as output:
            while remaining or block:
                if not block:
                    block = stream.read(min(2**16, remaining))
                    if not block:
                        raise ValueError('RAZF DEFLATE stream is truncated')
                    remaining -= len(block)
                try:
                    chunk = decoder.decompress(block, 2**20)
                except zlib.error as error:
                    raise ValueError('RAZF DEFLATE stream is invalid') from error
                block = decoder.unconsumed_tail
                if decoder.unused_data:
                    raise ValueError('RAZF DEFLATE ends before its indexed trailer')
                count += len(chunk)
                if count > plain_cap:
                    raise RuntimeError('capacity: complete decompression exceeds input envelope, not a FAI-derived upper bound')
                if count > plain_bytes:
                    raise ValueError('RAZF complete plain length exceeds footer')
                output.write(chunk)
                digest.update(chunk)
                crc = zlib.crc32(chunk, crc)
        if (not decoder.eof or count != plain_bytes or crc != trailer_crc
                or count % 2**32 != trailer_size):
            raise ValueError('RAZF DEFLATE EOF, CRC or complete size differs from trailer/footer')
    return dict(flavor='RAZF: one gzip member plus validated random-access index',
        header_hex=header.hex(), gzip_member_bytes=member_end, gzip_crc32=f'{crc:08x}',
        gzip_isize=trailer_size, index_entries=entries, index_bins=bins, index_bytes=index_bytes,
        index_sha256=hashlib.sha256(index).hexdigest(), footer_hex=footer.hex(),
        trailing_index_and_footer_bytes=index_bytes + 16,
        plain=dict(bytes=count, sha256=digest.hexdigest()))


def acquire_reference(root: Path, bgzip: Path) -> tuple[dict, list[dict]]:
    data = root / 'data'
    data.mkdir()
    head = root / 'metadata/provider-head.json'
    import urllib.request
    request = urllib.request.Request(URI, method='HEAD', headers={'User-Agent': 'rsomics-index-reference-real'})
    with urllib.request.urlopen(request, timeout=120) as response:
        headers = dict(response.headers)
        save(head, dict(url=URI, final_url=response.url, status=response.status, headers=headers))
    if (int(headers.get('Content-Length', '0')) != PROVIDER_BYTES or headers.get('ETag') != PROVIDER_ETAG
            or headers.get('Last-Modified') != 'Tue, 12 Jul 2011 14:52:39 GMT'):
        raise ValueError('provider reference HEAD changed before acquisition')
    fai = data / 'provider.fai'
    download(BASE + 'human_g1k_v37.fasta.fai', fai, 2746)
    if identity(fai) != dict(bytes=2746, sha256=FAI_SHA):
        raise ValueError('provider FAI body changed before acquisition')
    rows = parse_fai(fai)
    if len(rows) != 84 or sum(row['bases'] for row in rows) != 3101804739 or rows[0]['name'] != '1' or rows[0]['bases'] != 249250621:
        raise ValueError('complete provider reference contig/base admission differs')
    download(BASE, root / 'metadata/provider-directory.html', 2**16)
    for name, (url, expected) in TERMS.items():
        path = root / 'metadata/terms' / (name + '.html')
        download(url, path, 2**20)
        if sha256(path) != expected:
            raise ValueError('admitted reference terms/README changed: ' + name)
    ensembl = 'https://asia.ensembl.org/info/about/legal/disclaimer.html'
    observation = dict(url=ensembl, admission='Root read official Ensembl primary web disclaimer before outcomes: project data unrestricted, third-party-rights caveat remains. Raw HTML is best effort, not a fabricated hash or mandatory extra gate.',
        ebi_igsr_originals_required=True, reference_only=True, personal_genotypes=False,
        citation='IGSR; 1000Genomes Project; original README Ensembl/NC_012920 attribution')
    try:
        observation['raw_receipt'] = download(ensembl, root / 'metadata/terms/ensembl.html', 2**20)
    except Exception:
        observation['raw_fetch_failure'] = traceback.format_exc()
    save(root / 'metadata/terms/admission.json', observation)
    capacity(root, 'before-provider-body')
    original = data / 'human_g1k_v37.fasta.gz'
    receipt = download(URI, original, PROVIDER_BYTES)
    if (receipt['bytes'] != PROVIDER_BYTES or receipt['headers'].get('ETag') != PROVIDER_ETAG
            or receipt['sha256'] != PROVIDER_SHA):
        raise ValueError('provider original body size/ETag/observed SHA differs')
    plain = data / 'reference.fa'
    plain_cap = BUDGET['inputs'] - PROVIDER_BYTES - BGZF_CAP - fai.stat().st_size - 2**20
    compression = _decode_razf(original, plain, plain_cap)
    if compression['plain'] != dict(bytes=PLAIN_BYTES, sha256=PLAIN_SHA):
        raise ValueError('complete provider plain body differs from observed full-body identity')
    verify_plain(plain, rows)
    capacity(root, 'after-complete-plain')
    bgzf = data / 'reference.fa.gz'
    source.command(root, 'prepare-complete-bgzf', [bgzip, '-@', '4', '-l', '6', '-o', bgzf, plain])
    if bgzf.stat().st_size > BGZF_CAP:
        raise RuntimeError('capacity: complete canonical BGZF exceeds its frozen reservation')
    digest = hashlib.sha256()
    with gzip.open(bgzf, 'rb') as stream:
        while chunk := stream.read(2**20):
            digest.update(chunk)
    if digest.hexdigest() != sha256(plain):
        raise ValueError('complete BGZF decompression differs from preserved plain input')
    members = 0
    with bgzf.open('rb') as stream:
        while header := stream.read(18):
            if len(header) != 18 or header[:4] != b'\x1f\x8b\x08\x04' or header[10:16] != b'\x06\x00BC\x02\x00':
                raise ValueError('prepared canonical BGZF structure differs')
            size = struct.unpack_from('<H', header, 16)[0] + 1
            if size < 26 or len(stream.read(size - 18)) != size - 18:
                raise ValueError('prepared BGZF member is truncated')
            members += 1
    if members * 16 + 8 > 4 * 2**20:
        raise RuntimeError('capacity: complete source BGZF GZI exceeds frozen allowance')
    with original.open('rb') as stream:
        original_prefix = stream.read(64).hex()
    save(root / 'inputs.json', dict(original=identity(original), plain=identity(plain), bgzf=identity(bgzf),
        provider_fai=identity(fai), contigs=len(rows), bases=sum(row['bases'] for row in rows),
        original_gzip_prefix_hex=original_prefix, complete_gzip_crc_verified=True,
        original_compression_flavor=compression['flavor'], original_compression=compression,
        bgzf_members=members, plain_cap_from_input_envelope=plain_cap,
        checksum_provenance='No advertised provider checksum; compressed/plain SHA frozen from full original failed run41262994994 before qualification, not independently provider-authenticated.'))
    return dict(plain=plain, bgzf=bgzf), rows


def execute() -> None:
    root = source.hosted_root()
    validate_workflow(ROOT / WORKFLOW)
    capacity(root, 'initial')
    source.command(root, 'controller-head', ['git', '-c', 'safe.directory=' + str(ROOT), '-C', ROOT, 'rev-parse', 'HEAD'])
    if (root / 'calls/controller-head/stdout.txt').read_text().strip() != os.environ['GITHUB_SHA']:
        raise ValueError('controller checkout differs from dispatched exact head')
    controller = root / 'controller'
    for relative in (WORKFLOW, 'scripts/index_reference_real_performance.py', 'scripts/index_reference_source.py',
        'scripts/plink_first_workflow_oracle.py', 'scripts/plink_first_workflow_inputs.py',
        'scripts/test_index_reference_real_performance.py',
        'docs/plans/2026-11-01-index-reference-workflow.md',
        'docs/plans/2026-11-01-index-reference-real-performance.md'):
        destination = controller / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(ROOT / relative, destination)
    groups = {name: (Path('/sys/fs/cgroup') / name).read_text().strip()
              for name in ('cpu.max', 'memory.max', 'memory.current', 'memory.peak') if (Path('/sys/fs/cgroup') / name).is_file()}
    quota, period = groups.get('cpu.max', '').split()
    if quota == 'max' or int(quota) != 4 * int(period) or int(groups.get('memory.max', '0')) != 8 * 2**30:
        raise ValueError('actual four-CPU/eight-GiB execution bounds differ')
    cpus = sorted(os.sched_getaffinity(0))[:4]
    if len(cpus) != 4:
        raise ValueError('four eligible logical CPUs are required')
    save(root / 'machine.json', dict(platform=platform.platform(), python=sys.version,
        python_executable=identity(Path(sys.executable).resolve()), cgroup=groups,
        cpuinfo=Path('/proc/cpuinfo').read_text(), host_visible_meminfo=Path('/proc/meminfo').read_text(),
        host_memory_is_not_cgroup_entitlement=True,
        affinity=sorted(os.sched_getaffinity(0)), selected_cpus=cpus,
        disk=shutil.disk_usage(root)._asdict(), device=root.stat().st_dev,
        clock=time.get_clock_info('monotonic').__dict__, environment={key: os.environ.get(key) for key in ENV_KEYS},
        cache='Uncontrolled; acquisition/hashing/preparation precede timed invocation. No proven cold cache.',
        exact_aggregate_memory_bytes=None))
    admit_owner_ci(root)
    upstream = source.prepare_runtime(root)
    native = prepare_native(root)
    binaries = dict(source=upstream['samtools'], native=native)
    before = {name: identity(path) for name, path in binaries.items()}
    save(root / 'runtime-files.json', source.tree_files(root / 'runtime'))
    package_runtime(root / 'runtime', root / 'runtime.tar')
    capacity(root, 'sealed-runtime-before-provider')
    references, rows = acquire_reference(root, upstream['bgzip'])
    frozen_inputs = {name: identity(path) for name, path in references.items()}
    jobs = prepare_plan(root, rows, references, binaries)
    save(root / 'frozen-workload.json', dict(owner=OWNER, input_files=frozen_inputs,
        region_file=identity(root / 'data/regions.txt'), regions=len(make_regions(rows)),
        calls=len(jobs), graph=identity(root / 'graph.json'), qualification_excluded=True,
        separate_warmup_excluded=True, paired_order=['SN', 'NS', 'SN', 'NS', 'SN'],
        throughput='median<=0.90;max(native)<min(source);all5paired wins;all primary node medians<=1.10',
        memory='both chr1-worker1 median processRSS<=0.50,worstN<bestS;those walls<=1.05;pipeline median<=1.05',
        exact_aggregate_memory_bytes=None))
    records = []
    for index, job in enumerate(jobs):
        name = '/'.join((job['round'], job['engine'], job['representation'], job['stage']))
        capacity(root, f'call-{index + 1:03d}')
        result = timed_call(root, name, job['argv'], cpus=cpus[:job['workers']])
        record = dict(**{**job, 'outputs': {key: str(path) for key, path in job['outputs'].items()}}, **result)
        records.append(record)
        save(root / 'measurements.json', records)
        if result['status'] != 'returned':
            raise RuntimeError('complete-reference process failed; original result retained: ' + name)
        for kind, path in job['outputs'].items():
            if kind == 'gzi' and path.stat().st_size > 4 * 2**20 or kind == 'dict' and path.stat().st_size > 64 * 2**10:
                raise RuntimeError('capacity: complete sidecar/dictionary exceeds frozen allowance')
        if (index + 1) % 24 == 0:
            save(root / 'correctness.json', verify_outputs(jobs[:index + 1], dict(bytes=2746, sha256=FAI_SHA)))
    if before != {name: identity(path) for name, path in binaries.items()} or frozen_inputs != {name: identity(path) for name, path in references.items()}:
        raise ValueError('runtime/reference changed during full-reference measurements')
    report = reduce_measurements(records)
    save(root / 'performance.json', report)
    capacity(root, 'completed')
    save(root / 'outcome.json', dict(status='complete-reference-comparison-completed', controller=os.environ['GITHUB_SHA'],
        owner=OWNER, qualification=True, separate_warmup=True, measured_pairs=5, science_nodes=len(records),
        correctness=True, performance_accepted=report['decision']['performance_accepted'], publication=False))
    if not report['decision']['performance_accepted']:
        raise RuntimeError('correct full-reference result did not meet frozen useful-performance criteria')


def finalize() -> None:
    root = source.hosted_root()
    runtime, carrier = root / 'runtime', root / 'runtime.tar'
    if runtime.is_dir() and not carrier.exists():
        entries = list(runtime.rglob('*'))
        estimated_tar = sum(path.stat().st_size + 4096 if path.is_file() and not path.is_symlink() else 4096 for path in entries) + 2**20
        retained = sum(path.stat().st_size for path in root.rglob('*') if path.is_file() and not path.is_symlink())
        if retained + estimated_tar > sum(BUDGET.values()) or shutil.disk_usage(root).free < estimated_tar + HEADROOM:
            raise RuntimeError('capacity: available failed-runtime originals cannot be packaged within total retained budget/headroom')
        package_runtime(runtime, carrier)
    save(root / 'artifact-manifest.json', dict(schema='rsomics.index.reference-real.v1',
        controller=os.environ['GITHUB_SHA'], owner=OWNER, files=source.tree_files(root, ('artifact-manifest.json',)),
        runtime_route='runtime.tar preserves modes/symlinks; raw runtime directory is not duplicated in upload',
        data_route='All regular data/outputs/calls/controller/metadata/root observations retained directly',
        outcome=source.load(root / 'outcome.json') if (root / 'outcome.json').exists() else None,
        retention='Public assembly reference and derived FASTA, no personal genotypes; provider/source attribution and terms retained.',
        limitations='No cold-cache/global-memory/hard-loss guarantee. No prior13-lane evidence reclassification or publication.'))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=('init', 'execute', 'finalize'))
    args = parser.parse_args()
    try:
        {'init': initialize, 'execute': execute, 'finalize': finalize}[args.action]()
    except Exception:
        root = os.environ.get('INDEX_SESSION')
        if root:
            (Path(root) / ('failure-' + args.action + '.txt')).write_text(traceback.format_exc())
            if args.action == 'execute' and not (Path(root) / 'outcome.json').exists():
                save(Path(root) / 'outcome.json', dict(status='complete-reference-execution-failed',
                    exception=traceback.format_exc(), performance_accepted=False, publication=False))
        traceback.print_exc()
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
