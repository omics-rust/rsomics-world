"""Pinned Linux PLINK engines read the same newly generated native files."""

import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import struct
import tarfile
import urllib.error
import urllib.parse
import urllib.request
import zipfile

import plink_first_workflow_oracle as original
from plink_candidate import (CONTROLLERS, EXPECTED, LANES, POLICY, REPOSITORY, SCHEMA,
    SNAPSHOTS, _hash_file, check, check_tree,
    file_identity, load, portable_names, read_snapshot_manifest, save, tree_manifest)


RUN = 41221033473
HEAD = 'ebec87abd1339f9706ce65cdc690d343f879cea3'
ARTIFACT = 13050495530
JOB = 124595443390
ZIP_SIZE = 45934499
ZIP_SHA = '0e3ab523c286d573010212841e7c26f57482731af4de957c1359d92fc83db729'
TAR_MEMBER = '__w/_temp/plink-first-workflow-oracle.tar'
TAR_SIZE = 133447680
TAR_SHA = '96ae9afd4631528378625ddc3d982a95d05f5b168dd8b0dd420812e38b784707'
CAPTURE_SHA = '0a2a2a70ef03d56dd37449611f321ebaab74b0c699c3672d269abb104122c82a'
PREFIX = 'plink-first-workflow-oracle/'
ENGINES = {
    'source': ('runtime/source/2.0/build_dynamic/plink2', 'ec93f5a85e2d788acc40e808566e054eca89daae8f83af561ffcf557db75edba'),
    'official': ('runtime/official/plink2', '50ff05c332e8a4ff75f9bdbb9f513b0eab639e604c986ecd8b84b7f664253db2'),
    'helper': ('runtime/plink_pgen_capture', 'd77966679a8f9f30b432afd45dd8bfa8baaddebd61af94260817bab425512eca')}


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, file, code, message, headers, newurl):
        return None


def api(path, target, binary=False):
    target = Path(target); target.parent.mkdir(parents=True, exist_ok=True)
    request = urllib.request.Request('https://api.github.com/repos/' + REPOSITORY + '/' + path,
        headers={'Authorization': 'Bearer ' + os.environ['GH_TOKEN'],
                 'Accept': 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28'})
    try:
        response = urllib.request.build_opener(NoRedirect).open(request, timeout=120)
    except urllib.error.HTTPError as error:
        if not binary or error.code != 302:
            save(str(target) + '.http-error.json', dict(status=error.code, api_path=path,
                body=error.read(65536).decode('utf-8', errors='replace')))
            raise RuntimeError('original-evidence API failed: HTTP ' + str(error.code)) from None
        location = urllib.parse.urlsplit(error.headers['Location'])
        check(location.scheme == 'https' and location.hostname and any(location.hostname.endswith(suffix)
              for suffix in ('.blob.core.windows.net', '.actions.githubusercontent.com', '.githubusercontent.com')),
              'unexpected original-evidence redirect')
        response = urllib.request.urlopen(urllib.request.Request(error.headers['Location']), timeout=120)
    maximum = ZIP_SIZE if binary else 16 * 2**20
    size = 0
    with response, target.open('xb') as output:
        for block in iter(lambda: response.read(2**20), b''):
            size += len(block)
            check(size <= maximum, 'original acquisition exceeds its bound')
            output.write(block)
    return None if binary else load(target)


def relative_original(name):
    check(name.startswith(PREFIX), 'unexpected original archive root')
    relative = name[len(PREFIX):]
    check(relative and not relative.startswith('/') and '\\' not in relative and
          all(part not in ('', '.', '..') for part in relative.split('/')), 'unsafe original member')
    return relative


def verify_restore(archive_path, destination, capture_sha, expected_entries, choose):
    destination = Path(destination)
    check(not destination.exists() and not destination.is_symlink(), 'new original restoration path required')
    destination.mkdir(parents=True)
    with tarfile.open(archive_path, 'r:') as archive:
        members = archive.getmembers()
        check(len({item.name for item in members}) == len(members), 'duplicate original TAR name')
        cap = archive.getmember(PREFIX + 'CAPTURE.json')
        check(cap.isfile() and cap.size <= 16 * 2**20, 'invalid original CAPTURE member')
        body = archive.extractfile(cap).read()
        check(hashlib.sha256(body).hexdigest() == capture_sha, 'original CAPTURE identity differs')
        inventory = json.loads(body)
        check(inventory['schema'] == 1 and len(inventory['files']) == expected_entries,
              'original CAPTURE schema/population differs')
        observed, restored = {}, {}
        for member in members:
            if member.name == PREFIX[:-1] and member.isdir():
                continue
            name = relative_original(member.name)
            if member.isdir():
                continue
            if member.issym():
                observed[name] = dict(kind='symlink', target=member.linkname)
                check(not choose(name), 'selected runtime symlink requires a separate exact binding')
                continue
            check(member.isfile() and member.type == tarfile.REGTYPE, 'unexpected original member type')
            digest = hashlib.sha256(); target = None
            if choose(name):
                target = destination / name
                target.parent.mkdir(parents=True, exist_ok=True)
                output = target.open('xb')
            try:
                with archive.extractfile(member) as source:
                    for block in iter(lambda: source.read(2**20), b''):
                        digest.update(block)
                        if target is not None:
                            output.write(block)
            finally:
                if target is not None:
                    output.close()
            observed[name] = dict(kind='file', bytes=member.size, sha256=digest.hexdigest())
            if target is not None:
                target.chmod(member.mode & 0o777)
                restored[name] = observed[name]
        observed.pop('CAPTURE.json', None)
        check(observed == inventory['files'], 'original TAR/CAPTURE body or link membership differs')
        portable_names(restored)
    return dict(verified_entries=len(observed), restored=restored, capture_sha256=capture_sha)


def selected_original(name):
    return (name in {path for path, _ in ENGINES.values()} or
        name in {'runtime/runtime.json', 'runtime/loaded-libraries.json', 'runtime/compiler-closure.json',
                 'runtime/source/2.0/COPYING', 'runtime/source/2.0/COPYING.LESSER', 'inputs/founders64.id'} or
        name.startswith(('runtime/loaded-libraries/', 'controller/')))


def acquire(work):
    directory = Path(work) / 'original'; directory.mkdir()
    run = api(f'actions/runs/{RUN}', directory / 'run.json')
    check((run['id'], run['head_sha'], run['run_attempt'], run['status'], run['conclusion']) ==
          (RUN, HEAD, 1, 'completed', 'success') and run['repository']['full_name'] == REPOSITORY,
          'accepted source run identity/status differs')
    jobs = api(f'actions/runs/{RUN}/attempts/1/jobs?per_page=100', directory / 'jobs.json')
    check(jobs['total_count'] == 1 and jobs['jobs'][0]['id'] == JOB and
          jobs['jobs'][0]['conclusion'] == 'success', 'accepted source job differs')
    listing = api(f'actions/runs/{RUN}/artifacts?per_page=100', directory / 'artifacts.json')
    matches = [row for row in listing['artifacts'] if row['id'] == ARTIFACT]
    check(len(matches) == 1, 'original artifact is absent/ambiguous')
    row = matches[0]
    check(not row['expired'] and row['digest'] == 'sha256:' + ZIP_SHA and row['size_in_bytes'] == ZIP_SIZE and
          row['workflow_run']['id'] == RUN and row['workflow_run']['head_sha'] == HEAD and
          row['name'] == f'plink-first-workflow-oracle-{RUN}-1', 'original artifact API binding differs')
    carrier = directory / 'original.zip'; api(f'actions/artifacts/{ARTIFACT}/zip', carrier, binary=True)
    check(carrier.stat().st_size == ZIP_SIZE and _hash_file(carrier) == ZIP_SHA, 'original ZIP differs')
    tar_path = directory / 'original.tar'
    with zipfile.ZipFile(carrier) as archive:
        check(archive.namelist() == [TAR_MEMBER] and archive.getinfo(TAR_MEMBER).file_size == TAR_SIZE,
              'original ZIP carrier membership differs')
        with archive.open(TAR_MEMBER) as source, tar_path.open('xb') as target:
            shutil.copyfileobj(source, target, 2**20)
    check(tar_path.stat().st_size == TAR_SIZE and _hash_file(tar_path) == TAR_SHA, 'original TAR differs')
    proof = verify_restore(tar_path, Path(work) / 'restored', CAPTURE_SHA, 6360, selected_original)
    save(Path(work) / 'restoration.json', dict(**proof, zip_sha256=ZIP_SHA, tar_sha256=TAR_SHA,
        source_run=RUN, source_head=HEAD, original_body_payload_reuploaded=False))
    return Path(work) / 'restored'


def elf(path):
    with Path(path).open('rb') as stream:
        header = stream.read(64)
    check(len(header) == 64 and header[:6] == b'\x7fELF\x02\x01' and
          struct.unpack_from('<H', header, 18)[0] == 62, 'runtime/library is not x86_64 little-endian ELF')


def verify_runtime(restored, output):
    restored = Path(restored)
    identities = load(restored / 'runtime/runtime.json')
    libraries = load(restored / 'runtime/loaded-libraries.json')
    check(len(libraries) == 6, 'accepted image library population differs')
    for path, row in libraries.items():
        current = Path(path)
        check(current.resolve(strict=True) == Path(row['realpath']) and _hash_file(current) == row['sha256'] and
              _hash_file(restored / 'runtime' / row['retained']) == row['sha256'], 'image library identity differs: ' + path)
        elf(current)
    for path, digest in load(restored / 'runtime/compiler-closure.json').items():
        check(_hash_file(path) == digest, 'original image compiler/tool identity differs: ' + path)
    original.require(Path(output) / 'runtime-calls/compiler', ['/usr/local/bin/g++', '--version'])
    observed = {}; used = set()
    for name, (relative, digest) in ENGINES.items():
        executable = restored / relative
        check(_hash_file(executable) == identities[name]['sha256'] == digest, 'original engine identity differs: ' + name)
        elf(executable)
        call = Path(output) / 'runtime-calls' / (name + '-dependencies')
        status = original.capture(call, ['ldd', executable])
        text = (call / 'stdout.txt').read_text() + (call / 'stderr.txt').read_text()
        if name == 'official':
            check(status['returncode'] == 1 and re.search(r'not a dynamic executable|statically linked', text),
                  'official static dependency observation differs')
        else:
            check(status['returncode'] == 0 and 'not found' not in text, 'source runtime dependency load failed')
            paths = set(re.findall(r'(?:=>\s*)?(/[^\s]+)', text))
            check(paths <= libraries.keys(), 'source runtime uses an unbound image library')
            used.update(paths)
        if name != 'helper':
            call = Path(output) / 'runtime-calls' / (name + '-version')
            original.require(call, [executable, '--version'])
            observed[name] = (call / 'stdout.txt').read_text() + (call / 'stderr.txt').read_text()
            check(observed[name] == identities[name]['observed_version'], 'observed source version differs')
    check(used == libraries.keys(), 'actual source/helper loader closure differs from the original')
    return dict(engines=ENGINES, versions=observed, libraries=libraries,
                image=original.IMAGE, official_source_commit_correspondence='unproven')


def extract_artifact(archive_path, destination):
    destination = Path(destination)
    check(not destination.exists() and not destination.is_symlink(), 'new native restoration path required')
    with tarfile.open(archive_path, 'r:') as archive:
        members = archive.getmembers(); names = [item.name for item in members]
        portable_names(names)
        check(all(item.isfile() and item.type == tarfile.REGTYPE for item in members), 'native artifact has nonregular bodies')
        roots = {PurePosixPath(name).parts[0] for name in names}
        check(len(roots) == 1, 'native artifact root differs')
        prefix = roots.pop() + '/'
        member = archive.getmember(prefix + 'artifact-manifest.json')
        check(member.size <= 32 * 2**20, 'native artifact manifest is unexpectedly large')
        manifest = json.load(archive.extractfile(member))
        check(manifest['schema'] == SCHEMA, 'native artifact schema differs')
        files = manifest['files']; portable_names(files)
        check(set(names) == {prefix + name for name in files} | {member.name}, 'native artifact membership differs')
        destination.mkdir()
        for item in members:
            name = item.name.removeprefix(prefix)
            target = destination / name; target.parent.mkdir(parents=True, exist_ok=True)
            digest = hashlib.sha256()
            with archive.extractfile(item) as source, target.open('xb') as output:
                for block in iter(lambda: source.read(2**20), b''):
                    output.write(block); digest.update(block)
            if name in files:
                check(item.size == files[name]['bytes'] and digest.hexdigest() == files[name]['sha256'],
                      'native artifact body differs: ' + name)
    return manifest


def extract_native(archive_path, destination, world, *, expected_head=None, expected_controller_hashes=None):
    historical = expected_head is not None or expected_controller_hashes is not None
    if historical:
        check(isinstance(expected_head, str) and re.fullmatch('[0-9a-f]{40}', expected_head) and
              isinstance(expected_controller_hashes, dict) and set(expected_controller_hashes) == set(CONTROLLERS) and
              all(isinstance(value, str) and re.fullmatch('[0-9a-f]{64}', value)
                  for value in expected_controller_hashes.values()), 'complete paired historical native binding required')
    extract_artifact(archive_path, destination)
    root = Path(destination) / 'candidate'; world = Path(world)
    selection = load(root / 'selection.json')
    check(selection['controller_commit'] == (expected_head if historical else os.environ['GITHUB_SHA']) and
          selection['target'] == 'linux-x86_64' and
          selection['comparison_sha256'] == POLICY and selection['expected_manifest_sha256'] == EXPECTED,
          'native controller/target/comparison selection differs')
    check(re.fullmatch(r'green-[1-9][0-9]*', selection['snapshot']), 'native snapshot selector differs')
    snapshot = world / SNAPSHOTS / selection['snapshot']
    for name in ('README.md', 'files.sha256', 'source.tar.gz', 'checksums.sha256'):
        check(_hash_file(root / name) == _hash_file(snapshot / name), 'native source snapshot differs: ' + name)
    manifest = read_snapshot_manifest(root); check_tree(root / 'source', manifest)
    check(_hash_file(root / 'files.sha256') == selection['source_manifest_sha256'] and
          _hash_file(root / 'source.tar.gz') == selection['source_archive_sha256'] and
          _hash_file(root / 'source/Cargo.lock') == selection['lock_sha256'] and
          _hash_file(root / 'source/vendor/SOURCE.json') == selection['vendor_manifest_sha256'] and
          load(root / 'integrity-after.json')['failures'] == [], 'native source/lock/integrity binding differs')
    for name in CONTROLLERS:
        check(_hash_file(Path(destination) / 'controller' / name) ==
              (expected_controller_hashes[name] if historical else _hash_file(world / name)),
              'native controller byte binding differs: ' + name)
    for mode in ('debug', 'release'):
        check(_hash_file(root / ('binary-' + mode)) == load(root / ('binary-' + mode + '.json'))['sha256'],
              'native binary identity differs: ' + mode)
        binding = load(root / ('generations-' + mode + '.json'))
        graph = root / binding['graph']
        check(graph.resolve(strict=True).is_relative_to(root.resolve(strict=True)), 'restored graph route escapes native artifact')
        original_graph = load(graph / 'native-graph.json')
        original_prefixes = {(row['workers'], row['lane'], row['stage']): row['prefix'] for row in original_graph['generations']}
        for row in binding['generations']:
            expected_prefix = '/'.join(original_prefixes[(row['workers'], row['lane'], row['stage'])].split('/')[-2:])
            check(row['prefix'] == expected_prefix, 'portable/native original generation identity differs')
            for name, identity in row['files'].items():
                check(file_identity(graph / name) == identity, 'portable native generation body differs')
        check(binding['graph_sha256'] == _hash_file(graph / 'native-graph.json') and
              original_graph['status'] == 'complete_compact_native_graph_passed' and
              original_graph['raw_cells_checked'] == binding['raw_cells_checked'] == 74878 and
              binding['principal_report_files_per_worker'] == 114 and len(binding['generations']) == 36,
              'complete restored native graph receipt differs')
        for name, identity in binding['reports'].items():
            check(file_identity(graph / name) == identity, 'portable native principal report differs')
    receipt = dict(archive_sha256=_hash_file(archive_path), selection=selection,
                   manifest_sha256=_hash_file(Path(destination) / 'artifact-manifest.json'))
    if historical:
        receipt['historical_binding'] = dict(head_sha=expected_head, controller_sha256=expected_controller_hashes)
    return root, receipt


def compare_raw(native, bridge, shape):
    actual = load(native)
    with Path(bridge).open() as stream:
        expected = [json.loads(line) for line in stream if line.strip()]
    header, expected = expected[0], expected[1:]
    check(header['schema'] == 1 and [header['samples'], header['variants']] == shape and
          header['source'] == original.COMMIT and header['semantics'] == 'raw codec; chromosome correction belongs to caller',
          'raw bridge schema/source/dimensions differ')
    check(len(actual) == len(expected) == shape[1], 'raw variant population differs')
    cells = 0
    for index, (left, right) in enumerate(zip(actual, expected)):
        check(left['variant'] == right['variant'] == index and left == right and
              len(left['calls']) == len(right['calls']) == shape[0], 'exact raw phase/dosage/REF state differs at variant' + str(index))
        cells += len(left['calls'])
    return cells


def report_plan(graph, owner, output, workers):
    graph, owner, output = map(Path, (graph, owner, output))
    rows = []
    def add(name, args, kind, native, extensions=()):
        rows.append(dict(name=f'{workers}/' + name, args=list(map(str, args)), kind=kind,
            output=str(output / str(workers) / name), native=str(native), extensions=list(extensions)))
    for lane in LANES:
        for stage in ('I', 'F', 'P'):
            prefix = graph / str(workers) / f'{lane}.{stage}'
            add(f'{lane}/{stage}/validate', ['--pfile', prefix, '--validate'], 'validate', prefix)
            if stage == 'I':
                continue
            extensions = ('acount', 'gcount', 'scount', 'smiss', 'vmiss', 'het') + (() if lane == 'rich' else ('hardy',))
            add(f'{lane}/{stage}/qc', ['--pfile', prefix, *original.QC, *([] if lane == 'rich' else original.HWE)],
                'qc', graph / str(workers) / f'{lane}.{stage}.qc', extensions)
            eligible = ['--autosome', '--max-alleles', '2'] if lane == 'rich' else []
            add(f'{lane}/{stage}/hwe-log', ['--pfile', prefix, *eligible, *original.HWE, 'log10'],
                'hwe-log', graph / str(workers) / f'{lane}.{stage}.qc.hwe-log.hardy')
        prefix = graph / str(workers) / f'{lane}.F'
        add(f'{lane}/F/pairs', ['--pfile', prefix, *eligible, '--r2-unphased',
            'cols=chrom,pos,id,ref,alt,provref,maj,nonmaj,freq', '--ld-window', '1000000',
            '--ld-window-kb', '500', '--ld-window-r2', '-1'], 'pairs', graph / str(workers) / f'{lane}.F.ld.vcor')
        add(f'{lane}/F/prune', ['--pfile', prefix, *eligible, '--indep-pairwise', '500kb', '1', '0.2',
            '--indep-order', '2'], 'prune', graph / str(workers) / f'{lane}.F.prune')
    for mode in ('raw', 'log'):
        add('tail/' + mode, ['--pfile', graph / str(workers) / 'gt.I', '--keep',
            owner / 'tests/fixtures/inputs/founders64.id', '--hardy', 'midp', *(['log10'] if mode == 'log' else [])],
            'tail-' + mode, graph / str(workers) / ('tail.hardy' if mode == 'raw' else 'tail.hwe-log.hardy'))
    return rows


def comparator(owner):
    check(_hash_file(Path(owner) / 'tests/COMPARISON.md') == POLICY and
          _hash_file(Path(owner) / 'tests/fixtures/expected/SOURCE.json') == EXPECTED, 'source comparator authority differs')
    spec = importlib.util.spec_from_file_location('qualified_plink_comparison', Path(owner) / 'tests/compare_workflow.py')
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    return module


def compare_observation(job, comparison):
    prefix, native = Path(job['output']), Path(job['native'])
    if job['kind'] == 'validate':
        return dict(validated=True)
    if job['kind'] == 'qc':
        return {extension: comparison.compare_report(str(native) + '.' + extension,
                    str(prefix) + '.' + extension) for extension in job['extensions']}
    if job['kind'] == 'prune':
        for extension in ('prune.in', 'prune.out'):
            check(Path(str(native) + '.' + extension).read_text().splitlines() ==
                  Path(str(prefix) + '.' + extension).read_text().splitlines(), 'ordered native/source prune list differs')
        return dict(ordered_lists_equal=True)
    suffix = '.vcor' if job['kind'] == 'pairs' else '.hardy'
    return dict(rows=comparison.compare_report(native, str(prefix) + suffix,
                projection=job['kind'].startswith('tail-')))


def run(native_archive, native_output, work, output, world):
    native_archive, native_output, work, output, world = map(Path, (native_archive, native_output, work, output, world))
    original.hosted(output)
    check(os.environ.get('GITHUB_REPOSITORY') == REPOSITORY, 'source repository differs')
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    check(shutil.disk_usage(runner).free >= 2 * 2**30, 'insufficient compact source restoration space')
    for path in (native_archive, native_output, work, output):
        check(path.resolve().is_relative_to(runner) and path.parent.resolve(strict=True).stat().st_dev == runner.stat().st_dev,
              'source input/output escapes admitted runner device')
    check(not any(path.exists() or path.is_symlink() for path in (native_output, work, output)), 'fresh source routes required')
    output.mkdir(); work.mkdir()
    save(output / 'host.json', dict(run=os.environ['GITHUB_RUN_ID'], attempt=os.environ['GITHUB_RUN_ATTEMPT'],
        controller=os.environ['GITHUB_SHA'], image=original.IMAGE, source_execution='hosted Linux x86_64 only',
        native_elf_executed=False, disk=shutil.disk_usage(runner)._asdict()))
    protected = {}; observations = {}; failure = None
    image_before = {}; controller_before = {name: _hash_file(world / name) for name in CONTROLLERS}
    try:
        root, receipt = extract_native(native_archive, native_output, world)
        save(output / 'native-input-binding.json', receipt)
        protected[native_output] = tree_manifest(native_output)
        restored = acquire(work); os.environ.pop('GH_TOKEN', None)
        shutil.copy2(work / 'restoration.json', output / 'original-restoration.json')
        protected[restored] = tree_manifest(restored)
        libraries = load(restored / 'runtime/loaded-libraries.json')
        image_before = {path: _hash_file(path) for path in libraries}
        save(output / 'image-libraries-before.json', image_before)
        runtime = verify_runtime(restored, output); save(output / 'runtime-admission.json', runtime)
        check(_hash_file(world / 'scripts/plink_first_workflow_oracle.py') ==
              _hash_file(restored / 'controller/plink_first_workflow_oracle.py'), 'original source command controller differs')
        owner = root / 'source'; comparison = comparator(owner)
        check(_hash_file(owner / 'tests/fixtures/inputs/founders64.id') == _hash_file(restored / 'inputs/founders64.id'),
              'founder64 population selector differs')
        binding = load(root / 'generations-release.json'); graph = root / binding['graph']
        cells = 0
        for entry in binding['generations']:
            prefix = graph / entry['prefix']
            call = output / 'raw-calls' / str(entry['workers']) / (entry['lane'] + '.' + entry['stage'])
            original.require(call, [restored / ENGINES['helper'][0],
                *[str(prefix) + '.' + extension for extension in ('pgen', 'pvar', 'psam')]])
            cells += compare_raw(graph / entry['raw'], call / 'stdout.txt', entry['shape'])
        check(cells == 74878, 'whole source raw-cell population differs')
        observations['raw_cells'] = cells
        for profile in ('source', 'official'):
            for workers in (1, 4):
                jobs = report_plan(graph, owner, output / 'observed' / profile, workers)
                save(output / 'plans' / (profile + '-' + str(workers) + '.json'), jobs)
                results = {}
                for job in jobs:
                    prefix = Path(job['output']); prefix.parent.mkdir(parents=True, exist_ok=True)
                    argv = [restored / ENGINES[profile][0], *job['args'], '--threads', str(workers), '--out', prefix]
                    original.require(output / 'source-calls' / profile / job['name'], argv)
                    results[job['name']] = compare_observation(job, comparison)
                observations[f'{profile}/{workers}'] = results
        save(output / 'comparison.json', dict(status='same_native_generations_qualified', observations=observations,
            native_worker_profiles=[1, 4], source_profiles=['source', 'official'], performance='not_run', release=False,
            official_source_commit_correspondence='unproven'))
    except BaseException as error:
        failure = error
    finally:
        os.environ.pop('GH_TOKEN', None)
        original_directory = work / 'original'
        for path in sorted(original_directory.glob('*.json')):
            if path.is_file():
                target = output / 'original-api' / path.name; target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(path, target)
        carrier = original_directory / 'original.zip'
        if carrier.is_file() and (carrier.stat().st_size != ZIP_SIZE or _hash_file(carrier) != ZIP_SHA):
            target = output / 'failed-acquisition/original.zip'; target.parent.mkdir(parents=True)
            shutil.copy2(carrier, target)
            save(output / 'failed-acquisition/identity.json', file_identity(target))
        intact = {}
        for path, before in protected.items():
            after = tree_manifest(path)
            intact[str(path)] = dict(before=before, after=after, unchanged=before == after)
            if before != after and failure is None:
                failure = ValueError('protected source/native input changed')
        save(output / 'protected-identities.json', intact)
        image_after = {path: _hash_file(path) for path in image_before}
        save(output / 'image-libraries-after.json', dict(before=image_before, after=image_after,
            unchanged=image_before == image_after))
        controller_after = {name: _hash_file(world / name) for name in CONTROLLERS}
        save(output / 'controller-identities.json', dict(before=controller_before, after=controller_after,
            unchanged=controller_before == controller_after))
        if (image_before != image_after or controller_before != controller_after) and failure is None:
            failure = ValueError('protected image library/controller changed')
        save(output / 'result.json', dict(status='passed' if failure is None else 'failed',
            error=None if failure is None else str(failure), performance_claim=False, release=False))
    if failure is not None:
        raise failure


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--native-archive', type=Path, required=True)
    parser.add_argument('--native-output', type=Path, required=True)
    parser.add_argument('--work', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--world', type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    run(args.native_archive, args.native_output, args.work, args.output, args.world)


if __name__ == '__main__':
    main()
