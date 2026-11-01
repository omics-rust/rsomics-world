"""Private native-four qualification; no source-runtime or performance acceptance."""

import argparse
import json
import os
from pathlib import Path
import platform
import re
import shutil
import stat
import struct
import tarfile

from infercnv_source_snapshot import _hash_file, _manifest, _regular, read_snapshot_manifest, verify_and_extract
from signal_candidate import capture, check_tree, portable_names, tree_manifest


REPOSITORY = 'omics-rust/rsomics-world'
ASSET = 'assets/ecology-native-20261101.json'
SNAPSHOTS = '.autopilot/snapshots/ecology-native-2026-11-01'
OWNER = 'e1b03cb68876be2dd8bb9fbc520b5194be055d9e'
PATHS = ('CARGO_HOME', 'CARGO_TARGET_DIR', 'RUSTUP_HOME', 'TMPDIR', 'XDG_CACHE_HOME', 'EVIDENCE')
CONTROLLERS = ('scripts/ecology_candidate.py', 'scripts/test_ecology_workflows.py',
    'scripts/infercnv_source_snapshot.py', 'scripts/signal_candidate.py',
    '.github/workflows/ecology-candidate.yml', ASSET, '.autopilot/oracles/ecology-diversity-2026-10-28.json')


def check(condition, message):
    if not condition:
        raise ValueError(message)


def save(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('x') as stream:
        json.dump(value, stream, indent=2, sort_keys=True, allow_nan=False)
        stream.write('\n')


def load(path):
    return json.loads(Path(path).read_bytes())


def storage(environment=None):
    env = os.environ if environment is None else environment
    runner = Path(env['RUNNER_TEMP']).resolve(strict=True)
    paths = {name: Path(env[name]).resolve(strict=True) for name in PATHS}
    check(all(p != runner and p.is_relative_to(runner) and p.stat().st_dev == runner.stat().st_dev
              for p in paths.values()), 'hosted paths escape runner-temp device')
    usage = shutil.disk_usage(runner)
    check(usage.free >= 8 * 2**30, 'less than8GiB hosted headroom')
    return dict(paths={n: str(p) for n, p in paths.items()}, device=runner.stat().st_dev,
                disk=usage._asdict())


def hosted():
    check(os.environ.get('GITHUB_ACTIONS') == 'true' and os.environ.get('GITHUB_REPOSITORY') == REPOSITORY,
          'runtime actions require the actual hosted repository workflow')
    return storage()


def prepare_source(world, output, selector):
    world, output = Path(world), Path(output)
    authority = load(world / ASSET)
    check(selector == 'green-1' and authority['snapshot']['selector'] == selector, 'unreviewed snapshot selector')
    check(authority['owner_commit'] == OWNER and len(authority['files']) == 35,
          'reviewed owner closure differs')
    snapshot = world / SNAPSHOTS / selector
    check(set(p.name for p in snapshot.iterdir()) == {'README.md', 'files.sha256', 'checksums.sha256', 'source.tar.gz'},
          'snapshot metadata closure differs')
    manifest = read_snapshot_manifest(snapshot)
    portable_names(manifest)
    check(manifest == {n: row['sha256'] for n, row in authority['files'].items()}, 'authority source map differs')
    for name, row in authority['snapshot']['files'].items():
        _regular(snapshot / name)
        check((snapshot / name).stat().st_size == row['bytes'] and _hash_file(snapshot / name) == row['sha256'],
              'snapshot identity differs: ' + name)
    check(_hash_file(world / '.autopilot/oracles/ecology-diversity-2026-10-28.json') == authority['source_receipt_sha256'],
          'accepted source receipt differs')
    with tarfile.open(snapshot / 'source.tar.gz') as archive:
        check(all(m.mode == authority['files'][m.name]['mode'] == 0o644 for m in archive.getmembers()),
              'source Git mode differs')
    output.mkdir()
    verify_and_extract(snapshot, output / 'source', manifest)
    source = output / 'source'
    check_tree(source, manifest)
    check(all((source / n).stat().st_size == row['bytes'] for n, row in authority['files'].items()),
          'source size metadata differs')
    check(_hash_file(source / 'Cargo.lock') == authority['lock_sha256'], 'locked dependency identity differs')
    check(len(authority['fixtures']) == 8, 'complete captured fixture population differs')
    for name, row in authority['fixtures'].items():
        check(manifest[name] == row['sha256'] and (source / name).stat().st_size == row['bytes'],
              'source fixture differs: ' + name)
    for name in ('README.md', 'files.sha256', 'checksums.sha256', 'source.tar.gz'):
        shutil.copy2(snapshot / name, output / name)
    return authority


def prepare(world, output, selector):
    hosted()
    world, output = Path(world), Path(output)
    authority = prepare_source(world, output, selector)
    controller = Path(os.environ['EVIDENCE']) / 'controller'
    for name in CONTROLLERS:
        destination = controller / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(world / name, destination)
    save(controller / 'checksums.json', tree_manifest(controller))
    call = capture(output, 'controller-head', ['git', 'rev-parse', 'HEAD'], world)
    check((call / 'stdout.txt').read_text().strip() == os.environ['CONTROLLER_COMMIT'], 'actual controller Git head differs')
    save(output / 'selection.json', dict(owner_commit=OWNER, owner_tree=authority['owner_tree'], snapshot=selector,
         authority_sha256=_hash_file(world / ASSET), source_archive_sha256=_hash_file(output / 'source.tar.gz'),
         source_manifest_sha256=_hash_file(output / 'files.sha256'), lock_sha256=authority['lock_sha256'],
         controller_commit=os.environ['CONTROLLER_COMMIT'], target=os.environ['TARGET'],
         source_run=authority['source_run'], source_on_native_qualified=False, performance_claim=False, release=False))


def metadata(root):
    hosted()
    root = Path(root); source = root / 'source'
    authority = load(Path(os.environ['GITHUB_WORKSPACE']) / ASSET)
    check(platform.system() == os.environ['EXPECTED_SYSTEM'] and platform.machine() == os.environ['EXPECTED_ARCH'],
          'actual host is not the required native class')
    check(not any(os.environ.get(n) for n in ('CARGO_BUILD_TARGET', 'CARGO_BUILD_BUILD_DIR', 'RUSTC_WRAPPER',
          'RUSTC_WORKSPACE_WRAPPER', 'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS')), 'compiler/target override forbidden')
    call = capture(root, 'compiler', ['rustc', '-Vv'])
    compiler = dict(line.split(': ', 1) for line in (call / 'stdout.txt').read_text().splitlines() if ': ' in line)
    check(compiler['release'] == '1.91.0' and compiler['host'] == os.environ['EXPECTED_HOST'], 'native compiler pin differs')
    capture(root, 'cargo-version', ['cargo', '-V'])
    call = capture(root, 'metadata-preflight', ['cargo', 'metadata', '--locked', '--no-deps', '--format-version', '1'], source)
    target = Path(load(call / 'stdout.txt')['target_directory']).resolve(strict=True)
    check(target == Path(os.environ['CARGO_TARGET_DIR']).resolve(strict=True), 'Cargo target route differs')
    hosted()
    capture(root, 'fetch', ['cargo', 'fetch', '--locked'], source)
    call = capture(root, 'metadata-full', ['cargo', 'metadata', '--locked', '--offline', '--all-features', '--format-version', '1'], source)
    model = load(call / 'stdout.txt')
    check(Path(model['target_directory']).resolve(strict=True) == target, 'full dependency target differs')
    local = [p for p in model['packages'] if p['source'] is None]
    check(len(local) == 1 and local[0]['name'] == 'rsomics-ecology' and local[0]['publish'] == [], 'private/path boundary differs')
    check([t['name'] for t in local[0]['targets'] if 'bin' in t['kind']] == ['rsomics-ecology'], 'single binary identity differs')
    locked = {(p['name'], p['version'], p['source']): p for p in authority['dependencies']}
    for p in model['packages']:
        if p['source'] is not None:
            check((p['name'], p['version'], p['source']) in locked, 'resolved package not in exact registry lock')
        if p['name'].startswith('rsomics-'):
            check(p['name'] in {'rsomics-ecology', 'rsomics-common', 'rsomics-help', 'rsomics-phylo-tree'}, 'unexpected Layer-B/foundation dependency')
    for name, version in (('rsomics-common', '0.12.3'), ('rsomics-help', '0.4.0'), ('rsomics-phylo-tree', '0.2.0')):
        check([p['version'] for p in model['packages'] if p['name'] == name] == [version], 'foundation version differs')
    crates = {}
    for p in authority['dependencies']:
        name = p['name'] + '-' + p['version'] + '.crate'
        matches = list((Path(os.environ['CARGO_HOME']) / 'registry/cache').glob('*/' + name))
        check(len(matches) == 1 and _hash_file(matches[0]) == p['checksum'], 'fetched registry body differs: ' + name)
        crates[name] = dict(bytes=matches[0].stat().st_size, sha256=p['checksum'], source=p['source'])
    check(len(crates) == authority['registry_packages'], 'fetched lock population differs')
    check(_hash_file(source / 'Cargo.lock') == authority['lock_sha256'], 'Cargo changed the frozen lock')
    save(root / 'dependency-identities.json', dict(locked_registry=crates, active_packages=len(model['packages'])))
    capture(root, 'dependency-tree', ['cargo', 'tree', '--locked', '--offline', '--all-features', '-e', 'features'], source)


def quality(root):
    hosted()
    root = Path(root)
    capture(root, 'format', ['cargo', 'fmt', '--all', '--', '--check'], root / 'source')
    capture(root, 'clippy', ['cargo', 'clippy', '--locked', '--offline', '--all-targets', '--all-features', '--', '-D', 'warnings'], root / 'source')


def test_command(mode):
    check(mode in ('debug', 'release'), 'unknown test profile')
    return ['cargo', 'test', '--locked', '--offline', *(['--release'] if mode == 'release' else []),
            '--all-targets', '--all-features', '--no-fail-fast', '--', '--nocapture']


def tests(root, mode):
    hosted()
    root = Path(root)
    call = capture(root, 'test-' + mode, test_command(mode), root / 'source')
    text = (call / 'stdout.txt').read_text() + (call / 'stderr.txt').read_text()
    names = ('multi_alpha_preserves_sparse_identities_root_weights_and_unavailable_values',
             'invalid_inputs_and_aliases_leave_named_destinations_unchanged',
             'captured_sampling_profiles_replay_with_workers_and_feed_alpha',
             'zero_depth_and_failures_preserve_dimensions_and_atomic_outputs')
    check(all(re.search(r'test ' + name + r' \.\.\. ok', text) for name in names),
          'complete reviewed integration did not pass')
    summaries = re.findall(r'test result: ok\. ([0-9]+) passed; ([0-9]+) failed; ([0-9]+) ignored;', text)
    check(summaries and all(failed == ignored == '0' for _, failed, ignored in summaries)
          and sum(int(passed) for passed, _, _ in summaries) == 4, 'full reviewed test population differs')
    save(root / ('test-' + mode + '-summary.json'), dict(passed=4, integrations=names,
         workers=[1, 4], captured_fixture_files=8, per_cli_originals_retained=False, performance_claim=False))


def binary(root):
    hosted()
    root = Path(root)
    call = capture(root, 'build-release', ['cargo', 'build', '--locked', '--offline', '--release',
                   '--bin', 'rsomics-ecology', '--message-format=json'], root / 'source')
    messages = [json.loads(line) for line in (call / 'stdout.txt').read_text().splitlines()]
    paths = [Path(m['executable']).resolve(strict=True) for m in messages if m.get('reason') == 'compiler-artifact'
             and m.get('executable') and m['target']['name'] == 'rsomics-ecology' and 'bin' in m['target']['kind']
             and not m['profile']['test']]
    expected = Path(os.environ['CARGO_TARGET_DIR']) / 'release/rsomics-ecology'
    check(len(paths) == 1 and paths[0] == expected.resolve(strict=True), 'unique native Cargo binary route differs')
    with paths[0].open('rb') as stream:
        header = stream.read(32)
    target = os.environ['TARGET']
    if target.startswith('linux-'):
        check(header[:6] == b'\x7fELF\x02\x01' and struct.unpack_from('<H', header, 18)[0] ==
              (62 if target == 'linux-x86_64' else 183), 'actual Linux ELF architecture differs')
        kind = 'ELF64'
    else:
        check(header[:4] == b'\xcf\xfa\xed\xfe' and struct.unpack_from('<I', header, 4)[0] ==
              (0x01000007 if target == 'macos-x86_64' else 0x0100000c), 'actual Mach-O architecture differs')
        kind = 'Mach-O64'
    retained = root / 'native-binary' / ('rsomics-ecology-' + target)
    retained.parent.mkdir()
    shutil.copy2(paths[0], retained)
    check(bool(retained.stat().st_mode & stat.S_IXUSR), 'retained binary lost executable mode')
    save(retained.parent / 'identity.json', dict(target=target, format=kind, compiler_host=os.environ['EXPECTED_HOST'],
         bytes=retained.stat().st_size, sha256=_hash_file(retained), mode=stat.S_IMODE(retained.stat().st_mode),
         selection_sha256=_hash_file(root / 'selection.json')))


def finalize(root):
    root = Path(root)
    failures = []
    try:
        check_tree(root / 'source', _manifest((root / 'files.sha256').read_bytes()))
        controller = Path(os.environ['EVIDENCE']) / 'controller'
        recorded = load(controller / 'checksums.json')
        check({n: row for n, row in tree_manifest(controller).items() if n != 'checksums.json'} == recorded,
              'controller bodies changed')
    except (OSError, ValueError) as error:
        failures.append(str(error))
    save(root / 'integrity-after.json', dict(failures=failures, release=False, performance_claim=False))
    check(not failures, '; '.join(failures))


def package(root, archive_path):
    root, archive_path = Path(root).resolve(strict=True), Path(archive_path).absolute()
    check(not archive_path.is_relative_to(root) and not archive_path.exists() and not archive_path.is_symlink(),
          'new archive outside evidence required')
    files = {}
    for path in sorted(root.rglob('*')):
        if path.is_dir() and not path.is_symlink():
            continue
        _regular(path)
        name = path.relative_to(root).as_posix()
        files[name] = dict(bytes=path.stat().st_size, sha256=_hash_file(path), mode=stat.S_IMODE(path.stat().st_mode))
    portable_names(files)
    save(root / 'artifact-manifest.json', dict(schema='rsomics.ecology.native-four.v1', files=files,
         source_on_native_qualified=False, performance_claim=False, release=False))
    with tarfile.open(archive_path, 'x', format=tarfile.USTAR_FORMAT) as archive:
        for name in sorted([*files, 'artifact-manifest.json']):
            path = root / name
            member = tarfile.TarInfo(root.name + '/' + name)
            member.size, member.mode = path.stat().st_size, stat.S_IMODE(path.stat().st_mode)
            with path.open('rb') as stream:
                archive.addfile(member, stream)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['storage', 'prepare', 'metadata', 'quality', 'tests', 'binary', 'finalize', 'package'])
    parser.add_argument('--world', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--snapshot')
    parser.add_argument('--mode', choices=['debug', 'release'])
    parser.add_argument('--archive', type=Path)
    args = parser.parse_args()
    if args.action == 'storage':
        save(args.output, hosted())
    elif args.action == 'prepare':
        prepare(args.world, args.output, args.snapshot)
    elif args.action == 'tests':
        tests(args.output, args.mode)
    elif args.action == 'package':
        package(args.output, args.archive)
    else:
        {'metadata': metadata, 'quality': quality, 'binary': binary, 'finalize': finalize}[args.action](args.output)


if __name__ == '__main__':
    main()
