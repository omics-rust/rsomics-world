"""Hosted Linux-only pinned upstream consumers of the real native graph."""

import argparse
import json
import math
import os
from pathlib import Path
import platform
import struct
import sys

import signal_first_workflow_oracle as oracle
from infercnv_source_snapshot import _hash_file
from signal_candidate import (capture, compare_model, matrix_model, npz_model,
                              numeric_equal, save, storage, tree_manifest)


LOCK_SHA = '763a0eae15f15315f001ffb56dd261710f862362d5a3da6d94b107f13c022bc9'
PRODUCER_SHA = '39668a86b2fa084d8520e7f63f85bb837a2857c527de56917f9baef4ca107b57'
MATRIX_KEYS = ('sample_boundaries', 'group_boundaries', 'sample_labels', 'group_labels')


def assert_frozen_bands():
    if not numeric_equal(1.0 + 1e-12, 1.0) or numeric_equal(1.0 + 1e-7, 1.0):
        raise ValueError('frozen f64 abs1e-12+rel1e-12 policy changed')
    if not numeric_equal(1.0 + 0.4e-6, 1.0, absolute=0.5e-6) or \
            numeric_equal(1.0 + 0.6e-6, 1.0, absolute=0.5e-6):
        raise ValueError('frozen six-decimal matrix policy changed')


def hosted_runtime(runtime, lock):
    storage()
    if platform.system() != 'Linux' or platform.machine() != 'x86_64' or platform.python_version() != '3.12.10':
        raise ValueError('source imports require Linux x86_64 pinned Python3.12.10')
    if Path(sys.executable).resolve() != (runtime / 'python/bin/python3').resolve(strict=True):
        raise ValueError('interpreter differs from portable Python')
    if Path(sys.prefix).resolve() != (runtime / 'venv').resolve(strict=True):
        raise ValueError('not the exact isolated source virtual environment')
    if _hash_file(lock) != LOCK_SHA or _hash_file(oracle.__file__) != PRODUCER_SHA:
        raise ValueError('accepted producer/runtime lock changed')
    for name in oracle.THREADS:
        if os.environ.get(name) != '1':
            raise ValueError('source numeric thread control differs: ' + name)
    result = oracle.runtime_observation(dict(runtime=str(runtime), lock=str(lock)))
    result['installed_extension_bodies'] = {
        path.relative_to(sys.prefix).as_posix(): _hash_file(path)
        for path in sorted(Path(sys.prefix).rglob('*.so')) if path.is_file()}
    if not result['installed_extension_bodies']:
        raise ValueError('installed native scientific library body bindings absent')
    return result


def source_call(output, name, entrypoint, args):
    """Run unmodified accepted observation producer in a fresh pinned process."""
    call = output / 'source-calls' / name
    call.mkdir(parents=True)
    request = dict(kind='invoke', entrypoint=entrypoint, args=list(map(str, args)),
                   defaults=str(call / 'defaults.json'))
    save(call / 'request.json', request)
    capture(call, 'process', [sys.executable, '-B', oracle.__file__, '--observe', call / 'request.json',
                            '--observation', call / 'observation.json'])
    state = json.loads((call / 'observation.json').read_text())
    if state['status'] != 'returned':
        raise ValueError('required upstream reader failed: ' + name)


def floats_equal(a, b):
    if len(a) != len(b) or not all(numeric_equal(x, y) for x, y in zip(a, b)):
        raise ValueError('exact values, nonfinite classes or frozen reduction bound differ')


def compare_npz(native, source):
    native, source = npz_model(native), npz_model(source)
    if native['labels'] != source['labels']:
        raise ValueError('NPZ labels/dtype/shape differ')
    compare_model(native['matrix'], source['matrix'])


def compare_matrix(native, source):
    native, source = matrix_model(native), matrix_model(source)
    native['header'] = {key: native['header'][key] for key in MATRIX_KEYS}
    source['header'] = {key: source['header'][key] for key in MATRIX_KEYS}
    compare_model(native, source, matrix=True)


def from_bedgraph(path, observation):
    """Read the native six-significant-digit run stream; no copied source arithmetic."""
    import pyBigWig
    expected = []
    for line in path.read_text().splitlines():
        chrom, start, end, value = line.split('\t')
        quantized = struct.unpack('<f', struct.pack('<f', float(value)))[0]
        expected.append((chrom, int(start), int(end), quantized))
    actual = [(chrom, *row) for chrom, record in observation['data'].items() for row in record['intervals'] or []]
    if len(expected) != len(actual):
        raise ValueError('native BW differs from native quantized bedGraph run population')
    for e, a in zip(expected, actual):
        if e[:3] != a[:3] or not numeric_equal(e[3], a[3], absolute=0, relative=0):
            raise ValueError('native BW differs from exact float32 native run stream')
    comparison = path.with_suffix('.pybigwig.bw')
    with pyBigWig.open(str(comparison), 'w') as bw:
        bw.addHeader([tuple(row) for row in observation['chromosomes']], maxZooms=10)
        for chrom, start, end, value in expected:
            bw.addEntries([chrom], [start], ends=[end], values=[value])
    return comparison


def compare_bw(native, source):
    if native['chromosomes'] != source['chromosomes']:
        raise ValueError('BW chromosome dictionary/order differs')
    levels = lambda row: [z['reduction'] for z in row['raw_zoom_headers']]
    if levels(native) != levels(source):
        raise ValueError('native and pinned writer zoom levels differ')
    if native['header'].keys() != source['header'].keys():
        raise ValueError('BW summary schema differs')
    for key, value in native['header'].items():
        other = source['header'][key]
        if isinstance(value, (int, float)):
            if not numeric_equal(value, other):
                raise ValueError('BW header summary differs: ' + key)
        elif value != other:
            raise ValueError('BW header differs: ' + key)
    for chrom in native['data']:
        a, b = native['data'][chrom], source['data'][chrom]
        if a['intervals'] != b['intervals']:
            raise ValueError('BW intervals differ: ' + chrom)
        floats_equal(a['values'], b['values'])
        for first, second in zip(a['statistics'], b['statistics']):
            if (first['start'], first['end']) != (second['start'], second['end']):
                raise ValueError('BW query identity differs')
            for field in ('default', 'exact'):
                if len(first[field]) != len(second[field]):
                    raise ValueError('BW stats cardinality differs')
                for x, y in zip(first[field], second[field]):
                    if (x is None) != (y is None) or (x is not None and not numeric_equal(x, y)):
                        raise ValueError('same-stream BW ' + field + ' stats differ')


def expanded_stats(path, observation):
    import pyBigWig
    rows = []
    with pyBigWig.open(str(path)) as bw:
        for chrom, length in observation['chromosomes']:
            queries = [(0, length), (min(17, length - 1), min(1813, length))]
            if length > 82000:
                queries += [(81700, 81900), (length - 1003, length)]
            for start, end in queries:
                for bins in (1, 7):
                    for kind in ('mean', 'min', 'max', 'std', 'coverage', 'sum'):
                        for exact in (False, True):
                            rows.append(dict(chrom=chrom, start=start, end=end, bins=bins, kind=kind, exact=exact,
                                values=bw.stats(chrom, start, end, nBins=bins, type=kind, exact=exact)))
    return rows


def compare_stats(native, comparator):
    if len(native) != len(comparator):
        raise ValueError('expanded statistics query population differs')
    for a, b in zip(native, comparator):
        a, b = dict(a), dict(b)
        av, bv = a.pop('values'), b.pop('values')
        if a != b or len(av) != len(bv):
            raise ValueError('expanded statistics identities/cardinality differ')
        for x, y in zip(av, bv):
            if (x is None) != (y is None) or (x is not None and not numeric_equal(x, y)):
                raise ValueError('expanded same-stream default/exact statistics differ')


def observe_native_bw(root, output, name, bw, run_args):
    binary = root / 'binary-release'
    stem = output / name
    graph = stem.with_suffix('.bedgraph')
    capture(output / 'native-calls', name + '-bedgraph', [binary, *run_args, '--out', graph, '--format', 'bedgraph'])
    observation = oracle.bw_observation(bw)
    save(output / (name + '-pybigwig-native.json'), oracle.encode(observation))
    separately_written = from_bedgraph(graph, observation)
    comparator = oracle.bw_observation(separately_written)
    save(output / (name + '-pybigwig-comparator.json'), oracle.encode(comparator))
    compare_bw(observation, comparator)
    native_stats, comparator_stats = expanded_stats(bw, observation), expanded_stats(separately_written, comparator)
    save(output / (name + '-expanded-statistics.json'), oracle.encode(dict(native=native_stats, comparator=comparator_stats)))
    compare_stats(native_stats, comparator_stats)
    if name == 'rollover':
        if sum(len(r['intervals'] or []) for r in observation['data'].values()) != 12000:
            raise ValueError('meaningful 12000-interval rollover population missing')
        if [z['reduction'] for z in observation['raw_zoom_headers']] != [80, 320]:
            raise ValueError('rollover zoom level population differs')
    if name == 'empty':
        if bw.stat().st_size != 217 or observation['header']['nBasesCovered'] != 0 or \
                any(r['intervals'] for r in observation['data'].values()):
            raise ValueError('legitimate empty dictionary-only BW contract differs')
    full = stem.with_suffix('.full.npz')
    capture(output / 'native-calls', name + '-full-values', [binary, 'summarize', 'bins', '--input', 'bigwig',
            '--files', bw, '--out', full, '--bin-size', '1', '--labels', name])
    expected = []
    for chrom, length in sorted(observation['chromosomes']):
        values = observation['data'][chrom]['values']
        if not values and observation['header']['nBasesCovered'] == 0:
            values = [float('nan')] * length
        if len(values) != length:
            raise ValueError('pinned full-resolution values length differs')
        expected.extend(values)
    full_count = len(expected)
    decoded = npz_model(full)
    if decoded['matrix']['header']['shape'] != (len(expected), 1) or decoded['labels']['values'] != [name]:
        raise ValueError('native full-resolution summary schema differs')
    floats_equal(decoded['matrix']['values'], expected)
    bed = stem.with_suffix('.queries.bed')
    queries = []
    for chrom, length in observation['chromosomes']:
        queries += [(chrom, 0, length), (chrom, min(17, length - 1), min(1813, length))]
    bed.write_text(''.join(f'{c}\t{s}\t{e}\n' for c, s, e in queries))
    summary = stem.with_suffix('.default.npz')
    capture(output / 'native-calls', name + '-default-regions', [binary, 'summarize', 'regions', '--input', 'bigwig',
        '--files', bw, '--regions', bed, '--out', summary, '--labels', name])
    expected = [stat['default'][0] for chrom, _ in observation['chromosomes']
                for stat in observation['data'][chrom]['statistics']]
    # The Python BW API encodes empty means as None; NPZ represents this same missing class as NaN.
    floats_equal(npz_model(summary)['matrix']['values'], [float('nan') if v is None else v for v in expected])
    return dict(native_sha256=_hash_file(bw), native_bedgraph_sha256=_hash_file(graph),
                comparator_sha256=_hash_file(separately_written), intervals=sum(len(r['intervals'] or []) for r in observation['data'].values()),
                zooms=[z['reduction'] for z in observation['raw_zoom_headers']],
                source_same_file=True, full_values=full_count, statistics_queries=len(native_stats),
                comparator_replaces_native_input=False)


def run(root, runtime, lock, output):
    assert_frozen_bands()
    output.mkdir()
    before = hosted_runtime(runtime, lock)
    save(output / 'runtime-before.json', oracle.encode(before))
    binaries = dict(native_binary_sha256=_hash_file(root / 'binary-release'),
                    source_selection_sha256=_hash_file(root / 'selection.json'),
                    runtime_lock_sha256=_hash_file(lock), producer_sha256=_hash_file(oracle.__file__))
    save(output / 'bindings-before.json', binaries)
    graph = Path(json.loads((root / 'test-release-graph.json').read_text())['path'])
    fixture = root / 'fixtures'
    native = root / 'workers/1'
    tracks = [native / 'a.bw', native / 'b.bw']
    specs = {
        'a': (tracks[0], ['track', '--bam', graph / 'a.source-native.bam', '--bin-size', '20', '--normalize', 'CPM', '--exact-scaling']),
        'b': (tracks[1], ['track', '--bam', graph / 'b.source-native.bam', '--bin-size', '20', '--normalize', 'CPM', '--exact-scaling']),
        'compare': (native / 'compare.bw', ['compare', '--input', 'bigwig', '--first', tracks[0], '--second', tracks[1], '--bin-size', '20']),
        'average': (native / 'average.bw', ['average', '--bigwig', *tracks, tracks[0], '--scale-factors', '1:0.5:2', '--bin-size', '20']),
        'rollover': (graph / 'rollover-native.bw', ['average', '--bigwig', graph / 'rollover-input.bw', '--bin-size', '5', '--skip-nas']),
        'empty': (graph / 'legitimate-empty.bw', ['compare', '--input', 'bigwig', '--first', fixture / 'generated-inputs/signed-a.bw',
                  '--second', fixture / 'generated-inputs/signed-b.bw', '--bin-size', '20', '--operation', 'log2', '--pseudocount', '0'])}
    observed = {}
    failure = None
    try:
        for name, (bw, args) in specs.items():
            observed[name] = observe_native_bw(root, output, name, bw, args)
        derived = [native / 'compare.bw', native / 'average.bw']
        for mode, source_mode in (('bins', 'bins'), ('regions', 'BED-file')):
            out = output / ('summary-' + mode + '.npz')
            raw = output / ('summary-' + mode + '.tsv')
            args = [source_mode, '-b', *derived, '-o', out, '--labels', 'A', 'B', '--outRawCounts', raw, '-p', '1']
            args += ['--binSize', '100'] if mode == 'bins' else ['--BED', fixture / 'inputs/authored/regions.bed']
            source_call(output, 'summary-' + mode, 'multiBigwigSummary', args)
            compare_npz(native / (mode + '.npz'), out)
            # Raw tables carry coordinate identities absent from NPZ. Only numeric spelling may differ.
            native_lines = (native / (mode + '.tsv')).read_text().splitlines()
            source_lines = raw.read_text().splitlines()
            if len(native_lines) != len(source_lines):
                raise ValueError('summary identity row population differs')
            for a, b in zip(native_lines[1:], source_lines[1:]):
                a, b = a.split('\t'), b.split('\t')
                if a[:3] != b[:3] or len(a) != len(b):
                    raise ValueError('summary row coordinates/order differ')
                floats_equal(list(map(float, a[3:])), list(map(float, b[3:])))
        for mode, source_mode in (('reference', 'reference-point'), ('scale', 'scale-regions')):
            out = output / ('matrix-' + mode + '.gz')
            args = [source_mode, '-S', *derived, '-R', fixture / 'inputs/authored/groups.bed', '-o', out,
                    '--beforeRegionStartLength', '50', '--afterRegionStartLength', '100' if mode == 'reference' else '50',
                    '--samplesLabel', 'A', 'B', '-p', '1']
            if mode == 'scale':
                args += ['--regionBodyLength', '200', '--unscaled5prime', '20', '--unscaled3prime', '20']
            source_call(output, 'matrix-' + mode, 'computeMatrix', args)
            compare_matrix(native / ('matrix-' + mode + '.gz'), out)
        after = hosted_runtime(runtime, lock)
        save(output / 'runtime-after.json', oracle.encode(after))
        if before != after or binaries['native_binary_sha256'] != _hash_file(root / 'binary-release'):
            raise ValueError('source/runtime/native binary changed during interop')
    except Exception as exc:
        failure = dict(type=type(exc).__name__, message=str(exc))
        raise
    finally:
        save(output / 'qualification.json', dict(passed=failure is None, failure=failure, observed=observed,
             source_run=41192955170, repeated_accepted_source_observations=False,
             scope='Linux x86_64 pinned source reads actual connected native BWs; native full/default public summaries, '
                   'separate pinned writer of native quantized run streams, 12000-interval rollover and empty217B, '
                   'multiBigwigSummary bins/BED plus computeMatrix reference/scale. No private native statistics API.',
             approved_deltas=['SIGNAL-356-ARITHMETIC-LOSS', 'SIGNAL-356-ZERO-TAIL', 'SIGNAL-356-FILTER-BED-REJECTED'],
             delta_policy='Source consumers see corrected native input bytes; source producers are not rerun to replace native graph. '
                          'Original accepted source remains immutable. Normal zoom/BPM/aliases/nonfinite/shift metadata unchanged.',
             sorted_bed='observational only; human choice pending; not adopted by this gate',
             performance_claim=False, release=False))
        save(output / 'files.json', tree_manifest(output))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--candidate', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--lock', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    run(args.candidate, args.runtime, args.lock, args.output)


if __name__ == '__main__':
    main()
