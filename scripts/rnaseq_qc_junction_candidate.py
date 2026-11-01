"""Qualify the additive private junction candidate against adopted captured data."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import struct
import tarfile

from infercnv_source_snapshot import _hash_file, _name, _no_symlink_path, _regular
from infercnv_trial_process import TrialProcessError, run_child


ASSET = 'assets/rnaseq-qc-junction-candidate-20261101.json'
ASSET_SHA = '99bc21a9f8d8dc21da6b4e9096b1be6e0c06472f5b67d0e20db4375dcaed27fc'
RECEIPT_SHA = '8ea60c35c952a600fa526ebfd3e020e142b01706f50a87a9fbc8818ce9ce5b2b'
OWNER = 'a618e6ab293e2fb7a113d949197e1bb43ecc7733'
TREE = 'e81e89d6e6ae6a8ae7a83a6a6d82cdc9a2aa20bb'
CASES = ('informative-single', 'informative-paired', 'case-alias-only', 'no-N', 'all-short')
LABELS = dict(total_events='Total splicing  Events', known_events='Known Splicing Events',
    partial_novel_events='Partial Novel Splicing Events', novel_events='Novel Splicing Events',
    filtered_events='Filtered Splicing Events', total_junctions='Total splicing  Junctions',
    known_junctions='Known Splicing Junctions', partial_novel_junctions='Partial Novel Splicing Junctions',
    novel_junctions='Novel Splicing Junctions')


def check(condition, message):
    if not condition:
        raise ValueError(message)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        check(key not in result, f'duplicate JSON field: {key}')
        result[key] = value
    return result


def load(path):
    return json.loads(Path(path).read_text(), object_pairs_hook=unique_object)


def save(path, data):
    with Path(path).open('x') as target:
        target.write(json.dumps(data, indent=2, sort_keys=True) + '\n')


def owned(root, name):
    path = Path(root) / _name(name)
    _no_symlink_path(path)
    _regular(path)
    return path


def identity(path):
    _regular(path)
    return dict(bytes=path.stat().st_size, sha256=_hash_file(path))


def authority(workspace):
    workspace = Path(workspace)
    asset = owned(workspace, ASSET)
    check(_hash_file(asset) == ASSET_SHA, 'junction candidate asset identity differs')
    data = load(asset)
    check(data['schema'] == 'rsomics.rnaseq-qc.junction-candidate.v1'
          and data['authority'] == 'junctions' and data['snapshot'] == 'green-1'
          and data['owner_commit'] == OWNER and data['owner_tree'] == TREE,
          'junction candidate identity differs')
    receipt = owned(workspace, data['source_receipt']['path'])
    check(identity(receipt) == {k: data['source_receipt'][k] for k in ('bytes', 'sha256')}
          and _hash_file(receipt) == RECEIPT_SHA, 'adopted junction receipt differs')
    source = load(receipt)
    check(source['status'] == 'accepted' and source['source_qualified'] is True
          and source['root_fresh_review_required'] is False and source['run_id'] == 41277337904
          and source['run_attempt'] == 1 and source['artifact_id'] == 13071489886
          and source['head_sha'] == 'b4cde25cde7a7d52d4b3947004929991b897a727',
          'junction source authority is not the adopted actual observation')
    snapshot = workspace / data['snapshot_path']
    for name, row in data['snapshot_files'].items():
        check(identity(owned(snapshot, name)) == row, f'junction snapshot identity differs: {name}')
    check(len(data['source_files']) == 66 and len(data['fixture_provenance']['captured_files']) == 45,
          'junction source or fixture population differs')
    with tarfile.open(snapshot / 'source.tar.gz', 'r:gz') as archive:
        members = archive.getmembers()
        check(len(members) == 66 and {m.name for m in members} == set(data['source_files']),
              'junction archive population differs')
        for member in members:
            check(member.isfile() and member.type == tarfile.REGTYPE
                  and member.mode == data['source_files'][member.name]['mode'],
                  f'junction Git mode/type differs: {member.name}')
    return data


def verify_source(source, data):
    source = Path(source)
    _no_symlink_path(source)
    actual = set()
    for path in source.rglob('*'):
        if not path.is_dir() or path.is_symlink():
            actual.add(path.relative_to(source).as_posix())
    check(actual == set(data['source_files']), 'junction source file population changed')
    for name, row in data['source_files'].items():
        path = owned(source, name)
        check(identity(path) == {k: row[k] for k in ('bytes', 'sha256')}
              and stat.S_IMODE(path.stat().st_mode) == row['mode'], f'junction source body/mode changed: {name}')
        body = path.read_bytes()
        check(hashlib.sha1(f'blob {len(body)}\0'.encode() + body).hexdigest() == row['git_blob'],
              f'junction Git body differs: {name}')


def prepare(workspace, source, evidence):
    workspace, source, evidence = map(Path, (workspace, source, evidence))
    data = authority(workspace)
    verify_source(source, data)
    target = evidence / 'junction-authority'
    target.mkdir()
    shutil.copyfile(workspace / ASSET, target / 'asset.json')
    shutil.copyfile(workspace / data['source_receipt']['path'], target / 'receipt.json')
    fixture_root = source / 'tests/fixtures/junctions'
    provenance = load(owned(fixture_root, 'provenance.json'))
    check(provenance == data['fixture_provenance'] and provenance['root_receipt_sha256'] == RECEIPT_SHA,
          'captured source provenance differs')
    for name, row in provenance['captured_files'].items():
        original = owned(fixture_root, name)
        check(identity(original) == {k: row[k] for k in ('bytes', 'sha256')}, f'captured source changed: {name}')
        copied = target / 'fixtures' / name
        copied.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(original, copied)
    shutil.copyfile(fixture_root / 'provenance.json', target / 'fixtures/provenance.json')
    for case in CASES:
        source_expected(fixture_root / case, case)
    result = dict(status='prepared', source_files=66, captured_files=45, owner_commit=OWNER,
                  owner_tree=TREE, source_receipt_sha256=RECEIPT_SHA, asset_sha256=ASSET_SHA,
                  scope='adopted captured data, not restoration or rerun of private upstream runtime')
    save(target / 'verification.json', result)
    return result


def source_expected(fixture, case):
    fixture = Path(fixture)
    observation = load(owned(fixture, 'observation.json'))
    process = load(owned(fixture, 'process.process.json'))
    expected_exit = 1 if case == 'all-short' else 0
    check(observation['raw_returncode'] == expected_exit and process['returncode'] == expected_exit
          and process['group_empty'] is True and process['timed_out'] is False
          and process['interrupted_signal'] is None, f'original source status differs: {case}')
    primary = owned(fixture, 'observed.junction.xls').read_text().splitlines()
    rows = []
    if primary:
        check(primary[0] == 'chrom\tintron_st(0-based)\tintron_end(1-based)\tread_count\tannotation',
              'actual source primary header differs')
        for line in primary[1:]:
            chrom, start, end, count, annotation = line.split()
            rows.append(dict(chrom=chrom, start=int(start), end=int(end), read_count=int(count), annotation=annotation))
    emitted = {}
    stderr = owned(fixture, 'process.stderr').read_text()
    for name, label in LABELS.items():
        values = re.findall(r'^' + re.escape(label) + r':\s*(\d+)\s*$', stderr, re.MULTILINE)
        check(len(values) <= 1, f'duplicate actual source counter: {label}')
        if values:
            emitted[name] = int(values[0])
    totals = re.findall(r'^total = (\d+)$', owned(fixture, 'process.stdout').read_text(), re.MULTILINE)
    check(len(totals) == 1, 'actual source total is missing/duplicate')
    counts = dict.fromkeys(LABELS, 0)
    counts.update(emitted)
    check('total_events' not in emitted or emitted['total_events'] == int(totals[0]), 'actual source stdout/stderr totals differ')
    counts['total_events'] = int(totals[0])
    if case == 'no-N':
        check(not primary and not emitted and counts['total_events'] == 0, 'actual no-N anomaly differs')
    elif case == 'all-short':
        check(len(primary) == 1 and set(emitted) == set(list(LABELS)[:5])
              and counts['total_events'] == counts['filtered_events'] == 1, 'actual all-short anomaly differs')
    else:
        check(set(emitted) == set(LABELS) and rows == observation['rows'], 'actual source detail/counters incomplete')
        check(counts['total_junctions'] == len(rows) and sum(row['read_count'] for row in rows)
              == counts['total_events'] - counts['filtered_events'], 'actual source event/distinct identity differs')
    return dict(rows=rows, counts=counts, source_emitted_counters=emitted,
                source_exit=expected_exit, intentional_native_zero_difference=case in ('no-N', 'all-short'))


def verify_output(directory, expected, profile, operation, workers, records):
    directory = Path(directory)
    report = load(owned(directory, 'report.json'))
    metrics = ['mapping', 'junctions'] if operation == 'report' else ['junctions']
    check(report['schema_version'] == '1.0' and report['profile'] == profile
          and report['sample'] == 'input' and report['metrics'] == metrics
          and report['junctions'] == {k: expected[k] for k in ('rows', 'counts')},
          'native junction rows/order/support/classes/counters differ')
    detail = owned(directory, 'junctions.tsv').read_text().splitlines()
    check(detail[0] == 'chrom\tintron_start\tintron_end\tread_count\tannotation', 'native detail header differs')
    rows = []
    for line in detail[1:]:
        chrom, start, end, count, annotation = line.split('\t')
        rows.append(dict(chrom=chrom, start=int(start), end=int(end), read_count=int(count), annotation=annotation))
    check(rows == expected['rows'], 'native full TSV rows differ')
    summary = owned(directory, 'junctions-summary.tsv').read_text().splitlines()
    check(summary[0] == 'metric\tvalue' and len(summary) == 10, 'native summary population differs')
    counter_rows = [line.split('\t') for line in summary[1:]]
    check(len({key for key, _ in counter_rows}) == 9
          and {key: int(value) for key, value in counter_rows} == expected['counts'], 'native TSV counters differ')
    manifest = load(owned(directory, 'manifest.json'))
    policy = manifest['policy']['junctions']
    check(manifest['status'] == 'complete' and manifest['profile'] == profile
          and manifest['selected_metrics'] == metrics and 'junctions' not in manifest['omitted_metrics']
          and manifest['bam_traversals'] == 1 and manifest['original_records'] == records
          and manifest['inflate_workers'] == workers and policy['excluded_flags'] == 0x704
          and policy['minimum_mapq'] == 30 and policy['minimum_intron'] == 50
          and policy['omitted_outputs'] == ['R', 'PDF', 'BED12', 'Interact', 'junction-saturation'],
          'native manifest junction policy/provenance differs')
    names = {'report.json', 'junctions.tsv', 'junctions-summary.tsv'}
    if operation == 'report':
        names.add('mapping.tsv')
    check(len(manifest['outputs']) == len(names) and {r['path'] for r in manifest['outputs']} == names,
          'native transactional output set differs')
    check({p.name for p in directory.iterdir()} == names | {'manifest.json'}, 'unexpected native output artifact')
    for row in manifest['outputs']:
        check(identity(owned(directory, row['path'])) == {k: row[k] for k in ('bytes', 'sha256')},
              'native output checksum differs')
    return _hash_file(directory / 'report.json')


def binary_identity(binary, target):
    body = Path(binary).read_bytes()
    if target.startswith('linux-'):
        machine = 62 if target == 'linux-x86_64' else 183
        check(body[:6] == b'\x7fELF\x02\x01' and struct.unpack_from('<H', body, 18)[0] == machine,
              'actual native ELF architecture differs')
    else:
        cpu = 0x1000007 if target == 'macos-x86_64' else 0x100000c
        check(body[:4] == b'\xcf\xfa\xed\xfe' and struct.unpack_from('<I', body, 4)[0] == cpu,
              'actual native Mach-O architecture differs')
    return dict(identity(Path(binary)), target=target)


def compare(workspace, source, evidence, binary, target):
    data = authority(workspace)
    verify_source(source, data)
    evidence, source, binary = map(Path, (evidence, source, binary))
    initial = binary_identity(binary, target)
    output = evidence / 'junction-comparison'
    output.mkdir()
    results, baseline = [], {}
    failure = None
    try:
        for case in CASES:
            fixture = source / 'tests/fixtures/junctions' / case
            actual = source_expected(fixture, case)
            profiles = ('rseqc-5.0.5', 'canonical') if case in ('informative-single', 'case-alias-only', 'no-N', 'all-short') else ('rseqc-5.0.5',)
            for profile in profiles:
                expected = actual
                if profile == 'canonical' and case == 'informative-single':
                    coordinates = [('chr1',120,170,4,'annotated'), ('chr1',120,270,1,'annotated'),
                        ('chr1',120,171,1,'partial_novel'), ('chr1',190,240,1,'complete_novel'),
                        ('chr1',320,370,1,'complete_novel'), ('chr1',530,580,1,'complete_novel'),
                        ('CHR1',120,170,1,'complete_novel')]
                    expected = dict(rows=[dict(zip(('chrom','start','end','read_count','annotation'), row)) for row in coordinates],
                        counts=dict(total_events=11,known_events=5,partial_novel_events=1,novel_events=4,
                            filtered_events=1,total_junctions=7,known_junctions=2,partial_novel_junctions=1,novel_junctions=4))
                operations = ('junctions', 'report') if profile == 'rseqc-5.0.5' and case.startswith('informative') else ('junctions',)
                for operation in operations:
                    for workers in (1, 4):
                        name = f'{case}-{profile}-{operation}-{workers}'
                        directory = output / name
                        argv = [str(binary), operation, '--bam', str(fixture / 'input.bam'), '--genes', str(fixture / 'genes.bed12'),
                                '--out', str(directory), '--profile', profile, '--threads', str(workers), '--mapq', '30', '--min-intron', '50']
                        if operation == 'report':
                            argv += ['--metrics', 'mapping,junctions']
                        prefix = output / (name + '.trial')
                        try:
                            run_child(argv, prefix, timeout=300)
                        except TrialProcessError:
                            if case != 'case-alias-only':
                                raise
                        process = load(Path(str(prefix) + '.process.json'))
                        row = dict(case=case, profile=profile, operation=operation, workers=workers,
                                   source_exit=actual['source_exit'], returncode=process['returncode'])
                        if case == 'case-alias-only':
                            check(process['status'] == 'nonzero' and process['returncode'] > 0 and process['group_empty']
                                  and not process['timed_out'] and process['interrupted_signal'] is None
                                  and b'absent from the BAM header' in Path(str(prefix) + '.stderr').read_bytes()
                                  and not directory.exists(), 'native alias failure is not the intended strict-header rejection')
                            row['comparison'] = 'intentional strict exact-header rejection'
                        else:
                            records = 16 if case == 'informative-single' else 30 if case == 'informative-paired' else 1
                            digest = verify_output(directory, expected, profile, operation, workers, records)
                            key = case, profile, operation
                            check(key not in baseline or baseline[key] == digest, 'native report bytes differ across workers')
                            baseline[key] = digest
                            row.update(report_sha256=digest, counts=expected['counts'],
                                comparison='actual source equality' if profile == 'rseqc-5.0.5' and case.startswith('informative')
                                else 'declared canonical/complete-zero native difference')
                        results.append(row)
        check(len(results) == 22, 'native junction invocation population differs')
        check(binary_identity(binary, target) == initial, 'native executable changed during comparison')
        verify_source(source, data)
    except BaseException as error:
        failure = error
    summary = dict(status='failed' if failure else 'complete', binary=initial, results=results,
        source_receipt_sha256=RECEIPT_SHA, error=None if failure is None else f'{type(failure).__name__}: {failure}',
        scope='bounded captured junction correctness and native integration; not upstream rerun, performance or release acceptance')
    save(output / 'summary.json', summary)
    if failure:
        raise failure
    return summary


def verify_test_logs(evidence, data):
    expected = data['tests']['unit'] + data['tests']['workflow']
    check(len(data['tests']['unit']) == 4 and len(data['tests']['workflow']) == 8, 'expected test populations differ')
    for profile in ('debug', 'release'):
        text = owned(Path(evidence), f'test-{profile}.log').read_text()
        observed = re.findall(r'^test (\S+) \.\.\. (\S+)\s*$', text, re.MULTILINE)
        check(len(observed) == 12 and sorted(name for name, result in observed if result == 'ok') == sorted(expected),
              f'{profile} exact test names/population/status differ')
        check(sorted(map(int, re.findall(r'^running (\d+) tests?$', text, re.MULTILINE))) == [0, 4, 8],
              f'{profile} exact native test binary populations differ')
        results = re.findall(r'^test result: ok\. (\d+) passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;', text, re.MULTILINE)
        check(sorted(map(int, results)) == [0, 4, 8], f'{profile} full test result populations differ')


def finalize(workspace, source, evidence):
    evidence = Path(evidence)
    evidence.mkdir(parents=True, exist_ok=True)
    errors = []
    try:
        data = authority(workspace)
        verify_source(source, data)
        verify_test_logs(evidence, data)
        check(load(owned(evidence, 'junction-authority/verification.json'))['status'] == 'prepared', 'source authority preparation failed')
        comparison = load(owned(evidence, 'junction-comparison/summary.json'))
        check(comparison['status'] == 'complete' and len(comparison['results']) == 22, 'junction native comparisons incomplete')
        gates = json.loads(os.environ.get('JUNCTION_GATES', '{}'))
        check(all(gates.get(name, {}).get('outcome') == 'success' for name in ('snapshot', 'metadata', 'format', 'debug', 'release', 'comparison')),
              'one or more native candidate workflow gates failed or are absent')
    except (OSError, ValueError, KeyError, TypeError, tarfile.TarError) as error:
        errors.append(f'{type(error).__name__}: {error}')
    retained = {}
    for path in sorted(evidence.rglob('*')):
        if path.is_symlink():
            errors.append(f'symlinked evidence: {path.relative_to(evidence)}')
        elif path.is_file() and path.name != 'junction-finalization.log':
            retained[path.relative_to(evidence).as_posix()] = identity(path)
    result = dict(status='failed' if errors else 'complete', errors=errors, retained_files=retained,
                  owner_commit=OWNER, source_receipt_sha256=RECEIPT_SHA,
                  deferred_live_files=['junction-finalization.log'],
                  scope='one native platform correctness; four-platform root adoption and performance/publication remain separate')
    save(evidence / 'junction-finalization.json', result)
    check(not errors, '; '.join(errors))
    return result


def seal(evidence):
    evidence = Path(evidence)
    files = {}
    for path in sorted(evidence.rglob('*')):
        check(not path.is_symlink(), f'symlinked evidence: {path}')
        if path.is_file():
            files[path.relative_to(evidence).as_posix()] = identity(path)
    check('junction-finalization.json' in files, 'junction finalization record is absent')
    result = dict(files=files, finalization_status=load(evidence / 'junction-finalization.json')['status'],
                  scope='complete stable available evidence after finalizer log closes; not success inferred from file retention')
    save(evidence / 'junction-integrity.json', result)
    return dict(retained_files=len(files), finalization_status=result['finalization_status'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('operation', choices=('prepare', 'compare', 'finalize', 'seal'))
    for name in ('workspace', 'source', 'evidence'):
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--binary', type=Path)
    parser.add_argument('--target', choices=('linux-x86_64','linux-aarch64','macos-x86_64','macos-aarch64'))
    args = parser.parse_args()
    try:
        if args.operation == 'prepare':
            result = prepare(args.workspace, args.source, args.evidence)
        elif args.operation == 'compare':
            check(args.binary is not None and args.target is not None, 'compare requires binary and target')
            result = compare(args.workspace, args.source, args.evidence, args.binary, args.target)
        elif args.operation == 'finalize':
            result = finalize(args.workspace, args.source, args.evidence)
        else:
            result = seal(args.evidence)
        print(json.dumps({k: v for k, v in result.items() if k not in ('retained_files', 'results')}, sort_keys=True))
    except (OSError, ValueError, KeyError, TypeError, tarfile.TarError, TrialProcessError) as error:
        parser.exit(1, f'junction candidate {args.operation} failed: {error}\n')


if __name__ == '__main__':
    main()
