"""Pinned hosted upstream consumers of actual native peak workflow bytes."""

import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import shutil
import tarfile

from infercnv_source_snapshot import _hash_file, _manifest, _regular
from peak_candidate import (ASSET, check, check_tree, load, portable_names, save,
                            tree_manifest)
import peak_annotation_count_view as view
import peak_first_workflow_oracle as original


DETAIL = ('genic', 'intergenic', 'promoter', 'five_utr', 'three_utr', 'exon',
          'intron', 'downstream', 'distal_intergenic')
CORE = ('annotation', 'geneChr', 'geneStart', 'geneEnd', 'geneLength',
        'geneStrand', 'geneId', 'transcriptId', 'distanceToTSS')


def lines(path):
    with Path(path).open() as stream:
        for number, line in enumerate(stream, 1):
            yield number, line.rstrip('\n').split('\t')


def compare_sparse(native, source):
    import itertools
    rows = 0
    for left, right in itertools.zip_longest(lines(native), lines(source)):
        check(left is not None and right is not None, 'sparse row population differs')
        number, a = left; _, b = right
        check(len(a) == len(b) == 6, 'sparse row width differs at' + str(number))
        check(all(a[index] == b[index] for index in (0, 1, 2, 5)), 'sparse identity differs at' + str(number))
        check(all(float(a[index]) == float(b[index]) for index in (3, 4)), 'sparse six-digit value differs at' + str(number))
        rows += 1
    return rows


def compare_counts(native, source, ledger):
    import itertools
    native_rows = lines(native); header = next(native_rows)[1]
    check(header[-2:] == ['T1', 'T2'], 'native count sample order differs')
    rows = 0
    for a, b, peak in itertools.zip_longest(native_rows, lines(source), lines(ledger)):
        check(a is not None and b is not None and peak is not None, 'count/ledger populations differ')
        number, left = a; _, right = b; _, fields = peak
        check(len(left) == len(right) == 8 and left[:6] == right[:6] == fields,
              'count row identity differs at' + str(number))
        check([int(value) for value in left[6:]] == [int(value) for value in right[6:]],
              'alignment counts differ at' + str(number))
        rows += 1
    return rows


def compare_annotation(native, core, detail, ledger):
    with Path(native).open() as stream:
        actual = list(csv.DictReader(stream, delimiter='\t'))
    with Path(core).open() as stream:
        expected = list(csv.DictReader(stream, delimiter='\t'))
    with Path(detail).open() as stream:
        flags = list(csv.DictReader(stream, delimiter='\t'))
    with Path(ledger).open() as stream:
        identities = list(csv.DictReader(stream, delimiter='\t'))
    check(len(actual) == len(expected) == len(flags) == len(identities), 'annotation row populations differ')
    for number, (row, source, masks, identity) in enumerate(zip(actual, expected, flags, identities)):
        key = identity['row_key']
        check(source['row_key'] == masks['row_key'] == key and
              int(row['row_id']) == int(source['row_ordinal']) == int(masks['row_ordinal']) == number,
              'annotation row identity/order differs at' + str(number))
        for name in CORE:
            check(row[name] == source[name], f'annotation {name} differs at row{number}')
        model = json.loads(row['detail'])
        check(set(model) == set(DETAIL), 'native annotation detail schema differs')
        for name in DETAIL:
            check(masks[name] in ('TRUE', 'FALSE') and type(model[name]) is bool and
                  model[name] == (masks[name] == 'TRUE'), f'annotation detail {name} differs at row{number}')
    return dict(rows=len(actual), core_fields=list(CORE), detail_masks=list(DETAIL),
                private_feature_hit_ids_independently_oracle_qualified=False)


def extract_native(archive_path, destination, world):
    destination, world = Path(destination), Path(world)
    check(not destination.exists() and not destination.is_symlink(), 'native restoration output exists')
    with tarfile.open(archive_path, 'r:') as archive:
        members = archive.getmembers()
        names = [entry.name for entry in members]
        portable_names(names)
        check(all(entry.isfile() and entry.type == tarfile.REGTYPE for entry in members), 'native artifact contains nonregular members')
        check(all(name.startswith('peak-evidence/') for name in names), 'native artifact root differs')
        manifest_member = archive.getmember('peak-evidence/artifact-manifest.json')
        check(manifest_member.size <= 32 * 2**20, 'native file manifest is unexpectedly large')
        manifest = json.load(archive.extractfile(manifest_member))
        check(manifest['schema'] == 'rsomics.peak.gate-artifact.v1', 'native artifact manifest schema differs')
        files = manifest['files']; portable_names(files)
        check(set(names) == {'peak-evidence/' + name for name in files} | {manifest_member.name},
              'native artifact file population differs')
        destination.mkdir()
        for member in members:
            relative = member.name.removeprefix('peak-evidence/')
            expected = files.get(relative)
            target = destination / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            digest = hashlib.sha256()
            with archive.extractfile(member) as source, target.open('xb') as stream:
                for block in iter(lambda: source.read(2**20), b''):
                    digest.update(block); stream.write(block)
            if expected is not None:
                check(member.size == expected['bytes'] and digest.hexdigest() == expected['sha256'],
                      'native artifact body differs: ' + relative)
    root = destination / 'candidate'
    selection = load(root / 'selection.json')
    fixture_record = world / 'assets' / (ASSET + '.json')
    check(selection['controller_commit'] == os.environ['GITHUB_SHA'] and selection['target'] == 'linux-x86_64' and
          selection['fixture_manifest_sha256'] == _hash_file(fixture_record) and
          selection['fixture_archive_sha256'] == load(fixture_record)['archive_sha256'], 'native selection/controller/fixture binding differs')
    check(load(root / 'integrity-after.json')['failures'] == [] and load(root / 'worker-equality.json')['passed'],
          'native source/fixture/worker gate did not pass')
    check(_hash_file(root / 'binary-release') == load(root / 'binary-release.json')['sha256'] ==
          load(root / 'worker-equality.json')['binary_sha256'], 'actual native binary identity differs')
    check(_hash_file(root / 'files.sha256') == selection['source_manifest_sha256'] and
          _hash_file(root / 'source.tar.gz') == selection['source_archive_sha256'] and
          _hash_file(root / 'source/Cargo.lock') == selection['lock_sha256'], 'native source selection changed')
    check_tree(root / 'source', _manifest((root / 'files.sha256').read_bytes()))
    return root, dict(native_archive_sha256=_hash_file(archive_path), selection=selection,
                      delivered_manifest_sha256=_hash_file(destination / 'artifact-manifest.json'))


def restore_seacr(work, restored):
    record = load(work / 'restoration.json')['original_manifest']
    with tarfile.open(work / 'original' / view.TAR_MEMBER, 'r:') as archive:
        for name in ('runtime/SEACR_1.3.sh', 'runtime/SEACR_1.3.R'):
            member = archive.getmember(view.PREFIX + name)
            check(member.isfile() and member.size == record[name]['size'], 'original SEACR member differs')
            target = restored / name
            target.parent.mkdir(parents=True, exist_ok=True)
            with archive.extractfile(member) as source, target.open('xb') as output:
                shutil.copyfileobj(source, output, 2**20)
            check(_hash_file(target) == record[name]['sha256'], 'restored SEACR body differs')


def retain_acquisition(work, output):
    original_root = work / 'original'
    if original_root.is_dir():
        for name in ('run.json', 'jobs.json', 'artifacts.json', 'job-124562023993.log'):
            source = original_root / name
            if source.is_file():
                target = output / 'original-api' / name
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(source, target)
    if (work / 'restoration.json').is_file():
        shutil.copy2(work / 'restoration.json', output / 'original-restoration.json')


def run(native_archive, native_output, work, output, world):
    native_archive, native_output, work, output, world = map(Path, (native_archive, native_output, work, output, world))
    original.hosted(output)
    check(not output.exists() and not work.exists() and not native_output.exists(), 'new source evidence/work/native routes required')
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    for name in ('XDG_CONFIG_HOME', 'XDG_DATA_HOME', 'UV_CACHE_DIR', 'PYTHONPYCACHEPREFIX', 'R_LIBS_SITE'):
        path = Path(os.environ[name]).resolve(strict=True)
        check(path.is_relative_to(runner) and path.stat().st_dev == runner.stat().st_dev,
              'source cache/library route escapes runner device: ' + name)
    for path in (native_archive, native_output, work, output):
        check(path.resolve().is_relative_to(runner) and path.parent.resolve(strict=True).stat().st_dev == runner.stat().st_dev,
              'source input/output route escapes runner device')
    check(shutil.disk_usage(runner).free >= 12 * 2**30, 'insufficient original restoration/source space')
    output.mkdir(); work.mkdir()
    save(output / 'host.json', dict(run_id=os.environ['GITHUB_RUN_ID'], attempt=os.environ['GITHUB_RUN_ATTEMPT'],
        controller=os.environ['GITHUB_SHA'], image=original.IMAGE, source_execution='Linux x86_64 only',
        disk=shutil.disk_usage(runner)._asdict()))
    root = restored = native_before = None
    protected = image_before = None
    bodies = {}; observed = {}; failure = None; image_paths = {}
    try:
        root, binding = extract_native(native_archive, native_output, world)
        save(output / 'native-input-binding.json', binding)
        native_before = tree_manifest(native_output)
        save(output / 'native-input-files-before.json', native_before)
        restored = view.acquire(work)
        os.environ.pop('GH_TOKEN', None)
        restore_seacr(work, restored)
        annotation_input = extract_annotation(work, restored)
        check(_hash_file(original.ROOT / 'peak_first_workflow_oracle.R') ==
              _hash_file(restored / 'controller/peak_first_workflow_oracle.R'),
              'annotation/reload R oracle differs from the accepted original')
        fixture = load(world / 'assets' / (ASSET + '.json'))['files']
        check(_hash_file(annotation_input) == fixture['inputs/annotation.gff3']['sha256'],
              'source GFF3 differs from the native fixture')
        os.environ.update(R_LIBS_USER=str(restored / 'R-library'), R_LIBS_SITE=str(restored / 'R-library'))
        protected = original.files_identity(restored)
        save(output / 'original-protected-before.json', protected)
        final = load(restored / 'runtime/final-runtime-manifest.json')
        image_paths = {name: Path(row['LibPath']) / name for name, row in final['packages'].items() if row['Priority']}
        image_before = {name: original.files_identity(path) for name, path in image_paths.items()}
        save(output / 'image-package-identities-before.json', image_before)
        bodies = view.verify_runtime(restored, output)
        graph = root / 'workers/1'
        (output / 'seacr').mkdir()
        args = ['bash', restored / 'runtime/SEACR_1.3.sh', graph / 'call/native_treat_pileup.bdg',
                '0.10', 'non', 'stringent', output / 'seacr/result']
        original.require(output / 'source-calls/seacr', args)
        observed['sparse'] = dict(rows=compare_sparse(graph / 'seacr-1.3.bed', output / 'seacr/result.stringent.bed'),
                                  source_profile='unmodified SEACR1.3', corrected_branch_replaced=False)
        rows = graph / 'rows'
        annotation = output / 'annotation'
        original.require(output / 'source-calls/annotate', ['Rscript', '--vanilla',
            original.ROOT / 'peak_first_workflow_oracle.R', 'annotate',
            annotation_input, rows, annotation, 'gff3', 'unset'])
        original.require(output / 'source-calls/reload', ['Rscript', '--vanilla',
            original.ROOT / 'peak_first_workflow_oracle.R', 'reload', annotation, output / 'reload', 'unset'])
        original.require(output / 'source-calls/annotation-model', ['Rscript', '--vanilla',
            Path(__file__).with_suffix('.R'), annotation, output / 'annotation-model'])
        observed['annotation'] = compare_annotation(graph / 'annotation.tsv',
            output / 'annotation-model/core.tsv', output / 'annotation-model/detail.tsv', rows / 'workflow-rows.tsv')
        bams = []
        for sample in ('T1', 'T2'):
            name = f'inputs/alignments/{sample}.pe.bam'
            bam = restored / name; _regular(bam)
            check(_hash_file(bam) == fixture[name]['sha256'], 'source count BAM differs from native input: ' + sample)
            _regular(Path(str(bam) + '.bai'))
            bams.append(bam)
        original.require(output / 'source-calls/count', [restored / 'runtime/bedtools-build/bin/bedtools',
            'multicov', '-bams', *bams, '-bed', rows / 'workflow.bed'])
        observed['count'] = dict(rows=compare_counts(graph / 'counts.tsv',
            output / 'source-calls/count/stdout.txt', rows / 'workflow.bed'), unit='alignment', samples=['T1', 'T2'])
    except Exception as error:
        failure = dict(type=type(error).__name__, message=str(error))
    finally:
        retain_acquisition(work, output)
        if native_before is not None:
            native_after = tree_manifest(native_output)
            save(output / 'native-input-files-after.json', native_after)
            native_stable = native_before == native_after
            save(output / 'native-integrity-after.json', dict(unchanged=native_stable))
            if not native_stable and failure is None:
                failure = dict(type='ValueError', message='source consumer changed native input bytes')
        if protected is not None and image_before is not None:
            after = original.files_identity(restored)
            image_after = {name: original.files_identity(path) for name, path in image_paths.items()}
            stable = protected == after and image_before == image_after and all(_hash_file(path) == sha for path, sha in bodies.items())
            save(output / 'runtime-integrity-after.json', dict(unchanged=stable,
                restored_files=after, image_packages=image_after, system_elf_unchanged=all(_hash_file(path) == sha for path, sha in bodies.items())))
            if not stable and failure is None:
                failure = dict(type='ValueError', message='source runtime/package/ELF body changed')
        save(output / 'qualification.json', dict(passed=failure is None, failure=failure, observed=observed,
            original_run=view.RUN, source_on_same_native_files=native_before is not None, native_graph_rerun=False,
            profiles='explicit native seacr-1.3 compared; explicit corrected native rows retained downstream',
            repeated_original_macs_casebook=False, performance_claim=False, release=False))
    check(failure is None, 'source-on-native failed: ' + str(failure))


def extract_annotation(work, restored):
    name = 'inputs/annotation.gff3'
    identity = load(work / 'restoration.json')['original_manifest'][name]
    target = restored / name; target.parent.mkdir(parents=True, exist_ok=True)
    with tarfile.open(work / 'original' / view.TAR_MEMBER, 'r:') as archive:
        member = archive.getmember(view.PREFIX + name)
        check(member.isfile() and member.size == identity['size'], 'original GFF3 identity differs')
        with archive.extractfile(member) as source, target.open('xb') as stream:
            shutil.copyfileobj(source, stream)
    check(_hash_file(target) == identity['sha256'], 'original GFF3 body differs')
    return target


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
