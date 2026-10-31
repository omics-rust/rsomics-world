"""Private, adopted HGDP whole-workflow qualification; never uploads cohort artifacts."""

import argparse
import csv
import gzip
import hashlib
import itertools
import json
import os
from pathlib import Path
import platform
import re
import shutil
import signal
import stat
import statistics
import subprocess
import sys
import time
import urllib.error
import urllib.request
import zipfile

import plink_source_on_native as source
import plink_first_workflow_oracle as oracle
from plink_candidate import check, load, save, file_identity, tree_manifest, SNAPSHOTS, POLICY, EXPECTED
from signal_real_performance import _wait4, proc_group, tree_bytes, meminfo, cgroup_counters
from peak_real_performance import admit_elf, version_needs, check_version_closure


ROOT = Path(__file__).resolve().parents[1]
GIB = 2**30
LIMIT = 64 * GIB
HEADROOM = 8 * GIB
TARGETS = ['linux-x86_64', 'linux-aarch64', 'macos-x86_64', 'macos-aarch64']
REPORTS = ('acount', 'gcount', 'scount', 'smiss', 'vmiss', 'het', 'hardy', 'hwe-log.hardy')
CONTROLLERS = ('scripts/plink_real_performance.py', 'scripts/plink_observer_admission.py',
    'scripts/test_plink_real_performance.py', 'assets/plink-real-hgdp-20261101.json',
    '.github/workflows/plink-real-performance.yml', 'scripts/signal_real_performance.py',
    'scripts/peak_real_performance.py', *source.CONTROLLERS)


def config():
    row = load(ROOT / 'assets/plink-real-hgdp-20261101.json')
    check(row['schema'] == 'rsomics.plink.real-input.v1' and row['sample_count'] == 929 and
          row['chromosomes'] == ['19', '20', '21', '22'] and
          (row['start_inclusive'], row['end_inclusive']) == (20000001, 22000000) and
          row['application_memory_mib'] == 2048 and row['private_retention_required'] is True and
          row['public_redistribution_admitted'] is False, 'fixed real scientific/privacy profile differs')
    return row


def private_directory(path):
    path = Path(path).absolute()
    check(path.resolve(strict=True) == path and path.is_dir() and path != Path('/'), 'explicit existing private directory required')
    info = path.stat()
    check(info.st_uid == os.getuid() and stat.S_IMODE(info.st_mode) & 0o077 == 0,
          'private storage must be owner-only and owned by the actual runner UID')
    return path


def cpu_set(text):
    values = [int(value) for value in text.split(',')]
    check(len(values) == len(set(values)) == 4 and min(values) >= 0, 'four distinct allocated CPUs required')
    return values


def setup(storage, cpus, idle):
    check(os.environ.get('GITHUB_ACTIONS') == 'true' and os.environ.get('RUNNER_ENVIRONMENT') == 'self-hosted' and
          platform.system() == 'Linux' and platform.machine() == 'x86_64', 'explicit hosted self-hosted Linux route required')
    check(idle, 'fresh otherwise-idle host confirmation required')
    storage = private_directory(storage)
    check(storage.stat().st_dev != Path('/').stat().st_dev, 'private persistent storage cannot be the host boot filesystem')
    check(all(Path(os.environ[name]).resolve(strict=True).stat().st_dev == storage.stat().st_dev
              for name in ('GITHUB_WORKSPACE', 'RUNNER_TEMP')), 'runner project/cache routes are not on the admitted private device')
    allocated = cpu_set(cpus)
    check(set(allocated) <= os.sched_getaffinity(0), 'CPUs are not actually allocated')
    check(meminfo()['MemAvailable'] >= 8 * GIB and shutil.disk_usage(storage).free >= 72 * GIB,
          'private route requires72GiB free and8GiB currently available RAM')
    docker_root = subprocess.run(['docker', 'info', '--format', '{{.DockerRootDir}}'], check=True, capture_output=True, text=True).stdout.strip()
    check(Path(docker_root).resolve(strict=True).stat().st_dev == storage.stat().st_dev,
          'Docker state/cache is not on the admitted private storage device')
    image = subprocess.run(['docker', 'image', 'inspect', '--format', '{{json .RepoDigests}}',
        'gcc:14.3.0-bookworm@' + oracle.IMAGE], check=True, capture_output=True, text=True)
    check(any(value.endswith('@' + oracle.IMAGE) for value in json.loads(image.stdout)), 'exact image is not already present; no implicit pull allowed')
    root = storage / ('plink-' + os.environ['GITHUB_RUN_ID'] + '-' + os.environ['GITHUB_RUN_ATTEMPT'])
    root.mkdir(mode=0o700)
    save(root / 'private-host.json', dict(private_root=str(storage), device=storage.stat().st_dev,
        uid=os.getuid(), mode=oct(stat.S_IMODE(storage.stat().st_mode)), docker_root=docker_root,
        image=oracle.IMAGE, cpus=allocated, idle_operator_confirmed=True, available_memory=meminfo(),
        disk=shutil.disk_usage(storage)._asdict(), public_upload=False, standalone_mode=False))


def hosted(root):
    check(os.environ.get('GITHUB_ACTIONS') == 'true' and os.environ.get('GITHUB_REPOSITORY') == source.REPOSITORY and
          os.environ.get('RUNNER_ENVIRONMENT') == 'self-hosted', 'actual hosted self-hosted workflow required; standalone unsupported')
    root = private_directory(root)
    parent = private_directory(os.environ['PRIVATE_STORAGE_ROOT'])
    check(root.parent == parent and load(root / 'private-host.json')['image'] == oracle.IMAGE and
          os.environ.get('PRIVATE_RETENTION') == 'owner-only-persistent-no-upload', 'private persistent route has not been admitted')
    check(not any(os.environ.get(name) for name in ('LD_PRELOAD', 'LD_LIBRARY_PATH', 'CXXFLAGS', 'CFLAGS', 'LDFLAGS')),
          'unadmitted compiler/loader override')
    oracle.hosted(root)
    return root


def inner_environment(root):
    root = Path(root).resolve(strict=True)
    paths = {name: root / relative for name, relative in {
        'EVIDENCE': 'evidence', 'TMPDIR': 'scratch', 'XDG_CACHE_HOME': 'cache', 'XDG_CONFIG_HOME': 'config',
        'XDG_DATA_HOME': 'data', 'PIP_CACHE_DIR': 'cache/pip', 'PYTHONUSERBASE': 'python-user',
        'PYTHONPYCACHEPREFIX': 'cache/pycache', 'CCACHE_DIR': 'cache/ccache',
        'CARGO_HOME': 'unused-cargo', 'CARGO_TARGET_DIR': 'unused-target', 'RUSTUP_HOME': 'unused-rustup'}.items()}
    for name, path in paths.items():
        path.mkdir(parents=True, exist_ok=True, mode=0o700)
        os.environ[name] = str(path)
    hosted(root)


def committed_receipt(world, selector):
    world = Path(world).resolve(strict=True)
    check(isinstance(selector, str) and re.fullmatch(r'\.autopilot/oracles/[A-Za-z0-9_.-]+\.json', selector),
          'explicit committed native adoption required')
    result = subprocess.run(['git', '-c', 'safe.directory=' + str(world), 'show', 'HEAD:' + selector],
                            cwd=world, capture_output=True)
    check(result.returncode == 0, 'native adoption Git read failed: ' + result.stderr.decode(errors='replace'))
    check(result.stdout == (world / selector).read_bytes(), 'native adoption differs from its committed body')
    row = json.loads(result.stdout)
    check(row['schema'] == 'rsomics.plink.native-adoption.v1' and row['product'] == 'rsomics-plink' and
          row['repository'] == source.REPOSITORY and row['native_targets'] == TARGETS and row['workers'] == [1, 4] and
          row['workflow_path'] == '.github/workflows/plink-candidate.yml' and row['four_native_and_source_passed'] is True,
          'actual four-native/source adoption is absent')
    for field in ('artifact_sha256', 'binary_sha256', 'source_archive_sha256', 'source_manifest_sha256'):
        check(re.fullmatch('[0-9a-f]{64}', row[field]), 'invalid adoption digest: ' + field)
    check(re.fullmatch('[0-9a-f]{40}', row['head_sha']) and re.fullmatch(r'green-[1-9][0-9]*', row['snapshot']) and
          all(type(row[key]) is int and row[key] > 0 for key in ('run_id', 'run_attempt', 'artifact_id', 'size_in_bytes')),
          'invalid adopted run/snapshot identity')
    snapshot = world / SNAPSHOTS / row['snapshot']
    check(oracle.sha256(snapshot / 'source.tar.gz') == row['source_archive_sha256'] and
          oracle.sha256(snapshot / 'files.sha256') == row['source_manifest_sha256'], 'adopted source snapshot differs')
    return row, dict(selector=selector, sha256=hashlib.sha256(result.stdout).hexdigest())


def api_carrier(artifact, destination):
    request = urllib.request.Request(f'https://api.github.com/repos/{source.REPOSITORY}/actions/artifacts/{artifact}/zip',
        headers={'Authorization': 'Bearer ' + os.environ['GH_TOKEN'], 'Accept': 'application/vnd.github+json'})
    try:
        response = urllib.request.build_opener(source.NoRedirect).open(request, timeout=120)
    except urllib.error.HTTPError as error:
        check(error.code == 302, 'native carrier API failed: HTTP' + str(error.code))
        url = error.headers['Location']
        parsed = source.urllib.parse.urlsplit(url)
        check(parsed.scheme == 'https' and parsed.hostname and any(parsed.hostname.endswith(suffix) for suffix in
              ('.blob.core.windows.net', '.actions.githubusercontent.com', '.githubusercontent.com')), 'unadmitted carrier redirect')
        response = urllib.request.urlopen(url, timeout=120)
    total = 0
    with response, Path(destination).open('xb') as output:
        for block in iter(lambda: response.read(2**20), b''):
            total += len(block); check(total <= 512 * 2**20, 'compact native carrier exceeds512MiB')
            output.write(block)


def acquire_runtime(root, world, selector):
    root = hosted(root); receipt, adoption = committed_receipt(world, selector)
    save(root / 'evidence/native-adoption.json', dict(receipt=receipt, adoption=adoption))
    destination = root / 'acquisition/native'; destination.mkdir(parents=True)
    run = source.api(f"actions/runs/{receipt['run_id']}", destination / 'run.json')
    check((run['id'], run['run_attempt'], run['head_sha'], run['status'], run['conclusion'], run['path']) ==
          (receipt['run_id'], receipt['run_attempt'], receipt['head_sha'], 'completed', 'success', receipt['workflow_path']),
          'live native run differs from adoption')
    jobs = source.api(f"actions/runs/{receipt['run_id']}/attempts/{receipt['run_attempt']}/jobs?per_page=100", destination / 'jobs.json')
    check(jobs['total_count'] == len(jobs['jobs']) == 5 and {row['name'] for row in jobs['jobs']} ==
          set(TARGETS + ['Pinned source and official engines read same native generations']) and
          all(row['conclusion'] == 'success' for row in jobs['jobs']), 'all five native/source jobs must actually pass')
    item = source.api(f"actions/artifacts/{receipt['artifact_id']}", destination / 'artifact.json')
    check(not item['expired'] and item['name'] == receipt['artifact_name'] ==
          f"plink-native-{receipt['run_id']}-{receipt['run_attempt']}-linux-x86_64" and
          item['size_in_bytes'] == receipt['size_in_bytes'] and item['digest'] == 'sha256:' + receipt['artifact_sha256'] and
          item['workflow_run']['id'] == receipt['run_id'] and item['workflow_run']['head_sha'] == receipt['head_sha'],
          'live native artifact differs from adoption')
    archive = destination / 'original.zip'; api_carrier(receipt['artifact_id'], archive)
    check(file_identity(archive) == dict(bytes=receipt['size_in_bytes'], sha256=receipt['artifact_sha256']), 'native carrier identity differs')
    carrier = destination / 'plink-native-linux-x86_64.tar'
    with zipfile.ZipFile(archive) as zipped:
        check(zipped.namelist() == [carrier.name] and zipped.getinfo(carrier.name).file_size <= 512 * 2**20 and
              zipped.testzip() is None, 'native carrier layout/CRC differs')
        with zipped.open(carrier.name) as stream, carrier.open('xb') as output:
            shutil.copyfileobj(stream, output, 2**20)
    previous = os.environ['GITHUB_SHA']
    try:
        os.environ['GITHUB_SHA'] = receipt['head_sha']
        native, binding = source.extract_native(carrier, destination / 'restored', world)
    finally:
        os.environ['GITHUB_SHA'] = previous
    check(oracle.sha256(native / 'binary-release') == receipt['binary_sha256'] and
          oracle.sha256(native / 'source.tar.gz') == receipt['source_archive_sha256'] and
          oracle.sha256(native / 'files.sha256') == receipt['source_manifest_sha256'], 'restored adopted native bodies differ')
    save(root / 'evidence/native-artifact.json', binding)
    work = root / 'acquisition/source'; work.mkdir()
    restored = source.acquire(work)
    source.verify_runtime(restored, root / 'evidence/source-runtime')
    admit_elf(native / 'binary-release', root / 'evidence/native-elf')
    runtime = dict(native=str(native), restored=str(restored), binary=str(native / 'binary-release'))
    save(root / 'runtime.json', runtime)
    os.environ.pop('GH_TOKEN', None)
    return runtime


def graph(tool, workers, inputs, output, binary):
    check(tool in ('source', 'official', 'native') and workers in (1, 4), 'unadmitted engine/CPU profile')
    output = Path(output); jobs = []; native = tool == 'native'
    def add(name, args, prefix, outputs, group=None):
        argv = ([str(binary), '--json', '--threads', str(workers), '--memory-mib', '2048', *map(str, args), '--out', str(prefix)] if native else
                [str(binary), *map(str, args), '--threads', str(workers), '--memory', '2048', 'require', '--out', str(prefix)])
        jobs.append(dict(name=name, group=group or name, argv=argv, outputs=[str(path) for path in outputs]))
    def make(name, source_prefix, extra=()):
        prefix = output / name
        add(name, [*(['make'] if native else []), '--pfile', source_prefix, *extra, *([] if native else ['--make-pgen'])],
            prefix, [str(prefix) + '.' + ext for ext in ('pgen', 'pvar', 'psam')])
    make('I', inputs)
    make('F', output / 'I', ['--autosome', '--max-alleles', '2', '--mind', '0.125', '--geno', '0.125', '--maf', '0.05',
                           *([] if native else ['nonmajor', '--ac-founders'])])
    def qc(stage):
        prefix = output / (stage + '.qc')
        add(stage + '.qc', ['stats', '--pfile', output / stage] if native else ['--pfile', output / stage, *oracle.QC, *oracle.HWE],
            prefix, [str(prefix) + '.' + ext for ext in (REPORTS if native else REPORTS[:-1])])
        if not native:
            add(stage + '.hwe-log', ['--pfile', output / stage, *oracle.HWE, 'log10'], output / (stage + '.qc.hwe-log'),
                [str(prefix) + '.hwe-log.hardy'], stage + '.qc')
    qc('F')
    add('pairs', ['ld', 'pairs', '--pfile', output / 'F', '--window-variants', '1000000', '--window-bp', '500000', '--min-r2', '0.2'] if native else
        ['--pfile', output / 'F', '--r2-unphased', 'cols=chrom,pos,id,ref,alt,provref,maj,nonmaj,freq',
         '--ld-window', '1000000', '--ld-window-kb', '500', '--ld-window-r2', '0.2'], output / 'pairs', [output / 'pairs.vcor'])
    add('prune', ['ld', 'prune', '--pfile', output / 'F', '--window-bp', '500000', '--step', '1', '--r2', '0.2'] if native else
        ['--pfile', output / 'F', '--indep-pairwise', '500kb', '1', '0.2', '--indep-order', '2'], output / 'prune',
        [output / 'prune.prune.in', output / 'prune.prune.out'])
    make('P', output / 'F', ['--extract', output / 'prune.prune.in'])
    qc('P')
    return jobs


def preparation_plan(providers, prepared, binary):
    providers, prepared = Path(providers), Path(prepared)
    jobs = []
    for chrom in config()['chromosomes']:
        prefix = prepared / ('chr' + chrom)
        jobs.append(dict(name='slice-' + chrom, argv=list(map(str, [binary, '--pgen', providers / ('chr' + chrom + '.pgen'),
            '--pvar', providers / ('chr' + chrom + '.ids.pvar'), '--psam', providers / 'hgdp.psam', '--chr', chrom,
            '--from-bp', '20000001', '--to-bp', '22000000', '--make-pgen', '--threads', '1', '--memory', '2048', 'require', '--out', prefix]))))
    jobs.append(dict(name='merge', argv=list(map(str, [binary, '--pmerge-list', prepared / 'slices.txt', 'pfile',
        '--make-pgen', '--threads', '1', '--memory', '2048', 'require', '--out', prepared / 'INPUT']))))
    return jobs


def rewrite_ids(original, output, lineage, chrom, provider_sha):
    count = 0
    with Path(original).open(newline='') as source_file, Path(output).open('x', newline='') as target, gzip.open(lineage, 'xt') as sidecar:
        for line in source_file:
            if line.startswith('#'):
                target.write(line); continue
            fields = line.rstrip('\r\n').split('\t')
            check(len(fields) >= 5 and fields[0] in (chrom, 'chr' + chrom), 'provider PVAR chromosome/columns differ')
            count += 1; original_id = fields[2]
            fields[2] = f'HGDP20190516_C{chrom}_R{count:09d}'
            target.write('\t'.join(fields) + '\n')
            sidecar.write(json.dumps(dict(ordinal=count, id=fields[2], original_id=original_id, original_line=line,
                chromosome=fields[0], position=fields[1], ref=fields[3], alt=fields[4], provider_sha256=provider_sha)) + '\n')
    return dict(rows=count, provider_sha256=provider_sha, canonical=file_identity(output), lineage=file_identity(lineage))


def raw_stream(stream, output, shape, expected=None, gt_only=True, guard=None):
    expected_stream = gzip.open(expected, 'rb') if expected else None
    digest = hashlib.sha256(); size = cells = rows = 0; header = None
    try:
        with Path(output).open('xb') as target, gzip.GzipFile(fileobj=target, mode='wb', filename='', mtime=0) as compressed:
            for line in stream:
                check(len(line) <= 2**20, 'raw observer row exceeds admitted bound')
                compressed.write(line); digest.update(line); size += len(line)
                row = json.loads(line)
                if header is None:
                    header = row
                    check(row == dict(schema=1, samples=shape[0], variants=shape[1], source=oracle.COMMIT,
                        semantics='raw codec; chromosome correction belongs to caller'), 'raw observer header/shape differs')
                else:
                    check(row['variant'] == rows and len(row['calls']) == shape[0], 'raw row/cell cardinality differs')
                    check(all(len(call) == 6 for call in row['calls']), 'raw call width differs')
                    if gt_only:
                        check(all(call[4] is None and call[5] is None for call in row['calls']), 'unexpected real dosage/dphase track')
                    rows += 1; cells += len(row['calls'])
                if expected_stream is not None:
                    other = expected_stream.readline()
                    check(other and json.loads(other) == row, 'exact original raw phase/dosage/REF row differs')
                if guard is not None and rows % 1024 == 0:
                    guard()
            check(header is not None and rows == shape[1], 'raw observer row population differs')
            check(expected_stream is None or not expected_stream.read(1), 'raw comparison has trailing rows')
    finally:
        if expected_stream is not None: expected_stream.close()
    return dict(decompressed_sha256=digest.hexdigest(), decompressed_bytes=size, cells=cells,
                rows=rows, compressed=file_identity(output), original_raw_file_created=False)


def capture_raw(helper, prefix, output, expected=None, gt_only=True, root=None):
    samples, variants = oracle.metadata(prefix)
    check(0 < len(samples) <= 1000 and 0 < len(variants) <= 500000, 'observer admitted shape exceeded')
    output = Path(output); output.parent.mkdir(parents=True, exist_ok=True)
    argv = [str(helper), *[str(prefix) + '.' + ext for ext in ('pgen', 'pvar', 'psam')]]
    process = None
    with Path(str(output) + '.stderr').open('xb') as stderr:
        try:
            process = subprocess.Popen(argv, stdout=subprocess.PIPE, stderr=stderr, start_new_session=True)
            receipt = raw_stream(process.stdout, output, [len(samples), len(variants)], expected, gt_only,
                                 (lambda: budget_guard(root)) if root else None)
            check(process.wait(timeout=60) == 0, 'raw observer returned nonzero')
        finally:
            if process is not None:
                if process.poll() is None:
                    os.killpg(process.pid, signal.SIGKILL); process.wait()
                process.stdout.close()
    save(str(output) + '.json', dict(argv=argv, **receipt))
    return receipt


def compare_report(actual, expected, comparison):
    with Path(actual).open() as left, Path(expected).open() as right:
        a = csv.DictReader((line.lstrip('#') for line in left if not line.startswith('##')), delimiter='\t')
        b = csv.DictReader((line.lstrip('#') for line in right if not line.startswith('##')), delimiter='\t')
        check(a.fieldnames == b.fieldnames and bool(a.fieldnames), 'principal report header differs')
        count = 0
        for count, (av, bv) in enumerate(itertools.zip_longest(a, b), 1):
            check(av is not None and bv is not None and None not in av and None not in bv and
                  None not in av.values() and None not in bv.values(), 'principal report row shape differs')
            for key in a.fieldnames:
                if key in comparison.FLOAT_COLUMNS:
                    comparison.same_float(av[key], bv[key], f'principal row{count} {key}')
                elif key in ('REF_CT', 'ALT_CTS'):
                    check(list(map(comparison.dosage_units, av[key].split(','))) ==
                          list(map(comparison.dosage_units, bv[key].split(','))), 'exact dosage count differs')
                elif key in comparison.INTEGER_COLUMNS:
                    check(list(map(int, av[key].split(','))) == list(map(int, bv[key].split(','))), 'exact principal count differs')
                else:
                    check(key in comparison.CATEGORY_COLUMNS and av[key] == bv[key], 'principal categorical identity/schema differs')
    return count


def compare_projection(parent, child, parent_raw, child_raw):
    ps, pv = oracle.metadata(parent); cs, cv = oracle.metadata(child)
    sample_index = {tuple(row.get(key, '0') for key in ('FID', 'IID', 'SID')): i for i, row in enumerate(ps)}
    selected = [sample_index[tuple(row.get(key, '0') for key in ('FID', 'IID', 'SID'))] for row in cs]
    check(selected == sorted(set(selected)), 'child sample projection changed order/identity')
    variants = {row['ID']: i for i, row in enumerate(pv)}
    wanted = [variants[row['ID']] for row in cv]
    check(wanted == sorted(set(wanted)), 'child variant projection changed order/identity')
    for row, index in zip(cv, wanted):
        check(all(row[key] == pv[index][key] for key in ('CHROM', 'POS', 'ID', 'REF', 'ALT')), 'child allele/coordinate identity differs')
    with gzip.open(parent_raw, 'rt') as source_file, gzip.open(child_raw, 'rt') as target:
        next(source_file); next(target); index = 0
        for ordinal, line in enumerate(source_file):
            if index == len(wanted): break
            if ordinal != wanted[index]: continue
            parent_row = json.loads(line); child_row = json.loads(next(target))
            expected = dict(variant=index, provisional_ref=parent_row['provisional_ref'], calls=[parent_row['calls'][s] for s in selected])
            check(child_row == expected, 'exact own-parent raw projection differs')
            index += 1
        check(index == len(wanted) and not target.read(1), 'own-parent projection population differs')
    return len(cv) * len(cs)


def pair_schedule():
    return [dict(workers=workers, pair=index, order=(['source', 'native'] if (index + (workers == 4)) % 2 == 0 else ['native', 'source']))
            for workers in (1, 4) for index in range(5)]


def storage_bound(retained, qualification_sizes):
    check(len(qualification_sizes) == 6, 'six complete qualification sizes required')
    bound = retained + 20 * max(qualification_sizes) + 26 * 2**20
    check(bound <= LIMIT, 'whole retained campaign exceeds64GiB; fixed scientific profile must hold')
    return dict(retained_bound_bytes=bound, limit_bytes=LIMIT, headroom_bytes=HEADROOM,
                basis='retained originals + twenty maximum qualified complete graphs +1MiB per graph for variable logs')


def budget_guard(root):
    check(tree_bytes(root) <= LIMIT and shutil.disk_usage(root).free >= HEADROOM, 'private campaign64GiB/8GiB storage guard reached')
    check(meminfo()['MemAvailable'] >= 512 * 2**20, 'host available-memory abort guard reached')


def acquire_cohort(root):
    hosted(root); spec = config(); destination = root / 'providers'; destination.mkdir()
    pins = {}
    for name, row in [('hgdp.psam', spec['psam']), ('hgdp.metadata.txt', spec['metadata']), ('data-terms.txt', spec['terms']),
                      *[(f"chr{row['chromosome']}.{row['kind']}.zst", row) for row in spec['objects']]]:
        budget_guard(root)
        receipt = oracle.download(row['url'], destination / name, row['bytes'])
        check(receipt['bytes'] == row['bytes'] and (row['sha256'] is None or receipt['sha256'] == row['sha256']),
              'provider body size/pinned metadata hash changed; retain partials and hold')
        if row.get('etag'):
            headers = {key.lower(): value for key, value in receipt['headers'].items()}
            check(headers.get('etag', '').strip('"') == row['etag'], 'provider HEAD ETag changed before body adoption')
        pins[name] = receipt
    samples = oracle.table(destination / 'hgdp.psam'); metadata = {row['sample']: row for row in oracle.table(destination / 'hgdp.metadata.txt')}
    ids = [row['IID'] for row in samples]
    check(len(ids) == len(set(ids)) == len(metadata) == 929 and ids == sorted(ids) and
          hashlib.sha256(''.join(value + '\n' for value in ids).encode()).hexdigest() == spec['sample_ids_sha256'], 'full ordered cohort IDs changed')
    check(all(row['SEX'] == {'M': '1', 'F': '2'}[metadata[row['IID']]['sex']] and
          all(row[key] == metadata[row['IID']][key] for key in ('population', 'latitude', 'longitude', 'region')) for row in samples),
          'source PSAM/metadata sex or annotation changed')
    save(root / 'body-pins.json', dict(schema=1, files=pins, cohort='all929 HGDP', private_retention=True,
         founder_semantics='unknown parents; operational CLI founders only', hwe_filter=None, redistribution_admitted=False))
    return pins


def materialize(root, runtime, comparison):
    hosted(root); providers = root / 'providers'; prepared = root / 'prepared'; prepared.mkdir()
    pins = load(root / 'body-pins.json')['files']
    check(all(file_identity(providers / name) == dict(bytes=row['bytes'], sha256=row['sha256']) for name, row in pins.items()), 'frozen provider body changed')
    binary = Path(runtime['restored']) / source.ENGINES['source'][0]
    lineage = {}
    for chrom in config()['chromosomes']:
        for extension in ('pgen', 'pvar'):
            oracle.require(root / 'evidence/preparation' / ('decompress-' + chrom + '-' + extension),
                [binary, '--zst-decompress', providers / f'chr{chrom}.{extension}.zst', providers / f'chr{chrom}.{extension}'])
        lineage[chrom] = rewrite_ids(providers / f'chr{chrom}.pvar', providers / f'chr{chrom}.ids.pvar',
            providers / f'chr{chrom}.lineage.jsonl.gz', chrom, pins[f'chr{chrom}.pvar.zst']['sha256'])
    (prepared / 'slices.txt').write_text(''.join(str(prepared / ('chr' + chrom)) + '\n' for chrom in config()['chromosomes']))
    for job in preparation_plan(providers, prepared, binary):
        budget_guard(root); oracle.require(root / 'evidence/preparation' / job['name'], job['argv'])
    prefix = prepared / 'INPUT'; samples, variants = oracle.metadata(prefix)
    check(len(samples) == 929 and 0 < len(variants) <= 500000, 'prepared complete input sample/variant shape differs')
    check([row['IID'] for row in samples] == [row['IID'] for row in oracle.table(providers / 'hgdp.psam')], 'preparation changed ordered sample IDs')
    for sample, original_sample in zip(samples, oracle.table(providers / 'hgdp.psam')):
        check(all(sample[key] == value for key, value in original_sample.items()), 'preparation changed source sample metadata')
    expected = []
    for chrom in config()['chromosomes']:
        expected.extend(row for row in oracle.table(providers / f'chr{chrom}.ids.pvar') if 20000001 <= int(row['POS']) <= 22000000)
    check(len(expected) == len(variants) and all(all(a[key] == b[key] for key in ('CHROM', 'POS', 'ID', 'REF', 'ALT'))
          for a, b in zip(variants, expected)), 'fixed preparation changed selected variant lineage/order/alleles')
    check({row['CHROM'].removeprefix('chr') for row in variants} == set(config()['chromosomes']), 'a fixed chromosome region is empty')
    oracle.require(root / 'evidence/preparation/validate', [binary, '--pfile', prefix, '--validate', '--threads', '1', '--memory', '2048', 'require', '--out', prepared / 'validation'])
    capture_raw(runtime['observer'], prefix, root / 'evidence/input.raw.jsonl.gz', root=root)
    record = dict(prefix=str(prefix), shape=[len(samples), len(variants)], lineage=lineage,
                  files={ext: file_identity(str(prefix) + '.' + ext) for ext in ('pgen', 'pvar', 'psam')}, private_retention=True)
    save(root / 'prepared-input.json', record)
    return prefix


def execute(plan, cpus):
    rows = []; generations = {}; audit_ns = 0
    started_graph = time.monotonic_ns()
    for job in load(plan)['stages']:
        directory = Path(plan).parent / 'commands' / job['name']; directory.mkdir(parents=True)
        with (directory / 'stdout.txt').open('xb') as stdout, (directory / 'stderr.txt').open('xb') as stderr:
            result = _wait4(job['argv'], stdout, stderr, cpus)
        result.update(name=job['name'], group=job['group'])
        save(directory / 'measurement.json', result); rows.append(result)
        check(result['exit'] == 0 and all(Path(path).is_file() for path in job['outputs']), 'required whole-graph operation failed: ' + job['name'])
        if job['name'] in ('I', 'F', 'P'):
            started = time.monotonic_ns()
            generations[job['name']] = {name: file_identity(name) for name in job['outputs']}
            audit_ns += time.monotonic_ns() - started
    started = time.monotonic_ns()
    for files in generations.values():
        check(all(file_identity(name) == identity for name, identity in files.items()), 'creation-bound own generation was modified by descendants')
    audit_ns += time.monotonic_ns() - started
    save(Path(plan).parent / 'stages.json', rows)
    save(Path(plan).parent / 'generation-custody.json', dict(creation_files=generations,
        unchanged_after_descendants=True, audit_seconds=audit_ns / 1e9,
        audit_included_in_outer_wall=True, audit_excluded_from_scientific_cli_times=True))
    save(Path(plan).parent / 'graph-clock.json', dict(wall_seconds=(time.monotonic_ns()-started_graph)/1e9,
        scientific_cli_wall_seconds=sum(row['wall_seconds'] for row in rows),
        scope='child-monotonic whole CLI graph plus explicit custody audit; controller polling is not the timing clock'))


def trial(root, directory, tool, workers, inputs, binary, cpus, cgroup_parent=None):
    directory.mkdir(parents=True); products = directory / 'products'; products.mkdir()
    stages = graph(tool, workers, inputs, products, binary)
    save(directory / 'plan.json', dict(tool=tool, workers=workers, stages=stages))
    group = None
    if cgroup_parent:
        parent = Path(cgroup_parent).resolve(strict=True)
        check(parent.is_relative_to('/sys/fs/cgroup') and 'memory' in (parent / 'cgroup.controllers').read_text().split(), 'existing delegated memory cgroup required')
        group = parent / ('plink-' + hashlib.sha256(str(directory).encode()).hexdigest()[:20]); group.mkdir()
        (group / 'memory.max').write_text(str(8 * GIB)); (group / 'memory.oom.group').write_text('1')
        check((group / 'memory.peak').is_file(), 'exact delegated memory counter unavailable')
    command = [sys.executable, '-B', str(Path(__file__).resolve()), 'execute', '--root', str(root),
               '--plan', str(directory / 'plan.json'), '--cpus', ','.join(map(str, cpus[:workers]))]
    def enter():
        os.sched_setaffinity(0, cpus[:workers])
        if group is not None: (group / 'cgroup.procs').write_text(str(os.getpid()))
    child = None; failure = None; usage = None; peak = 0; started = time.monotonic_ns()
    with (directory / 'runner.stdout').open('xb') as stdout, (directory / 'runner.stderr').open('xb') as stderr:
        try:
            child = subprocess.Popen(command, stdout=stdout, stderr=stderr, start_new_session=True, preexec_fn=enter)
            while True:
                peak = max(peak, sum(row['rss_bytes'] for row in proc_group(child.pid)))
                check(peak <= 8 * GIB, 'sampled owned-process RSS guard exceeded8GiB')
                budget_guard(root)
                observed = os.wait4(child.pid, os.WNOHANG)
                if observed[0]:
                    _, status, usage = observed; child.returncode = os.waitstatus_to_exitcode(status); break
                time.sleep(0.5)
            check(child.returncode == 0 and not proc_group(child.pid), 'whole workflow failed or left live descendants')
        except BaseException as error:
            failure = str(error)
            if child is not None and child.returncode is None:
                try: os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError: pass
                _, status, usage = os.wait4(child.pid, 0); child.returncode = os.waitstatus_to_exitcode(status)
            raise
        finally:
            counters = cgroup_counters(group)
            record = dict(tool=tool, workers=workers, wall_seconds=(time.monotonic_ns()-started)/1e9,
                exit=child.returncode if child else None, user_seconds=usage.ru_utime if usage else None,
                system_seconds=usage.ru_stime if usage else None, kernel_max_rss_bytes=usage.ru_maxrss*1024 if usage else None,
                kernel_rss_scope='wait4 maximum process/waited-child RSS, NOT aggregate tree', sampled_sum_rss_peak_bytes=peak,
                sampled_rss_scope='live owned process-group sum; shared pages doublecounted; sampled abort guard only',
                aggregate_tree_peak_bytes=int(counters['memory.peak']) if counters else None,
                aggregate_tree_peak_scope='delegated cgroup v2 hierarchical memory.peak, NOT RSS' if counters else None,
                cgroup=counters, failure=failure, cache='warm-cache', fsync_durability_claim=False)
            save(directory / 'measurement.json', record)
    record['stages'] = load(directory / 'stages.json')
    record['controller_observed_wall_seconds'] = record['wall_seconds']
    record.update(load(directory / 'graph-clock.json'))
    record['generation_custody'] = load(directory / 'generation-custody.json')
    save(directory / 'complete-measurement.json', record)
    return record


def check_graph(root, directory, baseline, runtime, comparison, input_prefix):
    products = directory / 'products'; observed = {}; generations = {}
    comparison.compare_metadata(products / 'I', input_prefix)
    for stage in ('I', 'F', 'P'):
        prefix = products / stage
        if baseline:
            comparison.compare_metadata(prefix, baseline / 'products' / stage)
        expected_raw = (baseline / (stage + '.raw.jsonl.gz')) if baseline else (root / 'evidence/input.raw.jsonl.gz' if stage == 'I' else None)
        generations[stage] = capture_raw(runtime['observer'], prefix, directory / (stage + '.raw.jsonl.gz'), expected_raw, root=root)
        observed[stage] = {ext: file_identity(str(prefix) + '.' + ext) for ext in ('pgen', 'pvar', 'psam')}
    if baseline:
        for stage in ('F', 'P'):
            for extension in REPORTS:
                compare_report(products / (stage + '.qc.' + extension), baseline / 'products' / (stage + '.qc.' + extension), comparison)
        compare_report(products / 'pairs.vcor', baseline / 'products/pairs.vcor', comparison)
        for suffix in ('prune.in', 'prune.out'):
            check((products / ('prune.' + suffix)).read_bytes() == (baseline / 'products' / ('prune.' + suffix)).read_bytes(), 'ordered prune lists differ')
    fs, fv = oracle.metadata(products / 'F'); ps, pv = oracle.metadata(products / 'P')
    keep = (products / 'prune.prune.in').read_text().splitlines(); remove = (products / 'prune.prune.out').read_text().splitlines()
    ids = [row['ID'] for row in fv]
    kept, removed = set(keep), set(remove)
    check(len(kept | removed) == len(ids) == len(keep) + len(remove) and
          keep == [value for value in ids if value in kept] and remove == [value for value in ids if value in removed] and
          [row['ID'] for row in pv] == keep and ps == fs, 'ordered F/prune/P partition or sample lineage differs')
    compare_projection(products / 'I', products / 'F', directory / 'I.raw.jsonl.gz', directory / 'F.raw.jsonl.gz')
    compare_projection(products / 'F', products / 'P', directory / 'F.raw.jsonl.gz', directory / 'P.raw.jsonl.gz')
    for stage in ('I', 'F', 'P'):
        check(observed[stage] == {ext: file_identity(str(products / stage) + '.' + ext) for ext in ('pgen', 'pvar', 'psam')}, 'own generation changed during checks')
    for engine in ('source', 'official'):
        binary = Path(runtime['restored']) / source.ENGINES[engine][0]
        oracle.require(directory / ('fresh-P-' + engine), [binary, '--pfile', products / 'P', '--validate', '--threads', '1', '--memory', '2048', 'require', '--out', directory / ('P-check-' + engine)])
    worker_rows = {}
    plan = load(directory / 'plan.json')
    if plan['tool'] == 'native':
        for name in ('F.qc', 'pairs', 'prune', 'P.qc'):
            report = load(directory / 'commands' / name / 'stdout.txt')['result']
            key = 'effective_worker_limit' if name.endswith('.qc') else 'effective_workers'
            check(report['requested_workers'] == plan['workers'] and report['writer_workers'] == 1 and
                  0 <= report[key] <= plan['workers'], 'actual native worker report differs')
            worker_rows[name] = dict(requested=report['requested_workers'], compute=report[key], writer=1,
                measure='batch worker capacity' if key == 'effective_worker_limit' else 'actual distinct part-worker IDs')
    save(directory / 'whole-check.json', dict(exact_raw=generations, own_projection=True, fresh_P_validated=True,
        original_input=str(input_prefix), baseline=str(baseline) if baseline else None, generation_files=observed,
        native_worker_provenance=worker_rows))
    return generations


def coordinate_pairs(prefix):
    _, rows = oracle.metadata(prefix); by_chrom = {}
    for row in rows: by_chrom.setdefault(row['CHROM'], []).append(int(row['POS']))
    result = {}
    for chrom, positions in by_chrom.items():
        check(positions == sorted(positions), 'F positions are not source ordered')
        first = count = 0
        for end, position in enumerate(positions):
            while position - positions[first] > 500000 or end - first >= 1000000: first += 1
            count += end - first
        result[chrom] = dict(variants=len(positions), coordinate_candidate_pairs=count)
    return result


def decision(rows):
    source_rows = [row for row in rows if row['tool'] == 'source']; native_rows = [row for row in rows if row['tool'] == 'native']
    check(len(source_rows) == len(native_rows) == 5, 'five complete paired trials required')
    strict_wall = (max(row['wall_seconds'] for row in native_rows) < min(row['wall_seconds'] for row in source_rows) and
                   max(row['scientific_cli_wall_seconds'] for row in native_rows) < min(row['scientific_cli_wall_seconds'] for row in source_rows))
    exact = all(row['aggregate_tree_peak_bytes'] is not None for row in rows)
    memory = exact and max(row['aggregate_tree_peak_bytes'] for row in native_rows) < min(row['aggregate_tree_peak_bytes'] for row in source_rows)
    groups = {}; regressions = []
    for group in ('I', 'F', 'F.qc', 'pairs', 'prune', 'P', 'P.qc'):
        times = {tool: [sum(stage['wall_seconds'] for stage in row['stages'] if stage['group'] == group) for row in selected]
                 for tool, selected in (('source', source_rows), ('native', native_rows))}
        groups[group] = times
        if statistics.median(times['native']) > statistics.median(times['source']): regressions.append(group)
    wall_nonregression = statistics.median(row['wall_seconds'] for row in native_rows) <= statistics.median(row['wall_seconds'] for row in source_rows)
    return dict(strict_wall=strict_wall, exact_tree_memory_advantage=memory if exact else None,
        stage_times=groups, median_stage_regressions=regressions, whole_wall_nonregression=wall_nonregression,
        passed=(strict_wall or (memory and wall_nonregression)) and not regressions,
        criterion='strict outer AND scientific-CLI wall separation OR exact tree-memory separation with no median wall regression; no stage median regression',
        samples=rows, biological_claim=False, four_platform_performance=False)


def campaign(root, runtime, cpus, cgroup_parent=None):
    comparison = source.comparator(Path(runtime['native']) / 'source')
    inputs = Path(load(root / 'prepared-input.json')['prefix']); qualified = {}; sizes = []; measurements = []
    for workers in (1, 4):
        baseline = None
        for tool in ('source', 'native', 'official'):
            directory = root / 'evidence/qualification' / str(workers) / tool
            binary = runtime['binary'] if tool == 'native' else str(Path(runtime['restored']) / source.ENGINES[tool][0])
            measured = trial(root, directory, tool, workers, inputs, binary, cpus, cgroup_parent)
            check_graph(root, directory, baseline or (qualified.get((1, 'source')) if workers == 4 else None), runtime, comparison, inputs)
            if tool == 'source': baseline = directory
            qualified[workers, tool] = directory; sizes.append(tree_bytes(directory)); measurements.append(measured)
    geometry = {str(workers): coordinate_pairs(qualified[workers, 'source'] / 'products/F') for workers in (1, 4)}
    bound = storage_bound(tree_bytes(root), sizes)
    save(root / 'evidence/qualification.json', dict(status='six_whole_graphs_passed', storage=bound,
        chromosome_tasks=geometry, measurements=measurements, warmups=6, source_official_build_correspondence='unproven'))
    results = []
    for entry in pair_schedule():
        for tool in entry['order']:
            workers = entry['workers']; directory = root / 'evidence/trials' / str(workers) / str(entry['pair']) / tool
            binary = runtime['binary'] if tool == 'native' else str(Path(runtime['restored']) / source.ENGINES[tool][0])
            measured = trial(root, directory, tool, workers, inputs, binary, cpus, cgroup_parent)
            check_graph(root, directory, qualified[workers, 'source'], runtime, comparison, inputs)
            check(tree_bytes(directory) <= max(sizes) + 2**20, 'trial exceeded its unchanged-profile qualification storage bound')
            results.append(measured)
    outcome = {str(workers): decision([row for row in results if row['workers'] == workers]) for workers in (1, 4)}
    save(root / 'evidence/performance-decision.json', dict(profiles=outcome, passed=all(row['passed'] for row in outcome.values()),
        private_retention=True, release=False, real_scope='GT/phase PGEN; not DS/HDS/VCF-import or biological validation'))
    check(all(row['passed'] for row in outcome.values()), 'strict real whole-workflow performance gate did not pass; all originals retained')


def run(root, world, selector, cpus, cgroup_parent=None):
    check(os.environ.get('GITHUB_ACTIONS') == 'true', 'actual hosted execution required; no local/standalone mode')
    root = Path(root).resolve(strict=True); inner_environment(root)
    allocated = cpu_set(cpus)
    check(set(allocated) <= os.sched_getaffinity(0), 'allocated CPUs differ inside exact image')
    check(meminfo()['MemAvailable'] >= 8 * GIB and shutil.disk_usage(root).free >= 72 * GIB, 'initial72GiB/8GiB admission failed')
    before = {name: oracle.sha256(ROOT / name) for name in set(CONTROLLERS)}
    save(root / 'evidence/controller.json', dict(commit=os.environ['GITHUB_SHA'], files=before, image=oracle.IMAGE))
    failure = None
    try:
        runtime = acquire_runtime(root, world, selector)
        import plink_observer_admission
        runtime['observer'] = str(plink_observer_admission.admit(root, runtime))
        save(root / 'admitted-runtime.json', runtime)
        acquire_cohort(root)
        comparison = source.comparator(Path(runtime['native']) / 'source')
        materialize(root, runtime, comparison)
        campaign(root, runtime, allocated, cgroup_parent)
    except BaseException as error:
        failure = dict(type=type(error).__name__, message=str(error))
        raise
    finally:
        os.environ.pop('GH_TOKEN', None)
        check(before == {name: oracle.sha256(ROOT / name) for name in set(CONTROLLERS)}, 'protected controller changed')
        record = dict(status='failed' if failure else 'completed', failure=failure, files=tree_manifest(root),
                      performance_passed=load(root / 'evidence/performance-decision.json')['passed']
                        if (root / 'evidence/performance-decision.json').is_file() else None,
                      private_retention=True, public_upload=False, release=False)
        save(root / 'final-private-manifest.json', record)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['setup', 'run', 'execute'])
    parser.add_argument('--root', type=Path); parser.add_argument('--storage-root', type=Path)
    parser.add_argument('--world', type=Path, default=ROOT); parser.add_argument('--native-receipt')
    parser.add_argument('--cpus', required=True); parser.add_argument('--idle-host-confirmed', action='store_true')
    parser.add_argument('--cgroup-parent', type=Path); parser.add_argument('--plan', type=Path)
    args = parser.parse_args()
    if args.action == 'setup': setup(args.storage_root, args.cpus, args.idle_host_confirmed)
    elif args.action == 'run': run(args.root, args.world, args.native_receipt, args.cpus, args.cgroup_parent)
    else:
        root = hosted(args.root); plan = args.plan.resolve(strict=True)
        check(plan.is_relative_to(root / 'evidence'), 'graph plan escapes private campaign')
        execute(plan, [int(value) for value in args.cpus.split(',')])


if __name__ == '__main__':
    main()
