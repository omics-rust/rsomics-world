"""Hosted public beta adapter; scientific imports stay inside admitted actions."""
import argparse
import importlib
import io
import json
import os
from pathlib import Path
import sys
import warnings
import zipfile

import ecology_source_workflow as base

ROOT = Path(__file__).resolve().parents[1]
ASSET = ROOT / 'assets/ecology-beta-real-20261101.json'
LAYOUT = {}


def science():
    import numpy as np
    from skbio import TreeNode
    from skbio.diversity import beta_diversity
    return np, TreeNode, beta_diversity


def contiguous_pairwise(counts, metric, **kwargs):
    import numpy as np
    from scipy.spatial.distance import pdist
    contiguous = np.ascontiguousarray(counts)
    function = getattr(metric, 'func', metric)
    LAYOUT.update(dtype=str(counts.dtype), shape=list(counts.shape),
        original_strides=list(counts.strides), contiguous_strides=list(contiguous.strides),
        original_c=bool(counts.flags.c_contiguous), original_f=bool(counts.flags.f_contiguous),
        result_c=bool(contiguous.flags.c_contiguous), result_f=bool(contiguous.flags.f_contiguous),
        bytes=int(contiguous.nbytes), copy_required=not bool(counts.flags.c_contiguous),
        callable_module=getattr(function, '__module__', None),
        callable_name=getattr(function, '__qualname__', None),
        profile='contiguous-public', copy_time_and_rss_charged=True,
        upstream_default_memory_claim=False)
    return pdist(contiguous, metric=metric, **kwargs)


def public_matrix(metric, counts, features, samples, tree_text):
    _, TreeNode, beta_diversity = science()
    options = {}
    source_metric = metric
    LAYOUT.clear()
    if 'unifrac' in metric:
        options.update(taxa=features, tree=TreeNode.read(io.StringIO(tree_text)),
                       pairwise_func=contiguous_pairwise)
        if metric != 'unweighted_unifrac':
            source_metric = 'weighted_unifrac'
            options['normalized'] = metric == 'weighted_normalized_unifrac'
    with warnings.catch_warnings(record=True) as observed:
        matrix = beta_diversity(source_metric, counts, ids=samples, validate=True, **options)
    base.require(list(matrix.ids) == samples and list(matrix.shape) == [len(samples), len(samples)],
                 'public beta identities/shape differ')
    return matrix, dict(status='returned', metric=metric, source_metric=source_metric,
        normalized=options.get('normalized'), source_profile='contiguous-public',
        input_shape=list(counts.shape), input_dtype=str(counts.dtype), feature_count=len(features), validate=True,
        upstream_default_performance_claim=False, sample_ids=list(matrix.ids), shape=list(matrix.shape),
        layout=dict(LAYOUT) if LAYOUT else None,
        warnings=[dict(category=row.category.__name__, message=str(row.message),
                       filename=row.filename, line=row.lineno) for row in observed])


def write_matrix(path, matrix):
    path = Path(path)
    partial = Path(str(path) + '.partial')
    base.require(not path.exists() and not partial.exists(), 'fresh scientific output required')
    with partial.open('x', newline='') as output:
        matrix.write(output, format='lsmat', delimiter='\t')
        output.flush()
        os.fsync(output.fileno())
    os.rename(partial, path)
    base.sync_directory(path.parent)


def admit(session, policy):
    science()
    report = base.admit(session, policy, base.scientific())
    site = Path(sys.prefix).resolve(strict=True) / 'lib/python3.12/site-packages'
    loaded = {}
    for module, relative in [('scipy.spatial.distance', 'scipy/spatial/distance.py'),
                             ('skbio.io.format.lsmat', 'skbio/io/format/lsmat.py')]:
        path = Path(importlib.import_module(module).__file__).resolve(strict=True)
        base.require(path == (site / relative).resolve(strict=True), 'actual loaded public module differs')
        loaded[module] = dict(path=str(path), **base.identity(path))
    providers = []
    for pool in report['threadpools']:
        path = Path(pool['filepath']).resolve(strict=True)
        base.require(path.is_relative_to(site), 'loaded provider escapes the private wheel prefix')
        relative, matches = path.relative_to(site).as_posix(), []
        for wheel in policy['source_runtime']['wheels'].values():
            with zipfile.ZipFile(session / 'runtime/original-source/wheels' / wheel['filename']) as archive:
                if relative in archive.namelist():
                    import hashlib
                    body = archive.read(relative)
                    matches.append(dict(wheel=wheel['filename'], member=relative,
                        bytes=len(body), sha256=hashlib.sha256(body).hexdigest()))
        base.require(len(matches) == 1 and base.identity(path) ==
                     {key: matches[0][key] for key in ('bytes', 'sha256')}, 'actual provider body differs')
        providers.append(dict(pool=pool, original_wheel=matches[0]))
    base.save(session / 'runtime/beta-source-admission.json', dict(actual_loaded_modules=loaded,
        providers=providers, source_profile=policy['source_profile'], original_python='3.12.15',
        actual_python=sys.version, same_original_interpreter=False, cpp_kernel_source_inspected=False))


def bridge(session, metric, output):
    np, _, _ = science()
    output.mkdir(parents=True, exist_ok=False)
    comparisons = {}
    witness = session / 'runtime/original-source/witness'
    for name, relative in [('original', 'cases/original'), ('rarefied', 'cases/rarefied/diversity')]:
        request = base.load(witness / relative / 'request.json')
        matrix, observation = public_matrix(metric, np.asarray(request['counts'], dtype=np.int64),
            request['feature_ids'], request['sample_ids'], request['tree'])
        expected = base.load(witness / relative / 'beta' / metric / 'observation.json')
        base.require(expected['status'] == 'returned' and expected['result']['ids'] == list(matrix.ids)
            and expected['result']['shape'] == list(matrix.shape), 'original beta bridge identity differs')
        reference = np.asarray(expected['result']['values'], dtype=float)
        base.require(np.all(np.isfinite(reference)) and np.all(np.isfinite(matrix.data))
            and np.all(np.abs(matrix.data - reference) <= 1e-10 + 1e-10 * np.abs(reference)),
            'selected public layout path differs from original beta observations')
        write_matrix(output / (name + '.tsv'), matrix)
        base.save(output / (name + '.json'), observation)
        comparisons[name] = dict(cells=int(reference.size),
            maximum_absolute_error=float(np.max(np.abs(matrix.data - reference))))
    base.save(output / 'bridge.json', dict(metric=metric, status='qualified', comparisons=comparisons,
        source_profile='contiguous-public', new_source_capture=False, full_default_profile_executed=False))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['admit', 'prepare', 'bridge', 'matrix'])
    parser.add_argument('--session', type=Path, required=True)
    parser.add_argument('--metric', required=True)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    session = base.hosted(args.session)
    policy = base.load(ASSET)
    base.require(args.metric in policy['policy']['metrics']
        and policy['source_profile']['name'] == 'contiguous-public', 'unreviewed beta metric/profile')
    if args.action == 'admit':
        admit(session, policy)
    elif args.action == 'prepare':
        admitted = base.prepare(session, policy, base.scientific(need_biom=True))
        base.save(session / 'data/beta-input-scope.json', dict(full_original=True, rarefaction_performed=False,
            counts=admitted['counts'], nonzeros=admitted['nonzeros'], total_reads=admitted['total_reads'],
            reused_accepted_biom_to_tsv_conversion=True))
    elif args.action == 'bridge':
        bridge(session, args.metric, args.output)
    else:
        np, _, _ = science()
        admitted = base.load(session / 'data/admission.json')
        counts = base.read_counts(session / 'data/study.tsv', admitted['features'], admitted['samples'], np)
        tree = (session / 'data/tree.nwk').read_text() if 'unifrac' in args.metric else None
        matrix, observation = public_matrix(args.metric, counts, admitted['features'], admitted['samples'], tree)
        write_matrix(args.output, matrix)
        base.save(Path(str(args.output) + '.json'), observation)
    print(json.dumps(dict(status='ok', action=args.action, metric=args.metric)))


if __name__ == '__main__':
    main()
