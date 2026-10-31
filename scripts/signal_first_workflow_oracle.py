#!/usr/bin/env python3
"""Freeze and capture a hosted, pinned deepTools first-six source workflow."""

import argparse
import gzip
import hashlib
import importlib
import importlib.metadata
import inspect
import io
import json
import math
import os
from pathlib import Path, PurePosixPath
import platform
import re
import shutil
import subprocess
import struct
import sys
import tarfile
import traceback
import urllib.request
import venv
import zipfile


COMMIT = 'ea0f68bb4a1587d713dacb3791861308751ef7d0'
SOURCE_SHA = '2daf06abc8cf2df42f7e0ecacb3784ee95de1db4dd887f69f22a29f27e202369'
OPERATIONS = ['filter', 'track', 'compare', 'average', 'summarize', 'matrix']
THREADS = ('OMP_NUM_THREADS', 'OPENBLAS_NUM_THREADS', 'MKL_NUM_THREADS', 'BLIS_NUM_THREADS',
           'VECLIB_MAXIMUM_THREADS', 'NUMEXPR_NUM_THREADS')
ENTRYPOINTS = dict(bamCoverage='bamCoverage', bamCompare='bamCompare', bigwigCompare='bigwigCompare',
    bigwigAverage='bigwigAverage', multiBamSummary='multiBamSummary', multiBigwigSummary='multiBigwigSummary',
    computeMatrix='computeMatrix', alignmentSieve='alignmentSieve')


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
    return digest.hexdigest()


def encode(value):
    if isinstance(value, io.IOBase):
        return {'file': str(value.name), 'mode': value.mode, 'closed': value.closed}
    if isinstance(value, float) and not math.isfinite(value):
        return {'nonfinite': 'nan' if math.isnan(value) else '+inf' if value > 0 else '-inf'}
    if isinstance(value, dict):
        return {str(key): encode(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [encode(item) for item in value]
    if hasattr(value, 'tolist'):
        return encode(value.tolist())
    if isinstance(value, bytes):
        return {'bytes_hex': value.hex()}
    return value


def save(path, value):
    with Path(path).open('x') as stream:
        json.dump(encode(value), stream, indent=2, sort_keys=True, allow_nan=False)
        stream.write('\n')


def capture(output, argv, required=True):
    output = Path(output)
    output.mkdir(parents=True)
    save(output / 'request.json', dict(argv=list(map(str, argv)), cwd=str(Path.cwd()),
        threads={name: os.environ.get(name) for name in THREADS}))
    with (output / 'stdout.txt').open('wb') as stdout, (output / 'stderr.txt').open('wb') as stderr:
        completed = subprocess.run(list(map(str, argv)), stdout=stdout, stderr=stderr)
    record = dict(status=completed.returncode, stdout_sha256=sha256(output / 'stdout.txt'),
                  stderr_sha256=sha256(output / 'stderr.txt'))
    save(output / 'command.json', record)
    if required and completed.returncode != 0:
        raise RuntimeError('required command failed: ' + str(output))
    return record


def authored_inputs():
    headers = '@HD\tVN:1.6\tSO:coordinate\n@SQ\tSN:chr1\tLN:2049\n@SQ\tSN:chr2\tLN:333\n'
    samples = {}
    for sample, copies in (('a', 1), ('b', 2)):
        records = []

        def read(name, flag, chrom, position, cigar='60M', mate='*', mate_position=0, length=0, quality=40):
            size = sum(int(n) for n, op in re.findall(r'(\d+)([MIDNSHP=X])', cigar) if op in 'MIS=X')
            sequence = 'A' * size if size else '*'
            score = 'I' * size if size else '*'
            fields = [name, flag, chrom, position, quality, cigar, mate, mate_position, length,
                      sequence, score, 'NM:i:0', 'RG:Z:authored']
            records.append((chrom, position, name, '\t'.join(map(str, fields))))

        for copy in range(copies):
            for i, start in enumerate(range(11, 1800, 60)):
                name = f'{sample}-baseline-{copy}-{i}'
                read(name, 99, 'chr1', start, mate='=', mate_position=start + 100, length=160)
                read(name, 147, 'chr1', start + 100, mate='=', mate_position=start, length=-160)
        for name, position, cigar in (('split', 205, '10M40N10M'), ('deletion', 301, '12M5D8M'),
                ('equal-x', 401, '10=2X8='), ('insertion', 501, '8M4I12M'),
                ('soft-clip', 601, '4S16M'), ('hard-clip', 650, '4H20M')):
            read(sample + '-' + name, 0, 'chr1', position, cigar)
        for name, flag, position, quality in (('secondary', 256, 701, 40),
                ('supplementary', 2048, 801, 40), ('qc-fail', 512, 901, 40),
                ('duplicate-flag', 1024, 1001, 40), ('low-mapq', 0, 1101, 5)):
            read(sample + '-' + name, flag, 'chr1', position, '20M', quality=quality)
        for name, flags, start, distance in (('overlap', (99, 147), 1211, 30),
                ('improper', (65, 129), 1321, 100), ('positional-duplicate', (99, 147), 11, 100)):
            read(sample + '-' + name, flags[0], 'chr1', start, mate='=', mate_position=start + distance, length=distance + 60)
            read(sample + '-' + name, flags[1], 'chr1', start + distance, mate='=', mate_position=start, length=-distance - 60)
        read(sample + '-cross-reference', 65, 'chr1', 1551, '20M', mate='chr2', mate_position=101)
        read(sample + '-reverse-single', 16, 'chr1', 1701, '20M')
        read(sample + '-right-end', 0, 'chr1', 2001, '49M')
        read(sample + '-left-end', 0, 'chr2', 1, '20M')
        read(sample + '-chunk-boundary', 0, 'chr2', 121, '40M')
        read(sample + '-unmapped', 4, '*', 0, '*', quality=0)
        order = {'chr1': 0, 'chr2': 1, '*': 2}
        records.sort(key=lambda row: (order[row[0]], row[1], row[2]))
        samples['authored/' + sample + '.sam'] = (headers + ''.join(row[3] + '\n' for row in records)).encode()
    samples.update({
        'authored/groups.bed': b'chr1\t100\t300\tduplicate\t0\t+\nchr1\t500\t700\tduplicate\t0\t-\n#early\nchr1\t1100\t1300\tlate\t0\t+\nchr1\t1600\t1800\treverse\t0\t-\n#late\n',
        'authored/regions.bed': b'chr1\t0\t200\nchr1\t250\t650\nchr1\t1000\t1400\nchr2\t10\t320\n',
        'authored/exons.bed': b'chr1\t100\t400\tsplice-plus\t0\t+\t100\t400\t0\t2\t50,80,\t0,220,\nchr1\t800\t1100\tsplice-minus\t0\t-\t800\t1100\t0\t2\t50,80,\t0,220,\n',
        'authored/regions.gtf': b'chr1\tauthored\ttranscript\t101\t400\t.\t+\t.\tgene_id "g1"; transcript_id "t1"; deepTools_group "exons";\nchr1\tauthored\texon\t101\t150\t.\t+\t.\tgene_id "g1"; transcript_id "t1";\nchr1\tauthored\texon\t321\t400\t.\t+\t.\tgene_id "g1"; transcript_id "t1";\nchr1\tauthored\ttranscript\t801\t1100\t.\t-\t.\tgene_id "g2"; transcript_id "t2"; deepTools_group "exons";\nchr1\tauthored\texon\t801\t850\t.\t-\t.\tgene_id "g2"; transcript_id "t2";\nchr1\tauthored\texon\t1021\t1100\t.\t-\t.\tgene_id "g2"; transcript_id "t2";\n',
        'authored/blacklist.bed': b'chr1\t500\t550\nchr1\t1000\t1020\n',
    })
    signals = {
        'signed-a': dict(chroms=[['chr1', 131]], intervals=[['chr1', 0, 20, 2.], ['chr1', 20, 40, 0.], ['chr1', 60, 100, -2.], ['chr1', 100, 131, 0.]]),
        'signed-b': dict(chroms=[['chr1', 131]], intervals=[['chr1', 0, 20, -2.], ['chr1', 20, 60, 0.], ['chr1', 60, 100, 4.], ['chr1', 100, 131, 0.]]),
        'short-dictionary': dict(chroms=[['chr1', 121]], intervals=[['chr1', 0, 121, 1.]]),
        'aliased-dictionary': dict(chroms=[['1', 131]], intervals=[['1', 0, 131, 3.]]),
        'zoom': dict(chroms=[['chr1', 20003]], intervals=[['chr1', i * 10, i * 10 + 5, float(i % 7)] for i in range(2000)]),
    }
    samples['authored/signals.json'] = (json.dumps(signals, sort_keys=True, indent=2) + '\n').encode()
    return samples


def prepare_inputs(archive_path, lock_path, output):
    lock = json.loads(Path(lock_path).read_text())
    if sha256(archive_path) != lock['inputs']['sha256']:
        raise ValueError('input archive digest differs')
    output = Path(output)
    if output.exists() or output.is_symlink():
        raise ValueError('input destination exists')
    with tarfile.open(archive_path, 'r:gz') as archive:
        members = archive.getmembers()
        names = [member.name for member in members]
        if len(set(names)) != len(names):
            raise ValueError('duplicate input path')
        for member in members:
            name = member.name
            if not member.isfile() or member.type != tarfile.REGTYPE or str(PurePosixPath(name)) != name or \
                    name.startswith('/') or any(part in ('', '.', '..') for part in name.split('/')) or '\\' in name:
                raise ValueError('unsafe input archive member')
        provenance = json.load(archive.extractfile('provenance.json'))
        if set(names) != set(provenance['files']) | {'provenance.json'}:
            raise ValueError('input inventory differs')
        contents = {member.name: archive.extractfile(member).read() for member in members}
        if any(hashlib.sha256(contents[name]).hexdigest() != row['sha256'] for name, row in provenance['files'].items()):
            raise ValueError('input member digest differs')
        output.mkdir(parents=True)
        for name, data in contents.items():
            path = output / name
            path.parent.mkdir(parents=True, exist_ok=True)
            with path.open('xb') as stream:
                stream.write(data)
    return provenance


def hosted_root(output):
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    output = Path(output).resolve()
    if not output.is_relative_to(runner):
        raise ValueError('hosted output escapes RUNNER_TEMP')
    for name in ('TMPDIR', 'XDG_CACHE_HOME', 'PIP_CACHE_DIR', 'PYTHONUSERBASE', 'MPLCONFIGDIR'):
        path = Path(os.environ[name]).resolve(strict=True)
        if not path.is_relative_to(runner) or path.stat().st_dev != runner.stat().st_dev:
            raise ValueError('hosted cache escapes RUNNER_TEMP: ' + name)
    if shutil.disk_usage(runner).free < 2**30:
        raise ValueError('insufficient hosted external work space')
    return output


def download(item, destination):
    destination = Path(destination)
    request = urllib.request.Request(item['url'], headers={'User-Agent': 'rsomics-signal-source-witness'})
    with urllib.request.urlopen(request, timeout=90) as response, destination.open('xb') as output:
        shutil.copyfileobj(response, output)
    if sha256(destination) != item['sha256']:
        raise ValueError('download checksum differs: ' + destination.name)
    if 'size' in item and destination.stat().st_size != item['size']:
        raise ValueError('download byte length differs: ' + destination.name)


def acquire_runtime(lock_path, output):
    output = hosted_root(output)
    output.mkdir(parents=True)
    lock = json.loads(Path(lock_path).read_text())
    save(output / 'lock.json', lock)
    downloads = output / 'downloads'
    downloads.mkdir()
    download(lock['python'], downloads / lock['python']['filename'])
    download(lock['upstream'], downloads / 'deeptools-source.tar.gz')
    for item in lock['packages'].values():
        download(item, downloads / item['filename'])
    with tarfile.open(downloads / lock['python']['filename'], 'r:gz') as archive:
        archive.extractall(output, filter='data')
    python = output / 'python/bin/python3'
    if not python.is_file():
        raise ValueError('portable Python executable absent')
    save(output / 'acquisition.json', dict(python=str(python), python_sha256=sha256(python),
        files={p.name: sha256(p) for p in sorted(downloads.iterdir())}))


def requirements(lock, names, output, built=None):
    rows = []
    for name in names:
        item = lock['packages'][name]
        digest = sha256(built) if name == 'deeptoolsintervals' and built else item['sha256']
        rows.append(name + '==' + item['version'] + ' --hash=sha256:' + digest)
    Path(output).write_text('\n'.join(rows) + '\n')


def bootstrap_venv(output):
    output = Path(output)
    venv.EnvBuilder(symlinks=True, with_pip=False).create(output / 'venv')
    python = output / 'venv/bin/python'
    original = (output / 'python/bin/python3').resolve(strict=True)
    resolved = python.resolve(strict=True)
    configuration = output / 'venv/pyvenv.cfg'
    save(output / 'venv-binding.json', dict(executable=str(python), resolved_executable=str(resolved),
        original_executable=str(original), original_executable_sha256=sha256(original),
        executable_sha256=sha256(python), is_symlink=python.is_symlink(),
        configuration=configuration.read_text(), configuration_sha256=sha256(configuration)))
    if resolved != original:
        raise RuntimeError('virtual environment interpreter does not resolve to the original portable executable')
    capture(output / 'ensurepip', [python, '-m', 'ensurepip', '--upgrade', '--default-pip'])
    return python


def install_runtime(lock_path, output):
    output = hosted_root(output)
    lock = json.loads(Path(lock_path).read_text())
    if platform.python_version() != lock['python']['version'] or platform.system() != 'Linux' or platform.machine() != 'x86_64':
        raise RuntimeError('runtime is not the frozen Python3.12.10 Linux x86_64 class')
    save(output / 'interpreter.json', dict(version=sys.version, executable=sys.executable,
        executable_sha256=sha256(sys.executable), prefix=sys.prefix, platform=platform.platform()))
    python = bootstrap_venv(output)
    downloads = output / 'downloads'
    bootstrap = output / 'bootstrap.txt'
    requirements(lock, lock['bootstrap'], bootstrap)
    install = [python, '-m', 'pip', 'install', '--no-index', '--no-deps', '--no-compile',
               '--no-cache-dir', '--require-hashes', '--find-links', downloads]
    capture(output / 'bootstrap-install', [*install, '-r', bootstrap])
    built = output / 'built-wheels'
    built.mkdir()
    source = downloads / lock['packages']['deeptoolsintervals']['filename']
    capture(output / 'annotation-build', [python, '-m', 'pip', 'wheel', '--no-deps', '--no-index',
        '--no-cache-dir', '--no-build-isolation', '--wheel-dir', built, source])
    wheels = list(built.glob('*.whl'))
    if len(wheels) != 1:
        raise RuntimeError('annotation build did not produce exactly one wheel')
    save(output / 'built-wheels.json', {wheels[0].name: sha256(wheels[0])})
    runtime = output / 'requirements.txt'
    requirements(lock, sorted(lock['packages']), runtime, wheels[0])
    capture(output / 'runtime-install', [*install, '--find-links', built,
        '--report', output / 'install-report.json', '-r', runtime])
    capture(output / 'pip-check', [python, '-m', 'pip', 'check'])
    capture(output / 'pip-freeze', [python, '-m', 'pip', 'freeze', '--all'])
    capture(output / 'pip-inspect', [python, '-m', 'pip', 'inspect', '--local'])
    capture(output / 'compiler', ['cc', '--version'])
    capture(output / 'linker', ['ldd', '--version'])


def bam_observation(path):
    import pysam
    with pysam.AlignmentFile(str(path), 'rb') as bam:
        reads = []
        for read in bam.fetch(until_eof=True):
            reads.append(dict(sam=read.to_string(), name=read.query_name, flag=read.flag,
                reference_id=read.reference_id, reference_start=read.reference_start, reference_end=read.reference_end,
                cigar=read.cigartuples, cigar_text=read.cigarstring, mapq=read.mapping_quality,
                mate_reference_id=read.next_reference_id, mate_start=read.next_reference_start,
                tlen=read.template_length, sequence=read.query_sequence, qualities=read.qual,
                tags=read.get_tags(with_value_type=True)))
        ordered = [(read['reference_id'], read['reference_start']) for read in reads if not read['flag'] & 4]
        return dict(header=bam.header.to_dict(), header_text=bam.text, reads=reads,
                    coordinate_ordered=ordered == sorted(ordered), has_index=bam.has_index())


def bw_observation(path):
    import pyBigWig
    with pyBigWig.open(str(path)) as bw:
        chromosomes = list(bw.chroms().items())
        data = {}
        for chrom, length in chromosomes:
            regions = [(0, length), (min(17, length - 1), min(1813, length))]
            data[chrom] = dict(intervals=bw.intervals(chrom), values=bw.values(chrom, 0, length),
                statistics=[dict(start=start, end=end, default=bw.stats(chrom, start, end),
                    exact=bw.stats(chrom, start, end, exact=True)) for start, end in regions])
        raw = Path(path).read_bytes()
        endian = '<' if struct.unpack_from('<I', raw)[0] == 0x888FFC26 else '>'
        levels = struct.unpack_from(endian + 'H', raw, 6)[0]
        zoom = [dict(zip(('reduction', 'reserved', 'data_offset', 'index_offset'),
                        struct.unpack_from(endian + 'IIQQ', raw, 64 + 24 * index))) for index in range(levels)]
        return dict(header=bw.header(), chromosomes=chromosomes, raw_zoom_headers=zoom, data=data)


def table_observation(path):
    if Path(path).suffix == '.npz':
        import numpy as np
        with np.load(path, allow_pickle=False) as archive:
            return dict(zip_members=zipfile.ZipFile(path).namelist(),
                arrays={name: dict(shape=archive[name].shape, dtype=str(archive[name].dtype),
                                   values=archive[name]) for name in archive.files})
    if Path(path).suffix == '.gz':
        with gzip.open(path, 'rt') as stream:
            header = json.loads(next(stream).removeprefix('@'))
            rows = [line.rstrip('\n').split('\t') for line in stream]
        return dict(header=header, rows=[dict(identity=row[:6], values=list(map(float, row[6:]))) for row in rows])
    lines = Path(path).read_text().splitlines()
    if Path(path).suffix in ('.bedgraph', '.bg'):
        intervals = [dict(chrom=row[0], start=int(row[1]), end=int(row[2]), value=float(row[3]))
                     for row in (line.split('\t') for line in lines)]
        return dict(lines=lines, intervals=intervals, nonfinite_classes=sorted({encode(row['value'])['nonfinite']
                    for row in intervals if not math.isfinite(row['value'])}))
    return dict(lines=lines)


def source_defaults(module, args):
    if hasattr(module, 'process_args'):
        values = vars(module.process_args(args))
    else:
        parser = module.parseArguments() if hasattr(module, 'parseArguments') else module.parse_arguments()
        values = vars(parser.parse_args(args))
    resolved = encode(values)
    for value in values.values():
        if isinstance(value, io.IOBase):
            value.close()
    return resolved


def runtime_observation(request):
    import numpy as np
    import scipy
    import pysam
    import pyBigWig
    import py2bit
    import deeptools
    import deeptoolsintervals
    lock = json.loads(Path(request['lock']).read_text())
    versions = {name: importlib.metadata.version(name) for name in lock['packages']}
    if any(versions[name] != item['version'] for name, item in lock['packages'].items()):
        raise RuntimeError('installed package version differs from lock')
    archive_path = Path(request['runtime']) / 'downloads/deeptools-source.tar.gz'
    if sha256(archive_path) != SOURCE_SHA:
        raise ValueError('original deepTools source archive differs')
    source = Path(inspect.getfile(deeptools)).parent
    bindings = {}
    with tarfile.open(archive_path) as archive:
        for member in archive.getmembers():
            if member.isfile() and member.name.startswith('deeptools-3.5.6/deeptools/') and member.name.endswith('.py'):
                relative = member.name.removeprefix('deeptools-3.5.6/deeptools/')
                original = archive.extractfile(member).read()
                installed = source / relative
                if installed.read_bytes() != original:
                    raise ValueError('installed deepTools source differs: ' + relative)
                bindings[relative] = sha256(installed)
    modules = {name: dict(path=inspect.getfile(module), sha256=sha256(inspect.getfile(module)), version=versions[distribution]) for name, module, distribution in
        [('numpy', np, 'numpy'), ('scipy', scipy, 'scipy'), ('pysam', pysam, 'pysam'),
         ('pyBigWig', pyBigWig, 'pybigwig'), ('py2bit', py2bit, 'py2bit'),
         ('deeptools', deeptools, 'deeptools'), ('deeptoolsintervals', deeptoolsintervals, 'deeptoolsintervals')]}
    if any(not Path(row['path']).resolve().is_relative_to(Path(sys.prefix).resolve()) for row in modules.values()):
        raise RuntimeError('scientific import escaped the qualified virtual environment')
    np.show_config()
    scipy.show_config()
    return dict(python=sys.version, executable=sys.executable, versions=versions, modules=modules,
        pysam_native={key: value for key, value in vars(importlib.import_module('pysam.version')).items()
                      if 'version' in key.lower() and isinstance(value, str)},
        deepTools_source_bindings=bindings, thread_environment={name: os.environ.get(name) for name in THREADS})


def science_action(request):
    kind = request['kind']
    if kind == 'runtime':
        return runtime_observation(request)
    if kind == 'invoke':
        module = importlib.import_module('deeptools.' + ENTRYPOINTS[request['entrypoint']])
        args = request['args']
        sys.argv = [request['entrypoint'], *args]
        defaults = source_defaults(module, args)
        save(request['defaults'], dict(entrypoint=request['entrypoint'], module=inspect.getfile(module),
                                     module_sha256=sha256(inspect.getfile(module)), resolved=defaults))
        return module.main(args)
    if kind == 'build-bam':
        import pysam
        pysam.view('-b', '-o', request['unsorted'], request['sam'], catch_stdout=False)
        before = bam_observation(request['unsorted'])
        pysam.sort('-o', request['bam'], request['unsorted'])
        pysam.index(request['bam'])
        return dict(unsorted=before, sorted=bam_observation(request['bam']))
    if kind == 'sort-index':
        import pysam
        before = bam_observation(request['input'])
        pysam.sort('-o', request['bam'], request['input'])
        pysam.index(request['bam'])
        return dict(before=before, after=bam_observation(request['bam']))
    if kind == 'build-bw':
        import pyBigWig
        spec = request['definition']
        with pyBigWig.open(request['output'], 'w') as bw:
            bw.addHeader([tuple(row) for row in spec['chroms']], maxZooms=10)
            for chrom, start, end, value in spec['intervals']:
                bw.addEntries([chrom], [start], ends=[end], values=[value])
        return bw_observation(request['output'])
    if kind == 'bam':
        return bam_observation(request['input'])
    if kind == 'bw':
        return bw_observation(request['input'])
    if kind == 'table':
        return table_observation(request['input'])
    if kind == 'normalization':
        from deeptools.bamHandler import openBam
        from deeptools.getScaleFactor import get_num_kept_reads, get_scale_factor
        entrypoint = request.get('entrypoint', 'bamCoverage')
        module = importlib.import_module('deeptools.' + ENTRYPOINTS[entrypoint])
        args = module.process_args(request['args'])
        arguments = encode(vars(args))
        bams = [args.bam] if entrypoint == 'bamCoverage' else [args.bamfile1, args.bamfile2]
        counters = []
        functions = ['deeptools.bamHandler.openBam', 'deeptools.getScaleFactor.get_num_kept_reads']
        for bam in bams:
            args.bam = bam
            handle, mapped, unmapped, stats = openBam(bam, returnStats=True, nThreads=args.numberOfProcessors)
            handle.close()
            kept, total = get_num_kept_reads(args, stats)
            counters.append(dict(bam=bam, bam_sha256=sha256(bam), mapped=mapped, unmapped=unmapped,
                                 stats=stats, kept_reads=kept, total_reads=total))
        if args.normalizeUsing == 'None':
            args.normalizeUsing = None
        if entrypoint == 'bamCoverage':
            factors = [get_scale_factor(args, counters[0]['stats'])]
            functions.append('deeptools.getScaleFactor.get_scale_factor')
        else:
            factors = module.get_scale_factors(args, [row['stats'] for row in counters],
                                               [row['mapped'] for row in counters])
            functions.append('deeptools.bamCompare.get_scale_factors')
            if factors is None:
                if args.normalizeUsing is None:
                    factors = [1, 1]
                else:
                    args.scaleFactor = 1.0
                    factors = []
                    for row in counters:
                        args.bam = row['bam']
                        factors.append(get_scale_factor(args, row['stats']))
                    functions.append('deeptools.getScaleFactor.get_scale_factor')
        return dict(entrypoint=entrypoint, producer=request.get('producer'), arguments=arguments,
                    inputs=counters, scale_factors=factors, functions=functions)
    raise ValueError('unknown scientific action: ' + kind)


def observe_request(request_path, result_path):
    hosted_root(result_path)
    if platform.python_version() != '3.12.10' or platform.system() != 'Linux' or platform.machine() != 'x86_64':
        raise RuntimeError('scientific capture requires the frozen hosted runtime')
    request = json.loads(Path(request_path).read_text())
    try:
        result = science_action(request)
        record = dict(status='returned', result=encode(result))
    except SystemExit as error:
        if error.code in (None, 0):
            record = dict(status='returned', result=None, system_exit=error.code)
        else:
            record = dict(status='raised', exception=dict(type='SystemExit', message=str(error), traceback=traceback.format_exc()))
    except Exception as error:
        record = dict(status='raised', exception=dict(type=type(error).__name__, message=str(error), traceback=traceback.format_exc()))
    save(result_path, record)
    if record['status'] == 'raised':
        raise RuntimeError('scientific source call failed: ' + str(result_path))


def workflow_requests(root, inputs, runtime, lock):
    generated = root / 'generated-inputs'
    results = root / 'results'
    planned = []

    def add(name, kind, phase='main', **request):
        planned.append(dict(name=name, phase=phase, request=dict(kind=kind, **request)))

    def call(name, entrypoint, args, phase='main', expected=None):
        args = list(map(str, args))
        add(name, 'invoke', phase, entrypoint=entrypoint, args=args,
            defaults=str(root / 'calls' / name / 'defaults.json'), expected=expected)
        return args

    def normalization(name, producer, entrypoint, args):
        args = list(args)
        args[args.index('-o') + 1] = str(results / (name + '.parser-only'))
        add(name, 'normalization', entrypoint=entrypoint, producer=producer, args=args)

    def inspect_output(name, path, kind, phase='main'):
        add(name, kind, phase, input=str(path))

    add('runtime', 'runtime', 'runtime', runtime=str(runtime), lock=str(lock))
    bams = {}
    tracks = {}
    for sample in ('a', 'b'):
        raw = generated / (sample + '.raw.bam')
        add('build-' + sample, 'build-bam', sam=str(inputs / 'authored' / (sample + '.sam')),
            unsorted=str(generated / (sample + '.unsorted.bam')), bam=str(raw))
        filtered = results / (sample + '.filtered.bam')
        failed = results / (sample + '.excluded.bam')
        metrics = results / (sample + '.metrics.tsv')
        call('filter-' + sample, 'alignmentSieve', ['-b', raw, '-o', filtered, '--minMappingQuality', '20',
            '--samFlagExclude', '2816', '--genomeChunkLength', '256', '--filteredOutReads', failed, '--filterMetrics', metrics])
        inspect_output('filter-pass-' + sample, filtered, 'bam')
        inspect_output('filter-excluded-' + sample, failed, 'bam')
        inspect_output('filter-metrics-' + sample, metrics, 'table')
        shifted = results / (sample + '.shifted.bam')
        add('index-filtered-' + sample, 'sort-index', input=str(filtered), bam=str(generated / (sample + '.filtered-indexed.bam')))
        call('atac-' + sample, 'alignmentSieve', ['-b', generated / (sample + '.filtered-indexed.bam'), '-o', shifted, '--ATACshift', '--genomeChunkLength', '256'])
        bams[sample] = generated / (sample + '.source.bam')
        add('sort-shifted-' + sample, 'sort-index', input=str(shifted), bam=str(bams[sample]))
        fragments = results / (sample + '.fragments.bed')
        call('fragments-' + sample, 'alignmentSieve', ['-b', bams[sample], '-o', fragments, '--BED'])
        inspect_output('fragments-table-' + sample, fragments, 'table')
        for form in ('bedgraph', 'bigwig'):
            suffix = '.bw' if form == 'bigwig' else '.bedgraph'
            path = results / (sample + '.cpm' + suffix)
            name = 'track-' + sample + '-' + form
            args = call(name, 'bamCoverage', ['-b', bams[sample], '-o', path, '--verbose',
                 '--outFileFormat', form, '--binSize', '20', '--normalizeUsing', 'CPM', '--exactScaling'])
            inspect_output('track-values-' + sample + '-' + form, path, 'bw' if form == 'bigwig' else 'table')
            if form == 'bigwig':
                tracks[sample] = path
                normalization('normalization-main-track-' + sample, name, 'bamCoverage', args)
    for name, flags in (
            ('readCount', []), ('SES', ['--scaleFactorsMethod', 'SES', '--sampleLength', '50', '--numberOfSamples', '24']),
            ('manual', ['--scaleFactors', '2:0.5', '--operation', 'ratio', '--pseudocount', '2', '3']),
            ('normalized', ['--scaleFactorsMethod', 'None', '--normalizeUsing', 'CPM', '--exactScaling'])):
        path = results / ('compare-bam-' + name + '.bedgraph')
        args = call('compare-bam-' + name, 'bamCompare', ['-b1', bams['a'], '-b2', bams['b'], '-o', path,
             '--outFileFormat', 'bedgraph', '--binSize', '20', '--verbose', *flags])
        inspect_output('compare-bam-values-' + name, path, 'table')
        normalization('normalization-main-compare-' + name, 'compare-bam-' + name, 'bamCompare', args)
    derived = {}
    for name, entry, args in (
            ('compare-bw', 'bigwigCompare', ['-b1', tracks['a'], '-b2', tracks['b']]),
            ('average', 'bigwigAverage', ['-b', tracks['a'], tracks['b'], tracks['a'], '--scaleFactors', '1:0.5:2'])):
        path = results / (name + '.bw')
        call(name, entry, [*args, '-o', path, '--binSize', '20'])
        inspect_output(name + '-values', path, 'bw')
        derived[name] = path
    derived_files = [derived['compare-bw'], derived['average']]
    derived_labels = ['CPM A/B log2 ratio', 'CPM A/B/A scaled average']
    for source, entry, files, labels in (
            ('bam', 'multiBamSummary', [bams['a'], bams['b']], ['A', 'B']),
            ('bw', 'multiBigwigSummary', [tracks['a'], tracks['b']], ['A', 'B']),
            ('bw-derived', 'multiBigwigSummary', derived_files, derived_labels)):
        for mode in ('bins', 'BED-file'):
            name = 'summary-' + source + '-' + mode
            npz = results / (name + '.npz')
            raw = results / (name + '.tsv')
            flags = ['--binSize', '100'] if mode == 'bins' else ['--BED', inputs / 'authored/regions.bed']
            if source == 'bam':
                flags.extend(['--genomeChunkSize', '333'])
            call(name, entry, [mode, '-b', *files, '-o', npz, '--labels', *labels, '--outRawCounts', raw, *flags])
            inspect_output(name + '-arrays', npz, 'table')
            inspect_output(name + '-rows', raw, 'table')
    matrix_cases = (
        ('reference', 'reference-point', 'groups.bed', ['--referencePoint', 'TSS', '-b', '50', '-a', '100']),
        ('scale', 'scale-regions', 'groups.bed', ['-m', '200', '-b', '50', '-a', '50', '--unscaled5prime', '20', '--unscaled3prime', '20']),
        ('bed12', 'reference-point', 'exons.bed', ['--referencePoint', 'center', '-b', '50', '-a', '50', '--metagene']),
        ('gtf', 'scale-regions', 'regions.gtf', ['-m', '200', '-b', '20', '-a', '20', '--metagene']))
    for name, mode, regions, flags in matrix_cases:
        path = results / ('matrix-' + name + '.gz')
        files, labels = (derived_files, derived_labels) if name in ('reference', 'scale') else ([tracks['a'], tracks['b']], ['A', 'B'])
        call('matrix-' + name, 'computeMatrix', [mode, '-S', *files, '-R', inputs / 'authored' / regions,
            '-o', path, '--binSize', '10', '--samplesLabel', *labels, '--outFileSortedRegions', results / ('matrix-' + name + '.bed'), *flags])
        inspect_output('matrix-' + name + '-arrays', path, 'table')
    for asset, name in (('rsomics-atac-shift', 'atac_small.bam'), ('rsomics-alignment-sieve', 'reads.bam'),
                       ('rsomics-bam-compare', 'treat.bam'), ('rsomics-bam-compare', 'ctrl.bam')):
        inspect_output('original-' + asset + '-' + name, inputs / 'historical' / asset / 'tests/golden' / name, 'bam', 'diagnostic')
    original_a = inputs / 'historical/rsomics-bigwig-compare/tests/golden/a.bw'
    original_b = inputs / 'historical/rsomics-bigwig-compare/tests/golden/b.bw'
    for name, path in (('a', original_a), ('b', original_b)):
        inspect_output('original-bw-' + name, path, 'bw', 'diagnostic')
    path = results / 'original-bw-compare.bedgraph'
    call('original-bw-compare', 'bigwigCompare', ['-b1', original_a, '-b2', original_b, '-o', path,
        '--binSize', '25', '--outFileFormat', 'bedgraph'], 'diagnostic')
    inspect_output('original-bw-compare-values', path, 'table', 'diagnostic')
    definitions = json.loads((inputs / 'authored/signals.json').read_text())
    signals = {}
    for name, definition in definitions.items():
        signals[name] = generated / (name + '.bw')
        add('build-signal-' + name, 'build-bw', 'diagnostic', output=str(signals[name]), definition=definition)
    for name, entry, args in (
            ('signed-cancellation', 'bigwigCompare', ['--operation', 'subtract', '--skipZeroOverZero']),
            ('asymmetric', 'bigwigCompare', ['--operation', 'ratio', '--pseudocount', '2', '3']),
            ('nonfinite', 'bigwigCompare', ['--operation', 'ratio', '--pseudocount', '0', '--fixedStep']),
            ('nan-log', 'bigwigCompare', ['--operation', 'log2', '--pseudocount', '0']),
            ('nonfinite-log-asymmetric', 'bigwigCompare', ['--operation', 'log2', '--pseudocount', '0', '2', '--fixedStep']),
            ('missing-zero', 'bigwigCompare', ['--operation', 'subtract']),
            ('missing-skip', 'bigwigCompare', ['--operation', 'subtract', '--skipNAs']),
            ('average-missing', 'bigwigAverage', ['--skipNAs', '--scaleFactors', '1:0.5:2'])):
        path = results / (name + '.bedgraph')
        files = ['-b1', signals['signed-a'], '-b2', signals['signed-b']] if entry == 'bigwigCompare' else ['-b', signals['signed-a'], signals['signed-b'], signals['signed-a']]
        call(name, entry, [*files, '-o', path, '--outFileFormat', 'bedgraph', '--binSize', '20', *args], 'diagnostic')
        inspect_output(name + '-values', path, 'table', 'diagnostic')
    path = results / 'unequal-dictionary.bedgraph'
    call('unequal-dictionary', 'bigwigCompare', ['-b1', signals['signed-a'], '-b2', signals['short-dictionary'],
        '-o', path, '--outFileFormat', 'bedgraph', '--binSize', '20'], 'diagnostic')
    inspect_output('unequal-dictionary-values', path, 'table', 'diagnostic')
    raw_bam = generated / 'a.raw.bam'
    for name, flags in (
            ('blocks', []), ('extended-centered', ['--extendReads', '160', '--centerReads', '--smoothLength', '60']),
            ('offset-strand', ['--extendReads', '160', '--Offset', '5', '-1', '--filterRNAstrand', 'forward']),
            ('blacklist-grid', ['--blackListFileName', inputs / 'authored/blacklist.bed']),
            ('mnase', ['--MNase', '--minFragmentLength', '130', '--maxFragmentLength', '200'])):
        path = results / ('profile-' + name + '.bedgraph')
        call('profile-' + name, 'bamCoverage', ['-b', raw_bam, '-o', path, '--outFileFormat', 'bedgraph',
            '--binSize', '20', '--region', 'chr1:180:720', *flags], 'diagnostic')
        inspect_output('profile-' + name + '-values', path, 'table', 'diagnostic')
    for name, flags in (
            ('cpm', ['--normalizeUsing', 'CPM']), ('bpm', ['--normalizeUsing', 'BPM']),
            ('rpkm', ['--normalizeUsing', 'RPKM']), ('rpgc', ['--normalizeUsing', 'RPGC', '--effectiveGenomeSize', '2000']),
            ('dedup-only', ['--normalizeUsing', 'CPM', '--ignoreDuplicates']),
            ('filtered-exact', ['--normalizeUsing', 'CPM', '--exactScaling', '--minMappingQuality', '20', '--samFlagExclude', '2816']),
            ('blacklist', ['--normalizeUsing', 'CPM', '--blackListFileName', inputs / 'authored/blacklist.bed'])):
        add('normalization-' + name, 'normalization', 'diagnostic', args=list(map(str,
            ['-b', raw_bam, '-o', results / ('normalization-' + name + '.unused'), '--binSize', '20', '--verbose', *flags])))
    for source, entry, files in (('bam', 'multiBamSummary', bams), ('bw', 'multiBigwigSummary', tracks)):
        path = results / ('workers4-' + source + '.npz')
        raw = results / ('workers4-' + source + '.tsv')
        flags = ['--genomeChunkSize', '333'] if source == 'bam' else []
        call('workers4-' + source, entry, ['bins', '-b', files['a'], files['b'], '-o', path,
            '--binSize', '100', '--numberOfProcessors', '4', '--outRawCounts', raw, *flags], 'diagnostic')
        inspect_output('workers4-' + source + '-arrays', path, 'table', 'diagnostic')
        inspect_output('workers4-' + source + '-rows', raw, 'table', 'diagnostic')
    path = results / 'alias-summary.npz'
    call('alias-summary', 'multiBigwigSummary', ['bins', '-b', signals['signed-a'], signals['aliased-dictionary'],
        '-o', path, '--binSize', '20', '--outRawCounts', results / 'alias-summary.tsv'], 'diagnostic')
    inspect_output('alias-summary-arrays', path, 'table', 'diagnostic')
    call('bed-filtered-out-boundary', 'alignmentSieve', ['-b', raw_bam, '-o', results / 'boundary.bed', '--BED',
        '--minMappingQuality', '20', '--filteredOutReads', results / 'boundary-excluded.bam'], 'boundary',
        expected=dict(type='TypeError', message_contains='convertBED'))
    return planned


def bind_evidence(root):
    root = Path(root)
    root.mkdir(parents=True, exist_ok=True)
    record = dict(schema='rsomics.signal.source-artifact.v1',
        files={path.relative_to(root).as_posix(): dict(sha256=sha256(path), bytes=path.stat().st_size)
               for path in sorted(root.rglob('*')) if path.is_file() and path.name != 'artifact-manifest.json'},
        interpretation='read witness/qualification.json and original step statuses; retention alone is not success')
    save(root / 'artifact-manifest.json', record)


def run_workflow(inputs_archive, lock_path, runtime, output):
    root = hosted_root(output)
    root.mkdir(parents=True)
    lock = json.loads(Path(lock_path).read_text())
    if platform.python_version() != lock['python']['version'] or any(os.environ.get(name) != '1' for name in THREADS):
        raise RuntimeError('qualified Python/thread profile differs')
    inputs = root / 'inputs'
    provenance = prepare_inputs(inputs_archive, lock_path, inputs)
    for directory in ('generated-inputs', 'results', 'calls'):
        (root / directory).mkdir()
    plan = workflow_requests(root, inputs, Path(runtime), Path(lock_path))
    save(root / 'execution-plan.json', plan)
    save(root / 'pre-execution.json', dict(controller_sha256=sha256(__file__), runtime_lock_sha256=sha256(lock_path),
        input_archive_sha256=sha256(inputs_archive), upstream=provenance['upstream'], operations=OPERATIONS,
        execution_plan_sha256=sha256(root / 'execution-plan.json'), input_files=provenance['files']))
    completed = []
    state = dict(status='running', main_lifecycle_complete=False, completed=completed, failure=None,
                 native_policy_selected=False, performance_claim=False)
    main_names = {item['name'] for item in plan if item['phase'] == 'main'}
    try:
        for item in plan:
            name, request = item['name'], item['request']
            target = root / 'calls' / name
            request_file = root / 'calls' / (name + '-request.json')
            save(request_file, request)
            expected = request.get('expected')
            result = capture(target, [sys.executable, '-B', str(Path(__file__).resolve()), '--observe',
                request_file, '--observation', target / 'observation.json'], required=expected is None)
            observation = json.loads((target / 'observation.json').read_text())
            if result['status'] != 0:
                failure = observation.get('exception', {})
                if not expected or failure.get('type') != expected['type'] or expected['message_contains'] not in failure.get('message', ''):
                    raise RuntimeError('unexpected source exception: ' + name)
            completed.append(dict(name=name, phase=item['phase'], status=observation['status'],
                                  expected_exception=result['status'] != 0))
            returned = {item['name'] for item in completed if item['status'] == 'returned'}
            state['main_lifecycle_complete'] = main_names.issubset(returned)
        state['status'] = 'complete'
        state['operation_coverage'] = OPERATIONS
        state['supported_exception_probes'] = [item for item in completed if item['expected_exception']]
    except Exception as error:
        state['status'] = 'failed'
        state['failure'] = dict(type=type(error).__name__, message=str(error), traceback=traceback.format_exc())
        raise
    finally:
        changed_inputs = [name for name, row in provenance['files'].items() if sha256(inputs / name) != row['sha256']]
        changed_source = []
        runtime_record = root / 'calls/runtime/observation.json'
        if runtime_record.exists():
            runtime_state = json.loads(runtime_record.read_text())
            if runtime_state['status'] == 'returned':
                result = runtime_state['result']
                source = Path(result['modules']['deeptools']['path']).parent
                changed_source = [name for name, digest in result['deepTools_source_bindings'].items()
                                  if sha256(source / name) != digest]
        save(root / 'integrity-after.json', dict(changed_inputs=changed_inputs, changed_source=changed_source))
        if changed_inputs or changed_source:
            state['status'] = 'failed'
            state['failure'] = dict(type='IntegrityError', changed_inputs=changed_inputs, changed_source=changed_source)
        save(root / 'qualification.json', state)
        if changed_inputs or changed_source:
            raise RuntimeError('source/input bytes changed during capture')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument('--acquire', action='store_true')
    action.add_argument('--install', action='store_true')
    action.add_argument('--run', action='store_true')
    action.add_argument('--manifest', action='store_true')
    action.add_argument('--observe', type=Path)
    parser.add_argument('--runtime-lock', type=Path)
    parser.add_argument('--runtime', type=Path)
    parser.add_argument('--inputs', type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--observation', type=Path)
    args = parser.parse_args()
    if args.observe:
        observe_request(args.observe, args.observation)
    elif args.acquire:
        acquire_runtime(args.runtime_lock, args.output)
    elif args.install:
        install_runtime(args.runtime_lock, args.output)
    elif args.run:
        run_workflow(args.inputs, args.runtime_lock, args.runtime, args.output)
    elif args.manifest:
        bind_evidence(args.output)


if __name__ == '__main__':
    main()
