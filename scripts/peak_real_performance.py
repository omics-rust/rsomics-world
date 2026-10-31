"""One admitted real-input Peak workflow and paired performance campaign."""

import argparse
from collections import Counter
import csv
import gzip
import hashlib
import itertools
import json
import os
from pathlib import Path
import platform
import re
import shlex
import shutil
import signal
import statistics
import subprocess
import sys
import tarfile
import time
import urllib.request
import zipfile

import peak_annotation_count_view as view
import peak_first_workflow_oracle as original
import peak_source_on_native as consumer
from peak_candidate import check, load, save, tree_manifest
from infercnv_source_snapshot import _hash_file
from signal_real_performance import proc_group, tree_bytes, meminfo, _wait4


ROOT = Path(__file__).resolve().parent
REPOSITORY = 'omics-rust/rsomics-world'
OLD_PREFIX = Path('/__w/_temp/peak-first-workflow-oracle')
GIB = 2**30
MEMORY_LIMIT = 8 * GIB
HEADROOM = 2 * GIB
CAMPAIGN_LIMIT = 24 * GIB
CHROMOSOMES = {'chr19': 58617616, 'chr20': 64444167, 'chr21': 46709983, 'chr22': 50818468}
WINDOWS = {'chr19': (10000000, 15000000), 'chr20': (10000000, 15000000),
           'chr21': (20000000, 25000000), 'chr22': (20000000, 25000000)}
PROVIDERS = [
    ('T1', 'https://www.encodeproject.org/files/ENCFF355CYX/@@download/ENCFF355CYX.bam',
     327141442, '718ec72812d5ae62d40c78dc93192a7c'),
    ('T2', 'https://www.encodeproject.org/files/ENCFF453EJM/@@download/ENCFF453EJM.bam',
     746711189, '0e1a42cade8de2eeb8e31eb9e9225dd6'),
    ('C', 'https://www.encodeproject.org/files/ENCFF850RIE/@@download/ENCFF850RIE.bam',
     257041985, '4c5929d34b6d75bc85a867fb478af536'),
    ('annotation', 'https://ftp.ebi.ac.uk/pub/databases/gencode/Gencode_human/release_49/gencode.v49.basic.annotation.gtf.gz',
     67888546, '544c7badad30a5428efee84bd7eb0553')]
CONTROLLERS = ('peak_real_performance.py', 'peak_real_performance.R', 'test_peak_real_performance.py',
    'peak_candidate.py', 'peak_source_on_native.py', 'peak_source_on_native.R',
    'peak_annotation_count_view.py', 'peak_annotation_count_view.R',
    'peak_first_workflow_oracle.py', 'peak_first_workflow_oracle.R',
    'signal_real_performance.py', 'signal_candidate.py', 'signal_first_workflow_oracle.py',
    'infercnv_source_snapshot.py', 'verify_infercnv_oracle_artifact.py')
CALL_SUFFIXES = ('_peaks.narrowPeak', '_summits.bed', '_peaks.xls',
                 '_treat_pileup.bdg', '_control_lambda.bdg')


def admit_old_prefix(prefix, runner):
    prefix, runner = Path(prefix).absolute(), Path(runner).resolve(strict=True)
    check(prefix.is_relative_to(runner) and prefix != runner and
          prefix.parent.resolve(strict=True) == prefix.parent and
          prefix.parent.stat().st_dev == runner.stat().st_dev,
          'original Python prefix is outside the actual runner route')
    check(not prefix.exists() and not prefix.is_symlink(), 'original Python prefix already exists')
    return prefix


def hosted(root):
    check(os.environ.get('GITHUB_ACTIONS') == 'true' and
          os.environ.get('GITHUB_REPOSITORY') == REPOSITORY and
          platform.system() == 'Linux' and platform.machine() == 'x86_64',
          'real Peak execution requires its hosted Linux x86_64 workflow')
    original.hosted(root)
    root = Path(root).resolve(strict=True)
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    check(root.is_relative_to(runner) and root != runner and root.stat().st_dev == runner.stat().st_dev,
          'campaign route escapes actual runner device')
    for name in ('HOME', 'XDG_CONFIG_HOME', 'XDG_DATA_HOME', 'UV_CACHE_DIR',
                 'PYTHONPYCACHEPREFIX', 'R_LIBS_SITE', 'CARGO_HOME', 'CARGO_TARGET_DIR', 'RUSTUP_HOME'):
        path = Path(os.environ[name]).resolve(strict=True)
        check(path.is_relative_to(runner) and path.stat().st_dev == runner.stat().st_dev,
              'write/cache route escapes runner device: ' + name)
    return root


def setup(root, cpus, idle):
    check(idle, 'otherwise-idle host confirmation is required')
    check(os.environ.get('GITHUB_ACTIONS') == 'true' and platform.system() == 'Linux' and
          platform.machine() == 'x86_64', 'setup is hosted Linux x86_64 only')
    root = Path(root).absolute()
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    check(root.parent.resolve(strict=True) == runner and not root.exists() and not root.is_symlink(),
          'new campaign child of actual runner temp required')
    assigned = [int(x) for x in cpus.split(',')]
    check(len(assigned) == len(set(assigned)) == 4 and set(assigned) <= os.sched_getaffinity(0),
          'four explicit actually allocated CPU IDs required')
    quota = Path('/sys/fs/cgroup/cpu.max')
    quota_text = quota.read_text().strip() if quota.is_file() else None
    if quota_text and quota_text.split()[0] != 'max':
        units, period = map(int, quota_text.split())
        check(units >= 4 * period, 'cgroup quota is smaller than four CPUs')
    check(shutil.disk_usage(runner).free >= 18 * GIB and meminfo()['MemAvailable'] >= 4 * GIB,
          'insufficient real campaign storage or available memory')
    root.mkdir()
    paths = {name: root / relative for name, relative in {
        'EVIDENCE': 'evidence', 'TMPDIR': 'scratch', 'HOME': 'home', 'XDG_CACHE_HOME': 'cache',
        'XDG_CONFIG_HOME': 'config', 'XDG_DATA_HOME': 'data', 'PIP_CACHE_DIR': 'cache/pip',
        'UV_CACHE_DIR': 'cache/uv', 'PYTHONUSERBASE': 'python-user', 'PYTHONPYCACHEPREFIX': 'cache/pyc',
        'MPLCONFIGDIR': 'cache/matplotlib', 'R_LIBS_USER': 'unused-r-library',
        'R_LIBS_SITE': 'unused-r-library', 'R_USER_CACHE_DIR': 'cache/R',
        'CARGO_HOME': 'unused-cargo', 'CARGO_TARGET_DIR': 'unused-target', 'RUSTUP_HOME': 'unused-rustup'}.items()}
    for path in paths.values():
        path.mkdir(parents=True, exist_ok=True)
    with (root / 'environment.sh').open('x') as stream:
        for name, path in paths.items():
            stream.write('export ' + name + '=' + shlex.quote(str(path)) + '\n')
    save(root / 'host-plan.json', dict(cpus=assigned, affinity=sorted(os.sched_getaffinity(0)),
        cgroup_cpu_max=quota_text, memory=meminfo(), disk=shutil.disk_usage(root)._asdict(),
        platform=platform.platform(), uname=list(platform.uname()), image=original.IMAGE,
        filesystem_device=root.stat().st_dev, mountinfo=Path('/proc/self/mountinfo').read_text(),
        filesystem=subprocess.run(['df', '-PT', str(root)], check=True, capture_output=True, text=True).stdout,
        memory_abort_bytes=MEMORY_LIMIT, campaign_abort_bytes=CAMPAIGN_LIMIT,
        minimum_free_bytes=HEADROOM, idle_operator_confirmed=True))


def committed_receipt(world, selector):
    world, relative = Path(world).resolve(strict=True), Path(selector)
    check(relative.as_posix().startswith('.autopilot/oracles/') and relative.suffix == '.json' and
          not relative.is_absolute() and '..' not in relative.parts, 'explicit tracked native receipt required')
    result = subprocess.run(['git', '-c', 'safe.directory=' + str(world),
                             'show', 'HEAD:' + relative.as_posix()], cwd=world, capture_output=True)
    check(result.returncode == 0, 'tracked native receipt Git read failed: ' +
          result.stderr.decode(errors='replace').strip())
    blob = result.stdout
    check(blob == (world / relative).read_bytes(), 'native adoption differs from its committed body')
    receipt = json.loads(blob)
    check(receipt['schema'] == 'rsomics.peak.native-adoption.v1' and
          receipt['repository'] == REPOSITORY and receipt['workflow_path'] == '.github/workflows/peak-candidate.yml' and
          receipt['four_native_and_source_passed'] is True and receipt['run_attempt'] >= 1,
          'native-four/source adoption has not passed')
    for name in ('head_sha', 'artifact_sha256', 'binary_sha256', 'source_archive_sha256', 'source_manifest_sha256'):
        check(re.fullmatch('[0-9a-f]{40}' if name == 'head_sha' else '[0-9a-f]{64}', receipt[name]),
              'invalid native receipt identity: ' + name)
    return receipt, dict(path=relative.as_posix(), sha256=hashlib.sha256(blob).hexdigest())


def acquire_native(root, world, receipt):
    target = root / 'acquisition/native'; target.mkdir(parents=True)
    run = view.api(f"actions/runs/{receipt['run_id']}", target / 'run.json')
    check((run['id'], run['run_attempt'], run['head_sha'], run['status'], run['conclusion']) ==
          (receipt['run_id'], receipt['run_attempt'], receipt['head_sha'], 'completed', 'success') and
          run['repository']['full_name'] == REPOSITORY and run['path'] == receipt['workflow_path'],
          'current native run differs from adoption')
    jobs = view.api(f"actions/runs/{receipt['run_id']}/attempts/{receipt['run_attempt']}/jobs?per_page=100",
                    target / 'jobs.json')
    check(jobs['total_count'] == 5 and len(jobs['jobs']) == 5 and
          {job['name'] for job in jobs['jobs']} == {'linux-x86_64', 'linux-aarch64', 'macos-x86_64',
            'macos-aarch64', 'Linux pinned source reads same native files'} and
          all(job['conclusion'] == 'success' for job in jobs['jobs']), 'all four native and same-file source jobs required')
    item = view.api(f"actions/artifacts/{receipt['artifact_id']}", target / 'artifact.json')
    check(not item['expired'] and item['name'] == receipt['artifact_name'] ==
          f"peak-native-{receipt['run_id']}-{receipt['run_attempt']}-linux-x86_64" and
          item['size_in_bytes'] == receipt['size_in_bytes'] and item['digest'] == 'sha256:' + receipt['artifact_sha256'] and
          item['workflow_run']['id'] == receipt['run_id'] and item['workflow_run']['head_sha'] == receipt['head_sha'],
          'current native artifact binding differs')
    archive = target / 'original.zip'
    view.api(f"actions/artifacts/{receipt['artifact_id']}/zip", archive, binary=True)
    check(archive.stat().st_size == receipt['size_in_bytes'] and _hash_file(archive) == receipt['artifact_sha256'],
          'native original ZIP differs')
    carrier = target / 'peak-native-linux-x86_64.tar'
    with zipfile.ZipFile(archive) as zipped:
        check(zipped.namelist() == [carrier.name] and zipped.testzip() is None, 'native original ZIP layout/CRC differs')
        with zipped.open(carrier.name) as source, carrier.open('xb') as output:
            shutil.copyfileobj(source, output, 2**20)
    previous = os.environ.get('GITHUB_SHA')
    try:
        os.environ['GITHUB_SHA'] = receipt['head_sha']
        native, binding = consumer.extract_native(carrier, target / 'restored', world)
    finally:
        if previous is None: os.environ.pop('GITHUB_SHA', None)
        else: os.environ['GITHUB_SHA'] = previous
    check(_hash_file(native / 'binary-release') == receipt['binary_sha256'] and
          _hash_file(native / 'files.sha256') == receipt['source_manifest_sha256'] and
          _hash_file(native / 'source.tar.gz') == receipt['source_archive_sha256'], 'native executable/source differs from adoption')
    binary = root / 'binary'; shutil.copy2(native / 'binary-release', binary); binary.chmod(0o755)
    save(root / 'evidence/native-admission.json', dict(receipt=receipt, binding=binding, binary=str(binary)))
    return binary


def restore_python(work, prefix):
    admit_old_prefix(prefix, os.environ['RUNNER_TEMP'])
    manifest = load(work / 'restoration.json')['original_manifest']
    selected = {name: row for name, row in manifest.items()
                if name.startswith(('runtime/python/', 'runtime/venv/', 'runtime/built-wheels/')) or
                name in ('runtime/venv-binding.json', 'runtime/SEACR_1.3.sh', 'runtime/SEACR_1.3.R',
                         'runtime/bedtools-build/bin/bedtools')}
    check(selected and any(name.startswith('runtime/python/') for name in selected), 'original Python closure absent')
    prefix.mkdir()
    links = {}
    with tarfile.open(work / 'original' / view.TAR_MEMBER, 'r:') as archive:
        for name, identity in selected.items():
            member = archive.getmember(view.PREFIX + name)
            target = prefix / name
            target.parent.mkdir(parents=True, exist_ok=True)
            if 'symlink' in identity:
                check(member.issym() and member.linkname == identity['symlink'], 'original Python link differs: ' + name)
                resolved = Path(os.path.normpath(str(target.parent / member.linkname)))
                check(resolved.is_relative_to(prefix), 'Python link escapes admitted original prefix: ' + name)
                links[target] = member.linkname
            else:
                check(member.isfile() and member.size == identity['size'], 'original Python body size differs: ' + name)
                with archive.extractfile(member) as source, target.open('xb') as output:
                    shutil.copyfileobj(source, output, 2**20)
                target.chmod(member.mode & 0o777)
                check(_hash_file(target) == identity['sha256'], 'original Python body differs: ' + name)
    for target, link in links.items():
        target.symlink_to(link)
    check(all(path.resolve(strict=True).is_relative_to(prefix) for path in links), 'restored Python alias does not resolve internally')
    final = load(work / 'restored/runtime/final-runtime-manifest.json')
    runtime = prefix / 'runtime'
    check(original.files_identity(runtime / 'venv') == final['python_environment'], 'installed Python environment differs')
    check(_hash_file(runtime / 'python/bin/python3.12') == 'd3edc79066c9c08a4e4ff74116def227d423ce639f64af1a939862e787fc0dc3' and
          _hash_file(runtime / 'venv/pyvenv.cfg') == '87303dd4b1db5a4162745c556804262fc32b5a42b6c4aba68987c5ad18367dda' and
          _hash_file(runtime / 'built-wheels/macs3-3.1.0-cp312-cp312-linux_x86_64.whl') ==
          '6a44e78957c425cf3b203e9e93c8f8eba2c34fbb1222de9da1c509e4a01b6048', 'pinned Python/MACS/config identity differs')
    return runtime


def version_needs(text):
    output, library, active = {}, None, False
    for line in text.splitlines():
        if line.startswith('Version '): active = line.startswith('Version needs section:') or line.startswith('Version needs section ')
        if not active: continue
        if match := re.search(r'\bFile:\s+(\S+)', line):
            library = match.group(1); output.setdefault(library, set())
        if match := re.search(r'\bName:\s+(\S+)', line):
            check(library is not None, 'ELF needed version lacks provider'); output[library].add(match.group(1))
    return output


def check_version_closure(needs, providers):
    for library, required in needs.items():
        check(library in providers, 'ELF provider absent: ' + library)
        section = providers[library].split('Version definition section', 1)
        supplied = set(re.findall(r'\bName:\s+(\S+)', section[1].split('Version needs section', 1)[0])) if len(section) == 2 else set()
        missing = required - supplied
        check(not missing, 'ELF provider ' + library + ' lacks ' + ','.join(sorted(missing)))


def loader_providers(text, interpreter):
    paths = dict(re.findall(r'^\s*(\S+)\s+=>\s+(/\S+)\s+\(0x[0-9a-fA-F]+\)\s*$', text, re.MULTILINE))
    standalone = re.findall(r'^\s*(/\S+)\s+\(0x[0-9a-fA-F]+\)\s*$', text, re.MULTILINE)
    if interpreter in standalone:
        paths[Path(interpreter).name] = interpreter
    return paths


def admit_elf(binary, output, probe_version=True):
    output.mkdir()
    original.require(output / 'readelf', ['readelf', '-hldV', binary])
    text = (output / 'readelf/stdout.txt').read_text()
    check('Advanced Micro Devices X86-64' in text, 'native executable is not x86_64 ELF')
    interpreters = re.findall(r'Requesting program interpreter:\s*([^\]]+)', text)
    check(len(interpreters) == 1 and Path(interpreters[0]).is_file(), 'native ELF interpreter absent/ambiguous')
    check(not re.search(r'\((?:RPATH|RUNPATH)\)', text), 'unadmitted native ELF runtime search override')
    original.require(output / 'ldconfig', ['ldconfig', '-p'])
    table = (output / 'ldconfig/stdout.txt').read_text()
    needed = re.findall(r'\(NEEDED\).*?Shared library: \[([^\]]+)\]', text)
    paths = {}
    for name in needed:
        matches = re.findall(r'^\s*' + re.escape(name) + r'\s+\([^\n]*x86-64[^\n]*\)\s+=>\s+(\S+)', table, re.MULTILINE)
        check(matches, 'native ELF shared library absent: ' + name)
        paths[name] = Path(matches[0]).resolve(strict=True)
    providers = {}
    for index, (name, path) in enumerate(paths.items()):
        original.require(output / 'providers' / str(index), ['readelf', '-V', path])
        providers[name] = (output / 'providers' / str(index) / 'stdout.txt').read_text()
    check_version_closure(version_needs(text), providers)
    original.require(output / 'loader-list', [interpreters[0], '--list', binary])
    loaded = (output / 'loader-list/stdout.txt').read_text()
    check('not found' not in loaded, 'static-admitted native ELF loader closure failed')
    actual_paths = loader_providers(loaded, interpreters[0])
    for name, path in paths.items():
        actual = actual_paths.get(name)
        check(actual is not None and Path(actual).resolve(strict=True) == path, 'ELF actual provider differs: ' + name)
    save(output / 'admission.json', dict(binary_sha256=_hash_file(binary), interpreter=interpreters[0],
        providers={name: dict(path=str(path), sha256=_hash_file(path)) for name, path in paths.items()},
        required_versions={name: sorted(values) for name, values in version_needs(text).items()},
        scientific_binary_executed_before_static_admission=False))
    if probe_version:
        original.require(output / 'native-version', [binary, '--version'])


def acquire(root, world, selector):
    root = hosted(root)
    receipt, adoption = committed_receipt(world, selector)
    save(root / 'evidence/receipt-adoption.json', adoption)
    controller = root / 'evidence/controller'; controller.mkdir()
    for name in CONTROLLERS:
        shutil.copy2(ROOT / name, controller / name)
    shutil.copy2(Path(world) / '.github/workflows/peak-real-performance.yml', controller / 'peak-real-performance.yml')
    save(controller / 'checksums.json', tree_manifest(controller))
    binary = acquire_native(root, world, receipt)
    work = root / 'acquisition/source'; work.mkdir()
    restored = view.acquire(work)
    check(_hash_file(ROOT / 'peak_first_workflow_oracle.R') ==
          _hash_file(restored / 'controller/peak_first_workflow_oracle.R'),
          'annotation scientific wrapper differs from the accepted original')
    consumer.restore_seacr(work, restored)
    runtime = restore_python(work, OLD_PREFIX)
    os.environ.pop('GH_TOKEN', None)
    os.environ.update(R_LIBS_USER=str(restored / 'R-library'), R_LIBS_SITE=str(restored / 'R-library'))
    body_hashes = view.verify_runtime(restored, root / 'evidence')
    admit_elf(binary, root / 'evidence/native-elf')
    probe = '''import hashlib,importlib,importlib.metadata,json,os,sys
from pathlib import Path
modules={name:importlib.import_module(name) for name in ('MACS3','pysam','numpy','scipy')}
for name in ('MACS3.Commands.callpeak_cmd','MACS3.IO.Parser','MACS3.Signal.PeakModel','MACS3.Signal.PeakDetect'):
    importlib.import_module(name)
extensions={name:module.__file__ for name,module in sys.modules.items() if (getattr(module,'__file__',None) or '').endswith('.so')}
mapped=sorted({line.split()[-1] for line in Path('/proc/self/maps').read_text().splitlines() if line.split()[-1].startswith('/')})
versions={name:importlib.metadata.version(name) for name in modules}
print(json.dumps(dict(executable=sys.executable,prefix=sys.prefix,base_prefix=sys.base_prefix,path=sys.path,
versions=versions,modules={name:module.__file__ for name,module in modules.items()},
extensions={name:dict(path=path,sha256=hashlib.sha256(Path(path).read_bytes()).hexdigest()) for name,path in extensions.items()},
mapped={path:hashlib.sha256(Path(path).read_bytes()).hexdigest() for path in mapped if Path(path).is_file()}),sort_keys=True))'''
    original.require(root / 'evidence/runtime-python', [runtime / 'venv/bin/python', '-B', '-c', probe])
    observed = load(root / 'evidence/runtime-python/stdout.txt')
    check(observed['prefix'] == str(runtime / 'venv') and observed['base_prefix'] == str(runtime / 'python') and
          observed['versions'] == {'MACS3': '3.1.0', 'pysam': '0.23.3', 'numpy': '2.2.6', 'scipy': '1.15.3'} and
          all(Path(path).is_relative_to(runtime / 'venv') for path in observed['modules'].values()),
          'actual source Python prefix/module/version binding differs')
    for index, (name, row) in enumerate(observed['extensions'].items()):
        check(Path(row['path']).is_relative_to(runtime / 'venv') or
              Path(row['path']).is_relative_to(runtime / 'python'), 'Python extension escapes exact original prefix: ' + name)
        original.require(root / 'evidence/python-ldd' / str(index), ['ldd', row['path']])
        text = (root / 'evidence/python-ldd' / str(index) / 'stdout.txt').read_text()
        check('not found' not in text, 'restored Python extension has missing library: ' + name)
        body_hashes[row['path']] = row['sha256']
    save(root / 'evidence/runtime-before.json', dict(restored=original.files_identity(restored),
        exact_prefix=original.files_identity(OLD_PREFIX), system_bodies=body_hashes, mapped_python=observed['mapped']))
    save(root / 'runtime.json', dict(runtime=str(runtime), restored=str(restored), binary=str(binary)))
    with (root / 'runtime-environment.sh').open('x') as stream:
        for name in ('R_LIBS_USER', 'R_LIBS_SITE'):
            stream.write('export ' + name + '=' + shlex.quote(os.environ[name]) + '\n')
        stream.write('export PATH=' + shlex.quote(str(runtime / 'venv/bin')) + ':"$PATH"\n')
    providers = root / 'providers'; providers.mkdir()
    metadata = root / 'evidence/provider-metadata'; metadata.mkdir()
    for accession, kind in [('ENCSR000AKB', 'experiments'), ('ENCSR000AKJ', 'experiments'),
                             ('ENCFF875FTQ', 'files'), *[(url.split('/files/')[1].split('/')[0], 'files')
                                for role, url, _, _ in PROVIDERS if role != 'annotation']]:
        url = f'https://www.encodeproject.org/{kind}/{accession}/?format=json&frame=object'
        with urllib.request.urlopen(url, timeout=120) as response:
            body = response.read(8 * 2**20 + 1)
        check(len(body) <= 8 * 2**20, 'provider metadata exceeds bound')
        (metadata / (accession + '.json')).write_bytes(body)
    peak = load(metadata / 'ENCFF875FTQ.json')
    check(all('/files/' + url.split('/files/')[1].split('/')[0] + '/' in peak['derived_from']
              for role, url, _, _ in PROVIDERS if role != 'annotation'), 'official file-level control pairing differs')
    for name, url in [('encode-data-use.html', 'https://www.encodeproject.org/about/data-use-policy/'),
                      ('gencode-data-use.html', 'https://www.gencodegenes.org/pages/data_access.html')]:
        with urllib.request.urlopen(url, timeout=120) as response: body = response.read(8 * 2**20 + 1)
        check(len(body) <= 8 * 2**20, 'provider notice exceeds bound')
        (metadata / name).write_bytes(body)
    identities = {}
    for role, url, size, md5 in PROVIDERS:
        target = providers / (role + ('.gtf.gz' if role == 'annotation' else '.bam'))
        request = urllib.request.Request(url, headers={'User-Agent': 'rsomics-real-workflow/1'})
        with urllib.request.urlopen(request, timeout=120) as source, target.open('xb') as output:
            check(source.url.startswith('https://'), 'provider redirect is not HTTPS')
            shutil.copyfileobj(source, output, 2**20)
        actual = hashlib.md5()
        with target.open('rb') as stream:
            for block in iter(lambda: stream.read(2**20), b''): actual.update(block)
        check(target.stat().st_size == size and actual.hexdigest() == md5, 'provider body differs: ' + role)
        identities[role] = dict(url=url, bytes=size, md5=md5, sha256=_hash_file(target), path=str(target))
    save(root / 'evidence/provider-identities.json', identities)


def materialize(root):
    root = hosted(root)
    import pysam
    check(pysam.__version__ == '0.23.3', 'pinned pysam required')
    runtime = Path(load(root / 'runtime.json')['runtime'])
    check(Path(sys.prefix) == runtime / 'venv' and Path(sys.base_prefix) == runtime / 'python',
          'input materialization must use the restored bound interpreter')
    output = root / 'evidence/inputs'; output.mkdir()
    observations, pooled = {}, {'treatment': {chrom: Counter() for chrom in WINDOWS},
                               'control': {chrom: Counter() for chrom in WINDOWS}}
    for sample in ('T1', 'T2', 'C'):
        source_path = root / 'providers' / (sample + '.bam')
        retained_hash = hashlib.sha256(); records = Counter(); eligible = Counter()
        with pysam.AlignmentFile(source_path, 'rb') as source:
            dictionary = dict(zip(source.references, source.lengths))
            check(all(dictionary.get(name) == length for name, length in CHROMOSOMES.items()), 'real BAM dictionary differs: ' + sample)
            header = source.header.to_dict()
            with pysam.AlignmentFile(output / (sample + '.bam'), 'wb', header=source.header) as target:
                raw = 0
                for record in source.fetch(until_eof=True):
                    raw += 1
                    chrom = record.reference_name
                    if chrom not in WINDOWS: continue
                    start, end = WINDOWS[chrom]
                    if not start <= record.reference_start < end: continue
                    target.write(record); records[chrom] += 1
                    retained_hash.update(record.to_string().encode() + b'\n')
                    flags = record.flag
                    if flags & 2820 or (flags & 1 and (flags & 136 or not flags & 2)): continue
                    check(record.reference_end is not None and record.reference_start < record.reference_end <= dictionary[chrom],
                          'invalid caller-admitted real alignment geometry')
                    key = (bool(flags & 16), record.reference_end if flags & 16 else record.reference_start)
                    pooled['control' if sample == 'C' else 'treatment'][chrom][key] += 1
                    eligible[chrom] += 1
        reread_hash = hashlib.sha256()
        with pysam.AlignmentFile(output / (sample + '.bam'), 'rb') as reread:
            check(reread.header.to_dict() == header, 'slice changed BAM dictionary/header')
            for record in reread.fetch(until_eof=True): reread_hash.update(record.to_string().encode() + b'\n')
        check(reread_hash.digest() == retained_hash.digest(), 'slice record values/order changed')
        pysam.index(str(output / (sample + '.bam')))
        observations[sample] = dict(provider_raw=raw, retained_by_chromosome=dict(records),
            caller_eligible_by_chromosome=dict(eligible), record_values_sha256=retained_hash.hexdigest(), header=header)
    histogram = {}
    for population, chromosomes in pooled.items():
        admitted = sum(sum(values.values()) for values in chromosomes.values())
        check(0 < admitted <= 8388608, 'real caller input exceeds unchanged128MiB tag budget')
        histogram[population] = {chrom: {str(copies): sites for copies, sites in Counter(values.values()).items()}
                                 for chrom, values in chromosomes.items()}
    features = Counter(); lines = 0
    with gzip.open(root / 'providers/annotation.gtf.gz', 'rt') as source, (output / 'annotation.gtf').open('x') as target:
        for line in source:
            if line.startswith('#'):
                target.write(line); continue
            fields = line.rstrip('\n').split('\t')
            check(len(fields) == 9, 'real GTF record width differs')
            if fields[0] in CHROMOSOMES:
                target.write(line); features[fields[2]] += 1; lines += 1
    check(features['transcript'] > 0 and features['exon'] > 0 and features['CDS'] > 0, 'real annotation subset is not substantial')
    with (output / 'chrominfo.tsv').open('x') as stream:
        stream.write('chrom\tlength\tis_circular\n')
        for chrom, length in CHROMOSOMES.items(): stream.write(f'{chrom}\t{length}\tFALSE\n')
    save(output / 'admission.json', dict(windows=WINDOWS, chromosomes=CHROMOSOMES, observations=observations,
        duplicate_histogram=histogram, annotation=dict(lines=lines, features=dict(features)),
        derivative_files=tree_manifest(output), materializer_sha256=_hash_file(__file__),
        randomized=False, remapped=False, replicated=False, numerical_qualification=False))


def graph(tool, workers, inputs, products, binary, runtime, profile='seacr-1.3', include_call=True):
    check(tool in ('native', 'source') and workers in (1, 4), 'explicit tool and1/4 budget required')
    check(profile in ('seacr-1.3', 'corrected') and (tool == 'native' or profile == 'seacr-1.3'), 'corrected source is not adopted')
    inputs, products, binary, runtime = map(Path, (inputs, products, binary, runtime))
    call = products / 'call'; rows = products / 'rows'; plan = []
    python = runtime / 'venv/bin/python'
    def add(name, argv, outputs, stdout=None):
        plan.append(dict(name=name, argv=list(map(str, argv)), outputs=list(map(str, outputs)),
                         stdout=str(stdout) if stdout else None))
    if include_call:
        if tool == 'native':
            args = [binary, 'call', '--treatment', inputs / 'T1.bam', inputs / 'T2.bam', '--control', inputs / 'C.bam',
                '--format', 'bam', '--model', 'fitted', '--genome-size', '20000000', '--duplicates', 'auto',
                '--bandwidth', '300', '--mfold', '5', '50', '--d-min', '20', '--qvalue', '0.05',
                '--name', 'peaks', '--output-directory', call, '--write-tracks', '--threads', workers]
        else:
            args = [python, '-B', runtime / 'venv/bin/macs3', 'callpeak', '-t', inputs / 'T1.bam', inputs / 'T2.bam',
                '-c', inputs / 'C.bam', '-f', 'BAM', '-g', '20000000', '--keep-dup', 'auto', '--bw', '300',
                '--mfold', '5', '50', '--d-min', '20', '-q', '0.05', '-B', '-n', 'peaks', '--outdir', call,
                '--threads', workers, '--process-backend', 'serial' if workers == 1 else 'auto',
                '--tempdir', os.environ.get('TMPDIR', str(products / 'scratch'))]
        add('call', args, [call / ('peaks' + suffix) for suffix in CALL_SUFFIXES])
    if tool == 'native':
        args = [binary, 'call-sparse', '--target', call / 'peaks_treat_pileup.bdg', '--fraction', '0.10',
            '--normalization', 'non', '--mode', 'stringent', '--compatibility', profile,
            '--output', products / 'sparse.stringent.bed', '--report', products / 'sparse.json', '--threads', workers]
    else:
        args = ['bash', runtime / 'SEACR_1.3.sh', call / 'peaks_treat_pileup.bdg', '0.10', 'non', 'stringent', products / 'sparse']
    add('sparse', args, [products / 'sparse.stringent.bed'])
    add('rows', [python, '-B', __file__, 'rows', '--called', call / 'peaks_peaks.narrowPeak',
        '--sparse', products / 'sparse.stringent.bed', '--output', rows], [rows / 'workflow.bed', rows / 'workflow-rows.tsv'])
    if tool == 'native':
        args = [binary, 'annotate', '--peaks', rows / 'workflow.bed', '--annotation', inputs / 'annotation.gtf',
            '--format', 'gtf', '--tss-upstream', '3000', '--tss-downstream', '3000', '--downstream', '3000',
            '--output', products / 'annotation.tsv', '--report', products / 'annotation.json', '--threads', workers]
        add('annotate', args, [products / 'annotation.tsv'])
        add('count', [binary, 'count', '--peaks', rows / 'workflow.bed', '--sample', *[
            name + '=' + str(inputs / (name + '.bam')) for name in ('T1', 'T2', 'C')],
            '--output', products / 'counts.tsv', '--report', products / 'counts.json', '--threads', workers], [products / 'counts.tsv'])
    else:
        add('annotate', ['Rscript', '--vanilla', ROOT / 'peak_real_performance.R',
            inputs / 'annotation.gtf', rows, products / 'annotation', inputs / 'chrominfo.tsv'],
            [products / 'annotation/core.tsv', products / 'annotation/detail.tsv'])
        add('count', [runtime / 'bedtools-build/bin/bedtools', 'multicov', '-bams', *[
            inputs / (name + '.bam') for name in ('T1', 'T2', 'C')], '-bed', rows / 'workflow.bed'],
            [products / 'counts.tsv'], stdout=products / 'counts.tsv')
    return plan


def write_rows(called, sparse, output):
    output = Path(output); output.mkdir()
    ordinal = 0; populations = {}
    with (output / 'workflow.bed').open('x') as bed, (output / 'workflow-rows.tsv').open('x', newline='') as stream:
        ledger = csv.writer(stream, delimiter='\t', lineterminator='\n')
        ledger.writerow(['row_key', 'row_ordinal', 'source', 'source_row', 'display_id', 'original_fields_json'])
        for origin, path, width in [('call', called, 10), ('sparse', sparse, 6)]:
            count = 0
            for number, line in enumerate(Path(path).read_text().splitlines()):
                fields = line.split('\t')
                check(len(fields) == width and fields[0] in CHROMOSOMES and
                      0 <= int(fields[1]) < int(fields[2]) <= CHROMOSOMES[fields[0]], 'peak row outside adopted real dictionary')
                display = f'{origin}-{number}'
                bed.write('\t'.join([*fields[:3], display, '0', '.']) + '\n')
                ledger.writerow([f'{origin}:{number}', ordinal, origin, number, display, json.dumps(fields)])
                ordinal += 1; count += 1
            check(count > 0, origin + ' real rows are empty'); populations[origin] = count
    save(output / 'derivation.json', dict(rows=ordinal, populations=populations,
        sources={str(path): _hash_file(path) for path in (called, sparse)},
        deduplicated=False, added_rows=0, sparse_field6='max-region, not strand'))


def downstream_graph(tool, workers, inputs, products, rows, binary, runtime):
    products, rows = Path(products), Path(rows)
    stages = graph(tool, workers, inputs, products, binary, runtime)[-2:]
    for stage in stages:
        stage['argv'] = [value.replace(str(products / 'rows'), str(rows)) for value in stage['argv']]
    return stages


def compare_counts(native, source, bed):
    native_rows = consumer.lines(native)
    header = next(native_rows)[1]
    check(header[-3:] == ['T1', 'T2', 'C'], 'count sample columns differ')
    count = 0
    for left, right, original_row in itertools.zip_longest(native_rows, consumer.lines(source), consumer.lines(bed)):
        check(left is not None and right is not None and original_row is not None, 'count row population differs')
        number, a = left; _, b = right; _, row = original_row
        check(len(a) == len(b) == 9 and a[:6] == b[:6] == row, 'count row identity differs at' + str(number))
        check([int(x) for x in a[6:]] == [int(x) for x in b[6:]], 'count cells differ at' + str(number))
        count += 1
    return count


def execute_graph(plan_path, cpus):
    plan = load(plan_path); os.sched_setaffinity(0, cpus); measurements = []
    for stage in plan['stages']:
        directory = Path(plan['logs']) / stage['name']; directory.mkdir(parents=True)
        stdout_path = Path(stage['stdout']) if stage['stdout'] else directory / 'stdout.txt'
        with stdout_path.open('xb') as stdout, (directory / 'stderr.txt').open('xb') as stderr:
            result = _wait4(stage['argv'], stdout, stderr, cpus)
        result['name'] = stage['name']; save(directory / 'measurement.json', result)
        check(result['exit'] == 0, 'required real graph stage failed: ' + stage['name'])
        check(all(Path(path).is_file() for path in stage['outputs']), 'real graph output absent: ' + stage['name'])
        measurements.append(result)
    save(Path(plan['logs']).parent / 'stages.json', measurements)


def host_state(cpus):
    quota = Path('/sys/fs/cgroup/cpu.max')
    return dict(monotonic_ns=time.monotonic_ns(), unix_ns=time.time_ns(), cpus=cpus,
        affinity=sorted(os.sched_getaffinity(0)), load=list(os.getloadavg()), memory=meminfo(),
        cpu_model=Path('/proc/cpuinfo').read_text(), cgroup_cpu_max=quota.read_text() if quota.exists() else None,
        competing_processes=subprocess.run(['ps', '-eo', 'pid,ppid,comm,pcpu,rss'], check=True, capture_output=True, text=True).stdout)


def optional_cgroup(parent, directory):
    if parent is None: return None
    parent = Path(parent).resolve(strict=True)
    check(parent.is_relative_to('/sys/fs/cgroup') and 'memory' in (parent / 'cgroup.controllers').read_text().split(),
          'an existing delegated cgroup-v2 memory parent is required')
    group = parent / ('peak-' + hashlib.sha256(str(directory).encode()).hexdigest()[:20]); group.mkdir()
    (group / 'memory.max').write_text(str(MEMORY_LIMIT)); (group / 'memory.oom.group').write_text('1')
    check((group / 'memory.peak').is_file() and (group / 'memory.peak').read_text().strip() == '0', 'fresh exact cgroup peak unavailable')
    return group


def trial(root, directory, tool, workers, inputs, runtime, binary, cpus, cgroup_parent=None,
          profile='seacr-1.3', stages=None):
    directory.mkdir(parents=True)
    products = directory / 'products'; products.mkdir(); (products / 'call').mkdir()
    scratch = directory / 'scratch'; scratch.mkdir()
    plan = stages if stages is not None else graph(tool, workers, inputs, products, binary, runtime, profile)
    save(directory / 'plan.json', dict(stages=plan, logs=str(directory / 'commands'), tool=tool,
        workers=workers, profile=profile, cache='warm-cache', fsync_durability_claim=False))
    group = optional_cgroup(cgroup_parent, directory)
    argv = [str(runtime / 'venv/bin/python'), '-B', __file__, 'execute-graph', '--plan', str(directory / 'plan.json'),
            '--cpus', ','.join(map(str, cpus))]
    def enter():
        os.sched_setaffinity(0, cpus)
        if group is not None: (group / 'cgroup.procs').write_text(str(os.getpid()))
    child = None; usage = None; failure = None; peak = 0
    save(directory / 'host-before.json', host_state(cpus))
    started = time.monotonic_ns()
    with (directory / 'runner.stdout').open('xb') as stdout, (directory / 'runner.stderr').open('xb') as stderr:
        try:
            child = subprocess.Popen(argv, stdout=stdout, stderr=stderr, env=dict(os.environ, TMPDIR=str(scratch)),
                                     start_new_session=True, preexec_fn=enter)
            while True:
                sampled = sum(row['rss_bytes'] for row in proc_group(child.pid)); peak = max(peak, sampled)
                check(sampled <= MEMORY_LIMIT and meminfo()['MemAvailable'] >= 512 * 2**20, 'real workflow memory abort budget reached')
                check(tree_bytes(root) <= CAMPAIGN_LIMIT and shutil.disk_usage(root).free >= HEADROOM, 'real campaign storage abort budget reached')
                result = os.wait4(child.pid, os.WNOHANG)
                if result[0]:
                    _, status, usage = result; child.returncode = os.waitstatus_to_exitcode(status); break
                check(time.monotonic_ns() - started < 3600 * 10**9, 'real graph exceeded one-hour trusted-child limit')
                time.sleep(0.5)
            check(child.returncode == 0 and not proc_group(child.pid), 'complete real graph failed or left live descendants')
        except BaseException as error:
            failure = dict(type=type(error).__name__, message=str(error))
            if child is not None:
                for sig in (signal.SIGTERM, signal.SIGKILL):
                    try: os.killpg(child.pid, sig)
                    except ProcessLookupError: pass
                if child.returncode is None:
                    _, status, usage = os.wait4(child.pid, 0); child.returncode = os.waitstatus_to_exitcode(status)
            raise
        finally:
            wall = (time.monotonic_ns() - started) / 1e9
            result = dict(tool=tool, workers=workers, profile=profile, argv=argv,
                wall_seconds=wall, exit=child.returncode if child else None,
                user_seconds=usage.ru_utime if usage else None, system_seconds=usage.ru_stime if usage else None,
                cpu_utilization_percent=(usage.ru_utime + usage.ru_stime) / wall * 100 if usage else None,
                sampled_sum_rss_peak_bytes=peak, sampled_rss_scope='live group sum; shared pages double-counted; abort guard only',
                aggregate_tree_peak_bytes=int((group / 'memory.peak').read_text()) if group else None,
                aggregate_tree_peak_scope='dedicated cgroup-v2 hierarchical memory.peak' if group else None,
                cgroup=str(group) if group else None, failure=failure)
            save(directory / 'measurement.json', result); save(directory / 'host-after.json', host_state(cpus))
    return result


def source_model(text):
    fitted = re.findall(r'predicted fragment length is (\d+)', text)
    pairs = re.findall(r'Total number of paired peaks:\s*(\d+)', text)
    check(fitted and pairs and int(fitted[-1]) > 0 and int(pairs[-1]) >= 100,
          'real source caller lacks an actual fitted model with at least100 pairs')
    return dict(d=int(fitted[-1]), paired_peaks=int(pairs[-1]))


def compare_pair(source, native, output, runtime):
    source, native = Path(source) / 'products', Path(native) / 'products'; output.mkdir(parents=True)
    compared = {}
    for suffix in CALL_SUFFIXES:
        def body(path):
            return [line for line in path.read_text().splitlines() if line and not line.startswith('#')]
        a, b = [body(root / 'call' / ('peaks' + suffix)) for root in (source, native)]
        check(a == b and len(a) > 0, 'full real MACS table differs: ' + suffix); compared[suffix] = len(a)
    compared['sparse_rows'] = consumer.compare_sparse(native / 'sparse.stringent.bed', source / 'sparse.stringent.bed')
    for name in ('workflow.bed', 'workflow-rows.tsv'):
        left, right = [root / 'rows' / name for root in (source, native)]
        if name == 'workflow.bed': check(left.read_bytes() == right.read_bytes(), 'independently generated P row identity differs')
        else:
            with left.open() as a, right.open() as b:
                fields = ('row_key', 'row_ordinal', 'source', 'source_row', 'display_id')
                rows = itertools.zip_longest(csv.DictReader(a, delimiter='\t'), csv.DictReader(b, delimiter='\t'))
                for first, second in rows:
                    check(first is not None and second is not None and all(first[x] == second[x] for x in fields), 'P row ledger identity differs')
    compared['annotation'] = consumer.compare_annotation(native / 'annotation.tsv', source / 'annotation/core.tsv',
        source / 'annotation/detail.tsv', native / 'rows/workflow-rows.tsv')
    compared['counts'] = compare_counts(native / 'counts.tsv', source / 'counts.tsv', native / 'rows/workflow.bed')
    model = load(native / 'call/peaks_call.json')['model']
    check(not model['fallback_used'] and model['fitted_d'] == model['d'] and model['paired_peaks'] >= 100,
          'real native caller did not fit at least100 model pairs')
    log = (source.parent / 'commands/call/stderr.txt').read_text()
    source_fit = source_model(log)
    check(source_fit['d'] == model['d'], 'real source caller fit differs')
    report = load(native / 'call/peaks_call.json')
    check(report['workers_used'] == report['workers_requested'] and report['workers_used'] in (1, 4), 'four useful real caller chromosomes not exercised')
    count_report, annotation_report, sparse_report = [load(native / name) for name in
        ('counts.json', 'annotation.json', 'sparse.json')]
    workers = report['workers_requested']
    check(count_report['requested_threads'] == workers and count_report['effective_workers'] == min(workers, 3),
          'real count sample-task provenance differs')
    save(output / 'qualification.json', dict(passed=True, compared=compared, native_model=model,
        source_model=source_fit, report=report, count_report=count_report,
        annotation_report=annotation_report, sparse_report=sparse_report,
        call_comparison='all complete scientific wire tables exact; XLS comments excluded only',
        sparse_comparison='same seacr-1.3 rows/coordinates/six-digit values; corrected is separate'))
    return report


def corrected_view(root, workers, qualification, inputs, runtime, binary, cpus, cgroup_parent):
    directory = root / 'evidence/corrected' / str(workers); directory.mkdir(parents=True)
    call = qualification / 'products/call'
    products = directory / 'native/products'
    stages = graph('native', workers, inputs, products, binary, runtime, 'corrected', include_call=False)
    for stage in stages:
        stage['argv'] = [arg.replace(str(products / 'call'), str(call)) for arg in stage['argv']]
    trial(root, directory / 'native', 'native', workers, inputs, runtime, binary, cpus, cgroup_parent,
          profile='corrected', stages=stages)
    source = directory / 'source'; source.mkdir()
    original.require(source / 'annotate-command', ['Rscript', '--vanilla', ROOT / 'peak_real_performance.R',
        inputs / 'annotation.gtf', products / 'rows', source / 'annotation', inputs / 'chrominfo.tsv'])
    original.require(source / 'count-command', [runtime / 'bedtools-build/bin/bedtools', 'multicov', '-bams',
        *[inputs / (name + '.bam') for name in ('T1', 'T2', 'C')], '-bed', products / 'rows/workflow.bed'])
    annotation = consumer.compare_annotation(products / 'annotation.tsv', source / 'annotation/core.tsv',
        source / 'annotation/detail.tsv', products / 'rows/workflow-rows.tsv')
    counts = compare_counts(products / 'counts.tsv', source / 'count-command/stdout.txt', products / 'rows/workflow.bed')
    save(directory / 'qualification.json', dict(passed=True, annotation=annotation, count_rows=counts,
        caller_reused=True, source_reads_exact_native_rows=True, matched_end_to_end_speedup=None,
        sparse_speedup=None, not_equated_to_legacy=True))


def compare_downstream(source, native, rows, output):
    output.mkdir(parents=True)
    source, native = Path(source) / 'products', Path(native) / 'products'
    annotation = consumer.compare_annotation(native / 'annotation.tsv', source / 'annotation/core.tsv',
        source / 'annotation/detail.tsv', rows / 'workflow-rows.tsv')
    counts = compare_counts(native / 'counts.tsv', source / 'counts.tsv', rows / 'workflow.bed')
    save(output / 'qualification.json', dict(passed=True, annotation=annotation, count_rows=counts,
        ledger=tree_manifest(rows), matched_only='annotation/count on exactly the same corrected native rows'))


def pair_schedule():
    return [dict(workers=workers, pair=number, order=['source', 'native'] if number % 2 else ['native', 'source'])
            for workers in (1, 4) for number in range(1, 6)]


def strict_decision(rows):
    source, native = [[row for row in rows if row['tool'] == tool] for tool in ('source', 'native')]
    check(len(source) == len(native) == 5, 'five complete matched samples required')
    def describe(values):
        return dict(samples=values, minimum=min(values), median=statistics.median(values), maximum=max(values))
    a, b = [[row['wall_seconds'] for row in group] for group in (source, native)]
    exact = all(type(row.get('aggregate_tree_peak_bytes')) is int and row['aggregate_tree_peak_bytes'] > 0 and
                row.get('aggregate_tree_peak_scope') == 'dedicated cgroup-v2 hierarchical memory.peak' for row in rows)
    memory = max(row['aggregate_tree_peak_bytes'] for row in native) < min(row['aggregate_tree_peak_bytes'] for row in source) if exact else None
    wall = max(b) < min(a)
    no_wall_regression = statistics.median(b) <= statistics.median(a)
    return dict(source=describe(a), native=describe(b), strict_wall=wall, exact_tree_memory_advantage=memory,
        paired_wall_ratios_source_over_native=[left / right for left, right in zip(a, b)],
        memory_available=exact, memory_unavailable_reason=None if exact else 'no matched exact delegated cgroup counters',
        memory_source=describe([row['aggregate_tree_peak_bytes'] for row in source]) if exact else None,
        memory_native=describe([row['aggregate_tree_peak_bytes'] for row in native]) if exact else None,
        no_median_wall_regression=no_wall_regression, passed=wall or bool(memory and no_wall_regression))


def frozen_runtime(root):
    expected = load(root / 'evidence/runtime-before.json'); paths = load(root / 'runtime.json')
    check(original.files_identity(Path(paths['restored'])) == expected['restored'] and
          original.files_identity(OLD_PREFIX) == expected['exact_prefix'], 'restored source runtime bodies changed')
    check(all(_hash_file(path) == sha for name in ('system_bodies', 'mapped_python') for path, sha in expected[name].items()),
          'admitted loaded library/body changed')
    admission = load(root / 'evidence/native-admission.json')
    check(_hash_file(paths['binary']) == admission['receipt']['binary_sha256'], 'native binary changed')


def frozen_inputs(inputs):
    expected = load(inputs / 'admission.json')['derivative_files']
    current = tree_manifest(inputs); current.pop('admission.json')
    check(current == expected, 'frozen real slice/annotation changed')


def campaign(root, world, selector, cgroup_parent=None):
    root = hosted(root); paths = load(root / 'runtime.json')
    receipt, adoption = committed_receipt(world, selector)
    check(receipt == load(root / 'evidence/native-admission.json')['receipt'] and adoption == load(root / 'evidence/receipt-adoption.json'),
          'native receipt changed after admission')
    runtime, binary = Path(paths['runtime']), Path(paths['binary'])
    inputs = root / 'evidence/inputs'; frozen_inputs(inputs); frozen_runtime(root)
    lock = os.open(root / 'campaign.lock', os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600); os.close(lock)
    cpus = load(root / 'host-plan.json')['cpus']; results, stages, qualification = [], {}, {}
    corrected_results, corrected_stages = [], {}
    save(root / 'evidence/schedule.json', dict(pairs=pair_schedule(), qualification_pairs=2, warmup_pairs=2,
        whole_graph='independent call→seacr-1.3→own rows→annotation/count',
        corrected='separate native corrected graph and source-on-exact-native-ledger; not matched legacy timings',
        hot_path_memory='N/A; no exact stage cgroup counters', memory_alternative='exact tree advantage plus no median wall regression'))
    for workers in (1, 4):
        pair = {}
        for tool in ('source', 'native'):
            directory = root / 'evidence/qualification' / f'{workers}-{tool}'
            trial(root, directory, tool, workers, inputs, runtime, binary, cpus[:workers], cgroup_parent); pair[tool] = directory
        report = compare_pair(pair['source'], pair['native'], root / 'evidence/checks' / f'qualification-{workers}', runtime)
        histogram = load(inputs / 'admission.json')['duplicate_histogram']; retained = {}
        for population in ('treatment', 'control'):
            cap = report[population]['duplicate_cap']
            retained[population] = {chrom: sum(min(int(copies), cap) * sites for copies, sites in groups.items())
                                    for chrom, groups in histogram[population].items()}
            check(sum(retained[population].values()) == report[population]['retained'], 'native duplicate-cap admission disagrees with frozen real inputs')
        check(all(0 < retained['treatment'][chrom] + retained['control'][chrom] <= 262144 for chrom in WINDOWS),
              'fixed real slice exceeds four-worker event budget or lacks four useful chromosomes')
        qualification[workers] = report
        corrected_view(root, workers, pair['native'], inputs, runtime, binary, cpus[:workers], cgroup_parent)
    first, second = [root / 'evidence/qualification' / f'{workers}-native/products' for workers in (1, 4)]
    for name in [*['call/peaks' + suffix for suffix in CALL_SUFFIXES], 'sparse.stringent.bed',
                 'rows/workflow.bed', 'rows/workflow-rows.tsv', 'annotation.tsv', 'counts.tsv']:
        a, b = [(path / name).read_bytes() for path in (first, second)]
        if name.endswith('.xls'):
            a, b = [b'\n'.join(line for line in value.splitlines() if line and not line.startswith(b'#')) for value in (a, b)]
        check(a == b, 'real native1/4 scientific output differs: ' + name)
    base = tree_bytes(root)
    completed_bytes = tree_bytes(root / 'evidence/qualification') + tree_bytes(root / 'evidence/corrected')
    future = completed_bytes * 6
    check(base + future <= CAMPAIGN_LIMIT and future + HEADROOM <= shutil.disk_usage(root).free,
          'complete retained paired campaign cannot fit qualified storage; preserve and hold')
    save(root / 'evidence/resource-qualification.json', dict(retained_bytes=base, projected_additional_bytes=future,
        remaining_free_bytes=shutil.disk_usage(root).free, qualified_call_reports=qualification))
    for workers in (1, 4):
        pair = {}
        for tool in ('source', 'native'):
            directory = root / 'evidence/warmup' / f'{workers}-{tool}'
            trial(root, directory, tool, workers, inputs, runtime, binary, cpus[:workers], cgroup_parent); pair[tool] = directory
        compare_pair(pair['source'], pair['native'], root / 'evidence/checks' / f'warmup-{workers}', runtime)
        for row in [entry for entry in pair_schedule() if entry['workers'] == workers]:
            pair = {}; save(root / 'evidence' / f"pair-{workers}-{row['pair']}-host-before.json", host_state(cpus[:workers]))
            for tool in row['order']:
                directory = root / 'evidence/trials' / f"{workers}-{row['pair']}-{tool}"
                result = trial(root, directory, tool, workers, inputs, runtime, binary, cpus[:workers], cgroup_parent)
                results.append(result); pair[tool] = directory
                for stage in load(directory / 'stages.json'):
                    stages.setdefault((workers, stage['name']), []).append(dict(stage, tool=tool))
            compare_pair(pair['source'], pair['native'], root / 'evidence/checks' / f"trial-{workers}-{row['pair']}", runtime)
            save(root / 'evidence' / f"pair-{workers}-{row['pair']}-host-after.json", host_state(cpus[:workers]))
        rows = root / 'evidence/corrected' / str(workers) / 'native/products/rows'
        for phase, number, order in [('warmup', 0, ['source', 'native']), *[
                ('trial', row['pair'], row['order']) for row in pair_schedule() if row['workers'] == workers]]:
            pair = {}
            for tool in order:
                directory = root / 'evidence/corrected-timings' / f'{workers}-{phase}-{number}-{tool}'
                plan = downstream_graph(tool, workers, inputs, directory / 'products', rows, binary, runtime)
                result = trial(root, directory, tool, workers, inputs, runtime, binary, cpus[:workers], cgroup_parent,
                               profile='corrected-downstream', stages=plan)
                pair[tool] = directory
                if phase == 'trial':
                    corrected_results.append(result)
                    for stage in load(directory / 'stages.json'):
                        corrected_stages.setdefault((workers, stage['name']), []).append(dict(stage, tool=tool))
            compare_downstream(pair['source'], pair['native'], rows, root / 'evidence/checks' / f'C-{workers}-{phase}-{number}')
    frozen_inputs(inputs); frozen_runtime(root)
    decisions = {str(workers): strict_decision([row for row in results if row['workers'] == workers]) for workers in (1, 4)}
    hot_paths = {f'{workers}-{name}': strict_decision(rows) for (workers, name), rows in stages.items() if name != 'rows'}
    corrected_hot_paths = {f'{workers}-{name}': strict_decision(rows) for (workers, name), rows in corrected_stages.items()}
    all_decisions = [*decisions.values(), *hot_paths.values(), *corrected_hot_paths.values()]
    save(root / 'evidence/summary.json', dict(decisions=decisions, hot_paths=hot_paths, samples=results,
        corrected_downstream_samples=corrected_results, corrected_downstream_hot_paths=corrected_hot_paths,
        native_scaling=decisions['1']['native']['median'] / decisions['4']['native']['median'],
        source_scaling=decisions['1']['source']['median'] / decisions['4']['source']['median'],
        passed=all(row['passed'] for row in all_decisions),
        scope='fixed20Mb legacy36bp ENCODE CTCF slice; warm-cache process/startup-inclusive graph and individual CLI stages',
        cold_cache=False, biological_concordance=False, durability_claim=False, release=False))
    check(all(row['passed'] for row in all_decisions), 'strict same-work whole graph/hot-path performance gate remains held')


def finalize(root):
    root = Path(root); evidence = root / 'evidence'
    if not evidence.is_dir(): return
    failures = []
    try:
        if (evidence / 'runtime-before.json').is_file(): frozen_runtime(root)
        if (evidence / 'inputs/admission.json').is_file(): frozen_inputs(evidence / 'inputs')
    except (OSError, ValueError) as error: failures.append(str(error))
    acquisition = root / 'acquisition'
    for path in acquisition.rglob('*') if acquisition.exists() else []:
        if path.is_file() and path.suffix in ('.json', '.log') and 'restored' not in path.parts:
            target = evidence / 'acquisition' / path.relative_to(acquisition)
            target.parent.mkdir(parents=True, exist_ok=True); shutil.copy2(path, target)
    if (root / 'binary').is_file(): shutil.copy2(root / 'binary', evidence / 'native-binary')
    for name in ('host-plan.json', 'runtime.json'):
        if (root / name).is_file(): shutil.copy2(root / name, evidence / name)
    save(evidence / 'integrity-after.json', dict(failures=failures, source_runtime_rebuilt=False, release=False))
    save(evidence / 'artifact-manifest.json', dict(schema='rsomics.peak.real-performance.v1', files=tree_manifest(evidence),
        excluded_originals='verified downloaded old carriers/runtime/provider originals; identities retained, never deleted by this controller'))
    check(not failures, '; '.join(failures))


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['setup', 'acquire', 'materialize', 'campaign', 'execute-graph', 'rows', 'finalize'])
    parser.add_argument('--root', type=Path); parser.add_argument('--world', type=Path, default=ROOT.parent)
    parser.add_argument('--native-receipt'); parser.add_argument('--cpus'); parser.add_argument('--idle-host-confirmed', action='store_true')
    parser.add_argument('--cgroup-parent', type=Path); parser.add_argument('--plan', type=Path)
    parser.add_argument('--called', type=Path); parser.add_argument('--sparse', type=Path); parser.add_argument('--output', type=Path)
    args = parser.parse_args(argv)
    try:
        if args.action == 'setup': setup(args.root, args.cpus, args.idle_host_confirmed)
        elif args.action == 'rows':
            original.hosted(args.output)
            write_rows(args.called, args.sparse, args.output)
        elif args.action == 'execute-graph':
            original.hosted(args.plan)
            execute_graph(args.plan, [int(value) for value in args.cpus.split(',')])
        elif args.action == 'finalize':
            original.hosted(args.root)
            finalize(args.root)
        else:
            hosted(args.root)
            if args.action == 'acquire': acquire(args.root, args.world, args.native_receipt)
            elif args.action == 'materialize': materialize(args.root)
            elif args.action == 'campaign': campaign(args.root, args.world, args.native_receipt, args.cgroup_parent)
    except Exception as error:
        if args.root and args.root.is_dir() and (args.root / 'evidence').is_dir():
            save(args.root / 'evidence' / ('failure-' + args.action + '.json'), dict(type=type(error).__name__, message=str(error),
                status='HELD', performance_claim=False, release=False))
        print(type(error).__name__ + ': ' + str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
