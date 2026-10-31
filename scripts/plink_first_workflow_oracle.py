#!/usr/bin/env python3
"""Capture the compact whole-fileset PLINK oracle on a pinned hosted CPU image."""

import argparse
import csv
from decimal import Decimal, InvalidOperation
from fractions import Fraction
import hashlib
import json
import math
import os
from pathlib import Path, PurePosixPath
import platform
import re
import shutil
import signal
import subprocess
import sys
import tarfile
import time
import traceback
import urllib.request
import zipfile

import plink_first_workflow_inputs as inputs


COMMIT = 'cc2c95e61c509d49aaf884d2ec0956a921770745'
IMAGE = 'sha256:a689e29bc3adf4663ef9a141d23081252764d1319c63f591a027bd6fd676f4c1'
ROOT = Path(__file__).resolve().parents[1]
CACHE_KEYS = ('TMPDIR', 'XDG_CACHE_HOME', 'XDG_CONFIG_HOME', 'XDG_DATA_HOME',
              'PIP_CACHE_DIR', 'PYTHONUSERBASE', 'PYTHONPYCACHEPREFIX', 'CCACHE_DIR')
ENV_KEYS = CACHE_KEYS + ('PATH', 'CC', 'CXX', 'CFLAGS', 'CXXFLAGS', 'CPPFLAGS', 'LDFLAGS',
                         'LD_LIBRARY_PATH', 'OMP_NUM_THREADS', 'OPENBLAS_NUM_THREADS',
                         'GCC_VERSION', 'PLINK_CONTAINER_DIGEST', 'GITHUB_SHA', 'GITHUB_RUN_ID',
                         'GITHUB_RUN_ATTEMPT', 'RUNNER_TEMP')


def save(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, sort_keys=True, indent=2, allow_nan=False) + '\n')


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def validate_spec(spec):
    if spec['schema'] != 1 or spec['source']['commit'] != COMMIT or spec['source']['tag'] != 'v2.0.0-b.1.2':
        raise ValueError('source acquisition specification mismatch')
    if spec['image']['manifest_digest'] != IMAGE or spec['image']['platform'] != 'linux/amd64':
        raise ValueError('container acquisition specification mismatch')
    if spec['limits'] != dict(samples=72, baseline_variants=64, rich_variants=24, disk_bytes=4294967296):
        raise ValueError('compact casebook size specification mismatch')
    official = 'https://s3.amazonaws.com/plink2-assets/beta1/plink2_linux_x86_64_20261024.zip'
    if spec['official']['url'] != official or COMMIT not in spec['source']['url']:
        raise ValueError('runtime acquisition URL profile mismatch')


def hosted(output):
    if os.environ.get('GITHUB_ACTIONS') != 'true' or platform.system() != 'Linux' or platform.machine() != 'x86_64':
        raise RuntimeError('scientific execution/acquisition is hosted Linux x86_64 only')
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    output = Path(output).resolve()
    if not output.is_relative_to(runner):
        raise ValueError('output escapes RUNNER_TEMP')
    for key in CACHE_KEYS:
        path = Path(os.environ[key]).resolve(strict=True)
        if not path.is_relative_to(runner) or path.stat().st_dev != runner.stat().st_dev:
            raise ValueError('hosted cache/path escapes runner: ' + key)
    if os.environ.get('PLINK_CONTAINER_DIGEST') != IMAGE or os.environ.get('GCC_VERSION') != '14.3.0':
        raise ValueError('pinned compiler container environment mismatch')
    return output


def case_sensitive_probe(path):
    path = Path(path)
    path.mkdir()
    (path / 'A').write_text('upper\n')
    (path / 'a').write_text('lower\n')
    (path / 'link').symlink_to('A')
    if (path / 'A').read_text() != 'upper\n' or (path / 'a').read_text() != 'lower\n':
        raise ValueError('hosted capture filesystem case-folds distinct source members')
    return dict(case_sensitive=True, symlink_target=os.readlink(path / 'link'),
                files={p.name: sha256(p) for p in path.iterdir() if not p.is_symlink()})


def capture(target, argv, cwd=None, timeout=1800):
    target = Path(target)
    target.mkdir(parents=True, exist_ok=False)
    request = dict(argv=list(map(str, argv)), cwd=str(Path(cwd or target).resolve()),
                   environment={key: os.environ.get(key) for key in ENV_KEYS},
                   started_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()))
    save(target / 'request.json', request)
    with (target / 'stdout.txt').open('wb') as stdout, (target / 'stderr.txt').open('wb') as stderr:
        try:
            process = subprocess.Popen(request['argv'], cwd=request['cwd'], stdout=stdout, stderr=stderr,
                                       start_new_session=True)
            code = process.wait(timeout=timeout)
            result = dict(status='returned' if code == 0 else 'failed', returncode=code)
        except (OSError, subprocess.TimeoutExpired) as error:
            if isinstance(error, subprocess.TimeoutExpired):
                os.killpg(process.pid, signal.SIGTERM)
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
            result = dict(status='failed', returncode=None, error=str(error))
    result.update(stdout_sha256=sha256(target / 'stdout.txt'), stderr_sha256=sha256(target / 'stderr.txt'),
                  finished_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()))
    save(target / 'result.json', result)
    return result


def require(target, argv, cwd=None, timeout=1800):
    result = capture(target, argv, cwd, timeout)
    if result['status'] != 'returned':
        raise RuntimeError(f'command failed; originals retained at {target}')
    return result


def download(url, destination, maximum):
    destination = Path(destination)
    destination.parent.mkdir(parents=True, exist_ok=True)
    request = urllib.request.Request(url, headers={'User-Agent': 'rsomics-world-plink-source-oracle'})
    size = 0
    with urllib.request.urlopen(request, timeout=120) as response, destination.open('xb') as stream:
        headers = dict(response.headers)
        final_url = response.url
        while True:
            chunk = response.read(1024 * 1024)
            if not chunk:
                break
            size += len(chunk)
            if size > maximum:
                raise ValueError('acquisition exceeds bound: ' + url)
            stream.write(chunk)
    receipt = dict(url=url, final_url=final_url, headers=headers, bytes=size, sha256=sha256(destination),
                   acquired_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()))
    save(str(destination) + '.json', receipt)
    return receipt


def verify_source_archive(archive, tree, anchors, destination, maximum):
    expected = {e['path']: e for e in tree['tree'] if e['type'] == 'blob'}
    if tree.get('truncated') is not False or tree.get('sha') != COMMIT:
        raise ValueError('source tree identity/truncation mismatch')
    for path, blob in anchors.items():
        if expected.get(path, {}).get('sha') != blob:
            raise ValueError('source tree anchor mismatch: ' + path)
    destination = Path(destination)
    destination.mkdir(parents=True, exist_ok=False)
    seen = set()
    total = 0
    with tarfile.open(archive, 'r:gz') as stream:
        for member in stream:
            parts = PurePosixPath(member.name).parts
            if len(parts) == 1 and member.isdir():
                continue
            if not parts or parts[0] != 'plink-ng-' + COMMIT or '..' in parts or member.name.startswith('/'):
                raise ValueError('unsafe/unexpected source archive path')
            relative = '/'.join(parts[1:])
            target = destination / relative
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
                continue
            entry = expected.get(relative)
            if entry is None or relative in seen:
                raise ValueError('source archive member outside exact tree: ' + relative)
            seen.add(relative)
            if member.issym():
                payload = member.linkname.encode()
                link = PurePosixPath(member.linkname)
                normalized = os.path.normpath(str(PurePosixPath(relative).parent / link))
                if link.is_absolute() or normalized == '..' or normalized.startswith('../'):
                    raise ValueError('source symlink escapes root')
                if entry['mode'] != '120000':
                    raise ValueError('source symlink mode differs from tree')
            elif member.isfile():
                if entry['mode'] not in ('100644', '100755'):
                    raise ValueError('source regular-file mode differs from tree')
                total += member.size
                if total > maximum:
                    raise ValueError('source unpacked bytes exceed cap')
                payload = stream.extractfile(member).read()
            else:
                raise ValueError('unsupported source archive member')
            actual = hashlib.sha1(b'blob ' + str(len(payload)).encode() + b'\0' + payload).hexdigest()
            if actual != entry['sha']:
                raise ValueError('source Git blob mismatch: ' + relative)
            target.parent.mkdir(parents=True, exist_ok=True)
            if member.issym():
                target.symlink_to(member.linkname)
            else:
                target.write_bytes(payload)
                target.chmod(0o755 if entry['mode'] == '100755' else 0o644)
    if seen != set(expected):
        raise ValueError('incomplete exact source archive')
    return dict(status='verified_git_tree_and_anchors', commit=COMMIT, files=len(seen), unpacked_bytes=total)


def acquire_runtime(root, spec):
    root.mkdir(parents=True, exist_ok=False)
    save(root / 'acquisition-specification.json', spec)
    token = json.load(urllib.request.urlopen('https://auth.docker.io/token?service=registry.docker.io&scope=repository:library/gcc:pull', timeout=60))['token']
    for kind, url, digest in [
        ('manifest', 'https://registry-1.docker.io/v2/library/gcc/manifests/' + IMAGE, IMAGE),
        ('config', 'https://registry-1.docker.io/v2/library/gcc/blobs/' + spec['image']['config_digest'], spec['image']['config_digest'])]:
        request = urllib.request.Request(url, headers={'Authorization': 'Bearer ' + token,
            'Accept': 'application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json'})
        with urllib.request.urlopen(request, timeout=60) as response:
            body = response.read(2**20)
        if 'sha256:' + hashlib.sha256(body).hexdigest() != digest:
            raise ValueError('container ' + kind + ' digest differs')
        (root / ('image-' + kind + '.json')).write_bytes(body)
    manifest = json.loads((root / 'image-manifest.json').read_text())
    config = json.loads((root / 'image-config.json').read_text())
    if manifest['config']['digest'] != spec['image']['config_digest'] or config['architecture'] != 'amd64' or config['os'] != 'linux':
        raise ValueError('container manifest/config execution class differs')
    download(spec['source']['tree_url'], root / 'tree.json', 2097152)
    tree = json.loads((root / 'tree.json').read_text())
    source_receipt = download(spec['source']['url'], root / 'source.tar.gz', spec['source']['max_bytes'])
    source_receipt['tree_verification'] = verify_source_archive(root / 'source.tar.gz', tree,
        spec['source']['git_blob_anchors'], root / 'source', spec['source']['max_unpacked_bytes'])
    save(root / 'source-identity.json', source_receipt)
    official_receipt = download(spec['official']['url'], root / 'official.zip', spec['official']['max_bytes'])
    official_dir = root / 'official'
    official_dir.mkdir()
    with zipfile.ZipFile(root / 'official.zip') as stream:
        names = []
        for item in stream.infolist():
            name = PurePosixPath(item.filename)
            if name.is_absolute() or '..' in name.parts or item.file_size > 64 * 2**20:
                raise ValueError('unsafe official archive member')
            path = official_dir / item.filename
            if item.is_dir():
                path.mkdir(parents=True, exist_ok=True)
            else:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(stream.read(item))
            names.append(item.filename)
    official = official_dir / 'plink2'
    if not official.is_file():
        raise ValueError('actual official archive lacks plink2')
    official.chmod(0o755)
    official_receipt.update(members=names, executable_sha256=sha256(official),
                            source_correspondence='unproven; separate official runtime profile')
    save(root / 'official-identity.json', official_receipt)
    build = root / 'source' / spec['source']['make_directory']
    require(root / 'calls/compiler', ['/usr/local/bin/g++', '--version'])
    require(root / 'calls/compiler-detail', ['/usr/local/bin/g++', '-v'])
    require(root / 'calls/make-version', ['make', '--version'])
    require(root / 'calls/packages', ['dpkg-query', '-W', '-f=${Package}\t${Version}\n'])
    require(root / 'calls/build-dry-run', spec['source']['make_argv'][:1] + ['-n'] + spec['source']['make_argv'][2:], build)
    require(root / 'calls/build', spec['source']['make_argv'], build, 3600)
    source_binary = build / 'plink2'
    helper = root / 'plink_pgen_capture'
    base = root / 'source/2.0'
    helper_argv = ['/usr/local/bin/g++'] + spec['source']['helper_flags'] + [
        '-I' + str(base / 'include'), str(ROOT / 'scripts/plink_pgen_capture.cc')]
    helper_argv += [str(base / s) for s in spec['source']['helper_sources']]
    helper_argv += ['-o', str(helper)]
    require(root / 'calls/helper-build', helper_argv, build, 1800)
    save(root / 'compiler-closure.json', {str(Path(tool).resolve()): sha256(Path(tool).resolve())
        for tool in ['/usr/local/bin/gcc', '/usr/local/bin/g++', shutil.which('make'), sys.executable]})
    save(root / 'licenses.json', {name: sha256(root / 'source' / name) for name in spec['source']['licenses']})
    identities = {}
    for name, binary in [('official', official), ('source', source_binary)]:
        call = root / 'calls' / (name + '-version')
        require(call, [binary, '--version'])
        version = (call / 'stdout.txt').read_text() + (call / 'stderr.txt').read_text()
        if spec['official']['version_token'] not in version or spec['official']['date_token'] not in version:
            raise ValueError('observed runtime version/date profile mismatch: ' + name)
        dependency_call = root / 'calls' / (name + '-dependencies')
        dependency_status = capture(dependency_call, ['ldd', binary])
        dependency_text = (dependency_call / 'stdout.txt').read_text() + (dependency_call / 'stderr.txt').read_text()
        static = dependency_status['returncode'] == 1 and re.search(r'not a dynamic executable|statically linked', dependency_text)
        if dependency_status['status'] != 'returned' and not static:
            raise ValueError('runtime dependency inspection failed: ' + name)
        require(root / 'calls' / (name + '-help'), [binary, '--help', 'make-pgen', 'freq', 'geno-counts',
             'sample-counts', 'missing', 'hardy', 'het', 'r2-unphased', 'indep-pairwise'])
        identities[name] = dict(executable=str(binary), sha256=sha256(binary), observed_version=version,
            linkage='observed static executable' if static else 'ldd-bound loaded library closure',
            provenance='exact Git-tree-verified source build' if name == 'source' else 'dated official archive; commit correspondence unproven')
    identities['helper'] = dict(executable=str(helper), sha256=sha256(helper),
                               controller_source_sha256=sha256(ROOT / 'scripts/plink_pgen_capture.cc'))
    require(root / 'calls/helper-dependencies', ['ldd', helper])
    libraries = {}
    for path in (root / 'calls').glob('*-dependencies/stdout.txt'):
        for token in re.findall(r'(?:=>\s*)?(/[^\s]+)', path.read_text()):
            library = Path(token)
            if library.is_file():
                key = sha256(library)
                destination = root / 'loaded-libraries' / (library.name + '.' + key[:12])
                destination.parent.mkdir(exist_ok=True)
                if not destination.exists():
                    shutil.copy2(library.resolve(), destination)
                libraries[token] = dict(realpath=str(library.resolve()), sha256=key,
                                        retained=str(destination.relative_to(root)))
    save(root / 'loaded-libraries.json', libraries)
    save(root / 'runtime.json', identities)
    return identities


QC = ['--freq', 'counts', 'cols=chrom,pos,ref,alt,provref,reffreq,altfreq,nobs', '--ac-founders',
      '--geno-counts', 'cols=chrom,pos,ref,alt,provref,homref,refalt,altxy,hapref,hapalt,missing,nobs',
      '--sample-counts', 'cols=fid,sid,sex,homref,homalt,het,refalt,het2alt,hapref,hapalt,missing',
      '--missing', 'scols=fid,sid,nmissdosage,nmiss,nmisshh,hethap,nobs,fmissdosage,fmiss,fmisshh',
      'vcols=chrom,pos,ref,alt,provref,nmissdosage,nmiss,nmisshh,hethap,nobs,fmissdosage,fmiss,fmisshh',
      '--het', 'cols=fid,sid,hom,het,nobs,f']
HWE = ['--hardy', 'midp', 'cols=chrom,pos,ref,alt,provref,ax,gcounts,hetfreq,p']


def case_plan(binary, helper, source_inputs, output, threads):
    binary, helper, source_inputs, output = map(Path, (binary, helper, source_inputs, output))
    jobs = []
    def add(name, deps, args, out, failure=False, pattern=None, executable=None):
        argv = [str(executable or binary)] + list(map(str, args))
        if executable is None:
            argv += ['--threads', str(threads), '--out', str(out)]
        jobs.append(dict(name=name, deps=deps, argv=argv, output_prefix=str(out),
                         expect_failure=failure, error_pattern=pattern,
                         snapshot_prefix=str(out) if name.endswith(('.I.import', '.F.make', '.P.make')) else None))
    for lane in ('gt', 'bed', 'pgen', 'ds', 'hds', 'rich'):
        directory = output / lane
        iprefix, fprefix, pprefix = [directory / g / 'data' for g in ('I', 'F', 'P')]
        if lane == 'bed':
            args, deps = ['--bfile', source_inputs / 'baseline', '--make-pgen'], []
        elif lane == 'pgen':
            args, deps = ['--pfile', output / 'gt/I/data', '--make-pgen'], ['gt.I.import']
        else:
            vcf = source_inputs / ('baseline.vcf' if lane == 'gt' else lane + '.vcf')
            args, deps = ['--vcf', vcf], []
            if lane in ('ds', 'hds'):
                args += ['dosage=' + lane.upper(), '--hard-call-threshold', '0.1', '--dosage-erase-threshold', '0']
            args += ['--psam', source_inputs / 'baseline.psam', '--vcf-half-call', 'error']
            if lane == 'rich':
                args += ['--split-par', 'b37']
            args += ['--make-pgen']
        add(lane + '.I.import', deps, args, iprefix)
        add(lane + '.I.decode', [lane + '.I.import'], [str(iprefix) + e for e in ('.pgen', '.pvar', '.psam')],
            iprefix, executable=helper)
        args = ['--pfile', iprefix, '--keep', source_inputs / 'keep.id', '--remove', source_inputs / 'remove.id']
        if lane in ('gt', 'bed', 'pgen'):
            args += ['--extract', source_inputs / 'selection.ids', '--exclude', source_inputs / 'exclusion.ids',
                     '--autosome', '--max-alleles', '2', '--mind', '0.125', '--geno', '0.125',
                     '--hwe', '1e-6', '0', 'midp', '--maf', '0.05', 'nonmajor', '--ac-founders']
        add(lane + '.F.make', [lane + '.I.import'], args + ['--make-pgen'], fprefix)
        eligible = ['--autosome', '--max-alleles', '2'] if lane == 'rich' else []
        for generation, prefix, make in [('F', fprefix, lane + '.F.make'), ('P', pprefix, lane + '.P.make')]:
            add(f'{lane}.{generation}.decode', [make], [str(prefix) + e for e in ('.pgen', '.pvar', '.psam')],
                prefix, executable=helper)
            add(f'{lane}.{generation}.validate', [make], ['--pfile', prefix, '--validate'], prefix.parent / 'validation')
            add(f'{lane}.{generation}.qc', [make], ['--pfile', prefix] + QC + ([] if lane == 'rich' else HWE),
                prefix.parent / 'reports/qc')
            add(f'{lane}.{generation}.hwe-tail', [make], ['--pfile', prefix] + eligible + HWE + ['log10'],
                prefix.parent / 'reports/hwe-log')
        add(lane + '.F.ld', [lane + '.F.make'], ['--pfile', fprefix] + eligible + ['--r2-unphased',
            'cols=chrom,pos,id,ref,alt,provref,maj,nonmaj,freq', '--ld-window', '1000000',
            '--ld-window-kb', '500', '--ld-window-r2', '-1'], fprefix.parent / 'ld/pairs')
        add(lane + '.F.prune', [lane + '.F.make'], ['--pfile', fprefix] + eligible + ['--indep-pairwise',
            '500kb', '1', '0.2', '--indep-order', '2'], fprefix.parent / 'ld/prune')
        add(lane + '.P.make', [lane + '.F.prune'], ['--pfile', fprefix, '--extract',
            fprefix.parent / 'ld/prune.prune.in', '--make-pgen'], pprefix)
    iprefix = output / 'gt/I/data'
    flip = output / 'probes/phaseflip/data'
    add('phaseflip.import', [], ['--vcf', source_inputs / 'phaseflip.vcf', '--psam', source_inputs / 'baseline.psam',
        '--split-par', 'b37', '--keep', source_inputs / 'keep.id', '--remove', source_inputs / 'remove.id', '--make-pgen'], flip)
    add('phaseflip.decode', ['phaseflip.import'], [str(flip) + e for e in ('.pgen', '.pvar', '.psam')], flip, executable=helper)
    add('phaseflip.ld', ['phaseflip.import'], ['--pfile', flip, '--autosome', '--max-alleles', '2', '--r2-unphased',
        'cols=chrom,pos,id,ref,alt,provref,maj,nonmaj,freq', '--ld-window', '1000000', '--ld-window-kb', '500', '--ld-window-r2', '-1'],
        flip.parent / 'pairs')
    add('phaseflip.prune', ['phaseflip.import'], ['--pfile', flip, '--autosome', '--max-alleles', '2',
        '--indep-pairwise', '500kb', '1', '0.2', '--indep-order', '2'], flip.parent / 'prune')
    for n in (64, 65, 72):
        add(f'subset.{n}', ['gt.I.import'], ['--pfile', iprefix, '--keep', source_inputs / f'keep{n}.id', '--make-pgen'],
            output / f'probes/subset{n}/data')
        subset_prefix = output / f'probes/subset{n}/data'
        add(f'subset.{n}.decode', [f'subset.{n}'], [str(subset_prefix) + e for e in ('.pgen', '.pvar', '.psam')],
            subset_prefix, executable=helper)
    add('negative.49founders', ['gt.I.import'], ['--pfile', iprefix, '--keep', source_inputs / 'keep49.id',
        '--indep-pairwise', '500kb', '1', '0.2', '--indep-order', '2'], output / 'negative/49', True,
        r'(?i)(50|founder|bad-ld)')
    for name, filename, extra, pattern in [
        ('halfcall', 'halfcall.vcf', [], r'(?i)half.call'),
        ('invalid-allele', 'invalid-allele.vcf', [], r'(?i)(allele|genotype|GT|invalid)'),
        ('invalid-cardinality', 'invalid-cardinality.vcf', ['dosage=DS'], r'(?i)(dosage|DS|invalid|malformed)')]:
        add('negative.' + name, [], ['--vcf', source_inputs / filename] + extra +
            ['--vcf-half-call', 'error', '--make-pgen'], output / 'negative' / name, True, pattern)
        if name == 'invalid-cardinality':
            jobs[-1].update(expect_failure=False, observe_outcome=True,
                           diagnostic='DS parser may accept leading scalar before comma; observe without adopting native policy')
    add('halfcall.missing', [], ['--vcf', source_inputs / 'halfcall.vcf', '--vcf-half-call', 'missing',
        '--make-pgen'], output / 'probes/halfcall-missing/data')
    add('negative.multiallelic-dosage', [], ['--vcf', source_inputs / 'unsupported-multiallelic-dosage.vcf',
        'dosage=DS', '--psam', source_inputs / 'baseline.psam', '--split-par', 'b37', '--make-pgen'],
        output / 'negative/multiallelic-dosage', True, r'(?i)multiallelic dosage')
    add('range.intersection', ['gt.I.import'], ['--pfile', iprefix, '--chr', '1', '--from-bp', '600100',
        '--to-bp', '1100100', '--extract', source_inputs / 'selection.ids', '--force-intersect', '--make-pgen'],
        output / 'probes/range/data')
    add('balanced.ld', ['gt.I.import'], ['--pfile', iprefix, '--keep', source_inputs / 'founders64.id',
        '--r2-unphased', 'cols=chrom,pos,id,maj,nonmaj', '--ld-window-kb', '500', '--ld-window-r2', '-1'],
        output / 'probes/balanced/pairs')
    for threshold in ('0.25', '0.2499'):
        add('threshold.' + threshold, ['gt.I.import'], ['--pfile', iprefix, '--keep', source_inputs / 'founders64.id',
            '--extract', source_inputs / 'ld-pair.ids',
            '--indep-pairwise', '500kb', '1', threshold, '--indep-order', '2'], output / 'probes' / threshold / 'prune')
    for mode in ('raw', 'log'):
        add('tail.I.' + mode, ['gt.I.import'], ['--pfile', iprefix, '--keep', source_inputs / 'founders64.id',
             '--hardy', 'midp'] + (['log10'] if mode == 'log' else []), output / 'probes/tail' / mode)
    return jobs


def execute_jobs(jobs, calls):
    remaining = list(jobs)
    results = {}
    snapshots = []
    while remaining:
        progressed = False
        for job in remaining[:]:
            if not all(dep in results for dep in job['deps']):
                continue
            progressed = True
            remaining.remove(job)
            target = Path(calls) / job['name']
            if any(results[d]['status'] not in ('returned', 'expected_failure') for d in job['deps']):
                results[job['name']] = dict(status='blocked', deps=job['deps'])
                save(target / 'result.json', results[job['name']])
                continue
            if job.get('output_prefix'):
                Path(job['output_prefix']).parent.mkdir(parents=True, exist_ok=True)
            for prefix in snapshots:
                verify_generation_snapshot(prefix)
            result = capture(target, job['argv'])
            if job.get('observe_outcome') and result['returncode'] is not None:
                result.update(status='observed_return' if result['returncode'] == 0 else 'observed_failure',
                              diagnostic=job['diagnostic'], native_compatibility='not adopted')
            elif job['expect_failure']:
                text = (target / 'stdout.txt').read_text(errors='replace') + (target / 'stderr.txt').read_text(errors='replace')
                if result['returncode'] not in (None, 0) and re.search(job.get('error_pattern') or r'.+', text):
                    result['status'] = 'expected_failure'
                else:
                    result.update(status='failed', classification='unexpected success or wrong failure class')
            if job.get('snapshot_prefix') and result['status'] == 'returned':
                prefix = Path(job['snapshot_prefix'])
                suffixes = ['.pgen', '.pvar', '.psam']
                if Path(str(prefix) + '.pgen.pgi').exists():
                    suffixes.append('.pgen.pgi')
                files = {suffix: sha256(str(prefix) + suffix) for suffix in suffixes}
                save(prefix.parent / 'bound-files.json', dict(files=files, authority='caller-declared metadata bound at creation'))
                snapshots.append(prefix)
            results[job['name']] = result
            save(target / 'result.json', result)
        if not progressed:
            raise ValueError('cyclic or unknown case-plan dependency')
    return results


def mutation_probes(binary, helper, compressor, case, threads):
    original = case / 'gt/I/data'
    source_samples, source_variants = metadata(original)
    directory = case / 'mutations'
    directory.mkdir()
    jobs = []
    def clone(name):
        prefix = directory / name / 'data'
        prefix.parent.mkdir()
        for suffix in ('.pgen', '.pvar', '.psam'):
            shutil.copy2(str(original) + suffix, str(prefix) + suffix)
        return prefix
    def add(name, args, failure=False, pattern=None, executable=None):
        out = directory / name / 'result'
        out.parent.mkdir(exist_ok=True)
        argv = [str(executable or binary)] + list(map(str, args))
        if executable is None:
            argv += ['--threads', str(threads), '--out', str(out)]
        jobs.append(dict(name='mutation.' + name, deps=[], expect_failure=failure,
                         error_pattern=pattern, argv=argv, output_prefix=str(out)))
    duplicate = clone('duplicate-variant')
    lines = Path(str(duplicate) + '.pvar').read_text().splitlines()
    indexes = [i for i, line in enumerate(lines) if line and not line.startswith('#')]
    first = lines[indexes[0]].split('\t')[2]
    fields = lines[indexes[1]].split('\t')
    fields[2] = first
    lines[indexes[1]] = '\t'.join(fields)
    Path(str(duplicate) + '.pvar').write_text('\n'.join(lines) + '\n')
    add('duplicate-variant-stats', ['--pfile', duplicate, '--freq', 'counts', '--ac-founders'])
    add('duplicate-variant-prune', ['--pfile', duplicate, '--indep-pairwise', '500kb', '1', '0.2'],
        True, r'(?i)(unique variant|duplicate)')
    unsorted = clone('unsorted')
    lines = Path(str(unsorted) + '.pvar').read_text().splitlines()
    fields = lines[indexes[0]].split('\t')
    fields[1] = '2000000'
    lines[indexes[0]] = '\t'.join(fields)
    Path(str(unsorted) + '.pvar').write_text('\n'.join(lines) + '\n')
    add('unsorted', ['--pfile', unsorted, '--indep-pairwise', '500kb', '1', '0.2'], True, r'(?i)sort')
    truncated = clone('truncated')
    source = Path(str(truncated) + '.pgen').read_bytes()
    Path(str(truncated) + '.pgen').write_bytes(source[:max(3, len(source)//2)])
    add('truncated', ['--pfile', truncated, '--validate'], True, r'(?i)(read|short|size|invalid|malformed|corrupt)')
    add('truncated-codec', [str(truncated) + e for e in ('.pgen', '.pvar', '.psam')], True,
        r'(?i)(read|size|error|invalid)', helper)
    wrong_shape = clone('sample-shape')
    lines = Path(str(wrong_shape) + '.psam').read_text().splitlines()
    Path(str(wrong_shape) + '.psam').write_text('\n'.join(lines[:-1]) + '\n')
    add('sample-shape', ['--pfile', wrong_shape, '--validate'], True, r'(?i)(sample|mismatch|inconsistent)')
    for name, duplicate_key in [('structured-identity', False), ('duplicate-sample', True)]:
        prefix = clone(name)
        copied = [dict(row) for row in source_samples]
        for row in copied:
            row.setdefault('FID', '0')
            row.setdefault('SID', '0')
        copied[0].update(FID='A_B', IID='C')
        copied[1].update(FID='A', IID='B_C')
        copied[2].update(FID='0', IID='Repeated', SID='one')
        copied[3].update(FID='0', IID='Repeated', SID='one' if duplicate_key else 'two')
        keys = ['FID', 'IID', 'SID', 'PAT', 'MAT', 'SEX']
        Path(str(prefix) + '.psam').write_text('#' + '\t'.join(keys) + '\n' + ''.join(
            '\t'.join(s.get(k, '0') for k in keys) + '\n' for s in copied))
        save(prefix.parent / 'identity-map.json', dict(
            source=source_samples, declared_renaming=copied,
            caveat='caller-bound renamed metadata; raw PGEN does not independently certify names/order'))
        add(name, ['--pfile', prefix, '--make-pgen'], duplicate_key, r'(?i)duplicate')
    external = directory / 'external/data'
    external.parent.mkdir()
    require(case / 'mutation-setup/external-index', [compressor, '-i', str(original) + '.pgen', str(external) + '.pgen'])
    sidecar = Path(str(external) + '.pgen.pgi')
    if not sidecar.is_file():
        raise ValueError('pgen_compress -i did not emit its actual required .pgen.pgi sidecar')
    for suffix in ('.pvar', '.psam'):
        shutil.copy2(str(original) + suffix, str(external) + suffix)
    add('external-index', [str(external) + e for e in ('.pgen', '.pvar', '.psam')], executable=helper)
    no_index = directory / 'missing-index/data'
    no_index.parent.mkdir()
    for suffix in ('.pgen', '.pvar', '.psam'):
        shutil.copy2(str(external) + suffix, str(no_index) + suffix)
    add('missing-index', [str(no_index) + e for e in ('.pgen', '.pvar', '.psam')], True,
        r'(?i)(index|pgi|open|error)', helper)
    late = directory / 'late-io/result'
    late.parent.mkdir()
    Path(str(late) + '.pgen').symlink_to('/dev/full')
    add('late-io', ['--pfile', original, '--make-pgen'], True, r'(?i)(write|space|full|error)')
    save(case / 'mutation-plan.json', jobs)
    results = execute_jobs(jobs, case / 'mutation-calls')
    return results


def verify_generation_snapshot(prefix):
    prefix = Path(prefix)
    expected = json.loads((prefix.parent / 'bound-files.json').read_text())['files']
    actual_suffixes = {'.pgen', '.pvar', '.psam'}
    if Path(str(prefix) + '.pgen.pgi').exists():
        actual_suffixes.add('.pgen.pgi')
    if set(expected) != actual_suffixes:
        raise ValueError('manifest-bound generation sidecar membership changed')
    if any(sha256(str(prefix) + suffix) != value for suffix, value in expected.items()):
        raise ValueError('manifest-bound generation metadata/parent hash changed')


def table(path):
    with Path(path).open() as stream:
        reader = csv.DictReader((line.lstrip('#') for line in stream if not line.startswith('##')), delimiter='\t')
        rows = list(reader)
        if not reader.fieldnames or any(None in row or None in row.values() for row in rows):
            raise ValueError('malformed report schema: ' + str(path))
    return rows


def metadata(prefix):
    prefix = Path(prefix)
    psam = table(str(prefix) + '.psam')
    pvar = table(str(prefix) + '.pvar')
    keys = [(s.get('FID', '0'), s['IID'], s.get('SID', '0')) for s in psam]
    if len(keys) != len(set(keys)):
        raise ValueError('duplicate complete sample identity in observed generation')
    return psam, pvar


def decoded(path):
    values = [json.loads(line) for line in Path(path).read_text().splitlines()]
    if not values or len(values) != values[0]['variants'] + 1:
        raise ValueError('incomplete codec capture')
    if any(len(v['calls']) != values[0]['samples'] for v in values[1:]):
        raise ValueError('codec/sample shape mismatch')
    return values


def bind_generation(prefix, decode_path, parent=None, selection=None):
    prefix = Path(prefix)
    samples, variants = metadata(prefix)
    calls = decoded(decode_path)
    if len(samples) != calls[0]['samples'] or len(variants) != calls[0]['variants']:
        raise ValueError('metadata/raw codec shape disagreement')
    files = {str(Path(str(prefix) + suffix).name): sha256(str(prefix) + suffix)
             for suffix in ('.pgen', '.pvar', '.psam')}
    sidecar = Path(str(prefix) + '.pgen.pgi')
    if sidecar.exists():
        files[sidecar.name] = sha256(sidecar)
    record = dict(schema=1, files=files, samples=samples, variants=variants,
                  raw_tracks_sha256=sha256(decode_path), parent=parent, selection=selection,
                  identity_contract='manifest binds names/order; raw unmanifested PGEN cannot certify names')
    save(prefix.parent / 'generation.json', record)
    return record


def printed_close(token, expected, slack='1e-12'):
    try:
        actual = Decimal(token)
    except InvalidOperation as error:
        raise ValueError('report numeric token invalid: ' + token) from error
    if not actual.is_finite():
        if math.isfinite(float(expected)):
            raise ValueError('unexpected nonfinite report value')
        return
    interval = Decimal(5).scaleb(actual.as_tuple().exponent - 1)
    if abs(actual - Decimal(str(expected))) > interval + Decimal(slack):
        raise ValueError(f'displayed rounding interval differs: {token} vs {expected}')


def hwe_midp_log(homref, het, homalt):
    n = homref + het + homalt
    rare = 2 * min(homref, homalt) + het
    support = []
    for h in range(rare % 2, min(rare, 2*n-rare) + 1, 2):
        a = (rare-h)//2
        b = n-h-a
        logw = h*math.log(2) - math.lgamma(a+1) - math.lgamma(h+1) - math.lgamma(b+1)
        support.append((h, logw))
    maximum = max(w for _, w in support)
    norm = maximum + math.log(sum(math.exp(w-maximum) for _, w in support))
    observed = next(w for h, w in support if h == het)
    selected = [(h, w) for h, w in support if w <= observed + 1e-10]
    tail_max = max(w for _, w in selected)
    logp = tail_max + math.log(sum(math.exp(w-tail_max) * (0.5 if h == het else 1) for h, w in selected)) - norm
    return logp


def check_baseline(truth, prefix, values):
    samples, variants = metadata(prefix)
    by_iid = {s['iid']: i for i, s in enumerate(truth['samples'])}
    independent = {r['id']: r for r in truth['baseline']}
    sample_indices = [by_iid[s['IID']] for s in samples]
    for row, capture_row in zip(variants, values[1:]):
        source = independent[row['ID']]
        for index, call in zip(sample_indices, capture_row['calls']):
            g = source['gt'][index]
            expected = [-9, -9] if g is None else [[0, 0], [0, 1], [1, 1]][g]
            if call[:2] != expected or call[2:] != [0, 0, None, None]:
                raise ValueError('independent baseline genotype/raw track mismatch')
    return sample_indices


def expected_filtered_baseline(truth):
    variants = [v for v in truth['baseline'] if v['id'] != 'v63']
    samples = [i for i in range(70) if Fraction(sum(v['gt'][i] is None for v in variants), len(variants)) <= Fraction(1, 8)]
    founders = [i for i in samples if truth['samples'][i]['pat'] == truth['samples'][i]['mat'] == '0']
    if len(founders) < 50:
        raise ValueError('prepared normal LD population has fewer than 50 founders')
    selected = []
    for row in variants:
        if Fraction(sum(row['gt'][i] is None for i in samples), len(samples)) > Fraction(1, 8):
            continue
        counts = [sum(row['gt'][i] == g for i in founders) for g in (0, 1, 2)]
        if sum(counts) == 0:
            continue
        if hwe_midp_log(*counts) < math.log(1e-6):
            continue
        alt = counts[1] + 2*counts[2]
        if Fraction(min(alt, 2*sum(counts)-alt), 2*sum(counts)) < Fraction(1, 20):
            continue
        selected.append(row['id'])
    return samples, selected


def check_reports(truth, prefix, raw):
    samples, variants = metadata(prefix)
    rows = {v['ID']: value for v, value in zip(variants, raw[1:])}
    counts = table(prefix.parent / 'reports/qc.gcount')
    genotype_report_rows = len(counts)
    acount = table(prefix.parent / 'reports/qc.acount')
    vmiss = table(prefix.parent / 'reports/qc.vmiss')
    smiss = table(prefix.parent / 'reports/qc.smiss')
    founders = [i for i, s in enumerate(samples) if s.get('PAT', '0') == s.get('MAT', '0') == '0']
    for row in counts:
        calls = rows[row['ID']]['calls']
        expected = [sum(c[:2] == g for c in calls) for g in ([0, 0], [0, 1], [1, 1], [-9, -9])]
        actual = [int(row[k]) for k in ('HOM_REF_CT', 'HET_REF_ALT_CTS', 'TWO_ALT_GENO_CTS', 'MISSING_CT')]
        if actual != expected:
            raise ValueError('genotype report differs from exact hardcalls')
    for row in acount:
        calls = rows[row['ID']]['calls']
        dosage = sum(sum(calls[i][:2]) for i in founders if calls[i][0] >= 0)
        observations = 2*sum(calls[i][0] >= 0 for i in founders)
        if int(row['OBS_CT']) != observations:
            raise ValueError('founder allele observation mask differs')
        printed_close(row['ALT_CTS'], dosage)
    for row in vmiss:
        calls = rows[row['ID']]['calls']
        if int(row['OBS_CT']) != len(samples) or int(row['MISSING_CT']) != sum(c[0] < 0 for c in calls):
            raise ValueError('variant missingness numerator/denominator differs')
    for s, row in enumerate(smiss):
        missing = sum(v['calls'][s][0] < 0 for v in raw[1:])
        if row['IID'] != samples[s]['IID'] or int(row['OBS_CT']) != len(variants) or int(row['MISSING_CT']) != missing:
            raise ValueError('sample missingness stale population/denominator differs')
    for s, row in enumerate(table(prefix.parent / 'reports/qc.scount')):
        calls = [v['calls'][s] for v in raw[1:]]
        expected = [sum(c[:2] == g for c in calls) for g in ([0, 0], [0, 1], [1, 1], [-9, -9])]
        if [int(row[k]) for k in ('HOM_REF_CT', 'HET_CT', 'HOM_ALT_CT', 'MISSING_CT')] != expected:
            raise ValueError('sample hardcall counts disagree')
    for row in table(prefix.parent / 'reports/qc.hardy'):
        calls = [rows[row['ID']]['calls'][i] for i in founders]
        counts = [sum(c[:2] == g for c in calls) for g in ([0, 0], [0, 1], [1, 1])]
        observed = [int(row[k]) for k in ('HOM_A1_CT', 'HET_A1_CT', 'TWO_AX_CT')]
        if sorted(observed[::2]) != sorted(counts[::2]) or observed[1] != counts[1]:
            raise ValueError('HWE founder genotype counts differ')
        if sum(counts):
            printed_close(row['MIDP'], math.exp(hwe_midp_log(*counts)), slack='0')
    expected_hets = []
    for variant in raw[1:]:
        calls = variant['calls']
        n = sum(calls[i][0] >= 0 for i in founders)
        af = Fraction(sum(sum(calls[i][:2]) for i in founders if calls[i][0] >= 0), 2*n) if n else Fraction(0)
        expected_hets.append(2*af*(1-af))
    for s, row in enumerate(table(prefix.parent / 'reports/qc.het')):
        valid = [j for j, v in enumerate(raw[1:]) if v['calls'][s][0] >= 0 and expected_hets[j] > 0]
        observed_het = sum(raw[j+1]['calls'][s][:2] == [0, 1] for j in valid)
        expected_het = sum(expected_hets[j] for j in valid)
        if int(row['OBS_CT']) != len(valid) or int(row['O(HET)']) != observed_het:
            raise ValueError('heterozygosity observation mask/count differs')
        printed_close(row['E(HET)'], float(expected_het))
        if expected_het:
            printed_close(row['F'], float(1-Fraction(observed_het)/expected_het))
    return dict(variants=len(variants), samples=len(samples), founders=len(founders),
                exact_gcount=genotype_report_rows, exact_acount=len(acount), exact_vmiss=len(vmiss), exact_smiss=len(smiss))


def check_dosage_truth(truth, prefix, raw, lane):
    samples, variants = metadata(prefix)
    by_iid = {s['iid']: i for i, s in enumerate(truth['samples'])}
    sources = {v['id']: v for v in truth['dosage']}
    checked = 0
    for variant, decoded_row in zip(variants, raw[1:]):
        source = sources[variant['ID']]
        for sample, cell in zip(samples, decoded_row['calls']):
            token = source['ds'][by_iid[sample['IID']]]
            if token == '.':
                if cell[:2] != [-9, -9] or cell[4] is not None:
                    raise ValueError('missing dosage/hardcall truth differs')
                if lane == 'hds' and cell != [-9, -9, 0, 0, None, None]:
                    raise ValueError('HDS missing-call phase/dphase absence differs')
                continue
            value = Fraction(token)
            quantized = int(value*16384 + Fraction(1, 2))
            nearest = int(Fraction(quantized, 16384) + Fraction(1, 2))
            expected = [-9, -9] if abs(quantized-nearest*16384) > 1638 else [[0, 0], [0, 1], [1, 1]][nearest]
            if sorted(cell[:2]) != expected:
                raise ValueError('source-quantized dosage regenerated hardcall differs')
            if quantized % 16384 and cell[4] != quantized:
                raise ValueError('source-quantized explicit dosage/presence differs')
            if cell[4] is not None and cell[4] != quantized:
                raise ValueError('stored raw dosage units differ')
            if lane == 'hds':
                first = min(value, Fraction(1))
                delta = int((1+first-(value-first))*16384 + Fraction(1, 2))-16384
                explicit_dosage = (abs(quantized+delta-16384) < 16384 or
                                   abs(quantized-delta-16384) < 16384)
                expected_delta = delta if explicit_dosage and delta else None
                hardcall_phase = expected == [0, 1] and (not explicit_dosage or abs(delta) > 8192)
                orientation = int(hardcall_phase and delta > 0)
                if hardcall_phase:
                    expected = [1, 0] if orientation else [0, 1]
                    if abs(delta) == quantized or abs(delta)+quantized == 32768:
                        expected_delta = None
                expected_cell = expected + [int(hardcall_phase), orientation,
                                            quantized if explicit_dosage else None, expected_delta]
                if cell != expected_cell:
                    raise ValueError('HDS raw phase/dphase presence, units or hardcall orientation differs')
            checked += 1
    return checked


def check_dosage_reports(prefix, raw):
    samples, variants = metadata(prefix)
    tracks = {v['ID']: r['calls'] for v, r in zip(variants, raw[1:])}
    founders = [i for i, s in enumerate(samples) if s.get('PAT', '0') == s.get('MAT', '0') == '0']
    for row in table(prefix.parent / 'reports/qc.acount'):
        calls = tracks[row['ID']]
        values = [Fraction(calls[i][4], 16384) if calls[i][4] is not None else
                  sum(calls[i][:2]) if calls[i][0] >= 0 else None for i in founders]
        if int(row['OBS_CT']) != 2*sum(v is not None for v in values):
            raise ValueError('dosage allele report founder observation mask differs')
        printed_close(row['ALT_CTS'], float(sum(v for v in values if v is not None)))
    for row in table(prefix.parent / 'reports/qc.vmiss'):
        calls = tracks[row['ID']]
        expected = (sum(c[0] < 0 for c in calls), sum(c[0] < 0 and c[4] is None for c in calls))
        if int(row['OBS_CT']) != len(samples) or (int(row['MISSING_CT']), int(row['MISSING_DOSAGE_CT'])) != expected:
            raise ValueError('dosage versus hardcall missingness collapsed')
    return dict(dosage_allele_rows=len(variants), separate_missingness_rows=len(variants))


def check_rich_truth(truth, prefix, raw):
    samples, variants = metadata(prefix)
    by_iid = {s['iid']: i for i, s in enumerate(truth['samples'])}
    sources = {v['id']: v for v in truth['rich']}
    for row, captured in zip(variants, raw[1:]):
        source = sources[row['ID']]
        expected_chrom = source['chrom']
        if expected_chrom == 'X':
            expected_chrom = 'PAR1' if source['pos'] <= 2699520 else 'PAR2' if source['pos'] >= 154931044 else 'X'
        if row['CHROM'] != expected_chrom or row['ALT'].split(',') != source['alt']:
            raise ValueError('rich chromosome/PAR/unused ALT metadata differs')
        if expected_chrom == '1':
            for sample, cell in zip(samples, captured['calls']):
                token = source['gt'][by_iid[sample['IID']]]
                alleles = token.replace('|', '/').split('/')
                expected = [-9, -9] if '.' in alleles else list(map(int, alleles))
                if '/' in token:
                    expected.sort()
                if cell[:2] != expected:
                    raise ValueError('independent rich/multiallelic hardcall truth differs')
                phased_het = '|' in token and expected[0] != expected[1] and expected[0] >= 0
                if bool(cell[2]) != phased_het:
                    raise ValueError('rich explicit heterozygote phase presence differs')
    if (prefix.parent / 'reports/qc.vmiss').exists():
        variants_by_id = {v['ID']: v for v in variants}
        male_count = sum(s['SEX'] == '1' for s in samples)
        y_count = sum(v['CHROM'] == 'Y' for v in variants)
        for row in table(prefix.parent / 'reports/qc.vmiss'):
            expected = male_count if variants_by_id[row['ID']]['CHROM'] == 'Y' else len(samples)
            if int(row['OBS_CT']) != expected:
                raise ValueError('rich chrY sample-based denominator policy differs')
        for sample, row in zip(samples, table(prefix.parent / 'reports/qc.smiss')):
            expected = len(variants) - (0 if sample['SEX'] == '1' else y_count)
            if row['IID'] != sample['IID'] or int(row['OBS_CT']) != expected:
                raise ValueError('rich sample variant-based denominator policy differs')
    return dict(variants=len(variants), sample_identities=len(samples),
                ploidy_policy='raw PGEN plus observed source count/missing reports; no codec chromosome inference')


def check_ld(prefix, path, raw):
    samples, variants = metadata(prefix)
    founder_indices = [i for i, s in enumerate(samples) if s.get('PAT', '0') == s.get('MAT', '0') == '0']
    tracks = {v['ID']: r['calls'] for v, r in zip(variants, raw[1:])}
    checked = 0
    for row in table(path):
        pair = []
        for key in ('ID_A', 'ID_B'):
            values = []
            for index in founder_indices:
                cell = tracks[row[key]][index]
                values.append(Fraction(cell[4], 16384) if cell[4] is not None else
                              sum(cell[:2]) if cell[0] >= 0 else None)
            pair.append(values)
        expected = inputs.correlation_squared(*pair)
        token = row['UNPHASED_R2']
        if expected is None:
            if token.lower() not in ('nan', '-nan', 'na'):
                raise ValueError('undefined pair LD silently became numeric')
        else:
            printed_close(token, float(expected))
        checked += 1
    return checked


def check_case(root, case, truth):
    results = json.loads((case / 'results.json').read_text())
    bad = {k: v for k, v in results.items() if v['status'] not in ('returned', 'expected_failure', 'observed_return', 'observed_failure')}
    if bad:
        raise ValueError('source case failures/blocked cases: ' + ', '.join(bad))
    bindings = {}
    for lane in ('gt', 'bed', 'pgen', 'ds', 'hds', 'rich'):
        lane_root = case / lane
        generations = {}
        for generation in ('I', 'F', 'P'):
            prefix = lane_root / generation / 'data'
            verify_generation_snapshot(prefix)
            raw_path = case / 'calls' / f'{lane}.{generation}.decode/stdout.txt'
            generations[generation] = bind_generation(prefix, raw_path,
                None if generation == 'I' else generations['I' if generation == 'F' else 'F']['files'],
                None if generation != 'P' else sha256(lane_root / 'F/ld/prune.prune.in'))
            if lane in ('gt', 'bed', 'pgen'):
                raw = decoded(raw_path)
                check_baseline(truth, prefix, raw)
                if generation in ('F', 'P'):
                    generations[generation]['report_checks'] = check_reports(truth, prefix, raw)
                if generation == 'F':
                    expected_samples, expected_variants = expected_filtered_baseline(truth)
                    actual_samples, actual_variants = metadata(prefix)
                    if [s['IID'] for s in actual_samples] != [truth['samples'][i]['iid'] for i in expected_samples] or [v['ID'] for v in actual_variants] != expected_variants:
                        raise ValueError('independent single-pass filter/population mask differs')
            elif lane in ('ds', 'hds'):
                generations[generation]['independent_dosage_cells'] = check_dosage_truth(truth, prefix, decoded(raw_path), lane)
                if generation in ('F', 'P'):
                    generations[generation]['dosage_reports'] = check_dosage_reports(prefix, decoded(raw_path))
            else:
                generations[generation]['rich_truth'] = check_rich_truth(truth, prefix, decoded(raw_path))
        f_samples, f_variants = metadata(lane_root / 'F/data')
        p_samples, p_variants = metadata(lane_root / 'P/data')
        keep = (lane_root / 'F/ld/prune.prune.in').read_text().split()
        drop = (lane_root / 'F/ld/prune.prune.out').read_text().split()
        eligible = [v['ID'] for v in f_variants if lane != 'rich' or (v['CHROM'] in ('1', '2') and ',' not in v['ALT'])]
        if set(keep).intersection(drop) or set(keep + drop) != set(eligible):
            raise ValueError('pruning lists do not partition eligible F')
        if lane in ('gt', 'bed', 'pgen'):
            if 'v08' not in keep or len(set(keep).intersection({'x', 'copy', 'opposite'})) > 1:
                raise ValueError('initial isolated singleton prevented later correlated-window pruning')
        if p_samples != f_samples or [v['ID'] for v in p_variants] != [v['ID'] for v in f_variants if v['ID'] in set(keep)]:
            raise ValueError('P is not exact ordered extraction from F')
        f_raw = decoded(case / 'calls' / f'{lane}.F.decode/stdout.txt')
        p_raw = decoded(case / 'calls' / f'{lane}.P.decode/stdout.txt')
        by_id = {v['ID']: r for v, r in zip(f_variants, f_raw[1:])}
        for variant, row in zip(p_variants, p_raw[1:]):
            old = by_id[variant['ID']]
            if row['calls'] != old['calls'] or row['provisional_ref'] != old['provisional_ref']:
                raise ValueError('P extraction lost phase/dosage/hardcall/reference state')
        if lane != 'rich':
            generations['F']['ld_pairs_checked'] = check_ld(lane_root / 'F/data', lane_root / 'F/ld/pairs.vcor', f_raw)
        for generation in ('I', 'F', 'P'):
            for filename, digest in generations[generation]['files'].items():
                if sha256(lane_root / generation / filename) != digest:
                    raise ValueError('parent generation was mutated by downstream operations')
        bindings[lane] = generations
    balanced = table(case / 'probes/balanced/pairs.vcor')
    pairs = {(r['ID_A'], r['ID_B']): r['UNPHASED_R2'] for r in balanced}
    for names, expected in [(('x', 'y'), 0), (('x', 'z'), 0.25), (('x', 'copy'), 1), (('x', 'opposite'), 1)]:
        token = pairs.get(names) or pairs.get(names[::-1])
        if token is None:
            raise ValueError('balanced-founder LD sentinel pair absent')
        printed_close(token, expected)
    for name in ('v05', 'v06'):
        if ('x', name) not in pairs and (name, 'x') not in pairs:
            raise ValueError('500kb inclusive LD report boundary differs')
    if ('x', 'v07') in pairs or ('v07', 'x') in pairs:
        raise ValueError('500kb LD report outside endpoint included')
    expected_threshold_counts = {'0.25': 2, '0.2499': 1}
    for threshold, n in expected_threshold_counts.items():
        if len((case / 'probes' / threshold / 'prune.prune.in').read_text().split()) != n:
            raise ValueError('strict pruning threshold/equality boundary differs')
    for n in (64, 65, 72):
        subset_prefix = case / f'probes/subset{n}/data'
        check_baseline(truth, subset_prefix, decoded(case / 'calls' / f'subset.{n}.decode/stdout.txt'))
    if (case / 'probes/phaseflip/pairs.vcor').read_bytes() != (case / 'rich/F/ld/pairs.vcor').read_bytes():
        raise ValueError('phase-only flip changed unphased LD')
    for suffix in ('.prune.in', '.prune.out'):
        if (case / 'probes/phaseflip' / ('prune' + suffix)).read_bytes() != (case / 'rich/F/ld' / ('prune' + suffix)).read_bytes():
            raise ValueError('phase-only flip changed unphased pruning')
    raw_hwe = {r['ID']: r for r in table(case / 'probes/tail/raw.hardy')}
    log_hwe = {r['ID']: r for r in table(case / 'probes/tail/log.hardy')}
    reference_log = hwe_midp_log(0, 64, 0)
    printed_close(log_hwe['v12']['NEG_LOG10_MIDP'], -reference_log/math.log(10))
    token = raw_hwe['v12']['MIDP']
    if Decimal(token) <= 0 or Decimal(token) >= Decimal('1e-12'):
        raise ValueError('compact HWE tail class is not positive tiny nonunderflow')
    printed_close(token, math.exp(reference_log), slack='0')
    save(case / 'comparison.json', dict(status='qualified_compact_source_casebook', generations=bindings,
        diagnostic_outcomes={name: value for name, value in results.items() if value['status'].startswith('observed_')},
        hwe_tail=dict(raw=token, source_log10=log_hwe['v12']['NEG_LOG10_MIDP'],
                      scalar_log10=-reference_log/math.log(10), underflow='not reached by compact 64-founder fixture'),
        exclusions=['performance', 'native implementation', 'population/biological validation', 'four-platform release']))
    return bindings


def compare_cases(left, right):
    compared = []
    a_results, b_results = [json.loads((case / 'results.json').read_text()) for case in (left, right)]
    if set(a_results) != set(b_results) or any(a_results[name]['status'] != b_results[name]['status'] for name in a_results):
        raise ValueError('source/runtime/thread failure or diagnostic classification differs')
    suffixes = ('.acount', '.gcount', '.scount', '.smiss', '.vmiss', '.hardy', '.het')
    for lane in ('gt', 'bed', 'pgen', 'ds', 'hds', 'rich'):
        for g in ('I', 'F', 'P'):
            if metadata(left / lane / g / 'data') != metadata(right / lane / g / 'data'):
                raise ValueError('source/runtime/thread metadata mismatch')
            a = decoded(left / 'calls' / f'{lane}.{g}.decode/stdout.txt')
            b = decoded(right / 'calls' / f'{lane}.{g}.decode/stdout.txt')
            if a != b:
                raise ValueError('source/runtime/thread raw codec track mismatch')
            if g != 'I':
                for suffix in suffixes:
                    lp, rp = [p / lane / g / ('reports/qc' + suffix) for p in (left, right)]
                    if lp.exists() != rp.exists() or (lp.exists() and lp.read_bytes() != rp.read_bytes()):
                        raise ValueError('source/runtime/thread principal report mismatch: ' + str(lp))
        for suffix in ('.prune.in', '.prune.out'):
            if (left / lane / 'F/ld' / ('prune' + suffix)).read_bytes() != (right / lane / 'F/ld' / ('prune' + suffix)).read_bytes():
                raise ValueError('source/runtime/thread pruning order mismatch')
        if (left / lane / 'F/ld/pairs.vcor').read_bytes() != (right / lane / 'F/ld/pairs.vcor').read_bytes():
            raise ValueError('source/runtime/thread full LD report mismatch')
        for generation in ('F', 'P'):
            for lp in (left / lane / generation / 'reports').glob('hwe-log.hardy*'):
                rp = right / lp.relative_to(left)
                if lp.read_bytes() != rp.read_bytes():
                    raise ValueError('source/runtime/thread HWE log-tail report mismatch')
        compared.append(lane)
    return dict(status='exact', lanes=compared, includes='metadata, raw track units/presence, full principal reports and ordered prune lists')


def reload_generations(root, identities):
    frozen = root / 'reload-inputs'
    frozen.mkdir()
    cases = []
    for profile in ('source', 'official'):
        for workers in (1, 4):
            for lane in ('gt', 'bed', 'pgen', 'ds', 'hds', 'rich'):
                relative = Path(profile) / str(workers) / lane
                source = root / 'oracle' / relative / 'P'
                destination = frozen / relative
                shutil.copytree(source, destination, symlinks=True)
                shutil.copy2(root / 'oracle' / profile / str(workers) / 'calls' / f'{lane}.P.decode/stdout.txt',
                             destination / 'original-codec.jsonl')
                cases.append((profile, workers, lane, relative))
    bind_manifest(frozen)
    archive = root / 'reload-inputs.tar'
    package(frozen, archive)
    reload_parent = root / 'reload'
    reload_parent.mkdir()
    with tarfile.open(archive) as stream:
        for member in stream:
            path = PurePosixPath(member.name)
            if path.is_absolute() or '..' in path.parts or path.parts[0] != frozen.name:
                raise ValueError('unsafe reload archive path')
            target = reload_parent / member.name
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
            elif member.isfile():
                target.parent.mkdir(parents=True, exist_ok=True)
                with stream.extractfile(member) as source, target.open('xb') as output:
                    shutil.copyfileobj(source, output)
            else:
                raise ValueError('unexpected link/special entry in compact generation reload bundle')
    loaded = reload_parent / frozen.name
    verify_manifest(loaded)
    results = []
    for profile, workers, lane, relative in cases:
        directory = loaded / relative
        prefix = directory / 'data'
        output = root / 'reload-calls' / relative
        output.mkdir(parents=True)
        binary = identities[profile]['executable']
        helper = identities['helper']['executable']
        require(output / 'validate', [binary, '--pfile', prefix, '--validate', '--threads', str(workers), '--out', output / 'validation'])
        require(output / 'decode', [helper] + [str(prefix) + e for e in ('.pgen', '.pvar', '.psam')])
        if decoded(output / 'decode/stdout.txt') != decoded(directory / 'original-codec.jsonl'):
            raise ValueError('reloaded P raw codec tracks differ')
        require(output / 're-QC', [binary, '--pfile', prefix] + QC + ([] if lane == 'rich' else HWE) +
                ['--threads', str(workers), '--out', output / 'qc'])
        expected_reports = list((directory / 'reports').glob('qc.*'))
        for report in expected_reports:
            if report.suffix == '.log':
                continue
            if (output / report.name).read_bytes() != report.read_bytes():
                raise ValueError('reloaded P fresh QC differs from frozen original: ' + report.name)
        results.append(dict(profile=profile, requested_workers=workers, lane=lane,
                            dataset_files=len(list(directory.glob('data.*'))), reports=len(expected_reports), status='exact'))
    verify_manifest(loaded)
    verify_manifest(frozen)
    save(root / 'reload-checks.json', dict(status='exact', archive_sha256=sha256(archive),
        generation_cases=results, scope='24 materialized P reloads, raw tracks and fresh source QC; no regenerated goldens'))


def bind_manifest(root):
    root = Path(root)
    entries = {}
    for path in sorted(root.rglob('*')):
        if path == root / 'CAPTURE.json':
            continue
        if path.is_symlink():
            entries[str(path.relative_to(root))] = dict(kind='symlink', target=os.readlink(path))
        elif path.is_file():
            entries[str(path.relative_to(root))] = dict(kind='file', bytes=path.stat().st_size, sha256=sha256(path))
    save(root / 'CAPTURE.json', dict(schema=1, files=entries,
        qualification=json.loads((root / 'qualification.json').read_text()) if (root / 'qualification.json').exists()
        else dict(status='failed_or_not_run', reason='no qualification receipt exists')))


def verify_manifest(root):
    root = Path(root)
    record = json.loads((root / 'CAPTURE.json').read_text())
    actual = {str(p.relative_to(root)) for p in root.rglob('*') if (p.is_file() or p.is_symlink()) and p != root / 'CAPTURE.json'}
    if actual != set(record['files']):
        raise ValueError('manifest membership differs')
    for name, entry in record['files'].items():
        path = root / name
        if entry['kind'] == 'symlink':
            valid = path.is_symlink() and os.readlink(path) == entry['target']
        else:
            valid = not path.is_symlink() and path.stat().st_size == entry['bytes'] and sha256(path) == entry['sha256']
        if not valid:
            raise ValueError('manifest digest/link differs: ' + name)
    return record


def package(root, archive):
    root, archive = map(Path, (root, archive))
    verify_manifest(root)
    if archive.resolve().is_relative_to(root.resolve()):
        raise ValueError('capture archive must be outside capture root')
    with tarfile.open(archive, 'w', dereference=False) as stream:
        stream.add(root, arcname=root.name, recursive=True)
    save(str(archive) + '.json', dict(sha256=sha256(archive), bytes=archive.stat().st_size,
        internal_manifest_sha256=sha256(root / 'CAPTURE.json'), preserves='original files, case, symlinks',
        qualification=verify_manifest(root)['qualification']))


def run(root, spec):
    hosted(root)
    validate_spec(spec)
    if root.exists():
        raise ValueError('capture root already exists; refuse to overwrite originals')
    root.mkdir(parents=True)
    filesystem = case_sensitive_probe(root / 'case-preservation-probe')
    controller = root / 'controller'
    controller.mkdir()
    for filename in ['scripts/plink_first_workflow_oracle.py', 'scripts/plink_first_workflow_inputs.py',
                     'scripts/plink_pgen_capture.cc', 'scripts/test_plink_first_workflow_oracle.py',
                     'assets/plink-first-runtime-20261031.json', '.github/workflows/plink-first-workflow-oracle.yml']:
        shutil.copy2(ROOT / filename, controller / Path(filename).name)
    save(root / 'host.json', dict(platform=platform.platform(), machine=platform.machine(),
        filesystem=filesystem, disk=shutil.disk_usage(root)._asdict(), environment={k: os.environ.get(k) for k in ENV_KEYS}))
    if shutil.disk_usage(root).free < 6 * 2**30:
        raise ValueError('hosted scratch free space below compact source-build requirement')
    truth = inputs.generate(root / 'inputs')
    inputs.check_truth(truth)
    identities = acquire_runtime(root / 'runtime', spec)
    completed = []
    for profile in ('source', 'official'):
        for workers in (1, 4):
            case = root / 'oracle' / profile / str(workers)
            case.mkdir(parents=True)
            jobs = case_plan(identities[profile]['executable'], identities['helper']['executable'], root / 'inputs', case, workers)
            save(case / 'plan.json', jobs)
            statuses = execute_jobs(jobs, case / 'calls')
            if statuses.get('gt.I.import', {}).get('status') == 'returned':
                statuses.update(mutation_probes(identities[profile]['executable'], identities['helper']['executable'],
                    root / 'runtime/source/2.0/build_dynamic/pgen_compress', case, workers))
            save(case / 'results.json', statuses)
            allocated = sum(p.stat().st_size for p in root.rglob('*') if p.is_file() and not p.is_symlink())
            if allocated > spec['limits']['disk_bytes']:
                raise ValueError('compact capture exceeds explicit disk budget')
            try:
                check_case(root, case, truth)
                completed.append(case)
            except Exception:
                (case / 'comparison-failure.txt').write_text(traceback.format_exc())
    if len(completed) != 4:
        raise ValueError('one or more complete source/runtime/worker graphs failed; originals retained')
    comparisons = [compare_cases(completed[0], p) for p in completed[1:]]
    reload_generations(root, identities)
    save(root / 'qualification.json', dict(status='qualified_compact_source_oracle',
        profiles=['source', 'official'], actual_worker_requests=[1, 4], comparisons=comparisons,
        official_commit_correspondence='unproven; separately pinned executable',
        performance='not_run', native_product='not_implemented'))
    bind_manifest(root)
    verify_manifest(root)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--action', choices=['run', 'manifest', 'package'], required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--archive', type=Path)
    parser.add_argument('--runtime-lock', type=Path, default=ROOT / 'assets/plink-first-runtime-20261031.json')
    args = parser.parse_args()
    try:
        if args.action == 'run':
            run(args.output, json.loads(args.runtime_lock.read_text()))
        elif args.action == 'manifest':
            bind_manifest(args.output)
        elif args.action == 'package':
            if args.archive is None:
                raise ValueError('--archive required')
            package(args.output, args.archive)
    except Exception:
        if args.output.is_dir():
            (args.output / ('failure-' + args.action + '.txt')).write_text(traceback.format_exc())
            save(args.output / 'qualification.json', dict(status='failed', action=args.action))
        raise


if __name__ == '__main__':
    main()
