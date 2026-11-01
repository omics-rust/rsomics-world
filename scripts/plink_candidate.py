"""Four-native qualification of a frozen private PLINK product."""

import argparse
import json
import os
from pathlib import Path
import platform
import re
import shutil
import stat
import tarfile

from infercnv_source_snapshot import (_hash_file, _manifest, _regular,
                                     read_snapshot_manifest, verify_and_extract)
from signal_candidate import capture, check_tree, portable_names, storage, tree_manifest


REPOSITORY = 'omics-rust/rsomics-world'
SNAPSHOTS = '.autopilot/snapshots/plink-native-2026-11-01'
SCHEMA = 'rsomics.plink.gate-artifact.v1'
POLICY = '4b99373876681ecc7379e0653d26eff696a6c2f7dbc97482c57a31b7c3caf806'
EXPECTED = 'd04404749b8b28ba53fbfab3805c47101c2fc7b0e7fff35d69f76b3f532132bc'
LANES = ('gt', 'bed', 'pgen', 'ds', 'hds', 'rich')
SHAPES = {'gt': ((72, 64), (69, 58), (69, 5)), 'bed': ((72, 64), (69, 58), (69, 5)),
          'pgen': ((72, 64), (69, 58), (69, 5)), 'ds': ((72, 24), (70, 24), (70, 2)),
          'hds': ((72, 24), (70, 24), (70, 2)), 'rich': ((72, 24), (70, 24), (70, 1))}
CONTROLLERS = ('scripts/plink_candidate.py', 'scripts/plink_source_on_native.py',
    'scripts/test_plink_candidate_workflow.py', 'scripts/infercnv_source_snapshot.py',
    'scripts/signal_candidate.py', 'scripts/plink_first_workflow_oracle.py',
    'scripts/plink_first_workflow_inputs.py', 'scripts/plink_pgen_capture.cc',
    '.github/workflows/plink-candidate.yml', 'assets/plink-first-runtime-20261031.json')
GRAPH_POPULATIONS = {'green-1': (114, 50), 'green-2': (114, 50), 'green-3': (116, 52)}


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


def graph_population(selector):
    check(selector in GRAPH_POPULATIONS, 'unsupported source graph profile')
    return GRAPH_POPULATIONS[selector]


def hosted():
    check(os.environ.get('GITHUB_ACTIONS') == 'true' and
          os.environ.get('GITHUB_REPOSITORY') == REPOSITORY,
          'native runtime actions require the hosted repository workflow')
    result = storage(require_space=2**30)
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    for name in ('CLANG_MODULE_CACHE_PATH', 'CCACHE_DIR', 'SCCACHE_DIR'):
        path = Path(os.environ[name]).resolve(strict=True)
        check(path.is_relative_to(runner) and path != runner and path.stat().st_dev == runner.stat().st_dev,
              'native tool cache escapes checked runner device: ' + name)
    return result


def prepare_source(world, output, selector):
    check(isinstance(selector, str) and re.fullmatch(r'green-[1-9][0-9]*', selector), 'invalid snapshot selector')
    graph_population(selector)
    snapshot = Path(world) / SNAPSHOTS / selector
    manifest = read_snapshot_manifest(snapshot)
    portable_names(manifest)
    check((snapshot / 'source.tar.gz').stat().st_size < 50 * 2**20, 'unexpectedly large source snapshot')
    required = {'Cargo.toml', 'Cargo.lock', 'build.rs', 'native/pgen_shim.h', 'native/pgen_shim.cc',
        'vendor/SOURCE.json', 'LICENSES.md', 'THIRD_PARTY_NOTICES.md', 'src/lib.rs', 'src/cli.rs',
        'src/io/pgen.rs', 'src/stats/mod.rs', 'src/ld/pairs.rs', 'src/ld/prune.rs',
        'tests/workflow.rs', 'tests/workflow.py', 'tests/compare_workflow.py',
        'tests/COMPARISON.md', 'tests/fixtures/expected/SOURCE.json', 'tests/fixtures/inputs/manifest.json'}
    if selector == 'green-3':
        required.add('src/numeric.rs')
    check(required <= manifest.keys(), 'snapshot omits product/source/license/fixture closure')
    check(manifest['tests/COMPARISON.md'] == POLICY and
          manifest['tests/fixtures/expected/SOURCE.json'] == EXPECTED, 'frozen comparison/source contract differs')
    output = Path(output)
    output.mkdir()
    verify_and_extract(snapshot, output / 'source', manifest)
    for name in ('README.md', 'files.sha256', 'source.tar.gz', 'checksums.sha256'):
        shutil.copy2(snapshot / name, output / name)
    return manifest


def prepare(world, output, selector):
    hosted()
    world, output = Path(world), Path(output)
    manifest = prepare_source(world, output, selector)
    save(output / 'selection.json', dict(snapshot=selector, source_files=len(manifest),
        source_archive_sha256=_hash_file(output / 'source.tar.gz'),
        source_manifest_sha256=_hash_file(output / 'files.sha256'),
        lock_sha256=_hash_file(output / 'source/Cargo.lock'),
        vendor_manifest_sha256=_hash_file(output / 'source/vendor/SOURCE.json'),
        comparison_sha256=POLICY, expected_manifest_sha256=EXPECTED,
        controller_commit=os.environ['CONTROLLER_COMMIT'], target=os.environ['TARGET'],
        release=False, performance_claim=False))
    controller = Path(os.environ['EVIDENCE']) / 'controller'
    for name in CONTROLLERS:
        target = controller / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(world / name, target)
    save(controller / 'checksums.json', tree_manifest(controller))


def metadata(root):
    hosted()
    root = Path(root); source = root / 'source'
    check(platform.system() == os.environ['EXPECTED_SYSTEM'] and platform.machine() == os.environ['EXPECTED_ARCH'],
          'host differs from required actual native target')
    overrides = ('CARGO_BUILD_TARGET', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'RUSTFLAGS',
        'CARGO_ENCODED_RUSTFLAGS', 'CXX', 'CXXFLAGS', 'CPPFLAGS', 'CC', 'CFLAGS', 'CXXSTDLIB',
        'HOST_CXX', 'HOST_CC', 'TARGET_CXX', 'TARGET_CC', 'CC_KNOWN_WRAPPER_CUSTOM')
    check(not any(os.environ.get(name) for name in overrides), 'compiler/target override is forbidden')
    call = capture(root, 'compiler', ['rustc', '-Vv'])
    values = dict(line.split(': ', 1) for line in (call / 'stdout.txt').read_text().splitlines() if ': ' in line)
    check(values['release'] == '1.91.0' and values['host'] == os.environ['EXPECTED_HOST'],
          'compiler is not the pinned native Rust1.91 host')
    capture(root, 'cargo-version', ['cargo', '-V'])
    capture(root, 'cpp-version', ['c++', '--version'])
    for label, extra in [('metadata-preflight', ['--no-deps']), ('metadata-full', [])]:
        call = capture(root, label, ['cargo', 'metadata', '--locked', *extra, '--format-version', '1'], source)
        data = load(call / 'stdout.txt')
        check(Path(data['target_directory']).resolve() == Path(os.environ['CARGO_TARGET_DIR']).resolve(strict=True),
              'Cargo resolved target escapes admitted route')
        hosted()
        if label == 'metadata-preflight':
            continue
        locals_ = [row for row in data['packages'] if row['source'] is None]
        check(len(locals_) == 1 and locals_[0]['name'] == 'rsomics-plink' and locals_[0]['publish'] == [],
              'private owner/path-dependency boundary differs')
        check([row['name'] for row in locals_[0]['targets'] if 'bin' in row['kind']] == ['rsomics-plink'],
              'one product binary boundary differs')
        pinned = {'rsomics-common': '0.12.3', 'rsomics-help': '0.4.0'}
        check(all(row['name'] in {'rsomics-plink', *pinned} for row in data['packages']
                  if row['name'].startswith('rsomics-')), 'unexpected rsomics dependency')
        for name, version in pinned.items():
            check([row['version'] for row in data['packages'] if row['name'] == name] == [version],
                  'pinned foundation differs: ' + name)


def test_command(mode):
    check(mode in ('debug', 'release'), 'explicit debug/release mode required')
    return ['cargo', 'test', '--locked', *(['--release'] if mode == 'release' else []),
            '--', '--include-ignored', '--nocapture', '--test-threads=1']


def report_paths(selector='green-1'):
    reports, _ = graph_population(selector)
    rows = []
    for workers in (1, 4):
        for lane in LANES:
            for stage in ('F', 'P'):
                for extension in ('acount', 'gcount', 'scount', 'smiss', 'vmiss', 'het', 'hwe-log.hardy') + (() if lane == 'rich' else ('hardy',)):
                    rows.append(f'{workers}/{lane}.{stage}.qc.{extension}')
            rows += [f'{workers}/{lane}.F.{suffix}' for suffix in ('ld.vcor', 'prune.prune.in', 'prune.prune.out')]
        rows += [f'{workers}/tail.{suffix}' for suffix in ('hardy', 'hwe-log.hardy')]
        if reports == 116:
            rows += [f'{workers}/rounding.qc.{suffix}' for suffix in ('hardy', 'hwe-log.hardy')]
    return rows


def file_identity(path):
    path = Path(path); _regular(path)
    return dict(bytes=path.stat().st_size, sha256=_hash_file(path))


def bind_graph(graph, selector='green-1'):
    reports_per_worker, _ = graph_population(selector)
    graph = Path(graph).resolve(strict=True)
    record = load(graph / 'native-graph.json')
    check(record['status'] == 'complete_compact_native_graph_passed' and
          record['principal_report_files'] == reports_per_worker and record['raw_cells_checked'] == 74878,
          'whole native graph is not complete at the frozen compact profile')
    keys = {(worker, lane, stage) for worker in (1, 4) for lane in LANES for stage in ('I', 'F', 'P')}
    observed = set(); generations = []
    for entry in record['generations']:
        key = entry['workers'], entry['lane'], entry['stage']
        check(key in keys and key not in observed, 'native generation membership differs')
        observed.add(key)
        workers, lane, stage = key
        relative = f'{workers}/{lane}.{stage}'
        check(Path(entry['prefix']).resolve() == graph / relative, 'native generation path is outside its bound graph')
        check(tuple(entry['shape']) == SHAPES[lane][('I', 'F', 'P').index(stage)], 'native generation dimensions differ')
        files = {relative + '.' + extension: file_identity(graph / (relative + '.' + extension))
                 for extension in ('pgen', 'pvar', 'psam', 'rsomics.json', 'raw.json')}
        generations.append(dict(workers=workers, lane=lane, stage=stage,
            prefix=relative, shape=entry['shape'], raw=relative + '.raw.json', files=files))
    check(observed == keys, 'native generation is absent')
    reports = {name: file_identity(graph / name) for name in report_paths(selector)}
    portable_names([*reports, *(name for entry in generations for name in entry['files'])])
    return dict(schema='rsomics.plink.portable-generations.v1', graph_sha256=_hash_file(graph / 'native-graph.json'),
                generations=generations, reports=reports, raw_cells_checked=74878,
                principal_report_files_per_worker=reports_per_worker, original_absolute_graph_preserved=True)


def worker_provenance(graph, selector='green-1'):
    _, expected = graph_population(selector)
    rows = {}
    for workers in (1, 4):
        for call in sorted((Path(graph) / str(workers) / 'calls').iterdir()):
            if not call.name.endswith(('.stats', '.pairs', '.prune')):
                continue
            model = load(call / 'stdout.txt')['result']
            field = 'effective_worker_limit' if call.name.endswith('.stats') else 'effective_workers'
            check(model['requested_workers'] == workers and model['writer_workers'] == 1 and
                  0 <= model[field] <= workers, 'native worker provenance differs')
            rows[f'{workers}/{call.name}'] = dict(requested=workers, compute=model[field],
                measure='batch parallel capacity' if field == 'effective_worker_limit' else 'observed distinct part-worker ids',
                writer=1)
    check(len(rows) == expected, 'native QC/LD worker report membership differs')
    return rows


def tests(root, mode):
    hosted()
    root = Path(root); scratch = root / ('test-' + mode + '-scratch'); scratch.mkdir()
    before = dict(os.environ)
    try:
        os.environ.update(TMPDIR=str(scratch), PYTHONDONTWRITEBYTECODE='1')
        logs = capture(root, 'test-' + mode, test_command(mode), root / 'source')
        text = (logs / 'stdout.txt').read_text() + (logs / 'stderr.txt').read_text()
        populations = [int(value) for value in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;', text)]
        check(sorted(populations) == [0, 0, 1, 4], 'complete4-library+1-workflow test population absent')
        graphs = [path for path in scratch.glob('plink-whole-native-*') if path.is_dir()]
        check(len(graphs) == 1, 'one complete kept native workflow graph required per mode')
        selector = load(root / 'selection.json')['snapshot']
        graph = graphs[0]; binding = bind_graph(graph, selector)
        binary = Path(os.environ['CARGO_TARGET_DIR']) / mode / 'rsomics-plink'
        _regular(binary)
        check(load(graph / 'identity.json')['binary_sha256'] == _hash_file(binary), 'whole graph binary differs')
        shutil.copy2(binary, root / ('binary-' + mode))
        save(root / ('binary-' + mode + '.json'), dict(sha256=_hash_file(binary),
            compiler_host=os.environ['EXPECTED_HOST'], selection_sha256=_hash_file(root / 'selection.json')))
        save(root / ('generations-' + mode + '.json'), dict(graph=graph.relative_to(root).as_posix(), **binding))
        save(root / ('workers-' + mode + '.json'), worker_provenance(graph, selector))
        records = {}
        for path in sorted((Path(os.environ['CARGO_TARGET_DIR']) / mode / 'build').glob('rsomics-plink-*/out/compiler.txt')):
            target = root / 'cpp-build' / mode / path.parent.parent.name / path.name
            target.parent.mkdir(parents=True, exist_ok=True); shutil.copy2(path, target)
            records[target.relative_to(root).as_posix()] = file_identity(target)
        check(records, 'actual native C++ compiler/build provenance absent')
        save(root / ('cpp-' + mode + '.json'), records)
        args = ['readelf', '--file-header', '--program-headers', '--dynamic', '--version-info', binary]
        if platform.system() == 'Darwin':
            capture(root, 'linkage-' + mode, ['otool', '-L', binary])
            args = ['otool', '-l', binary]
        capture(root, 'abi-' + mode, args)
    finally:
        os.environ.clear(); os.environ.update(before)
        save(root / ('test-' + mode + '-scratch-files.json'), tree_manifest(scratch))


def finalize(root):
    root = Path(root); failures = []
    try:
        check_tree(root / 'source', _manifest((root / 'files.sha256').read_bytes()))
        for mode in ('debug', 'release'):
            path = root / ('generations-' + mode + '.json')
            if path.is_file():
                binding = load(path); graph = root / binding['graph']
                check(bind_graph(graph, load(root / 'selection.json')['snapshot']) ==
                      {key: value for key, value in binding.items() if key != 'graph'},
                      'native generation/report identity changed: ' + mode)
    except (ValueError, OSError) as error:
        failures.append(str(error))
    save(root / 'integrity-after.json', dict(failures=failures, performance_claim=False, release=False))
    check(not failures, '; '.join(failures))


def package(root, archive_path):
    root, archive_path = Path(root).resolve(strict=True), Path(archive_path).absolute()
    check(not archive_path.is_relative_to(root) and not archive_path.exists() and not archive_path.is_symlink(),
          'new archive outside evidence required')
    files = {}
    for path in sorted(root.rglob('*')):
        if path.is_dir() and not path.is_symlink():
            continue
        name = path.relative_to(root).as_posix()
        if name == 'artifact-manifest.json':
            continue
        files[name] = file_identity(path)
    portable_names(files)
    save(root / 'artifact-manifest.json', dict(schema=SCHEMA, files=files))
    with tarfile.open(archive_path, 'w', format=tarfile.USTAR_FORMAT) as archive:
        for name in sorted([*files, 'artifact-manifest.json']):
            path = root / name; item = tarfile.TarInfo(root.name + '/' + name)
            item.size = path.stat().st_size
            item.mode = 0o755 if path.stat().st_mode & stat.S_IXUSR else 0o644
            with path.open('rb') as stream:
                archive.addfile(item, stream)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['storage', 'prepare', 'metadata', 'tests', 'finalize', 'package'])
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--world', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--snapshot')
    parser.add_argument('--mode', choices=['debug', 'release'])
    parser.add_argument('--archive', type=Path)
    args = parser.parse_args()
    hosted(); runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    for path in [args.output, *([args.archive] if args.action == 'package' else [])]:
        check(path is not None and path.resolve().is_relative_to(runner) and path.resolve() != runner and
              path.parent.resolve(strict=True).stat().st_dev == runner.stat().st_dev,
              'argument escapes checked runner device')
    if args.action == 'storage': save(args.output, hosted())
    elif args.action == 'prepare': prepare(args.world, args.output, args.snapshot)
    elif args.action == 'metadata': metadata(args.output)
    elif args.action == 'tests': tests(args.output, args.mode)
    elif args.action == 'finalize': finalize(args.output)
    elif args.action == 'package': package(args.output, args.archive)


if __name__ == '__main__':
    main()
