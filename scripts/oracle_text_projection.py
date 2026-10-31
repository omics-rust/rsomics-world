"""Verify one original oracle carrier and deliver bounded diagnostic text only."""

import argparse
import collections
import hashlib
import json
import os
from pathlib import Path
import platform
import posixpath
import re
import shutil
import stat
import tarfile
import urllib.error
import urllib.parse
import urllib.request
import zipfile


REPOSITORY = 'omics-rust/rsomics-world'
TEXT_LIMIT = 2 * 2**20
TOTAL_TEXT_LIMIT = 32 * 2**20
MANIFEST_LIMIT = 32 * 2**20
MEMBER_LIMIT = 262144
STATUS = 'verified text projection, not full original retention or source admission'


def check(condition, message):
    if not condition:
        raise ValueError(message)


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(2**20), b''):
            digest.update(block)
    return digest.hexdigest()


def save(path, data):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('x') as output:
        json.dump(data, output, indent=2, sort_keys=True, allow_nan=False)
        output.write('\n')


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        check(key not in result, 'duplicate JSON key')
        result[key] = value
    return result


def archive_name(name, directory=False):
    if directory and name.endswith('/'):
        name = name[:-1]
    check(isinstance(name, str) and 0 < len(name) <= 4096 and not name.startswith('/') and
          '\\' not in name and all(ord(character) >= 32 and ord(character) != 127 for character in name) and
          all(part not in ('', '.', '..') for part in name.split('/')), 'unsafe archive path')
    return name


def selected_text(name):
    if name in {'failure.json', 'host.json', 'storage-paths.json', 'integrity-after.json',
                'statuses.json', 'qualification.json'}:
        return True
    parts = name.split('/')
    if len(parts) == 4 and parts[:2] == ['runtime-check', 'ldd'] and re.fullmatch(r'[0-9]{1,8}', parts[2]):
        return parts[3] in {'request.json', 'process.json', 'status.json', 'stdout.txt', 'stderr.txt'}
    if len(parts) == 3 and parts[0] == 'runtime-check':
        if parts[1] in {'R', 'bedtools-version'}:
            return parts[2] in {'request.json', 'process.json', 'status.json', 'stdout.txt', 'stderr.txt'}
        if parts[1] == 'R-state':
            return parts[2] in {'namespaces.tsv', 'DLLs.tsv', 'installed.tsv', 'session.txt',
                                'sessionInfo.txt', 'options.txt', 'environment.tsv'}
    return False


def manifest_table(data):
    value = json.loads(data, object_pairs_hook=unique_object)
    check(isinstance(value, dict) and isinstance(value.get('files'), dict), 'missing current artifact-manifest.json files table')
    table = value['files']
    check(0 < len(table) <= MEMBER_LIMIT, 'original manifest population exceeds bound or is empty')
    for name, identity in table.items():
        archive_name(name)
        check(name != 'artifact-manifest.json' and isinstance(identity, dict), 'invalid manifest identity')
        if set(identity) == {'size', 'sha256'}:
            check(type(identity['size']) is int and identity['size'] >= 0 and
                  isinstance(identity['sha256'], str) and re.fullmatch('[0-9a-f]{64}', identity['sha256']),
                  'invalid regular manifest identity')
        else:
            check(set(identity) == {'symlink'} and isinstance(identity['symlink'], str) and
                  0 < len(identity['symlink']) <= 4096 and '\0' not in identity['symlink'] and
                  '\\' not in identity['symlink'], 'invalid symlink manifest identity')
    return table


def virtual_alias(name, identities, active=()):
    check(name not in active and len(active) < 64, 'cyclic or excessively deep virtual symlink')
    target = identities[name]['symlink']
    if target.startswith('/'):
        return dict(kind='external absolute original target; not resolved or materialized', target=target)
    relative = posixpath.normpath(posixpath.join(posixpath.dirname(name), target))
    if relative == '..' or relative.startswith('../'):
        return dict(kind='external relative original target; not resolved or materialized', target=target)
    check(relative in identities, 'virtual symlink target absent from original manifest')
    if 'symlink' in identities[relative]:
        return virtual_alias(relative, identities, (*active, name))
    return dict(kind='verified internal regular body', path=relative, sha256=identities[relative]['sha256'])


def project_archive(carrier, staging, delivery, prefix, expected_sha256, provenance):
    carrier, staging, delivery = Path(carrier), Path(staging), Path(delivery)
    check(prefix.endswith('/') and archive_name(prefix[:-1]) + '/' == prefix, 'explicit root prefix must be canonical and end in /')
    check(sha256(carrier) == expected_sha256, 'original ZIP digest differs')
    staging.mkdir(parents=True, exist_ok=False)
    delivery.mkdir(parents=True, exist_ok=True)
    save(delivery / 'projection-started.json', dict(status='INCOMPLETE until projection-proof.json exists', provenance=provenance))
    tar_path = staging / 'original.tar'
    zip_inventory = []
    with zipfile.ZipFile(carrier) as archive:
        members = archive.infolist()
        check(0 < len(members) <= 16, 'unexpected original ZIP member population')
        names = [archive_name(item.filename, item.is_dir()) for item in members]
        check(len(set(names)) == len(names), 'duplicate original ZIP member')
        carriers = [item for item in members if not item.is_dir() and item.filename.endswith('.tar')]
        check(len(carriers) == 1, 'original ZIP must contain exactly one uncompressed TAR carrier; other archive formats are unsupported')
        check(shutil.disk_usage(staging).free >= carriers[0].file_size + 128 * 2**20, 'insufficient scratch for original TAR')
        for item, name in zip(members, names):
            mode = stat.S_IFMT(item.external_attr >> 16)
            check(mode in (0, stat.S_IFREG, stat.S_IFDIR) and (mode != stat.S_IFDIR or item.is_dir()),
                  'special or symlink original ZIP member')
            is_tar = item is carriers[0]
            check(item.is_dir() or is_tar or name.endswith('.json') and item.file_size <= TEXT_LIMIT,
                  'unsupported non-TAR ZIP payload')
            digest, count = hashlib.sha256(), 0
            output = tar_path.open('xb') if is_tar else None
            try:
                with archive.open(item) as stream:
                    for block in iter(lambda: stream.read(2**20), b''):
                        digest.update(block)
                        count += len(block)
                        if output:
                            output.write(block)
            finally:
                if output:
                    output.close()
            check(count == item.file_size, 'original ZIP body length differs')
            zip_inventory.append(dict(name=name, bytes=count, sha256=digest.hexdigest(), crc32=item.CRC,
                                      kind='TAR carrier' if is_tar else 'auxiliary original metadata or directory'))
    tar_record = next(row for row in zip_inventory if row['kind'] == 'TAR carrier')
    manifest_name = prefix + 'artifact-manifest.json'
    manifest_bytes = None
    with tarfile.open(tar_path, 'r:') as archive:
        for index, item in enumerate(archive):
            check(index < MEMBER_LIMIT, 'original TAR member population exceeds bound')
            archive_name(item.name, item.isdir())
            if item.name == manifest_name:
                check(item.isfile() and item.size <= MANIFEST_LIMIT, 'original manifest is not a bounded regular body')
                manifest_bytes = archive.extractfile(item).read()
                break
    check(manifest_bytes is not None, 'explicit root has no artifact-manifest.json')
    expected = manifest_table(manifest_bytes)
    identities, seen, folded = {}, set(), collections.defaultdict(list)
    projected, omitted, aliases = [], [], []
    text_bytes, total_regular_bytes, regular_count, directory_count = 0, 0, 0, 0
    selected_folds = set()
    with tarfile.open(tar_path, 'r:') as archive:
        for index, item in enumerate(archive):
            check(index < MEMBER_LIMIT, 'original TAR member population exceeds bound')
            name = archive_name(item.name, item.isdir())
            check(name not in seen, 'duplicate original TAR member')
            seen.add(name)
            folded[name.casefold()].append(name)
            check(name == prefix[:-1] and item.isdir() or name.startswith(prefix), 'original TAR member outside explicit root prefix')
            if item.isdir():
                directory_count += 1
                continue
            relative = name[len(prefix):]
            if item.issym():
                identity = dict(symlink=item.linkname)
                check(expected.get(relative) == identity, 'original manifest identity differs: ' + relative)
                identities[relative] = identity
                aliases.append(dict(path=relative, target=item.linkname))
                continue
            check(item.isfile(), 'special or hard-link original TAR member')
            digest, count, selected = hashlib.sha256(), 0, selected_text(relative)
            body = bytearray() if selected and item.size <= TEXT_LIMIT else None
            with archive.extractfile(item) as stream:
                for block in iter(lambda: stream.read(2**20), b''):
                    digest.update(block)
                    count += len(block)
                    if body is not None:
                        body.extend(block)
            check(count == item.size, 'original TAR body length differs: ' + relative)
            identity = dict(size=count, sha256=digest.hexdigest())
            if name == manifest_name:
                check(identity == dict(size=len(manifest_bytes), sha256=hashlib.sha256(manifest_bytes).hexdigest()),
                      'original manifest body changed')
                continue
            check(expected.get(relative) == identity, 'original manifest identity differs: ' + relative)
            identities[relative] = identity
            regular_count += 1
            total_regular_bytes += count
            if not selected:
                continue
            record = dict(path=relative, bytes=count, sha256=identity['sha256'])
            if body is None:
                omitted.append(dict(record, reason='exceeds 2MiB selected-text cap; original body fully verified'))
                continue
            try:
                body.decode('utf-8')
            except UnicodeDecodeError:
                raise ValueError('selected diagnostic is not UTF-8 text: ' + relative) from None
            check(b'\0' not in body, 'selected diagnostic contains binary NUL: ' + relative)
            check(relative.casefold() not in selected_folds, 'casefold-colliding selected text paths')
            selected_folds.add(relative.casefold())
            text_bytes += count
            check(text_bytes <= TOTAL_TEXT_LIMIT, 'selected text exceeds 32MiB total cap')
            destination = delivery / 'text' / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            with destination.open('xb') as output:
                output.write(body)
            projected.append(record)
    check(identities == expected, 'original manifest identity population differs')
    for alias in aliases:
        alias['virtual_resolution'] = virtual_alias(alias['path'], identities)
    check(projected and text_bytes > 0, 'empty diagnostic text projection is not success')
    collisions = [names for names in folded.values() if len(names) > 1]
    proof = dict(schema_version=1, status=STATUS, provenance=provenance,
        original_zip=dict(bytes=carrier.stat().st_size, sha256=expected_sha256, crc_verified_entries=zip_inventory),
        original_tar=tar_record, manifest=dict(path=manifest_name, bytes=len(manifest_bytes),
            sha256=hashlib.sha256(manifest_bytes).hexdigest(), entries=len(expected), every_identity_verified=True),
        verified_regular_bodies=regular_count, verified_regular_bytes=total_regular_bytes,
        verified_symlinks=len(aliases), directories=directory_count, virtual_aliases=aliases,
        casefold_collision_groups=len(collisions), casefold_collisions=collisions,
        projected=sorted(projected, key=lambda row: row['path']), omitted_text=sorted(omitted, key=lambda row: row['path']),
        projected_text_bytes=text_bytes, limits=dict(per_text_bytes=TEXT_LIMIT, all_text_bytes=TOTAL_TEXT_LIMIT),
        archive_execution=False, runtime_extraction=False, scientific_execution=False)
    check(len(json.dumps(proof).encode()) <= 2 * 2**20, 'projection proof exceeds small-delivery bound')
    save(delivery / 'projection-proof.json', proof)
    return proof


class ArtifactRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, response, code, message, headers, newurl):
        url = urllib.parse.urlsplit(newurl)
        check(url.scheme == 'https' and url.hostname and any(url.hostname.endswith(suffix) for suffix in
              ('.blob.core.windows.net', '.actions.githubusercontent.com', '.githubusercontent.com')),
              'unadmitted artifact redirect host')
        return super().redirect_request(request, response, code, message, headers, newurl)


def fetch(endpoint, target, limit):
    token = os.environ.get('GH_TOKEN') or os.environ.get('GITHUB_TOKEN')
    check(token, 'read-only Actions API token required')
    request = urllib.request.Request('https://api.github.com/repos/' + REPOSITORY + '/' + endpoint,
        headers={'Accept': 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28',
                 'User-Agent': 'rsomics-bounded-oracle-text-projection'})
    request.add_unredirected_header('Authorization', 'Bearer ' + token)
    try:
        with urllib.request.build_opener(ArtifactRedirect()).open(request, timeout=120) as response, Path(target).open('xb') as output:
            total = 0
            for block in iter(lambda: response.read(2**20), b''):
                total += len(block)
                check(total <= limit, 'original acquisition exceeds declared byte bound')
                output.write(block)
    except urllib.error.HTTPError as error:
        raise ValueError('original API/artifact request failed with HTTP ' + str(error.code)) from None
    except urllib.error.URLError:
        raise ValueError('original API/artifact transport failed; no credential or redirect URL retained') from None
    return dict(endpoint=endpoint, bytes=Path(target).stat().st_size, sha256=sha256(target))


def acquire(args, download, delivery):
    download.mkdir()
    api_directory = delivery / 'original-api'
    api_directory.mkdir()
    requests = []

    def metadata(name, endpoint):
        path = api_directory / (name + '.json')
        requests.append(fetch(endpoint, path, 16 * 2**20))
        return json.loads(path.read_bytes(), object_pairs_hook=unique_object)

    run = metadata('run', f'actions/runs/{args.run_id}')
    check(run['id'] == args.run_id and run['run_attempt'] == args.attempt and run['head_sha'] == args.head_sha and
          run['repository']['full_name'] == REPOSITORY and run['status'] == 'completed', 'original terminal run/head/attempt differs')
    jobs = metadata('jobs', f'actions/runs/{args.run_id}/attempts/{args.attempt}/jobs?per_page=100')
    check(jobs['total_count'] == len(jobs['jobs']) and 0 < len(jobs['jobs']) <= 100 and
          all(row['run_id'] == args.run_id and row['run_attempt'] == args.attempt and
              row['head_sha'] == args.head_sha and row['status'] == 'completed'
              for row in jobs['jobs']), 'original completed attempt job population differs')
    artifacts = metadata('artifacts', f'actions/runs/{args.run_id}/artifacts?per_page=100')
    check(artifacts['total_count'] == len(artifacts['artifacts']), 'original artifact listing is truncated')
    matches = [row for row in artifacts['artifacts'] if row['id'] == args.artifact_id]
    check(len(matches) == 1, 'explicit original artifact absent or ambiguous')
    artifact = matches[0]
    check(not artifact['expired'] and artifact['workflow_run']['id'] == args.run_id and
          artifact['workflow_run']['head_sha'] == args.head_sha and
          artifact['digest'] == 'sha256:' + args.artifact_sha256 and
          re.search(r'(?:^|-)' + str(args.run_id) + '-' + str(args.attempt) + r'(?:-|$)', artifact['name']),
          'original artifact run/head/attempt-name/digest differs')
    size = artifact['size_in_bytes']
    check(type(size) is int and size > 0 and shutil.disk_usage(download).free >= size + 128 * 2**20,
          'original artifact has invalid size or insufficient hosted scratch')
    save(delivery / 'acquisition-started.json', dict(repository=REPOSITORY, run=args.run_id,
         attempt=args.attempt, head_sha=args.head_sha, artifact_id=args.artifact_id,
         artifact_name=artifact['name'], expected_size=size, expected_sha256=args.artifact_sha256,
         verified_metadata_requests=requests, complete_original_acquisition=False))
    carrier = download / 'original.zip'
    requests.append(fetch(f'actions/artifacts/{args.artifact_id}/zip', carrier, size))
    check(carrier.stat().st_size == size and sha256(carrier) == args.artifact_sha256, 'complete original ZIP size/digest differs')
    save(delivery / 'acquisition.json', dict(repository=REPOSITORY, run=args.run_id, attempt=args.attempt,
         head_sha=args.head_sha, conclusion=run['conclusion'], artifact_id=args.artifact_id,
         artifact_name=artifact['name'], artifact_size=size, artifact_sha256=args.artifact_sha256, requests=requests))
    return carrier


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run-id', type=int, required=True)
    parser.add_argument('--attempt', type=int, required=True)
    parser.add_argument('--head-sha', required=True)
    parser.add_argument('--artifact-id', type=int, required=True)
    parser.add_argument('--artifact-sha256', required=True)
    parser.add_argument('--root-prefix', required=True)
    parser.add_argument('--work', type=Path, required=True)
    args = parser.parse_args()
    check(args.run_id > 0 and args.attempt > 0 and args.artifact_id > 0 and
          re.fullmatch('[0-9a-f]{40}', args.head_sha) and re.fullmatch('[0-9a-f]{64}', args.artifact_sha256),
          'explicit positive original identities and lowercase digests required')
    check(args.root_prefix.endswith('/') and archive_name(args.root_prefix[:-1]) + '/' == args.root_prefix,
          'explicit root prefix must be canonical and end in /')
    check(os.environ.get('GITHUB_ACTIONS') == 'true' and os.environ.get('GITHUB_REPOSITORY') == REPOSITORY and
          platform.system() == 'Linux', 'network acquisition is restricted to the manual hosted fixed-repository workflow')
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    work = args.work.resolve(strict=True)
    check(work.is_relative_to(runner) and work != runner and work.stat().st_dev == runner.stat().st_dev,
          'projection work must be a dedicated existing RUNNER_TEMP subtree')
    for name in ['TMPDIR', 'XDG_CACHE_HOME', 'XDG_CONFIG_HOME', 'XDG_DATA_HOME']:
        path = Path(os.environ[name]).resolve(strict=True)
        check(path.is_relative_to(work) and path.stat().st_dev == runner.stat().st_dev, 'projection cache escapes explicit hosted scratch')
    delivery = work / 'delivery'
    delivery.mkdir()
    try:
        head = os.environ['GITHUB_SHA']
        check(re.fullmatch('[0-9a-f]{40}', head), 'projection controller head absent')
        world = Path(__file__).resolve().parents[1]
        files = ['scripts/oracle_text_projection.py', 'scripts/test_oracle_text_projection.py',
                 '.github/workflows/oracle-text-projection.yml']
        controller = dict(repository=REPOSITORY, head_sha=head, run_id=os.environ['GITHUB_RUN_ID'],
                          attempt=os.environ['GITHUB_RUN_ATTEMPT'], files={name: sha256(world / name) for name in files})
        for name in files:
            target = delivery / 'controller' / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(world / name, target)
        save(delivery / 'controller.json', controller)
        carrier = acquire(args, work / 'download', delivery)
        source = json.loads((delivery / 'acquisition.json').read_bytes())
        proof = project_archive(carrier, work / 'archive-staging', delivery, args.root_prefix,
                                args.artifact_sha256, dict(original=source, projection_controller=controller))
        print(json.dumps(dict(status=proof['status'], projected_files=len(proof['projected']),
                              projected_text_bytes=proof['projected_text_bytes'], original_run=args.run_id)))
    except Exception as error:
        save(delivery / 'failure.json', dict(status='projection failed; partial text is not verified admission',
             exception=type(error).__name__, message=str(error) if isinstance(error, ValueError) else 'unexpected projection failure; inspect retained metadata',
             original_run=args.run_id, original_artifact=args.artifact_id, source_execution=False))
        raise SystemExit('projection failed; bounded diagnostics retained without credentials or signed URLs') from None


if __name__ == '__main__':
    main()
