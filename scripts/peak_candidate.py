"""Private four-native qualification of a frozen real peak product."""

import argparse
import copy
import csv
import gzip
import json
import os
from pathlib import Path
import platform
import re
import shutil
import stat
import subprocess
import tarfile

from infercnv_source_snapshot import (_hash_file, _manifest, _regular, _verified_members,
                                     read_snapshot_manifest, verify_and_extract)
from signal_candidate import capture, check_tree, portable_names, storage, tree_manifest


REPOSITORY = 'omics-rust/rsomics-world'
ASSET = 'peak-native-fixtures-20261101'
SNAPSHOTS = '.autopilot/snapshots/peak-native-2026-11-01'
SOURCE_RUN = 41209740891
SOURCE_REPORT = '83cbf36dbe59c17f87745a02c43c5f2f045c3d29ba9627b22ee31ccce3e348ed'
SOURCE_MANIFEST = '5f7c351e3b7e4e7d0e2ec8837124862a590dc01e1af8010d972c4249b7be305f'
BASE_BYTES = 128876798
FIXTURE_BYTES = BASE_BYTES + 8477
ADDON_RUN = 41228249024
ADDON_PROOF = 'a0cf912c415f8a9ce2cc4993f711f5c58d96aa9a76faeed5fa8248d0fc3a3984'
CONTROLLERS = ('scripts/peak_candidate.py', 'scripts/peak_source_on_native.py',
    'scripts/peak_source_on_native.R', 'scripts/test_peak_candidate_workflow.py',
    'scripts/infercnv_source_snapshot.py', 'scripts/signal_candidate.py',
    'scripts/peak_annotation_count_view.py', 'scripts/peak_annotation_count_view.R',
    'scripts/peak_first_workflow_oracle.py', 'scripts/peak_first_workflow_oracle.R',
    'scripts/signal_first_workflow_oracle.py', '.github/workflows/peak-candidate.yml',
    'assets/' + ASSET + '.json')


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


def hosted():
    check(os.environ.get('GITHUB_ACTIONS') == 'true' and
          os.environ.get('GITHUB_REPOSITORY') == REPOSITORY, 'native runtime actions require the hosted repository workflow')
    return storage(require_space=2**30)


def base_fixture_names():
    names = {f'inputs/alignments/{sample}.{kind}.bam'
             for sample in ('T1', 'T2', 'C1', 'C2') for kind in ('se', 'pe')}
    names.update(('inputs/annotation.gtf', 'inputs/annotation.gff3',
                  'witness/rows/workflow.bed', 'witness/calls/count-unit-summary/observation.json'))
    for number in range(1, 6):
        names.update(f'witness/results/M{number}/M{number}{suffix}' for suffix in
            ('_peaks.narrowPeak', '_summits.bed', '_peaks.xls', '_treat_pileup.bdg', '_control_lambda.bdg'))
    names.update(f'witness/calls/{name}/annotation.tsv'
                 for name in ('A-gtf-default', 'A-gff3-default', 'A-gtf-300'))
    names.add('witness/calls/A-gtf-default/transcripts.tsv')
    names.update(f'witness/calls/{name}/stdout.txt'
                 for name in ('C-default', 'C-proper', 'C-flags', 'C-split', 'track-target', 'track-control'))
    for label in ('control-norm', 'control-non', 'numeric'):
        for mode in ('stringent', 'relaxed'):
            case = 'S-' + label + '-' + mode
            names.update((f'witness/calls/{case}/result.{mode}.bed',
                          f'witness/calls/D-{case}/retained-r/thresholds.tsv'))
    return names


def addon_fixture_names():
    return {'inputs/annotation.bed', 'inputs/count.bed',
        *{f'calls/{name}/annotation.tsv' for name in ('A-gtf-narrow', 'A-gff3-narrow', 'A-gtf-default')},
        *{f'calls/{name}/stdout.txt' for name in ('C-default', 'C-q20', 'C-proper', 'C-include', 'C-split')}}


def fixture_names():
    return base_fixture_names() | {'addon/' + name for name in addon_fixture_names()}


def external_pack_paths(paths):
    check(platform.system() == 'Darwin', 'local fixture packing requires the external Mac guard')
    volumes = [Path('/Volumes/KIOXIA'), Path("/Volumes/Zane's HDD")]
    import plistlib
    devices = {}
    for volume in volumes:
        body = subprocess.run(['/usr/sbin/diskutil', 'info', '-plist', str(volume)],
                              check=True, capture_output=True).stdout
        info = plistlib.loads(body)
        check(info.get('Internal') is False and info.get('MountPoint') == str(volume),
              'fixture path volume is not actually external: ' + str(volume))
        devices[volume] = volume.stat().st_dev
    for raw in paths:
        path = Path(raw).resolve(strict=True)
        check(any(path.is_relative_to(volume) and path.stat().st_dev == device
                  for volume, device in devices.items()), 'fixture packing path escapes external devices')


def pack_fixtures(sealed, world, supplement):
    check(supplement is not None, 'the accepted source supplement directory is required')
    sealed, world = Path(sealed), Path(world)
    supplement = Path(supplement)
    external_pack_paths((sealed, world / 'assets', supplement))
    check(_hash_file(sealed / 'REPORT.md') == SOURCE_REPORT, 'accepted source report differs')
    source_manifest = sealed / 'selected/artifact-manifest.json'
    check(_hash_file(source_manifest) == SOURCE_MANIFEST, 'accepted original manifest differs')
    verified = load(sealed / 'retention-verification.json')
    check(verified['run_id'] == SOURCE_RUN and verified['run_attempt'] == 1 and
          verified['failures'] == [] and verified['verified_entries'] == 39404, 'original was not completely verified')
    original = load(source_manifest)['files']
    files = {}
    sources = {}
    for name in sorted(base_fixture_names()):
        path = sealed / 'selected' / name
        _regular(path)
        identity = original[name]
        check(set(identity) == {'size', 'sha256'} and path.stat().st_size == identity['size'] and
              _hash_file(path) == identity['sha256'], 'fixture differs from original: ' + name)
        files[name] = dict(bytes=identity['size'], sha256=identity['sha256'],
                           original_member='peak-first-workflow-oracle/' + name, origin=SOURCE_RUN)
        sources[name] = path
    check(sum(row['bytes'] for row in files.values()) == BASE_BYTES, 'base fixture bytes changed')
    proof_path = supplement / 'selected/projection-proof.json'
    check(_hash_file(proof_path) == ADDON_PROOF, 'accepted source supplement proof differs')
    proof = load(proof_path)
    check(proof['manifest']['every_identity_verified'] and proof['manifest']['entries'] == 21839 and
          proof['manifest']['sha256'] == '427f931fd47f962d03f3e08487e6488a909344826306a393ca66dc723e096241' and
          proof['original_zip']['sha256'] == 'c754936a89b64bc499e72c0b0e0b0fe8e8e8d55571976b99a35804ff5a2a702d',
          'supplement full-original identity differs')
    projected = {row['path']: row for row in proof['projected']}
    for relative in sorted(addon_fixture_names()):
        path = supplement / 'selected/text' / relative
        _regular(path)
        row = projected[relative]
        check(path.stat().st_size == row['bytes'] and _hash_file(path) == row['sha256'],
              'supplement input/answer differs: ' + relative)
        name = 'addon/' + relative
        files[name] = dict(bytes=row['bytes'], sha256=row['sha256'],
                           original_member='capture/' + relative, origin=ADDON_RUN)
        sources[name] = path
    check(set(files) == fixture_names() and sum(row['bytes'] for row in files.values()) == FIXTURE_BYTES,
          'complete composite fixture closure differs')
    files = dict(sorted(files.items()))
    portable_names(files)
    archive_path = world / 'assets' / (ASSET + '.tar.gz')
    record_path = world / 'assets' / (ASSET + '.json')
    check(not archive_path.exists() and not record_path.exists(), 'fixture derivative already exists')
    with archive_path.open('xb') as raw, gzip.GzipFile(fileobj=raw, mode='wb', filename='', mtime=0) as compressed:
        with tarfile.open(fileobj=compressed, mode='w', format=tarfile.USTAR_FORMAT) as archive:
            for name, row in files.items():
                entry = tarfile.TarInfo(name); entry.size, entry.mode = row['bytes'], 0o644
                with sources[name].open('rb') as stream:
                    archive.addfile(entry, stream)
    check(archive_path.stat().st_size <= 50 * 2**20, 'portable fixture exceeds50MiB; preserved, not truncated')
    save(record_path, dict(schema='rsomics.peak.portable-fixtures.v1', files=files,
        archive_sha256=_hash_file(archive_path), archive_bytes=archive_path.stat().st_size,
        uncompressed_bytes=FIXTURE_BYTES, accepted_sources=dict(baseline=dict(run_id=SOURCE_RUN, run_attempt=1,
            head_sha='b92eb0404aad1a0222ad580c2a81c5431ff42957', artifact_id=13046468658,
            zip_sha256='c6a8a06c14788c557c6165f524ce3524d367b58779636826ceef1abf238afa51',
            tar_sha256='0e19577fef849bfa54ff663d671028f29ca85186145b42951f14fb1442f60ef3',
            report_sha256=SOURCE_REPORT, manifest_sha256=SOURCE_MANIFEST,
            adoption='tracked peak-source-preflight-2026-10-31 state and peak product dossier'),
            addon=dict(run_id=ADDON_RUN, run_attempt=1, head_sha='7b14e338453cf4114747da6d121ff29e144bdedc',
                artifact_id=13052618072, zip_sha256=proof['original_zip']['sha256'],
                tar_sha256=proof['original_tar']['sha256'], manifest_sha256=proof['manifest']['sha256'],
                projection_proof_sha256=ADDON_PROOF,
                adoption='accepted targeted source supplement in tracked peak-source-preflight state')),
        scope='69 unmodified regular files from TWO actual source originals; no runtime/RDS/links; not native or release admission.'))


def extract_regular(archive_path, output, files):
    output = Path(output)
    check(not output.exists() and not output.is_symlink(), 'archive output already exists')
    portable_names(files)
    manifest = {name: row['sha256'] for name, row in files.items()}
    with tarfile.open(archive_path, 'r:gz') as archive:
        portable_names([member.name for member in archive.getmembers()])
        members = _verified_members(archive, manifest)
        check(all(member.size == files[member.name]['bytes'] for member in members), 'archive length differs')
        output.mkdir(parents=True)
        for member in members:
            target = output / member.name
            target.parent.mkdir(parents=True, exist_ok=True)
            with archive.extractfile(member) as source, target.open('xb') as stream:
                shutil.copyfileobj(source, stream, 2**20)
    check_tree(output, manifest)


def extract_fixtures(world, output):
    world = Path(world)
    archive = world / 'assets' / (ASSET + '.tar.gz')
    record = load(world / 'assets' / (ASSET + '.json'))
    check(record['schema'] == 'rsomics.peak.portable-fixtures.v1' and
          record['accepted_sources']['baseline']['run_id'] == SOURCE_RUN and
          record['accepted_sources']['baseline']['report_sha256'] == SOURCE_REPORT and
          record['accepted_sources']['baseline']['manifest_sha256'] == SOURCE_MANIFEST and
          record['accepted_sources']['addon']['run_id'] == ADDON_RUN and
          record['accepted_sources']['addon']['projection_proof_sha256'] == ADDON_PROOF and
          set(record['files']) == fixture_names() and
          sum(row['bytes'] for row in record['files'].values()) == FIXTURE_BYTES and
          all(row['origin'] == (ADDON_RUN if name.startswith('addon/') else SOURCE_RUN)
              for name, row in record['files'].items()),
          'portable fixture source/closure differs')
    check(archive.stat().st_size == record['archive_bytes'] <= 50 * 2**20 and
          _hash_file(archive) == record['archive_sha256'], 'fixture carrier differs')
    extract_regular(archive, output, record['files'])
    return record


def prepare_source(world, output, selector):
    check(isinstance(selector, str) and re.fullmatch('green-[1-9][0-9]*', selector), 'explicit green-N selector required')
    snapshot = Path(world) / SNAPSHOTS / selector
    manifest = read_snapshot_manifest(snapshot)
    check(set(path.name for path in snapshot.iterdir()) ==
          {'README.md', 'files.sha256', 'source.tar.gz', 'checksums.sha256'}, 'snapshot inventory differs')
    check({'Cargo.toml', 'Cargo.lock', 'src/lib.rs', 'src/cli.rs', 'src/call/mod.rs',
           'src/sparse/mod.rs', 'src/annotation/mod.rs', 'src/count/mod.rs',
           'tests/count_workflow.rs', 'tests/peak_workflow.rs'} <= set(manifest), 'snapshot omits a functioning operation/test')
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
    fixtures = extract_fixtures(world, output / 'fixtures')
    save(output / 'selection.json', dict(snapshot=selector, source_files=len(manifest),
        source_archive_sha256=_hash_file(output / 'source.tar.gz'),
        source_manifest_sha256=_hash_file(output / 'files.sha256'),
        lock_sha256=_hash_file(output / 'source/Cargo.lock'),
        fixture_archive_sha256=fixtures['archive_sha256'],
        fixture_manifest_sha256=_hash_file(world / 'assets' / (ASSET + '.json')),
        controller_commit=os.environ['CONTROLLER_COMMIT'], target=os.environ['TARGET'],
        admitted=False, release=False, performance_claim=False))
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
          'host is not the actual native target')
    check(not any(os.environ.get(name) for name in ('CARGO_BUILD_TARGET', 'RUSTC_WRAPPER',
          'RUSTC_WORKSPACE_WRAPPER', 'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS')), 'compiler/target overrides are forbidden')
    compiler = capture(root, 'compiler', ['rustc', '-Vv'])
    values = dict(line.split(': ', 1) for line in (compiler / 'stdout.txt').read_text().splitlines() if ': ' in line)
    check(values['release'] == '1.91.0' and values['host'] == os.environ['EXPECTED_HOST'], 'compiler is not pinned native Rust1.91')
    capture(root, 'cargo-version', ['cargo', '-V'])
    for label, args in [('metadata-preflight', ['--no-deps']), ('metadata-full', [])]:
        data = capture(root, label, ['cargo', 'metadata', '--locked', *args, '--format-version', '1'], source)
        data = load(data / 'stdout.txt')
        check(Path(data['target_directory']).resolve() == Path(os.environ['CARGO_TARGET_DIR']).resolve(strict=True),
              'Cargo target escapes the checked route')
        if label == 'metadata-preflight':
            hosted()
            continue
        locals_ = [row for row in data['packages'] if row['source'] is None]
        check(len(locals_) == 1 and locals_[0]['name'] == 'rsomics-peak' and locals_[0]['publish'] == [],
              'private owner/path-dependency boundary differs')
        check([row['name'] for row in locals_[0]['targets'] if 'bin' in row['kind']] == ['rsomics-peak'],
              'single binary boundary differs')
        pinned = {'rsomics-common': '0.12.3', 'rsomics-help': '0.4.0',
                  'rsomics-bamio': '0.8.4', 'rsomics-intervals': '0.4.0'}
        check(all(row['name'] in {'rsomics-peak', *pinned} for row in data['packages']
                  if row['name'].startswith('rsomics-')), 'unexpected rsomics dependency')
        for name, version in pinned.items():
            check([row['version'] for row in data['packages'] if row['name'] == name] == [version],
                  'pinned foundation differs: ' + name)


def fixture_environment(root, scratch):
    root = Path(root)
    return dict(os.environ, TMPDIR=str(scratch),
        RSOMICS_PEAK_FIXTURES=str(root / 'fixtures'), RSOMICS_PEAK_ORACLE=str(root / 'fixtures'),
        RSOMICS_PEAK_SOURCE_FIXTURE=str(root / 'fixtures'),
        RSOMICS_PEAK_SOURCE_CALLS=str(root / 'fixtures/witness/calls'))


def test_command(mode):
    check(mode in ('debug', 'release'), 'explicit debug/release mode required')
    return ['cargo', 'test', '--locked', *(['--release'] if mode == 'release' else []),
            '--', '--include-ignored', '--nocapture', '--test-threads=1']


def tests(root, mode):
    hosted()
    root = Path(root); scratch = root / ('test-' + mode + '-scratch'); scratch.mkdir()
    before = dict(os.environ)
    try:
        os.environ.update(fixture_environment(root, scratch))
        logs = capture(root, 'test-' + mode, test_command(mode), root / 'source')
        text = (logs / 'stdout.txt').read_text() + (logs / 'stderr.txt').read_text()
        populations = [int(number) for number in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;', text)]
        check(all(number in populations for number in (6, 3, 1)), 'complete6+3+1 native test population absent')
        binary = Path(os.environ['CARGO_TARGET_DIR']) / mode / 'rsomics-peak'
        _regular(binary)
        shutil.copy2(binary, root / ('binary-' + mode))
        save(root / ('binary-' + mode + '.json'), dict(sha256=_hash_file(binary),
            original=str(binary), compiler_host=os.environ['EXPECTED_HOST'],
            selection_sha256=_hash_file(root / 'selection.json')))
    finally:
        os.environ.clear(); os.environ.update(before)
        save(root / ('test-' + mode + '-scratch-files.json'), tree_manifest(scratch))


def native_rows(called, sparse, output):
    called, sparse, output = Path(called), Path(sparse), Path(output)
    output.mkdir()
    records = []
    for origin, path, width in [('call', called, 10), ('sparse', sparse, 6)]:
        for number, line in enumerate(path.read_text().splitlines()):
            fields = line.split('\t')
            check(len(fields) == width and int(fields[1]) < int(fields[2]), 'native peak row schema differs')
            records.append((origin, number, fields, [*fields[:3], f'{origin}-{number}', '0', '.']))
    check(records and records[0][0] == 'call' and any(row[0] == 'sparse' for row in records),
          'connected native call/sparse rows are empty')
    records.append(('duplicate', 0, records[0][2], records[0][3]))
    with (output / 'workflow.bed').open('x') as bed, (output / 'workflow-rows.tsv').open('x', newline='') as ledger:
        writer = csv.writer(ledger, delimiter='\t', lineterminator='\n')
        writer.writerow(['row_key', 'row_ordinal', 'source', 'source_row', 'display_id', 'original_fields_json'])
        for ordinal, (origin, number, original, fields) in enumerate(records):
            bed.write('\t'.join(fields) + '\n')
            writer.writerow([f'{origin}:{number}', ordinal, origin, number, fields[3], json.dumps(original)])
    save(output / 'derivation.json', dict(inputs={str(path): _hash_file(path) for path in (called, sparse)},
        rows=len(records), deduplicated=False, sparse_field6='max-region, not strand'))


def worker_plan(fixture, output, workers):
    fixture, output = Path(fixture), Path(output)
    bam = fixture / 'inputs/alignments'; call = output / 'call'; rows = output / 'rows/workflow.bed'
    commands = []
    def add(name, args):
        commands.append(dict(name=name, args=[*map(str, args), '--threads', str(workers)]))
    add('call', ['call', '--treatment', bam / 'T1.pe.bam', bam / 'T2.pe.bam',
        '--control', bam / 'C1.pe.bam', bam / 'C2.pe.bam', '--format', 'bampe',
        '--model', 'paired-end', '--genome-size', '20000000', '--duplicates', '1',
        '--qvalue', '0.05', '--name', 'native', '--output-directory', call, '--write-tracks'])
    for profile in ('seacr-1.3', 'corrected'):
        add('sparse-' + profile, ['call-sparse', '--target', call / 'native_treat_pileup.bdg',
            '--fraction', '0.1', '--normalization', 'non', '--mode', 'stringent', '--compatibility', profile,
            '--output', output / (profile + '.bed'), '--report', output / (profile + '.json')])
    add('annotate', ['annotate', '--peaks', rows, '--annotation', fixture / 'inputs/annotation.gff3',
        '--format', 'gff3', '--tss-upstream', '3000', '--tss-downstream', '3000', '--downstream', '3000',
        '--output', output / 'annotation.tsv', '--report', output / 'annotation.json'])
    add('count', ['count', '--peaks', rows, '--sample', 'T1=' + str(bam / 'T1.pe.bam'),
        'T2=' + str(bam / 'T2.pe.bam'), '--output', output / 'counts.tsv', '--report', output / 'counts.json'])
    return commands


def report_model(path, kind, workers):
    data = copy.deepcopy(load(path))
    if kind == 'call':
        check(data.pop('workers_requested') == workers and data.pop('workers_used') == min(workers, 2), 'caller worker provenance differs')
        for name in ('narrow_peak_path', 'summits_path', 'xls_path', 'treatment_track_path', 'control_track_path', 'report_path'):
            data.pop(name)
        data['config'].pop('output_directory')
    elif kind == 'sparse':
        check(data.pop('requested_threads') == workers and data.pop('reading_workers') == 1 and
              data.pop('merging_workers') == min(workers, 2), 'sparse worker provenance differs')
        for name in ('output', 'report'): data.pop(name)
    else:
        requested = data.pop('requested_threads'); effective = data.pop('effective_workers')
        expected = min(workers, 2) if kind == 'count' else min(workers, max(1, (data['rows'] + 1023) // 1024))
        check(requested == workers and effective == expected, kind + ' worker provenance differs')
        for name in ('output', 'report', 'scratch', 'peaks'): data.pop(name)
    return data


def workers(root):
    hosted()
    root = Path(root); binary = root / 'binary-release'
    for number in (1, 4):
        output = root / 'workers' / str(number); output.mkdir(parents=True)
        (output / 'call').mkdir()
        plan = worker_plan(root / 'fixtures', output, number)
        save(output / 'plan.json', dict(commands=plan, binary_sha256=_hash_file(binary),
            row_derivation='native call+corrected sparse; tagged original-field ordinal ledger; duplicate retained'))
        for index, command in enumerate(plan):
            if index == 3:
                native_rows(output / 'call/native_peaks.narrowPeak', output / 'corrected.bed', output / 'rows')
            capture(output / 'commands', command['name'], [binary, *command['args']])
        save(output / 'files.json', tree_manifest(output))
    first, second = [root / 'workers' / str(number) for number in (1, 4)]
    scientific = ['call/native' + suffix for suffix in ('_peaks.narrowPeak', '_summits.bed',
        '_peaks.xls', '_treat_pileup.bdg', '_control_lambda.bdg')]
    scientific += ['seacr-1.3.bed', 'corrected.bed', 'rows/workflow.bed', 'rows/workflow-rows.tsv', 'annotation.tsv', 'counts.tsv']
    for name in scientific:
        values = [path.joinpath(name).read_bytes() for path in (first, second)]
        if name.endswith('.xls'):
            values = [b'\n'.join(line for line in value.splitlines() if line and not line.startswith(b'#')) for value in values]
        check(values[0] == values[1], 'worker scientific output differs: ' + name)
    for name, kind in [('call/native_call.json', 'call'), ('seacr-1.3.json', 'sparse'),
                       ('corrected.json', 'sparse'), ('annotation.json', 'annotation'), ('counts.json', 'count')]:
        check(report_model(first / name, kind, 1) == report_model(second / name, kind, 4),
              'worker scientific/report state differs: ' + name)
    save(root / 'worker-equality.json', dict(passed=True, workers=[1, 4], scientific_files=scientific,
        binary_sha256=_hash_file(binary), source_manifest_sha256=_hash_file(root / 'files.sha256'),
        differences='XLS comments, output/input-row paths and verified requested/effective-worker provenance only',
        performance_claim=False, release=False))


def finalize(root):
    root = Path(root); failures = []
    try:
        check_tree(root / 'source', _manifest((root / 'files.sha256').read_bytes()))
        record = load(Path(os.environ['GITHUB_WORKSPACE']) / 'assets' / (ASSET + '.json'))
        check_tree(root / 'fixtures', {name: row['sha256'] for name, row in record['files'].items()})
    except (ValueError, OSError) as error:
        failures.append(str(error))
    save(root / 'integrity-after.json', dict(failures=failures, release=False, performance_claim=False))
    check(not failures, '; '.join(failures))


def retained_files(root):
    rows = {}
    for path in sorted(Path(root).rglob('*')):
        name = path.relative_to(root).as_posix()
        if name == 'artifact-manifest.json' or name.startswith('candidate/fixtures/'):
            continue
        if path.is_dir() and not path.is_symlink(): continue
        _regular(path)
        rows[name] = dict(bytes=path.stat().st_size, sha256=_hash_file(path))
    portable_names(rows)
    return rows


def package(root, archive_path):
    root, archive_path = Path(root).resolve(strict=True), Path(archive_path).absolute()
    check(not archive_path.is_relative_to(root) and not archive_path.exists() and not archive_path.is_symlink(),
          'new archive outside evidence required')
    files = retained_files(root)
    save(root / 'artifact-manifest.json', dict(schema='rsomics.peak.gate-artifact.v1', files=files,
        excluded_payloads=['candidate/fixtures (unchanged identities bound by selection/controller/integrity records)']))
    with tarfile.open(archive_path, 'w', format=tarfile.USTAR_FORMAT) as archive:
        for name in sorted([*files, 'artifact-manifest.json']):
            path = root / name
            info = tarfile.TarInfo(root.name + '/' + name)
            info.size = path.stat().st_size
            info.mode = 0o755 if path.stat().st_mode & stat.S_IXUSR else 0o644
            with path.open('rb') as stream:
                archive.addfile(info, stream)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['storage', 'prepare', 'metadata', 'tests', 'workers', 'finalize', 'package', 'pack-fixtures'])
    parser.add_argument('--output', type=Path)
    parser.add_argument('--world', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--snapshot')
    parser.add_argument('--mode', choices=['debug', 'release'])
    parser.add_argument('--sealed', type=Path)
    parser.add_argument('--supplement', type=Path)
    parser.add_argument('--archive', type=Path)
    args = parser.parse_args()
    if args.action == 'pack-fixtures':
        check(args.sealed is not None and args.supplement is not None and args.output is None,
              'pack-fixtures requires --sealed/--supplement/--world, not --output')
        pack_fixtures(args.sealed, args.world, args.supplement)
        return
    check(args.output is not None, '--output is required for this action')
    hosted()
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    for path in [args.output, *([args.archive] if args.action == 'package' else [])]:
        check(path is not None and path.resolve().is_relative_to(runner) and path.resolve() != runner and
              path.parent.resolve(strict=True).stat().st_dev == runner.stat().st_dev,
              'runtime output/archive argument escapes the checked runner device')
    if args.action == 'storage': save(args.output, hosted())
    elif args.action == 'prepare': prepare(args.world, args.output, args.snapshot)
    elif args.action == 'metadata': metadata(args.output)
    elif args.action == 'tests': tests(args.output, args.mode)
    elif args.action == 'workers': workers(args.output)
    elif args.action == 'finalize': finalize(args.output)
    elif args.action == 'package': package(args.output, args.archive)


if __name__ == '__main__':
    main()
