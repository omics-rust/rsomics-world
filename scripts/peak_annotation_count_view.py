#!/usr/bin/env python3
"""Capture the adopted annotation/count view in the original pinned Linux runtime."""

import argparse
import csv
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import sys
import tarfile
import traceback
import urllib.error
import urllib.parse
import urllib.request
import zipfile

import peak_first_workflow_oracle as original


ROOT = Path(__file__).resolve().parent
REPO = 'omics-rust/rsomics-world'
RUN = 41209740891
HEAD = 'b92eb0404aad1a0222ad580c2a81c5431ff42957'
ARTIFACT = 13046468658
ZIP_SIZE = 1206132297
ZIP_SHA = 'c6a8a06c14788c557c6165f524ce3524d367b58779636826ceef1abf238afa51'
TAR_MEMBER = '__w/_temp/peak-first-workflow-oracle.tar'
TAR_SIZE = 3178844160
TAR_SHA = '0e19577fef849bfa54ff663d671028f29ca85186145b42951f14fb1442f60ef3'
PREFIX = 'peak-first-workflow-oracle/'
VIEW = 'peak-annotation-count-view-v1'
save = original.save
sha256 = original.sha256


def relative_name(name):
    parts = PurePosixPath(name).parts
    if not parts or name.startswith('/') or any(x in ('', '.', '..') for x in name.split('/')):
        raise ValueError('unsafe original archive path: ' + name)
    if not name.startswith(PREFIX):
        raise ValueError('unexpected original archive root: ' + name)
    return name[len(PREFIX):]


def selected(name):
    return (name.startswith(('R-library/', 'controller/')) or
            name in ('artifact-manifest.json', 'runtime/final-runtime-manifest.json',
                     'runtime/admitted-install-manifest.json', 'runtime/bedtools-build/bin/bedtools') or
            name.startswith(('runtime/image-', 'runtime/R-runtime/', 'runtime/installed-inventory/')) or
            name in ['inputs/alignments/' + sample + suffix
                     for sample in ('T1', 'T2') for suffix in ('.pe.bam', '.pe.bam.bai')] or
            (name.startswith('witness/calls/A-') and
             (name.endswith('/TxDb.sqlite') or '/installed-source-' in name or
              name.endswith('/session-options-DLLs.rds'))))


def verify_restore(archive, destination, choose):
    destination = Path(destination)
    if destination.exists():
        raise ValueError('restoration destination already exists')
    destination.mkdir(parents=True)
    with tarfile.open(archive, 'r:') as source:
        members = source.getmembers()
        names = [x.name for x in members]
        if len(set(names)) != len(names):
            raise ValueError('duplicate original archive member')
        manifest_member = source.getmember(PREFIX + 'artifact-manifest.json')
        if not manifest_member.isfile() or manifest_member.size > 32 * 2**20:
            raise ValueError('invalid original manifest')
        manifest = json.load(source.extractfile(manifest_member))['files']
        identities, restored, links = {}, {}, {}
        for member in members:
            if member.name == PREFIX[:-1] and member.isdir():
                continue
            name = relative_name(member.name)
            if member.isdir():
                continue
            if member.issym():
                identities[name] = dict(symlink=member.linkname)
                if choose(name):
                    links[name] = member.linkname
                continue
            if not member.isfile():
                raise ValueError('unexpected original member type: ' + name)
            digest = hashlib.sha256()
            output = None
            if choose(name):
                target = destination / name
                target.parent.mkdir(parents=True, exist_ok=True)
                output = target.open('xb')
            try:
                with source.extractfile(member) as stream:
                    for block in iter(lambda: stream.read(1024 * 1024), b''):
                        digest.update(block)
                        if output:
                            output.write(block)
            finally:
                if output:
                    output.close()
            identities[name] = dict(size=member.size, sha256=digest.hexdigest())
            if choose(name):
                (destination / name).chmod(member.mode & 0o777)
                restored[name] = identities[name]
        identities.pop('artifact-manifest.json', None)
        if identities != manifest:
            differing = sorted(k for k in set(identities) | set(manifest)
                               if identities.get(k) != manifest.get(k))
            raise ValueError('original manifest identity mismatch: ' + str(differing[:20]))
        folded = [name.casefold() for name in (*restored, *links)]
        if len(set(folded)) != len(folded):
            raise ValueError('selected original members have casefold collisions')
        for name, link in links.items():
            if PurePosixPath(link).is_absolute() or '..' in PurePosixPath(link).parts:
                raise ValueError('unsafe selected original alias: ' + name)
            target_name = str(PurePosixPath(name).parent / link)
            if target_name not in restored:
                raise ValueError('selected alias does not name a verified regular target: ' + name)
            (destination / name).symlink_to(link)
        return dict(regular_verified=sum('sha256' in x for x in identities.values()),
                    links_verified=sum('symlink' in x for x in identities.values()),
                    restored=restored, aliases=links, original_manifest=manifest)


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, file, code, message, headers, newurl):
        return None


def api(path, target, binary=False):
    target.parent.mkdir(parents=True, exist_ok=True)
    url = 'https://api.github.com/repos/' + REPO + '/' + path
    request = urllib.request.Request(url, headers={
        'Authorization': 'Bearer ' + os.environ['GH_TOKEN'],
        'Accept': 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28'})
    opener = urllib.request.build_opener(NoRedirect)
    try:
        response = opener.open(request, timeout=120)
    except urllib.error.HTTPError as error:
        if not binary or error.code != 302:
            save(Path(str(target) + '.http-error.json'), dict(status=error.code,
                 api_path=path, body=error.read(65536).decode('utf-8', errors='replace')))
            raise RuntimeError('GitHub original-evidence API failed: HTTP ' + str(error.code)) from None
        location = urllib.parse.urlsplit(error.headers['Location'])
        if location.scheme != 'https' or not location.hostname or not (
            location.hostname.endswith('.blob.core.windows.net') or
            location.hostname.endswith('.actions.githubusercontent.com') or
            location.hostname.endswith('.githubusercontent.com')):
            raise ValueError('unexpected original-evidence download redirect')
        response = urllib.request.urlopen(urllib.request.Request(error.headers['Location']), timeout=120)
    with response, target.open('xb') as output:
        shutil.copyfileobj(response, output, length=1024 * 1024)
    return None if binary else json.loads(target.read_text())


def acquire(root):
    directory = root / 'original'
    directory.mkdir()
    run = api('actions/runs/' + str(RUN), directory / 'run.json')
    if (run['id'], run['head_sha'], run['run_attempt'], run['status'], run['conclusion']) != (
            RUN, HEAD, 1, 'completed', 'success') or run['repository']['full_name'] != REPO:
        raise ValueError('original run identity/status mismatch')
    jobs = api('actions/runs/' + str(RUN) + '/attempts/1/jobs?per_page=100', directory / 'jobs.json')
    if jobs['total_count'] != 1 or jobs['jobs'][0]['id'] != 124562023993 or jobs['jobs'][0]['conclusion'] != 'success':
        raise ValueError('original completed job identity mismatch')
    api('actions/jobs/124562023993/logs', directory / 'job-124562023993.log', binary=True)
    listing = api('actions/runs/' + str(RUN) + '/artifacts?per_page=100', directory / 'artifacts.json')
    matches = [x for x in listing['artifacts'] if x['id'] == ARTIFACT]
    if len(matches) != 1:
        raise ValueError('original artifact absent/ambiguous')
    row = matches[0]
    if row['expired'] or row['digest'] != 'sha256:' + ZIP_SHA or row['size_in_bytes'] != ZIP_SIZE:
        raise ValueError('original artifact API binding differs')
    if row['workflow_run']['id'] != RUN or row['workflow_run']['head_sha'] != HEAD:
        raise ValueError('original artifact head/run mismatch')
    if row['name'] != 'peak-first-workflow-oracle-41209740891-1':
        raise ValueError('unexpected original artifact name')
    carrier = directory / (row['name'] + '.zip')
    api('actions/artifacts/' + str(ARTIFACT) + '/zip', carrier, binary=True)
    if carrier.stat().st_size != ZIP_SIZE or sha256(carrier) != ZIP_SHA:
        raise ValueError('original ZIP identity differs')
    tar_path = directory / TAR_MEMBER
    tar_path.parent.mkdir(parents=True)
    with zipfile.ZipFile(carrier) as archive:
        if archive.namelist() != [TAR_MEMBER] or archive.getinfo(TAR_MEMBER).file_size != TAR_SIZE:
            raise ValueError('original ZIP carrier membership differs')
        with archive.open(TAR_MEMBER) as stream, tar_path.open('xb') as output:
            shutil.copyfileobj(stream, output, length=1024 * 1024)
    if sha256(tar_path) != TAR_SHA:
        raise ValueError('original TAR identity differs')
    result = verify_restore(tar_path, root / 'restored', selected)
    regular = {k: v for k, v in result['restored'].items() if k.startswith('R-library/')}
    if (len(regular), sum(x['size'] for x in regular.values())) != (21337, 789875706):
        raise ValueError('original source-installed R library population differs')
    if result['aliases'] != {'R-library/Rhtslib/usrlib/libhts.so.2': 'libhts.so'}:
        raise ValueError('original selected aliases differ')
    save(root / 'restoration.json', result)
    return root / 'restored'


def make_inputs(root):
    root.mkdir(parents=True)
    annotation = []
    for strand, starts in [('+', [3980002, 3980030, 3981100, 3980500, 3982170]),
                           ('-', [3922195, 3922170, 3921100, 3920500, 3920030])]:
        for category, start in zip(['promoter', '5UTR', 'exon', 'intron', '3UTR'], starts):
            annotation.append(['chrA', start, start + 1,
                               ('plus-' if strand == '+' else 'minus-') + category, 0, strand])
    count = [['chrA', 8700000 + 400 * i + 5, 8700000 + 400 * i + 15,
              'panel-' + str(i), 0, '+'] for i in [0, 1, 2, 3, 4, 5, 7, 8, 21]]
    duplicate = ['chrA', 8700405, 8700415, 'duplicate-display', 0, '+']
    count += [duplicate, duplicate]
    count += [['chrA', start, end, name, 0, '+'] for start, end, name in [
        (8702405, 8702415, 'split-block'), (8702440, 8702450, 'split-N-gap'),
        (8702420, 8702490, 'split-two-blocks'), (8702550, 8702560, 'mate-interior'),
        (8702400, 8702710, 'outer-span')]]
    for name, rows in [('annotation', annotation), ('count', count)]:
        with (root / (name + '.bed')).open('w') as stream:
            csv.writer(stream, delimiter='\t', lineterminator='\n').writerows(rows)
        with (root / (name + '-rows.tsv')).open('w') as stream:
            writer = csv.writer(stream, delimiter='\t', lineterminator='\n')
            writer.writerow(['row_key', 'row_ordinal', 'display_id'])
            writer.writerows([[name + ':' + str(i), i, row[3]] for i, row in enumerate(rows)])
    save(root / 'input-manifest.json', dict(view=VIEW, files=original.files_identity(root),
         derivation='new coordinate views only; original graph/BAM/answers unchanged', unit='alignment'))


def case_plan(restored, inputs, calls):
    plan = []
    for name, original_name, tss in [('A-gtf-narrow', 'A-gtf-default', 'narrow'),
                                     ('A-gff3-narrow', 'A-gff3-default', 'narrow'),
                                     ('A-gtf-default', 'A-gtf-default', 'default')]:
        plan.append(dict(name=name, family='annotate', deps=[], argv=[
            'Rscript', '--vanilla', ROOT / 'peak_annotation_count_view.R', 'annotate',
            restored / 'witness/calls' / original_name / 'TxDb.sqlite', inputs,
            calls / name, tss]))
    plan.append(dict(name='reload', family='reload', deps=[x['name'] for x in plan], argv=[
        'Rscript', '--vanilla', ROOT / 'peak_annotation_count_view.R', 'reload', calls]))
    bedtools = restored / 'runtime/bedtools-build/bin/bedtools'
    for name, flags in [('C-default', []), ('C-q20', ['-q', '20']), ('C-proper', ['-p']),
                        ('C-include', ['-D', '-F']), ('C-split', ['-split'])]:
        plan.append(dict(name=name, family='count', deps=[], argv=[bedtools, 'multicov',
            '-bams', restored / 'inputs/alignments/T1.pe.bam', restored / 'inputs/alignments/T2.pe.bam',
            '-bed', inputs / 'count.bed', *flags]))
    for job in plan:
        job['argv'] = list(map(str, job['argv']))
    return plan


def check_count_effects(matrices):
    base, q, proper, include, split = [matrices[name] for name in
        ['C-default', 'C-q20', 'C-proper', 'C-include', 'C-split']]
    predicates = {
        'MAPQ': base[0] == [1, 1] and q[0] == [0, 0] and q[1] == q[2] == [1, 1],
        'proper': base[5] == [1, 1] and proper[5] == [0, 0],
        'duplicate': base[3] == [0, 0] and include[3] == [1, 1],
        'QC': base[4] == [0, 0] and include[4] == [1, 1],
        'N-gap': base[12] == [1, 1] and split[12] == [0, 0],
        'block': base[11] == split[11] == [1, 1],
        'two-block-one-alignment': base[13] == split[13] == [1, 1],
        'mate-interior': all(matrix[14] == [0, 0] for matrix in matrices.values()),
        'outer-two-alignments': base[15] == split[15] == [2, 2],
        'secondary-supplementary': base[6] == base[7] == [2, 2],
        'duplicate-rows': all(matrix[1] == matrix[9] == matrix[10] for matrix in matrices.values()),
    }
    if not all(predicates.values()):
        raise ValueError('count branch observations differ: ' + ', '.join(k for k, v in predicates.items() if not v))
    return predicates


def qualify_counts(root, inputs):
    with (inputs / 'count.bed').open() as stream:
        expected = list(csv.reader(stream, delimiter='\t'))
    matrices = {}
    for name in ['C-default', 'C-q20', 'C-proper', 'C-include', 'C-split']:
        with (root / 'calls' / name / 'stdout.txt').open() as stream:
            rows = list(csv.reader(stream, delimiter='\t'))
        if len(rows) != len(expected) or any(row[:-2] != wanted or len(row) != 8
                                           for row, wanted in zip(rows, expected)):
            raise ValueError('count row/column identity differs: ' + name)
        if any(not re.fullmatch(r'\d+', value) for row in rows for value in row[-2:]):
            raise ValueError('noninteger count: ' + name)
        matrices[name] = [[int(value) for value in row[-2:]] for row in rows]
    effects = check_count_effects(matrices)
    save(root / 'count-observations.json', dict(unit='alignment', matrices=matrices, effects=effects,
                                              sample_order=['T1', 'T2'], fragment_qualification=False))


def check_namespace_body(old, new):
    def body(path):
        return re.sub(r'^<bytecode: 0x[0-9a-fA-F]+>$', '<bytecode>', path.read_text(), flags=re.MULTILINE)
    if body(old) != body(new):
        raise ValueError('installed annotation namespace body differs: ' + old.name)


def dll_file(row, restored):
    if row == {'name': 'base', 'path': 'base'}:
        return None
    path = Path(row['path'])
    if not path.is_absolute():
        raise ValueError('unadmitted relative R DLL path: ' + row['path'])
    path = path.resolve(strict=True)
    if not (path.is_relative_to(restored / 'R-library') or path.is_relative_to('/usr/local/lib/R')):
        raise ValueError('unadmitted R DLL path: ' + str(path))
    return path


def check_namespace_versions(namespaces, packages):
    def components(version):
        if not re.fullmatch(r'[0-9]+(?:[.-][0-9]+)+', version):
            raise ValueError('invalid R package version: ' + version)
        return tuple(map(int, re.split(r'[.-]', version)))
    for row in namespaces:
        name = row['namespace']
        if name not in packages or components(row['version']) != components(packages[name]['Version']):
            raise ValueError('unadmitted loaded R namespace: ' + name)


def verify_runtime(restored, root):
    final = json.loads((restored / 'runtime/final-runtime-manifest.json').read_text())
    admitted = restored / 'runtime/admitted-install-manifest.json'
    if (final['image_digest'] != original.IMAGE or sha256(admitted) != final['install_manifest_sha256'] or
            final['status'] != 'installed and admitted; no scientific case executed'):
        raise ValueError('original runtime source/install binding mismatch')
    if sha256(original.shared.__file__) != final['shared_controller_sha256']:
        raise ValueError('shared controller differs from original runtime binding')
    if sha256(Path(original.__file__)) != sha256(restored / 'controller/peak_first_workflow_oracle.py'):
        raise ValueError('reused peak controller differs from unmodified original')
    for name, row in final['packages'].items():
        location = restored / 'R-library' / name if not row['Priority'] else Path(row['LibPath']) / name
        if original.files_identity(location) != final['package_files'][name]:
            raise ValueError('original installed package body mismatch: ' + name)
    for name in ['R', 'Rscript', 'ldd', 'bash', 'cc', 'g++']:
        row = final['executables'][name]
        if sha256(Path(row['path']).resolve(strict=True)) != row['sha256']:
            raise ValueError('immutable image executable mismatch: ' + name)
    binary = restored / 'runtime/bedtools-build/bin/bedtools'
    if sha256(binary) != final['executables']['bedtools']['sha256']:
        raise ValueError('original bedtools binary mismatch')
    original.require(root / 'runtime-check/bedtools-version', [binary, '--version'])
    if (root / 'runtime-check/bedtools-version/stdout.txt').read_text().strip() != 'bedtools v2.31.1':
        raise ValueError('restored bedtools version differs')
    original.require(root / 'runtime-check/R', ['Rscript', '--vanilla', ROOT / 'peak_annotation_count_view.R',
        'admit', restored, root / 'runtime-check/R-state'])
    for name in ['annotatePeak', 'getGenomicAnnotation', 'getNearestFeatureIndicesAndDistances']:
        old = restored / 'witness/calls/A-gtf-default' / ('installed-source-' + name + '.R')
        new = root / 'runtime-check/R-state' / ('installed-source-' + name + '.R')
        check_namespace_body(old, new)
    with (root / 'runtime-check/R-state/installed.tsv').open() as stream:
        packages = list(csv.DictReader(stream, delimiter='\t'))
    if {x['Package']: x['Version'] for x in packages} != {
            name: row['Version'] for name, row in final['packages'].items()}:
        raise ValueError('restored R installed-package population/version differs')
    with (root / 'runtime-check/R-state/namespaces.tsv').open() as stream:
        namespaces = list(csv.DictReader(stream, delimiter='\t'))
    check_namespace_versions(namespaces, final['packages'])
    with (root / 'runtime-check/R-state/DLLs.tsv').open() as stream:
        dlls = list(csv.DictReader(stream, delimiter='\t'))
    dll_paths = [path for row in dlls if (path := dll_file(row, restored)) is not None]
    libraries = set()
    targets = [binary, Path('/usr/local/lib/R/lib/libR.so')]
    targets += sorted((restored / 'R-library').rglob('*.so'))
    for index, target in enumerate(targets):
        directory = root / 'runtime-check/ldd' / str(index)
        original.require(directory, ['ldd', target])
        text = (directory / 'stdout.txt').read_text()
        if 'not found' in text:
            raise ValueError('unresolved ELF dependency: ' + str(target))
        for match in re.finditer(r'(?:=>\s+)?(/[^\s()]+)', text):
            libraries.add(str(Path(match.group(1)).resolve(strict=True)))
    libraries.update(map(str, dll_paths))
    libraries.update(str(Path(final['executables'][name]['path']).resolve(strict=True))
                     for name in ['R', 'Rscript', 'ldd', 'bash', 'cc', 'g++'])
    libraries.update(str(target.resolve(strict=True)) for target in targets)
    bodies = {name: sha256(name) for name in sorted(libraries)}
    save(root / 'runtime-check/elf-bodies.json', dict(image=original.IMAGE, libraries=bodies,
         interpretation='original package/binary bodies exact; system ELF closure pinned by same immutable image'))
    return bodies


def run(root):
    original.hosted(root)
    if root.exists():
        raise ValueError('source-view output already exists')
    root.mkdir(parents=True)
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    disk = shutil.disk_usage(runner)
    save(root / 'host.json', dict(run=os.environ['GITHUB_RUN_ID'], head=os.environ['GITHUB_SHA'],
         image=original.IMAGE, disk=disk._asdict(), runner=str(runner), runner_device=runner.stat().st_dev,
         python=sys.version, platform=original.platform.platform(), output_device=root.stat().st_dev,
         required_free_bytes=12 * 2**30))
    if root.stat().st_dev != runner.stat().st_dev or disk.free < 12 * 2**30:
        raise ValueError('insufficient or escaped hosted source-view storage; actual devices/free space in host.json')
    cache_paths = {}
    for name in ('EVIDENCE', 'TMPDIR', 'XDG_CACHE_HOME', 'XDG_CONFIG_HOME', 'XDG_DATA_HOME',
                 'PIP_CACHE_DIR', 'UV_CACHE_DIR', 'PYTHONUSERBASE', 'PYTHONPYCACHEPREFIX',
                 'MPLCONFIGDIR', 'R_LIBS_USER', 'R_LIBS_SITE', 'R_USER_CACHE_DIR'):
        path = Path(os.environ[name]).resolve(strict=True)
        if not path.is_relative_to(runner) or path.stat().st_dev != runner.stat().st_dev:
            raise ValueError('hosted cache/write path escapes RUNNER_TEMP: ' + name)
        cache_paths[name] = str(path)
    save(root / 'storage-paths.json', cache_paths)
    controller = root / 'controller'
    controller.mkdir()
    for path in [Path(__file__), ROOT / 'peak_annotation_count_view.R',
                 ROOT / 'test_peak_annotation_count_view.py', Path(original.__file__),
                 Path(original.shared.__file__), ROOT.parent / '.github/workflows/peak-annotation-count-view.yml']:
        shutil.copy2(path, controller / path.name)
    restored = acquire(root)
    os.environ.pop('GH_TOKEN', None)
    r_library = str(restored / 'R-library')
    os.environ.update(R_LIBS_USER=r_library, R_LIBS_SITE=r_library)
    before = original.files_identity(restored)
    save(root / 'protected-before.json', before)
    final = json.loads((restored / 'runtime/final-runtime-manifest.json').read_text())
    image_locations = {name: Path(row['LibPath']) / name for name, row in final['packages'].items()
                       if row['Priority']}
    image_before = {name: original.files_identity(path) for name, path in image_locations.items()}
    save(root / 'image-packages-before.json', image_before)
    bodies, input_before = {}, None
    inputs = root / 'inputs'
    try:
        bodies = verify_runtime(restored, root)
        make_inputs(inputs)
        input_before = original.files_identity(inputs)
        plan = case_plan(restored, inputs, root / 'calls')
        save(root / 'execution-plan.json', plan)
        statuses = {}
        for job in plan:
            if any(statuses[dep]['status'] != 'returned' for dep in job['deps']):
                statuses[job['name']] = dict(status='blocked', deps=job['deps'])
            else:
                statuses[job['name']] = original.capture(root / 'calls' / job['name'], job['argv'])
            save(root / 'statuses.json', statuses)
        if not all(x['status'] == 'returned' for x in statuses.values()):
            raise ValueError('source-view calls failed/blocked; originals retained')
        original.require(root / 'annotation-qualification', ['Rscript', '--vanilla',
            ROOT / 'peak_annotation_count_view.R', 'qualify', root / 'calls'])
        qualify_counts(root, inputs)
    finally:
        after = original.files_identity(restored)
        input_after = original.files_identity(inputs) if input_before is not None else None
        image_after = {name: original.files_identity(path) for name, path in image_locations.items()}
        stable = before == after and image_before == image_after and input_before == input_after and all(
            sha256(path) == digest for path, digest in bodies.items())
        save(root / 'integrity-after.json', dict(unchanged=stable, after=after,
             input_before=input_before, input_after=input_after,
             image_packages_after=image_after,
             system_ELF_bodies_unchanged=all(sha256(path) == digest for path, digest in bodies.items())))
        if not stable:
            raise ValueError('original restored source/runtime/input bodies changed')
    save(root / 'qualification.json', dict(status='complete', view=VIEW, original_run=RUN,
         source_only=True, annotation_cases=3, count_cases=5, reload_cases=1,
         native_admission=False, performance_claim=False, unit='alignment'))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--action', required=True, choices=['inputs', 'run', 'manifest', 'package'])
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--archive', type=Path)
    args = parser.parse_args()
    try:
        if args.action == 'run':
            run(args.output)
        elif args.action == 'inputs':
            make_inputs(args.output)
        elif args.action == 'manifest':
            original.bind_evidence(args.output)
        elif args.action == 'package':
            original.package_evidence(args.output, args.archive)
    except Exception as error:
        temporary = os.environ.get('RUNNER_TEMP' if os.environ.get('GITHUB_ACTIONS') == 'true' else 'TMPDIR')
        if temporary:
            temporary = Path(temporary).resolve(strict=True)
            allowed = args.output.resolve().is_relative_to(temporary)
            if original.platform.system() == 'Darwin' and temporary.stat().st_dev == Path('/').stat().st_dev:
                allowed = False
            if allowed:
                args.output.mkdir(parents=True, exist_ok=True)
                save(args.output / 'failure.json', dict(status='failed', exception=type(error).__name__,
                     message=str(error), traceback=traceback.format_exc(), source_success=False))
        raise


if __name__ == '__main__':
    main()
