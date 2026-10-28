#!/usr/bin/env python3
"""Capture one actual scikit-bio count-table diversity lifecycle on a hosted runner."""

import argparse
import contextlib
import csv
import hashlib
import importlib
import importlib.metadata
import inspect
import io
import json
import math
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import tarfile
import traceback
import urllib.request
import warnings


SOURCES = {
    'scikit-bio': dict(version='0.7.3', commit='a3beaa632077a532330343d5db0d8e30e58b6705'),
    'biom-format': dict(version='2.1.17', commit='2b81a013aae29004d9b82d13847450b27fd6b161'),
}
ALPHA = ('observed_features', 'shannon', 'simpson', 'dominance', 'inv_simpson',
         'simpson_e', 'pielou_e', 'chao1', 'goods_coverage')
BETA = ('braycurtis', 'jaccard', 'euclidean', 'canberra', 'cityblock',
        'unweighted_unifrac', 'weighted_unifrac', 'weighted_normalized_unifrac')
PHYDIV = (('auto', None, False), ('rooted-weighted', True, True),
          ('unrooted-weighted', False, True), ('rooted-quarter', True, .25),
          ('unrooted-half', False, .5))
MAX_EXACT_INTEGER = 2**53 - 1
BOUNDS = dict(alpha=[1e-10, 1e-10], phylogenetic=[1e-10, 1e-10], beta=[1e-10, 1e-10])
PRIMARY_ROWS = ((0, 5, 6, 6, 5, 0), (6, 6, 5, 6, 6, 0), (5, 5, 3, 2, 6, 0),
                (3, 6, 6, 5, 1, 0), (3, 4, 4, 1, 2, 3), (6, 1, 3, 6, 3, 2),
                (0, 6, 3, 5, 3, 7), (0, 3, 1, 2, 0, 2), (0, 4, 0, 0, 4, 0),
                (0, 2, 4, 7, 1, 3), (0, 1, 7, 3, 5, 6), (0, 7, 0, 7, 5, 1))
PRIMARY_TREE = ('(((((U1:0.5,U2:0.5):0.5,U3:1.0):1.0,(U9:0.4,U10:0.6):0.7):0.3,'
                '(U4:0.75,(U5:0.5,((U6:0.33,U7:0.62):0.5,U8:0.5):0.5):0.5):1.25):0.2,'
                '(U11:0.9,U12:1.1):0.8)root;\n')
SMALL_ROWS = (
    (1,22,21),(35,26,41),(40,5,44),(9,12,36),(40,32,33),(18,27,45),(4,1,29),(5,44,31),
    (1,11,11),(23,11,30),(35,29,11),(49,31,44),(1,49,36),(48,41,41),(29,33,23),(38,12,30),
    (43,13,27),(23,12,37),(2,35,23),(0,24,42),(12,40,27),(40,9,37),(8,40,32),(20,11,3),
    (39,0,7),(15,23,28),(47,11,45),(27,35,41),(37,48,5),(46,38,45),(36,48,39),(44,2,48),
    (17,42,14),(9,19,34),(32,18,35),(21,40,32),(47,27,40),(41,35,27),(35,45,0),(19,29,14))
BIG_COUNTS = tuple(map(int, '''
175 196 25 67 211 103 348 185 398 23 72 345 366 42 218 392 167 68 383 391 0 75 311 250
6 275 44 191 325 312 152 183 112 251 189 290 312 201 38 260 137 343 195 72 339 257 192
374 349 92 252 71 279 10 233 291 317 391 151 92 247 224 149 356 67 379 219 19 4 275 33
123 286 185 229 329 383 130 83 21 119 67 203 47 272 12 349 107 233 393 332 323 110 142
124 276 309 399 315 35 304 260 299 250 54 43 215 39 131 396 335 285 136 54 166 373 198
149 44 135 209 197 100 180 169 29 282 320 252 190 214 226 12 134 106 110 372 349 234
17 5 335 49 332 105 182 175 87 149 260 76 233 331 262 376 225 85 158 363 312 307 247
183 121 396 76 365 199 85 186 297 230 399 299 315 338 63 389 35 77 288 117 215 161 45
116 398 45 361 42 352 290 95 368 183 230 265 269 341 1 287 398 208 48 153 161 255 338
64 314 266 27 156 8 247 100 254 310 298 348 7 241 143 128 210 231 176 44 9 302 176 127
5 144 11 362 160 296 36 69 363 93 49 317 366 223 135 92 314 178 176 334 47 126 211 105
77 24 67 391 49 33 331 347 68 230 95 125 251 61 350 78 364 76 369 122 380 353 8 180
94 353 207 184 181 34 268 260 44 311 276 299 274 184 149 121 18 359 247 5
'''.split()))


def sha256(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def save(path, value):
    with Path(path).open('x') as stream:
        json.dump(encode(value), stream, indent=2, allow_nan=False)
        stream.write('\n')


def encode(value):
    if isinstance(value, float) and not math.isfinite(value):
        return dict(nonfinite='nan' if math.isnan(value) else '+inf' if value > 0 else '-inf')
    if isinstance(value, dict):
        return {str(key): encode(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [encode(item) for item in value]
    if hasattr(value, 'tolist'):
        return encode(value.tolist())
    return value


def table_text(features, samples, rows, header='feature'):
    return '\t'.join([header, *samples]) + '\n' + ''.join(
        '\t'.join([feature, *map(str, row)]) + '\n' for feature, row in zip(features, rows))


def prepare_inputs(output):
    output.mkdir(parents=True)
    texts = {
        'primary-counts.tsv': table_text([f'U{i}' for i in range(1, 13)], [f'S{i}' for i in range(1, 7)], PRIMARY_ROWS),
        'primary-tree.nwk': PRIMARY_TREE,
        'legacy-small.tsv': table_text([f'OTU{i}' for i in range(40)], ['S0', 'S1', 'S2'], SMALL_ROWS, '#OTU ID'),
        'legacy-big.tsv': table_text([f'OTU{i}' for i in range(300)], ['S0'], [(x,) for x in BIG_COUNTS], '#OTU ID'),
    }
    expected = dict(zip(texts, ('b79678b55391e8c43a8e47a84de4a71e92f2d738353526cc9af6547de53559a7',
        '61ea355b828455e9cca5bc92ad99959ebf3afb0df6b846af7ad4f34206565cbd',
        'c8f56e0111400ce52398a253b182fede1af2e1016b200526987573024f5e06b4',
        '88ee972c87413a617591cc93bb679944ecb29811c19b70d72c00894e2fba46da')))
    records = {}
    for name, text in texts.items():
        path = output / name
        path.write_text(text)
        if sha256(path) != expected[name]:
            raise ValueError(f'embedded historical fixture bytes differ: {name}')
        product = 'rsomics-unifrac' if name.startswith('primary') else 'rsomics-subsample-counts'
        records[name] = dict(sha256=expected[name], size_bytes=path.stat().st_size, source_asset=product,
            source_commit='a631051290342991f411b68d7c10735493162a25' if product == 'rsomics-unifrac'
                          else '715ab363e0cabe91ddaa6457ef064930a9d278a9')
    save(output / 'provenance.json', records)
    return records


def read_counts(path):
    with Path(path).open(newline='') as stream:
        records = list(csv.reader(stream, delimiter='\t'))
    header, rows = records[0], records[1:]
    if header[0] not in ('feature', '#OTU ID') or len(header) < 2:
        raise ValueError('expected explicit feature-by-sample TSV header')
    features, samples = [row[0] for row in rows], header[1:]
    if any(not key for key in features + samples) or len(set(features)) != len(features) or len(set(samples)) != len(samples):
        raise ValueError('empty or duplicate identity')
    counts = []
    for row in rows:
        if len(row) != len(header) or any(not re.fullmatch(r'[0-9]+', cell.strip()) for cell in row[1:]):
            raise ValueError('ragged or non-integer count row')
        counts.append([int(cell) for cell in row[1:]])
    if any(sum(column) > MAX_EXACT_INTEGER for column in zip(*counts)):
        raise ValueError('sample total exceeds declared exact-integer source profile')
    return features, samples, counts


def observe(output, callback, required=True):
    output.mkdir(parents=True)
    with (output / 'stdout.txt').open('w') as stdout, (output / 'stderr.txt').open('w') as stderr, \
            contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr), warnings.catch_warnings(record=True) as seen:
        warnings.simplefilter('always')
        try:
            result = dict(status='returned', result=encode(callback()))
        except Exception as error:
            result = dict(status='raised', exception=dict(type=type(error).__name__, module=type(error).__module__,
                message=str(error), traceback=traceback.format_exc()))
        result['warnings'] = [dict(type=type(row.message).__name__, message=str(row.message),
                                   filename=row.filename, lineno=row.lineno) for row in seen]
    save(output / 'observation.json', result)
    if required and result['status'] != 'returned':
        raise RuntimeError(f'required source call failed: {output}')
    return result


def qualify(output, skbio_source, biom_source):
    import numpy as np
    import scipy
    import skbio
    import biom
    from threadpoolctl import threadpool_info

    output.mkdir()
    prefix = Path(sys.prefix).resolve(strict=True)
    if not prefix.is_relative_to(Path(os.environ['RUNNER_TEMP']).resolve(strict=True)):
        raise ValueError('analysis environment is not in hosted runner-temp')
    packages = {}
    for name in ('numpy', 'scipy', 'scikit-bio', 'biom-format'):
        distribution = importlib.metadata.distribution(name)
        packages[name] = dict(version=distribution.version, location=str(distribution.locate_file('')))
        if not Path(packages[name]['location']).resolve(strict=True).is_relative_to(prefix):
            raise ValueError(f'analysis package escapes qualified environment: {name}')
        if name in SOURCES and distribution.version != SOURCES[name]['version']:
            raise ValueError(f'incorrect installed source release: {name}')
    roots = dict(skbio=Path(skbio.__file__).parent, biom=Path(biom.__file__).parent)
    compared = {}
    for relative in ('skbio/diversity/_driver.py', 'skbio/diversity/_util.py', 'skbio/diversity/alpha/_base.py',
                     'skbio/diversity/alpha/_chao1.py', 'skbio/diversity/alpha/_pd.py',
                     'skbio/diversity/beta/_unifrac.py', 'skbio/stats/_subsample.py',
                     'skbio/tree/_utils.py', 'skbio/tree/_tree.py', 'skbio/util/_misc.py'):
        name, rest = relative.split('/', 1)
        source, installed = skbio_source / relative, roots[name] / rest
        if sha256(source) != sha256(installed):
            raise ValueError(f'installed Python body differs from pinned source: {relative}')
        compared[relative] = dict(source_sha256=sha256(source), installed_sha256=sha256(installed))
    compared['biom/_subsample.pyx'] = dict(source_sha256=sha256(biom_source / 'biom/_subsample.pyx'),
        identity_scope='pinned released Cython source; installed released wheel binary is separately SHA-bound, not rebuilt here')
    modules = ('numpy.random._generator', 'numpy.random._pcg64', 'biom._subsample', 'skbio.diversity._phylogenetic')
    binaries = {name: dict(path=importlib.import_module(name).__file__, sha256=sha256(importlib.import_module(name).__file__))
                for name in modules}
    functions = output / 'functions'
    functions.mkdir()
    names = ['skbio.diversity.' + name for name in ('alpha_diversity', 'beta_diversity')]
    names += ['skbio.diversity.alpha.' + name for name in (*ALPHA, 'faith_pd', 'phydiv')]
    names += ['skbio.stats.subsample_counts', 'skbio.util.get_rng']
    function_records = {}
    for name in names:
        module, attribute = name.rsplit('.', 1)
        function = getattr(importlib.import_module(module), attribute)
        path = functions / (name + '.py')
        path.write_text(inspect.getsource(inspect.unwrap(function)))
        function_records[name] = dict(module=function.__module__, qualname=function.__qualname__,
            source_file=inspect.getsourcefile(function), captured_body_sha256=sha256(path))
    with (output / 'blas-config.txt').open('w') as stream, contextlib.redirect_stdout(stream):
        np.show_config()
        scipy.show_config()
    np_version = packages['numpy']['version']
    with urllib.request.urlopen(f'https://pypi.org/pypi/numpy/{np_version}/json', timeout=60) as response:
        metadata = json.load(response)
    save(output / 'numpy-pypi.json', metadata)
    sdists = [item for item in metadata['urls'] if item['packagetype'] == 'sdist']
    if len(sdists) != 1:
        raise ValueError('ambiguous exact NumPy source archive')
    record = sdists[0]
    archive = output / record['filename']
    with urllib.request.urlopen(record['url'], timeout=120) as response, archive.open('wb') as stream:
        while chunk := response.read(1024 * 1024):
            stream.write(chunk)
    if sha256(archive) != record['digests']['sha256']:
        raise ValueError('NumPy source archive digest differs from exact PyPI release')
    with tarfile.open(archive) as source:
        member = source.getmember(f'numpy-{np_version}/numpy/random/_generator.pyx')
        (output / 'numpy-_generator.pyx').write_bytes(source.extractfile(member).read())
    runtime = dict(python=sys.version, executable=sys.executable, prefix=sys.prefix, machine=platform.uname()._asdict(),
        packages=packages, resolved_distributions=sorted((dist.metadata['Name'], dist.version)
            for dist in importlib.metadata.distributions()), source_pins=SOURCES, source_body_comparisons=compared,
        compiled_runtime_modules=binaries, functions=function_records, threadpools=threadpool_info(),
        thread_environment={key: os.environ.get(key) for key in ('OMP_NUM_THREADS', 'OPENBLAS_NUM_THREADS', 'MKL_NUM_THREADS')},
        numpy_generator_source=dict(version=np_version, archive_sha256=sha256(archive),
            choice_source_sha256=sha256(output / 'numpy-_generator.pyx')),
        rng=dict(bit_generator=type(np.random.default_rng(42).bit_generator).__name__,
                 initial_state=np.random.default_rng(42).bit_generator.state,
                 column_policy='independent vector source calls, each freshly initialized with the same supplied seed'))
    if runtime['rng']['bit_generator'] != 'PCG64' or any(value != '1' for value in runtime['thread_environment'].values()):
        raise ValueError('unqualified generator or source threading profile')
    save(output / 'runtime.json', runtime)
    return runtime


def series_result(value):
    return dict(kind='series', ids=list(value.index), values=value.to_numpy(), dtype=str(value.dtype), name=value.name)


def matrix_result(value):
    return dict(kind='distance-matrix', ids=list(value.ids), values=value.data,
                shape=list(value.data.shape), dtype=str(value.data.dtype))


def capture_diversity(output, features, samples, counts, tree_text, required, full=True):
    import numpy as np
    from skbio import TreeNode
    from skbio.diversity import alpha_diversity, beta_diversity

    output.mkdir(parents=True)
    save(output / 'request.json', dict(feature_ids=features, sample_ids=samples, counts=counts, tree=tree_text,
        input_orientation='sample-by-feature for the source public drivers', required_return=required))
    counts = np.asarray(counts)
    results = {}
    if full:
        for metric in ALPHA:
            results[metric] = observe(output / 'alpha' / metric,
                lambda metric=metric: series_result(alpha_diversity(metric, counts, ids=samples)), required)
    tree = TreeNode.read(io.StringIO(tree_text))
    results['faith_pd'] = observe(output / 'alpha/faith_pd', lambda: series_result(
        alpha_diversity('faith_pd', counts, ids=samples, taxa=features, tree=tree)), required)
    variants = PHYDIV if full else PHYDIV[:1]
    for name, rooted, weight in variants:
        options = dict(weight=weight)
        if rooted is not None:
            options['rooted'] = rooted
        results['phydiv-' + name] = observe(output / 'alpha' / ('phydiv-' + name), lambda options=options: series_result(
            alpha_diversity('phydiv', counts, ids=samples, taxa=features, tree=tree, **options)), required)
    for metric in BETA if full else BETA[5:]:
        options = dict(taxa=features, tree=tree) if 'unifrac' in metric else {}
        source_metric = 'weighted_unifrac' if metric == 'weighted_normalized_unifrac' else metric
        if metric == 'weighted_normalized_unifrac':
            options['normalized'] = True
        results[metric] = observe(output / 'beta' / metric, lambda metric=source_metric, options=options: matrix_result(
            beta_diversity(metric, counts, ids=samples, **options)), required)
    if full:
        with (output / 'alpha.tsv').open('w') as stream:
            writer = csv.writer(stream, delimiter='\t', lineterminator='\n')
            writer.writerow(['sample', *ALPHA])
            for index, sample in enumerate(samples):
                values = [results[metric]['result']['values'][index] for metric in ALPHA]
                writer.writerow([sample, *[value['nonfinite'].removeprefix('+') if isinstance(value, dict)
                                          else value for value in values]])
    save(output / 'summary.json', {name: dict(status=result['status'],
         exception=result.get('exception', {}).get('type')) for name, result in results.items()})
    return {name: result['status'] for name, result in results.items()}


def capture_rarefaction(output, features, samples, rows, depth, seed):
    import numpy as np
    from skbio.stats import subsample_counts

    output.mkdir(parents=True)
    save(output / 'request.json', dict(feature_ids=features, sample_ids=samples, rows=rows, depth=depth, seed=seed,
        source_call='skbio.stats.subsample_counts(column, n=depth, replace=False, seed=seed)',
        column_policy='new same-seed vector call for every sample'))
    columns = np.asarray(rows, dtype=np.int64).T
    returned = []
    for sample, column in zip(samples, columns):
        observed = observe(output / sample, lambda column=column: subsample_counts(column, depth, replace=False, seed=seed))
        values = observed['result']
        if sum(values) != depth or any(not 0 <= value <= count for value, count in zip(values, column)):
            raise ValueError('source rarefaction violated integer-count conservation')
        returned.append(values)
    result_rows = [list(row) for row in zip(*returned)]
    (output / 'counts.tsv').write_text(table_text(features, samples, result_rows, '#OTU ID'))
    save(output / 'result.json', dict(feature_ids=features, sample_ids=samples, rows=result_rows, depth=depth, seed=seed))
    return result_rows


def probe_child(output, payload):
    import numpy as np
    from skbio.stats import subsample_counts

    return observe(output, lambda: dict(result=subsample_counts(payload['counts'], payload['depth'],
        replace=False, seed=payload['seed']), source_float_counts=np.asarray(payload['counts'], dtype=float),
        initial_generator_state=np.random.default_rng(payload['seed']).bit_generator.state), required=False)


def capture_probes(output):
    output.mkdir(parents=True)
    probes = dict(empty_zero=dict(counts=[], depth=0), zero_zero=dict(counts=[0, 0], depth=0),
        zero_positive=dict(counts=[0, 0], depth=1), excess=dict(counts=[1, 2], depth=4),
        exact_total=dict(counts=[0, 3, 0, 1], depth=4), fractional=dict(counts=[1.5, 2.5], depth=2),
        negative=dict(counts=[-1, 3], depth=1), exact_integer_max=dict(counts=[MAX_EXACT_INTEGER, 0], depth=1),
        above_exact_integer=dict(counts=[2**53 + 1, 0], depth=0))
    results = {}
    for name, payload in probes.items():
        payload['seed'] = 42
        request = output / (name + '.json')
        save(request, payload)
        destination = output / name
        command = [sys.executable, '-B', str(Path(__file__).resolve()), '--output', str(destination), '--probe', str(request)]
        try:
            result = subprocess.run(command, cwd=output, capture_output=True, timeout=60)
            record = dict(command=command, exit_status=result.returncode,
                          signal=-result.returncode if result.returncode < 0 else None)
            (output / (name + '.stdout')).write_bytes(result.stdout)
            (output / (name + '.stderr')).write_bytes(result.stderr)
        except subprocess.TimeoutExpired as error:
            record = dict(command=command, status='timeout', timeout_seconds=60)
            (output / (name + '.stdout')).write_bytes(error.stdout or b'')
            (output / (name + '.stderr')).write_bytes(error.stderr or b'')
        observation = destination / 'observation.json'
        record['observation'] = json.loads(observation.read_text()) if observation.is_file() else dict(status='no_return')
        save(output / (name + '.process.json'), record)
        results[name] = record
    return results


def capture(output, skbio_source, biom_source):
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    if platform.system() != 'Linux' or not output.resolve().is_relative_to(runner):
        raise ValueError('source execution requires hosted Linux RUNNER_TEMP output')
    output.mkdir()
    runtime = qualify(output / 'qualification', skbio_source, biom_source)
    fixtures = prepare_inputs(output / 'inputs')
    features, samples, rows = read_counts(output / 'inputs/primary-counts.tsv')
    columns = [list(column) for column in zip(*rows)]
    cases = {}
    cases['original'] = capture_diversity(output / 'cases/original', features, samples, columns, PRIMARY_TREE, True)
    rarefied = capture_rarefaction(output / 'cases/rarefied', features, samples, rows, 8, 42)
    cases['rarefied'] = capture_diversity(output / 'cases/rarefied/diversity', features, samples,
        [list(column) for column in zip(*rarefied)], PRIMARY_TREE, True)
    for name, depth, seed in (('small', 200, 42), ('big', 29648, 123)):
        fids, sids, values = read_counts(output / f'inputs/legacy-{name}.tsv')
        capture_rarefaction(output / 'sampling-regimes' / name, fids, sids, values, depth, seed)
    boundary = dict(zero_samples=(features, ['empty-a', 'empty-b', 'positive'], [[0] * 12, [0] * 12, columns[0]], PRIMARY_TREE, True),
        no_features=([], ['a', 'b'], [[], []], PRIMARY_TREE, True),
        root_length=(features, samples, columns, PRIMARY_TREE.replace('root;', 'root:2.5;'), False),
        zero_lengths=(features, samples, columns, re.sub(r':[0-9.]+', ':0', PRIMARY_TREE), False),
        extra_tips=(features[:-1], samples, [column[:-1] for column in columns], PRIMARY_TREE, False),
        polytomy=(['U1', 'U2', 'U3'], ['a', 'b'], [[1, 0, 2], [0, 3, 1]], '(U1:1,U2:2,U3:3)root;', False),
        unary=(['U1', 'U2'], ['a', 'b'], [[1, 0], [0, 2]], '(((U1:1,U2:2):1):2)root;', False),
        missing_tip=(['missing', *features[1:]], samples, columns, PRIMARY_TREE, False),
        duplicate_taxa=([features[0], *features[:-1]], samples, columns, PRIMARY_TREE, False),
        duplicate_tip=(features, samples, columns, PRIMARY_TREE.replace('U2:', 'U1:'), False),
        missing_length=(features, samples, columns, PRIMARY_TREE.replace('U1:0.5', 'U1'), False),
        negative_length=(features, samples, columns, PRIMARY_TREE.replace('U1:0.5', 'U1:-0.5'), False))
    for name, (fids, sids, values, tree, full) in boundary.items():
        cases[name] = capture_diversity(output / 'source-boundaries' / name, fids, sids, values, tree, False, full)
    for name, value in (('negative', -1), ('fractional', .5), ('nan', float('nan')), ('positive-infinity', float('inf'))):
        changed = [column.copy() for column in columns]
        changed[0][0] = value
        cases[name] = capture_diversity(output / 'source-boundaries' / ('counts-' + name), features, samples,
            changed, PRIMARY_TREE, False, full=False)
    probes = capture_probes(output / 'rarefaction-boundaries')
    files = {str(path.relative_to(output)): sha256(path) for path in sorted(output.rglob('*')) if path.is_file()}
    witness = dict(schema_version=1, status='complete_source_capture', product='rsomics-ecology', source_pins=SOURCES,
        scope='actual pinned source diversity and without-replacement vector workflow; not native acceptance, performance or a public undefined-distance policy',
        runtime=runtime, fixtures=fixtures, cases=cases, rarefaction_boundary_statuses={name: dict(
            exit_status=result.get('exit_status'), status=result['observation']['status']) for name, result in probes.items()},
        alpha_metrics=list(ALPHA), phydiv_profiles=PHYDIV, beta_metrics=list(BETA), bounds=BOUNDS,
        exact_contract=['sample and feature identities/order', 'integer rarefaction outputs', 'source unavailable classifications'],
        integer_profile=dict(kind='nonnegative integer counts', maximum_cell_and_sample_total=MAX_EXACT_INTEGER,
            policy='conservative exact float-to-int64 source input profile; wider source behavior is captured, not advertised'),
        distance_public_boundary='pending user decision; source NaN/exception observations are not silently repaired', files=files)
    save(output / 'witness.json', witness)
    files['witness.json'] = sha256(output / 'witness.json')
    manifest = output / 'sha256-manifest.tsv'
    manifest.write_text('sha256\tpath\n' + ''.join(f'{digest}\t{name}\n' for name, digest in sorted(files.items())))
    return dict(witness_sha256=files['witness.json'], manifest_sha256=sha256(manifest), files=len(files), status=witness['status'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--skbio-source', type=Path)
    parser.add_argument('--biom-source', type=Path)
    parser.add_argument('--probe', type=Path)
    args = parser.parse_args()
    existed = args.output.exists()
    try:
        if args.probe:
            result = probe_child(args.output, json.loads(args.probe.read_text()))
        else:
            if args.skbio_source is None or args.biom_source is None:
                parser.error('hosted capture requires --skbio-source and --biom-source')
            result = capture(args.output, args.skbio_source, args.biom_source)
        print(json.dumps(result, allow_nan=False))
    except (OSError, ValueError, RuntimeError, ImportError, tarfile.TarError) as error:
        if args.output.is_dir() and not existed:
            save(args.output / 'failure.json', dict(type=type(error).__name__, message=str(error), traceback=traceback.format_exc()))
        parser.exit(1, f'actual ecology source capture failed: {error}\n')


if __name__ == '__main__':
    main()
