"""Qualified full-six warm-cache performance controller; no rebuild or implicit receipt."""

import argparse
import gzip
import hashlib
import importlib.metadata
import io
import json
import math
import os
from pathlib import Path
import platform
import posixpath
import re
import resource
import shlex
import shutil
import signal
import socket
import sqlite3
import stat
import statistics
import struct
import subprocess
import sys
import tarfile
import time
import uuid
import urllib.request
import zipfile

from infercnv_source_snapshot import _hash_file, read_snapshot_manifest
from verify_infercnv_oracle_artifact import _archive_name, _artifact, _run
import signal_first_workflow_oracle as oracle

REPOSITORY = 'omics-rust/rsomics-world'
TARGETS = ['linux-x86_64', 'linux-aarch64', 'macos-x86_64', 'macos-aarch64']
CASES = ('main', 'track20', 'atac')
PREFIX = b'rsomics-signal-perf-v1\0'
GIB = 2**30
MEMORY_LIMIT = 16 * GIB
CAMPAIGN_LIMIT = 160 * GIB
HEADROOM = 32 * GIB
LIMITS = dict(main=5*GIB, track20=GIB, atac=GIB)
LOCK_SHA = '763a0eae15f15315f001ffb56dd261710f862362d5a3da6d94b107f13c022bc9'
PRODUCER_SHA = '39668a86b2fa084d8520e7f63f85bb837a2857c527de56917f9baef4ca107b57'
INPUT = dict(repository=REPOSITORY, workflow_path='.github/workflows/rnaseq-qc-performance.yml',
    run_id=40592936910, run_attempt=1, head_sha='5fcb3820198c8fe8fd3e463a609b276df7ae590a',
    artifact_id=12781035705, artifact_name='rnaseq-qc-performance-whole-40592936910-1',
    size_in_bytes=492933159, artifact_sha256='2072ae441c3a2fca21a160db385b612c5596dfb594c50e0cd649edf68c3284a7')
INPUT_FILES = {
    'benchmark/inputs/paired.bam': (437662663, '38174082a8f36ae76a6852b9b970fcb334ab5c8186c133993a0fd652d1d147cd'),
    'benchmark/inputs/paired.bam.bai': (2723112, '05076af3c06906d3ecdf1c448e0f0b5b2d4e1243a3d63f585c004d772f782957'),
    'benchmark/inputs/genes.bed12': (9018094, '5132086afd5b33bb4d5b2724ef5c4e8275aaba7854a7ec5af0d1902171b88ada'),
    'benchmark/inputs/originals/hg19_RefSeq.bed.gz': (2567192, 'eb7070cfb52b9011c132346fd27baffb8a7acf3c034ebf996cebd52fbb52aa19'),
    'benchmark/inputs/annotation-exclusions.jsonl': (710759, '9023c977ca350d191092b5175168e47f8f54f427763b1a1d0dbfb136e82c7ae2'),
    'benchmark/inputs/qualification.json': (None, '76f51a277a3582b507dbdc3c6e2c2e90d96334d457c8b12a03f22890319bc599'),
    'benchmark/inputs/record-counts.json': (None, '379c2017a65e7d42a297b906e2643e17427567d017ec0f03ae4c735ae939bf89'),
    'benchmark/inputs/bam-header.sam': (913, '6fd9c063620ede1950108741d054e6920a6864a0bc0e6417dfdf81a6e8d42dca'),
    'benchmark/inputs.json': (None, 'a1fc9eb186ffdc83099b46b85cb7ff168d8ec0c18711368adfdd4c26d962a3e1'),
    'benchmark/artifact-inputs.json': (None, '4e3822881a1c7f43bd6a5cacccc36bc161ca3fd44b9d89c95965b0a2fca857ad'),
}
CONTROLLERS = ('signal_real_performance.py', 'signal_real_performance_check.py',
               'test_signal_real_performance.py', 'signal_first_workflow_oracle.py',
               'signal_candidate.py', 'signal_source_on_native.py', 'infercnv_source_snapshot.py',
               'verify_infercnv_oracle_artifact.py')
SSH_PREFIX = Path('/home/runner/work/_temp/signal-evidence/runtime')
IMAGE_ASSET = 'assets/signal-ssh-image-20261101.json'


def execution_policy(idle_confirmed, shared, context):
    check(not (idle_confirmed and shared), 'shared execution cannot claim an idle host')
    if shared:
        check(context.get('kind')=='ssh-docker-shared' and context.get('provider_admitted') is True,
              'HELD: genuine SSH context and provider admission required before shared campaign')
        return 'shared/non-exclusive; original idle-host and isolated scalability gates unearned'
    check(idle_confirmed, 'otherwise-idle host must be explicitly confirmed')
    return 'operator-confirmed otherwise-idle host'


def verify_runtime_carrier(carrier, destination=None):
    if destination is not None:
        destination=Path(destination)
        check(destination==SSH_PREFIX, 'restore must retain the exact accepted runtime prefix')
        if destination.exists() or destination.is_symlink():
            raise FileExistsError('fresh runtime prefix required; existing data is never replaced')
    with tarfile.open(carrier,'r:') as archive:
        members=archive.getmembers(); by_name={row.name:row for row in members}
        check(len(members)==len(by_name)<=100000, 'duplicate/oversized original TAR inventory')
        for row in members:
            check(_archive_name(row.name,False)==row.name and row.type in (tarfile.REGTYPE,tarfile.DIRTYPE,tarfile.SYMTYPE),
                  'unsafe original native TAR member')
            for parent in Path(row.name).parents:
                if str(parent)!='.':check(str(parent) in by_name and by_name[str(parent)].isdir(), 'TAR parent is not a directory')
        index=by_name['signal-evidence/artifact-manifest.json']
        check(index.isfile() and index.size<16*2**20, 'original source manifest absent or oversized')
        model=json.load(archive.extractfile(index))
        check(model['schema']=='rsomics.signal.source-artifact.v1', 'original source manifest schema differs')
        expected=model['files']; identities={}; links={}
        def resolve(name, seen=()):
            check(name in by_name and name not in seen and len(seen)<32, 'original link escapes carrier or cycles')
            row=by_name[name]
            if not row.issym():return row
            target=row.linkname
            if target.startswith('/'):
                path=Path(target)
                check(path.is_relative_to(SSH_PREFIX.parent), 'absolute original link escapes accepted prefix')
                target='signal-evidence/'+path.relative_to(SSH_PREFIX.parent).as_posix()
            else:target=posixpath.normpath(posixpath.join(posixpath.dirname(name),target))
            check(target=='signal-evidence' or target.startswith('signal-evidence/'), 'relative original link escapes carrier')
            return resolve(target,(*seen,name))
        for row in members:
            if row.isfile():
                with archive.extractfile(row) as stream:
                    identities[row.name]=dict(bytes=row.size,sha256=hashlib.file_digest(stream,'sha256').hexdigest())
            elif row.issym():links[row.name]=resolve(row.name).name
        observed={name.removeprefix('signal-evidence/'):value for name,value in identities.items()
                  if Path(name).name!='artifact-manifest.json'}
        for name,target in links.items():
            if target in identities:observed[name.removeprefix('signal-evidence/')]=identities[target]
        check(observed==expected, 'complete original TAR manifest membership/body differs')
        runtime={row.name:row for row in members if row.name.startswith('signal-evidence/runtime/')}
        check(runtime, 'original runtime missing')
        for name,row in runtime.items():
            check(not row.mode&0o7000, 'privileged runtime mode refused')
            if row.issym():check(links[name].startswith('signal-evidence/runtime/'), 'runtime link leaves restored runtime')
        if destination is not None:
            check(destination.parent.resolve(strict=True)==destination.parent, 'runtime parent must not traverse links')
            destination.mkdir(mode=0o700)
            for name,row in runtime.items():
                path=destination/name.removeprefix('signal-evidence/runtime/')
                path.parent.mkdir(parents=True,exist_ok=True,mode=0o700)
                if row.isdir():path.mkdir(exist_ok=True,mode=0o700)
                elif row.isfile():
                    with archive.extractfile(row) as source,path.open('xb') as output:shutil.copyfileobj(source,output,1024*1024)
                    path.chmod(row.mode)
            for name,row in runtime.items():
                if row.issym():
                    path=destination/name.removeprefix('signal-evidence/runtime/')
                    path.symlink_to(row.linkname)
            for name,row in runtime.items():
                path=destination/name.removeprefix('signal-evidence/runtime/')
                if not row.isdir() and not (row.issym() and by_name[links[name]].isdir()):
                    check(_hash_file(path)==observed[name.removeprefix('signal-evidence/')]['sha256'], 'restored runtime body differs')
            for name,row in runtime.items():
                if row.isdir():(destination/name.removeprefix('signal-evidence/runtime/')).chmod(row.mode)
            destination.chmod(by_name['signal-evidence/runtime'].mode)
        return dict(verified_manifest_entries=len(observed),runtime_members=len(runtime),
                    carrier_sha256=_hash_file(carrier),original_prefix=str(SSH_PREFIX),
                    self_manifest_exceptions=[name for name in identities if Path(name).name=='artifact-manifest.json'],
                    restored=destination is not None,scientific_execution=False)


def check(condition, message):
    if not condition:
        raise ValueError(message)


def save(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('x') as out:
        json.dump(value, out, indent=2, sort_keys=True, allow_nan=False)
        out.write('\n')


def load(path):
    with Path(path).open() as stream:
        return json.load(stream)


def manifest(root):
    root = Path(root)
    rows = {}
    for path in sorted(root.rglob('*')):
        if path.is_dir() and not path.is_symlink():
            continue
        check(not path.is_symlink() and path.is_file(), 'nonregular evidence member: '+str(path))
        rows[path.relative_to(root).as_posix()] = dict(bytes=path.stat().st_size, sha256=_hash_file(path))
    return rows


def tree_bytes(root):
    total=0
    for path in Path(root).rglob('*'):
        try:
            observed=path.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISREG(observed.st_mode):
            total+=observed.st_size
    return total


def checked_paths(root):
    root = Path(root).resolve(strict=True)
    check(root != Path('/') and not root.is_symlink(), 'invalid campaign root')
    for name in ('TMPDIR', 'XDG_CACHE_HOME', 'PIP_CACHE_DIR', 'PYTHONUSERBASE', 'MPLCONFIGDIR', 'EVIDENCE'):
        path = Path(os.environ[name]).resolve(strict=True)
        check(path.is_relative_to(root) and path != root and path.stat().st_dev == root.stat().st_dev,
              'campaign write/cache route escapes root: '+name)
    return root


def cpu_set(text):
    values = [int(v) for v in text.split(',')]
    check(len(values) == len(set(values)) == 4 and all(n >= 0 for n in values), 'four distinct explicit logical CPUs required')
    return values


def meminfo(path='/proc/meminfo'):
    values = {}
    for line in Path(path).read_text().splitlines():
        key, value = line.split(':', 1)
        parts = value.split()
        values[key] = int(parts[0]) * (1024 if len(parts)>1 and parts[1]=='kB' else 1)
    check('MemAvailable' in values, 'actual available-memory counter absent')
    return values


def resource_plan(cpus, allowed, memory, free):
    check(set(cpus) <= set(allowed) and len(cpus) == len(set(cpus)) == 4, 'explicit CPU set is not actually allocated')
    check(memory['MemAvailable'] >= GIB, 'insufficient currently available memory for qualification')
    check(free >= HEADROOM, 'less than32GiB live storage headroom')
    return dict(cpus=list(cpus), memory_abort_bytes=MEMORY_LIMIT, campaign_abort_bytes=CAMPAIGN_LIMIT,
        minimum_free_bytes=HEADROOM, minimum_available_bytes=512*2**20,
        sampled_rss_scope='sum of live owned process-group RSS; shared pages may be counted more than once; sampled, not exact peak',
        exact_tree_peak='optional predelegated cgroup only; otherwise unavailable')


def setup(root, cpus, execution_ledger=None):
    root = Path(root).absolute()
    check(not root.exists() and root.parent.resolve(strict=True) == root.parent, 'new real campaign root required')
    check(platform.system()=='Linux' and platform.machine()=='x86_64', 'real execution requires Linux x86_64')
    values = cpu_set(cpus)
    plan = resource_plan(values, os.sched_getaffinity(0), meminfo(), shutil.disk_usage(root.parent).free)
    context=validate_ssh_context(load(execution_ledger),root) if execution_ledger is not None else None
    root.mkdir()
    paths = {name:root/relative for name,relative in {
        'EVIDENCE':'evidence', 'TMPDIR':'scratch', 'XDG_CACHE_HOME':'cache',
        'PIP_CACHE_DIR':'cache/pip', 'PYTHONUSERBASE':'python-user', 'MPLCONFIGDIR':'cache/matplotlib',
        'CARGO_HOME':'unused-cargo-home', 'CARGO_TARGET_DIR':'unused-cargo-target', 'RUSTUP_HOME':'unused-rustup-home'}.items()}
    for path in paths.values():
        path.mkdir(parents=True, exist_ok=True)
    if execution_ledger is not None:
        save(root/'evidence/execution-context.json',context)
        paths['HOME']=root/'user-home';paths['HOME'].mkdir()
    environment = dict(paths, RUNNER_TEMP=root, PYTHONDONTWRITEBYTECODE='1', PYTHONNOUSERSITE='1',
        PIP_CONFIG_FILE='/dev/null', PIP_DISABLE_PIP_VERSION_CHECK='1', PIP_NO_INPUT='1')
    environment.update({name:'1' for name in oracle.THREADS})
    with (root/'environment.sh').open('x') as out:
        for name,value in environment.items():
            out.write('export '+name+'='+shlex.quote(str(value))+'\n')
    save(root/'host-plan.json', dict(plan=plan, platform=platform.platform(), uname=list(platform.uname()),
        available_memory=meminfo(), disk=shutil.disk_usage(root)._asdict(),
        affinity=sorted(os.sched_getaffinity(0)), durability_claim=False, cache='warm-cache only'))
    return root


def image_authority(world):
    row=load(Path(world)/IMAGE_ASSET)
    check(row['schema']=='rsomics.signal.ssh-image.v1' and row['platform']=='linux/amd64', 'image authority differs')
    for name in ('index','manifest','config'):
        check('sha256:'+hashlib.sha256(row['raw'][name].encode()).hexdigest()==row[name+'_digest'], 'official image metadata body differs')
    index,manifest,config=[json.loads(row['raw'][name]) for name in ('index','manifest','config')]
    check(any(x['digest']==row['manifest_digest'] and x['platform']==dict(os='linux',architecture='amd64') for x in index['manifests']) and
          manifest['config']['digest']==row['config_digest'] and config['os']=='linux' and config['architecture']=='amd64',
          'official image platform/config chain differs')
    return row


def record_ssh_context(parent, world, container, cpus, output):
    check(platform.system()=='Linux' and platform.machine()=='x86_64' and os.environ.get('SSH_CONNECTION'),
          'HELD: run context capture in the actual Linux SSH session')
    ancestors=[];pid=os.getppid()
    for _ in range(32):
        path=Path('/proc')/str(pid)
        ancestors.append((path/'comm').read_text().strip())
        text=(path/'stat').read_text();pid=int(text[text.rfind(')')+2:].split()[1])
        if pid<=1:break
    check(any(name.startswith('sshd') for name in ancestors), 'HELD: no actual sshd ancestry')
    check(re.fullmatch('[0-9a-f]{12,64}',container), 'actual Docker container ID required')
    parent=Path(parent).resolve(strict=True)
    check(parent.stat().st_uid==os.getuid() and parent.stat().st_mode&0o077==0, 'private user-owned host work parent required')
    raw=subprocess.check_output(['docker','inspect','--type','container',container]);items=json.loads(raw)
    check(len(items)==1 and items[0]['State']['Running'], 'HELD: existing running task container required')
    actual=items[0];image_raw=subprocess.check_output(['docker','image','inspect',actual['Image']]);image=json.loads(image_raw)[0]
    authority=image_authority(world);base=json.loads(authority['raw']['config'])['rootfs']['diff_ids']
    check(image['RootFS']['Layers'][:len(base)]==base and image['Config']['Labels'].get('org.rsomics.signal.base-manifest')==authority['manifest_digest'] and
          image['Config']['Labels'].get('org.rsomics.signal.recipe-sha256')==_hash_file(Path(world)/'assets/signal-ssh.Dockerfile'),
          'derivative image does not retain the admitted official base')
    config=actual['HostConfig'];mounts={row['Destination']:row for row in actual['Mounts']}
    check(not {value.split('=',1)[0] for value in actual['Config']['Env']} &
          {'GITHUB_ACTIONS','RUNNER_ENVIRONMENT','GITHUB_SHA','GH_TOKEN','GITHUB_TOKEN'},
          'scientific container must not contain Actions identity flags or API credentials')
    check(not config['Privileged'] and config['ReadonlyRootfs'] and not config.get('PidMode') and not config.get('IpcMode')=='host' and
          config.get('NetworkMode')=='none' and 'ALL' in (config.get('CapDrop') or []) and
          any(value.startswith('no-new-privileges') for value in config.get('SecurityOpt') or []),
          'private nonprivileged read-only-root container required')
    check(set(cpu_set(config['CpusetCpus']))==set(cpu_set(cpus)), 'actual Docker CPU binding differs')
    check(mounts[str(SSH_PREFIX.parent.parent)]['RW'] and Path(mounts[str(SSH_PREFIX.parent.parent)]['Source']).resolve()==parent and
          all(name in mounts and not mounts[name]['RW'] for name in ('/work/world','/capsule')),
          'actual work/world/capsule Docker mounts differ')
    check(not any(row['Destination']=='/var/run/docker.sock' for row in actual['Mounts']), 'Docker socket must not enter scientific container')
    originals={}
    for name,body in (('ssh-container-inspect.json',raw),('ssh-image-inspect.json',image_raw)):
        path=Path(output).parent/name
        with path.open('xb') as stream:stream.write(body)
        originals[name]=dict(bytes=len(body),sha256=hashlib.sha256(body).hexdigest())
    save(output,dict(schema='rsomics.signal.execution-context.v1',kind='ssh-docker-shared',execution_id=str(uuid.uuid4()),
        host=platform.node(),boot_id=Path('/proc/sys/kernel/random/boot_id').read_text().strip(),
        ssh_connection=os.environ['SSH_CONNECTION'],sshd_ancestry=ancestors,host_uid=os.getuid(),
        controller=image_and_controller(world),
        host_parent=str(parent),container=actual,image=image,inspect_originals=originals,image_authority_sha256=_hash_file(Path(world)/IMAGE_ASSET),
        cpus=cpu_set(cpus),provider_admitted=False,original_idle_host_gate_earned=False,
        scientific_execution=False,source_of_truth='actual SSH process ancestry and existing Docker daemon inspect; not a GitHub execution'))


def validate_ssh_context(context, root):
    check(context.get('schema')=='rsomics.signal.execution-context.v1' and context.get('kind')=='ssh-docker-shared' and
          str(uuid.UUID(context['execution_id']))==context['execution_id'], 'genuine SSH execution ledger required')
    check(not any(os.environ.get(name) for name in ('GITHUB_ACTIONS','RUNNER_ENVIRONMENT','GITHUB_SHA','GH_TOKEN','GITHUB_TOKEN')),
          'SSH scientific context must not impersonate Actions or contain an API token')
    check(Path(root)==SSH_PREFIX.parent and socket.gethostname()==context['container']['Config']['Hostname'] and
          Path('/proc/sys/kernel/random/boot_id').read_text().strip()==context['boot_id'] and
          os.getuid()==int(context['container']['Config']['User'].split(':')[0]), 'actual container/prefix/host identity differs')
    check(set(os.sched_getaffinity(0))==set(context['cpus']), 'actual scientific container affinity differs')
    return context


def native_carrier(root):
    target=root/'original.tar'
    with zipfile.ZipFile(root/'artifact.zip') as zipped:
        rows=zip_members(zipped);files=[row for row in rows if not row.is_dir()]
        check(len(files)==1 and files[0].filename.endswith('.tar') and zipped.testzip() is None, 'original native carrier differs')
        with zipped.open(files[0]) as source,target.open('xb') as output:shutil.copyfileobj(source,output,1024*1024)
    return target


def seal_capsule(output,world,selector,native_zip,input_zip):
    output=Path(output);output.mkdir(mode=0o700)
    receipt,adoption=load_receipt(world,selector)
    for name,authority,original in (('native',receipt,native_zip),('input',INPUT,input_zip)):
        acquire_artifact(output/name,authority,name=='native',original_zip=original)
    extract_native(output/'native',receipt,world)
    proof=verify_runtime_carrier(native_carrier(output/'native'))
    with zipfile.ZipFile(output/'input/artifact.zip') as archive:
        check(len(zip_members(archive))==521 and archive.testzip() is None, 'original input carrier differs')
        for name,(size,sha) in INPUT_FILES.items():
            with archive.open(name) as stream:actual=hashlib.file_digest(stream,'sha256').hexdigest()
            check(actual==sha and (size is None or archive.getinfo(name).file_size==size), 'original input body differs')
    save(output/'capsule.json',dict(schema='rsomics.signal.ssh-capsule.v1',receipt=receipt,receipt_adoption=adoption,
        runtime=proof,controller=image_and_controller(world),files=manifest(output),scientific_execution=False))


def image_and_controller(world):
    world=Path(world)
    return dict(head=subprocess.check_output(['git','-C',str(world),'rev-parse','HEAD'],text=True).strip(),
                image_authority=image_authority(world),recipe_sha256=_hash_file(world/'assets/signal-ssh.Dockerfile'),
                files={name:_hash_file(world/'scripts'/name) for name in CONTROLLERS})


def verified_capsule(path,world,receipt,adoption):
    path=Path(path).resolve(strict=True);row=load(path/'capsule.json')
    check(row['schema']=='rsomics.signal.ssh-capsule.v1' and row['receipt']==receipt and row['receipt_adoption']==adoption and
          row['controller']==image_and_controller(world),
          'capsule native/controller/official-image authority differs')
    current=manifest(path);current.pop('capsule.json')
    check(current==row['files'] and row['runtime']['scientific_execution'] is False, 'complete capsule membership/body differs')
    return path


def committed_receipt(world, selector):
    world = Path(world).resolve(strict=True)
    check(isinstance(selector,str) and re.fullmatch(r'\.autopilot/oracles/[a-zA-Z0-9_.-]+\.json',selector),
          'explicit tracked native_receipt selector required; no default or local binary fallback')
    path = world/selector
    check(path.resolve(strict=True).is_relative_to(world/'.autopilot/oracles') and not path.is_symlink(), 'receipt route escaped')
    def git(*args):
        result=subprocess.run(['git','-C',str(world),*args],capture_output=True,check=False)
        check(result.returncode==0,'cannot establish adopted receipt HEAD/blob: '+selector)
        return result.stdout
    head=git('rev-parse','HEAD').decode().strip()
    check(re.fullmatch('[0-9a-f]{40}',head),'actual adopted receipt HEAD absent')
    entry=git('ls-tree','-z',head,'--',selector)
    check(entry.endswith(b'\0') and entry.count(b'\0')==1,'receipt is not committed in actual HEAD: '+selector)
    metadata,name=entry[:-1].split(b'\t',1)
    mode,kind,blob=metadata.decode().split()
    check(name.decode()==selector and mode in ('100644','100755') and kind=='blob',
          'adopted receipt is not a committed regular file')
    check(int(git('cat-file','-s',blob))<=1024*1024 and path.stat().st_size<=1024*1024,'receipt exceeds1MiB bound')
    data=git('cat-file','blob',blob)
    check(path.read_bytes()==data,'working receipt differs from adopted HEAD blob: '+selector)
    return data,dict(selector=selector,commit=head,blob=blob,sha256=hashlib.sha256(data).hexdigest())


def load_receipt(world, selector):
    world = Path(world).resolve(strict=True)
    data,adoption=committed_receipt(world,selector)
    row = json.loads(data)
    for field in ('run_id','run_attempt','artifact_id','size_in_bytes','source_file_count'):
        check(type(row.get(field)) is int and row[field]>0, 'incomplete native receipt: '+field)
    for field in ('artifact_sha256','binary_sha256','verification_sha256','source_archive_sha256',
                  'source_manifest_sha256','lock_sha256','worker_equality_sha256','integrity_sha256',
                  'source_on_native_sha256'):
        check(isinstance(row.get(field),str) and re.fullmatch('[0-9a-f]{64}',row[field]), 'incomplete native digest: '+field)
    check(row.get('schema_version')==1 and row.get('product')=='rsomics-signal' and
          row.get('repository')==REPOSITORY and row.get('workflow_path')=='.github/workflows/signal-candidate.yml',
          'native receipt product/workflow differs')
    check(row.get('native_targets')==TARGETS and row.get('workers')==[1,4] and
          row.get('source_on_native_passed') is True, 'actual native-four/same-file adoption is absent')
    check(re.fullmatch('[0-9a-f]{40}',row.get('head_sha','')) and re.fullmatch('[0-9a-f]{40}',row.get('owner_commit','')),
          'native head/owner commit absent')
    check(re.fullmatch(r'green-[1-9][0-9]*',row.get('snapshot','')), 'explicit adopted snapshot absent')
    check(row.get('artifact_name')==f"signal-native-{row['run_id']}-{row['run_attempt']}-linux-x86_64",
          'native artifact identity differs')
    snapshot = world/'.autopilot/snapshots/signal-native-2026-10-31'/row['snapshot']
    files = read_snapshot_manifest(snapshot)
    check(_hash_file(snapshot/'source.tar.gz')==row['source_archive_sha256'] and
          _hash_file(snapshot/'files.sha256')==row['source_manifest_sha256'] and
          len(files)==row['source_file_count'] and files['Cargo.lock']==row['lock_sha256'],
          'receipt does not bind actual referenced snapshot')
    check(re.search(r'Owning main commit:\s*\x60'+row['owner_commit']+r'\x60', (snapshot/'README.md').read_text()),
          'receipt owner commit differs from snapshot metadata')
    check(isinstance(row.get('controller_files'),dict) and row['controller_files'] and
          all(re.fullmatch('[A-Za-z0-9_.-]+',k) and re.fullmatch('[0-9a-f]{64}',v)
              for k,v in row['controller_files'].items()), 'accepted controller bindings absent')
    check({'signal_candidate.py','signal_source_on_native.py','signal_first_workflow_oracle.py',
        'test_signal_candidate_workflow.py','infercnv_source_snapshot.py','signal-candidate.yml',
        'signal-native-fixtures-20261031.json','signal-first-runtime-20261031.json'} <= set(row['controller_files']),
        'native admission omits actual complete controller/lock/fixture bindings')
    return row,adoption


def api(path, destination, binary=False):
    token = os.environ.get('GH_TOKEN') or os.environ.get('GITHUB_TOKEN')
    check(token, 'authenticated read-only Actions API token required')
    request = urllib.request.Request('https://api.github.com/repos/'+REPOSITORY+'/'+path,
        headers={'Accept':'application/vnd.github+json',
                 'X-GitHub-Api-Version':'2022-11-28','User-Agent':'rsomics-signal-qualified-performance'})
    request.add_unredirected_header('Authorization','Bearer '+token)
    with urllib.request.urlopen(request,timeout=120) as response, Path(destination).open('xb') as out:
        shutil.copyfileobj(response,out,1024*1024)
    if not binary:
        return load(destination)


def zip_members(archive):
    rows = archive.infolist()
    names = set()
    folded = set()
    for row in rows:
        name = _archive_name(row.filename,row.is_dir())
        kind = stat.S_IFMT(row.external_attr>>16) if row.create_system==3 else 0
        check(row.orig_filename==row.filename and not row.flag_bits&1 and name not in names and
              name.casefold() not in folded and kind in (0,stat.S_IFDIR if row.is_dir() else stat.S_IFREG),
              'unsafe/duplicate/casefold ZIP carrier member')
        names.add(name); folded.add(name.casefold())
    check(not any('/'.join(n.split('/')[:i]) in names and
                  not next(r for r in rows if r.filename.rstrip('/')=='/'.join(n.split('/')[:i])).is_dir()
                  for n in names for i in range(1,len(n.split('/')))), 'ZIP file/directory collision')
    return rows


def acquire_artifact(root, receipt, native=False, capsule=None, original_zip=None):
    root.mkdir(parents=True)
    def metadata(endpoint,name):
        if capsule is None:return api(endpoint,root/name)
        shutil.copyfile(Path(capsule)/name,root/name);return load(root/name)
    run = metadata(f"actions/runs/{receipt['run_id']}/attempts/{receipt['run_attempt']}",'run.json')
    _run(root/'run.json',receipt)
    item = metadata(f"actions/artifacts/{receipt['artifact_id']}",'artifact.json')
    check(item.get('size_in_bytes')==receipt['size_in_bytes'] and item.get('expired') is False and
          item.get('digest')=='sha256:'+receipt['artifact_sha256'], 'current artifact size/digest/admission differs')
    if native:
        jobs=metadata(f"actions/runs/{receipt['run_id']}/attempts/{receipt['run_attempt']}/jobs?per_page=100",'jobs.json')
        check(jobs['total_count']==len(jobs['jobs'])==4 and {j['name'] for j in jobs['jobs']}==set(TARGETS),
              'actual four native jobs are absent')
        check(all(j['conclusion']=='success' and j['status']=='completed' and
                  j['run_id']==receipt['run_id'] and j['head_sha']==receipt['head_sha'] and
                  j['run_attempt']==receipt['run_attempt'] for j in jobs['jobs']),
              'native platform job failed or identity differs')
    if capsule is not None:shutil.copyfile(Path(capsule)/'artifact.zip',root/'artifact.zip')
    elif original_zip is not None:shutil.copyfile(original_zip,root/'artifact.zip')
    else:api(f"actions/artifacts/{receipt['artifact_id']}/zip",root/'artifact.zip',True)
    _artifact(root/'artifact.json',receipt,root/'artifact.zip')
    check(_hash_file(root/'artifact.zip')==receipt['artifact_sha256'], 'downloaded original artifact differs')
    return run


def extract_native(root, receipt, world):
    prefix='signal-evidence/'
    fixed=['candidate/selection.json','candidate/files.sha256','candidate/source.tar.gz','candidate/source/Cargo.lock',
           'candidate/binary-release','candidate/binary-release.json','candidate/worker-equality.json',
           'candidate/integrity-after.json','source-on-native/qualification.json',
           'source-on-native/bindings-before.json','controller-checksums.json']
    wanted={prefix+n for n in fixed}
    wanted.update(prefix+'controller/'+name for name in receipt['controller_files'])
    selected=root/'selected'; selected.mkdir()
    inventory=[]
    with zipfile.ZipFile(root/'artifact.zip') as zipped:
        rows=zip_members(zipped)
        carriers=[r for r in rows if not r.is_dir()]
        check(len(carriers)==1 and carriers[0].filename.endswith('.tar'), 'actual native ZIP is not a single TAR carrier')
        check(zipped.testzip() is None, 'native carrier CRC failed')
        found=set()
        with zipped.open(carriers[0]) as raw, tarfile.open(fileobj=raw,mode='r|*') as archive:
            for member in archive:
                inventory.append(dict(name=member.name,kind=member.type.decode('latin1'),bytes=member.size))
                if member.name not in wanted:
                    continue
                check(member.name not in found and member.isfile(), 'selected native TAR member is not unique regular')
                _archive_name(member.name,False)
                check(member.size <= (512*2**20 if member.name.endswith('binary-release') else 32*2**20),
                      'selected native member exceeds bound')
                found.add(member.name)
                output=selected/member.name.removeprefix(prefix)
                output.parent.mkdir(parents=True,exist_ok=True)
                with archive.extractfile(member) as src,output.open('xb') as out:
                    shutil.copyfileobj(src,out,1024*1024)
        check(found==wanted,'native carrier lacks required bound evidence')
    save(root/'tar-inventory.json',dict(members=inventory,extraction_scope='selected regular metadata/source/binary only; runtime links not extracted'))
    candidate=selected/'candidate'
    selection=load(candidate/'selection.json')
    check(selection['snapshot']==receipt['snapshot'] and selection['source_archive_sha256']==receipt['source_archive_sha256'] and
          selection['source_manifest_sha256']==receipt['source_manifest_sha256'] and selection['lock_sha256']==receipt['lock_sha256'] and
          selection['controller_commit']==receipt['head_sha'], 'native selected source/controller differs')
    for name,key in [('source.tar.gz','source_archive_sha256'),('files.sha256','source_manifest_sha256'),('source/Cargo.lock','lock_sha256'),
                     ('binary-release','binary_sha256'),('worker-equality.json','worker_equality_sha256'),('integrity-after.json','integrity_sha256')]:
        check(_hash_file(candidate/name)==receipt[key],'native bound body differs: '+name)
    check(_hash_file(selected/'source-on-native/qualification.json')==receipt['source_on_native_sha256'],
          'actual same-native source qualification differs')
    worker=load(candidate/'worker-equality.json')
    check(worker['passed'] is True and worker['workers']==[1,4] and worker['source_sha256']==receipt['source_manifest_sha256'],
          'worker semantic qualification absent')
    check(load(candidate/'integrity-after.json')['failures']==[], 'native integrity failed')
    source=load(selected/'source-on-native/qualification.json')
    check(source['passed'] is True and source['failure'] is None and
          {'a','b','compare','average','rollover','empty'} <= set(source['observed']) and
          all(v['source_same_file'] is True for v in source['observed'].values()), 'source-on-native qualification incomplete')
    check(load(selected/'source-on-native/bindings-before.json')['native_binary_sha256']==receipt['binary_sha256'],
          'qualified source consumer used another binary')
    controllers=load(selected/'controller-checksums.json')
    check({k:v['sha256'] for k,v in controllers.items()}==receipt['controller_files'], 'archived controller set differs')
    check(all(_hash_file(selected/'controller'/name)==sha for name,sha in receipt['controller_files'].items()),
          'archived controller bytes differ')
    fixtures=load(selected/'controller/signal-native-fixtures-20261031.json')
    check(fixtures['accepted_source']['verification_sha256']==receipt['verification_sha256'] and
          selection['fixture_archive_sha256']==fixtures['archive_sha256'] and
          selection['fixture_manifest_sha256']==receipt['controller_files']['signal-native-fixtures-20261031.json'],
          'accepted original-source/portable fixture binding differs')
    source_bindings=load(selected/'source-on-native/bindings-before.json')
    check(source_bindings['runtime_lock_sha256']==LOCK_SHA and source_bindings['producer_sha256']==PRODUCER_SHA,
          'qualified source-on-native used another source runtime/controller')
    files=read_snapshot_manifest(Path(world)/'.autopilot/snapshots/signal-native-2026-10-31'/receipt['snapshot'])
    import signal_candidate
    with (candidate/'source.tar.gz').open('rb') as raw:
        signal_candidate.validate_archive(raw,files)
    binary=candidate/'binary-release'
    with binary.open('rb') as stream: elf=stream.read(20)
    check(elf[:6]==b'\x7fELF\x02\x01' and int.from_bytes(elf[18:20],'little')==62, 'qualified executable is not Linux x86_64 ELF')
    metadata=load(candidate/'binary-release.json')
    check(metadata['compiler_host']=='x86_64-unknown-linux-gnu' and metadata['sha256']==receipt['binary_sha256'] and
          metadata['selection_sha256']==_hash_file(candidate/'selection.json'), 'native executable metadata differs')
    binary.chmod(0o755)
    save(root/'verified.json',dict(receipt=receipt,binary=str(binary),binary_sha256=_hash_file(binary),admitted=True))
    return binary


def extract_inputs(root):
    selected=root/'selected'; selected.mkdir()
    with zipfile.ZipFile(root/'artifact.zip') as archive:
        rows=zip_members(archive)
        check(len(rows)==521 and archive.testzip() is None,'whole public input carrier differs')
        by_name={r.filename:r for r in rows}
        for name,(size,sha) in INPUT_FILES.items():
            row=by_name[name]
            check(not row.is_dir() and (size is None or row.file_size==size),'selected input member shape differs')
            target=selected/name; target.parent.mkdir(parents=True,exist_ok=True)
            with archive.open(row) as src,target.open('xb') as out: shutil.copyfileobj(src,out,1024*1024)
            check(_hash_file(target)==sha,'selected input body differs: '+name)
    save(root/'verified.json',dict(receipt=INPUT,files=manifest(selected),data_license='not established; no raw corpus in repository/package'))
    return selected/'benchmark/inputs'


def acquire(root,world,selector,capsule=None):
    root=checked_paths(root); world=Path(world)
    check(not (root/'evidence/execution-context.json').exists() or capsule is not None,
          'HELD: SSH acquisition requires its verified capsule, never an API token')
    receipt,adoption=load_receipt(world,selector)
    controller_binding(root,world)
    if capsule is not None:
        context=validate_ssh_context(load(root/'evidence/execution-context.json'),root)
        check(context['controller']==image_and_controller(world), 'actual SSH/controller Git identity differs')
        capsule=verified_capsule(capsule,world,receipt,adoption)
    native=root/'evidence/acquisition/native'
    acquire_artifact(native,receipt,True,capsule/'native' if capsule else None)
    binary=extract_native(native,receipt,world)
    save(root/'evidence/native-admission.json',dict(receipt_selector=selector,receipt_sha256=adoption['sha256'],
        receipt=receipt,receipt_adoption=adoption,binary=str(binary),binary_sha256=_hash_file(binary)))
    inputs=root/'evidence/acquisition/input'
    acquire_artifact(inputs,INPUT,capsule=capsule/'input' if capsule else None)
    extract_inputs(inputs)
    check(_hash_file(world/'assets/signal-first-runtime-20261031.json')==LOCK_SHA and _hash_file(oracle.__file__)==PRODUCER_SHA,
          'immutable source acquisition helper/lock changed')
    if capsule is None:
        check(not (root/'evidence/execution-context.json').exists(), 'SSH source acquisition requires its verified capsule')
        oracle.acquire_runtime(world/'assets/signal-first-runtime-20261031.json',root/'runtime')
    else:
        save(root/'evidence/runtime-restore.json',verify_runtime_carrier(native_carrier(native),root/'runtime'))


def controller_binding(root,world):
    world=Path(world).resolve(strict=True)
    files=['scripts/'+name for name in CONTROLLERS]+[
        '.github/workflows/signal-real-performance.yml','assets/signal-first-runtime-20261031.json',
        'docs/plans/2026-10-31-signal-whole-workflow-performance.md',IMAGE_ASSET,'assets/signal-ssh.Dockerfile']
    head=subprocess.run(['git','-C',str(world),'rev-parse','HEAD'],check=True,
                        capture_output=True,text=True).stdout.strip()
    check(re.fullmatch('[0-9a-f]{40}',head),'actual performance controller commit absent')
    current=dict(head_sha=head,files={name:_hash_file(world/name) for name in files})
    target=Path(root)/'evidence/performance-controller.json'
    if target.exists(): check(load(target)==current,'performance controller bytes/commit changed after acquisition')
    else: save(target,current)
    return current


def admission_command(output, argv):
    output=Path(output);output.mkdir(parents=True)
    save(output/'request.json',dict(argv=list(map(str,argv)),timeout_seconds=30))
    try:
        with (output/'stdout.txt').open('xb') as stdout,(output/'stderr.txt').open('xb') as stderr:
            completed=subprocess.run(list(map(str,argv)),stdout=stdout,stderr=stderr,timeout=30)
    except (OSError,subprocess.TimeoutExpired) as error:
        save(output/'command.json',dict(status='failed',type=type(error).__name__,message=str(error)))
        raise
    save(output/'command.json',dict(status=completed.returncode,
        stdout_sha256=_hash_file(output/'stdout.txt'),stderr_sha256=_hash_file(output/'stderr.txt')))
    check(completed.returncode==0,'HELD: admission command failed; originals retained at '+str(output))
    return (output/'stdout.txt').read_text()+(output/'stderr.txt').read_text()


def ssh_admission(root,world):
    root=checked_paths(root);world=Path(world)
    context=validate_ssh_context(load(root/'evidence/execution-context.json'),root)
    check(not os.environ.get('LD_PRELOAD') and not os.environ.get('LD_LIBRARY_PATH'), 'unbound loader overrides refused')
    for tool in ('readelf','ldd','ldconfig','git'):
        check(shutil.which(tool), 'HELD: derivative system tool missing: '+tool)
    closure=Path('/usr/local/share/rsomics-system-closure')
    bodies={}
    for line in (closure/'bodies.sha256').read_text().splitlines():
        sha,name=line.split(None,1);name=name.strip()
        check(re.fullmatch('[0-9a-f]{64}',sha) and _archive_name(name,False)==name, 'invalid system closure index')
        path=closure/name;check(path.is_file() and not path.is_symlink() and _hash_file(path)==sha, 'actual system package/index body differs')
        bodies[name]=dict(bytes=path.stat().st_size,sha256=sha)
    check(any(name.startswith('archives/') and name.endswith('.deb') for name in bodies) and
          any('InRelease' in name for name in bodies), 'retained official APT originals/index absent')
    save(root/'evidence/system-tool-closure.json',dict(files=bodies,packages=(closure/'packages.tsv').read_text(),
        packages_sha256=_hash_file(closure/'packages.tsv'),image=context['image']['Id'],
        dockerfile_sha256=_hash_file(world/'assets/signal-ssh.Dockerfile'),source_runtime_rebuilt=False))
    command_number=0
    def command(argv):
        nonlocal command_number
        command_number+=1
        return admission_command(root/'evidence/ssh-admission-commands'/str(command_number),argv)
    cache={}
    def elf(path):
        path=Path(path).resolve(strict=True)
        if path in cache:return cache[path]
        with path.open('rb') as stream:header=stream.read(20)
        check(header[:6]==b'\x7fELF\x02\x01' and int.from_bytes(header[18:20],'little')==62, 'actual ELF class/ISA differs: '+str(path))
        text=command(['readelf','--program-headers','--dynamic','--version-info',path])
        needs={};current=None;section=''
        for line in text.splitlines():
            if line.startswith('Version '):section=line
            if 'Version needs section' in section:
                found=re.search(r'File:\s*(\S+)',line)
                if found:current=found[1];needs.setdefault(current,set())
                found=re.search(r'Name:\s*(\S+)',line)
                if found and current:needs[current].add(found[1])
        definitions=set(re.findall(r'Name:\s*(\S+)',text.split('Version definition section',1)[-1].split('Version needs section',1)[0])) if 'Version definition section' in text else set()
        result=dict(needed=re.findall(r'\(NEEDED\).*?\[([^\]]+)\]',text),versions=needs,definitions=definitions,
                    sonames=re.findall(r'\(SONAME\).*?\[([^\]]+)\]',text),sha256=_hash_file(path))
        interpreter=re.findall(r'Requesting program interpreter:\s*([^\]]+)\]',text)
        result['interpreter']=interpreter[0] if interpreter else None
        cache[path]=result;return result
    candidates={}
    for line in command(['ldconfig','-p']).splitlines():
        match=re.match(r'\s*(\S+)\s+\([^)]*x86-64[^)]*\)\s+=>\s+(/\S+)',line)
        if match:candidates.setdefault(match[1],set()).add(Path(match[2]).resolve(strict=True))
    runtime=root/'runtime';paths={Path(load(root/'evidence/native-admission.json')['binary']).resolve(strict=True)}
    for path in runtime.rglob('*'):
        if path.is_file() and ('.so' in path.name or path.name.startswith('python')):
            with path.open('rb') as stream:magic=stream.read(4)
            if magic==b'\x7fELF':paths.add(path.resolve(strict=True))
    for path in paths:
        info=elf(path)
        for name in [path.name,*info['sonames']]:candidates.setdefault(name,set()).add(path)
    slots=[]
    for path in sorted(paths):
        info=elf(path)
        if info['interpreter']:elf(Path(info['interpreter']).resolve(strict=True))
        for name in info['needed']:
            providers={Path(name.replace('$ORIGIN',str(path.parent))).resolve(strict=True)} if '/' in name else candidates.get(name,set())
            required=info['versions'].get(name,set())
            eligible=[provider for provider in sorted(providers) if required<=elf(provider)['definitions']]
            check(eligible, 'HELD: actual provider/version closure unresolved: '+str(path)+' → '+name)
            slots.append(dict(elf=str(path),needed=name,versions=sorted(required),providers=list(map(str,eligible))))
    for path in (Path(load(root/'evidence/native-admission.json')['binary']),runtime/'python/bin/python3'):
        text=command(['ldd',path])
        check('not found' not in text, 'HELD: actual executable-root loader failure')
    save(root/'evidence/ssh-static-admission.json',dict(passed=True,slots=slots,
        bodies={str(path):row['sha256'] for path,row in cache.items()},container_id=context['container']['Id'],
        scientific_import=False,scientific_execution=False))
    oracle.capture(root/'evidence/ssh-source-import',[runtime/'venv/bin/python','-B',Path(__file__),'ssh-source-probe',
        '--root',root,'--world',world])
    probe=load(root/'evidence/ssh-source-import/providers.json')
    save(root/'evidence/ssh-provider-admission.json',dict(provider_admitted=True,container_id=context['container']['Id'],
        static_sha256=_hash_file(root/'evidence/ssh-static-admission.json'),mapped=probe,
        scientific_import=True,scientific_producer_executed=False,
        qualified_graphs=False,performance_claim=False,release=False))


def ssh_source_probe(root,world):
    root=checked_paths(root)
    context=validate_ssh_context(load(root/'evidence/execution-context.json'),root)
    static=load(root/'evidence/ssh-static-admission.json')
    check(static['passed'] and static['container_id']==context['container']['Id'] and
          all(_hash_file(path)==sha for path,sha in static['bodies'].items()), 'static runtime/provider bodies changed before imports')
    import signal_real_performance_check as checker
    checker.pinned_runtime(root,world)
    mapped={}
    for line in Path('/proc/self/maps').read_text().splitlines():
        fields=line.split()
        if len(fields)<6 or not fields[-1].startswith('/'):continue
        path=Path(fields[-1]).resolve(strict=True)
        with path.open('rb') as stream:magic=stream.read(4)
        if magic!=b'\x7fELF':continue
        check(str(path) in static['bodies'] and _hash_file(path)==static['bodies'][str(path)],
              'actual imported ELF provider was not statically admitted: '+str(path))
        mapped[str(path)]=static['bodies'][str(path)]
    save(root/'evidence/ssh-source-import/providers.json',mapped)


def ssh_campaign_context(root):
    context=validate_ssh_context(load(root/'evidence/execution-context.json'),root)
    admission=load(root/'evidence/ssh-provider-admission.json');static=load(root/'evidence/ssh-static-admission.json')
    check(admission['provider_admitted'] and admission['container_id']==context['container']['Id'] and
          admission['static_sha256']==_hash_file(root/'evidence/ssh-static-admission.json') and
          all(_hash_file(path)==sha for path,sha in static['bodies'].items()), 'SSH runtime/provider admission changed')
    return dict(context,provider_admitted=True)


def frozen_inputs(inputs,qualification):
    for name,row in qualification['files'].items():
        check(_hash_file(inputs/name)==row['sha256'],'frozen prepared input changed: '+name)


def bucket(qname):
    return int.from_bytes(hashlib.sha256(PREFIX+qname).digest()[:8],'big')%3


def pair_schedule():
    return [dict(case=case,workers=workers,pair=i+1,order=['source','native'] if i%2==0 else ['native','source'])
            for case in CASES for workers in (1,4) for i in range(5)]


def graph(tool,case,workers,inputs,output,binary,python,runtime,sorted_bed=True):
    output=Path(output); inputs=Path(inputs); native=tool=='native'
    stages=[]
    def add(name,entry,nargs,sargs,outputs):
        if native and entry in ('alignmentSieve','bamCoverage','bamCompare'):
            report=name+'.report.json'
            nargs=[*nargs,'--report',output/report]; outputs=[*outputs,report]
        command=[str(binary),*map(str,nargs),'--threads',str(workers)] if native else [
            str(python),'-B',str(Path(runtime)/'venv/bin'/entry),*map(str,sargs),'-p',str(workers)]
        stages.append(dict(name=name,argv=command,outputs=[str(output/p) for p in outputs],shared=False))
    def sorting(sample):
        name=sample+'.sorted.bam'
        stages.append(dict(name='sort-index-'+sample,argv=[str(python),'-B',str(Path(__file__).resolve()),'samtools',
            '--input',str(output/(sample+'.filtered.bam')),'--output',str(output/name),'--workers',str(workers)],
            outputs=[str(output/name),str(output/(name+'.bai'))],shared=True))
    if case=='track20':
        add('track20','bamCoverage',['track','--bam',inputs/'whole.bam','--out',output/'whole20.bw','--bin-size',20,'--normalize','CPM'],
            ['-b',inputs/'whole.bam','-o',output/'whole20.bw','--binSize',20,'--normalizeUsing','CPM'],['whole20.bw'])
        return stages
    if case=='atac':
        add('proper-atac','alignmentSieve',['filter','--bam',inputs/'whole.bam','--out',output/'atac.filtered.bam',
            '--filtered-out',output/'atac.rejected.bam','--metrics',output/'atac.metrics.tsv','--include-flags',2,'--atac-shift','--genome-chunk-length',1000000],
            ['-b',inputs/'whole.bam','-o',output/'atac.filtered.bam','--filteredOutReads',output/'atac.rejected.bam',
             '--filterMetrics',output/'atac.metrics.tsv','--samFlagInclude',2,'--ATACshift','--genomeChunkLength',1000000],
            ['atac.filtered.bam','atac.rejected.bam','atac.metrics.tsv'])
        sorting('atac')
        add('macs-fragments','alignmentSieve',['filter','--bam',output/'atac.sorted.bam','--out',output/'fragments.bed','--format','fragments-bed','--genome-chunk-length',1000000],
            ['-b',output/'atac.sorted.bam','-o',output/'fragments.bed','--BED','--genomeChunkLength',1000000],['fragments.bed'])
        return stages
    check(case=='main' and tool in ('source','native'),'unknown fixed graph')
    tracks=[]
    bams=[]
    for i in range(3):
        sample='p'+str(i); bam=inputs/(sample+'.bam'); sorted_bam=output/(sample+'.sorted.bam')
        add('filter-'+sample,'alignmentSieve',['filter','--bam',bam,'--out',output/(sample+'.filtered.bam'),
            '--filtered-out',output/(sample+'.rejected.bam'),'--metrics',output/(sample+'.metrics.tsv'),
            '--min-mapq',20,'--exclude-flags',2816,'--ignore-duplicates','--genome-chunk-length',1000000],
            ['-b',bam,'-o',output/(sample+'.filtered.bam'),'--filteredOutReads',output/(sample+'.rejected.bam'),
             '--filterMetrics',output/(sample+'.metrics.tsv'),'--minMappingQuality',20,'--samFlagExclude',2816,
             '--ignoreDuplicates','--genomeChunkLength',1000000],[sample+'.filtered.bam',sample+'.rejected.bam',sample+'.metrics.tsv'])
        sorting(sample); bams.append(sorted_bam)
        track=output/(sample+'.bw'); tracks.append(track)
        add('track-'+sample,'bamCoverage',['track','--bam',sorted_bam,'--out',track,'--bin-size',50,'--normalize','CPM'],
            ['-b',sorted_bam,'-o',track,'--binSize',50,'--normalizeUsing','CPM'],[sample+'.bw'])
    add('compare-bam','bamCompare',['compare','--input','bam','--first',bams[0],'--second',bams[1],
        '--out',output/'CBAM.bw','--bin-size',50,'--operation','ratio','--scale-method','readCount','--pseudocount',1,1],
        ['-b1',bams[0],'-b2',bams[1],'-o',output/'CBAM.bw','--binSize',50,'--operation','ratio',
         '--scaleFactorsMethod','readCount','--pseudocount',1,1],['CBAM.bw'])
    add('compare-bw','bigwigCompare',['compare','--input','bigwig','--first',tracks[0],'--second',tracks[1],
        '--out',output/'CBW.bw','--bin-size',50,'--operation','ratio','--scale-factors','1:1','--pseudocount',1,1],
        ['-b1',tracks[0],'-b2',tracks[1],'-o',output/'CBW.bw','--binSize',50,'--operation','ratio',
         '--scaleFactors','1:1','--pseudocount',1,1],['CBW.bw'])
    add('average','bigwigAverage',['average','--bigwig',*tracks,'--out',output/'AVG.bw','--bin-size',50,'--scale-factors','1:0.5:2'],
        ['-b',*tracks,'-o',output/'AVG.bw','--binSize',50,'--scaleFactors','1:0.5:2'],['AVG.bw'])
    for kind,files in [('bam',bams),('bw',[output/'CBAM.bw',output/'CBW.bw',output/'AVG.bw'])]:
        for mode in ('bins','regions'):
            stem='summary-'+kind+'-'+mode
            nargs=['summarize',mode,'--input','bam' if kind=='bam' else 'bigwig','--files',*files,
                '--out',output/(stem+'.npz'),'--raw-counts',output/(stem+'.tsv'),'--labels','P0','P1','P2']
            sargs=['bins' if mode=='bins' else 'BED-file','-b',*files,'-o',output/(stem+'.npz'),
                '--outRawCounts',output/(stem+'.tsv'),'--labels','P0','P1','P2']
            outputs=[stem+'.npz',stem+'.tsv']
            if mode=='bins':
                nargs += ['--bin-size',10000]; sargs += ['--binSize',10000]
            else:
                nargs += ['--regions',inputs/'genes.bed12','--metagene']
                sargs += ['--BED',inputs/'genes.bed12','--metagene']
            if kind=='bam':
                nargs += ['--scaling-factors',output/(stem+'.factors.tsv')]
                sargs += ['--scalingFactors',output/(stem+'.factors.tsv')]
                outputs.append(stem+'.factors.tsv')
            add(stem,'multiBamSummary' if kind=='bam' else 'multiBigwigSummary',nargs,sargs,outputs)
    for mode in ('reference','scale'):
        stem='matrix-'+mode
        nargs=['matrix',mode,'--bigwig',*tracks,'--regions',inputs/'plus.bed',inputs/'minus.bed',
            '--out',output/(stem+'.gz'),'--matrix-tsv',output/(stem+'.tsv'),'--labels','P0','P1','P2',
            '--bin-size',50,'--before',500,'--after',1500 if mode=='reference' else 500,'--metagene','--sort-regions','keep']
        sargs=['reference-point' if mode=='reference' else 'scale-regions','-S',*tracks,'-R',inputs/'plus.bed',inputs/'minus.bed',
            '-o',output/(stem+'.gz'),'--outFileNameMatrix',output/(stem+'.tsv'),'--samplesLabel','P0','P1','P2',
            '--binSize',50,'--beforeRegionStartLength',500,'--afterRegionStartLength',1500 if mode=='reference' else 500,
            '--metagene','--sortRegions','keep']
        if mode=='reference':
            nargs += ['--reference-point','TSS']; sargs += ['--referencePoint','TSS']
        else:
            nargs += ['--body-length',3000,'--unscaled-5prime',0,'--unscaled-3prime',0]
            sargs += ['--regionBodyLength',3000,'--unscaled5prime',0,'--unscaled3prime',0]
        outputs=[stem+'.gz',stem+'.tsv']
        if sorted_bed:
            nargs += ['--sorted-regions',output/(stem+'.bed')]; sargs += ['--outFileSortedRegions',output/(stem+'.bed')]
            outputs.append(stem+'.bed')
        add(stem,'computeMatrix',nargs,sargs,outputs)
    return [row for prefix in ('filter-','sort-index-','track-') for row in stages if row['name'].startswith(prefix)] + [row for row in stages if not row['name'].startswith(('filter-','sort-index-','track-'))]

def partition_stream(source, outputs, database):
    import signal_real_performance_check as checker
    header, references = checker.bam_header(source)
    for out in outputs:
        out.write(header)
    db=sqlite3.connect(database)
    db.execute('pragma cache_size=-8192'); db.execute('pragma journal_mode=persist')
    db.execute('create table records(bucket integer, ordinal integer, qname blob, payload blob, primary key(bucket,ordinal))')
    db.execute('create table qnames(qname blob primary key,bucket integer,records integer)')
    counts=[dict(records=0,mapped=0,proper_pair=0,flags={},cigar_ops={}) for _ in range(3)]
    previous=[None]*3
    for record in checker.bam_records(source):
        fields=checker.record_fields(record)
        qname=fields['qname']; group=bucket(qname); row=counts[group]
        position=(fields['reference'],fields['position'])
        if fields['flags']&4==0:
            check(previous[group] is None or previous[group]<=position,'original partition coordinate order is not nondecreasing')
            previous[group]=position
            row['mapped']+=1
        row['proper_pair']+=bool(fields['flags']&2)
        flag=str(fields['flags']); row['flags'][flag]=row['flags'].get(flag,0)+1
        for operation,_ in fields['cigar']:
            key=str(operation); row['cigar_ops'][key]=row['cigar_ops'].get(key,0)+1
        outputs[group].write(struct.pack('<i',len(record))); outputs[group].write(record)
        db.execute('insert into records values(?,?,?,?)',(group,row['records'],qname,record))
        db.execute('insert into qnames values(?,?,1) on conflict(qname) do update set records=records+1',(qname,group))
        row['records']+=1
        if sum(r['records'] for r in counts)%50000==0:
            db.commit()
    db.commit()
    bad=db.execute('select count(*) from qnames where bucket<0 or bucket>2').fetchone()[0]
    check(bad==0,'QNAME membership failed')
    qnames=db.execute('select bucket,count(*) from qnames group by bucket').fetchall()
    db.close()
    return dict(header_sha256=hashlib.sha256(header).hexdigest(),references=references,
                partitions=counts,qnames=dict(qnames),kind='one specimen; exact-QNAME engineering partitions')


def prepare(root, world):
    import signal_real_performance_check as checker
    root=checked_paths(root); world=Path(world)
    if (root/'evidence/execution-context.json').exists():ssh_campaign_context(root)
    checker.pinned_runtime(root,world)
    admission=load(root/'evidence/native-admission.json')
    check(_hash_file(admission['binary'])==admission['binary_sha256'],'admitted native binary changed')
    original=root/'evidence/acquisition/input/selected/benchmark/inputs'
    output=root/'evidence/prepared'; output.mkdir()
    for name,source in [('whole.bam',original/'paired.bam'),('whole.bam.bai',original/'paired.bam.bai'),('genes.bed12',original/'genes.bed12')]:
        shutil.copyfile(source,output/name)
    import pysam
    streams=[]
    try:
        streams=[pysam.BGZFile(str(output/f'p{i}.bam'),'wb') for i in range(3)]
        with gzip.open(output/'whole.bam','rb') as source:
            record=partition_stream(source,streams,output/'partition-exact.sqlite')
    finally:
        for stream in streams: stream.close()
    check(sum(r['records'] for r in record['partitions'])==9810963 and
          sum(r['mapped'] for r in record['partitions'])==9810963 and
          sum(r['proper_pair'] for r in record['partitions'])==6959152 and
          all(r['mapped']>=2000000 for r in record['partitions']), 'whole/partition populations do not qualify')
    check(len(record['references'])==25 and sum(n for _,n in record['references'])==3095693983,'whole dictionary differs')
    db=sqlite3.connect(output/'partition-exact.sqlite')
    for group in range(3):
        expected=db.execute('select qname,payload from records where bucket=? order by ordinal',(group,))
        with gzip.open(output/f'p{group}.bam','rb') as stream:
            header,refs=checker.bam_header(stream)
            check(hashlib.sha256(header).hexdigest()==record['header_sha256'] and refs==record['references'],'partition changed full header/dictionary')
            count=0
            for payload in checker.bam_records(stream):
                row=expected.fetchone()
                check(row is not None and row[1]==payload and row[0]==checker.record_fields(payload)['qname'] and bucket(row[0])==group,
                      'exact routed payload/order/QNAME union differs')
                count+=1
            check(expected.fetchone() is None and count==record['partitions'][group]['records'],'partition union cardinality differs')
        pysam.index('-@','3',str(output/f'p{group}.bam'))
    db.close()
    plus=minus=multi=0; names=set(); duplicates=0
    with (output/'genes.bed12').open('rb') as source,(output/'plus.bed').open('xb') as positive,(output/'minus.bed').open('xb') as negative:
        for line in source:
            fields=line.rstrip(b'\r\n').split(b'\t')
            check(len(fields)==12 and fields[5] in (b'+',b'-'),'qualified BED12 row shape/strand differs')
            (positive if fields[5]==b'+' else negative).write(line)
            plus+=fields[5]==b'+'; minus+=fields[5]==b'-'; multi+=int(fields[9])>1
            duplicates+=fields[3] in names; names.add(fields[3])
    check((plus,minus,multi,duplicates)==(29903,28481,54030,62),'rich annotation cohort differs')
    exclusions=sum(1 for _ in (original/'annotation-exclusions.jsonl').open())
    check(exclusions==2793,'original absent-reference exclusions differ')
    record.update(groups=dict(plus=plus,minus=minus,multi_exon=multi,duplicate_name_rows=duplicates,absent_reference_exclusions=exclusions),
                  files=manifest(output),exact_payload_order_membership_union=True,indexed=True,
                  source_input_receipt=INPUT,redistribution_license='not established; raw corpus excluded from repository/package')
    save(output/'qualification.json',record)


def proc_group(group):
    records=[]
    for directory in Path('/proc').iterdir():
        if not directory.name.isdigit(): continue
        try:
            raw=(directory/'stat').read_text()
            fields=raw[raw.rfind(')')+2:].split()
            if int(fields[2])!=group: continue
            io_counters={}
            try:
                io_counters={k:int(v) for k,v in (line.split(':',1) for line in (directory/'io').read_text().splitlines())}
            except (OSError,ValueError):
                io_counters=None
            records.append(dict(pid=int(directory.name),ppid=int(fields[1]),start_ticks=int(fields[19]),
                rss_bytes=int(fields[21])*os.sysconf('SC_PAGE_SIZE'),utime_ticks=int(fields[11]),stime_ticks=int(fields[12]),
                minor_faults=int(fields[7]),major_faults=int(fields[9]),io=io_counters))
            if len(records)>4096:
                raise RuntimeError('owned process-group cardinality exceeds4096')
        except (OSError,ValueError,IndexError):
            continue
    return records


def cgroup_optional(parent, name):
    if not parent: return None
    parent=Path(parent).resolve(strict=True)
    check(str(parent).startswith('/sys/fs/cgroup/') and (parent/'cgroup.controllers').is_file(),
          'optional cgroup is not an existing delegated v2 directory')
    check('memory' in (parent/'cgroup.controllers').read_text().split(),'memory controller is not delegated')
    path=parent/name; path.mkdir()
    try:
        (path/'memory.max').write_text(str(MEMORY_LIMIT))
        (path/'memory.oom.group').write_text('1')
        check((path/'memory.peak').is_file(),'exact cgroup memory peak unavailable')
    except OSError as exc:
        raise ValueError('optional cgroup is not actually delegated: '+str(exc)) from exc
    return path


def cgroup_counters(path):
    if path is None: return None
    out={}
    for name in ('memory.peak','memory.stat','memory.events','cpu.stat','io.stat','cgroup.events'):
        target=path/name
        out[name]=target.read_text() if target.is_file() else None
    return out


def _wait4(command, stdout, stderr, cpus, cgroup=None):
    def enter():
        os.sched_setaffinity(0,cpus)
        if cgroup is not None: (cgroup/'cgroup.procs').write_text(str(os.getpid()))
    start=time.monotonic_ns()
    child=subprocess.Popen(command,stdout=stdout,stderr=stderr,preexec_fn=enter)
    pid,status,usage=os.wait4(child.pid,0)
    child.returncode=os.waitstatus_to_exitcode(status)
    return dict(pid=pid,argv=command,exit=child.returncode,wall_seconds=(time.monotonic_ns()-start)/1e9,
        user_seconds=usage.ru_utime,system_seconds=usage.ru_stime,kernel_max_rss_bytes=usage.ru_maxrss*1024,
        kernel_max_rss_scope='Linux wait4 KiB normalized to bytes; maximum process/waited-child RSS, NOT aggregate tree',
        minor_faults=usage.ru_minflt,major_faults=usage.ru_majflt,
        filesystem_input_blocks=usage.ru_inblock,filesystem_output_blocks=usage.ru_oublock,
        io_bytes=None,aggregate_tree_peak_bytes=None,shared=False)


def execute_graph(plan_file,cpus):
    plan=load(plan_file)
    os.sched_setaffinity(0,cpus)
    rows=[]
    for stage in plan['stages']:
        directory=Path(plan['logs'])/stage['name']; directory.mkdir(parents=True)
        with (directory/'stdout.txt').open('xb') as stdout,(directory/'stderr.txt').open('xb') as stderr:
            row=_wait4(stage['argv'],stdout,stderr,cpus)
        row['name']=stage['name']; row['shared']=stage['shared']
        save(directory/'measurement.json',row)
        check(row['exit']==0,'required stage failed: '+stage['name'])
        check(all(Path(p).is_file() for p in stage['outputs']),'required useful output absent: '+stage['name'])
        rows.append(row)
    save(Path(plan['logs']).parent/'stages.json',rows)


def trial(root,directory,tool,case,workers,inputs,binary,runtime,cpus,cgroup_parent=None,stages_override=None,watched=()):
    directory.mkdir(parents=True)
    products=directory/'products'; products.mkdir()
    scratch=directory/'scratch'; scratch.mkdir()
    logs=directory/'commands'; logs.mkdir()
    stages=stages_override if stages_override is not None else graph(tool,case,workers,inputs,products,binary,Path(runtime)/'venv/bin/python',runtime)
    save(directory/'plan.json',dict(tool=tool,case=case,workers=workers,stages=stages,logs=str(logs),
        timed_explicit_sort_index=True,fsync_durability_claim=False,source_native_graphs_independent=True))
    group=cgroup_optional(cgroup_parent,'signal-'+hashlib.sha256(str(directory).encode()).hexdigest()[:20]+'-'+str(os.getpid()))
    command=[str(Path(runtime)/'venv/bin/python'),'-B',str(Path(__file__).resolve()),'execute-graph',
             '--plan',str(directory/'plan.json'),'--cpus',','.join(map(str,cpus))]
    env=dict(os.environ,TMPDIR=str(scratch))
    started=time.monotonic_ns()
    def enter():
        os.sched_setaffinity(0,cpus)
        if group is not None: (group/'cgroup.procs').write_text(str(os.getpid()))
    child=None; failure=None; peak=0; peak_bytes=0; usage=None
    with (directory/'runner.stdout').open('xb') as stdout,(directory/'runner.stderr').open('xb') as stderr,(directory/'process-samples.jsonl').open('x') as samples:
        try:
            child=subprocess.Popen(command,stdout=stdout,stderr=stderr,env=env,start_new_session=True,preexec_fn=enter)
            while True:
                records=proc_group(child.pid)
                summed=sum(r['rss_bytes'] for r in records)
                peak=max(peak,summed)
                size=tree_bytes(directory); peak_bytes=max(peak_bytes,size)
                available=meminfo()['MemAvailable']
                samples.write(json.dumps(dict(monotonic_ns=time.monotonic_ns(),processes=records,summed_rss_bytes=summed,
                                    output_scratch_bytes=size,mem_available_bytes=available))+'\n'); samples.flush()
                check(summed<=MEMORY_LIMIT,'sampled live-tree RSS abort budget exceeded')
                check(available>=512*2**20,'host available-memory abort guard reached')
                check(size<=LIMITS[case],'trial useful-output/scratch budget exceeded')
                check(all(tree_bytes(path)<=LIMITS[case] for path in watched),'retained useful-output/diagnostic budget exceeded')
                check(tree_bytes(root)<=CAMPAIGN_LIMIT,'combined campaign storage budget exceeded')
                check(shutil.disk_usage(root).free>=HEADROOM,'external storage headroom exhausted')
                result=os.wait4(child.pid,os.WNOHANG)
                if result[0]:
                    _,status,usage=result; child.returncode=os.waitstatus_to_exitcode(status); break
                check(time.monotonic_ns()-started<24*3600*10**9,'trial exceeded24-hour trusted-child limit')
                time.sleep(0.5)
            check(child.returncode==0,'complete graph returned nonzero')
            check(not proc_group(child.pid),'complete graph left a live process-group descendant')
        except BaseException as exc:
            failure=dict(type=type(exc).__name__,message=str(exc))
            if child is not None:
                try: os.killpg(child.pid,signal.SIGTERM)
                except ProcessLookupError: pass
                time.sleep(0.2)
                try: os.killpg(child.pid,signal.SIGKILL)
                except ProcessLookupError: pass
                if child.returncode is None:
                    _,status,usage=os.wait4(child.pid,0); child.returncode=os.waitstatus_to_exitcode(status)
            raise
        finally:
            counters=cgroup_counters(group)
            exact_peak=int(counters['memory.peak'].strip()) if counters else None
            report=dict(tool=tool,case=case,workers=workers,argv=command,pid=child.pid if child else None,
                wall_seconds=(time.monotonic_ns()-started)/1e9,exit=child.returncode if child else None,
                user_seconds=usage.ru_utime if usage else None,system_seconds=usage.ru_stime if usage else None,
                kernel_max_rss_bytes=usage.ru_maxrss*1024 if usage else None,
                kernel_rss_scope='Linux wait4 maximum process/waited-child RSS; not aggregate multiprocess RSS',
                minor_faults=usage.ru_minflt if usage else None,major_faults=usage.ru_majflt if usage else None,
                filesystem_input_blocks=usage.ru_inblock if usage else None,filesystem_output_blocks=usage.ru_oublock if usage else None,
                raw_io_byte_counters_scope='sampled /proc individual live PIDs; not complete total; no I/O advantage claim',
                aggregate_tree_peak_bytes=exact_peak,aggregate_tree_peak_scope='dedicated cgroup v2 hierarchical memory.peak; anon/file/kernel usage, not RSS' if counters else None,
                cgroup=counters,sampled_sum_rss_peak_bytes=peak,sampled_rss_caveat='sampled live group sum, shared pages doublecounted, exited peaks may be missed; abort guard only',
                output_scratch_peak_bytes=peak_bytes,failure=failure,cache='warm-cache',durability_claim=False)
            save(directory/'measurement.json',report)
    save(directory/'products.json',manifest(products))
    return report


def checked_pair(root,world,case,workers,source,native,inputs,binary,runtime,cpus,cgroup_parent,qualification=False):
    directory=root/'evidence/checks'/(native.parent.name+'-'+native.name)
    command=[str(Path(runtime)/'venv/bin/python'),'-B',str(Path(__file__).with_name('signal_real_performance_check.py')),
        '--root',str(root),'--world',str(world),'--case',case,'--workers',str(workers),'--source',str(source),'--native',str(native),
        '--output',str(directory/'products')]
    if qualification: command.append('--qualification')
    stages=[dict(name='validate-all-products',argv=command,outputs=[str(directory/'products/qualification.json')],shared=True)]
    result=trial(root,directory,'checker',case,workers,inputs,binary,runtime,cpus,cgroup_parent,stages,(source,native))
    return result


def project_storage(root,qualification):
    base=tree_bytes(root)
    future=6*sum(row['output_scratch_peak_bytes'] for row in qualification)
    check(base+future<=CAMPAIGN_LIMIT and future+HEADROOM<=shutil.disk_usage(root).free,
          'qualified actual products/spools cannot fit complete retained campaign; revise workload explicitly before timing')
    return dict(actual_current_bytes=base,conservative_additional_bytes=future,combined_bytes=base+future,
                campaign_limit=CAMPAIGN_LIMIT,headroom=HEADROOM)


def strict_decision(rows,*,stage=False):
    source=[r['wall_seconds'] for r in rows if r['tool']=='source']
    native=[r['wall_seconds'] for r in rows if r['tool']=='native']
    check(len(source)==len(native)==5,'five complete paired samples required')
    wall=max(native)<min(source)
    s_peak=[r.get('aggregate_tree_peak_bytes') for r in rows if r['tool']=='source']
    n_peak=[r.get('aggregate_tree_peak_bytes') for r in rows if r['tool']=='native']
    exact=not stage and all(type(v) is int and v>0 for v in s_peak+n_peak) and {r.get('aggregate_tree_peak_scope') for r in rows}=={'dedicated cgroup v2 hierarchical memory.peak; anon/file/kernel usage, not RSS'}
    memory=exact and max(n_peak)<min(s_peak)
    def stats(values): return dict(samples=values,median=statistics.median(values),minimum=min(values),maximum=max(values))
    return dict(source=stats(source),native=stats(native),strict_wall=wall,
        strict_exact_tree_memory=None if stage else bool(memory),exact_tree_memory_available=exact,strict_advantage=bool(wall or memory),
        criterion='native max wall < source min wall' if stage else 'native max wall < source min wall OR exact matched whole-graph cgroup tree peak separation',
        memory_unavailable_reason='stage-specific exact tree memory is not measured; whole-graph peaks are not stage peaks' if stage else None,
        memory_source=stats(s_peak) if exact else None,memory_native=stats(n_peak) if exact else None,
        filesystem_blocks_are_not_bytes=True,durability_claim=False)


def campaign(root,world,selector,idle_confirmed,cgroup_parent=None,shared_host=False):
    import signal_real_performance_check as checker
    root=checked_paths(root); world=Path(world)
    context=ssh_campaign_context(root) if shared_host else {}
    host_policy=execution_policy(idle_confirmed,shared_host,context)
    check(shared_host or not (root/'evidence/execution-context.json').exists(), 'SSH context requires explicit shared-host mode')
    controller_binding(root,world)
    receipt,adoption=load_receipt(world,selector)
    admission=load(root/'evidence/native-admission.json')
    check(receipt==admission['receipt'] and adoption==admission['receipt_adoption'],'adopted receipt HEAD/blob changed after admission')
    binary=Path(admission['binary'])
    check(_hash_file(binary)==receipt['binary_sha256'],'qualified ELF changed')
    runtime=root/'runtime'; checker.pinned_runtime(root,world)
    inputs=root/'evidence/prepared'
    qualification=load(inputs/'qualification.json')
    frozen_inputs(inputs,qualification)
    descriptor=os.open(root/'campaign.lock',os.O_CREAT|os.O_EXCL|os.O_WRONLY,0o600)
    os.write(descriptor,str(os.getpid()).encode()); os.close(descriptor)
    save(root/'evidence/schedule.json',dict(pair_schedule=pair_schedule(),warmups_per_tool_worker_case=1,
        qualification_before_timing=True,cache='warm-cache',
        whole_graph_criterion='native max wall < source min wall OR exact matched whole-graph cgroup tree peak separation',
        replacement_stage_criterion='native max wall < source min wall; wall-only, exact stage memory N/A',
        no_concurrent_trials=True))
    cpus=load(root/'host-plan.json')['plan']['cpus']
    qualified=[]; results=[]; stage_rows={}
    for case in CASES:
        for workers in (1,4):
            assigned=cpus[:workers]
            pair={}
            for tool in ('source','native'):
                directory=root/'evidence/qualification'/f'{case}-{workers}-{tool}'
                qualified.append(trial(root,directory,tool,case,workers,inputs,binary,runtime,assigned,cgroup_parent))
                pair[tool]=directory
            qualified.append(checked_pair(root,world,case,workers,pair['source'],pair['native'],inputs,binary,runtime,assigned,cgroup_parent,True))
    save(root/'evidence/resource-qualification.json',project_storage(root,qualified))
    for case in CASES:
        for workers in (1,4):
            assigned=cpus[:workers]
            pair={}
            for tool in ('source','native'):
                directory=root/'evidence/warmup'/f'{case}-{workers}-{tool}'
                trial(root,directory,tool,case,workers,inputs,binary,runtime,assigned,cgroup_parent)
                pair[tool]=directory
            checked_pair(root,world,case,workers,pair['source'],pair['native'],inputs,binary,runtime,assigned,cgroup_parent)
            for row in [s for s in pair_schedule() if s['case']==case and s['workers']==workers]:
                pair={}
                for tool in row['order']:
                    check(_hash_file(binary)==receipt['binary_sha256'],'native binary changed between trials')
                    directory=root/'evidence/trials'/f"{case}-{workers}-{row['pair']}-{tool}"
                    result=trial(root,directory,tool,case,workers,inputs,binary,runtime,assigned,cgroup_parent)
                    results.append(result); pair[tool]=directory
                    for stage in load(directory/'stages.json'):
                        if stage['shared']: continue
                        key=(case,workers,stage['name'])
                        stage_rows.setdefault(key,[]).append(dict(stage,tool=tool))
                checked_pair(root,world,case,workers,pair['source'],pair['native'],inputs,binary,runtime,assigned,cgroup_parent)
    decisions={f'{case}-{workers}':strict_decision([r for r in results if r['case']==case and r['workers']==workers])
               for case in CASES for workers in (1,4)}
    stage_decisions={f'{case}-{workers}-{name}':strict_decision(values,stage=True) for (case,workers,name),values in stage_rows.items()}
    before=load(root/'evidence/runtime-before.json')
    after=checker.pinned_runtime(root,world)
    check(before==after and _hash_file(binary)==receipt['binary_sha256'],'qualified runtime/native changed during campaign')
    frozen_inputs(inputs,qualification)
    if shared_host:ssh_campaign_context(root)
    controller_binding(root,world)
    save(root/'evidence/summary.json',dict(decisions=decisions,stage_decisions=stage_decisions,all_samples=results,
        passed=all(r['strict_advantage'] for r in decisions.values()) and all(r['strict_advantage'] for r in stage_decisions.values()),
        native_four_and_source_adoption=receipt,receipt_adoption=adoption,sorted_bed='observational export only; default policy pending',
        host_policy=host_policy,execution_context=context,original_idle_host_gate_earned=False if shared_host else None,
        resource_claims='whole-graph exact matched cgroup alternative only; replacement stages strictly wall-only with exact stage memory N/A; no I/O/durability/cold-cache claims',
        release=False))
    check(all(r['strict_advantage'] for r in decisions.values()) and all(r['strict_advantage'] for r in stage_decisions.values()),
          'strict whole graph/individual replacement hot-path criterion not met; optimization gate remains')


def samtools(source,output,workers):
    import pysam
    check(pysam.__samtools_version__=='1.21','matched dispatcher is not actual pinned samtools1.21')
    threads=str(max(0,workers-1))
    pysam.sort('-@',threads,'-T',str(output.parent/(output.name+'.sort-tmp')),'-o',str(output),str(source),catch_stdout=False)
    pysam.index('-@',threads,str(output))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    sub=parser.add_subparsers(dest='action',required=True)
    setup_args=sub.add_parser('setup'); setup_args.add_argument('--root',type=Path,required=True); setup_args.add_argument('--cpus',required=True)
    setup_args.add_argument('--execution-ledger',type=Path)
    for action in ('acquire','prepare','campaign'):
        args=sub.add_parser(action); args.add_argument('--root',type=Path,required=True)
        args.add_argument('--world',type=Path,required=True)
        if action in ('acquire','campaign'): args.add_argument('--native-receipt',required=True)
        if action=='acquire':args.add_argument('--capsule',type=Path)
        if action=='campaign':
            args.add_argument('--idle-host-confirmed',action='store_true'); args.add_argument('--cgroup-parent',type=Path)
            args.add_argument('--shared-host',action='store_true')
    args=sub.add_parser('capsule');args.add_argument('--root',type=Path,required=True);args.add_argument('--world',type=Path,required=True)
    args.add_argument('--native-receipt',required=True);args.add_argument('--native-zip',type=Path,required=True);args.add_argument('--input-zip',type=Path,required=True)
    args=sub.add_parser('ssh-context');args.add_argument('--root',type=Path,required=True);args.add_argument('--world',type=Path,required=True)
    args.add_argument('--container-id',required=True);args.add_argument('--cpus',required=True);args.add_argument('--execution-ledger',type=Path,required=True)
    for action in ('admit-ssh','ssh-source-probe'):
        args=sub.add_parser(action);args.add_argument('--root',type=Path,required=True);args.add_argument('--world',type=Path,required=True)
    args=sub.add_parser('execute-graph'); args.add_argument('--plan',type=Path,required=True); args.add_argument('--cpus',required=True)
    args=sub.add_parser('samtools'); args.add_argument('--input',type=Path,required=True); args.add_argument('--output',type=Path,required=True); args.add_argument('--workers',type=int,choices=[1,4],required=True)
    args=sub.add_parser('finalize'); args.add_argument('--root',type=Path,required=True)
    args=parser.parse_args()
    if args.action=='setup': setup(args.root,args.cpus,args.execution_ledger)
    elif args.action=='acquire': acquire(args.root,args.world,args.native_receipt,args.capsule)
    elif args.action=='prepare': prepare(args.root,args.world)
    elif args.action=='campaign': campaign(args.root,args.world,args.native_receipt,args.idle_host_confirmed,args.cgroup_parent,args.shared_host)
    elif args.action=='capsule':seal_capsule(args.root,args.world,args.native_receipt,args.native_zip,args.input_zip)
    elif args.action=='ssh-context':record_ssh_context(args.root,args.world,args.container_id,args.cpus,args.execution_ledger)
    elif args.action=='admit-ssh':ssh_admission(args.root,args.world)
    elif args.action=='ssh-source-probe':ssh_source_probe(args.root,args.world)
    elif args.action=='execute-graph':
        cpus=[int(v) for v in args.cpus.split(',')]
        check(len(cpus) in (1,4) and set(cpus)<=os.sched_getaffinity(0),'actual trial CPU set differs')
        execute_graph(args.plan,cpus)
    elif args.action=='samtools': samtools(args.input,args.output,args.workers)
    else:
        root=checked_paths(args.root)
        save(root/'evidence/files.json',manifest(root/'evidence'))


if __name__=='__main__':
    try:
        main()
    except Exception as error:
        ssh_action=any(value in sys.argv for value in ('capsule','ssh-context','admit-ssh','ssh-source-probe',
                                                      '--capsule','--shared-host','--execution-ledger'))
        if not ssh_action:raise
        if '--root' in sys.argv:
            root=Path(sys.argv[sys.argv.index('--root')+1])
            if (root/'evidence/execution-context.json').is_file():
                checked_paths(root)
                path=root/'evidence'/('failure-'+sys.argv[1]+'.json')
                if not path.exists():save(path,dict(status='HELD',type=type(error).__name__,message=str(error),
                    qualified_graphs=False,performance_claim=False,release=False))
        print('HELD: '+type(error).__name__+': '+str(error),file=sys.stderr)
        raise SystemExit(1)
