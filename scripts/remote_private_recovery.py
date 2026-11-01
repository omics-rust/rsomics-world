"""Authenticate a retained SSH carrier through local age without moving its identity."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shlex
import shutil
import signal
import stat
import subprocess
import threading
import time
import traceback

import private_artifact


REMOTE_PROGRAM = r'''
import hashlib,json,os,shutil,signal,stat,sys,time,zipfile
from pathlib import Path
os.umask(0o077)
action,spec=sys.argv[1],json.loads(sys.argv[2])
def expired(signum,frame):raise TimeoutError('remote recovery deadline reached')
signal.signal(signal.SIGALRM,expired);signal.alarm(spec['timeout_seconds'])
root=Path(spec['remote_root'])
assert root.is_dir() and root.resolve(strict=True)==root
def inside(name):
    path=Path(name)
    assert path.is_absolute() and path.resolve(strict=False)==path and path.is_relative_to(root)
    return path
carrier=inside(spec['zip_path']);attempt=inside(str(root/spec['attempt']))
def identity(path):
    before=path.lstat();assert stat.S_ISREG(before.st_mode)
    digest=hashlib.sha256();count=0
    with path.open('rb') as stream:
        for block in iter(lambda:stream.read(2**20),b''):digest.update(block);count+=len(block)
    after=path.lstat();assert (before.st_ino,before.st_size,before.st_mtime_ns)==(after.st_ino,after.st_size,after.st_mtime_ns)
    return dict(bytes=count,sha256=digest.hexdigest())
def save(name,value):
    with (attempt/name).open('x') as stream:
        json.dump(value,stream,indent=2,sort_keys=True);stream.write('\n');stream.flush();os.fsync(stream.fileno())
def check_carrier():
    assert identity(carrier)==spec['zip']
    with zipfile.ZipFile(carrier) as archive:
        members=archive.infolist()
        assert len(members)==2 and {m.filename for m in members}=={'payload.tar.age','receipt.json'}
        assert all(not m.is_dir() and stat.S_IFMT(m.external_attr>>16) in (0,stat.S_IFREG) for m in members)
        bound=2**21+spec['tar_bound_bytes']+16+(spec['tar_bound_bytes']//65536+1)*16
        assert 0<archive.getinfo('payload.tar.age').file_size<=bound
        assert archive.getinfo('receipt.json').file_size<=65536 and archive.testzip() is None
        receipt=json.loads(archive.read('receipt.json'))
        assert set(receipt)=={'schema','status','context','ciphertext','recipient_sha256','age_version','age_archive_sha256','producer_exit','age_exit'}
        assert receipt['schema']=='rsomics.private-artifact.v1' and receipt['status']=='sealed'
        assert receipt['context']==spec['context'] and receipt['recipient_sha256']==spec['recipient_sha256']
        assert receipt['age_version']==spec['age_version'] and receipt['age_archive_sha256']==spec['producer_age_archive_sha256']
        assert receipt['producer_exit']==receipt['age_exit']==0
        cipher=receipt['ciphertext'];info=archive.getinfo('payload.tar.age')
        assert set(cipher)=={'bytes','sha256'} and type(cipher['bytes']) is int and cipher['bytes']==info.file_size
        assert len(cipher['sha256'])==64 and all(c in '0123456789abcdef' for c in cipher['sha256'])
    return dict(zip=spec['zip'],receipt=receipt,device=root.stat().st_dev,free_bytes=shutil.disk_usage(root).free)
def capacity():
    assert shutil.disk_usage(root).free>=spec['minimum_free_bytes'],'remote recovery reserve unavailable'
if action=='inspect':
    capacity();print(json.dumps(check_carrier()))
elif action=='prepare':
    capacity();attempt.mkdir(mode=0o700)
    save('request.json',spec)
    fd=os.open(attempt/'candidate.tar',os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600);os.close(fd)
    print(json.dumps(dict(attempt=str(attempt),candidate=str(attempt/'candidate.tar'),device=root.stat().st_dev)))
elif action in ('source','sink'):
    assert json.loads((attempt/'request.json').read_bytes())==spec
    record=dict(action=action,complete=False,input_eof=False,bytes=0,sha256=None)
    digest=hashlib.sha256()
    try:
        if action=='source':
            admitted=check_carrier();expected=admitted['receipt']['ciphertext']
            with zipfile.ZipFile(carrier) as archive,archive.open('payload.tar.age') as stream:
                for block in iter(lambda:stream.read(2**20),b''):
                    record['bytes']+=len(block);assert record['bytes']<=expected['bytes']
                    digest.update(block);sys.stdout.buffer.write(block)
            sys.stdout.buffer.flush();record['input_eof']=True
            assert dict(bytes=record['bytes'],sha256=digest.hexdigest())==expected
        else:
            path=attempt/'candidate.tar'
            fd=os.open(path,os.O_WRONLY|os.O_NOFOLLOW)
            with os.fdopen(fd,'wb') as target:
                assert os.fstat(target.fileno()).st_size==0 and stat.S_IMODE(os.fstat(target.fileno()).st_mode)==0o600
                for block in iter(lambda:sys.stdin.buffer.read(2**20),b''):
                    assert record['bytes']+len(block)<=spec['tar_bound_bytes'],'plaintext exceeds admitted TAR ceiling'
                    assert shutil.disk_usage(root).free>=spec['free_floor_bytes']+len(block),'remote free floor reached'
                    target.write(block);digest.update(block);record['bytes']+=len(block)
                record['input_eof']=True;target.flush();os.fsync(target.fileno())
        record.update(complete=True,sha256=digest.hexdigest())
    except BaseException as error:
        record.update(error=type(error).__name__+': '+str(error),sha256=digest.hexdigest())
        raise
    finally:
        save(action+'-status.json',record)
    if action=='sink':print(json.dumps(record))
elif action=='finish':
    base={k:v for k,v in spec.items() if k not in ('expected_tar','pipeline_returncodes')}
    assert json.loads((attempt/'request.json').read_bytes())==base
    source=json.loads((attempt/'source-status.json').read_bytes());sink=json.loads((attempt/'sink-status.json').read_bytes())
    assert source['complete'] and source['input_eof'] and sink['complete'] and sink['input_eof']
    assert spec['pipeline_returncodes']==dict(source=0,age=0,sink=0)
    admitted=check_carrier();assert {k:source[k] for k in ('bytes','sha256')}==admitted['receipt']['ciphertext']
    expected=spec['expected_tar'];assert {k:sink[k] for k in expected}==expected
    assert identity(attempt/'candidate.tar')==expected
    record=dict(complete_input_authenticated=True,archive_validated=False,tar=expected,
        remote_tar=str(attempt/'candidate.tar'),zip=spec['zip'],context=spec['context'],
        pipeline_returncodes=spec['pipeline_returncodes'],scientific_execution=False)
    save('AUTHENTICATED.json',record)
    fd=os.open(attempt,os.O_RDONLY|os.O_DIRECTORY);os.fsync(fd);os.close(fd)
    print(json.dumps(record))
elif action=='status':
    result={}
    for name in ('request.json','source-status.json','sink-status.json','AUTHENTICATED.json'):
        path=attempt/name
        if path.exists():
            assert path.is_file() and not path.is_symlink() and path.stat().st_size<2**20
            result[name]=json.loads(path.read_bytes())
    print(json.dumps(result))
else:raise ValueError('unknown recovery action')
'''


class RecoveryError(RuntimeError):
    pass


def _save(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write('\n')


def _validate(request):
    required = {'host', 'remote_python', 'remote_root', 'zip_path', 'attempt', 'zip', 'context',
        'recipient_sha256', 'age_version', 'producer_age_archive_sha256', 'tar_bound_bytes',
        'retained_bound_bytes', 'minimum_free_bytes', 'free_floor_bytes', 'local_headroom_bytes',
        'timeout_seconds', 'status_timeout_seconds', 'log_limit_bytes'}
    if set(request) != required or not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_.-]*', request['host']):
        raise RecoveryError('explicit complete recovery request and SSH host alias required')
    if not re.fullmatch(r'[A-Za-z0-9_./-]+', request['remote_python']) or not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_.-]{0,95}', request['attempt']):
        raise RecoveryError('invalid interpreter or fresh attempt name')
    for name in ('remote_root', 'zip_path'):
        path = Path(request[name])
        if not path.is_absolute() or '..' in path.parts or '://' in request[name] or any(ord(c) < 32 for c in request[name]):
            raise RecoveryError('absolute canonical remote task paths required')
    if not Path(request['zip_path']).is_relative_to(Path(request['remote_root'])):
        raise RecoveryError('retained carrier must belong to the recovery task directory')
    private_artifact._context(request['context'])
    if set(request['context']) != {'repository', 'product', 'run_id', 'run_attempt', 'head_sha', 'workflow_path', 'phase'}:
        raise RecoveryError('full original workflow context required')
    for value in (request['zip']['sha256'], request['recipient_sha256'], request['producer_age_archive_sha256']):
        if not re.fullmatch(r'[0-9a-f]{64}', value):
            raise RecoveryError('explicit original digest required')
    for key in ('tar_bound_bytes', 'retained_bound_bytes', 'minimum_free_bytes', 'free_floor_bytes',
                'local_headroom_bytes', 'timeout_seconds', 'status_timeout_seconds', 'log_limit_bytes'):
        if type(request[key]) is not int or request[key] <= 0:
            raise RecoveryError('positive caller-admitted recovery bounds required')
    if set(request['zip']) != {'bytes', 'sha256'} or type(request['zip']['bytes']) is not int or request['zip']['bytes'] <= 0:
        raise RecoveryError('exact retained whole ZIP identity required')
    if request['minimum_free_bytes'] < request['free_floor_bytes'] + request['tar_bound_bytes'] + request['retained_bound_bytes']:
        raise RecoveryError('remote reserve must include TAR, subsequent body custody and independent floor')


def _external_evidence(path):
    if platform.system() == 'Darwin':
        import plistlib
        anchors = [Path('/Volumes/KIOXIA'), Path("/Volumes/Zane's HDD")]
        physical = path.parent.resolve(strict=True)
        anchor = next((root for root in anchors if physical.is_relative_to(root)), None)
        if anchor is None:
            raise RecoveryError('Mac recovery evidence must be on the allowed external disks')
        info = plistlib.loads(subprocess.check_output(['/usr/sbin/diskutil', 'info', '-plist', str(anchor)]))
        if info.get('Internal') is not False or info.get('MountPoint') != str(anchor) or physical.stat().st_dev != anchor.stat().st_dev:
            raise RecoveryError('actual external recovery device differs')


def _remote_command(request, action):
    command = [request['remote_python'], '-B', '-c', REMOTE_PROGRAM, action, json.dumps(request, separators=(',', ':'))]
    return ['ssh', '-o', 'BatchMode=yes', '-o', 'ConnectTimeout=10', '-o', 'ServerAliveInterval=10',
            '-o', 'ServerAliveCountMax=3', '--', request['host'], shlex.join(command)]


def _age_command(tool_dir, identity_path, request):
    model, _, _ = private_artifact._admit(tool_dir, private_artifact.current_platform())
    if model['version'] != request['age_version'] or not identity_path.is_file() or identity_path.is_symlink():
        raise RecoveryError('pinned age or existing identity path differs')
    if stat.S_IMODE(identity_path.stat().st_mode) & 0o077:
        raise RecoveryError('private identity path has group or other permissions')
    return [str(Path(tool_dir) / 'age'), '--decrypt', '--identity', str(identity_path)]


def _stop(process):
    if process is not None and process.poll() is None:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.wait(timeout=10)


def _limits(directory, request, deadline):
    if time.monotonic() >= deadline or shutil.disk_usage(directory).free < request['local_headroom_bytes']:
        raise RecoveryError('recovery time or local external headroom bound reached')
    if any(path.stat().st_size > request['log_limit_bytes'] for path in directory.glob('*.txt')):
        raise RecoveryError('recovery raw-log bound reached')


def _capture(directory, name, command, request, deadline):
    _save(directory / (name + '-command.json'), command)
    process = None
    with (directory / (name + '.stdout.txt')).open('xb') as output, (directory / (name + '.stderr.txt')).open('xb') as error:
        try:
            process = subprocess.Popen(command, stdout=output, stderr=error, start_new_session=True)
            while process.poll() is None:
                _limits(directory, request, deadline)
                time.sleep(.05)
            if process.returncode != 0:
                raise RecoveryError(name + ' subprocess failed')
            _limits(directory, request, deadline)
        finally:
            _stop(process)
            _save(directory / (name + '-status.json'), dict(returncode=process.returncode if process else None))
    return json.loads((directory / (name + '.stdout.txt')).read_bytes())


def _pipeline(directory, request, age_command, state, deadline):
    processes, handles, threads, streams = {}, [], [], {}

    def pump(name, source, target, limit):
        count, digest = 0, hashlib.sha256()
        try:
            for block in iter(lambda: source.read(2**20), b''):
                count += len(block)
                if count > limit:
                    raise RecoveryError(name + ' byte bound reached')
                digest.update(block)
                target.write(block)
            target.flush()
            streams[name] = dict(bytes=count, sha256=digest.hexdigest(), input_eof=True)
        except BaseException as error:
            streams[name] = dict(bytes=count, sha256=digest.hexdigest(), input_eof=False, error=type(error).__name__ + ': ' + str(error))
        finally:
            source.close()
            try:
                target.close()
            except OSError as error:
                streams[name].update(input_eof=False, error=type(error).__name__ + ': ' + str(error))

    try:
        commands = dict(sink=_remote_command(request, 'sink'), age=age_command,
                        source=_remote_command(request, 'source'))
        _save(directory / 'pipeline-commands.json', dict(source=commands['source'], sink=commands['sink'],
            age='Pinned local age --decrypt --identity <existing private path>; identity bytes never handled by controller'))
        for name in ('sink', 'age', 'source'):
            error = (directory / (name + '.stderr.txt')).open('xb'); handles.append(error)
            output = subprocess.PIPE
            if name == 'sink':
                output = (directory / 'sink.stdout.txt').open('xb'); handles.append(output)
            processes[name] = subprocess.Popen(commands[name], stdin=subprocess.PIPE if name != 'source' else subprocess.DEVNULL,
                stdout=output, stderr=error, start_new_session=True)
        links = [('ciphertext', processes['source'].stdout, processes['age'].stdin, state['ciphertext']['bytes']),
                 ('plaintext', processes['age'].stdout, processes['sink'].stdin, request['tar_bound_bytes'])]
        for name, source, target, limit in links:
            thread = threading.Thread(target=pump, args=(name, source, target, limit), daemon=True)
            threads.append(thread); thread.start()
        while any(process.poll() is None for process in processes.values()) or any(thread.is_alive() for thread in threads):
            _limits(directory, request, deadline)
            if any(process.poll() not in (None, 0) for process in processes.values()) or any('error' in row for row in tuple(streams.values())):
                raise RecoveryError('source, age or sink failed; original attempt remains unauthenticated')
            time.sleep(.05)
        if any(process.returncode != 0 for process in processes.values()) or any('error' in row for row in tuple(streams.values())):
            raise RecoveryError('source, age or sink failed; original attempt remains unauthenticated')
        if not all(streams[name]['input_eof'] for name in ('ciphertext', 'plaintext')):
            raise RecoveryError('complete source and plaintext EOF required')
        if {key: streams['ciphertext'][key] for key in ('bytes', 'sha256')} != state['ciphertext']:
            raise RecoveryError('streamed ciphertext differs from original receipt')
        state['tar'] = {key: streams['plaintext'][key] for key in ('bytes', 'sha256')}
        sink = json.loads((directory / 'sink.stdout.txt').read_bytes())
        if not sink['complete'] or not sink['input_eof'] or {key: sink[key] for key in state['tar']} != state['tar']:
            raise RecoveryError('remote sink and local complete plaintext stream differ')
    finally:
        for process in processes.values():
            _stop(process)
        for thread in threads:
            thread.join(timeout=10)
        for handle in handles:
            handle.close()
        state['pipeline_returncodes'] = {name: processes[name].returncode if name in processes else None for name in ('source', 'age', 'sink')}
        state['streams'] = streams
        _save(directory / 'pipeline-status.json', dict(returncodes=state['pipeline_returncodes'], streams=streams,
            ciphertext_stdout_retained_in_original_zip=True,
            plaintext_stdout_destination='remote candidate; byte bounds or sink failure can leave only a prefix'))


def recover(request, evidence, tool_dir, identity_path):
    request = dict(request)
    _validate(request)
    directory, identity_path = Path(evidence), Path(identity_path)
    _external_evidence(directory)
    os.umask(0o077)
    directory.mkdir(mode=0o700)
    _save(directory / 'request.json', request)
    (directory / 'remote-program.py').write_text(REMOTE_PROGRAM)
    (directory / 'controller.py').write_bytes(Path(__file__).read_bytes())
    state = dict(complete_input_authenticated=False, archive_validated=False, scientific_execution=False,
                 pipeline_returncodes=dict(source=None, age=None, sink=None))
    deadline = time.monotonic() + request['timeout_seconds']
    try:
        _limits(directory, request, deadline)
        age_command = _age_command(tool_dir, identity_path, request)
        admitted = _capture(directory, 'inspect', _remote_command(request, 'inspect'), request, deadline)
        state['original'] = admitted
        state['ciphertext'] = admitted['receipt']['ciphertext']
        _capture(directory, 'prepare', _remote_command(request, 'prepare'), request, deadline)
        _pipeline(directory, request, age_command, state, deadline)
        finish = dict(request, expected_tar=state['tar'], pipeline_returncodes=state['pipeline_returncodes'])
        result = _capture(directory, 'finish', _remote_command(finish, 'finish'), request, deadline)
        if not result['complete_input_authenticated'] or result['archive_validated'] or result['tar'] != state['tar']:
            raise RecoveryError('remote final authentication binding differs')
        state.update(result)
        return state
    except BaseException:
        state['failure'] = traceback.format_exc()
        raise
    finally:
        try:
            _capture(directory, 'remote-snapshot', _remote_command(request, 'status'), request,
                     time.monotonic() + request['status_timeout_seconds'])
        except BaseException as error:
            state['status_snapshot_failure'] = type(error).__name__ + ': ' + str(error)
        _save(directory / 'status.json', state)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--request', type=Path, required=True)
    parser.add_argument('--evidence', type=Path, required=True)
    parser.add_argument('--tool-dir', type=Path, required=True)
    parser.add_argument('--identity', type=Path, required=True)
    args = parser.parse_args()
    recover(json.loads(args.request.read_bytes()), args.evidence, args.tool_dir, args.identity)


if __name__ == '__main__':
    main()
