"""Hosted pinned-source adapter; no scientific imports during local preparation."""

import argparse
import csv
import hashlib
import importlib
import importlib.metadata
import io
import json
import math
import multiprocessing
import os
from pathlib import Path
import platform
import resource
import signal
import sys
import time
import traceback


ROOT = Path(__file__).resolve().parents[1]
ASSET = ROOT / 'assets/ecology-whole-real-20261101.json'
CTX = {}


def load(path):
    return json.loads(Path(path).read_bytes())


def save(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('x') as stream:
        json.dump(value, stream, indent=2, sort_keys=True, allow_nan=False)
        stream.write('\n')


def identity(path):
    path = Path(path)
    with path.open('rb') as stream:
        return dict(bytes=path.stat().st_size, sha256=hashlib.file_digest(stream, 'sha256').hexdigest())


def require(condition, message):
    if not condition:
        raise ValueError(message)


def hosted(session):
    session = Path(session).resolve(strict=True)
    require(os.environ.get('GITHUB_ACTIONS') == 'true'
            and os.environ.get('GITHUB_REPOSITORY') == 'omics-rust/rsomics-world'
            and platform.system() == 'Linux' and platform.machine() == 'x86_64',
            'source execution requires the actual hosted Linux workflow')
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    require(session.is_relative_to(runner) and session != runner
            and session.stat().st_dev == runner.stat().st_dev, 'private source root escapes runner device')
    require(Path(sys.prefix).resolve(strict=True).is_relative_to(session / 'runtime')
            and sys.version_info[:3] == (3, 12, 10), 'source interpreter/prefix differs')
    for name in ('OMP_NUM_THREADS', 'OPENBLAS_NUM_THREADS', 'MKL_NUM_THREADS', 'NUMEXPR_NUM_THREADS'):
        require(os.environ.get(name) == '1', 'source thread environment differs: ' + name)
    return session


def scientific(need_biom=False):
    import numpy as np
    from skbio import TreeNode
    from skbio.diversity import alpha_diversity
    from skbio.diversity import alpha
    from skbio.diversity._phylogenetic import _nodes_by_counts
    from skbio.diversity._util import _validate_counts_matrix
    from skbio.diversity.alpha._pd import _faith_pd, _phydiv
    from skbio.stats import subsample_counts
    from skbio.tree._utils import _validate_taxa_and_tree
    if need_biom:
        import h5py
    return dict(np=np, h5py=h5py if need_biom else None, TreeNode=TreeNode, alpha_diversity=alpha_diversity,
                alpha=alpha, nodes=_nodes_by_counts, validate_counts=_validate_counts_matrix,
                faith=_faith_pd, phydiv=_phydiv, subsample=subsample_counts,
                validate_tree=_validate_taxa_and_tree)


def admit(session, policy, science):
    from threadpoolctl import threadpool_info
    prefix = Path(sys.prefix).resolve(strict=True)
    site = prefix / 'lib/python3.12/site-packages'
    versions = {}
    for name, row in policy['source_runtime']['wheels'].items():
        distribution = importlib.metadata.distribution(name)
        require(distribution.version == row['version']
                and Path(distribution.locate_file('')).resolve(strict=True).is_relative_to(prefix),
                'installed wheel/version escapes source contract: ' + name)
        versions[name] = distribution.version
    source_bodies = {}
    for name, sha in policy['source_runtime']['installed_python_sources'].items():
        source_bodies[name] = identity(site / name)
        require(source_bodies[name]['sha256'] == sha, 'installed source differs: ' + name)
    modules = {}
    for name, row in policy['source_runtime']['compiled_modules'].items():
        path = Path(importlib.import_module(name).__file__).resolve(strict=True)
        require(path == (site / row['relative']).resolve(strict=True), 'compiled module prefix differs')
        modules[name] = identity(path)
        require(modules[name]['sha256'] == row['sha256'], 'compiled module body differs: ' + name)
    pools = threadpool_info()
    require(all(row['num_threads'] == 1 for row in pools), 'source thread pool is oversubscribed')
    report = dict(python=sys.version, executable=sys.executable, prefix=str(prefix), versions=versions,
                  installed_source=source_bodies, compiled_modules=modules, threadpools=pools,
                  original_python='3.12.15', identical_original_interpreter=False)
    save(session / 'runtime/source-admission.json', report)
    return report


def ids_hash(ids):
    return hashlib.sha256(json.dumps(ids, ensure_ascii=False).encode()).hexdigest()


def sync_directory(path):
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def write_counts(path, counts, features, samples):
    with Path(path).open('x', newline='') as stream:
        stream.write('\t'.join(['feature', *samples]) + '\n')
        for column, feature in enumerate(features):
            stream.write(feature + '\t' + '\t'.join(map(str, counts[:, column].tolist())) + '\n')
        stream.flush()
        os.fsync(stream.fileno())
    sync_directory(Path(path).parent)


def write_alpha(path, values, samples, metrics):
    with Path(path).open('x', newline='') as stream:
        stream.write('\t'.join(['sample', *metrics]) + '\n')
        for sample, row in zip(samples, values):
            stream.write(sample + '\t' + '\t'.join(str(float(value)) for value in row) + '\n')
        stream.flush()
        os.fsync(stream.fileno())
    sync_directory(Path(path).parent)


def read_counts(path, features, samples, np):
    matrix = np.empty((len(samples), len(features)), dtype=np.int64)
    with Path(path).open() as stream:
        require(stream.readline().rstrip('\n').split('\t') == ['feature', *samples], 'count sample header differs')
        for column, feature in enumerate(features):
            name, separator, text = stream.readline().rstrip('\n').partition('\t')
            require(separator and name == feature, 'count feature order differs')
            values = np.fromstring(text, dtype=np.int64, sep='\t')
            require(len(values) == len(samples), 'count column count differs')
            matrix[:, column] = values
        require(stream.read() == '', 'extra count row')
    return matrix


def prepare(session, policy, science):
    np, h5py = science['np'], science['h5py']
    data = session / 'data'
    study = policy['study']
    with h5py.File(data / 'feature-table.biom', 'r') as table:
        features = table['observation/ids'].asstr()[:].tolist()
        samples = table['sample/ids'].asstr()[:].tolist()
        require(len(features) == study['features'] and len(samples) == study['samples']
                and ids_hash(features) == study['feature_ids_sha256']
                and ids_hash(samples) == study['sample_ids_sha256'], 'original BIOM identities differ')
        sparse = []
        for name, size, other in (('observation', len(features), len(samples)),
                                  ('sample', len(samples), len(features))):
            group = table[name + '/matrix']
            values, indices, pointers = (group[key][:] for key in ('data', 'indices', 'indptr'))
            require(len(values) == len(indices) == study['nonzeros'] and len(pointers) == size + 1
                    and pointers[0] == 0 and pointers[-1] == len(values)
                    and np.all(np.diff(pointers) >= 0) and np.all((indices >= 0) & (indices < other))
                    and np.all(np.isfinite(values)) and np.all(values > 0)
                    and np.all(values == np.floor(values)) and np.all(values <= 2**53 - 1),
                    'BIOM sparse count structure/domain differs: ' + name)
            rows = np.repeat(np.arange(size), np.diff(pointers))
            coordinates = np.column_stack((rows, indices)) if name == 'observation' else np.column_stack((indices, rows))
            order = np.lexsort((coordinates[:, 1], coordinates[:, 0]))
            coordinates, values = coordinates[order], values[order].astype(np.int64)
            require(not np.any(np.all(coordinates[1:] == coordinates[:-1], axis=1)), 'duplicate BIOM coordinate')
            sparse.append((coordinates, values))
    require(np.array_equal(sparse[0][0], sparse[1][0]) and np.array_equal(sparse[0][1], sparse[1][1]),
            'complete BIOM sparse views disagree')
    coordinates, values = sparse[0]
    matrix = np.zeros((len(samples), len(features)), dtype=np.int64)
    matrix[coordinates[:, 1], coordinates[:, 0]] = values
    totals = matrix.sum(axis=1)
    require(int(totals.sum()) == study['total_reads'] and int(totals.min()) == 3
            and int(totals.max()) == 84945 and np.all(totals <= 2**53 - 1), 'whole-study totals differ')
    output = data / 'study.tsv'
    write_counts(output, matrix, features, samples)
    reparsed = read_counts(output, features, samples, np)
    require(np.array_equal(matrix, reparsed), 'complete BIOM-to-TSV coordinate/value conversion differs')
    tree = science['TreeNode'].read(io.StringIO((data / 'tree.nwk').read_text()))
    nodes, tips = list(tree.traverse()), list(tree.tips())
    tip_ids = [tip.name for tip in tips]
    require(len(nodes) == study['tree_nodes'] and len(tips) == len(set(tip_ids)) == study['tree_tips']
            and len(tree.children) == study['root_degree'] and tree.length is None
            and len(set(tip_ids) - set(features)) == study['tree_extra_tips']
            and set(features) <= set(tip_ids), 'complete tree topology/identity differs')
    require(all(node.length is not None and math.isfinite(node.length) and node.length >= 0
                for node in nodes if node is not tree), 'required original branch length differs')
    with (data / 'sample-metadata.tsv').open(newline='') as stream:
        rows = list(csv.reader(stream, delimiter='\t'))
    metadata = [row for row in rows[1:] if row and not row[0].startswith('#')]
    metadata_ids = [row[0] for row in metadata]
    require(len(metadata) == len(set(metadata_ids)) == study['metadata_rows']
            and all(len(row) == len(rows[0]) for row in rows[1:] if row)
            and set(samples) <= set(metadata_ids)
            and len(set(metadata_ids) - set(samples)) == study['metadata_extra_rows'], 'whole-study metadata join differs')
    coordinate_hash = hashlib.sha256(coordinates.astype('<i8').tobytes() + values.astype('<i8').tobytes()).hexdigest()
    report = dict(features=features, samples=samples, feature_ids_sha256=ids_hash(features),
        sample_ids_sha256=ids_hash(samples), sparse_coordinate_count_sha256=coordinate_hash,
        counts=identity(output), tree=identity(data / 'tree.nwk'), samples_total=totals.tolist(),
        source_biom=identity(data / 'feature-table.biom'), nonzeros=len(values), total_reads=int(totals.sum()),
        tree_nodes=len(nodes), tree_tips=len(tips), depth=3, seed=42, no_sample_or_feature_drop=True)
    save(data / 'admission.json', report)
    return report


def usage(row):
    return dict(user_seconds=row.ru_utime, system_seconds=row.ru_stime, rss_bytes=row.ru_maxrss * 1024,
                minor_faults=row.ru_minflt, major_faults=row.ru_majflt,
                input_blocks=row.ru_inblock, output_blocks=row.ru_oublock)


def calculate(kind, start, end):
    np, counts = CTX['science']['np'], CTX['counts'][start:end]
    if kind == 'rarefy':
        return np.stack([CTX['science']['subsample'](row, CTX['depth'], replace=False, seed=CTX['seed'])
                         for row in counts])
    node_counts = CTX['science']['nodes'](counts, CTX['taxa'], CTX['index']).T
    functions = [getattr(CTX['science']['alpha'], name) for name in CTX['metrics'][:9]]
    result = np.empty((end - start, len(CTX['metrics'])), dtype=float)
    for number, (row, nodes) in enumerate(zip(counts, node_counts)):
        result[number, :9] = [function(row) for function in functions]
        result[number, 9] = CTX['science']['faith'](nodes, CTX['lengths'])
        result[number, 10] = CTX['science']['phydiv'](nodes, CTX['lengths'], CTX['rooted'], .25)
    return result


def parallel(kind, workers):
    np = CTX['science']['np']
    count = len(CTX['samples'])
    if workers == 1:
        return calculate(kind, 0, count)
    cpus = sorted(os.sched_getaffinity(0))
    require(len(cpus) == workers == 4, 'source parallel allowance differs')
    shape = (count, len(CTX['features']) if kind == 'rarefy' else len(CTX['metrics']))
    result = np.empty(shape, dtype=np.int64 if kind == 'rarefy' else float)
    children, readers, waited = [], [], set()
    try:
        for number in range(workers):
            start, end = count * number // workers, count * (number + 1) // workers
            receive, send = multiprocessing.Pipe(duplex=False)
            pid = os.fork()
            if pid == 0:
                receive.close()
                for connection in readers:
                    connection.close()
                try:
                    os.sched_setaffinity(0, {cpus[number]})
                    send.send(('ok', calculate(kind, start, end)))
                    send.close()
                    os._exit(0)
                except BaseException:
                    traceback.print_exc()
                    try:
                        send.send(('failed', traceback.format_exc()))
                    finally:
                        send.close()
                        os._exit(1)
            send.close()
            children.append((pid, start, end, cpus[number]))
            readers.append(receive)
        for (pid, start, end, cpu), receive in zip(children, readers):
            status, values = receive.recv()
            _, code, counters = os.wait4(pid, 0)
            waited.add(pid)
            CTX['children'].append(dict(kind=kind, pid=pid, start=start, end=end, cpu=cpu,
                                        returncode=os.waitstatus_to_exitcode(code), **usage(counters)))
            require(status == 'ok' and os.waitstatus_to_exitcode(code) == 0, 'source worker failed: ' + str(values)[:1000])
            result[start:end] = values
        return result
    finally:
        for connection in readers:
            connection.close()
        for pid, _, _, _ in children:
            if pid not in waited:
                try:
                    os.kill(pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
                os.waitpid(pid, 0)


def measure(end_ns, started_ns):
    own = usage(resource.getrusage(resource.RUSAGE_SELF))
    children = CTX['children']
    return dict(wall=(end_ns - started_ns) / 1e9,
                cpu=own['user_seconds'] + own['system_seconds']
                    + sum(row['user_seconds'] + row['system_seconds'] for row in children),
                rss=max([own['rss_bytes'], *[row['rss_bytes'] for row in children]]),
                parent=own, children=list(children), aggregate_rss_exact=False)


def workflow(session, policy, science, output, workers, started_ns, *, bridge=False, public=False):
    np = science['np']
    if bridge:
        witness = session / 'runtime/original-source/witness'
        request = load(witness / 'cases/original/request.json')
        features, samples = request['feature_ids'], request['sample_ids']
        counts = np.asarray(request['counts'], dtype=np.int64)
        tree_text, depth = request['tree'], 8
    else:
        admitted = load(session / 'data/admission.json')
        features, samples = admitted['features'], admitted['samples']
        counts = read_counts(session / 'data/study.tsv', features, samples, np)
        tree_text, depth = (session / 'data/tree.nwk').read_text(), 3
    counts = science['validate_counts'](counts)
    tree = science['TreeNode'].read(io.StringIO(tree_text))
    science['validate_tree'](features, tree, rooted=True, lengths=True)
    rooted = tree._is_rooted()
    require(rooted is True, 'source auto-root differs from admitted two-child profile')
    metrics = policy['policy']['metrics']
    CTX.clear()
    CTX.update(science=science, counts=counts, taxa=np.asarray(features), features=features, samples=samples,
               metrics=metrics, rooted=rooted, depth=depth, seed=42, children=[])
    if not public:
        index = tree.to_array(nan_length_value=0.0)
        CTX.update(index=index, lengths=index['length'])
    output.mkdir(parents=True, exist_ok=False)
    calls = []

    def alpha_values():
        if not public:
            return parallel('alpha', workers)
        series = []
        for metric in metrics:
            options = dict(taxa=features, tree=tree) if metric in ('faith_pd', 'phydiv') else {}
            if metric == 'phydiv':
                options['weight'] = .25
            returned = science['alpha_diversity'](metric, CTX['counts'], ids=samples, **options)
            require(list(returned.index) == samples, 'public driver sample order differs')
            series.append(returned.to_numpy())
            calls.append(dict(function='skbio.diversity.alpha_diversity', metric=metric, samples=len(samples), returned=True))
        return np.column_stack(series)

    write_alpha(output / 'original-alpha.tsv', alpha_values(), samples, metrics)
    original_end = time.monotonic_ns()
    original = measure(original_end, started_ns)
    if public:
        rarefied = np.stack([science['subsample'](row, depth, replace=False, seed=42) for row in CTX['counts']])
        calls.append(dict(function='skbio.stats.subsample_counts', calls=len(samples), depth=depth, seed=42, returned=True))
    else:
        rarefied = parallel('rarefy', workers)
    write_counts(output / 'rarefied.tsv', rarefied, features, samples)
    CTX['counts'] = rarefied
    del counts
    rarefied_end = time.monotonic_ns()
    write_alpha(output / 'rarefied-alpha.tsv', alpha_values(), samples, metrics)
    end = time.monotonic_ns()
    report = dict(a=original, w=measure(end, started_ns),
                  stages={'rarefy': (rarefied_end - original_end) / 1e9,
                          'rarefied-alpha': (end - rarefied_end) / 1e9},
                  public_driver_reference=public, old_case_bridge=bridge, workers=workers,
                  depth=depth, seed=42, metrics=metrics, rooted='auto', effective_rooted=rooted,
                  weight=.25, public_calls=calls, counts_reused_between_stages=True,
                  shared_node_setup=not public)
    save(output / 'workflow.json', report)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['admit', 'prepare', 'bridge', 'reference', 'workflow'])
    parser.add_argument('--session', type=Path, required=True)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--workers', type=int, choices=[1, 4], default=1)
    parser.add_argument('--started-ns', type=int)
    args = parser.parse_args()
    session = hosted(args.session)
    policy, science = load(ASSET), scientific(need_biom=args.action == 'prepare')
    if args.action == 'admit':
        admit(session, policy, science)
    elif args.action == 'prepare':
        prepare(session, policy, science)
    else:
        workflow(session, policy, science, args.output, args.workers,
                 args.started_ns or time.monotonic_ns(), bridge=args.action == 'bridge',
                 public=args.action == 'reference')
    print(json.dumps(dict(status='ok', action=args.action), allow_nan=False))


if __name__ == '__main__':
    main()
