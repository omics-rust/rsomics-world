#!/usr/bin/env python3
"""Manual hosted popgen source capture with private, encrypted original retention."""

import argparse
from email.parser import BytesParser
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import platform
import re
import resource
import shutil
import signal
import stat
import subprocess
import sys
import tarfile
import tempfile
import time
import traceback
import urllib.request
import zipfile

import private_artifact as transport
import popgen_first_workflow_oracle as oracle


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = '.github/workflows/popgen-first-workflow-oracle.yml'
RUNTIME = ROOT / 'assets/popgen-first-runtime-20261101.json'
INPUT = ROOT / 'assets/popgen-first-input-20261101.json'
RECIPIENT = ROOT / 'assets/private-artifacts-20261101.recipient'
RECIPIENT_SHA = '8bb86dc97982fc7b64d7e33f7c3741e7f3d27a21690f48ddce9dada4a230680d'
RETAINED_LIMIT = 16 * 2**30
TAR_OVERHEAD = 2**30
MAX_MEMBERS = 100000
PROCESS_AS_LIMIT = 8 * 2**30
CONTAINER_MEMORY = 12 * 2**30
CACHE_NAMES = ('TMPDIR', 'XDG_CACHE_HOME', 'XDG_CONFIG_HOME', 'XDG_DATA_HOME', 'PIP_CACHE_DIR',
               'PYTHONUSERBASE', 'PYTHONPYCACHEPREFIX', 'CCACHE_DIR', 'CLANG_MODULE_CACHE_PATH')
SOURCE_ANCHORS = {
    'allel/stats/diversity.py': '77f1b6123b0f2c117c6520bca1c4c2dccb3b0bb69b837cfb428318d42a177cb9',
    'allel/stats/fst.py': 'ae8d8eb0514b10c49524f478be0829ed0a9d7fa0ef102bfb9db9543591ac621e',
    'allel/stats/sf.py': '0c4f73c54a65d53fbb3d5b9c0f3f030bc2925b7288be7379bb4cb3c64df515a3',
    'allel/stats/window.py': 'c5a9e5a25d59c8976dd11ef4ca5ce1a9bcd828e958ddca2cf9c4a9b18b6cff12',
    'allel/io/vcf_read.py': 'cf15f858bf4f2547aeaa7ebc891c276ce8456f5d3e7a1a9c04e4e885d9fb2492'}


def save(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    path.write_text(json.dumps(value, sort_keys=True, indent=2, allow_nan=False) + '\n')


def identity(path: Path) -> dict:
    return dict(bytes=path.stat().st_size, sha256=oracle.inputs.sha256(path))


def retention_plan() -> dict:
    final = transport.finalization_space(RETAINED_LIMIT, RETAINED_LIMIT + TAR_OVERHEAD)
    return dict(retained_limit_bytes=RETAINED_LIMIT, tar_bound_bytes=RETAINED_LIMIT + TAR_OVERHEAD,
                initial_free_bytes=RETAINED_LIMIT + final['required_free_bytes'], **final)


def new_session(parent: Path) -> dict[str, Path]:
    session = Path(tempfile.mkdtemp(prefix='popgen-source-', dir=parent))
    paths = dict(session=session, private=session / 'private', cipher=session / 'cipher')
    for name in ('private', 'cipher'):
        paths[name].mkdir(mode=0o700)
    return paths


def public_outputs(session: Path) -> dict[str, Path]:
    return {name: session / 'cipher' / name for name in ('payload.tar.age', 'receipt.json')}


def hosted_session() -> dict[str, Path]:
    if (os.environ.get('GITHUB_ACTIONS') != 'true' or os.environ.get('GITHUB_REPOSITORY') != 'omics-rust/rsomics-world'
            or platform.system() != 'Linux' or platform.machine() != 'x86_64'):
        raise RuntimeError('real omics-rust hosted Linux x86_64 execution required')
    if os.environ.get('POPGEN_CONTAINER_DIGEST') != oracle.IMAGE or os.environ.get('GCC_VERSION') != '14.3.0':
        raise ValueError('immutable GCC image declaration differs')
    if not re.fullmatch(r'[0-9a-f]{40}', os.environ['GITHUB_SHA']):
        raise ValueError('hosted head identity absent')
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    session = Path(os.environ['POPGEN_SESSION']).resolve(strict=True)
    if not session.is_relative_to(runner) or session == runner or session.stat().st_dev != runner.stat().st_dev:
        raise ValueError('private session escapes runner device')
    paths = dict(session=session, private=session / 'private', cipher=session / 'cipher')
    for path in paths.values():
        if path.is_symlink() or stat.S_IMODE(path.stat().st_mode) != 0o700:
            raise ValueError('private session directory is not0700')
    for name in CACHE_NAMES:
        path = Path(os.environ[name]).resolve(strict=True)
        if not path.is_relative_to(paths['private']) or path.stat().st_dev != session.stat().st_dev:
            raise ValueError('runtime cache escapes private tree: ' + name)
    return paths


def initialize() -> None:
    if os.environ.get('GITHUB_ACTIONS') != 'true' or platform.system() != 'Linux':
        raise RuntimeError('initialization is hosted only')
    paths = new_session(Path(os.environ['RUNNER_TEMP']).resolve(strict=True))
    os.environ['POPGEN_SESSION'] = str(paths['session'])
    values = {'POPGEN_SESSION': str(paths['session'])}
    for name in CACHE_NAMES:
        path = paths['private'] / 'cache' / name
        path.mkdir(parents=True, mode=0o700)
        values[name] = str(path)
    with Path(os.environ['GITHUB_ENV']).open('a') as output:
        for name, value in values.items():
            output.write(name + '=' + value + '\n')
    save(paths['private'] / 'initial-request.json', dict(head_sha=os.environ['GITHUB_SHA'],
         run_id=os.environ['GITHUB_RUN_ID'], run_attempt=os.environ['GITHUB_RUN_ATTEMPT'],
         workflow_ref=os.environ.get('GITHUB_WORKFLOW_REF'), capacity=retention_plan(),
         limitations='Finalization needs a surviving host/process and sufficient space; cancellation, timeout, encryption failure and hard loss are not zero-loss guarantees.'))


def retained_size(root: Path) -> int:
    total = 0
    for path in root.rglob('*'):
        try:
            metadata = path.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISREG(metadata.st_mode):
            total += metadata.st_size
    return total


def capacity(root: Path, *, initial: bool = False) -> dict:
    size = retained_size(root)
    plan = retention_plan()
    required = plan['initial_free_bytes'] if initial else max(0, RETAINED_LIMIT - size) + plan['required_free_bytes']
    free = shutil.disk_usage(root).free
    if size > RETAINED_LIMIT or free < required:
        raise RuntimeError('private retained/cipher/headroom capacity is insufficient')
    return dict(retained_bytes=size, free_bytes=free, required_free_bytes=required, plan=plan)


def recipient() -> Path:
    if RECIPIENT.is_symlink() or identity(RECIPIENT)['sha256'] != RECIPIENT_SHA:
        raise ValueError('fixed public recipient identity differs')
    body = RECIPIENT.read_bytes()
    if not body.startswith(b'age1pq1') or len(body.splitlines()) != 1:
        raise ValueError('fixed recipient grammar differs')
    return RECIPIENT


def prepare_transport(paths: dict[str, Path]) -> None:
    recipient()
    save(paths['private'] / 'capacity-admission.json', capacity(paths['private'], initial=True))
    tool = transport.prepare_tool('linux-amd64', paths['private'] / 'age-tool')
    probe = paths['cipher'] / 'recipient-probe.age'
    receipt = transport.encrypt(tool, RECIPIENT, probe,
        producer=[sys.executable, '-B', '-c', 'import sys;sys.stdout.buffer.write(b"rsomics-popgen-public-transport-probe-v1\\n")'],
        plaintext_limit=128, context={'repository': 'omics-rust/rsomics-world', 'product': 'rsomics-popgen', 'phase': 'transport-admission'})
    save(paths['private'] / 'transport-admission.json', dict(recipient_sha256=RECIPIENT_SHA,
         tool=json.loads((tool / 'admission.json').read_text()), probe=receipt,
         private_key='not requested, read, copied or available to this workflow'))


def cgroup_observation() -> dict:
    result = {}
    for name in ('memory.max', 'memory.current', 'memory.peak', 'memory.events', 'cpu.max'):
        path = Path('/sys/fs/cgroup') / name
        if path.is_file():
            result[name] = path.read_text().strip()
    result['scope'] = '/sys/fs/cgroup as mounted for this actual job container; memory.peak is lifetime, not a per-stage process maximum'
    return result


def machine_admission(root: Path) -> None:
    group = cgroup_observation()
    available = int(next(line.split()[1] for line in Path('/proc/meminfo').read_text().splitlines()
                         if line.startswith('MemAvailable:'))) * 1024
    quota, period = group.get('cpu.max', '').split()
    if quota == 'max' or int(quota) / int(period) != 4 or len(os.sched_getaffinity(0)) < 4:
        raise ValueError('actual four-CPU container quota/affinity was not admitted')
    if int(group.get('memory.max', '0')) != CONTAINER_MEMORY:
        raise ValueError('actual12GiB container memory limit was not admitted')
    available = min(available, CONTAINER_MEMORY - int(group['memory.current']))
    if available < PROCESS_AS_LIMIT:
        raise ValueError('less than8GiB actual available runtime memory')
    save(root / 'machine-admission.json', dict(cgroup=group, host_available_bytes=available,
         affinity=sorted(os.sched_getaffinity(0)), process_address_limit_bytes=PROCESS_AS_LIMIT,
         limits='Container cgroup memory is actual; sampled descendant RSS is observational and not a global/no-loss guarantee.'))


def tree_rss(pid: int) -> int:
    rows = {}
    for directory in Path('/proc').iterdir():
        if not directory.name.isdigit():
            continue
        try:
            fields = (directory / 'stat').read_text().rsplit(')', 1)[1].split()
            rows[int(directory.name)] = (int(fields[1]), int(fields[21]) * os.sysconf('SC_PAGE_SIZE'))
        except (FileNotFoundError, ProcessLookupError, PermissionError):
            continue
    selected = {pid}
    while True:
        extra = {child for child, (parent, _) in rows.items() if parent in selected}
        if extra <= selected:
            break
        selected |= extra
    return sum(rows.get(child, (0, 0))[1] for child in selected)


def child_limits() -> None:
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    resource.setrlimit(resource.RLIMIT_AS, (PROCESS_AS_LIMIT, PROCESS_AS_LIMIT))


def capture(root: Path, name: str, argv: list, *, cwd: Path | None = None,
            timeout: int = 1800, allowed: tuple[int, ...] = (0,)) -> Path:
    call = root / 'calls' / name
    call.mkdir(parents=True, exist_ok=False)
    save(call / 'request.json', dict(argv=list(map(str, argv)), cwd=str(cwd or root),
         environment={name: os.environ.get(name) for name in (*CACHE_NAMES, 'CC', 'CXX', 'CFLAGS', 'CXXFLAGS',
                     'CPPFLAGS', 'LDFLAGS', 'OMP_NUM_THREADS', 'OPENBLAS_NUM_THREADS', 'MKL_NUM_THREADS')},
         timeout_seconds=timeout, process_address_limit_bytes=PROCESS_AS_LIMIT))
    started = time.monotonic()
    process = None
    peak = 0
    try:
        with ((call / 'stdout.txt').open('xb') as stdout, (call / 'stderr.txt').open('xb') as stderr,
              (call / 'observations.jsonl').open('x') as observations):
            process = subprocess.Popen(list(map(str, argv)), cwd=cwd or root, stdout=stdout, stderr=stderr,
                                       start_new_session=True, preexec_fn=child_limits)
            while process.poll() is None:
                elapsed = time.monotonic() - started
                sampled = tree_rss(process.pid)
                peak = max(peak, sampled)
                observations.write(json.dumps(dict(seconds=elapsed, sampled_descendant_rss_bytes=sampled,
                                                   cgroup=cgroup_observation())) + '\n')
                observations.flush()
                capacity(root)
                if sampled > PROCESS_AS_LIMIT or elapsed > timeout:
                    raise RuntimeError('process exceeded sampled-RSS budget or stage timeout')
                try:
                    process.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    pass
        save(call / 'status.json', dict(returncode=process.returncode, elapsed_seconds=time.monotonic() - started,
             sampled_descendant_rss_peak_bytes=peak, sampled_rss_is_not_global=True))
        if process.returncode not in allowed:
            raise RuntimeError('source/runtime command failed: ' + name)
    except Exception:
        (call / 'failure.txt').write_text(traceback.format_exc())
        raise
    finally:
        if process is not None and process.poll() is None:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=10)
    return call


def acquire(root: Path, specification: dict, target: Path, *, maximum: int | None = None) -> dict:
    target.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    partial = Path(str(target) + '.partial')
    if target.exists() or target.is_symlink():
        raise FileExistsError('fresh original acquisition target required')
    limit = specification.get('size') or maximum
    if limit is None:
        raise ValueError('explicit acquisition byte bound required')
    request = urllib.request.Request(specification['url'], headers={'User-Agent': 'rsomics-popgen-source-admission/1'})
    receipt = dict(url=specification['url'], expected=specification, target=str(target), started_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()))
    try:
        with urllib.request.urlopen(request, timeout=120) as response, partial.open('xb') as output:
            receipt.update(status=response.status, final_url=response.url, headers=dict(response.headers))
            if response.status != 200 or (specification.get('etag') and response.headers.get('ETag') != specification['etag']):
                raise ValueError('provider body status/ETag differs from admitted original')
            size = 0
            while block := response.read(2**20):
                size += len(block)
                if size > limit:
                    raise ValueError('acquired body exceeds frozen byte bound')
                output.write(block)
                if size % (32 * 2**20) == 0:
                    capacity(root)
        actual = identity(partial)
        if specification.get('size') is not None and actual['bytes'] != specification['size']:
            raise ValueError('acquired body size differs')
        if specification.get('sha256') is not None and actual['sha256'] != specification['sha256']:
            raise ValueError('acquired body SHA256 differs')
        partial.rename(target)
        receipt.update(status='acquired-and-bound', identity=actual)
        return receipt
    except Exception:
        receipt.update(status='failed-original-retained', exception=traceback.format_exc())
        raise
    finally:
        save(Path(str(target) + '.receipt.json'), receipt)


def extract_archive(archive: Path, destination: Path, limit: int) -> None:
    destination.mkdir(mode=0o700)
    with tarfile.open(archive, 'r:gz') as stream:
        members = stream.getmembers()
        names = {member.name.rstrip('/') for member in members}
        if len(names) != len(members) or sum(member.size for member in members if member.isfile()) > limit:
            raise ValueError('runtime archive size/member identity differs')
        symlinks = {member.name.rstrip('/') for member in members if member.issym()}
        for member in members:
            name = PurePosixPath(member.name)
            if name.is_absolute() or '..' in name.parts or any('/'.join(name.parts[:i]) in symlinks for i in range(1, len(name.parts))):
                raise ValueError('runtime archive path escapes through traversal/symlink')
            target = destination / member.name
            if not (member.isdir() or member.isfile() or member.issym() or member.islnk()):
                raise ValueError('unsupported runtime archive member')
            if member.issym() and not (target.parent / member.linkname).resolve().is_relative_to(destination.resolve()):
                raise ValueError('runtime archive symlink escapes')
            if member.islnk() and (member.linkname not in names or member.linkname in symlinks):
                raise ValueError('runtime hardlink target differs')
        for member in members:
            target = destination / member.name
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True, mode=member.mode & 0o777)
            elif member.isfile():
                target.parent.mkdir(parents=True, exist_ok=True)
                with stream.extractfile(member) as source, target.open('xb') as output:
                    shutil.copyfileobj(source, output)
                target.chmod(member.mode & 0o777)
                os.utime(target, (member.mtime, member.mtime))
        for member in members:
            target = destination / member.name
            if member.issym():
                target.parent.mkdir(parents=True, exist_ok=True)
                target.symlink_to(member.linkname)
            elif member.islnk():
                os.link(destination / member.linkname, target)


def registry_image(root: Path, lock: dict) -> None:
    token = json.load(urllib.request.urlopen('https://auth.docker.io/token?service=registry.docker.io&scope=repository:library/gcc:pull', timeout=60))['token']
    for name, route, digest in [('manifest', 'manifests/', lock['image']['manifest_digest']),
                                ('config', 'blobs/', lock['image']['config_digest'])]:
        request = urllib.request.Request('https://registry-1.docker.io/v2/library/gcc/' + route + digest,
            headers={'Authorization': 'Bearer ' + token, 'Accept': 'application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json'})
        with urllib.request.urlopen(request, timeout=90) as response:
            body = response.read(2**20 + 1)
        if 'sha256:' + hashlib.sha256(body).hexdigest() != digest:
            raise ValueError('official image manifest/config identity differs')
        (root / ('image-' + name + '.json')).write_bytes(body)
    manifest = json.loads((root / 'image-manifest.json').read_text())
    config = json.loads((root / 'image-config.json').read_text())
    if manifest['config']['digest'] != lock['image']['config_digest'] or config['os'] != 'linux' or config['architecture'] != 'amd64':
        raise ValueError('official image runtime class differs')
    declared = dict(value.split('=', 1) for value in config['config']['Env'])
    if declared['GCC_VERSION'] != os.environ['GCC_VERSION']:
        raise ValueError('actual GCC environment differs from pinned image')


def verify_wheel(path: Path, package: dict) -> None:
    if identity(path) != dict(bytes=package['size'], sha256=package['sha256']):
        raise ValueError('wheel identity differs')
    with zipfile.ZipFile(path) as archive:
        body = archive.read(package['metadata_member'])
    metadata = BytesParser().parsebytes(body)
    if hashlib.sha256(body).hexdigest() != package['metadata_sha256'] or metadata.get_all('Requires-Dist', []) != package['requires_dist']:
        raise ValueError('wheel METADATA closure differs')


def runtime_probe(output: Path) -> None:
    import importlib.metadata
    import numpy
    import allel
    import dask
    lock = json.loads(RUNTIME.read_text())
    versions = {name: importlib.metadata.version(name) for name in lock['python_packages']}
    if versions != {name: row['version'] for name, row in lock['python_packages'].items()} or platform.python_version() != '3.12.10':
        raise ValueError('actual imported runtime versions differ')
    if any(not Path(module.__file__).resolve().is_relative_to(Path(sys.prefix).resolve()) for module in (numpy, allel, dask)):
        raise ValueError('scientific imports escape the admitted virtual environment')
    for relative, expected in SOURCE_ANCHORS.items():
        path = Path(allel.__file__).parent.parent / relative
        if identity(path)['sha256'] != expected:
            raise ValueError('actual installed scikit source differs: ' + relative)
    loaded = sorted({line.split()[-1] for line in Path('/proc/self/maps').read_text().splitlines()
                     if len(line.split()) >= 6 and line.split()[-1].startswith('/')})
    save(output, dict(python=sys.version, executable=str(Path(sys.executable).resolve()),
         executable_identity=identity(Path(sys.executable).resolve()), versions=versions,
         imported={'numpy': numpy.__file__, 'allel': allel.__file__, 'dask': dask.__file__},
         installed=[dict(name=row.metadata['Name'], version=row.version) for row in importlib.metadata.distributions()],
         loaded_files=loaded, loaded_file_identities={name: identity(Path(name).resolve()) for name in loaded if Path(name).is_file()}))
    numpy.show_config()


def library_closure(root: Path, executables: list[Path], loaded_file_identities: dict) -> None:
    entrypoints = {path.resolve() for path in executables}
    imported = {Path(name).resolve(): expected for name, expected in loaded_file_identities.items()}
    libraries = {}
    for i, executable in enumerate(sorted(entrypoints | imported.keys())):
        with executable.open('rb') as source:
            header = source.read(20)
        if header[:4] != b'\x7fELF':
            continue
        if header[4:6] != b'\x02\x01' or int.from_bytes(header[18:20], 'little') != 62:
            raise ValueError('actual loaded ELF is not little-endian64-bit x86_64')
        capture(root, f'elf-{i}', ['readelf', '--file-header', '--program-headers', '--dynamic', '--version-info', executable])
        paths = {str(executable)}
        if executable in entrypoints:
            call = capture(root, f'ldd-{i}', ['ldd', executable], allowed=(0, 1))
            text = (call / 'stdout.txt').read_text() + (call / 'stderr.txt').read_text()
            static = re.search(r'not a dynamic executable|statically linked', text)
            if 'not found' in text or (json.loads((call / 'status.json').read_text())['returncode'] and not static):
                raise ValueError('observed ELF dependency resolution failed')
            paths.update(re.findall(r'(?:=>\s*)?(/[^\s]+)', text))
        for token in sorted(paths):
            path = Path(token)
            if not path.is_file():
                continue
            actual = identity(path.resolve())
            if path.resolve() in imported and actual != imported[path.resolve()]:
                raise ValueError('actual imported library changed after the runtime probe')
            target = root / 'loaded-libraries' / (actual['sha256'] + '-' + path.name)
            target.parent.mkdir(exist_ok=True)
            if not target.exists():
                shutil.copy2(path.resolve(), target)
            libraries[token] = dict(realpath=str(path.resolve()), retained=str(target.relative_to(root)),
                                   actual_import=path.resolve() in imported, **actual)
    save(root / 'loaded-libraries.json', libraries)


def prepare_runtime(private: Path) -> tuple[Path, Path]:
    lock = json.loads(RUNTIME.read_text())
    oracle.validate_runtime_lock(lock)
    runtime = private / 'runtime'
    runtime.mkdir()
    save(runtime / 'runtime-lock.json', lock)
    registry_image(runtime, lock)
    capture(private, 'gcc-version', ['/usr/local/bin/gcc', '-dumpfullversion', '-dumpversion'])
    if (private / 'calls/gcc-version/stdout.txt').read_text().strip() != '14.3.0':
        raise ValueError('actual GCC compiler version differs')
    for name, argv in [('gxx', ['/usr/local/bin/g++', '-v']), ('make', ['make', '--version']),
                       ('perl', ['perl', '-V']), ('zlib', ['pkg-config', '--cflags', '--libs', 'zlib']),
                       ('packages', ['dpkg-query', '-W', '-f=${Package}\t${Version}\n'])]:
        capture(private, name, argv)
    if 'gcc version 14.3.0' not in (private / 'calls/gxx/stderr.txt').read_text():
        raise ValueError('actual C++ compiler version differs')
    downloads = runtime / 'downloads'
    downloads.mkdir()
    python_archive = downloads / lock['python']['filename']
    acquire(private, lock['python'], python_archive)
    extract_archive(python_archive, runtime / 'portable', 1024 * 2**20)
    base_python = runtime / 'portable/python/bin/python3'
    capture(private, 'python-version', [base_python, '-I', '-c', 'import platform,sys;assert platform.python_version()=="3.12.10";print(sys.version)'])
    capture(private, 'venv', [base_python, '-I', '-m', 'venv', '--symlinks', '--without-pip', runtime / 'venv'])
    python = runtime / 'venv/bin/python3'
    capture(private, 'venv-pip', [python, '-I', '-m', 'ensurepip', '--upgrade', '--default-pip'])
    wheels = runtime / 'wheels'
    wheels.mkdir()
    requirements = []
    for name, package in lock['python_packages'].items():
        path = wheels / package['filename']
        acquire(private, package, path)
        verify_wheel(path, package)
        requirements.append(name + '==' + package['version'] + ' --hash=sha256:' + package['sha256'])
    requirement_file = runtime / 'requirements.lock'
    requirement_file.write_text('\n'.join(requirements) + '\n')
    capture(private, 'wheel-install', [python, '-I', '-m', 'pip', '--isolated', 'install', '--no-index', '--no-deps',
            '--no-cache-dir', '--require-hashes', '--only-binary=:all:', '--find-links', wheels, '-r', requirement_file])
    capture(private, 'pip-check', [python, '-I', '-m', 'pip', '--isolated', 'check'])
    capture(private, 'pip-list', [python, '-I', '-m', 'pip', '--isolated', 'list', '--format=json'])
    authority = runtime / 'source-authority'
    authority.mkdir()
    for relative, digest in SOURCE_ANCHORS.items():
        acquire(private, dict(url='https://raw.githubusercontent.com/cggh/scikit-allel/' + oracle.COMMIT + '/' + relative, sha256=digest),
                authority / relative.replace('/', '_'), maximum=2**20)
    acquire(private, dict(url='https://raw.githubusercontent.com/cggh/scikit-allel/' + oracle.COMMIT + '/LICENSE',
                         size=1105, sha256='d9ad8672e1c07bd68fa0c84240b6e3e5304f2a8a814a059277ffcfabc983952d'),
            authority / 'scikit-allel-LICENSE')
    source = lock['vcftools']
    original = downloads / 'vcftools-0.1.17.tar.gz'
    acquire(private, source, original)
    extract_archive(original, runtime / 'vcftools-build', 64 * 2**20)
    build = runtime / 'vcftools-build/vcftools-0.1.17'
    if (build / '.tarball-version').read_text().strip() != source['version']:
        raise ValueError('vcftools original release version differs')
    for name, digest in source['source_anchor'].items():
        if identity(build / name)['sha256'] != digest:
            raise ValueError('vcftools release source anchor differs')
    save(runtime / 'vcftools-original-members.json', archive_inventory(build))
    save(runtime / 'vcftools-install-layout.json', dict(
         original_frozen_parameter='--with-pmdir={runtime}/vcftools/perl',
         corrected_frozen_parameter='--with-pmdir=perl',
         original_makefile_member='vcftools-0.1.17/src/perl/Makefile.am',
         source_line='pmdir = $(exec_prefix)/$(PMDIR)',
         reason='PMDIR is prefix-relative in the original upstream Makefile; this changes only private Perl installation layout, not source or science.'))
    for i, argv in enumerate(source['build_argv']):
        capture(private, 'vcftools-build-' + str(i), [argument.format(runtime=runtime) for argument in argv], cwd=build)
    for name, digest in source['source_anchor'].items():
        if identity(build / name)['sha256'] != digest:
            raise ValueError('vcftools scientific source anchor changed during build')
    executable = runtime / 'vcftools/bin/vcftools'
    call = capture(private, 'vcftools-version', [executable, '--version'])
    if not re.search(r'\b0\.1\.17\b', (call / 'stdout.txt').read_text() + (call / 'stderr.txt').read_text()):
        raise ValueError('installed vcftools executable version differs')
    capture(private, 'runtime-import', [python, '-B', Path(__file__), 'runtime-probe', '--output', runtime / 'actual-import.json'])
    observed = json.loads((runtime / 'actual-import.json').read_text())
    library_closure(private, [python, executable, Path('/usr/local/bin/gcc'), Path('/usr/local/bin/g++')],
                    observed['loaded_file_identities'])
    save(runtime / 'installed-runtime-members.json', archive_inventory(runtime))
    save(runtime / 'admitted-runtime.json', dict(python=str(python), python_identity=identity(python),
         vcftools=str(executable), vcftools_identity=identity(executable), imported=observed,
         runtime_lock=identity(RUNTIME), scientific_profile_unchanged=True,
         image_manifest=identity(runtime / 'image-manifest.json'), image_config=identity(runtime / 'image-config.json')))
    return python, executable


def source_run(private: Path, name: str, python: Path, vcftools: Path, inputs: Path, specification: Path) -> None:
    admission = private / (name + '-admission.json')
    capture(private, name + '-admit', [python, '-B', ROOT / 'scripts/popgen_first_workflow_inputs.py', 'admit',
            inputs, specification, admission, '--legacy-output', private / (name + '-legacy-inputs')], timeout=3600)
    prefix = [python, '-B', ROOT / 'scripts/popgen_first_workflow_oracle.py']
    options = ['--inputs', inputs, '--specification', specification, '--admission', admission]
    capture(private, name + '-verify', prefix + ['verify-admission', *options], timeout=3600)
    capture(private, name + '-source', prefix + ['run', *options, '--runtime', RUNTIME,
            '--output', private / (name + '-source'), '--vcftools', vcftools], timeout=3600)


def require_casebook(private: Path) -> None:
    result_path = private / 'casebook-source/result.json'
    if not result_path.is_file():
        raise ValueError('complete actual casebook success is required before real acquisition')
    result = json.loads(result_path.read_text())
    status = json.loads((private / 'calls/casebook-source/status.json').read_text())
    if status['returncode'] != 0 or not result.get('independent_casebook') or len(result['vcftools']['calls']) != 5:
        raise ValueError('casebook/legacy whole workflow did not pass')
    if any(call['returncode'] != 0 for call in result['vcftools']['calls']):
        raise ValueError('casebook legacy command failed')
    manifest = json.loads((private / 'casebook-source/output-manifest.json').read_text())
    for name, expected in manifest.items():
        actual = identity(private / 'casebook-source' / name)
        if actual != dict(bytes=expected['size'], sha256=expected['sha256']):
            raise ValueError('casebook output manifest differs')


def execute(paths: dict[str, Path]) -> None:
    private = paths['private']
    admission = json.loads((private / 'transport-admission.json').read_text())
    if admission['recipient_sha256'] != RECIPIENT_SHA:
        raise ValueError('early recipient/tool admission absent')
    validate_workflow(ROOT / WORKFLOW)
    machine_admission(private)
    controller = private / 'controller'
    for relative in [WORKFLOW, 'scripts/popgen_hosted_source.py', 'scripts/popgen_first_workflow_oracle.py',
                     'scripts/popgen_first_workflow_inputs.py', 'scripts/private_artifact.py',
                     'scripts/test_popgen_first_workflow_oracle.py', 'docs/plans/2026-11-01-popgen-source-preflight.md',
                     'assets/private-artifact-tools-20261101.json', 'assets/private-artifacts-20261101.recipient',
                     str(RUNTIME.relative_to(ROOT)), str(INPUT.relative_to(ROOT))]:
        destination = controller / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(ROOT / relative, destination)
    python, vcftools = prepare_runtime(private)
    capture(private, 'casebook-inputs', [python, '-B', ROOT / 'scripts/popgen_first_workflow_inputs.py',
                                      'casebook', private / 'casebook'])
    source_run(private, 'casebook', python, vcftools, private / 'casebook', private / 'casebook/specification.json')
    require_casebook(private)
    spec = json.loads(INPUT.read_text())
    real = private / 'real'
    real.mkdir()
    for entry in spec['files'].values():
        acquire(private, entry, real / entry['filename'])
    documents = private / 'provider-documentation'
    documents.mkdir()
    for name in ('mask_codes', 'ancestral_annotation', 'terms'):
        acquire(private, dict(url=spec['documentation'][name]), documents / (name + '.original'), maximum=2**20)
    source_run(private, 'real', python, vcftools, real, INPUT)
    save(private / 'complete-source-acceptance.json', dict(status='complete_real_and_casebook_source_returned',
         head_sha=os.environ['GITHUB_SHA'], original_specification=identity(INPUT),
         real_result=identity(private / 'real-source/result.json'), casebook_result=identity(private / 'casebook-source/result.json'),
         release=False, native_implementation=False, performance_claim=False))


def archive_inventory(root: Path) -> list[dict]:
    root = root.resolve(strict=True)
    result = []
    for path in sorted(root.rglob('*')):
        metadata = path.lstat()
        name = path.relative_to(root).as_posix()
        if len(name.encode()) > 2048:
            raise ValueError('private archive member name exceeds admitted bound')
        row = dict(name=name, mode=stat.S_IMODE(metadata.st_mode), mtime=int(metadata.st_mtime))
        if path.is_symlink():
            target = os.readlink(path)
            if len(target.encode()) > 2048 or not path.resolve().is_relative_to(root):
                raise ValueError('private archive symlink escapes original tree')
            row.update(type='symlink', target=target, bytes=0)
        elif stat.S_ISREG(metadata.st_mode):
            row.update(type='file', **identity(path))
        elif stat.S_ISDIR(metadata.st_mode):
            row.update(type='directory', bytes=0)
        else:
            raise ValueError('unsupported private evidence file type')
        result.append(row)
    if len(result) > MAX_MEMBERS or sum(row['bytes'] for row in result) > RETAINED_LIMIT:
        raise ValueError('private evidence exceeds retained member/body admission')
    return result


def write_tar(root: Path, members: list[dict], output) -> None:
    with tarfile.open(fileobj=output, mode='w|', format=tarfile.PAX_FORMAT) as archive:
        for row in members:
            path = root / row['name']
            member = tarfile.TarInfo(row['name'])
            member.mode, member.mtime = row['mode'], row['mtime']
            if row['type'] == 'directory':
                member.type = tarfile.DIRTYPE
                archive.addfile(member)
            elif row['type'] == 'symlink':
                if os.readlink(path) != row['target']:
                    raise ValueError('private symlink changed after finalization')
                member.type, member.linkname = tarfile.SYMTYPE, row['target']
                archive.addfile(member)
            else:
                if identity(path) != {key: row[key] for key in ('bytes', 'sha256')}:
                    raise ValueError('private evidence changed after finalization')
                member.size = row['bytes']
                with path.open('rb') as source:
                    archive.addfile(member, source)


def finalize(paths: dict[str, Path]) -> None:
    private = paths['private']
    recipient()
    tool = transport.prepare_tool('linux-amd64', private / 'age-tool')
    members = archive_inventory(private)
    save(private / 'archive-members.json', dict(members=members, scope='all quiescent private originals/logs/runtime/output; excludes this self-index and transport-time diagnostics',
         limitation='transport stderr remains private in sibling cipher directory; failed sealing, cancellation or hard host loss has no plaintext fallback or zero-loss guarantee'))
    members = archive_inventory(private)
    manifest = paths['session'] / 'sealed-members.json'
    save(manifest, members)
    tar_bound = sum(row['bytes'] for row in members) + len(members) * 8192 + 10240
    space = transport.finalization_space(sum(row['bytes'] for row in members), tar_bound)
    if shutil.disk_usage(private).free < space['required_free_bytes']:
        raise RuntimeError('actual streamed TAR/cipher/headroom finalization capacity unavailable')
    outputs = public_outputs(paths['session'])
    context = dict(repository='omics-rust/rsomics-world', product='rsomics-popgen', head_sha=os.environ['GITHUB_SHA'],
                   run_id=int(os.environ['GITHUB_RUN_ID']), run_attempt=int(os.environ['GITHUB_RUN_ATTEMPT']),
                   workflow_path=WORKFLOW, phase='source-originals')
    receipt = transport.encrypt(tool, RECIPIENT, outputs['payload.tar.age'], producer=[sys.executable, '-B', Path(__file__),
        'tar', '--root', private, '--members', manifest], plaintext_limit=tar_bound, context=context)
    if receipt['recipient_sha256'] != RECIPIENT_SHA:
        raise ValueError('transport receipt recipient identity differs')
    save(outputs['receipt.json'], receipt)


def validate_workflow(path: Path) -> None:
    text = path.read_text()
    events = re.search(r'^on:\n((?:  .*\n)+)', text, re.MULTILINE)
    if events is None or events.group(1) != '  workflow_dispatch:\n':
        raise ValueError('manual-only source workflow event differs')
    if 'continue-on-error:' in text or 'secrets.' in text:
        raise ValueError('source workflow must not mask failures or request private keys')
    images = [line.strip() for line in text.splitlines() if line.strip().startswith('image:')]
    if len(images) != 1 or oracle.IMAGE not in images[0] or 'runs-on: ubuntu-24.04' not in text:
        raise ValueError('source workflow image/host class differs')
    upload = text.split('uses: actions/upload-artifact@v4', 1)
    if len(upload) != 2 or 'payload.tar.age' not in upload[1] or 'receipt.json' not in upload[1]:
        raise ValueError('ciphertext/public-receipt-only upload absent')
    block = re.search(r'          path: \|\n((?:            .*\n)+)', upload[1])
    expected = ['${{ env.POPGEN_SESSION }}/cipher/payload.tar.age', '${{ env.POPGEN_SESSION }}/cipher/receipt.json']
    if block is None or [line.strip() for line in block.group(1).splitlines()] != expected:
        raise ValueError('plaintext or wildcard upload is forbidden')


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['init', 'transport', 'execute', 'finalize', 'runtime-probe', 'tar'])
    parser.add_argument('--output', type=Path)
    parser.add_argument('--root', type=Path)
    parser.add_argument('--members', type=Path)
    args = parser.parse_args()
    if args.action == 'init':
        try:
            initialize()
            return 0
        except Exception:
            if os.environ.get('POPGEN_SESSION'):
                (Path(os.environ['POPGEN_SESSION']) / 'init-failure.txt').write_text(traceback.format_exc())
            print('private source initialization failed before cohort acquisition', file=sys.stderr)
            return 1
    paths = hosted_session()
    try:
        if args.action == 'transport':
            prepare_transport(paths)
        elif args.action == 'execute':
            execute(paths)
        elif args.action == 'finalize':
            finalize(paths)
        elif args.action == 'runtime-probe':
            runtime_probe(args.output)
        else:
            if args.root.resolve() != paths['private'] or not args.members.resolve().is_relative_to(paths['session']):
                raise ValueError('TAR producer paths escape private session')
            write_tar(args.root, json.loads(args.members.read_text()), sys.stdout.buffer)
        return 0
    except Exception:
        destination = paths['session'] if args.action in ('finalize', 'tar') else paths['private']
        (destination / (args.action + '-failure.txt')).write_text(traceback.format_exc())
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
