#!/usr/bin/env python3
"""Prepare and capture the hosted, current full-four peak source workflow."""

import argparse
import csv
import hashlib
import importlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import signal
import subprocess
import sys
import tarfile
import time
import traceback
import urllib.error
import urllib.request

import signal_first_workflow_oracle as shared


IMAGE = 'sha256:81bf4c3e9f1b08b0ada0a45ef16ef51d03dc91b0b06cda4b408d667cb8c2c645'
THREADS = ('OMP_NUM_THREADS', 'OPENBLAS_NUM_THREADS', 'MKL_NUM_THREADS',
           'BLIS_NUM_THREADS', 'VECLIB_MAXIMUM_THREADS', 'NUMEXPR_NUM_THREADS')
SAMPLES = ('T1', 'T2', 'C1', 'C2')
ROOT = Path(__file__).resolve().parent


def sha256(path):
    return shared.sha256(path)


def save(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('w') as stream:
        json.dump(value, stream, indent=2, sort_keys=True, allow_nan=False)
        stream.write('\n')


def hosted(output):
    if os.environ.get('GITHUB_ACTIONS') != 'true' or platform.system() != 'Linux' or platform.machine() != 'x86_64':
        raise RuntimeError('installation/scientific capture is hosted Linux x86_64 only')
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    target = Path(output).resolve()
    if not target.is_relative_to(runner):
        raise ValueError('hosted output escapes RUNNER_TEMP')
    for name in ('EVIDENCE', 'TMPDIR', 'XDG_CACHE_HOME', 'PIP_CACHE_DIR', 'PYTHONUSERBASE',
                 'MPLCONFIGDIR', 'R_LIBS_USER', 'R_USER_CACHE_DIR'):
        path = Path(os.environ[name]).resolve(strict=True)
        if not path.is_relative_to(runner) or path.stat().st_dev != runner.stat().st_dev:
            raise ValueError('hosted write/cache root escapes RUNNER_TEMP: ' + name)
    if os.environ.get('PEAK_CONTAINER_DIGEST') != IMAGE or os.environ.get('BIOCONDUCTOR_VERSION') != '3.23':
        raise RuntimeError('immutable Bioconductor image identity differs')
    if any(os.environ.get(name) != '1' for name in THREADS):
        raise RuntimeError('unfrozen numerical thread budget')
    return target


def capture(target, argv, cwd=None, env=None):
    target = Path(target)
    target.mkdir(parents=True, exist_ok=True)
    cwd = Path(cwd or target).resolve()
    request = dict(argv=list(map(str, argv)), cwd=str(cwd),
                   env={name: (env or os.environ).get(name) for name in
                        (*THREADS, 'PATH', 'TMPDIR', 'R_LIBS_USER', 'R_LIBS_SITE',
                         'R_USER_CACHE_DIR', 'R_ENVIRON_USER', 'R_PROFILE_USER',
                         'CC', 'CXX', 'LDSHARED', 'LDCXXSHARED', 'LD', 'AR',
                         'CFLAGS', 'CXXFLAGS', 'CPPFLAGS', 'LDFLAGS',
                         'PEAK_R_MODE', 'PEAK_SEACR_SOURCE_R', 'PEAK_SEACR_CAPTURE')},
                   started_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()))
    save(target / 'request.json', request)
    with (target / 'stdout.txt').open('wb') as stdout, (target / 'stderr.txt').open('wb') as stderr:
        try:
            process = subprocess.Popen(list(map(str, argv)), cwd=cwd, env=env, stdout=stdout, stderr=stderr,
                                       start_new_session=True)
            code = process.wait(timeout=3600)
            record = dict(status='returned' if code == 0 else 'failed', returncode=code)
        except (subprocess.TimeoutExpired, OSError) as error:
            if isinstance(error, subprocess.TimeoutExpired):
                os.killpg(process.pid, signal.SIGTERM)
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
            record = dict(status='failed', returncode=None, exception=type(error).__name__, message=str(error))
    record.update(
                  finished_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
                  stdout_sha256=sha256(target / 'stdout.txt'), stderr_sha256=sha256(target / 'stderr.txt'))
    save(target / 'process.json', record)
    return record


def require(target, argv, cwd=None, env=None):
    result = capture(target, argv, cwd, env)
    if result['status'] != 'returned':
        raise RuntimeError('required process failed: ' + str(target))
    return result


def extract_archive(archive_path, output):
    output = Path(output)
    output.mkdir()
    with tarfile.open(archive_path, 'r:gz') as archive:
        archive.extractall(output, filter='data')


def read_dcf(text):
    values = {}
    field = None
    for line in text.splitlines():
        if not line.strip():
            continue
        if line[0].isspace():
            values[field] += ' ' + line.strip()
        else:
            field, value = line.split(':', 1)
            values[field] = value.strip()
    return values


def r_admission_mode(row):
    return 'immutable-image' if row.get('Priority') in ('base', 'recommended') else 'source'


def version(value):
    numbers = tuple(map(int, re.findall(r'\d+', value)))
    return numbers + (0,) * max(0, 8 - len(numbers))


def dependency_rows(description):
    for field in ('Depends', 'Imports', 'LinkingTo'):
        for value in description.get(field, '').split(','):
            value = value.strip()
            if not value or value == 'NA':
                continue
            match = re.fullmatch(r'([^\s(]+)\s*(?:\(\s*(>=|<=|==|!=|>|<)\s*([^\s)]+)\s*\))?', value)
            if not match:
                raise ValueError('unrecognized R dependency: ' + value)
            yield match.groups()


def check_constraints(selected, descriptions):
    for package, description in descriptions.items():
        for name, operator, bound in dependency_rows(description):
            if name not in selected:
                raise ValueError('missing dependency: ' + package + ' -> ' + name)
            if operator:
                left, right = version(selected[name]), version(bound)
                valid = {'>=': left >= right, '<=': left <= right, '==': left == right,
                         '!=': left != right, '>': left > right, '<': left < right}[operator]
                if not valid:
                    raise ValueError('incompatible dependency: ' + package + ' -> ' + name +
                                     ' ' + operator + ' ' + bound + '; actual ' + selected[name])


def install_order(descriptions, installed):
    remaining = set(descriptions) - set(installed)
    ordered = []
    while remaining:
        ready = sorted(name for name in remaining if
                       {dep[0] for dep in dependency_rows(descriptions[name])} <= set(installed) | set(ordered))
        if not ready:
            raise ValueError('R install dependency cycle or unresolved dependencies: ' + ','.join(sorted(remaining)))
        ordered.extend(ready)
        remaining.difference_update(ready)
    return ordered


def download_r(row, destination):
    urls = [row['source_url']]
    if row['repository'] == 'cran':
        urls.append('https://cran.r-project.org/src/contrib/Archive/' + row['package'] + '/' +
                    row['package'] + '_' + row['version'] + '.tar.gz')
    for index, url in enumerate(urls):
        request = urllib.request.Request(url, headers={'User-Agent': 'rsomics-peak-source-witness'})
        try:
            response = urllib.request.urlopen(request, timeout=120)
        except urllib.error.HTTPError as error:
            if error.code == 404 and index + 1 < len(urls):
                continue
            raise
        with response, destination.open('xb') as stream:
            shutil.copyfileobj(response, stream)
        actual_md5 = hashlib.md5(destination.read_bytes()).hexdigest()
        if actual_md5 != row['official_index_md5']:
            raise ValueError('captured R index MD5 differs: ' + row['package'])
        with tarfile.open(destination, 'r:gz') as archive:
            members = [m for m in archive.getmembers() if m.isfile() and m.name.count('/') == 1 and
                       m.name.endswith('/DESCRIPTION')]
            if len(members) != 1:
                raise ValueError('R DESCRIPTION identity is ambiguous')
            description = read_dcf(archive.extractfile(members[0]).read().decode())
        if description['Package'] != row['package'] or description['Version'] != row['version']:
            raise ValueError('R source package/version differs: ' + row['package'])
        return dict(row, actual_url=url, sha256=sha256(destination), size=destination.stat().st_size,
                    description=description, archive=str(destination))
    raise RuntimeError('no exact captured R source: ' + row['package'])


def files_identity(root):
    root = Path(root)
    return {str(path.relative_to(root)): ({'symlink': os.readlink(path)} if path.is_symlink() else
            {'sha256': sha256(path), 'size': path.stat().st_size})
            for path in sorted(root.rglob('*')) if path.is_file() or path.is_symlink()}


def r_inventory(runtime, name):
    destination = runtime / name
    require(runtime / 'calls' / name, ['Rscript', '--vanilla', ROOT / 'peak_first_workflow_oracle.R',
                                     'inventory', destination])
    with (destination / 'installed.tsv').open() as stream:
        return list(csv.DictReader(stream, delimiter='\t'))


def verify_registry_image(lock, runtime):
    token_url = ('https://auth.docker.io/token?service=registry.docker.io&'
                 'scope=repository:bioconductor/bioconductor_docker:pull')
    with urllib.request.urlopen(token_url, timeout=90) as response:
        token = json.load(response)['token']
    base = 'https://registry-1.docker.io/v2/bioconductor/bioconductor_docker/'
    request = urllib.request.Request(base + 'manifests/' + IMAGE,
        headers={'Authorization': 'Bearer ' + token, 'Accept': 'application/vnd.oci.image.manifest.v1+json'})
    with urllib.request.urlopen(request, timeout=90) as response:
        manifest = response.read()
    if hashlib.sha256(manifest).hexdigest() != IMAGE.split(':')[1]:
        raise ValueError('official immutable image manifest differs')
    config_digest = json.loads(manifest)['config']['digest']
    request = urllib.request.Request(base + 'blobs/' + config_digest,
                                    headers={'Authorization': 'Bearer ' + token})
    with urllib.request.urlopen(request, timeout=90) as response:
        config = response.read()
    if hashlib.sha256(config).hexdigest() != lock['r_base_container']['config_sha256']:
        raise ValueError('official immutable image configuration differs')
    declared = dict(item.split('=', 1) for item in json.loads(config)['config']['Env'])
    for name in ('R_VERSION', 'BIOCONDUCTOR_VERSION', 'BIOCONDUCTOR_DOCKER_VERSION', 'TARGETARCH'):
        if os.environ.get(name) != declared.get(name):
            raise ValueError('actual container environment differs from pinned configuration: ' + name)
    (runtime / 'official-image-manifest.json').write_bytes(manifest)
    (runtime / 'official-image-config.json').write_bytes(config)


def acquire(lock_path, output):
    runtime = hosted(output)
    runtime.mkdir(parents=True)
    lock = json.loads(Path(lock_path).read_text())
    save(runtime / 'preparation-packet.json', lock)
    verify_registry_image(lock, runtime)
    require(runtime / 'calls/image-R-gate', ['Rscript', '--vanilla', '-e',
        'stopifnot(as.character(getRversion())=="4.6.1", Sys.getenv("BIOCONDUCTOR_VERSION")=="3.23"); print(sessionInfo()); print(Sys.getenv(c("BIOCONDUCTOR_VERSION","R_VERSION","TARGETARCH")))'])
    save(runtime / 'image-identity.json', dict(expected_digest=IMAGE,
         declared_digest=os.environ['PEAK_CONTAINER_DIGEST'], R_VERSION=os.environ.get('R_VERSION'),
         BIOCONDUCTOR_VERSION=os.environ['BIOCONDUCTOR_VERSION'],
         controller_sha256=sha256(__file__), shared_controller_sha256=sha256(shared.__file__)))
    downloads = runtime / 'downloads'
    downloads.mkdir()
    shared.download(lock['python'], downloads / lock['python']['filename'])
    extract_archive(downloads / lock['python']['filename'], runtime / 'portable')
    shutil.move(runtime / 'portable/python', runtime / 'python')
    for item in lock['python_packages'].values():
        shared.download(item, downloads / item['filename'])
    shared.download(lock['bedtools'], downloads / 'bedtools-source.tar.gz')
    for item in lock['seacr']['files']:
        shared.download(item, downloads / Path(item['url']).name)
    for item in lock['hosted_completion_recipe'].get('additional_sources', []):
        shared.download(item, downloads / item['filename'])
    baseline = r_inventory(runtime, 'image-inventory')
    initial = {row['Package']: row for row in baseline if r_admission_mode(row) == 'immutable-image'}
    descriptions = {name: row for name, row in initial.items()}
    selected = {'R': '4.6.1', **{name: row['Version'] for name, row in initial.items()}}
    admitted = []
    sources = runtime / 'r-sources'
    sources.mkdir()
    for row in lock['r_candidate_closure']:
        if row['package'] in initial:
            admitted.append(dict(package=row['package'], version=initial[row['package']]['Version'],
                                 admission='immutable-image', identity=initial[row['package']]))
        else:
            record = download_r(row, sources / (row['package'] + '_' + row['version'] + '.tar.gz'))
            pinned = next((item for item in lock['r_sources'] if item['package'] == row['package']), None)
            if pinned and (pinned['version'] != row['version'] or pinned['sha256'] != record['sha256']):
                raise ValueError('pinned current R oracle source SHA256 differs: ' + row['package'])
            admitted.append(dict(record, admission='source'))
            selected[row['package']] = row['version']
            descriptions[row['package']] = record['description']
    check_constraints(selected, descriptions)
    order = install_order(descriptions, {'R', *initial})
    image_bindings = {row['Package']: files_identity(row['LibPath'] + '/' + row['Package'])
                      for row in initial.values()}
    save(runtime / 'admitted-install-manifest.json', dict(status='admitted before installation; not installed',
         R='4.6.1', Bioconductor='3.23', image_digest=IMAGE, packages=admitted, order=order,
         selected_versions=selected, image_base_recommended_files=image_bindings,
         source_packet_sha256=sha256(lock_path), shared_controller_sha256=sha256(shared.__file__)))
    save(runtime / 'acquisition-files.json', files_identity(downloads))


def pinned_requirements(packages, names, path, replacement=None):
    lines = []
    for name in names:
        item = packages[name]
        digest = replacement['sha256'] if name == 'macs3' and replacement else item['sha256']
        lines.append(name + '==' + item['version'] + ' --hash=sha256:' + digest)
    path.write_text('\n'.join(lines) + '\n')


def macs_build_environment(original, gcc, gxx):
    for executable in (gcc, gxx):
        if not Path(executable).is_absolute() or any(character.isspace() for character in executable):
            raise ValueError('MACS compiler must be a bare absolute executable path')
    return dict(original, CC=str(gcc), CXX=str(gxx))


def prepare_macs_compilers(runtime):
    hosted(runtime)
    import sysconfig
    names = ('CC', 'CXX', 'LDSHARED', 'LDCXXSHARED', 'CCSHARED', 'CFLAGS',
             'CXXFLAGS', 'CPPFLAGS', 'LDFLAGS', 'CONFIG_ARGS')
    original = {name: sysconfig.get_config_var(name) for name in names}
    compilers = {}
    for name in ('gcc', 'g++'):
        located = shutil.which(name)
        if not located:
            save(runtime / 'compiler-selection-failure.json', dict(status='failed', missing=name,
                 Python_compiler_configuration=original, compiler_install_attempted=False))
            raise RuntimeError('required existing image compiler absent: ' + name)
        resolved = Path(located).resolve(strict=True)
        if not resolved.is_file() or not os.access(resolved, os.X_OK):
            raise RuntimeError('existing image compiler is not executable: ' + str(resolved))
        target = runtime / 'calls' / ('macs-compiler-' + name.replace('+', 'x'))
        require(target, [resolved, '--version'])
        compilers[name] = dict(discovered_path=located, resolved_path=str(resolved),
            sha256=sha256(resolved), version_stdout=(target / 'stdout.txt').read_text(),
            version_stderr=(target / 'stderr.txt').read_text(),
            process_sha256=sha256(target / 'process.json'))
    environment = macs_build_environment(os.environ, compilers['gcc']['resolved_path'],
                                         compilers['g++']['resolved_path'])
    save(runtime / 'macs-compiler-binding.json', dict(status='existing image compilers bound before MACS build',
         Python_compiler_configuration=original, compilers=compilers,
         inherited_overrides={name: os.environ.get(name) for name in
             ('CC', 'CXX', 'LDSHARED', 'LDCXXSHARED', 'LD', 'AR', 'CFLAGS', 'CXXFLAGS', 'CPPFLAGS', 'LDFLAGS')},
         build_overrides={name: environment[name] for name in ('CC', 'CXX')},
         scope='MACS wheel build child only; later stages retain original environment',
         compiler_install_attempted=False))
    return environment


def install(lock_path, output):
    runtime = hosted(output)
    lock = json.loads(Path(lock_path).read_text())
    if platform.python_version() != '3.12.10':
        raise RuntimeError('portable CPython3.12.10 required')
    admission = json.loads((runtime / 'admitted-install-manifest.json').read_text())
    if admission['source_packet_sha256'] != sha256(lock_path) or admission['image_digest'] != IMAGE:
        raise ValueError('runtime admission identity differs')
    python = shared.bootstrap_venv(runtime)
    packages = lock['python_packages']
    downloads = runtime / 'downloads'
    installer = [python, '-m', 'pip', 'install', '--no-index', '--no-deps', '--no-compile',
                 '--no-cache-dir', '--require-hashes', '--find-links', downloads]
    for label, names in [('bootstrap', ['pip', 'setuptools', 'wheel']),
                         ('build-dependencies', ['setuptools', 'cython', 'numpy']),
                         ('runtime-dependencies', sorted(set(packages) - {'macs3'}))]:
        file = runtime / (label + '.txt')
        pinned_requirements(packages, names, file)
        require(runtime / 'calls' / ('python-' + label), [*installer, '-r', file])
    wheels = runtime / 'built-wheels'
    wheels.mkdir()
    build_environment = prepare_macs_compilers(runtime)
    require(runtime / 'calls/macs-build', [python, '-m', 'pip', 'wheel', '--no-index', '--no-deps',
        '--no-build-isolation', '--no-cache-dir', '--wheel-dir', wheels, downloads / packages['macs3']['filename']],
        env=build_environment)
    built = list(wheels.glob('*.whl'))
    if len(built) != 1:
        raise RuntimeError('MACS build must produce one retained wheel')
    file = runtime / 'macs-built.txt'
    pinned_requirements(packages, ['macs3'], file, dict(sha256=sha256(built[0])))
    require(runtime / 'calls/macs-install', [*installer, '--find-links', wheels, '-r', file])
    for label, argv in [('pip-check', ['check']), ('pip-inspect', ['inspect', '--local']),
                        ('pip-freeze', ['freeze', '--all'])]:
        require(runtime / 'calls' / label, [python, '-m', 'pip', *argv])
    extract_archive(downloads / 'bedtools-source.tar.gz', runtime / 'bedtools-original')
    original = next((runtime / 'bedtools-original').iterdir())
    original_hashes = files_identity(original)
    build = runtime / 'bedtools-build'
    shutil.copytree(original, build, symlinks=True)
    require(runtime / 'calls/bedtools-build', ['make', '-j', '2'], cwd=build)
    shutil.copy2(build / 'bin/bedtools', runtime / 'venv/bin/bedtools')
    if files_identity(original) != original_hashes:
        raise ValueError('bedtools original source changed')
    for name in ('SEACR_1.3.sh', 'SEACR_1.3.R'):
        shutil.copy2(downloads / name, runtime / name)
    libraries = Path(os.environ['R_LIBS_USER'])
    for name in admission['order']:
        record = next(row for row in admission['packages'] if row['package'] == name)
        require(runtime / 'calls' / ('R-install-' + name), ['R', 'CMD', 'INSTALL', '--no-multiarch',
                '--no-docs', '--no-html', '--library=' + str(libraries), record['archive']])
    installed = r_inventory(runtime, 'installed-inventory')
    actual = {row['Package']: row for row in installed}
    for name, expected in admission['selected_versions'].items():
        if name != 'R' and (name not in actual or actual[name]['Version'] != expected):
            raise ValueError('installed R package differs from admitted version: ' + name)
    require(runtime / 'calls/R-runtime-gate', ['Rscript', '--vanilla', ROOT / 'peak_first_workflow_oracle.R',
                                              'runtime', runtime / 'R-runtime'])
    executables = {}
    env = dict(os.environ, PATH=str(runtime / 'venv/bin') + ':' + os.environ['PATH'])
    for program in ('cc', 'g++', 'ldd', 'bash', 'awk', 'sort', 'cut', 'sed', 'R', 'Rscript', 'bedtools', 'macs3'):
        executable = shutil.which(program, path=env['PATH'])
        if not executable:
            raise RuntimeError('required runtime executable absent: ' + program)
        executables[program] = dict(path=executable, sha256=sha256(Path(executable).resolve()))
        flags = ['-W', 'version'] if program == 'awk' else ['--version']
        require(runtime / 'calls' / ('version-' + program), [executable, *flags], env=env)
    save(runtime / 'final-runtime-manifest.json', dict(status='installed and admitted; no scientific case executed',
         R='4.6.1', Bioconductor='3.23', Python='3.12.10', image_digest=IMAGE,
         install_manifest_sha256=sha256(runtime / 'admitted-install-manifest.json'),
         built_macs_wheel=dict(path=str(built[0]), sha256=sha256(built[0])),
         packages=actual, package_files={name: files_identity(row['LibPath'] + '/' + name)
                                      for name, row in actual.items()},
         python_environment=files_identity(runtime / 'venv'), executables=executables,
         sources=files_identity(runtime / 'r-sources'), shared_controller_sha256=sha256(shared.__file__)))


def prepare_inputs(inputs, runtime):
    hosted(inputs)
    if platform.python_version() != '3.12.10' or not (runtime / 'final-runtime-manifest.json').is_file():
        raise RuntimeError('admitted runtime required before input conversion')
    from collections import Counter
    import pysam
    generator = importlib.import_module('peak_first_workflow_inputs')
    manifest = generator.generate(inputs)
    alignments = inputs / 'alignments'
    alignments.mkdir()
    observations = {}
    for sample in SAMPLES:
        for form in ('se', 'pe'):
            sam = inputs / (sample + '.' + form + '.sam')
            unsorted = alignments / (sample + '.' + form + '.unsorted.bam')
            bam = alignments / (sample + '.' + form + '.bam')
            pysam.view('-b', '-o', str(unsorted), str(sam), catch_stdout=False)
            pysam.sort('-@', '1', '-T', str(Path(os.environ['TMPDIR']) / (sample + form)), '-o', str(bam), str(unsorted))
            pysam.index(str(bam))
            pysam.quickcheck(str(bam))
            expected = Counter(line.rstrip('\n') for line in sam.open() if not line.startswith('@'))
            decoded = Counter()
            primary = {}
            previous = (-1, -1)
            total = 0
            with pysam.AlignmentFile(str(bam), 'rb') as reader, (alignments / (sample + '.' + form + '.records.jsonl')).open('w') as records:
                if dict(zip(reader.references, reader.lengths)) != {'chrA': 10000000, 'chrB': 10000000} or not reader.has_index():
                    raise ValueError('BAM dictionary/index differs')
                for read in reader.fetch(until_eof=True):
                    total += 1
                    decoded[read.to_string()] += 1
                    if not read.is_unmapped:
                        position = (read.reference_id, read.reference_start)
                        if position < previous or read.reference_end > reader.lengths[read.reference_id]:
                            raise ValueError('BAM ordering/record boundary differs')
                        previous = position
                    if form == 'pe' and not read.flag & (256 | 2048):
                        primary.setdefault(read.query_name, []).append((read.flag, read.reference_id, read.reference_start,
                            read.reference_end, read.next_reference_id, read.next_reference_start, read.template_length))
                    records.write(json.dumps(dict(name=read.query_name, flag=read.flag, mapq=read.mapping_quality,
                        reference_id=read.reference_id, start=read.reference_start, end=read.reference_end,
                        mate_reference_id=read.next_reference_id, mate_start=read.next_reference_start,
                        tlen=read.template_length, cigar=read.cigarstring, sam=read.to_string())) + '\n')
            if decoded != expected:
                raise ValueError('BAM full-decode records differ from frozen SAM')
            for name, pair in primary.items():
                if len(pair) != 2 or {read[0] & 192 for read in pair} != {64, 128}:
                    raise ValueError('primary pair identity differs: ' + name)
                a, b = pair
                if a[4:6] != b[1:3] or b[4:6] != a[1:3] or a[6] != -b[6]:
                    raise ValueError('BAM mate links/TLEN differ: ' + name)
                if all(read[0] & 2 and not read[0] & 4 for read in pair):
                    span = max(a[3], b[3]) - min(a[2], b[2])
                    if abs(a[6]) != span:
                        raise ValueError('BAM proper-pair fragment span/TLEN differ: ' + name)
            if form == 'pe' and len(primary) != manifest['molecules'][sample]:
                raise ValueError('BAM molecule count differs from generated ledger')
            observations[sample + '.' + form] = dict(records=total, primary_molecules=len(primary),
                bam_sha256=sha256(bam), index_sha256=sha256(Path(str(bam) + '.bai')))
    for group, samples in [('target', ('T1', 'T2')), ('control', ('C1', 'C2'))]:
        rows = [line.rstrip().split('\t') for sample in samples for line in (inputs / (sample + '.fragments.bed')).open()]
        rows.sort(key=lambda row: (row[0], int(row[1]), int(row[2]), row[3]))
        (inputs / (group + '.fragments.sorted.bed')).write_text(''.join('\t'.join(row) + '\n' for row in rows))
    if sum(path.stat().st_size for path in inputs.rglob('*') if path.is_file()) > 2**31:
        raise ValueError('generated input corpus exceeds 2GiB')
    save(inputs / 'pre-source-input-manifest.json', dict(status='generated/full-decoded/indexed; no source case executed',
         generated_manifest=manifest, generator_sha256=sha256(generator.__file__), observations=observations,
         files=files_identity(inputs), runtime_manifest_sha256=sha256(runtime / 'final-runtime-manifest.json')))


def case_plan(inputs, runtime, output):
    inputs, runtime, output = map(Path, (inputs, runtime, output))
    python, bedtools = runtime / 'venv/bin/python', runtime / 'venv/bin/bedtools'
    bam = inputs / 'alignments'
    planned = []
    def add(name, family, argv, deps=(), **extra):
        planned.append(dict(name=name, family=family, argv=list(map(str, argv)), deps=list(deps), **extra))
    for group in ('target', 'control'):
        add('track-' + group, 'input-track', [bedtools, 'genomecov', '-bg', '-i',
            inputs / (group + '.fragments.sorted.bed'), '-g', inputs / 'genome.sizes'])
    common = [runtime / 'venv/bin/macs3', 'callpeak', '-g', '20000000', '-B',
              '--threads', '1', '--process-backend', 'serial', '--tempdir', os.environ.get('TMPDIR', str(output / 'tmp'))]
    modes = {
        'M1': ['-f', 'BAM', '-t', bam / 'T1.se.bam', '--keep-dup', '1', '-q', '0.05'],
        'M2': ['-f', 'BAM', '-t', bam / 'T1.se.bam', bam / 'T2.se.bam', '-c',
               bam / 'C1.se.bam', bam / 'C2.se.bam', '--keep-dup', 'auto', '-q', '0.05'],
        'M3': ['-f', 'BAM', '-t', bam / 'T1.se.bam', bam / 'T2.se.bam', '--nomodel',
               '--extsize', '200', '--keep-dup', 'all', '-p', '0.0001'],
        'M4': ['-f', 'BAMPE', '-t', bam / 'T1.pe.bam', bam / 'T2.pe.bam', '-c',
               bam / 'C1.pe.bam', bam / 'C2.pe.bam', '--keep-dup', '1', '-q', '0.05'],
        'M5': ['-f', 'BAMPE', '-t', bam / 'T1.pe.bam', bam / 'T2.pe.bam', '--keep-dup', 'all', '-p', '0.0001']}
    for name, flags in modes.items():
        add(name, 'call', [*common, '-n', name, '--outdir', output / 'results' / name, *flags],
            validate='model' if name in ('M1', 'M2') else 'macs')
    sparse = []
    for control in ('norm', 'non'):
        for threshold in ('stringent', 'relaxed'):
            sparse.append(('S-control-' + control + '-' + threshold,
                           output / 'calls/track-control/stdout.txt', control, threshold))
    for threshold in ('stringent', 'relaxed'):
        sparse.append(('S-numeric-' + threshold, '0.10', 'non', threshold))
    for name, control, norm, threshold in sparse:
        deps = ['track-target'] + (['track-control'] if name.startswith('S-control') else [])
        argv = ['bash', runtime / 'SEACR_1.3.sh', output / 'calls/track-target/stdout.txt',
                control, norm, threshold, output / 'calls' / name / 'result']
        add(name, 'call-sparse', argv, deps, validate='seacr', threshold=threshold)
        diagnostic = 'D-' + name
        diagnostic_argv = argv[:-1] + [output / 'calls' / diagnostic / 'result']
        add(diagnostic, 'source-state', diagnostic_argv, [name], validate='seacr-diagnostic',
            baseline=name, threshold=threshold, diagnostic_env=True)
        add('reload-' + diagnostic, 'reload', ['Rscript', '--vanilla', ROOT / 'peak_first_workflow_oracle.R',
            'seacr-reload', output / 'calls' / diagnostic / 'retained-r',
            output / 'calls' / ('reload-' + diagnostic)], [diagnostic])
    upstream = ['M2', *[name for name, *_ in sparse]]
    action = [python, '-B', Path(__file__).resolve(), '--inputs', inputs, '--runtime', runtime, '--output', output]
    add('row-ledger', 'row-ledger', [*action, '--action', 'rows'], upstream)
    for name, form, threshold in [('A-gtf-default', 'gtf', 'unset'), ('A-gff3-default', 'gff3', 'unset'),
                                  ('A-gtf-300', 'gtf', '300')]:
        target = output / 'calls' / name
        add(name, 'annotate', ['Rscript', '--vanilla', ROOT / 'peak_first_workflow_oracle.R',
            'annotate', inputs / ('annotation.' + form), output / 'rows', target, form, threshold], ['row-ledger'])
        add('reload-' + name, 'reload', ['Rscript', '--vanilla', ROOT / 'peak_first_workflow_oracle.R',
            'reload', target, output / 'calls' / ('reload-' + name), threshold], [name])
    base = [bedtools, 'multicov', '-bams', bam / 'T1.pe.bam', bam / 'T2.pe.bam',
            '-bed', output / 'rows/workflow.bed']
    for name, flags in [('C-default', []), ('C-proper', ['-q', '20', '-p']),
                        ('C-flags', ['-q', '20', '-D', '-F']), ('C-split', ['-q', '20', '-split'])]:
        add(name, 'count', [*base, *flags], ['row-ledger'], validate='count')
    add('count-unit-summary', 'count-summary', [*action, '--action', 'count-summary'],
        ['C-default', 'C-proper', 'C-flags', 'C-split'])
    add('molecule-fragment-diagnostic', 'fragment-diagnostic', [*action, '--action', 'fragments'], ['row-ledger'])
    return planned


def write_rows(output, collections):
    output = Path(output)
    output.mkdir(parents=True)
    with (output / 'workflow.bed').open('w') as bed, (output / 'workflow-rows.tsv').open('w', newline='') as stream:
        writer = csv.writer(stream, delimiter='\t', lineterminator='\n')
        writer.writerow(['row_key', 'row_ordinal', 'source', 'source_row', 'display_id', 'original_fields_json'])
        ordinal = 0
        for collection in collections:
            source, rows = collection[:2]
            originals = collection[2] if len(collection) == 3 else rows
            for source_row, fields in enumerate(rows):
                if len(fields) < 6 or fields[0] not in ('chrA', 'chrB') or not 0 <= int(fields[1]) < int(fields[2]) <= 10000000:
                    raise ValueError('invalid source peak row')
                bed.write('\t'.join(fields[:6]) + '\n')
                writer.writerow([source + ':' + str(source_row), ordinal, source, source_row, fields[3],
                                 json.dumps(originals[source_row], ensure_ascii=True)])
                ordinal += 1


def assemble_rows(inputs, output):
    collections = []
    source = output / 'results/M2/M2_peaks.narrowPeak'
    bindings = {str(source): sha256(source)}
    collections.append(('M2', [line.rstrip().split('\t') for line in source.open() if line.strip()]))
    for job in case_plan(inputs, Path(os.environ['PEAK_RUNTIME']), output):
        if job['family'] != 'call-sparse':
            continue
        path = output / 'calls' / job['name'] / ('result.' + job['threshold'] + '.bed')
        rows = []
        originals = []
        for index, line in enumerate(path.open()):
            fields = line.rstrip().split('\t')
            if len(fields) != 6:
                raise ValueError('SEACR original row width differs')
            rows.append([*fields[:3], job['name'] + ':' + str(index), '0', '.', *fields[3:]])
            originals.append(fields)
        collections.append((job['name'], rows, originals))
        bindings[str(path)] = sha256(path)
    collections.append(('diagnostic', [line.rstrip().split('\t') for line in (inputs / 'diagnostic.bed').open()]))
    write_rows(output / 'rows', collections)
    bindings[str(inputs / 'diagnostic.bed')] = sha256(inputs / 'diagnostic.bed')
    save(output / 'rows/source-bindings.json', dict(source_files=bindings,
        semantics='ordered source/row keys; original source files retained; no deduplication'))


def diagnostic_environment(runtime, cwd):
    tools = cwd / 'capture-tools'
    tools.mkdir()
    retained = cwd / 'retained-r'
    retained.mkdir()
    actual_r = shutil.which('Rscript')
    actual_rm = shutil.which('rm')
    if not actual_r or not actual_rm:
        raise RuntimeError('diagnostic executables missing')
    wrapper = '#!/bin/sh\nset -eu\ntest "$1" = "$PEAK_SEACR_SOURCE_R"\nshift\nexec "$PEAK_REAL_RSCRIPT" --vanilla "$PEAK_R_DRIVER" "$@"\n'
    (tools / 'Rscript').write_text(wrapper)
    rm_code = """#!{python}
import os
from pathlib import Path
import shutil
import sys
cwd = Path.cwd().resolve()
retained = cwd / 'retained-cleanup'
retained.mkdir(exist_ok=True)
for argument in sys.argv[1:]:
    if argument == '-f':
        continue
    path = Path(argument).resolve()
    if path.parent != cwd:
        raise SystemExit('cleanup-retention path escapes diagnostic cwd')
    if path.is_file():
        shutil.copy2(path, retained / path.name)
os.execv(os.environ['PEAK_REAL_RM'], [os.environ['PEAK_REAL_RM'], *sys.argv[1:]])
""".format(python=runtime / 'venv/bin/python')
    (tools / 'rm').write_text(rm_code)
    for path in tools.iterdir():
        path.chmod(0o755)
    return dict(os.environ, PATH=str(tools) + ':' + os.environ['PATH'], PEAK_REAL_RSCRIPT=actual_r,
        PEAK_REAL_RM=actual_rm, PEAK_R_MODE='seacr-capture',
        PEAK_R_DRIVER=str(ROOT / 'peak_first_workflow_oracle.R'),
        PEAK_SEACR_SOURCE_R=str(runtime / 'SEACR_1.3.R'), PEAK_SEACR_CAPTURE=str(retained))


def validate_job(job, target, root):
    validator = job.get('validate')
    if validator in ('model', 'macs'):
        name = job['name']
        results = root / 'results' / name
        for suffix in ('peaks.narrowPeak', 'peaks.xls', 'summits.bed', 'treat_pileup.bdg', 'control_lambda.bdg'):
            if not (results / (name + '_' + suffix)).is_file():
                raise ValueError('MACS requested output absent: ' + suffix)
        if validator == 'model':
            text = (target / 'stdout.txt').read_text() + (target / 'stderr.txt').read_text()
            fitted = re.findall(r'predicted fragment length is (\d+)', text)
            if not fitted or int(fitted[-1]) <= 0 or not (results / (name + '_model.r')).is_file():
                raise ValueError('model case did not retain a real fitted model')
            save(target / 'model-state.json', dict(d=int(fitted[-1]), source_log_sha256=sha256(target / 'stderr.txt'),
                                                  fallback_accepted=False))
    elif validator in ('seacr', 'seacr-diagnostic'):
        path = target / ('result.' + job['threshold'] + '.bed')
        if not path.is_file():
            raise ValueError('SEACR final output absent')
        if validator == 'seacr-diagnostic':
            original = root / 'calls' / job['baseline'] / path.name
            if path.read_bytes() != original.read_bytes():
                raise ValueError('instrumented SEACR output differs; diagnostic state NOT admitted')
            save(target / 'diagnostic-admission.json', dict(admitted=True, baseline_sha256=sha256(original),
                                                           diagnostic_sha256=sha256(path), performance_evidence=False))
    elif validator == 'count':
        original = (root / 'rows/workflow.bed').read_text().splitlines()
        observed = (target / 'stdout.txt').read_text().splitlines()
        if len(original) != len(observed):
            raise ValueError('multicov lost input rows')
        for before, after in zip(original, observed):
            fields = after.split('\t')
            if fields[:6] != before.split('\t') or len(fields) != 8 or any(int(value) < 0 for value in fields[6:]):
                raise ValueError('multicov row/sample identity differs')


def execute_plan(plan, root, runtime=None):
    root = Path(root)
    root.mkdir(parents=True, exist_ok=True)
    statuses = {}
    for job in plan:
        name = job['name']
        target = root / 'calls' / name
        target.mkdir(parents=True, exist_ok=True)
        failed = [dep for dep in job['deps'] if statuses.get(dep, {}).get('status') != 'returned']
        if failed:
            record = dict(status='blocked', blocked_by=failed, executed=False)
            save(target / 'process.json', record)
        else:
            try:
                env = diagnostic_environment(Path(runtime), target) if job.get('diagnostic_env') else None
                record = capture(target, job['argv'], env=env)
                if record['status'] == 'returned':
                    validate_job(job, target, root)
            except Exception as error:
                record = dict(status='failed', exception=type(error).__name__, message=str(error),
                              traceback=traceback.format_exc())
                save(target / 'validation-failure.json', record)
        statuses[name] = record
        save(root / 'case-statuses.json', statuses)
    return statuses


def fragment_diagnostic(inputs, output):
    rows = [line.rstrip().split('\t') for line in (output / 'rows/workflow.bed').open()]
    index = {}
    for ordinal, row in enumerate(rows):
        for window in range(int(row[1]) // 10000, (int(row[2]) - 1) // 10000 + 1):
            index.setdefault((row[0], window), []).append(ordinal)
    counts = {sample: [0] * len(rows) for sample in ('T1', 'T2')}
    units = {}
    with (output / 'calls/molecule-fragment-diagnostic/assignments.tsv').open('w') as assignments:
        assignments.write('sample\tmolecule_key\trow_ordinal\n')
        for sample in ('T1', 'T2'):
            units[sample] = 0
            for line in (inputs / (sample + '.fragments.bed')).open():
                chrom, start, end, molecule = line.rstrip().split('\t')
                start, end = int(start), int(end)
                units[sample] += 1
                candidates = {ordinal for window in range(start // 10000, (end - 1) // 10000 + 1)
                              for ordinal in index.get((chrom, window), [])}
                for ordinal in sorted(candidates):
                    row = rows[ordinal]
                    if int(row[1]) < end and start < int(row[2]):
                        counts[sample][ordinal] += 1
                        assignments.write(sample + '\t' + molecule + '\t' + str(ordinal) + '\n')
    save(output / 'calls/molecule-fragment-diagnostic/observation.json',
         dict(unit='input-selected molecule outer span', native_policy_selected=False, multicov_parity=False,
              eligible_units=units, row_counts=counts, peak_assignments={s: sum(v) for s, v in counts.items()}))


def count_summary(inputs, output):
    hosted(output)
    import pysam
    summary = {}
    for name in ('C-default', 'C-proper', 'C-flags', 'C-split'):
        columns = [row.split('\t')[6:] for row in (output / 'calls' / name / 'stdout.txt').read_text().splitlines()]
        sample_summary = {}
        for i, sample in enumerate(('T1', 'T2')):
            examined = eligible = 0
            with pysam.AlignmentFile(str(inputs / 'alignments' / (sample + '.pe.bam')), 'rb') as reader:
                for read in reader.fetch(until_eof=True):
                    examined += 1
                    if read.is_unmapped or (name != 'C-default' and read.mapping_quality < 20):
                        continue
                    if name == 'C-proper' and not read.is_proper_pair:
                        continue
                    if name != 'C-flags' and (read.is_duplicate or read.is_qcfail):
                        continue
                    eligible += 1
            sample_summary[sample] = dict(examined_alignments=examined, eligible_mapped_alignments=eligible,
                                          peak_assignments=sum(int(row[i]) for row in columns))
        summary[name] = dict(unit='alignment; -p is a filter not collapse', samples=sample_summary)
    save(output / 'calls/count-unit-summary/observation.json', summary)


def run(inputs, runtime, output):
    root = hosted(output)
    root.mkdir(parents=True)
    admitted = runtime / 'final-runtime-manifest.json'
    if platform.python_version() != '3.12.10' or not admitted.is_file():
        raise RuntimeError('complete admitted runtime required before source cases')
    os.environ['PATH'] = str(runtime / 'venv/bin') + ':' + os.environ['PATH']
    os.environ['PEAK_RUNTIME'] = str(runtime)
    prepare_inputs(inputs, runtime)
    plan = case_plan(inputs, runtime, root)
    save(root / 'execution-plan.json', plan)
    save(root / 'pre-source.json', dict(final_runtime_sha256=sha256(admitted),
         input_manifest_sha256=sha256(inputs / 'pre-source-input-manifest.json'), source_controller_sha256=sha256(__file__),
         shared_controller_sha256=sha256(shared.__file__), plan_sha256=sha256(root / 'execution-plan.json')))
    final_runtime = json.loads(admitted.read_text())
    protected = dict(inputs=files_identity(inputs), sources=files_identity(runtime / 'r-sources'),
                     R_libraries={name: files_identity(row['LibPath'] + '/' + name)
                                  for name, row in final_runtime['packages'].items()},
                     python_environment=files_identity(runtime / 'venv'),
                     seacr={name: sha256(runtime / name) for name in ('SEACR_1.3.sh', 'SEACR_1.3.R')})
    statuses = execute_plan(plan, root, runtime)
    after = dict(inputs=files_identity(inputs), sources=files_identity(runtime / 'r-sources'),
                 R_libraries={name: files_identity(row['LibPath'] + '/' + name)
                              for name, row in final_runtime['packages'].items()},
                 python_environment=files_identity(runtime / 'venv'),
                 seacr={name: sha256(runtime / name) for name in ('SEACR_1.3.sh', 'SEACR_1.3.R')})
    integrity = protected == after
    save(root / 'integrity-after.json', dict(unchanged=integrity, before=protected, after=after))
    complete = integrity and all(row['status'] == 'returned' for row in statuses.values())
    save(root / 'qualification.json', dict(status='complete' if complete else 'failed', cases=statuses,
         operations=['call', 'call-sparse', 'annotate', 'count'], native_policy_selected=False,
         performance_claim=False, interpretation='source capture only; original process and object evidence is authority'))
    if not complete:
        raise RuntimeError('source workflow has failed/blocked cases or integrity changes')


def bind_evidence(root):
    root = Path(root)
    root.mkdir(parents=True, exist_ok=True)
    files = files_identity(root)
    files.pop('artifact-manifest.json', None)
    save(root / 'artifact-manifest.json', dict(files=files,
        interpretation='retention is not source success; read runtime and case status/qualification',
        extraction_policy='archive symlinks may be absolute original runtime aliases; never blindly extract; validate/map targets first'))


def package_evidence(root, destination):
    root, destination = Path(root).resolve(), Path(destination).resolve()
    if destination.is_relative_to(root) or destination.exists() or destination.is_symlink():
        raise ValueError('archive must be a new path outside evidence')
    with tarfile.open(destination, 'w') as archive:
        archive.add(root, arcname=root.name, recursive=True)
    save(Path(str(destination) + '.json'), dict(sha256=sha256(destination), size=destination.stat().st_size,
        symlinks='preserved original aliases; extraction requires validation/mapping, not blind absolute-link extraction'))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--action', choices=('acquire', 'install', 'run', 'rows', 'fragments', 'count-summary', 'manifest', 'package'), required=True)
    parser.add_argument('--runtime-lock', type=Path)
    parser.add_argument('--runtime', type=Path)
    parser.add_argument('--inputs', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--archive', type=Path)
    args = parser.parse_args()
    if args.action == 'manifest':
        bind_evidence(args.output)
    elif args.action == 'package':
        package_evidence(args.output, args.archive)
    else:
        hosted(args.output)
        if args.action == 'acquire':
            acquire(args.runtime_lock, args.output)
        elif args.action == 'install':
            install(args.runtime_lock, args.output)
        elif args.action == 'run':
            run(args.inputs, args.runtime, args.output)
        elif args.action == 'rows':
            assemble_rows(args.inputs, args.output)
        elif args.action == 'fragments':
            fragment_diagnostic(args.inputs, args.output)
        elif args.action == 'count-summary':
            count_summary(args.inputs, args.output)


if __name__ == '__main__':
    main()
