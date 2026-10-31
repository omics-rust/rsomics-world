"""Private hosted qualification controller, not a release or performance receipt."""

import argparse
import ast
import gzip
import hashlib
import io
import json
import math
import os
from pathlib import Path
import platform
import re
import shutil
import struct
import subprocess
import tarfile
import zipfile

from infercnv_source_snapshot import (_hash_file, _manifest, _names_without_collisions,
                                     _no_symlink_path, _regular, _verified_members,
                                     read_snapshot_manifest, verify_and_extract)


ASSET = 'signal-native-fixtures-20261031'
SNAPSHOTS = '.autopilot/snapshots/signal-native-2026-10-31'
PATHS = ('CARGO_HOME', 'CARGO_TARGET_DIR', 'RUSTUP_HOME', 'TMPDIR', 'XDG_CACHE_HOME',
         'PIP_CACHE_DIR', 'PYTHONUSERBASE', 'MPLCONFIGDIR', 'EVIDENCE')
SOURCE_RUN = 41192955170
REPORT_SHA = '9b520a38db6aa2426ded5ebefc7004966b663fd8064b3b4165fff9ffea65adca'
VERIFICATION_SHA = '60eb9bae95d804b02a9ce08fbdc194be638693ccc6f0d45126ad8ebf04a99cc3'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def save(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + '\n')


def portable_names(names):
    _names_without_collisions(names)
    folded = set()
    prefixes = {}
    for name in names:
        if any(part.startswith('._') or part == '__MACOSX' for part in name.split('/')):
            raise ValueError('metadata member: ' + name)
        if name.casefold() in folded:
            raise ValueError('casefold collision: ' + name)
        folded.add(name.casefold())
        parts = name.split('/')
        for i in range(1, len(parts) + 1):
            prefix = '/'.join(parts[:i])
            previous = prefixes.setdefault(prefix.casefold(), prefix)
            if previous != prefix:
                raise ValueError('casefold directory collision: ' + prefix)
    _names_without_collisions(folded)


def validate_archive(file, manifest):
    portable_names(manifest)
    with tarfile.open(fileobj=file, mode='r:gz') as archive:
        portable_names([member.name for member in archive.getmembers()])
        _verified_members(archive, manifest)


def storage(environment=None, require_space=2**30):
    env = os.environ if environment is None else environment
    runner = Path(env['RUNNER_TEMP']).resolve(strict=True)
    paths = {name: Path(env[name]).resolve(strict=True) for name in PATHS}
    for name, path in paths.items():
        if not path.is_relative_to(runner) or path == runner or path.stat().st_dev != runner.stat().st_dev:
            raise ValueError('hosted write path escapes runner device: ' + name)
    usage = shutil.disk_usage(runner)
    if usage.free < require_space:
        raise ValueError('insufficient hosted free space')
    return dict(runner_temp=str(runner), device=runner.stat().st_dev,
                paths={name: str(path) for name, path in paths.items()}, disk=usage._asdict())


def pack_fixtures(sealed, world):
    """Create a regular-only exact derivative of the already retained source witness."""
    sealed, world = Path(sealed), Path(world)
    if _hash_file(sealed / 'REPORT.md') != REPORT_SHA or _hash_file(sealed / 'verification.json') != VERIFICATION_SHA:
        raise ValueError('sealed source adoption anchors differ')
    verification = json.loads((sealed / 'verification.json').read_text())
    if verification['failures'] or verification['receipt']['run_id'] != SOURCE_RUN:
        raise ValueError('source was not verified')
    original = json.loads((sealed / 'source-manifest-verification.json').read_text())
    if any(original[name] for name in ('failures', 'missing', 'unlisted')):
        raise ValueError('original member verification failed')
    original = {row['path']: row for row in original['verified']}
    source_manifest = sealed / 'selected/artifact-manifest.json'
    if _hash_file(source_manifest) != verification['source_manifest_sha256']:
        raise ValueError('original source manifest bytes differ from accepted verification')
    source_files = json.loads(source_manifest.read_text())['files']
    retained = json.loads((sealed / 'retained-hash-manifest.json').read_text())['files']
    witness = sealed / 'selected/witness'
    files = {}
    for path in sorted(witness.rglob('*')):
        if path.is_dir() and not path.is_symlink():
            continue
        _regular(path)
        name = path.relative_to(witness).as_posix()
        row = original['witness/' + name]
        sha = _hash_file(path)
        if sha != row['sha256'] or sha != retained['selected/witness/' + name]['sha256'] or \
                sha != source_files['witness/' + name]['sha256']:
            raise ValueError('selected source bytes differ: ' + name)
        files[name] = dict(sha256=sha, bytes=path.stat().st_size,
                           original_member='signal-first-workflow-oracle/witness/' + name)
    portable_names(files)
    output = world / 'assets' / (ASSET + '.tar.gz')
    if output.exists():
        raise ValueError('fixture derivative already exists')
    with output.open('xb') as raw, gzip.GzipFile(fileobj=raw, mode='wb', filename='', mtime=0) as compressed:
        with tarfile.open(fileobj=compressed, mode='w', format=tarfile.USTAR_FORMAT) as archive:
            for name, row in files.items():
                info = tarfile.TarInfo(name)
                info.size, info.mode = row['bytes'], 0o644
                with (witness / name).open('rb') as source:
                    archive.addfile(info, source)
    if output.stat().st_size >= 5 * 2**20:
        raise ValueError('fixture derivative unexpectedly large')
    save(output.with_suffix('').with_suffix('.json'), dict(schema='rsomics.signal.portable-witness.v1',
         archive_sha256=_hash_file(output), files=files,
         accepted_source=dict(run_id=SOURCE_RUN, report_sha256=REPORT_SHA,
             verification_sha256=VERIFICATION_SHA, receipt=verification['receipt'],
             original_tar=verification['outer_zip'], source_manifest_sha256=verification['source_manifest_sha256']),
         scope='Unmodified regular witness files and original notices only; no runtime/venv or links. '
               'Original expected answers remain source observations, not repaired native answers.'))


def extract_fixtures(world, output):
    world, output = Path(world), Path(output)
    archive_path = world / 'assets' / (ASSET + '.tar.gz')
    record = json.loads((world / 'assets' / (ASSET + '.json')).read_text())
    if _hash_file(archive_path) != record['archive_sha256'] or record['accepted_source']['run_id'] != SOURCE_RUN:
        raise ValueError('fixture archive binding differs')
    manifest = {name: row['sha256'] for name, row in record['files'].items()}
    _no_symlink_path(output.parent)
    if output.exists() or output.is_symlink():
        raise ValueError('fixture output already exists')
    with archive_path.open('rb') as stream:
        validate_archive(stream, manifest)
    output.mkdir()
    with tarfile.open(archive_path) as archive:
        for member in archive.getmembers():
            path = output / member.name
            path.parent.mkdir(parents=True, exist_ok=True)
            with archive.extractfile(member) as source, path.open('xb') as target:
                shutil.copyfileobj(source, target)
    check_tree(output, manifest)
    return record


def check_tree(root, manifest):
    root = Path(root)
    actual = {}
    for path in sorted(root.rglob('*')):
        if path.is_dir() and not path.is_symlink():
            continue
        _regular(path)
        actual[path.relative_to(root).as_posix()] = _hash_file(path)
    if actual != manifest:
        raise ValueError('exact source/fixture inventory or bytes changed')
    return actual


def prepare(world, output, selector):
    world, output = Path(world), Path(output)
    if not re.fullmatch(r'green-[1-9][0-9]*', selector):
        raise ValueError('explicit green-N snapshot selector required')
    snapshot = world / SNAPSHOTS / selector
    manifest = read_snapshot_manifest(snapshot)
    if set(p.name for p in snapshot.iterdir()) != {'README.md', 'files.sha256', 'source.tar.gz', 'checksums.sha256'}:
        raise ValueError('extra snapshot metadata')
    if not {'Cargo.toml', 'Cargo.lock', 'tests/first_workflow.rs', 'tests/support/fixtures.rs'} <= set(manifest):
        raise ValueError('snapshot is not the complete signal product')
    with (snapshot / 'source.tar.gz').open('rb') as stream:
        validate_archive(stream, manifest)
    output.mkdir()
    source = output / 'source'
    verify_and_extract(snapshot, source, manifest)
    for name in ('README.md', 'files.sha256', 'checksums.sha256', 'source.tar.gz'):
        shutil.copy2(snapshot / name, output / name)
    fixtures = extract_fixtures(world, output / 'fixtures')
    save(output / 'selection.json', dict(snapshot=selector, files=len(manifest),
         source_archive_sha256=_hash_file(snapshot / 'source.tar.gz'),
         source_manifest_sha256=_hash_file(snapshot / 'files.sha256'),
         lock_sha256=_hash_file(source / 'Cargo.lock'), fixture_archive_sha256=fixtures['archive_sha256'],
         fixture_manifest_sha256=_hash_file(world / 'assets' / (ASSET + '.json')),
         controller_commit=os.environ.get('CONTROLLER_COMMIT'), admitted=False, release=False))
    return source


def capture(root, name, args, cwd=None):
    out = Path(root) / name
    out.mkdir(parents=True)
    save(out / 'request.json', dict(argv=list(map(str, args)), cwd=str(cwd) if cwd else None))
    with (out / 'stdout.txt').open('wb') as stdout, (out / 'stderr.txt').open('wb') as stderr:
        result = subprocess.run(list(map(str, args)), cwd=cwd, stdout=stdout, stderr=stderr)
    save(out / 'status.json', dict(status=result.returncode, stdout_sha256=_hash_file(out / 'stdout.txt'),
                                  stderr_sha256=_hash_file(out / 'stderr.txt')))
    print(name, 'exit', result.returncode, flush=True)
    if result.returncode:
        raise RuntimeError('required command failed: ' + name)
    return out


def metadata(root):
    storage()
    root = Path(root)
    source = root / 'source'
    if platform.system() != os.environ['EXPECTED_SYSTEM'] or platform.machine() != os.environ['EXPECTED_ARCH']:
        raise ValueError('host is not the native target class')
    if any(os.environ.get(name) for name in ('CARGO_BUILD_TARGET', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER',
                                           'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS')):
        raise ValueError('compiler/target override forbidden')
    compiler = capture(root, 'compiler', ['rustc', '-Vv'])
    values = dict(line.split(': ', 1) for line in (compiler / 'stdout.txt').read_text().splitlines() if ': ' in line)
    if values['release'] != '1.91.0' or values['host'] != os.environ['EXPECTED_HOST']:
        raise ValueError('compiler is not pinned native Rust 1.91')
    capture(root, 'cargo-version', ['cargo', '-V'])
    preflight = capture(root, 'metadata-preflight', ['cargo', 'metadata', '--locked', '--no-deps', '--format-version', '1'], source)
    preflight = json.loads((preflight / 'stdout.txt').read_text())
    if Path(preflight['target_directory']).resolve() != Path(os.environ['CARGO_TARGET_DIR']).resolve(strict=True):
        raise ValueError('Cargo resolved target overrides storage')
    storage()
    resolved = capture(root, 'metadata-full', ['cargo', 'metadata', '--locked', '--format-version', '1'], source)
    data = json.loads((resolved / 'stdout.txt').read_text())
    if Path(data['target_directory']).resolve() != Path(os.environ['CARGO_TARGET_DIR']).resolve(strict=True):
        raise ValueError('full dependency metadata changed Cargo output route')
    locals_ = [p for p in data['packages'] if p['source'] is None]
    if len(locals_) != 1 or locals_[0]['name'] != 'rsomics-signal' or locals_[0]['publish'] != []:
        raise ValueError('private product/path-dependency boundary differs')
    if [t['name'] for t in locals_[0]['targets'] if 'bin' in t['kind']] != ['rsomics-signal']:
        raise ValueError('single binary boundary differs')
    for package in data['packages']:
        if package['name'].startswith('rsomics-') and package['name'] not in {
                'rsomics-signal', 'rsomics-common', 'rsomics-help', 'rsomics-bamio'}:
            raise ValueError('unexpected rsomics dependency: ' + package['name'])
    for name, version in {'rsomics-common': '0.12.3', 'rsomics-help': '0.4.0',
                          'rsomics-bamio': '0.8.4', 'bigtools': '0.5.8'}.items():
        if [p['version'] for p in data['packages'] if p['name'] == name] != [version]:
            raise ValueError('pinned product dependency differs: ' + name)


def tests(root, mode):
    storage()
    root = Path(root)
    scratch = root / ('test-' + mode + '-scratch')
    scratch.mkdir()
    os.environ['TMPDIR'] = str(scratch)
    os.environ['RSOMICS_SIGNAL_FIXTURES'] = str(root / 'fixtures')
    args = ['cargo', 'test', '--locked', '--no-fail-fast']
    if mode == 'release':
        args.append('--release')
    args.extend(['--', '--nocapture'])
    try:
        logs = capture(root, 'test-' + mode, args, root / 'source')
        text = (logs / 'stdout.txt').read_text() + (logs / 'stderr.txt').read_text()
        populations = [int(n) for n in re.findall(r'test result: ok\. ([0-9]+) passed; 0 failed; 0 ignored;', text)]
        if not any(n >= 9 for n in populations) or 1 not in populations:
            raise ValueError('full nine-library plus connected integration population changed; review required')
        graphs = sorted(scratch.glob('signal-first-six-*'))
        if len(graphs) != 1:
            raise ValueError('retained connected native graph is not unique')
        calls = sorted(graphs[0].glob('call-*.json'))
        if not calls:
            raise ValueError('native CLI logs absent')
        statuses = [json.loads(p.read_text())['status'] for p in calls]
        # Population may grow after review; preserve all observed failures and let the Rust test adjudicate them.
        save(root / ('test-' + mode + '-graph.json'), dict(path=str(graphs[0]), calls=len(calls),
             successes=statuses.count(0), nonzero=[s for s in statuses if s != 0],
             files=tree_manifest(graphs[0])))
        binary = Path(json.loads(calls[0].read_text())['binary']).resolve(strict=True)
        expected = Path(os.environ['CARGO_TARGET_DIR']) / mode / 'rsomics-signal'
        if binary != expected.resolve(strict=True):
            raise ValueError('integration did not execute expected native binary')
        retained = root / ('binary-' + mode)
        shutil.copy2(binary, retained)
        save(root / ('binary-' + mode + '.json'), dict(sha256=_hash_file(retained), original=str(binary),
             compiler_host=os.environ['EXPECTED_HOST'], selection_sha256=_hash_file(root / 'selection.json')))
    finally:
        save(root / ('test-' + mode + '-scratch-files.json'), tree_manifest(scratch))


def numeric_equal(a, b, absolute=1e-12, relative=1e-12):
    if math.isnan(a) or math.isnan(b):
        return math.isnan(a) and math.isnan(b)
    if math.isinf(a) or math.isinf(b):
        return a == b
    return abs(a - b) <= absolute + relative * abs(b)


def npz_model(path):
    arrays = {}
    with zipfile.ZipFile(path) as archive:
        if set(archive.namelist()) != {'matrix.npy', 'labels.npy'} or len(archive.namelist()) != 2:
            raise ValueError('NPZ member schema differs')
        for name in archive.namelist():
            with archive.open(name) as stream:
                if stream.read(8) != b'\x93NUMPY\x01\x00':
                    raise ValueError('NPY v1 schema differs')
                header = ast.literal_eval(stream.read(struct.unpack('<H', stream.read(2))[0]).decode('latin1').strip())
                data = stream.read()
            if header['fortran_order']:
                raise ValueError('unexpected Fortran storage')
            count = math.prod(header['shape'])
            dtype = header['descr']
            if dtype == '<f8':
                values = list(struct.unpack('<' + 'd' * count, data))
            elif re.fullmatch(r'<U[0-9]+', dtype):
                width = int(dtype[2:]) * 4
                if len(data) != count * width:
                    raise ValueError('Unicode array length differs')
                values = [data[i:i + width].decode('utf-32-le').rstrip('\0') for i in range(0, len(data), width)]
            else:
                raise ValueError('unexpected NPZ dtype: ' + dtype)
            arrays[name[:-4]] = dict(header=header, values=values)
    return arrays


def matrix_model(path):
    with gzip.open(path, 'rt') as stream:
        line = next(stream)
        if not line.startswith('@'):
            raise ValueError('missing matrix header')
        header = json.loads(line[1:])
        rows = [line.rstrip('\n').split('\t') for line in stream]
    for row in rows:
        if len(row) != 6 + header['sample_boundaries'][-1]:
            raise ValueError('matrix shape differs')
    return dict(header=header, identities=[r[:6] for r in rows], values=[float(v) for r in rows for v in r[6:]])


def worker_matrix_model(path, workers):
    model = matrix_model(path)
    observed = model['header'].pop('proc number', None)
    if type(observed) is not int or observed != workers:
        raise ValueError('matrix processor metadata differs from requested workers')
    return model


def bam_model(path):
    """Record payloads and reference dictionary, excluding truthful path-dependent PG/CL text."""
    with gzip.open(path, 'rb') as stream:
        if stream.read(4) != b'BAM\x01':
            raise ValueError('not BAM')
        header = stream.read(struct.unpack('<i', stream.read(4))[0]).decode()
        references = []
        for _ in range(struct.unpack('<i', stream.read(4))[0]):
            name = stream.read(struct.unpack('<i', stream.read(4))[0]).decode().rstrip('\0')
            references.append((name, struct.unpack('<i', stream.read(4))[0]))
        records = []
        while block := stream.read(4):
            size = struct.unpack('<i', block)[0]
            if size < 32:
                raise ValueError('invalid BAM record length')
            record = stream.read(size)
            if len(record) != size:
                raise ValueError('truncated BAM record')
            records.append(record.hex())
    return dict(header=['\t'.join(field for field in line.split('\t') if not field.startswith('CL:'))
                        if line.startswith('@PG\t') else line for line in header.splitlines()],
                references=references, records=records)


def compare_model(a, b, matrix=False):
    a, b = dict(a), dict(b)
    av, bv = a.pop('values'), b.pop('values')
    if a != b or len(av) != len(bv):
        raise ValueError('semantic schema/identity differs')
    if not all(numeric_equal(x, y, absolute=0.5e-6 if matrix else 1e-12) for x, y in zip(av, bv)):
        raise ValueError('numeric classes or fixed-stage values differ')


def worker_plan(native_graph, fixture, output, workers):
    graph, fixture, output = map(Path, (native_graph, fixture, output))
    commands = []
    def add(name, args):
        commands.append(dict(name=name, args=['--threads', str(workers), *map(str, args)]))
    tracks = [output / (s + '.bw') for s in ('a', 'b')]
    for sample, track in zip(('a', 'b'), tracks):
        add('track-' + sample, ['track', '--bam', graph / (sample + '.source-native.bam'), '--out', track,
            '--bin-size', '20', '--normalize', 'CPM', '--exact-scaling'])
    add('compare', ['compare', '--input', 'bigwig', '--first', tracks[0], '--second', tracks[1],
                   '--out', output / 'compare.bw', '--bin-size', '20'])
    add('average', ['average', '--bigwig', *tracks, tracks[0], '--scale-factors', '1:0.5:2',
                   '--out', output / 'average.bw', '--bin-size', '20'])
    derived = [output / 'compare.bw', output / 'average.bw']
    for mode in ('bins', 'regions'):
        args = ['summarize', mode, '--input', 'bigwig', '--files', *derived, '--labels', 'A', 'B',
                '--out', output / (mode + '.npz'), '--raw-counts', output / (mode + '.tsv'), '--bin-size', '100']
        if mode == 'regions':
            args += ['--regions', fixture / 'inputs/authored/regions.bed']
        add('summary-' + mode, args)
    for mode in ('reference', 'scale'):
        args = ['matrix', mode, '--bigwig', *derived, '--regions', fixture / 'inputs/authored/groups.bed',
                '--out', output / ('matrix-' + mode + '.gz'), '--before', '50',
                '--after', '100' if mode == 'reference' else '50', '--labels', 'A', 'B']
        if mode == 'scale':
            args += ['--body-length', '200', '--unscaled-5prime', '20', '--unscaled-3prime', '20']
        add('matrix-' + mode, args)
    add('filter', ['filter', '--bam', fixture / 'generated-inputs/a.raw.bam', '--out', output / 'passed.bed',
        '--format', 'fragments-bed', '--filtered-out', output / 'rejected.bam', '--metrics', output / 'metrics.tsv',
        '--atac-shift', '--min-mapq', '20', '--exclude-flags', '2816'])
    for bw in ('a', 'b', 'compare', 'average'):
        for size in (1, 1000):
            add('decode-' + bw + '-' + str(size), ['summarize', 'bins', '--input', 'bigwig', '--files', output / (bw + '.bw'),
                '--out', output / (bw + '-' + str(size) + '.npz'), '--bin-size', str(size), '--labels', bw])
    return dict(workers=workers, commands=commands,
                prerequisite='Actual integration-test native filter/shift plus explicit TEST-only Rust sort/index BAMs; '
                             'not retained upstream BAMs. This lane begins from those common native prerequisites.')


def workers(root):
    storage()
    root = Path(root)
    graph = Path(json.loads((root / 'test-release-graph.json').read_text())['path'])
    binary = root / 'binary-release'
    for n in (1, 4):
        output = root / 'workers' / str(n)
        output.mkdir(parents=True)
        plan = worker_plan(graph, root / 'fixtures', output, n)
        save(output / 'plan.json', plan)
        for row in plan['commands']:
            capture(output / 'commands', row['name'], [binary, *row['args']])
        save(output / 'bindings.json', dict(binary_sha256=_hash_file(binary),
             prerequisite_files={p.name: _hash_file(p) for p in graph.glob('*.source-native.bam*')},
             files=tree_manifest(output)))
    first, second = [root / 'workers' / str(n) for n in (1, 4)]
    compared = []
    for a in sorted(first.iterdir()):
        b = second / a.name
        if a.suffix == '.npz':
            av, bv = npz_model(a), npz_model(b)
            if av['labels'] != bv['labels']:
                raise ValueError('sample labels differ')
            compare_model(av['matrix'], bv['matrix'])
        elif a.suffix == '.gz':
            compare_model(worker_matrix_model(a, 1), worker_matrix_model(b, 4), matrix=True)
        elif a.suffix == '.bam':
            if bam_model(a) != bam_model(b):
                raise ValueError('BAM record semantics differ')
        elif a.suffix in ('.bed', '.tsv'):
            if a.read_bytes() != b.read_bytes():
                raise ValueError('identities, counters or serialized values differ: ' + a.name)
        else:
            continue
        compared.append(a.name)
    if len(compared) < 15:
        raise ValueError('worker semantic comparison population incomplete')
    save(root / 'worker-equality.json', dict(passed=True, workers=[1, 4], files=compared,
         bw_comparison='native decoded per-base NPZ plus default coarse summaries; same-file upstream checked only on Linux',
         ignored='gzip/ZIP/BAM container headers and BAM PG CL command-line paths only; other PG fields and all numeric classes exact',
         source_sha256=_hash_file(root / 'files.sha256'), performance_claim=False, release=False))


def tree_manifest(root):
    return {p.relative_to(root).as_posix(): dict(bytes=p.stat().st_size, sha256=_hash_file(p))
            for p in sorted(Path(root).rglob('*')) if p.is_file() and not p.is_symlink()}


def finalize(root):
    root = Path(root)
    failures = []
    try:
        check_tree(root / 'source', _manifest((root / 'files.sha256').read_bytes()))
        record = json.loads((root / 'controller' / (ASSET + '.json')).read_text())
        check_tree(root / 'fixtures', {n: r['sha256'] for n, r in record['files'].items()})
    except (OSError, ValueError) as exc:
        failures.append(str(exc))
    save(root / 'integrity-after.json', dict(failures=failures, release=False, performance_claim=False))
    save(root / 'artifact-manifest.json', tree_manifest(root))
    if failures:
        raise ValueError('; '.join(failures))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['prepare', 'metadata', 'tests', 'workers', 'finalize', 'pack-fixtures'])
    parser.add_argument('--world', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--snapshot')
    parser.add_argument('--mode', choices=['debug', 'release'])
    parser.add_argument('--sealed', type=Path)
    args = parser.parse_args()
    if args.action == 'prepare':
        storage()
        prepare(args.world, args.output, args.snapshot)
    elif args.action == 'metadata':
        metadata(args.output)
    elif args.action == 'tests':
        tests(args.output, args.mode)
    elif args.action == 'workers':
        workers(args.output)
    elif args.action == 'finalize':
        finalize(args.output)
    else:
        pack_fixtures(args.sealed, args.world)


if __name__ == '__main__':
    main()
