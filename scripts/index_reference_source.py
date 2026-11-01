#!/usr/bin/env python3
"""Manual hosted samtools 1.24 reference-workflow source observations."""

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import platform
import re
import shutil
import stat
import struct
import sys
import tarfile
import tempfile
import time
import traceback
import urllib.request
import zlib

from plink_first_workflow_oracle import capture, download, save, sha256


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = '.github/workflows/index-reference-source.yml'
IMAGE = 'sha256:a689e29bc3adf4663ef9a141d23081252764d1319c63f591a027bd6fd676f4c1'
IMAGE_CONFIG = 'sha256:f3d916a4884034b89cb6148781f07d8c92d94b6c6dc1b74dcbec3475d16400da'
CACHE_KEYS = ('TMPDIR', 'XDG_CACHE_HOME', 'XDG_CONFIG_HOME', 'XDG_DATA_HOME',
              'PYTHONUSERBASE', 'PYTHONPYCACHEPREFIX', 'CCACHE_DIR', 'CLANG_MODULE_CACHE_PATH')
EOF_MEMBER = bytes.fromhex('1f8b08040000000000ff0600424302001b0003000000000000000000')
SOURCES = {
    'samtools': dict(version='1.24', repository='samtools/samtools', tag='1.24', prefix='samtools-1.24',
        filename='samtools-1.24.tar.bz2', size=9491012, unpacked_limit=128 * 2**20,
        sha256='89b2a440123eeaa400392ce1736e7d60ce9041843027d76819753c5a8246bfdd',
        commit='dc71c7274044d1050ccb64901731373ec7e915b6', anchors={
            'faidx.c': '17a6ac6cbea5897b249eac64d8ef61b1725e36038b81710c3107996c34942bcd',
            'dict.c': 'e33cd7d37020deacf2a88f532fdd0264bdbdafeb8cbff048ea0d0374e60da6b9'}),
    'htslib': dict(version='1.24', repository='samtools/htslib', tag='1.24', prefix='htslib-1.24',
        filename='htslib-1.24.tar.bz2', size=5004265, unpacked_limit=96 * 2**20,
        sha256='28a8de191381c7a97a35675ceac76fa1ea95e7b678d6a2e9d600a7874e4077de',
        commit='4b705e4fada8ee2b6b15746f725ee8ac51631803', anchors={
            'faidx.c': '158a56998136bba5477975b40a173fec2ef9dc5efac38ec3044a068eb395aec5',
            'bgzf.c': 'd402e1a6a5bdd8e03a7e4f07bb9d19647560c28e3cf6755e7362805007453647',
            'hts.c': '98800fbbac0ebd55f37aaf9d32686ad78e095456fbb253dbb7c6f3ba28954471'}),
    'libdeflate': dict(version='1.25', repository='ebiggers/libdeflate', tag='v1.25', prefix='libdeflate-1.25',
        filename='libdeflate-1.25.tar.gz', size=186474, unpacked_limit=8 * 2**20,
        sha256='fed5cd22f00f30cc4c2e5329f94e2b8a901df9fa45ee255cb70e2b0b42344477', anchors={
            'CMakeLists.txt': '6f39900a4199daaeeb13a39d6b5e5c5f0a5f80a6d49c1b65975316e52cff40fc'}),
    'cmake': dict(version='3.31.8', repository='Kitware/CMake', tag='v3.31.8', prefix='cmake-3.31.8-linux-x86_64',
        filename='cmake-3.31.8-linux-x86_64.tar.gz', size=55012213, unpacked_limit=256 * 2**20,
        sha256='630615d8e98ac33eba7fbe472626dff5c899c85af3c024585ae109166a6909d0', anchors={}),
}
DONORS = {
    'fasta-donor': dict(owner='rsomics-fasta-index', commit='fb86ed124f069141b2934641f1291a8adc83a4be',
        path='tests/golden/two.fa', sha256='f6aa242815f8cd80990041fcd2fff32cda7a5a76fb1da5e50622f54336a41331',
        body=b'>chr1\nACGTACGTACGT\n>chr2\nGGGGGGGG\n'),
    'dict-donor': dict(owner='rsomics-bam-dict', commit='e3dd9a007965f9dd741882690d973b9e51044bf3',
        path='tests/golden/small.fa', sha256='35bba8db8b245b3443f7c00c6c2eaebf17da1fef60ba191720d5cf35ad934e3f',
        body=b'>chr1\nACGTACGTACGT\n>chr2 extra description\nNNNTACG\n>chr3 some other text\nGCTAGCTAGCTANNNACGT\n'),
}


def load(path: Path) -> dict:
    return json.loads(path.read_text())


def identity(path: Path) -> dict:
    return dict(bytes=path.stat().st_size, sha256=sha256(path))


def tree_files(root: Path, exclude: tuple[str, ...] = ()) -> dict:
    files = {}
    for path in sorted(root.rglob('*')):
        relative = path.relative_to(root).as_posix()
        if relative in exclude:
            continue
        mode = path.lstat().st_mode
        if stat.S_ISLNK(mode):
            files[relative] = dict(type='symlink', target=os.readlink(path))
        elif stat.S_ISREG(mode):
            files[relative] = dict(type='file', mode=stat.S_IMODE(mode), **identity(path))
        elif not stat.S_ISDIR(mode):
            raise ValueError('nonregular source evidence member: ' + relative)
    return files


def shell_blocks(path: Path) -> list[str]:
    return re.findall(r'^        run: \|\n((?:          .*\n|\n)+)', path.read_text(), re.M)


def validate_workflow(path: Path) -> None:
    text = path.read_text()
    event = re.search(r'^on:\n(.*?)(?=^\S)', text, re.M | re.S)
    if event is None or event.group(1).strip() != 'workflow_dispatch:':
        raise ValueError('index source workflow must be manual only')
    if f'gcc:14.3.0-bookworm@{IMAGE}' not in text or 'runs-on: ubuntu-24.04' not in text:
        raise ValueError('source workflow runtime declaration differs')


def hosted_root() -> Path:
    if (os.environ.get('GITHUB_ACTIONS') != 'true' or platform.system() != 'Linux'
            or platform.machine() != 'x86_64' or os.environ.get('GITHUB_REPOSITORY') != 'omics-rust/rsomics-world'
            or os.environ.get('GITHUB_EVENT_NAME') != 'workflow_dispatch'):
        raise RuntimeError('real manual hosted Linux x86_64 source execution required')
    if os.environ.get('INDEX_CONTAINER_DIGEST') != IMAGE or os.environ.get('GCC_VERSION') != '14.3.0':
        raise ValueError('immutable compiler container declaration differs')
    if not re.fullmatch('[0-9a-f]{40}', os.environ.get('GITHUB_SHA', '')):
        raise ValueError('actual controller head is absent')
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    root = Path(os.environ['INDEX_SESSION']).resolve(strict=True)
    if root == runner or not root.is_relative_to(runner) or root.stat().st_dev != runner.stat().st_dev:
        raise ValueError('source session escapes runner device')
    for key in CACHE_KEYS:
        path = Path(os.environ[key]).resolve(strict=True)
        if not path.is_relative_to(root) or path.stat().st_dev != root.stat().st_dev:
            raise ValueError('source cache escapes session device: ' + key)
    return root


def initialize() -> None:
    if os.environ.get('GITHUB_ACTIONS') != 'true' or platform.system() != 'Linux':
        raise RuntimeError('source initialization is hosted only')
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix='index-reference-source-', dir=runner))
    values = dict(INDEX_SESSION=str(root), INDEX_CARRIER=str(root.parent / (root.name + '.tar')))
    for key in CACHE_KEYS:
        path = root / 'cache' / key
        path.mkdir(parents=True)
        values[key] = str(path)
    with Path(os.environ['GITHUB_ENV']).open('a') as stream:
        for key, value in values.items():
            stream.write(f'{key}={value}\n')
    save(root / 'request.json', dict(controller=os.environ['GITHUB_SHA'],
         run_id=os.environ['GITHUB_RUN_ID'], run_attempt=os.environ['GITHUB_RUN_ATTEMPT'],
         workflow_ref=os.environ.get('GITHUB_WORKFLOW_REF'), purpose='source-only engineering casebook; no provider reference/native product'))


def command(root: Path, name: str, argv: list, *, cwd: Path | None = None, required: bool = True) -> dict:
    started = time.monotonic()
    result = capture(root / 'calls' / name, argv, cwd, timeout=1800)
    result['elapsed_seconds'] = time.monotonic() - started
    result['environment'] = {key: os.environ.get(key) for key in
                             ('INDEX_CONTAINER_DIGEST', 'PKG_CONFIG_PATH', 'GITHUB_SHA', 'GITHUB_RUN_ID')}
    save(root / 'calls' / name / 'observation.json', result)
    if required and result['status'] != 'returned':
        raise RuntimeError('required source/runtime call failed: ' + name)
    return result


def verify_body(path: Path, specification: dict) -> None:
    if identity(path) != dict(bytes=specification['size'], sha256=specification['sha256']):
        raise ValueError('official archive body identity differs: ' + path.name)


def extract_release(archive: Path, destination: Path, specification: dict) -> Path:
    verify_body(archive, specification)
    destination.mkdir()
    with tarfile.open(archive, 'r:*') as stream:
        members = stream.getmembers()
        if sum(member.size for member in members if member.isfile()) > specification['unpacked_limit']:
            raise ValueError('official unpacked source exceeds bound')
        names = [member.name.rstrip('/') for member in members]
        if len(names) != len(set(names)):
            raise ValueError('duplicate official archive member')
        symlinks = {member.name.rstrip('/') for member in members if member.issym()}
        for member in members:
            path = PurePosixPath(member.name)
            if path.is_absolute() or '..' in path.parts or not path.parts or path.parts[0] != specification['prefix']:
                raise ValueError('official archive path differs')
            if any('/'.join(path.parts[:i]) in symlinks for i in range(1, len(path.parts))):
                raise ValueError('official archive member traverses a symlink')
            target = destination / member.name
            if member.issym() and not (target.parent / member.linkname).resolve().is_relative_to(destination.resolve()):
                raise ValueError('official archive symlink escapes')
            if member.islnk() and (member.linkname not in names or member.linkname in symlinks):
                raise ValueError('official archive hardlink differs')
            if not (member.isdir() or member.isfile() or member.issym() or member.islnk()):
                raise ValueError('unsupported official archive member')
        for member in members:
            target = destination / member.name
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
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
    return destination / specification['prefix']


def acquire_release(root: Path, name: str) -> Path:
    specification = SOURCES[name]
    downloads = root / 'runtime/downloads'
    api = downloads / (name + '-release.json')
    download(f'https://api.github.com/repos/{specification["repository"]}/releases/tags/{specification["tag"]}', api, 2 * 2**20)
    assets = [asset for asset in load(api)['assets'] if asset['name'] == specification['filename']]
    url = f'https://github.com/{specification["repository"]}/releases/download/{specification["tag"]}/{specification["filename"]}'
    if (len(assets) != 1 or assets[0]['size'] != specification['size']
            or assets[0].get('digest') != 'sha256:' + specification['sha256']
            or assets[0]['browser_download_url'] != url):
        raise ValueError('official release asset metadata differs: ' + name)
    path = downloads / specification['filename']
    try:
        download(url, path, specification['size'])
        verify_body(path, specification)
    except Exception:
        save(downloads / (name + '-failure.json'), dict(exception=traceback.format_exc(),
             retained_original=identity(path) if path.is_file() else None))
        raise
    source = extract_release(path, root / 'runtime' / (name + '-original'), specification)
    for relative, expected in specification['anchors'].items():
        if sha256(source / relative) != expected:
            raise ValueError('fixed source commit anchor differs: ' + name + '/' + relative)
    save(root / 'runtime' / (name + '-original-files.json'), tree_files(source))
    return source


def admit_image(root: Path) -> None:
    token = json.load(urllib.request.urlopen('https://auth.docker.io/token?service=registry.docker.io&scope=repository:library/gcc:pull', timeout=60))['token']
    for name, route, digest in (('manifest', 'manifests', IMAGE), ('config', 'blobs', IMAGE_CONFIG)):
        request = urllib.request.Request(f'https://registry-1.docker.io/v2/library/gcc/{route}/{digest}', headers={
            'Authorization': 'Bearer ' + token,
            'Accept': 'application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json'})
        with urllib.request.urlopen(request, timeout=60) as response:
            body = response.read(2**20 + 1)
        if 'sha256:' + hashlib.sha256(body).hexdigest() != digest:
            raise ValueError('official image body differs')
        (root / 'runtime' / ('image-' + name + '.json')).write_bytes(body)
    manifest = load(root / 'runtime/image-manifest.json')
    config = load(root / 'runtime/image-config.json')
    if manifest['config']['digest'] != IMAGE_CONFIG or config['architecture'] != 'amd64' or config['os'] != 'linux':
        raise ValueError('official image execution class differs')


def elf_admission(root: Path, binaries: dict[str, Path], provider: Path) -> dict:
    result = {}
    for name, binary in binaries.items():
        binary = binary.resolve(strict=True)
        with binary.open('rb') as stream:
            header = stream.read(20)
        if header[:6] != b'\x7fELF\x02\x01' or int.from_bytes(header[18:20], 'little') != 62:
            raise ValueError('actual runtime is not little-endian ELF64 x86_64: ' + name)
        command(root, 'elf-' + name, ['readelf', '--file-header', '--dynamic', '--version-info', binary])
        command(root, 'ldd-' + name, ['ldd', binary])
        text = (root / 'calls' / ('ldd-' + name) / 'stdout.txt').read_text()
        if 'not found' in text:
            raise ValueError('actual loaded library is unresolved: ' + name)
        libraries = {Path(path).resolve() for path in re.findall(r'(?:=>\s*)?(/[^\s]+)', text)}
        if name in ('samtools', 'bgzip') and provider not in libraries:
            raise ValueError('actual samtools/HTSlib did not load the pinned libdeflate provider')
        retained = {}
        for library in sorted(libraries):
            record = identity(library)
            target = root / 'runtime/loaded-libraries' / (record['sha256'] + '-' + library.name)
            target.parent.mkdir(exist_ok=True)
            if not target.exists():
                shutil.copy2(library, target)
            retained[str(library)] = dict(retained=target.relative_to(root).as_posix(), **record)
        result[name] = dict(path=str(binary), **identity(binary), libraries=retained)
    return result


def prepare_runtime(root: Path) -> dict[str, Path]:
    hosted_root()
    (root / 'runtime').mkdir()
    save(root / 'runtime/lock.json', dict(image=IMAGE, image_config=IMAGE_CONFIG, sources=SOURCES))
    admit_image(root)
    for name, argv in {
        'compiler': ['/usr/local/bin/gcc', '-dumpfullversion', '-dumpversion'],
        'compiler-detail': ['/usr/local/bin/gcc', '-v'], 'make': ['make', '--version'],
        'packages': ['dpkg-query', '-W', '-f=${Package}\t${Version}\n'],
        'zlib': ['pkg-config', '--modversion', 'zlib'], 'python': [sys.executable, '--version'],
    }.items():
        command(root, name, argv)
    if (root / 'calls/compiler/stdout.txt').read_text().strip() != '14.3.0':
        raise ValueError('actual GCC compiler differs')
    source = {name: acquire_release(root, name) for name in SOURCES}
    cmake = source['cmake'] / 'bin/cmake'
    command(root, 'cmake-version', [cmake, '--version'])
    if (root / 'calls/cmake-version/stdout.txt').read_text().splitlines()[0] != 'cmake version 3.31.8':
        raise ValueError('actual CMake version differs')
    prefix = root / 'runtime/prefix'
    build = root / 'runtime/libdeflate-build'
    command(root, 'libdeflate-configure', [cmake, '-S', source['libdeflate'], '-B', build,
        '-DCMAKE_BUILD_TYPE=Release', '-DCMAKE_INSTALL_PREFIX=' + str(prefix), '-DCMAKE_INSTALL_LIBDIR=lib',
        '-DLIBDEFLATE_BUILD_STATIC_LIB=OFF', '-DLIBDEFLATE_BUILD_SHARED_LIB=ON',
        '-DLIBDEFLATE_BUILD_GZIP=OFF', '-DLIBDEFLATE_BUILD_TESTS=OFF'])
    command(root, 'libdeflate-build', [cmake, '--build', build, '--parallel', '2'])
    command(root, 'libdeflate-install-prefix', [cmake, '--install', build])
    provider = (prefix / 'lib/libdeflate.so.0').resolve(strict=True)
    if '#define LIBDEFLATE_VERSION_STRING\t"1.25"' not in (prefix / 'include/libdeflate.h').read_text():
        raise ValueError('actual installed libdeflate header version differs')
    os.environ.update(CPPFLAGS='-I' + str(prefix / 'include'),
        LDFLAGS='-L' + str(prefix / 'lib') + ' -Wl,-rpath,' + str(prefix / 'lib'),
        LD_LIBRARY_PATH=str(prefix / 'lib'), PKG_CONFIG_PATH=str(prefix / 'lib/pkgconfig'))
    command(root, 'htslib-configure', ['./configure', '--prefix=' + str(prefix), '--with-libdeflate', '--disable-libcurl'], cwd=source['htslib'])
    command(root, 'htslib-build', ['make', '-j2'], cwd=source['htslib'])
    command(root, 'samtools-configure', ['./configure', '--prefix=' + str(prefix), '--without-curses',
        '--with-htslib=' + str(source['htslib'])], cwd=source['samtools'])
    command(root, 'samtools-build', ['make', '-j2'], cwd=source['samtools'])
    binaries = dict(samtools=source['samtools'] / 'samtools', bgzip=source['htslib'] / 'bgzip', cmake=cmake)
    command(root, 'samtools-version', [binaries['samtools'], '--version'])
    command(root, 'bgzip-version', [binaries['bgzip'], '--version'])
    observed = (root / 'calls/samtools-version/stdout.txt').read_text()
    if (not observed.startswith('samtools 1.24\nUsing htslib 1.24\n')
            or not re.search(r'libdeflate(?:=|:|\s+)yes', observed)):
        raise ValueError('actual samtools/HTSlib version or libdeflate feature differs')
    if (root / 'calls/bgzip-version/stdout.txt').read_text().splitlines()[0] != 'bgzip (htslib) 1.24':
        raise ValueError('actual bgzip version differs')
    admission = elf_admission(root, binaries, provider)
    save(root / 'runtime/admission.json', dict(executables=admission, libdeflate_provider=identity(provider),
         libdeflate_path=str(provider), source_versions={name: spec['version'] for name, spec in SOURCES.items()},
         loaded_library_scope='ELF loader resolution under the exact environment used for these source processes'))
    return binaries


def prepare_inputs(root: Path) -> dict[str, Path]:
    root.mkdir()
    inputs = {}
    for name, donor in DONORS.items():
        path = root / (name + '.fa')
        if hashlib.sha256(donor['body']).hexdigest() != donor['sha256']:
            raise ValueError('embedded team donor bytes differ')
        path.write_bytes(donor['body'])
        inputs[name] = path
    records = [
        ('long', 262177, 101, b'\n'), ('crlf', 127, 31, b'\r\n'),
        ('colon', 137, 67, b'\n'), ('colon:2-7', 19, 19, b'\n'),
        ('MT', 83, 41, b'\n'), ('chrM', 91, 23, b'\n'),
        ('M', 65, 32, b'\n'), ('chrMT', 67, 33, b'\n'), ('last', 83, 83, b'\n')]
    pattern = b'acgTNrySKMBDHV-'
    chunks = []
    for name, length, width, ending in records:
        sequence = (pattern * (length // len(pattern) + 1))[:length]
        description = b' description' if name in ('long', 'crlf') else b''
        chunks.append(b'>' + name.encode() + description + ending)
        chunks.extend(sequence[position:position + width] + ending for position in range(0, length, width))
    inputs['workflow'] = root / 'workflow.fa'
    inputs['workflow'].write_bytes(b''.join(chunks)[:-1])
    inputs['regions'] = root / 'regions.txt'
    inputs['regions'].write_text('long:1-350\nlong:65,500-66,100\nlong:130001-131007\ncrlf:30-67\n{colon}:2-7\n{colon:2-7}\nMT\nlast:1-83\nlong:262170-262200\n')
    save(root / 'specification.json', dict(profile='authored reference engineering fixture, not biological performance evidence',
         records={name: dict(bases=length, line_bases=width, ending='CRLF' if ending == b'\r\n' else 'LF')
                  for name, length, width, ending in records}, final_lf=False,
         donors={name: {key: value for key, value in donor.items() if key != 'body'} for name, donor in DONORS.items()},
         files={name: dict(path=path.name, **identity(path)) for name, path in inputs.items()}))
    return inputs


def query_jobs(samtools: Path, reference: Path, regions: Path, output: Path,
               *, fai: Path | None = None, gzi: Path | None = None) -> list[dict]:
    indexes = (['--fai-idx', fai] if fai else []) + (['--gzi-idx', gzi] if gzi else [])
    return [dict(name=name, output=output / (name + '.fa'), argv=[samtools, 'faidx', *indexes, *flags,
                 '-r', regions, '-o', output / (name + '.fa'), reference, 'long:1-350', 'crlf'])
            for name, flags in (('default', []), ('unwrapped', ['-n', '0']), ('four-workers', ['-@', '3']))]


def bgzf_members(path: Path) -> list[bytes]:
    body = path.read_bytes()
    members = []
    position = 0
    while position < len(body):
        if body[position:position + 4] != b'\x1f\x8b\x08\x04':
            raise ValueError('source bgzip member header differs')
        extra_length = struct.unpack_from('<H', body, position + 10)[0]
        extra = body[position + 12:position + 12 + extra_length]
        offset = 0
        size = None
        while offset < len(extra):
            key, length = struct.unpack_from('<2sH', extra, offset)
            if key == b'BC' and length == 2:
                size = struct.unpack_from('<H', extra, offset + 4)[0] + 1
            offset += 4 + length
        if size is None or position + size > len(body):
            raise ValueError('source bgzip BC size differs')
        members.append(body[position:position + size])
        position += size
    if len(members) < 3 or members[-1] != EOF_MEMBER:
        raise ValueError('nontrivial canonical source BGZF fixture required')
    return members


def execute_casebook(root: Path, binaries: dict[str, Path]) -> None:
    inputs = prepare_inputs(root / 'inputs')
    output = root / 'outputs'
    output.mkdir()
    samtools, bgzip = binaries['samtools'], binaries['bgzip']
    observations = {}
    for name in ('fasta-donor', 'dict-donor'):
        reference = inputs[name]
        command(root, name + '-index', [samtools, 'faidx', reference])
        command(root, name + '-fetch', [samtools, 'faidx', reference, 'chr1', 'chr2'])
        command(root, name + '-dict', [samtools, 'dict', '-u', 'file:///ref.fa', reference])
    reference = inputs['workflow']
    command(root, 'plain-index', [samtools, 'faidx', reference])
    command(root, 'source-bgzip', [bgzip, '-c', reference])
    compressed = inputs['workflow'].with_suffix('.fa.gz')
    shutil.copy2(root / 'calls/source-bgzip/stdout.txt', compressed)
    members = bgzf_members(compressed)
    deflater = zlib.compressobj(level=1, wbits=-15)
    empty_payload = deflater.compress(b'') + deflater.flush()
    empty_header = bytes.fromhex('1f8b0804000000000003060042430200')
    empty = empty_header + struct.pack('<H', len(empty_header) + 2 + len(empty_payload) + 8 - 1) + empty_payload + b'\0' * 8
    embedded = reference.with_suffix('.empty-members.fa.gz')
    embedded.write_bytes(members[0] + empty * 2400 + EOF_MEMBER + b''.join(members[1:]))
    representations = dict(plain=reference, bgzf=compressed, empty_members=embedded)
    for name, path in representations.items():
        if name != 'plain':
            command(root, name + '-index', [samtools, 'faidx', path])
        explicit = output / (name + '-explicit')
        explicit.mkdir()
        fai = explicit / 'reference.fai'
        gzi = explicit / 'reference.gzi' if name != 'plain' else None
        argv = [samtools, 'faidx', '--fai-idx', fai]
        if gzi:
            argv += ['--gzi-idx', gzi]
        command(root, name + '-explicit-index', [*argv, path])
        queries = output / name
        queries.mkdir()
        for job in query_jobs(samtools, path, inputs['regions'], queries, fai=fai, gzi=gzi):
            command(root, name + '-query-' + job['name'], job['argv'])
        command(root, name + '-dict', [samtools, 'dict', '-u', 'file:///reference.fa', '-o', queries / 'reference.dict', path])
        observations[name] = dict(input=identity(path), fai=identity(fai),
             gzi=identity(gzi) if gzi else None, outputs={p.name: identity(p) for p in queries.iterdir()})
    for name in ('bgzf', 'empty_members'):
        for filename in ('default.fa', 'unwrapped.fa', 'four-workers.fa', 'reference.dict'):
            if (output / name / filename).read_bytes() != (output / 'plain' / filename).read_bytes():
                raise ValueError('source representation output differs: ' + name + '/' + filename)
    for name in representations:
        if (output / name / 'default.fa').read_bytes() != (output / name / 'four-workers.fa').read_bytes():
            raise ValueError('source worker output differs')
    cross = output / 'cross-index'
    cross.mkdir()
    plain_fai = output / 'plain-explicit/reference.fai'
    for name, path in (('bgzf', compressed), ('empty_members', embedded)):
        rebuilt = cross / (name + '.bgzip.gzi')
        command(root, name + '-bgzip-reindex', [bgzip, '-r', '-I', rebuilt, path])
        queried = cross / (name + '.fa')
        command(root, name + '-cross-faidx', [samtools, 'faidx', '--fai-idx', plain_fai,
            '--gzi-idx', rebuilt, '-r', inputs['regions'], '-o', queried, path, 'long:1-350', 'crlf'])
        if queried.read_bytes() != (output / 'plain/default.fa').read_bytes():
            raise ValueError('plain FAI / bgzip GZI source cross-read differs')
        command(root, name + '-cross-bgzip', [bgzip, '-d', '-c', '-b', '65500', '-s', '1001',
            '-I', output / (name + '-explicit/reference.gzi'), path])
        if (root / 'calls' / (name + '-cross-bgzip') / 'stdout.txt').read_bytes() != reference.read_bytes()[65500:66501]:
            raise ValueError('bgzip source did not consume samtools GZI correctly')
    ordinary_gzip = reference.with_suffix('.ordinary.fa.gz')
    encoder = zlib.compressobj(wbits=31)
    ordinary_gzip.write_bytes(encoder.compress(reference.read_bytes()) + encoder.flush())
    command(root, 'dict-ordinary-gzip', [samtools, 'dict', '-u', 'file:///reference.fa', ordinary_gzip])
    if (root / 'calls/dict-ordinary-gzip/stdout.txt').read_bytes() != (output / 'plain/reference.dict').read_bytes():
        raise ValueError('source ordinary-gzip dictionary differs')
    command(root, 'dict-default-uri', [samtools, 'dict', reference])
    command(root, 'dict-stdin', ['bash', '-c', 'exec "$1" dict - < "$2"', 'source-dict', samtools, reference])
    for name, flags in (('alias', ['-A']), ('no-header', ['-H']), ('assembly', ['-a', 'fixture-assembly']),
                        ('species', ['-s', 'fixture species']), ('combined', ['-A', '-H', '-a', 'fixture-assembly', '-s', 'fixture species'])):
        command(root, 'dict-' + name, [samtools, 'dict', '-u', 'file:///reference.fa', *flags, reference])
    (root / 'observations').mkdir()
    duplicate = root / 'observations/duplicate.fa'
    duplicate.write_bytes(b'>same\nACGT\n>same\nTGCA\n')
    ragged = root / 'observations/ragged.fa'
    ragged.write_bytes(b'>ragged\nACGTA\nCGT\nACGTA\n')
    no_marker = root / 'observations/no-final-marker.fa.gz'
    no_marker.write_bytes(b''.join(members[:-1]))
    damaged = root / 'observations/crc-damaged.fa.gz'
    corrupt = bytearray(members[0])
    corrupt[-8] ^= 1
    damaged.write_bytes(bytes(corrupt) + b''.join(members[1:]))
    diagnostics = {
        'duplicate-name': [samtools, 'faidx', duplicate], 'ragged': [samtools, 'faidx', ragged],
        'missing-region': [samtools, 'faidx', reference, 'not_present'],
        'ambiguous-name': [samtools, 'faidx', reference, 'colon:2-7'],
        'missing-eof-marker': [samtools, 'faidx', no_marker], 'crc-damaged': [samtools, 'faidx', damaged]}
    for name, argv in diagnostics.items():
        observations[name] = command(root, 'observe-' + name, argv, required=False)
    save(root / 'casebook-observations.json', dict(status='required-source-chain-completed', representations=observations,
         diagnostics_are_observations_not_acceptance_policy=True, native_executed=False,
         real_reference_performance=False, empty_member_bytes=len(empty), empty_members_inserted=2400,
         source_bgzf_members=len(members)))


def execute() -> None:
    root = hosted_root()
    validate_workflow(ROOT / WORKFLOW)
    if shutil.disk_usage(root).free < 8 * 2**30:
        raise RuntimeError('less than8GiB free for the small source runtime/casebook')
    command(root, 'controller-head', ['git', '-c', 'safe.directory=' + str(ROOT),
                                    '-C', ROOT, 'rev-parse', 'HEAD'])
    if (root / 'calls/controller-head/stdout.txt').read_text().strip() != os.environ['GITHUB_SHA']:
        raise ValueError('actual checkout differs from dispatched controller head')
    controller = root / 'controller'
    controller.mkdir()
    for relative in (WORKFLOW, 'scripts/index_reference_source.py', 'scripts/plink_first_workflow_oracle.py',
                     'scripts/plink_first_workflow_inputs.py', 'docs/plans/2026-11-01-index-reference-workflow.md'):
        path = controller / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(ROOT / relative, path)
    groups = {name: (Path('/sys/fs/cgroup') / name).read_text().strip()
              for name in ('cpu.max', 'memory.max', 'memory.current', 'memory.peak') if (Path('/sys/fs/cgroup') / name).is_file()}
    quota, period = groups.get('cpu.max', '').split()
    if quota == 'max' or int(quota) != 4 * int(period) or int(groups.get('memory.max', '0')) != 8 * 2**30:
        raise ValueError('actual four-CPU/eight-GiB container bounds differ')
    save(root / 'machine.json', dict(platform=platform.platform(), python=sys.version,
         executable=str(Path(sys.executable).resolve()), executable_identity=identity(Path(sys.executable).resolve()),
         cgroup=groups, affinity=sorted(os.sched_getaffinity(0)), disk=shutil.disk_usage(root)._asdict(),
         limitations='Source-only engineering evidence; no performance/global-RSS/cold-cache/hard-loss guarantees.'))
    binaries = prepare_runtime(root)
    before = {name: identity(path) for name, path in binaries.items()}
    execute_casebook(root, binaries)
    if {name: identity(path) for name, path in binaries.items()} != before:
        raise ValueError('actual runtime executable changed during casebook')
    save(root / 'outcome.json', dict(status='source-casebook-completed', executables=before,
         controller=os.environ['GITHUB_SHA'], source_only=True, native_product_implemented=False))


def finalize() -> None:
    root = hosted_root()
    save(root / 'artifact-manifest.json', dict(schema='rsomics.index.reference-source.v1',
         controller=os.environ['GITHUB_SHA'], files=tree_files(root, ('artifact-manifest.json',)),
         outcome=load(root / 'outcome.json') if (root / 'outcome.json').exists() else None,
         retention='Public authored engineering fixtures, unmodified public upstream archives and source diagnostics; no provider reference/clinical inputs.'))
    package(root, Path(os.environ['INDEX_CARRIER']))


def package(root: Path, carrier: Path) -> None:
    if carrier.parent.resolve() != root.parent.resolve() or carrier == root:
        raise ValueError('source TAR must be a sibling of the closed evidence root')
    with tarfile.open(carrier, 'x', format=tarfile.PAX_FORMAT) as archive:
        for path in [root, *sorted(root.rglob('*'))]:
            name = Path('index-reference-source') / path.relative_to(root)
            member = archive.gettarinfo(path, arcname=name.as_posix())
            if path.is_file() and not path.is_symlink():
                member.type, member.linkname, member.size = tarfile.REGTYPE, '', path.stat().st_size
                with path.open('rb') as source:
                    archive.addfile(member, source)
            else:
                archive.addfile(member)


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
            if args.action == 'execute':
                save(Path(root) / 'outcome.json', dict(status='source-execution-failed',
                     native_product_implemented=False, exception=traceback.format_exc()))
        traceback.print_exc()
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
