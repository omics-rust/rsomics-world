"""Pack and verify the private PLINK offline source delivery, without building."""

import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tomllib

from plink_candidate import (SNAPSHOTS, check, file_identity, load, portable_names,
                             prepare_source, read_snapshot_manifest, save, tree_manifest)

BUNDLES = {
    'private-green-1': ('assets/plink-source-private-green-1-20261101', 'green-1'),
    'private-green-2': ('assets/plink-source-private-green-2-20261101', 'green-2'),
    'private-green-3': ('assets/plink-source-private-green-3-20261101', 'green-3'),
}
SCHEMA = 'rsomics.plink.private-offline-source.v1'
SUPPLEMENTS = 'assets/plink-source-delivery-20261101'


def source_tree(root):
    root = Path(root)
    check(root.is_dir() and not root.is_symlink(), 'regular source directory required')
    for path in root.rglob('*'):
        check(not path.is_symlink() and (path.is_file() or path.is_dir()),
              'source tree has a nonregular entry')
    return tree_manifest(root)


def verify_original(root):
    root = Path(root)
    model = load(root / 'bundle.json')
    check(model['schema'] == SCHEMA and model['selector'] in BUNDLES and
          model['snapshot'] == BUNDLES[model['selector']][1] and
          model['public_release'] is False, 'private source bundle identity differs')
    actual = source_tree(root)
    actual.pop('bundle.json', None)
    check(actual == model['files'], 'original delivered source changed')
    check(read_snapshot_manifest(root / 'authority') == model['owner_files'],
          'bundled original source authority differs')
    for name, digest in model['owner_files'].items():
        check(model['files']['code/' + name]['sha256'] == digest,
              'original owner body differs: ' + name)
    lock = tomllib.loads((root / 'code/Cargo.lock').read_text())
    expected = {(p['name'], p['version']): p['checksum'] for p in lock['package'] if 'source' in p}
    check(len(expected) == len(model['dependencies']) == 85, 'offline dependency closure differs')
    for item in model['dependencies']:
        directory = root / 'dependencies' / item['directory']
        check(file_identity(root / item['archive'])['sha256'] == item['checksum'],
              'original registry archive differs')
        checksum = load(directory / '.cargo-checksum.json')
        check(checksum['package'] == expected[(item['name'], item['version'])] == item['checksum'],
              'registry package identity differs')
        for name, digest in checksum['files'].items():
            check(file_identity(directory / name)['sha256'] == digest, 'vendored registry body differs')
    return model


def unpack_bundle(world, selector, output):
    check(selector in BUNDLES, 'unknown private source-bundle selector')
    asset, snapshot = BUNDLES[selector]
    world, output = Path(world), Path(output)
    receipt = load(world / (asset + '.json'))
    archive = world / (asset + '.tar.gz')
    check(receipt['selector'] == selector and file_identity(archive) == receipt['archive'],
          'delivered source archive differs')
    check(not output.exists() and not output.is_symlink(), 'fresh delivery restoration required')
    with tarfile.open(archive, 'r:gz') as carrier:
        members = carrier.getmembers()
        names = [item.name for item in members]
        portable_names(names)
        check(all(item.isfile() and item.type == tarfile.REGTYPE for item in members),
              'source bundle has nonregular members')
        manifest = carrier.getmember('bundle.json')
        check(manifest.size < 16 * 2**20, 'source manifest is too large')
        model = json.load(carrier.extractfile(manifest))
        check(set(names) == set(model['files']) | {'bundle.json'}, 'source bundle member set differs')
        check(sum(item.size for item in members) < 512 * 2**20, 'source bundle exceeds bound')
        output.mkdir()
        for item in members:
            target = output / item.name
            target.parent.mkdir(parents=True, exist_ok=True)
            with carrier.extractfile(item) as source, target.open('xb') as dest:
                shutil.copyfileobj(source, dest)
            target.chmod(item.mode & 0o777)
    model = verify_original(output)
    check(model['selector'] == selector, 'delivered source selector differs')
    authority = world / SNAPSHOTS / snapshot
    check(model['owner_files'] == read_snapshot_manifest(authority) and
          model['source_archive_sha256'] == file_identity(authority / 'source.tar.gz')['sha256'] and
          model['source_manifest_sha256'] == file_identity(authority / 'files.sha256')['sha256'],
          'delivered owner differs from actual frozen authority')
    check(model['source_archive_sha256'] == receipt['source_archive_sha256'] and
          model['source_manifest_sha256'] == receipt['source_manifest_sha256'],
          'source authority binding differs')
    return model


def pack(world, cache, work, selector):
    check(selector in BUNDLES, 'unknown private source-bundle selector')
    asset, snapshot = BUNDLES[selector]
    world, cache, work = Path(world).resolve(strict=True), Path(cache).resolve(strict=True), Path(work)
    check(os.environ.get('RSOMICS_GUARDED_TARGET_DIR') and
          cache == Path(os.environ['CARGO_HOME']).resolve(strict=True), 'owning external guard is required')
    check(not work.exists() and work.parent.resolve(strict=True).stat().st_dev == cache.stat().st_dev,
          'new external packaging directory required')
    work.mkdir()
    authority = work / 'frozen'
    owner = prepare_source(world, authority, snapshot)
    root = work / 'bundle'; root.mkdir()
    shutil.copytree(authority / 'source', root / 'code')
    (root / 'authority').mkdir()
    for name in ('README.md', 'files.sha256', 'checksums.sha256', 'source.tar.gz'):
        shutil.copy2(authority / name, root / 'authority' / name)
    shutil.copy2(root / 'code/Cargo.lock', root / 'authority/lock.copy')
    lock = tomllib.loads((root / 'code/Cargo.lock').read_text())
    packages = [p for p in lock['package'] if 'source' in p]
    check(len(packages) == 85, 'private locked dependency population differs')
    for package in packages:
        paths = list((cache / 'registry/cache').glob(f"*/{package['name']}-{package['version']}.crate"))
        check(len(paths) == 1 and file_identity(paths[0])['sha256'] == package['checksum'],
              'exact cached registry archive is absent or changed')
    argv = [os.environ['CARGO'], 'vendor', '--locked', '--offline', '--versioned-dirs',
            str(root / 'dependencies')]
    result = subprocess.run(argv, cwd=root / 'code', text=True, capture_output=True)
    (work / 'vendor.stdout.txt').write_text(result.stdout)
    (work / 'vendor.stderr.txt').write_text(result.stderr)
    check(result.returncode == 0, 'offline Cargo vendor failed; retained packaging logs')
    snippet = tomllib.loads(result.stdout)
    check(snippet['source']['vendored-sources']['directory'] == str(root / 'dependencies'),
          'Cargo vendor source route differs')
    config = root / 'code/.cargo/config.toml'; config.parent.mkdir()
    config.write_text('[source.crates-io]\nreplace-with = "delivered"\n\n'
                      '[source.delivered]\ndirectory = "../dependencies"\n')
    delivery = world / SUPPLEMENTS
    shutil.copytree(delivery, root / 'delivery')
    if selector == 'private-green-3':
        history = root / 'delivery/CHANGES.md'
        history.write_text(history.read_text() + '\n2026-11-01: private-green-3 binds owning revision'
            '00ce7b39f8decbeab40ed7f92f1191a304827b74 and its accepted native/source gate41258923590.'
            ' It includes the shared private QC/LD numeric formatter, reference-frequency arithmetic,'
            ' bounded pair-weighted cache partitions and runtime-gated x86 POPCNT with portable fallback.'
            ' All85 locked dependencies and original pgenlib/SIMDe terms/bodies remain unchanged.'
            ' Snapshot authority README status describes capture time; the new offline replacement'
            ' proof remains pending and earlier bundle proofs do not qualify this revision.\n')
        instructions = root / 'delivery/REBUILD.md'
        instructions.write_text(instructions.read_text() + '\nPrivate-green-3 retains the green-2 ordinary'
            ' downstream guard policy. Its303 owner files include the subsequent QC/LD implementation'
            ' changes recorded in code/MERGE_RECORD.md; rebuild and replacement results must bind this'
            ' exact source, not the prior green-2 acceptance.\n')
    for item in load(delivery / 'licenses.json')['files']:
        check(file_identity(root / 'delivery' / item['path']) ==
              {key: item[key] for key in ('bytes', 'sha256')}, 'license supplement body differs')
    dependencies = []
    (root / 'archives').mkdir()
    for package in packages:
        stem = package['name'] + '-' + package['version']
        source = root / 'dependencies' / stem
        original = next((cache / 'registry/cache').glob(f'*/{stem}.crate'))
        shutil.copy2(original, root / 'archives' / original.name)
        original_files = set()
        archived_only = []
        with tarfile.open(original, 'r:gz') as archive:
            for member in archive:
                if member.isdir():
                    continue
                check(member.isfile() and member.name.startswith(stem + '/'), 'registry source member differs')
                name = member.name.removeprefix(stem + '/')
                portable_names([name])
                body = archive.extractfile(member).read()
                if (source / name).is_file():
                    check(file_identity(source / name) == dict(bytes=len(body),
                          sha256=hashlib.sha256(body).hexdigest()), 'offline source differs from original crate')
                else:
                    archived_only.append(name)
                original_files.add(name)
        check(set(source_tree(source)) == (original_files - set(archived_only)) | {'.cargo-checksum.json'},
              'unexpected offline source addition')
        data = tomllib.loads((source / 'Cargo.toml').read_text())['package']
        notices = [p.relative_to(root).as_posix() for p in sorted(source.rglob('*'))
                   if p.is_file() and p.name.upper().startswith(
                       ('LICENSE', 'LICENCE', 'COPYING', 'NOTICE', 'COPYRIGHT', 'AUTHORS'))]
        dependencies.append(dict(name=package['name'], version=package['version'],
                                 checksum=package['checksum'], directory=stem,
                                 archive='archives/' + original.name, archived_only=archived_only,
                                 license=data.get('license'), notices=notices))
    model = dict(schema=SCHEMA, selector=selector, scope=selector + '; not public-install qualified',
                 public_release=False, snapshot=snapshot, owner_files=owner, dependencies=dependencies,
                 source_archive_sha256=file_identity(authority / 'source.tar.gz')['sha256'],
                 source_manifest_sha256=file_identity(authority / 'files.sha256')['sha256'],
                 public_holds=['ordinary downstream macOS install contract', 'final public owner exact-head native-four',
                               'same-release source/binary publication and notices', 'representative real-cohort performance'],
                 files=source_tree(root))
    save(root / 'bundle.json', model)
    verify_original(root)
    output = world / (asset + '.tar.gz')
    check(not output.exists(), 'do not replace an existing source bundle')
    with output.open('xb') as stream, gzip.GzipFile(fileobj=stream, mode='wb', mtime=0, filename='') as gz:
        with tarfile.open(fileobj=gz, mode='w', format=tarfile.PAX_FORMAT) as archive:
            for name in sorted([*model['files'], 'bundle.json']):
                path = root / name; item = tarfile.TarInfo(name)
                item.size = path.stat().st_size; item.mode = 0o755 if path.stat().st_mode & 0o111 else 0o644
                with path.open('rb') as source: archive.addfile(item, source)
    check(output.stat().st_size < 50 * 2**20, 'source bundle exceeds ordinary tracked-asset limit')
    save(world / (asset + '.json'), dict(selector=selector, scope=model['scope'], archive=file_identity(output),
         source_archive_sha256=model['source_archive_sha256'],
         source_manifest_sha256=model['source_manifest_sha256'], public_release=False))
    return dict(files=len(model['files']), dependencies=len(dependencies), archive=file_identity(output))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--world', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--work', type=Path, required=True)
    parser.add_argument('--bundle', choices=sorted(BUNDLES), required=True)
    args = parser.parse_args()
    print(json.dumps(pack(args.world, args.cache, args.work, args.bundle), sort_keys=True))


if __name__ == '__main__':
    main()
