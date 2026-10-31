"""Private hosted-Linux proof of offline source delivery and codec replacement."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import urllib.error
import urllib.parse
import urllib.request
import zipfile

import plink_candidate as native
from plink_source_bundle import (SELECTOR, check, file_identity, load, save,
                                 source_tree, unpack_bundle, verify_original)
from plink_source_on_native import NoRedirect, extract_artifact

MARKER = b'rsomics-pgenlib-relink-proof-v1'
CODEC = 'code/vendor/pgenlib/include/pgenlib_read.cc'
JOBS = {'linux-x86_64', 'linux-aarch64', 'macos-x86_64', 'macos-aarch64',
        'Pinned source and official engines read same native generations'}


def scientific_paths(binding):
    paths = set(binding['reports'])
    for generation in binding['generations']:
        paths.update(name for name in generation['files']
                     if name.endswith(('.pvar', '.psam', '.raw.json')))
    check(len(paths) == 336, 'complete scientific comparison membership differs')
    return paths


def compare_scientific(left, right, paths):
    files = {}
    for name in sorted(paths):
        first, second = file_identity(Path(left) / name), file_identity(Path(right) / name)
        check(first == second, 'base/modified scientific output differs: ' + name)
        files[name] = first
    check(files, 'empty scientific comparison')
    return dict(file_count=len(files), files=files)


def replace_codec(original, derivative):
    original, derivative = Path(original), Path(derivative)
    model = verify_original(original)
    check(not derivative.exists(), 'fresh derivative required')
    shutil.copytree(original, derivative)
    path = derivative / CODEC
    before = path.read_bytes()
    anchor = b'PglErr PgrInit(const char* fname,'
    check(before.count(anchor) == 1 and MARKER not in before, 'exact PgrInit replacement anchor differs')
    start = before.index(b'{', before.index(anchor)) + 1
    insertion = (b'\n  // Modified 2026-11-01 solely for the private source/relink delivery proof.\n'
                 b'  static const volatile char marker[] = "' + MARKER + b'";\n'
                 b'  for (unsigned i = 0; i + 1 < sizeof(marker); ++i) { (void)marker[i]; }\n')
    path.write_bytes(before[:start] + insertion + before[start:])
    proof = dict(original_bundle_sha256=file_identity(original / 'bundle.json')['sha256'],
                 changed_paths=[CODEC], before_sha256=hashlib.sha256(before).hexdigest(),
                 after_sha256=file_identity(path)['sha256'], insertion=insertion.decode(),
                 abi=2, marker=MARKER.decode(), scientific_options_changed=False)
    verify_derivative(derivative, model, proof)
    return proof


def verify_derivative(root, original, proof):
    root = Path(root)
    check(file_identity(root / 'bundle.json')['sha256'] == proof['original_bundle_sha256'],
          'original delivery manifest was rewritten')
    observed = source_tree(root); observed.pop('bundle.json', None)
    expected = dict(original['files'])
    check(proof['changed_paths'] == [CODEC] and proof['before_sha256'] == expected[CODEC]['sha256'],
          'replacement ancestry differs')
    body = (root / CODEC).read_bytes()
    insertion = proof['insertion'].encode()
    check(body.count(insertion) == 1 and hashlib.sha256(body.replace(insertion, b'', 1)).hexdigest()
          == proof['before_sha256'] and file_identity(root / CODEC)['sha256'] == proof['after_sha256'],
          'replacement is not the declared harmless codec-body patch')
    expected[CODEC] = file_identity(root / CODEC)
    check(observed == expected, 'unrelated derivative mutation')


def hosted(work, evidence, minimum_free=8 * 2**30):
    check(os.environ.get('GITHUB_ACTIONS') == 'true' and
          os.environ.get('GITHUB_REPOSITORY') == native.REPOSITORY and
          platform.system() == 'Linux' and platform.machine() == 'x86_64',
          'source/relink execution requires the hosted Linux repository workflow')
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    for name in ('CARGO_HOME', 'CARGO_TARGET_DIR', 'RUSTUP_HOME', 'TMPDIR', 'XDG_CACHE_HOME',
                 'CLANG_MODULE_CACHE_PATH', 'CCACHE_DIR', 'SCCACHE_DIR'):
        path = Path(os.environ[name]).resolve(strict=True)
        check(path.is_relative_to(runner) and path != runner and path.stat().st_dev == runner.stat().st_dev,
              'tool route escapes admitted runner device: ' + name)
    for path in (Path(work), Path(evidence)):
        check(path.resolve().is_relative_to(runner) and path.resolve() != runner and
              path.parent.resolve(strict=True).stat().st_dev == runner.stat().st_dev,
              'source/relink route escapes admitted runner device')
    check(shutil.disk_usage(runner).free >= minimum_free, 'insufficient source/relink scratch space')
    return runner


def api(path, target, binary=False):
    target = Path(target)
    request = urllib.request.Request('https://api.github.com/repos/' + native.REPOSITORY + '/' + path,
        headers={'Authorization': 'Bearer ' + os.environ['GH_TOKEN'],
                 'Accept': 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28'})
    try:
        response = urllib.request.build_opener(NoRedirect).open(request, timeout=120)
    except urllib.error.HTTPError as error:
        if not binary or error.code != 302:
            save(str(target) + '.error.json', dict(status=error.code, api_path=path))
            raise RuntimeError('native admission API failed: ' + str(error.code)) from None
        location = urllib.parse.urlsplit(error.headers['Location'])
        check(location.scheme == 'https' and location.hostname and any(location.hostname.endswith(suffix)
              for suffix in ('.blob.core.windows.net', '.actions.githubusercontent.com', '.githubusercontent.com')),
              'untrusted native artifact redirect')
        response = urllib.request.urlopen(error.headers['Location'], timeout=120)
    count = 0
    with response, target.open('xb') as output:
        for block in iter(lambda: response.read(2**20), b''):
            count += len(block)
            check(count <= (128 * 2**20 if binary else 8 * 2**20), 'native admission exceeds bound')
            output.write(block)
    return None if binary else load(target)


def admit(world, selector, run_id, work, evidence):
    hosted(work, evidence)
    check(re.fullmatch(r'[1-9][0-9]*', str(run_id)), 'actual native run id required')
    evidence, work = Path(evidence), Path(work)
    evidence.mkdir(exist_ok=True); work.mkdir(exist_ok=True)
    model = unpack_bundle(world, selector, work / 'admission-source')
    run = api(f'actions/runs/{run_id}', evidence / 'native-run.json')
    check(run['id'] == int(run_id) and run['repository']['full_name'] == native.REPOSITORY and
          run['status'] == 'completed' and run['conclusion'] == 'success' and
          run['path'] == '.github/workflows/plink-candidate.yml', 'native/source run is not accepted')
    jobs = api(f"actions/runs/{run_id}/attempts/{run['run_attempt']}/jobs?per_page=100", evidence / 'native-jobs.json')
    passed = {job['name'] for job in jobs['jobs'] if job['conclusion'] == 'success'}
    check(jobs['total_count'] == len(jobs['jobs']) and JOBS <= passed, 'actual native four/source jobs are absent')
    listing = api(f'actions/runs/{run_id}/artifacts?per_page=100', evidence / 'native-artifacts.json')
    name = f"plink-native-{run_id}-{run['run_attempt']}-linux-x86_64"
    rows = [row for row in listing['artifacts'] if row['name'] == name]
    check(len(rows) == 1 and not rows[0]['expired'], 'native Linux original artifact is absent')
    row = rows[0]
    check(row['workflow_run']['head_sha'] == run['head_sha'], 'native artifact head differs')
    carrier = work / 'native.zip'
    api(f"actions/artifacts/{row['id']}/zip", carrier, binary=True)
    check(file_identity(carrier) == dict(bytes=row['size_in_bytes'], sha256=row['digest'].removeprefix('sha256:')),
          'native original ZIP differs')
    with zipfile.ZipFile(carrier) as archive:
        check(len(archive.infolist()) == 1 and archive.infolist()[0].file_size <= 512 * 2**20,
              'native original carrier membership differs')
        with archive.open(archive.infolist()[0]) as source, (work / 'native.tar').open('xb') as target:
            shutil.copyfileobj(source, target)
    extract_artifact(work / 'native.tar', work / 'native')
    candidate = work / 'native/candidate'
    selection = load(candidate / 'selection.json')
    check(selection['target'] == 'linux-x86_64' and selection['snapshot'] == 'green-1' and
          selection['controller_commit'] == run['head_sha'] and
          selection['source_archive_sha256'] == model['source_archive_sha256'] and
          selection['source_manifest_sha256'] == model['source_manifest_sha256'] and
          load(candidate / 'integrity-after.json')['failures'] == [], 'native and delivered source bindings differ')
    check(load(candidate / 'generations-release.json')['raw_cells_checked'] == 74878,
          'actual full native graph is absent')
    binary = file_identity(candidate / 'binary-release')
    check(load(candidate / 'binary-release.json')['sha256'] == binary['sha256'], 'native binary binding differs')
    save(evidence / 'native-admission.json', dict(qualified=True, run_id=int(run_id), head=run['head_sha'],
         attempt=run['run_attempt'], selector=selector, source_archive_sha256=model['source_archive_sha256'],
         source_manifest_sha256=model['source_manifest_sha256'], original_zip=file_identity(carrier),
         original_tar=file_identity(work / 'native.tar'), native_binary=binary,
         selection=selection, public_release=False))


def build(root, label, work, evidence):
    root, work, evidence = Path(root), Path(work), Path(evidence)
    output = evidence / label; output.mkdir()
    target = work / (label + '-target'); target.mkdir()
    cargo_home = work / (label + '-cargo-home'); cargo_home.mkdir()
    scratch = output / 'scratch'; scratch.mkdir()
    env = dict(os.environ, CARGO_HOME=str(cargo_home), CARGO_TARGET_DIR=str(target),
               TMPDIR=str(scratch), CARGO_INCREMENTAL='0', CC_ENABLE_DEBUG_OUTPUT='1')
    env.pop('GH_TOKEN', None)
    args = ['cargo', 'test', '--release', '--locked', '--offline', '-vv', '--',
            '--include-ignored', '--nocapture', '--test-threads=1']
    save(output / 'request.json', dict(argv=args, cwd=str(root / 'code'), cargo_home=str(cargo_home),
                                    target=str(target), scratch=str(scratch), network='Cargo offline'))
    with (output / 'stdout.txt').open('xb') as stdout, (output / 'stderr.txt').open('xb') as stderr:
        result = subprocess.run(args, cwd=root / 'code', env=env, stdout=stdout, stderr=stderr)
    save(output / 'status.json', dict(returncode=result.returncode))
    check(result.returncode == 0, 'offline delivered-source workflow failed: ' + label)
    text = (output / 'stdout.txt').read_text() + (output / 'stderr.txt').read_text()
    populations = [int(n) for n in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;', text)]
    check(sorted(populations) == [0, 0, 1, 4], 'existing whole test population differs')
    graphs = list(scratch.glob('plink-whole-native-*'))
    check(len(graphs) == 1, 'complete kept native graph absent')
    graph = graphs[0]
    save(output / 'generations.json', native.bind_graph(graph))
    binary = target / 'release/rsomics-plink'
    check(load(graph / 'identity.json')['binary_sha256'] == file_identity(binary)['sha256'],
          'whole workflow ran a different binary')
    objects = list((target / 'release/build').glob('rsomics-plink-*/out/*pgenlib_read*.o'))
    check(len(objects) == 1, 'actual reader object is absent or ambiguous')
    archive = objects[0].parent / 'librsomics_pgen_codec.a'
    proof = {}
    for name, path in (('reader.o', objects[0]), ('codec.a', archive), ('rsomics-plink', binary)):
        shutil.copy2(path, output / name)
        proof[name] = dict(**file_identity(path), marker_present=MARKER in path.read_bytes())
    shutil.copy2(objects[0].parent / 'compiler.txt', output / 'compiler.txt')
    save(output / 'objects.json', proof)
    save(output / 'source-after.json', source_tree(root))
    return proof, graph


def run(world, selector, work, evidence):
    hosted(work, evidence)
    work, evidence = Path(work), Path(evidence)
    admission = load(evidence / 'native-admission.json')
    check(admission['qualified'] is True and admission['selector'] == selector,
          'completed actual native/source admission required before rebuild')
    check(not os.environ.get('GH_TOKEN'), 'drop the API token before dependency build scripts execute')
    overrides = ('CARGO_BUILD_TARGET', 'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC_WRAPPER',
                 'RUSTC_WORKSPACE_WRAPPER', 'CXX', 'CXXFLAGS', 'CC', 'CFLAGS', 'CPPFLAGS', 'LDFLAGS')
    check(not any(os.environ.get(name) for name in overrides), 'undeclared compiler/target override')
    version = subprocess.run(['rustc', '-Vv'], text=True, capture_output=True, check=True).stdout
    check('release: 1.91.0\n' in version and 'host: x86_64-unknown-linux-gnu\n' in version,
          'pinned native Rust1.91 required')
    (evidence / 'rustc.txt').write_text(version)
    original = work / 'original'
    model = unpack_bundle(world, selector, original)
    check(admission['source_archive_sha256'] == model['source_archive_sha256'] and
          admission['source_manifest_sha256'] == model['source_manifest_sha256'], 'admission source differs')
    baseline, baseline_graph = build(original, 'base', work, evidence)
    verify_original(original)
    derivative = work / 'modified'
    patch = replace_codec(original, derivative)
    save(evidence / 'replacement.json', patch)
    modified, modified_graph = build(derivative, 'modified', work, evidence)
    verify_original(original); verify_derivative(derivative, model, patch)
    for name in ('reader.o', 'codec.a', 'rsomics-plink'):
        check(baseline[name]['sha256'] != modified[name]['sha256'] and
              baseline[name]['marker_present'] is False and modified[name]['marker_present'] is True,
              'actual linked codec substitution is unproven: ' + name)
    paths = scientific_paths(native.bind_graph(baseline_graph))
    check(paths == scientific_paths(native.bind_graph(modified_graph)), 'paired scientific membership differs')
    comparison = compare_scientific(baseline_graph, modified_graph, paths)
    save(evidence / 'scientific-equality.json', comparison)
    save(evidence / 'receipt.json', dict(status='private_offline_replace_codec_passed',
         source_archive_sha256=model['source_archive_sha256'], base=baseline, modified=modified,
         native_run_id=admission['run_id'], public_release=False, ordinary_macos_install_qualified=False,
         performance_claim=False, fixed_whole_workflow_preserved=True,
         exact_scientific_files_compared=comparison['file_count'],
         scientific_equality=file_identity(evidence / 'scientific-equality.json')))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['admit', 'run', 'package'])
    parser.add_argument('--bundle', required=True)
    parser.add_argument('--world', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--work', type=Path, required=True)
    parser.add_argument('--evidence', type=Path, required=True)
    parser.add_argument('--native-run')
    parser.add_argument('--archive', type=Path)
    args = parser.parse_args()
    hosted(args.work, args.evidence, minimum_free=0 if args.action == 'package' else 8 * 2**30)
    if args.action == 'admit': admit(args.world, args.bundle, args.native_run, args.work, args.evidence)
    elif args.action == 'run': run(args.world, args.bundle, args.work, args.evidence)
    else:
        runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
        check(args.archive is not None and args.archive.resolve().is_relative_to(runner), 'archive route differs')
        native.package(args.evidence, args.archive)


if __name__ == '__main__':
    main()
