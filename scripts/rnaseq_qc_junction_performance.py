"""Manual complete-input junction gate; preparation does not earn performance."""

import argparse
from collections import Counter
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import re
import resource
import shutil
import signal
import stat
import statistics
import struct
import sys
import tarfile
import tempfile
import time
import traceback
import zipfile

from ecology_beta_real import capture
import private_artifact as transport
import rnaseq_qc_benchmark_inputs as input_contract
import rnaseq_qc_junction_candidate as candidate
import rnaseq_qc_junctions as source
from infercnv_source_snapshot import verify_and_extract
from verify_infercnv_oracle_artifact import _archive_name, _artifact, _extract, _run


ROOT = Path(__file__).resolve().parents[1]
ASSET = 'assets/rnaseq-qc-junction-performance-20261101.json'
WORKFLOW = '.github/workflows/rnaseq-qc-junction-performance.yml'
SOURCE_SHA = '8ea60c35c952a600fa526ebfd3e020e142b01706f50a87a9fbc8818ce9ce5b2b'
NATIVE_SHA = '647d2f5b77ccfb411606cd68af46dc7ece018e86b48f52a6b676d5056f38828c'
BINARY_SHA = 'eb472af80dfa26a00819448f895ccbcceb570de3ad629ac81f9507c7534282ae'
save, identity, check = source.save, source.identity, source.require
LABELS = candidate.LABELS


def load(path):
    return json.loads(Path(path).read_text(), object_pairs_hook=candidate.unique_object)


def read_policy(world=ROOT):
    world = Path(world)
    p = load(world / ASSET)
    limits = p['limits']
    check(p['schema'] == 'rsomics.rnaseq-qc.junction-performance.v1'
          and p['owner_commit'] == 'a618e6ab293e2fb7a113d949197e1bb43ecc7733'
          and p['owner_tree'] == 'e81e89d6e6ae6a8ae7a83a6a6d82cdc9a2aa20bb'
          and p['native']['binary_sha256'] == BINARY_SHA,
          'accepted junction owner/native identity differs')
    check((limits['source_seconds'], limits['native_seconds'], limits['controller_seconds'],
        limits['admission_seconds'], limits['comparison_seconds'], limits['transport_seconds'],
        limits['finalization_seconds'], limits['workflow_seconds']) == (600,300,10800,1500,900,180,2400,14400),
        'frozen complete campaign deadlines differ')
    check((limits['retained_bytes'], limits['tar_bytes'], limits['initial_free_bytes'], limits['memory_bytes'],
        limits['headroom_bytes'], limits['cpus'], limits['output_file_bytes'], limits['max_members'],
        limits['recovery_bytes']) == (8*2**30,9*2**30,26*2**30,8*2**30,8*2**30,4,64*2**20,100000,44*2**30),
        'frozen complete retention/machine policy differs')
    check(p['budgets'] == dict(runtime=2*2**30, inputs=1251403660, native=5668619,
        science=1145336745, metadata=512*2**20), 'complete original bucket budget differs')
    check((p['input']['artifact_id'], p['input']['bytes'], p['input']['uncompressed_bytes'], len(p['input']['files']))
        == (12781035705,492933159,758470501,521)
        and p['input']['sha256'] == '2072ae441c3a2fca21a160db385b612c5596dfb594c50e0cd649edf68c3284a7'
        and (p['native']['artifact_id'], p['native']['bytes'], p['native']['uncompressed_bytes'], len(p['native']['files']))
        == (13074285515,1413965,4199451,226), 'actual full carrier/member population differs')
    check(p['recorder']['function'] == 'capture' and p['recorder']['private_argument'] is None
          and p['recorder']['body']['sha256'] == '31909f2f5b8a780e807566f7fd88b745cbc7346a902ef32aa8ff286b52442829',
          'unchanged reviewed wait4 recorder differs')
    check(p['context'][p['source_receipt']]['sha256'] == SOURCE_SHA
          and p['context'][p['native_receipt']]['sha256'] == NATIVE_SHA, 'adopted receipt pins differ')
    runtime = load(world / p['runtime_asset'])
    check(len(runtime['packages']) == 18 and runtime['python']['version'] == '3.12.10'
          and runtime['image']['manifest_digest'] == 'sha256:a689e29bc3adf4663ef9a141d23081252764d1319c63f591a027bd6fd676f4c1',
          'fresh original-version runtime/image policy differs')
    return p


def verify_context(world, policy):
    world = Path(world)
    for name, row in policy['context'].items():
        check(identity(candidate.owned(world, name)) == row, 'frozen import/authority body differs: ' + name)
    old = load(world / policy['source_receipt'])
    new = load(world / policy['native_receipt'])
    check(old['status'] == 'accepted' and old['source_qualified'] is True
          and new['status'] == 'accepted' and new['native_four_qualified'] is True
          and new['source_receipt_sha256'] == SOURCE_SHA and new['binary_sha256'] == BINARY_SHA
          and new['owner_commit'] == policy['owner_commit'] and new['owner_tree'] == policy['owner_tree']
          and new['source_files'] == 66 and new['native_cli_calls_total'] == 88,
          'actual adopted source/native closure differs')
    return new


def budget_plan():
    d = 338587
    science = 7*(d*49+16384) + 14*(d*(166+48)+2**20)
    allocation = 2*2**30 + 1251403660 + 5668619 + science + 512*2**20
    return dict(science_bytes=science, allocated_bytes=allocation,
                **transport.finalization_space(8*2**30, 9*2**30))


def schedule():
    base = [('source',1), ('native',1), ('native',4)]
    result = [dict(phase=phase,pair=0,engine=e,workers=w) for phase, order in
              [('qualification',base),('warmup',list(reversed(base)))] for e,w in order]
    for pair in range(1,6):
        result.extend(dict(phase='measured',pair=pair,engine=e,workers=w)
                      for e,w in (base if pair%2 else list(reversed(base))))
    return result


def input_paths(private):
    root = Path(private) / 'input-acquisition/artifact/benchmark/inputs'
    return root / 'paired.bam', root / 'genes.bed12'


def command_plan(private, binary, python):
    private = Path(private)
    bam, genes = input_paths(private)
    script = Path(python).parent / 'junction_annotation.py'
    result = []
    for row in schedule():
        name = f'{row["phase"]}-{row["pair"]}-{row["engine"]}-{row["workers"]}'
        directory = private / 'runs' / name
        if row['engine'] == 'source':
            argv = [python,'-B',script,'-i',bam,'-r',genes,'-q','30','-m','50',
                    '--skip-plot','--skip-bed','--skip-interact','-o',directory/'observed']
            timeout = 600
        else:
            argv = [binary,'junctions','--bam',bam,'--genes',genes,'--out',directory/'report',
                    '--profile','rseqc-5.0.5','--mapq','30','--min-intron','50','--threads',str(row['workers'])]
            timeout = 300
        result.append(dict(row,name=name,argv=list(map(str,argv)),timeout=timeout))
    return result


def evaluate(outcomes):
    expected = schedule()
    check(len(outcomes) == 21 and all(all(actual[k] == wanted[k] for k in wanted)
          and actual['returncode'] == 0 and actual['compatible'] is True and actual['group_empty'] is True
          for actual,wanted in zip(outcomes,expected)), 'missing/failed/incompatible whole command; no performance claim')
    samples = {}
    for engine,workers,key in [('source',1,'source'),('native',1,'native1'),('native',4,'native4')]:
        rows = [r for r in outcomes if r['phase'] == 'measured' and r['engine'] == engine and r['workers'] == workers]
        check(len(rows) == 5, 'five measured repeats required')
        values = {name:[r[name] for r in rows] for name in ('wall','rss_bytes','cpu')}
        check(all(type(v) in (float,int) and math.isfinite(v) and v > 0
                  for name in ('wall','rss_bytes') for v in values[name])
              and all(math.isfinite(v) and v >= 0 for v in values['cpu']), 'invalid process resource observation')
        samples[key] = dict(values, raw_wait4=[r['raw_wait4'] for r in rows])
    median = lambda key,name: statistics.median(samples[key][name])
    wall = median('native1','wall') / median('source','wall')
    rss = median('native1','rss_bytes') / median('source','rss_bytes')
    separated_wall = max(samples['native1']['wall']) < min(samples['source']['wall'])
    separated_rss = max(samples['native1']['rss_bytes']) < min(samples['source']['rss_bytes'])
    throughput = separated_wall and wall <= .80
    memory = separated_rss and rss <= .75 and wall <= 1.05
    distributions = {key:{name:dict(values=v,minimum=min(v),maximum=max(v),median=statistics.median(v))
                         for name,v in row.items() if name != 'raw_wait4'} for key,row in samples.items()}
    for key,row in samples.items():
        for name in ('user_seconds','system_seconds','input_blocks','output_blocks','minor_faults','major_faults'):
            values = [raw[name] for raw in row['raw_wait4']]
            check(all(type(v) in (float,int) and math.isfinite(v) and v >= 0 for v in values), 'invalid raw wait4 counter')
            distributions[key][name] = dict(values=values,minimum=min(values),maximum=max(values),median=statistics.median(values))
    median_ratios = {key:{name:(row['median']/distributions['source'][name]['median']
                             if distributions['source'][name]['median'] else None)
                         for name,row in distributions[key].items()} for key in ('native1','native4')}
    median_deltas = {key:{name:row['median']-distributions['source'][name]['median']
                         for name,row in distributions[key].items()} for key in ('native1','native4')}
    return dict(passed=throughput or memory,throughput=throughput,memory=memory,wall_ratio=wall,rss_ratio=rss,
        separated_wall=separated_wall,separated_rss=separated_rss,primary='Native1/Source1 ratio of medians',
        native4_scaling=median('native1','wall')/median('native4','wall'),native4_cannot_rescue_primary=True,
        distributions=distributions,median_ratios_vs_source=median_ratios,median_deltas_vs_source=median_deltas,
        zero_source_median_ratio_is_unavailable=True,raw_wait4={k:v['raw_wait4'] for k,v in samples.items()},
        rss_is_process_peak_not_aggregate=True,io_counters_are_not_exact_storage_bytes=True,cold_cache_proven=False)


def extract_original(carrier, output, pin):
    carrier,output = Path(carrier),Path(output)
    check(identity(carrier) == {k:pin[k] for k in ('bytes','sha256')}, 'complete original carrier differs')
    with zipfile.ZipFile(carrier) as archive:
        names, files = set(), {}
        for entry in archive.infolist():
            name = _archive_name(entry.filename,entry.is_dir())
            kind = stat.S_IFMT(entry.external_attr >> 16) if entry.create_system == 3 else 0
            check(entry.orig_filename == entry.filename and not entry.flag_bits&1
                  and kind in (0,stat.S_IFDIR if entry.is_dir() else stat.S_IFREG)
                  and name not in names, 'unsafe/duplicate/special original ZIP member')
            names.add(name)
            if not entry.is_dir():
                files[name] = entry
        check(set(files) == set(pin['files']) and sum(e.file_size for e in files.values()) == pin['uncompressed_bytes']
              and all(not any('/'.join(n.split('/')[:i]) in files for i in range(1,len(n.split('/')))) for n in names),
              'complete original member closure differs')
        check(archive.testzip() is None, 'original ZIP CRC differs')
        for name,entry in files.items():
            check(entry.file_size == pin['files'][name]['bytes'], 'original member size differs')
            with archive.open(entry) as stream:
                check(hashlib.file_digest(stream,'sha256').hexdigest() == pin['files'][name]['sha256'],
                      'original member body differs: '+name)
        _extract(archive,output)
    return dict(members=len(files),bytes=pin['uncompressed_bytes'],carrier=identity(carrier))


def source_result(directory):
    directory = Path(directory)
    primary = candidate.owned(directory,'observed.junction.xls')
    rows = source.parse_primary(primary.read_text())
    check(len(rows) <= 338587, 'source accepted distinct count exceeds raw-N bound')
    stdout = candidate.owned(directory,'stdout.txt').read_text()
    stderr = candidate.owned(directory,'stderr.txt').read_text()
    counts = {}
    for key,label in LABELS.items():
        values = re.findall(r'^'+re.escape(label)+r':\s*(\d+)\s*$',stderr,re.MULTILINE)
        check(len(values) == 1, 'missing/duplicate actual full source counter: '+label)
        counts[key] = int(values[0])
    check(re.findall(r'^total = (\d+)$',stdout,re.MULTILINE) == [str(counts['total_events'])]
          and counts['total_events'] <= 338587 and counts['total_junctions'] == len(rows)
          and sum(r['read_count'] for r in rows) == counts['total_events']-counts['filtered_events'],
          'source event/support/distinct accounting differs')
    for cls,prefix in [('annotated','known'),('partial_novel','partial_novel'),('complete_novel','novel')]:
        check(sum(r['read_count'] for r in rows if r['annotation'] == cls) == counts[prefix+'_events']
              and sum(r['annotation'] == cls for r in rows) == counts[prefix+'_junctions'],
              'source class accounting differs')
    check(not re.search(r'Traceback|Invalid bed line|Warning:|No splice',stderr), 'source diagnostic is not full success')
    plot = candidate.owned(directory,'observed.junction_plot.r').read_text()
    check('events=c(' in plot and 'junction=c(' in plot
          and {p.name for p in directory.glob('observed*')} == {'observed.junction.xls','observed.junction_plot.r'},
          'complete declared source output set differs')
    return dict(rows=rows,counts=counts)


def native_result(directory, expected, workers):
    directory = Path(directory)
    report = load(candidate.owned(directory,'report.json'))
    check(report['schema_version'] == '1.0' and report['profile'] == 'rseqc-5.0.5'
          and report['sample'] == 'paired' and report['metrics'] == ['junctions']
          and report['junctions'] == expected, 'native full rows/order/support/classes/counters differ')
    lines = candidate.owned(directory,'junctions.tsv').read_text().splitlines()
    check(lines[0] == 'chrom\tintron_start\tintron_end\tread_count\tannotation', 'native detail header differs')
    rows = []
    for line in lines[1:]:
        chrom,start,end,count,annotation = line.split('\t')
        rows.append(dict(chrom=chrom,start=int(start),end=int(end),read_count=int(count),annotation=annotation))
    check(rows == expected['rows'], 'native detail rows differ')
    lines = candidate.owned(directory,'junctions-summary.tsv').read_text().splitlines()
    check(lines[0] == 'metric\tvalue' and len(lines) == 10, 'native summary population differs')
    values = [line.split('\t') for line in lines[1:]]
    check(len({k for k,v in values}) == 9 and {k:int(v) for k,v in values} == expected['counts'], 'native TSV counters differ')
    manifest = load(candidate.owned(directory,'manifest.json'))
    policy = manifest['policy']['junctions']
    check(manifest['status'] == 'complete' and manifest['profile'] == 'rseqc-5.0.5'
          and manifest['selected_metrics'] == ['junctions'] and 'junctions' not in manifest['omitted_metrics']
          and manifest['bam_traversals'] == 1 and manifest['original_records'] == 9810963
          and manifest['inflate_workers'] == workers and policy['excluded_flags'] == 0x704
          and policy['minimum_mapq'] == 30 and policy['minimum_intron'] == 50
          and policy['omitted_outputs'] == ['R','PDF','BED12','Interact','junction-saturation'],
          'native complete input/worker/metric policy differs')
    names = {'report.json','junctions.tsv','junctions-summary.tsv'}
    check(len(manifest['outputs']) == 3 and {r['path'] for r in manifest['outputs']} == names
          and {p.name for p in directory.iterdir()} == names|{'manifest.json'}, 'native atomic output set differs')
    for row in manifest['outputs']:
        check(identity(candidate.owned(directory,row['path'])) == {k:row[k] for k in ('bytes','sha256')}, 'native manifest body differs')
    return {name:identity(directory/name) for name in sorted(names)}


def capacity(private, policy=None):
    private = Path(private)
    policy = read_policy() if policy is None else policy
    buckets = dict.fromkeys(policy['budgets'],0)
    members = 0
    for path in private.rglob('*'):
        observed = path.lstat()
        members += 1
        if path.is_symlink():
            check(path.resolve().is_relative_to(private.resolve()), 'evidence symlink escapes private tree')
            continue
        if path.is_dir(): continue
        check(stat.S_ISREG(observed.st_mode), 'special evidence member')
        parts = path.relative_to(private).parts
        bucket = 'metadata'
        if parts[0] == 'runtime': bucket = 'runtime'
        elif parts[0] in ('input-acquisition','native-acquisition') and (parts[1] == 'artifact' or path.name == 'artifact.zip'):
            bucket = 'inputs' if parts[0] == 'input-acquisition' else 'native'
        elif parts[0] == 'runs' and (path.name in ('report.json','manifest.json','junctions.tsv','junctions-summary.tsv')
                or path.name.endswith(('.junction.xls','.junction_plot.r'))): bucket = 'science'
        buckets[bucket] += observed.st_size
    check(members < policy['limits']['max_members']
          and all(buckets[k] <= policy['budgets'][k] for k in buckets)
          and sum(buckets.values()) <= policy['limits']['retained_bytes'], 'RNA retention/output quota reached; HOLD without deletion')
    check(shutil.disk_usage(private).free >= budget_plan()['required_free_bytes'], 'cipher/headroom reserve reached; HOLD')
    return dict(buckets=buckets,members=members)


def bind_context(private, policy):
    paths = sorted(set(policy['context']) | {ASSET,WORKFLOW,'scripts/rnaseq_qc_junction_performance.py',
                                           'scripts/test_rnaseq_qc_junction_performance.py'})
    for name in paths:
        destination = Path(private)/'context'/name
        destination.parent.mkdir(parents=True,exist_ok=True)
        shutil.copy2(ROOT/name,destination)
    bindings = {name:identity(ROOT/name) for name in paths}
    save(Path(private)/'context-bindings.json',bindings)
    return bindings


def hosted(reset_affinity=True):
    policy = read_policy()
    runtime = load(ROOT/policy['runtime_asset'])
    check(os.environ.get('GITHUB_ACTIONS') == 'true' and os.environ.get('GITHUB_EVENT_NAME') == 'workflow_dispatch'
          and os.environ.get('GITHUB_REPOSITORY') == 'omics-rust/rsomics-world'
          and platform.system() == 'Linux' and platform.machine() == 'x86_64'
          and os.environ.get('JUNCTION_IMAGE_DIGEST') == runtime['image']['manifest_digest'], 'actual pinned manual Linux job required')
    session = Path(os.environ['JUNCTION_SESSION']).resolve(strict=True)
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    check(session != runner and session.is_relative_to(runner) and session.stat().st_dev == runner.stat().st_dev,
          'session escapes runner-temp device')
    for name in source.CACHES:
        check(Path(os.environ[name]).resolve(strict=True).is_relative_to(session/'private'), 'cache escapes private root')
    cpus = list(map(int,os.environ['JUNCTION_PERFORMANCE_CPUS'].split(',')))
    check(len(cpus) == len(set(cpus)) == 4, 'four declared distinct allowed CPUs required')
    if reset_affinity:
        os.sched_setaffinity(0,cpus)
    else:
        check(set(os.sched_getaffinity(0)) <= set(cpus), 'child affinity escapes declared allowance')
    return session,session/'private',policy


def initialize():
    check(os.environ.get('GITHUB_ACTIONS') == 'true' and os.environ.get('GITHUB_EVENT_NAME') == 'workflow_dispatch'
          and os.environ.get('JUNCTION_PERFORMANCE_RECOVERY_ADMITTED') == 'true',
          'manual fresh44GiB recovery attestation required before launch; boolean is not storage proof')
    policy = read_policy()
    verify_context(ROOT,policy)
    session = Path(tempfile.mkdtemp(prefix='rnaseq-junction-performance-',dir=Path(os.environ['RUNNER_TEMP']).resolve(strict=True)))
    session.chmod(0o700)
    for name in ('private','cipher'):(session/name).mkdir(mode=0o700)
    cpus = sorted(os.sched_getaffinity(0))[:4]
    check(len(cpus) == 4 and shutil.disk_usage(session).free >= policy['limits']['initial_free_bytes'], 'fresh26GiB/four CPUs unavailable')
    env = dict(JUNCTION_SESSION=str(session),JUNCTION_PERFORMANCE_CPUS=','.join(map(str,cpus)))
    for name in source.CACHES:
        path = session/'private/cache'/name
        path.mkdir(parents=True,mode=0o700)
        env[name] = str(path)
    with Path(os.environ['GITHUB_ENV']).open('a') as stream:
        for key,value in env.items():
            stream.write(f'{key}={value}\n');os.environ[key] = value
    _,private,_ = hosted()
    check(resource.getrlimit(resource.RLIMIT_AS) == (resource.RLIM_INFINITY,resource.RLIM_INFINITY), 'unexpected inherited address-space limit')
    save(private/'request.json',dict(started_monotonic=time.monotonic(),head_sha=os.environ['GITHUB_SHA'],
        run_id=int(os.environ['GITHUB_RUN_ID']),run_attempt=int(os.environ['GITHUB_RUN_ATTEMPT']),
        policy=identity(ROOT/ASSET),retention=budget_plan(),recovery_attested=True,
        recovery_storage_verified_by_hosted_job=False,cache_cold_proven=False))
    bind_context(private,policy)


def transport_admission():
    session,private,policy = hosted()
    capacity(private,policy)
    check(identity(ROOT/policy['recipient']) == policy['context'][policy['recipient']], 'fixed public recipient differs')
    tool = transport.prepare_tool('linux-amd64',private/'transport-tool')
    receipt = transport.encrypt(tool,ROOT/policy['recipient'],private/'transport-probe.age',
        producer=[sys.executable,'-B','-c','import sys;sys.stdout.buffer.write(b"junction-performance transport admission\\n")'],
        plaintext_limit=128,context=dict(repository='omics-rust/rsomics-world',product='rsomics-rnaseq-qc',phase='transport-admission'))
    save(private/'transport-admission.json',receipt)


def acquire_original(private, kind, pin, token):
    directory = Path(private)/(kind+'-acquisition')
    directory.mkdir()
    base = 'https://api.github.com/repos/omics-rust/rsomics-world/actions/'
    run = source.download(dict(url=base+f'runs/{pin["run_id"]}/attempts/{pin["run_attempt"]}'),
                          directory/'run.json',maximum=2**21,token=token)
    artifact = source.download(dict(url=base+f'artifacts/{pin["artifact_id"]}'),
                               directory/'artifact.json',maximum=2**21,token=token)
    carrier = source.download(dict(url=base+f'artifacts/{pin["artifact_id"]}/zip',bytes=pin['bytes'],sha256=pin['sha256']),
                              directory/'artifact.zip',token=token)
    receipt = dict(pin,artifact_sha256=pin['sha256'])
    _run(run,receipt)
    _artifact(artifact,receipt,carrier)
    verified = extract_original(carrier,directory/'artifact',pin)
    save(directory/'verification.json',dict(pin={k:v for k,v in pin.items() if k != 'files'},**verified))
    return directory/'artifact'


def empty_group(group):
    check(type(group) is int and group > 1 and group != os.getpgrp(), 'unowned process-group identity')
    try:
        os.killpg(group,0)
    except ProcessLookupError:
        return True
    return False


def measured(private, name, argv, workers, timeout, policy, parent='runs'):
    private = Path(private)
    root = private/parent
    root.mkdir(parents=True,exist_ok=True)
    before = capacity(private,policy)
    call, failure = None,None
    try:
        call = capture(root,name,argv,workers=workers,timeout=timeout,private=None,
                       file_limit=policy['limits']['output_file_bytes'])
    except BaseException as error:
        failure = error
    directory = root/name
    status = load(directory/'status.json') if (directory/'status.json').is_file() else None
    process = load(directory/'process.json') if (directory/'process.json').is_file() else None
    group_empty, actions = False,[]
    if process and process.get('pid') == process.get('process_group'):
        group = process['pid']
        group_empty = empty_group(group)
        if not group_empty:
            for signum in (signal.SIGTERM,signal.SIGKILL):
                if empty_group(group): break
                os.killpg(group,signum)
                actions.append(signal.Signals(signum).name)
                end = time.monotonic()+2
                while not empty_group(group) and time.monotonic()<end: time.sleep(.02)
            group_empty = empty_group(group)
            failure = failure or RuntimeError('owned process left descendants; HOLD')
    after = capacity(private,policy)
    record = dict(before=before,after=after,group_empty=group_empty,cleanup_actions=actions,
                  raw_status=status,error=None if failure is None else f'{type(failure).__name__}: {failure}')
    save(directory/'rna-custody.json',record)
    check(failure is None and status is not None and status['returncode'] == 0
          and status['failure'] is None and group_empty and not actions, 'required complete RNA command failed; originals retained: '+name)
    for events in ('memory_events_before','memory_events_after'):
        check(status[events] is not None and not re.search(r'^oom(?:_kill|_group_kill)? [1-9]',status[events],re.MULTILINE),
              'OOM is not a performance success')
    return dict(status,group_empty=group_empty)


def admit_native(private, policy, token):
    private = Path(private)
    accepted = verify_context(ROOT,policy)
    artifact = acquire_original(private,'native',policy['native'],token)
    integrity = load(artifact/'junction-integrity.json')
    check(set(integrity['files'])|{'junction-integrity.json'} == set(policy['native']['files'])
          and integrity['finalization_status'] == 'complete', 'accepted native integrity inventory differs')
    for name,row in integrity['files'].items():
        check(identity(candidate.owned(artifact,name)) == row, 'accepted native retained body differs: '+name)
    final = load(artifact/'junction-finalization.json')
    check(final['status'] == 'complete' and final['errors'] == [] and final['owner_commit'] == policy['owner_commit']
          and final['source_receipt_sha256'] == SOURCE_SHA, 'accepted actual native finalization differs')
    for name,digest in [('source.tar.gz',accepted['source_archive_sha256']),('files.sha256',accepted['source_manifest_sha256']),
                        ('Cargo.lock',accepted['lock_sha256']),('junction-authority/receipt.json',SOURCE_SHA)]:
        check(identity(artifact/name)['sha256'] == digest, 'accepted native/source/lock body differs: '+name)
    runtime = private/'native-runtime'
    runtime.mkdir()
    owner = runtime/'source'
    verify_and_extract(artifact,owner)
    authority = candidate.authority(ROOT)
    with tarfile.open(artifact/'source.tar.gz') as archive:
        members = archive.getmembers()
        check(len(members) == 66 and {m.name for m in members} == set(authority['source_files']), 'native Git TAR population differs')
        for member in members:
            check(member.isfile() and member.mode == authority['source_files'][member.name]['mode'], 'native Git TAR mode/type differs')
            (owner/member.name).chmod(member.mode)
    candidate.verify_source(owner,authority)
    check(identity(artifact/'junction-authority/asset.json')['sha256'] == identity(ROOT/policy['candidate_asset'])['sha256'],
          'captured native source authority differs')
    original = artifact/'native-binary/rsomics-rnaseq-qc-linux-x86_64'
    check(identity(original) == dict(bytes=2870600,sha256=BINARY_SHA), 'actual accepted native ELF differs')
    binary = runtime/'rsomics-rnaseq-qc'
    shutil.copyfile(original,binary)
    binary.chmod(0o700)
    candidate.binary_identity(binary,'linux-x86_64')
    measured(private,'loader',['ldd',str(binary)],1,30,policy,parent='native-runtime/calls')
    text = (runtime/'calls/loader/stdout.txt').read_text()
    check('not found' not in text and 'statically linked' not in text, 'Bookworm native provider resolution failed')
    providers = {}
    for raw in re.findall(r'(?:=>\s+|^\s*)(/\S+)',text,re.MULTILINE):
        path = Path(raw).resolve(strict=True)
        body = identity(path)
        destination = runtime/'providers'/(body['sha256']+'-'+path.name)
        destination.parent.mkdir(exist_ok=True)
        if not destination.exists(): shutil.copy2(path,destination)
        providers[raw] = dict(resolved=str(path),retained=str(destination.relative_to(private)),**body)
    check(len(providers) >= 4 and any('libc.so' in p for p in providers)
          and any('libm.so' in p for p in providers) and any('libgcc_s.so' in p for p in providers)
          and any('ld-linux' in p for p in providers), 'complete native loader/provider identities absent')
    measured(private,'version',[str(binary),'--version'],1,30,policy,parent='native-runtime/calls')
    check((runtime/'calls/version/stdout.txt').read_text().strip() == 'rsomics-rnaseq-qc 0.1.0', 'actual Bookworm native version differs')
    fixture = owner/'tests/fixtures/junctions/informative-single'
    expected = candidate.source_expected(fixture,'informative-single')
    for workers in (1,4):
        name = 'compact-'+str(workers)
        output = runtime/'calls'/name/'report'
        argv = [binary,'junctions','--bam',fixture/'input.bam','--genes',fixture/'genes.bed12','--out',output,
                '--profile','rseqc-5.0.5','--mapq','30','--min-intron','50','--threads',str(workers)]
        measured(private,name,list(map(str,argv)),workers,30,policy,parent='native-runtime/calls')
        candidate.verify_output(output,expected,'rseqc-5.0.5','junctions',workers,16)
    check(identity(binary)['sha256'] == BINARY_SHA, 'admitted ELF changed')
    save(runtime/'admission.json',dict(binary=identity(binary),native_rebuilt=False,actual_bookworm_execution=True,
        providers=providers,provider_scope='actual ldd resolution and compact execution; not per-call dynamic mapping tracing',
        accepted_native_receipt=identity(ROOT/policy['native_receipt']),source_files=66,compact_workers=[1,4]))
    return binary


def qualify_inputs(private, policy):
    import pysam

    private = Path(private)
    bam,genes = input_paths(private)
    old = bam.parent
    qualification = load(old/'qualification.json')
    before = {name:identity(old/name) for name in ('paired.bam','genes.bed12','annotation-exclusions.jsonl','originals/hg19_RefSeq.bed.gz')}
    for name,row in before.items():
        check(row == policy['input']['files']['benchmark/inputs/'+name], 'actual fixed input differs: '+name)
    counts = input_contract.empty_record_counts()
    previous = None
    with pysam.AlignmentFile(str(bam),'rb') as stream:
        references = dict(zip(stream.references,stream.lengths))
        check(references == qualification['header']['references'] and len(references) == 25, 'actual full header differs')
        for record in stream.fetch(until_eof=True):
            chromosome = stream.get_reference_name(record.reference_id) if record.reference_id >= 0 else '*'
            if record.reference_id >= 0 and record.reference_start >= 0:
                coordinate = record.reference_id,record.reference_start
                check(previous is None or coordinate >= previous, 'actual whole BAM is not coordinate monotonic')
                previous = coordinate
            input_contract.observe_record(counts,record,chromosome)
    counts = input_contract.ordered_counts(counts)
    original = load(old/'record-counts.json')
    check(counts == original['selected'] == original['source'] and counts['records'] == 9810963
          and counts['cigar_operations'] == {'M':10149550,'N':338587}, 'unchanged full record/CIGAR inventory differs')
    output = private/'input-validation'
    output.mkdir()
    annotation = input_contract.derive_annotation(old/'originals/hg19_RefSeq.bed.gz',references,
        output/'genes.bed12',output/'annotation-exclusions.jsonl','whole')
    check((annotation['source_transcripts'],annotation['selected_transcripts'],annotation['excluded_transcripts'],
        annotation['distinct_names'],annotation['duplicate_name_rows']) == (61177,58384,2793,58322,62)
        and annotation['exclusion_reasons'] == {'reference_absent_from_bam':2793}
        and identity(output/'genes.bed12') == before['genes.bed12']
        and identity(output/'annotation-exclusions.jsonl') == before['annotation-exclusions.jsonl'],
        'complete original annotation/duplicate/exclusion derivation differs')
    check(before == {name:identity(old/name) for name in before}, 'fixed original input changed during validation')
    save(output/'qualification.json',dict(records=counts,annotation=annotation,inputs=before,
        scope='untimed full fixed-body validation; no rewriting/indexing/filtering of timed inputs'))
    return before


def reserve_call(started, now, remaining, comparison_spent, policy):
    allowance = policy['limits']['comparison_seconds']-comparison_spent
    check(allowance >= 0 and now+remaining+allowance <= started+policy['limits']['controller_seconds'],
          'insufficient frozen full-call/comparison allowance; HOLD')


def compare_call(private, name):
    private = Path(private)
    plan = load(private/'before-outcome-plan.json')['commands']
    row = next(r for r in plan if r['name'] == name)
    reference_dir = private/'runs'/plan[0]['name']
    expected = source_result(reference_dir)
    directory = private/'runs'/name
    if row['engine'] == 'source':
        check(source_result(directory) == expected, 'actual complete source rows/counters changed across repeats')
        bodies = {n:identity(directory/n) for n in ('observed.junction.xls','observed.junction_plot.r')}
    else:
        bodies = native_result(directory/'report',expected,row['workers'])
    record = dict(compatible=True,counters=expected['counts'],rows=len(expected['rows']),output_bodies=bodies,
                  source_reference=str(reference_dir.relative_to(private)),source_primary=identity(reference_dir/'observed.junction.xls'))
    save(directory/'comparison-result.json',record)
    return record


def execute():
    session,private,policy = hosted()
    request = load(private/'request.json')
    started = request['started_monotonic']
    def interrupt(signum,frame):
        raise TimeoutError('owned complete junction controller deadline/termination; HOLD')
    for signum in (signal.SIGALRM,signal.SIGTERM,signal.SIGINT): signal.signal(signum,interrupt)
    signal.setitimer(signal.ITIMER_REAL,max(.001,started+policy['limits']['admission_seconds']-time.monotonic()))
    try:
        verify_context(ROOT,policy)
        token = os.environ.pop('GH_TOKEN')
        check(bool(token), 'authenticated artifact acquisition token is absent')
        runtime_policy = load(ROOT/policy['runtime_asset'])
        source.admit_machine(private,dict(image=runtime_policy['image'],limits=policy['limits']))
        python = source.prepare_runtime(private,runtime_policy)
        capacity(private,policy)
        binary = admit_native(private,policy,token)
        acquire_original(private,'input',policy['input'],token)
        del token
        capacity(private,policy)
        measured(private,'qualify-complete-original-inputs',[str(python),'-B',__file__,'qualify-inputs','--root',str(private)],
                 1,600,policy,parent='admission-calls')
        qualification = load(private/'input-validation/qualification.json')
        check(time.monotonic() <= started+policy['limits']['admission_seconds'], 'fresh admission exceeded25min; HOLD')
        signal.setitimer(signal.ITIMER_REAL,max(.001,started+policy['limits']['controller_seconds']-time.monotonic()))
        plan = command_plan(private,binary,python)
        save(private/'before-outcome-plan.json',dict(commands=plan,inputs=qualification['inputs'],source_receipt_sha256=SOURCE_SHA,
            native_receipt_sha256=NATIVE_SHA,binary=identity(binary),recorder=policy['recorder'],limits=policy['limits'],
            cpu_policy=dict(source=1,native1=1,native4=4,cores=os.environ['JUNCTION_PERFORMANCE_CPUS']),
            performance_gate='disjoint wall and median<=.80 OR disjoint RSS and median<=.75 plus wall<=1.05'))
        outcomes,reference,native_reference = [],None,None
        scientific_seconds = 0.0
        measurement_started = time.monotonic()
        script = Path(python).parent/'junction_annotation.py'
        script_body = identity(script)
        for index,row in enumerate(plan):
            comparison_spent = time.monotonic()-measurement_started-scientific_seconds
            reserve_call(started,time.monotonic(),sum(r['timeout'] for r in plan[index:]),comparison_spent,policy)
            old = input_paths(private)[0].parent
            check(qualification['inputs'] == {name:identity(old/name) for name in qualification['inputs']}
                  and identity(binary)['sha256'] == BINARY_SHA and identity(script) == script_body,
                  'admitted whole input/script/ELF changed before call')
            call = measured(private,row['name'],row['argv'],row['workers'],row['timeout'],policy)
            scientific_seconds += call['wall']
            directory = private/'runs'/row['name']
            measured(private,row['name'],[sys.executable,'-B',__file__,'compare','--root',str(private),'--name',row['name']],
                     1,120,policy,parent='comparison-calls')
            comparison = load(directory/'comparison-result.json')
            output_bodies = comparison['output_bodies']
            if reference is None:
                check(row['engine'] == 'source' and row['phase'] == 'qualification', 'first reference is not actual source qualification')
                reference = dict(path=comparison['source_reference'],counts=comparison['counters'],primary=comparison['source_primary'])
                save(private/'full-source-reference.json',dict(reference,input_identity=qualification['inputs'],
                    source_script=script_body,scope='actual full source observation; rows retained in original primary'))
            check(comparison['source_primary'] == reference['primary'] and comparison['counters'] == reference['counts'],
                  'actual complete source reference changed')
            if row['engine'] == 'native':
                check(native_reference is None or native_reference == output_bodies, 'native report/detail/summary bytes differ across workers/repeats')
                native_reference = output_bodies
            raw = call['raw_wait4']
            result = dict(row,wall=call['wall'],cpu=raw['user_seconds']+raw['system_seconds'],rss_bytes=raw['rss_bytes'],
                raw_wait4=raw,returncode=call['returncode'],group_empty=call['group_empty'],compatible=True,
                counters=comparison['counters'],output_bodies=output_bodies,
                started_monotonic_ns=call['started_monotonic_ns'],finished_monotonic_ns=call['finished_observed_monotonic_ns'])
            check(not outcomes or outcomes[-1]['finished_monotonic_ns'] <= result['started_monotonic_ns'], 'whole-call intervals overlap')
            save(directory/'comparison.json',result)
            outcomes.append(result)
            capacity(private,policy)
            check(time.monotonic()-measurement_started-scientific_seconds <= policy['limits']['comparison_seconds'],
                  'untimed full comparison/metadata allowance exceeded15min; HOLD')
        decision = evaluate(outcomes)
        save(private/'performance.json',dict(schema='rsomics.rnaseq-qc.junction-performance.v1',status='complete' if decision['passed'] else 'rejected_no_advantage',
            decision=decision,commands=outcomes,binary_sha256=BINARY_SHA,source_receipt_sha256=SOURCE_SHA,
            native_receipt_sha256=NATIVE_SHA,all_originals_retained=True,
            scope='one warm-cache full-input junction workload; no old-five replay, exact aggregate RSS, cold-cache or universal scaling claim'))
        check(decision['passed'], 'complete fixed workload has no required Native1 advantage; HOLD')
        check(identity(binary)['sha256'] == BINARY_SHA, 'measured original ELF changed')
        save(private/'execution-status.json',dict(status='complete-and-fixed-gate-passed',whole_calls=21,
            full_original_records=9810963,prepared_annotation_rows=58384))
    except BaseException:
        if not (private/'execution-status.json').exists():
            save(private/'execution-status.json',dict(status='HOLD',traceback=traceback.format_exc(),
                source_failure_is_not_native_win=True,performance_claim=False,available_originals_retained=True))
        raise
    finally:
        signal.setitimer(signal.ITIMER_REAL,0)


def tar_bound(members):
    return sum(row.get('bytes',0) for row in members)+len(members)*8192+10240


def seal_manifest(private, policy, bindings=None):
    private = Path(private)
    failures = []
    try:
        if bindings is None: bindings = load(private/'context-bindings.json')
        check(bindings == {name:identity(ROOT/name) for name in bindings}, 'controller/authority/import closure changed')
        capacity(private,policy)
    except (OSError,ValueError) as error:
        failures.append(str(error))
    original = source.inventory(private)
    execution = load(private/'execution-status.json') if (private/'execution-status.json').exists() else dict(status='HOLD',reason='execute did not complete')
    save(private/'artifact-manifest.json',dict(schema='rsomics.rnaseq-qc.junction-performance-originals.v1',files=original,
        execution=execution,finalization_failures=failures,source_receipt_sha256=SOURCE_SHA,native_receipt_sha256=NATIVE_SHA,
        policy=identity(ROOT/ASSET),all_available_originals_retained=True,source_failure_is_not_native_win=True,
        scope='new complete junction workload only; no publication or old-five pipeline claim'))
    members = source.inventory(private)
    check(sum(r['bytes'] for r in members) <= policy['limits']['retained_bytes']
          and len(members) < policy['limits']['max_members'] and tar_bound(members) <= policy['limits']['tar_bytes'],
          'complete available finalization exceeds frozen envelope; no deletion/plaintext fallback')
    return members


def finalize():
    session,private,policy = hosted()
    tool = transport.prepare_tool('linux-amd64',private/'transport-tool')
    save(private/'final-cgroup.json',source.cgroup())
    members = seal_manifest(private,policy)
    save(session/'sealed-members.json',members)
    check(shutil.disk_usage(private).free >= transport.finalization_space(sum(r['bytes'] for r in members),tar_bound(members))['required_free_bytes'],
          'actual streamed finalization capacity unavailable')
    receipt = transport.encrypt(tool,ROOT/policy['recipient'],session/'cipher/evidence.tar.age',
        producer=[sys.executable,'-B',__file__,'tar','--root',str(private),'--members',str(session/'sealed-members.json')],
        plaintext_limit=tar_bound(members),context=dict(repository='omics-rust/rsomics-world',product='rsomics-rnaseq-qc',
            run_id=int(os.environ['GITHUB_RUN_ID']),run_attempt=int(os.environ['GITHUB_RUN_ATTEMPT']),
            head_sha=os.environ['GITHUB_SHA'],workflow_path=WORKFLOW,phase='junction-performance'))
    save(session/'cipher/transport.json',receipt)
    if os.environ.get('GITHUB_OUTPUT'):
        with Path(os.environ['GITHUB_OUTPUT']).open('a') as output:
            output.write('sealed=true\n')
    check(not load(private/'artifact-manifest.json')['finalization_failures'],
          'available originals encrypted but finalization integrity/retention remains HOLD')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action',choices=('init','transport','execute','qualify-inputs','compare','finalize','tar'))
    parser.add_argument('--root',type=Path)
    parser.add_argument('--members',type=Path)
    parser.add_argument('--name')
    args = parser.parse_args()
    if args.action == 'init': initialize()
    elif args.action == 'transport':
        signal.signal(signal.SIGALRM,lambda signum,frame: (_ for _ in ()).throw(TimeoutError('transport admission deadline')))
        signal.alarm(180)
        try: transport_admission()
        finally: signal.alarm(0)
    elif args.action == 'execute': execute()
    elif args.action == 'finalize':
        signal.signal(signal.SIGALRM,lambda signum,frame: (_ for _ in ()).throw(TimeoutError('finalization deadline')))
        signal.alarm(2400)
        try: finalize()
        finally: signal.alarm(0)
    else:
        _,private,policy = hosted(reset_affinity=False)
        check(args.root.resolve(strict=True) == private, 'child private root differs')
        if args.action == 'qualify-inputs': qualify_inputs(private,policy)
        elif args.action == 'compare': compare_call(private,args.name)
        else: source.write_tar(private,load(args.members))


if __name__ == '__main__':
    main()
