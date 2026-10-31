"""Pinned age transport; callers own archive membership and scientific verification."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import signal
import subprocess
import sys
import tarfile
import urllib.request


ROOT = Path(__file__).resolve().parents[1]
TOOLS = ROOT / 'assets/private-artifact-tools-20261101.json'
HEADROOM = 8 * 2**30


class ArtifactError(RuntimeError):
    pass


def current_platform():
    name = (platform.system(), platform.machine())
    try:
        return {('Darwin', 'arm64'): 'darwin-arm64', ('Linux', 'x86_64'): 'linux-amd64'}[name]
    except KeyError:
        raise ArtifactError('operator platform has no pinned age distribution') from None


def _identity(path):
    path = Path(path)
    if not path.is_file() or path.is_symlink():
        raise ArtifactError('regular transport file required')
    with path.open('rb') as stream:
        return dict(bytes=path.stat().st_size, sha256=hashlib.file_digest(stream, 'sha256').hexdigest())


def _new(path):
    return os.fdopen(os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600), 'wb')


def _publish(partial, output):
    os.link(partial, output)
    partial.unlink()


def _download(spec, directory):
    output = directory / spec['name']
    if not output.exists():
        partial = Path(str(output) + '.partial')
        count = 0
        with urllib.request.urlopen(spec['url'], timeout=120) as response, _new(partial) as target:
            for block in iter(lambda: response.read(2**20), b''):
                count += len(block)
                if count > spec['bytes']:
                    raise ArtifactError('age distribution exceeds pinned size')
                target.write(block)
        if _identity(partial) != {key: spec[key] for key in ('bytes', 'sha256')}:
            raise ArtifactError('age distribution differs from pinned release')
        _publish(partial, output)
    if _identity(output) != {key: spec[key] for key in ('bytes', 'sha256')}:
        raise ArtifactError('cached age distribution differs from pinned release')
    return output


def _admit(directory, platform_name, extract=False):
    directory = Path(directory)
    model = json.loads(TOOLS.read_text())
    spec = model['platforms'][platform_name]
    archive = directory / spec['archive']['name']
    if _identity(archive) != {key: spec['archive'][key] for key in ('bytes', 'sha256')}:
        raise ArtifactError('age archive identity differs')
    binaries = {}
    with tarfile.open(archive, 'r:gz') as carrier:
        for name in ('age', 'age-keygen'):
            member = carrier.getmember('age/' + name)
            if not member.isfile() or member.size > 32 * 2**20:
                raise ArtifactError('age executable archive entry differs')
            body = carrier.extractfile(member).read()
            path = directory / name
            if extract and not path.exists():
                with _new(path) as output:
                    output.write(body)
                path.chmod(0o700)
            expected = dict(bytes=len(body), sha256=hashlib.sha256(body).hexdigest())
            if _identity(path) != expected:
                raise ArtifactError('age executable differs from its pinned archive')
            binaries[name] = expected
    version = subprocess.run([directory / 'age', '--version'], capture_output=True, timeout=10)
    if version.returncode != 0 or version.stdout.decode().strip() != model['version']:
        raise ArtifactError('age executable version differs')
    return model, spec, binaries


def prepare_tool(platform_name, directory):
    directory = Path(directory)
    directory.mkdir(parents=True, exist_ok=True, mode=0o700)
    if directory.is_symlink():
        raise ArtifactError('regular tool directory required')
    model = json.loads(TOOLS.read_text())
    spec = model['platforms'][platform_name]
    for name in ('archive', 'proof'):
        _download(spec[name], directory)
    _, _, binaries = _admit(directory, platform_name, extract=True)
    admission = dict(version=model['version'], platform=platform_name, archive=spec['archive'],
        proof=spec['proof'], executables=binaries, sigsum_proof_verified=False,
        verification='pinned release digests; no Sigsum verifier invoked')
    path = directory / 'admission.json'
    if path.exists():
        if json.loads(path.read_text()) != admission:
            raise ArtifactError('existing tool admission differs')
    else:
        with path.open('x') as output:
            json.dump(admission, output, sort_keys=True, indent=2); output.write('\n')
    return directory


def finalization_space(retained_bytes, archive_bytes, materialize_tar=False):
    if type(retained_bytes) is not int or type(archive_bytes) is not int or min(retained_bytes, archive_bytes) < 0:
        raise ValueError('nonnegative retained and TAR bounds required')
    ciphertext = 2**21 + archive_bytes + 16 + (archive_bytes // 65536 + 1) * 16
    additional = ciphertext + HEADROOM + (archive_bytes if materialize_tar else 0)
    return dict(ciphertext_bound_bytes=ciphertext, required_free_bytes=additional,
                peak_project_bytes=retained_bytes + additional, headroom_bytes=HEADROOM)


def _context(context):
    patterns = dict(repository=r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', product=r'rsomics-[a-z0-9-]+',
        head_sha=r'[0-9a-f]{40}', workflow_path=r'\.github/workflows/[A-Za-z0-9_.-]+\.ya?ml',
        phase=r'[a-z][a-z0-9-]{0,47}')
    context = dict(context or {})
    if context.keys() - (patterns.keys() | {'run_id', 'run_attempt'}):
        raise ValueError('public receipt contains an unapproved field')
    for key, value in context.items():
        valid = (type(value) is int and value > 0) if key in ('run_id', 'run_attempt') else (
            isinstance(value, str) and re.fullmatch(patterns[key], value))
        if not valid:
            raise ValueError('invalid public transport context')
    return context


def _output_paths(output):
    output = Path(output)
    partial = Path(str(output) + '.partial')
    if any(path.exists() or path.is_symlink() for path in (output, partial)):
        raise FileExistsError('fresh transport output required')
    return output, partial


def _stop(process):
    if process is not None and process.poll() is None:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.wait(timeout=10)


def encrypt(tool_dir, recipients, output, *, input_file=None, producer=None, plaintext_limit=None, context=None):
    context = _context(context)
    if (input_file is None) == (producer is None):
        raise ValueError('one existing file or producer command required')
    if input_file is not None:
        path = Path(input_file)
        if not path.is_file() or path.is_symlink():
            raise ArtifactError('regular finalized input required')
        plaintext_limit = path.stat().st_size
    if type(plaintext_limit) is not int or plaintext_limit < 0:
        raise ValueError('explicit nonnegative producer byte bound required')
    model, spec, _ = _admit(tool_dir, current_platform())
    recipient_identity = _identity(recipients)
    output, partial = _output_paths(output)
    source_process = age_process = source = None
    with _new(str(output) + '.age.stderr') as age_error, _new(str(output) + '.producer.stderr') as producer_error:
        try:
            if producer is not None:
                source_process = subprocess.Popen(producer, stdout=subprocess.PIPE, stderr=producer_error, start_new_session=True)
                source = source_process.stdout
            else:
                source = Path(input_file).open('rb')
            with _new(partial) as target:
                age_process = subprocess.Popen([Path(tool_dir) / 'age', '--encrypt', '--recipients-file', recipients],
                    stdin=subprocess.PIPE, stdout=target, stderr=age_error, start_new_session=True)
                count = 0
                for block in iter(lambda: source.read(2**20), b''):
                    count += len(block)
                    if count > plaintext_limit:
                        raise ArtifactError('producer exceeds admitted TAR bound')
                    age_process.stdin.write(block)
                age_process.stdin.close()
                if input_file is not None and count != plaintext_limit:
                    raise ArtifactError('finalized input changed during encryption')
                producer_exit = source_process.wait() if source_process else None
                age_exit = age_process.wait()
                if age_exit != 0 or producer_exit not in (None, 0):
                    raise ArtifactError('producer or age encryption failed')
                target.flush(); os.fsync(target.fileno())
            _publish(partial, output)
        except BrokenPipeError:
            raise ArtifactError('age encryption stopped before producer completion') from None
        finally:
            if source is not None:
                source.close()
            _stop(source_process); _stop(age_process)
    return dict(schema='rsomics.private-artifact.v1', status='sealed', context=context,
        ciphertext=_identity(output), recipient_sha256=recipient_identity['sha256'], age_version=model['version'],
        age_archive_sha256=spec['archive']['sha256'], producer_exit=producer_exit, age_exit=age_exit)


def decrypt(tool_dir, identity, input_file, output):
    _admit(tool_dir, current_platform())
    output, partial = _output_paths(output)
    with _new(str(output) + '.age.stderr') as error, _new(partial) as target:
        result = subprocess.run([Path(tool_dir) / 'age', '--decrypt', '--identity', identity, input_file],
                                stdout=target, stderr=error)
        if result.returncode != 0:
            raise ArtifactError('age decryption did not authenticate the complete input')
        target.flush(); os.fsync(target.fileno())
    _publish(partial, output)
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['tool', 'encrypt', 'decrypt', 'space'])
    parser.add_argument('--tool-dir', type=Path)
    parser.add_argument('--platform', default=current_platform())
    parser.add_argument('--recipients', type=Path); parser.add_argument('--identity', type=Path)
    parser.add_argument('--input', type=Path); parser.add_argument('--output', type=Path)
    parser.add_argument('--context', default='{}')
    parser.add_argument('--retained-bytes', type=int); parser.add_argument('--archive-bytes', type=int)
    parser.add_argument('--materialize-tar', action='store_true')
    args = parser.parse_args()
    try:
        if args.action == 'tool':
            prepare_tool(args.platform, args.tool_dir)
            result = dict(tool_admitted=True, sigsum_proof_verified=False)
        elif args.action == 'space':
            result = finalization_space(args.retained_bytes, args.archive_bytes, args.materialize_tar)
        elif args.action == 'encrypt':
            result = encrypt(args.tool_dir, args.recipients, args.output, input_file=args.input, context=json.loads(args.context))
        else:
            decrypt(args.tool_dir, args.identity, args.input, args.output)
            result = dict(decryption_complete=True, archive_validated=False)
        print(json.dumps(result, sort_keys=True))
        return 0
    except Exception:
        print('private artifact operation failed', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
