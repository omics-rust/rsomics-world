#!/usr/bin/env python3
"""Run one complete pinned popgen source workflow on already admitted inputs."""

import argparse
from collections import Counter
import csv
import gzip
import hashlib
import importlib.metadata
import json
import math
import os
from pathlib import Path
import platform
import resource
import subprocess
import sys
import time
import traceback
import warnings

import popgen_first_workflow_inputs as inputs


ROOT = Path(__file__).resolve().parents[1]
IMAGE = 'sha256:a689e29bc3adf4663ef9a141d23081252764d1319c63f591a027bd6fd676f4c1'
COMMIT = 'f9e17f71b395695d3d99c58f58c76a06c4ad8e12'


def scalar(value):
    if hasattr(value, 'tolist'):
        return scalar(value.tolist())
    if isinstance(value, dict):
        return {str(key): scalar(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [scalar(item) for item in value]
    if isinstance(value, float) and not math.isfinite(value):
        return 'NaN' if math.isnan(value) else ('+Inf' if value > 0 else '-Inf')
    return value


def save(path: Path, value: object) -> None:
    inputs.save(path, scalar(value))


def validate_runtime_lock(lock: dict) -> None:
    if lock['schema'] != 1 or lock['image']['manifest_digest'] != IMAGE:
        raise ValueError('pinned runtime image differs')
    if lock['scikit_allel']['commit'] != COMMIT or lock['python']['version'] != '3.12.10':
        raise ValueError('pinned source/interpreter differs')
    packages = lock['python_packages']
    for name, package in packages.items():
        if not package['filename'].endswith('.whl') or len(package['sha256']) != 64:
            raise ValueError('wheel-only identity absent: ' + name)
        if not set(package['selected_dependencies']).issubset(packages):
            raise ValueError('wheel dependency closure absent: ' + name)
    if packages['scikit-allel']['version'] != '1.3.13' or packages['numpy']['version'] != '1.26.4' or packages['dask']['version'] != '2024.9.0':
        raise ValueError('scientific runtime profile differs')


def runtime_identity(lock: dict) -> dict:
    validate_runtime_lock(lock)
    if platform.system() != 'Linux' or platform.machine() != 'x86_64' or platform.python_version() != lock['python']['version']:
        raise RuntimeError('source science requires admitted Linux x86_64 CPython3.12.10')
    versions = {name: importlib.metadata.version(name) for name in lock['python_packages']}
    if versions != {name: package['version'] for name, package in lock['python_packages'].items()}:
        raise RuntimeError('installed wheel versions differ from complete lock')
    return dict(python=sys.version, executable=str(Path(sys.executable).resolve()),
                executable_sha256=inputs.sha256(Path(sys.executable)), platform=platform.platform(),
                packages=versions, environment={name: os.environ.get(name) for name in
                ['OMP_NUM_THREADS', 'OPENBLAS_NUM_THREADS', 'MKL_NUM_THREADS', 'BLIS_NUM_THREADS',
                 'NUMEXPR_NUM_THREADS', 'TMPDIR', 'XDG_CACHE_HOME', 'PIP_CACHE_DIR']})


def specification_identity(spec: dict) -> str:
    return hashlib.sha256(json.dumps(spec, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def bind_admission(root: Path, spec: dict, admission: dict) -> dict[str, Path]:
    if admission['schema'] != 1 or admission['specification_sha256'] != specification_identity(spec):
        raise ValueError('input admission specification identity differs')
    paths = inputs.input_paths(root, spec)
    for name, path in paths.items():
        if path.stat().st_size != admission['files'][name]['size']:
            raise ValueError('input admission file identity differs: ' + name)
    return paths


def verify_admission(root: Path, spec: dict, admission: dict) -> None:
    for name, path in bind_admission(root, spec, admission).items():
        if inputs.sha256(path) != admission['files'][name]['sha256']:
            raise ValueError('input admission file identity differs: ' + name)
    if 'legacy' in admission:
        legacy_root = bind_legacy(admission)
        for name, identity in admission['legacy']['files'].items():
            if inputs.sha256(legacy_root / name) != identity['sha256']:
                raise ValueError('legacy input admission identity differs: ' + name)


def bind_legacy(admission: dict) -> Path:
    legacy = admission['legacy']
    if legacy['input_gt_stream_sha256'] != admission['selected_gt_stream_sha256']:
        raise ValueError('legacy input original GT identity differs')
    root = Path(legacy['root']).resolve(strict=True)
    for name, identity in legacy['files'].items():
        path = (root / name).resolve(strict=True)
        if not path.is_relative_to(root) or path.stat().st_size != identity['size']:
            raise ValueError('legacy input admission identity differs: ' + name)
    return root


def read_options(admission: dict) -> dict:
    return dict(fields=['samples', 'calldata/GT', 'variants/CHROM', 'variants/POS', 'variants/REF',
                        'variants/ALT', 'variants/FILTER_PASS', 'variants/AA'],
                samples=admission['sample_ids'], alt_number=admission['max_alt_number'],
                types={'calldata/GT': 'genotype/i2', 'variants/CHROM': 'O', 'variants/REF': 'O',
                       'variants/ALT': 'O', 'variants/AA': 'O'},
                numbers={'calldata/GT': 2, 'variants/AA': 1},
                fills={'calldata/GT': -1, 'variants/ALT': '', 'variants/AA': ''})


def resolve_ancestral(raw: str, alleles: list[str]) -> tuple[int, str]:
    if raw in ('', '.'):
        return -1, 'missing'
    token = raw.split('|', 1)[0].upper()
    if token not in inputs.BASES:
        return -1, 'non_nucleotide'
    indices = [index for index, allele in enumerate(alleles) if allele.upper() == token]
    return ((indices[0], 'matched') if len(indices) == 1 else
            (-1, 'ambiguous' if indices else 'allele_mismatch'))


def read_science(np, allel, paths: dict[str, Path], admission: dict) -> dict:
    data = allel.read_vcf(str(paths['vcf']), **read_options(admission))
    if data is None or data['calldata/GT'].shape != tuple(admission['gt_shape']):
        raise ValueError('scientific VCF reader GT shape differs from admission')
    if data['samples'].tolist() != admission['sample_ids']:
        raise ValueError('scientific VCF reader changed sample order/membership')
    digest = hashlib.sha256()
    for start in range(0, len(data['calldata/GT']), 10000):
        digest.update(np.asarray(data['calldata/GT'][start:start + 10000], dtype='<i2').tobytes(order='C'))
    if digest.hexdigest() != admission['selected_gt_stream_sha256']:
        raise ValueError('scientific VCF reader GT values differ from admission')
    identity = hashlib.sha256()
    for i in range(len(data['variants/POS'])):
        alts = [str(value) for value in data['variants/ALT'][i] if value != '']
        inputs.record_identity(identity, i, str(data['variants/CHROM'][i]), int(data['variants/POS'][i]),
                               str(data['variants/REF'][i]), alts, bool(data['variants/FILTER_PASS'][i]),
                               str(data['variants/AA'][i]))
    if identity.hexdigest() != admission['record_stream_sha256']:
        raise ValueError('scientific VCF reader record/ALT/AA values differ from admission')
    return data


def table(path: Path, header: list[str], rows) -> None:
    def cell(value):
        if hasattr(value, 'item'):
            value = value.item()
        if isinstance(value, float):
            return format(value, '.17g')
        if isinstance(value, (list, tuple)):
            return json.dumps(scalar(value), separators=(',', ':'), allow_nan=False)
        return value
    with gzip.open(path, 'wt', encoding='utf-8', newline='') as stream:
        writer = csv.writer(stream, delimiter='\t', lineterminator='\n')
        writer.writerow(header)
        for row in rows:
            writer.writerow([cell(value) for value in row])


def eligible_data(np, data: dict, masks: dict[str, bytearray]) -> tuple[dict, dict]:
    flags = np.zeros(len(data['variants/POS']), dtype=bool)
    reasons = Counter(dict(retained=0, not_pass=0, not_literal_snv=0, inaccessible=0))
    for i, (chrom, pos, ref, alts, passed) in enumerate(zip(data['variants/CHROM'], data['variants/POS'],
            data['variants/REF'], data['variants/ALT'], data['variants/FILTER_PASS'])):
        alleles = [str(alt) for alt in alts if alt != '']
        record = inputs.Record(i, str(chrom), int(pos), str(ref), alleles, bool(passed), '', [], [])
        reason = inputs.exclusion(record, masks)
        reasons[reason] += 1
        flags[i] = reason == 'retained'
    retained = {key: values[flags] for key, values in data.items() if key != 'samples'}
    retained['ordinal'] = np.flatnonzero(flags)
    retained['n_alt'] = np.count_nonzero(retained['variants/ALT'] != '', axis=1)
    return retained, dict(reasons)


def spectra(np, allel, ac: dict, n_alt, ancestors, populations: list[str], indices: dict) -> dict:
    counts = {pop: len(indices[pop]) * 2 for pop in populations}
    complete = {pop: (n_alt == 1) & (ac[pop].sum(axis=1) == counts[pop]) for pop in populations}
    result = dict(profile='supplied eligible VCF rows only; no projection; no invented invariant bin0; independently folded marginal minor counts',
                  chromosome_counts=counts, populations={})
    for pop in populations:
        folded_rows = complete[pop]
        unfolded_rows = folded_rows & (ancestors >= 0) & (ancestors < 2)
        row_indices = np.flatnonzero(unfolded_rows)
        derived = ac[pop][row_indices, 1 - ancestors[unfolded_rows]]
        result['populations'][pop] = dict(
            folded=allel.sfs_folded(ac[pop][folded_rows, :2], n=counts[pop]),
            unfolded=allel.sfs(derived, n=counts[pop]),
            eligibility=dict(retained=len(n_alt), biallelic=int(np.count_nonzero(n_alt == 1)),
                             folded=int(np.count_nonzero(folded_rows)), unfolded=len(row_indices),
                             incomplete_biallelic=int(np.count_nonzero((n_alt == 1) & ~folded_rows)),
                             unusable_ancestral_complete=int(np.count_nonzero(folded_rows & ~unfolded_rows))))
    first, second = populations
    joint = complete[first] & complete[second]
    polarized = joint & (ancestors >= 0) & (ancestors < 2)
    rows = np.flatnonzero(polarized)
    result['joint'] = dict(
        populations=populations,
        folded=allel.joint_sfs_folded(ac[first][joint, :2], ac[second][joint, :2], n1=counts[first], n2=counts[second]),
        unfolded=allel.joint_sfs(ac[first][rows, 1 - ancestors[polarized]], ac[second][rows, 1 - ancestors[polarized]],
                                 n1=counts[first], n2=counts[second]),
        eligibility=dict(folded=int(np.count_nonzero(joint)), unfolded=len(rows),
                         incomplete_biallelic=int(np.count_nonzero((n_alt == 1) & ~joint)),
                         unusable_ancestral_complete=int(np.count_nonzero(joint & ~polarized))))
    return result


def contig_statistics(np, allel, chrom: str, selected: dict, mask: bytearray,
                      admission: dict, spec: dict, output: Path) -> dict:
    output.mkdir()
    pos = selected['variants/POS']
    g = allel.GenotypeArray(selected['calldata/GT'], copy=False)
    indices = admission['population_indices']
    populations = spec['population_order']
    first, second = populations
    width = admission['max_alt_number']
    ac = {pop: g.count_alleles(subpop=indices[pop], max_allele=width) for pop in populations}
    an = {pop: ac[pop].sum(axis=1) for pop in populations}
    accessible = np.frombuffer(mask, dtype='u1') == ord('P')
    region = dict(start=1, stop=len(mask), is_accessible=accessible)
    windows = allel.position_windows(pos, size=spec['window_bp'], start=1, stop=len(mask), step=spec['window_bp'])
    mpd = {pop: allel.mean_pairwise_difference(ac[pop], fill=np.nan) for pop in populations}
    dxy = allel.mean_pairwise_difference_between(ac[first], ac[second], fill=np.nan)
    fixed = allel.locate_fixed_differences(ac[first], ac[second])
    num, den = allel.hudson_fst(ac[first], ac[second], fill=np.nan)
    a, b, c = allel.weir_cockerham_fst(g, [indices[pop] for pop in populations], max_allele=width)
    ancestor = [resolve_ancestral(str(raw), [str(ref), *[str(alt) for alt in alts if alt != '']])
                for raw, ref, alts in zip(selected['variants/AA'], selected['variants/REF'], selected['variants/ALT'])]
    ancestor_indices = np.array([value[0] for value in ancestor], dtype=int)
    header = ['ordinal0', 'chrom', 'pos1', 'ref', 'alts', 'raw_AA', 'ancestral_allele_index', 'ancestral_status']
    header += [pop + '_' + term for pop in populations for term in ['AN', 'AC', 'mpd', 'no_pair']]
    header += ['dxy_mpd', 'fixed_difference', 'hudson_num', 'hudson_den', 'wc_a', 'wc_b', 'wc_c']
    def sites():
        for i in range(len(pos)):
            row = [int(selected['ordinal'][i]), chrom, int(pos[i]), str(selected['variants/REF'][i]),
                   [str(alt) for alt in selected['variants/ALT'][i] if alt != ''],
                   str(selected['variants/AA'][i]), *ancestor[i]]
            for pop in populations:
                row.extend([an[pop][i], ac[pop][i].tolist(), mpd[pop][i], bool(an[pop][i] < 2)])
            yield row + [dxy[i], bool(fixed[i]), num[i], den[i], a[i].tolist(), b[i].tolist(), c[i].tolist()]
    table(output / 'sites.tsv.gz', header, sites())
    region_results = dict(chrom=chrom, start=1, stop=len(mask), accessible_bp=int(accessible.sum()),
                          records=len(pos), populations={}, between={},
                          ancestral_status_counts=dict(Counter(value[1] for value in ancestor)),
                          denominator='project accessibility, not individual callability')
    columns = {'start1': windows[:, 0], 'stop1': windows[:, 1]}
    def add_windows(prefix: str, value: tuple, per_base: bool) -> None:
        values, actual, *terms = value
        if not np.array_equal(actual, windows):
            raise ValueError('upstream window identity differs: ' + prefix)
        columns[prefix] = values
        if per_base:
            columns[prefix + '_accessible_bp'] = terms[0]
        columns[prefix + '_records'] = terms[-1]
    for pop in populations:
        region_results['populations'][pop] = dict(
            pi=allel.sequence_diversity(pos, ac[pop], **region),
            theta=allel.watterson_theta(pos, ac[pop], **region),
            tajima_d=allel.tajima_d(ac[pop], pos=pos, start=1, stop=len(mask), min_sites=3),
            segregating_sites=int(ac[pop].count_segregating()), n_max=int(an[pop].max()),
            called_AN_min=int(an[pop].min()), no_pair_sites=int(np.count_nonzero(an[pop] < 2)),
            mpd_sum_fill0=np.sum(allel.mean_pairwise_difference(ac[pop], fill=0)),
            theta_tajima_n_policy='source n=max(called AN); no imputed invariant genotypes')
        add_windows(pop + '_pi', allel.windowed_diversity(pos, ac[pop], windows=windows, is_accessible=accessible), True)
        add_windows(pop + '_theta', allel.windowed_watterson_theta(pos, ac[pop], windows=windows, is_accessible=accessible), True)
        add_windows(pop + '_tajima_d', allel.windowed_tajima_d(pos, ac[pop], windows=windows, min_sites=3), False)
    region_results['between'] = dict(dxy=allel.sequence_divergence(pos, ac[first], ac[second], **region),
                                    fixed_differences=int(np.count_nonzero(fixed)),
                                    fixed_density=np.count_nonzero(fixed) / accessible.sum(),
                                    hudson_num_sum=np.nansum(num), hudson_den_sum=np.nansum(den),
                                    wc_a_sum=np.nansum(a), wc_b_sum=np.nansum(b), wc_c_sum=np.nansum(c))
    add_windows('dxy', allel.windowed_divergence(pos, ac[first], ac[second], windows=windows, is_accessible=accessible), True)
    add_windows('fixed_density', allel.windowed_df(pos, ac[first], ac[second], windows=windows, is_accessible=accessible), True)
    add_windows('hudson_fst', allel.windowed_hudson_fst(pos, ac[first], ac[second], windows=windows), False)
    add_windows('wc_fst', allel.windowed_weir_cockerham_fst(pos, g, [indices[pop] for pop in populations],
                                                        windows=windows, max_allele=width), False)
    for name, values in {'hudson_num_sum': num, 'hudson_den_sum': den,
                         'wc_a_sum': a, 'wc_b_sum': b, 'wc_c_sum': c}.items():
        sums, actual, _ = allel.windowed_statistic(pos, values, statistic=np.nansum, windows=windows, fill=0)
        if not np.array_equal(actual, windows):
            raise ValueError('component window identity differs')
        columns[name] = sums
    table(output / 'windows.tsv.gz', list(columns), zip(*columns.values()))
    fst = {}
    calls = {'hudson': allel.average_hudson_fst(ac[first], ac[second], blen=spec['jackknife_variants']),
             'weir_cockerham': allel.average_weir_cockerham_fst(g, [indices[pop] for pop in populations],
                                                             blen=spec['jackknife_variants'], max_allele=width)}
    blen = spec['jackknife_variants']
    blocks = list(allel.index_windows(pos, size=blen, start=0, stop=len(pos), step=blen))
    for name, (point, se, vb, vj) in calls.items():
        fst[name] = dict(point=point, se=se, vb=vb, vj=vj)
    fst.update(profile='source variant-count block jackknife; all-row point estimate; incomplete residual tail excluded from resampling; not demographic confidence interval',
               block_size=blen, point_rows=len(pos), jackknife_rows=len(blocks) * blen,
               residual_tail_rows=len(pos) - len(blocks) * blen,
               residual_tail_index_range=[len(blocks) * blen, len(pos)],
               residual_tail_ordinals=selected['ordinal'][len(blocks) * blen:])
    table(output / 'jackknife-membership.tsv.gz', ['block0', 'start_index0', 'stop_index_exclusive', 'first_ordinal0', 'last_ordinal0', 'first_pos1', 'last_pos1'],
          ((i, start, stop, selected['ordinal'][start], selected['ordinal'][stop - 1], pos[start], pos[stop - 1])
           for i, (start, stop) in enumerate(blocks)))
    save(output / 'region.json', region_results)
    save(output / 'fst-jackknife.json', fst)
    save(output / 'sfs.json', spectra(np, allel, ac, selected['n_alt'], ancestor_indices, populations, indices))
    count_digest = hashlib.sha256()
    for pop in populations:
        count_digest.update(pop.encode() + b'\0')
        count_digest.update(np.asarray(ac[pop], dtype='<i8').tobytes(order='C'))
    return dict(chrom=chrom, retained_rows=len(pos), window_rows=len(windows), allele_counts_sha256=count_digest.hexdigest(),
                allele_counts_identity='population insertion order, UTF8 name NUL, full-width int64 little-endian variant/allele counts',
                output_files={path.name: dict(size=path.stat().st_size, sha256=inputs.sha256(path))
                              for path in output.iterdir() if path.is_file()})


def legacy_commands(destination: Path, executable: str, window_bp: int, prepared: Path | None = None) -> list[dict]:
    prepared = prepared or destination
    prefix = [executable, '--vcf', str(prepared / 'eligible.vcf')]
    fst = ['--weir-fst-pop', str(prepared / 'YRI.txt'), '--weir-fst-pop', str(prepared / 'CEU.txt')]
    operations = [('site-pi', ['--site-pi']),
                  ('window-pi', ['--window-pi', str(window_bp), '--window-pi-step', str(window_bp)]),
                  ('tajima-d', ['--TajimaD', str(window_bp)]),
                  ('weir-fst', fst),
                  ('window-weir-fst', fst + ['--fst-window-size', str(window_bp), '--fst-window-step', str(window_bp)])]
    return [dict(name=name, argv=prefix + flags + ['--out', str(destination / name)]) for name, flags in operations]


def check_casebook_results(output: Path, truth: dict) -> None:
    for chrom, expected in truth.items():
        with gzip.open(output / chrom / 'windows.tsv.gz', 'rt') as stream:
            windows = list(csv.DictReader(stream, delimiter='\t'))
        if [[int(row['start1']), int(row['stop1'])] for row in windows] != expected['windows']:
            raise ValueError('casebook source window coordinates differ: ' + chrom)
        sfs = json.loads((output / chrom / 'sfs.json').read_text())
        for pop in ('YRI', 'CEU'):
            for form in ('folded', 'unfolded'):
                if sfs['populations'][pop][form] != expected[form][pop]:
                    raise ValueError('casebook source SFS differs: ' + chrom + '/' + pop + '/' + form)
        joint = sfs['joint']['unfolded']
        nonzero = [[i, j, count] for i, row in enumerate(joint) for j, count in enumerate(row) if count]
        if sfs['joint']['folded'] != expected['joint_folded'] or nonzero != expected['joint_unfolded_nonzero']:
            raise ValueError('casebook source joint SFS differs: ' + chrom)
        with gzip.open(output / chrom / 'sites.tsv.gz', 'rt') as stream:
            sites = {row['pos1']: row for row in csv.DictReader(stream, delimiter='\t')}
        for position, terms in expected['sites'].items():
            for name, value in terms.items():
                if float(sites[position][name]) != value:
                    raise ValueError('casebook independent site formula differs: ' + chrom + '/' + position + '/' + name)


def legacy(spec: dict, admission: dict, output: Path, executable: Path) -> dict:
    output.mkdir()
    version = subprocess.run([str(executable), '--version'], capture_output=True, text=True, check=True)
    if '0.1.17' not in version.stdout + version.stderr:
        raise ValueError('vcftools source runtime version differs')
    prepared = bind_legacy(admission)
    calls = []
    for call in legacy_commands(output, str(executable), spec['window_bp'], prepared):
        started = time.perf_counter()
        with (output / (call['name'] + '.stdout')).open('xb') as stdout, (output / (call['name'] + '.stderr')).open('xb') as stderr:
            process = subprocess.run(call['argv'], stdout=stdout, stderr=stderr, check=False)
        calls.append(dict(**call, returncode=process.returncode, elapsed_seconds=time.perf_counter() - started))
        save(output / 'commands.json', calls)
        if process.returncode != 0:
            raise RuntimeError('vcftools command failed: ' + call['name'])
    return dict(version_stdout=version.stdout, version_stderr=version.stderr,
                executable_sha256=inputs.sha256(executable), derivative=admission['legacy'],
                profile='named vcftools legacy bins/denominators, not canonical project-accessibility estimates', calls=calls)


def run(root: Path, spec: dict, admission: dict, lock: dict, output: Path, executable: Path | None) -> dict:
    runtime = runtime_identity(lock)
    paths = bind_admission(root, spec, admission)
    bind_legacy(admission)
    import numpy as np
    import allel
    if np.__version__ != '1.26.4' or allel.__version__ != '1.3.13':
        raise RuntimeError('imported scientific versions differ from installed runtime metadata')
    masks = inputs.load_masks(paths, spec)
    data = read_science(np, allel, paths, admission)
    selected, counts = eligible_data(np, data, masks)
    del data
    if counts != admission['filter_counts']:
        raise ValueError('source filtering differs from unified input admission')
    save(output / 'runtime.json', dict(**runtime, numpy_file=np.__file__, scikit_allel_file=allel.__file__))
    save(output / 'input-admission.json', admission)
    save(output / 'runtime-lock.json', lock)
    save(output / 'filter-counts.json', counts)
    results = []
    for chrom in spec['contigs']:
        rows = selected['variants/CHROM'] == chrom
        contig = selected if rows.all() else {key: values[rows] for key, values in selected.items()}
        results.append(contig_statistics(np, allel, chrom, contig, masks[chrom], admission, spec, output / chrom))
    result = dict(contigs=results, selected_gt_stream_sha256=admission['selected_gt_stream_sha256'],
                  record_stream_sha256=admission['record_stream_sha256'],
                  exclusions='selection/admixture/PBS/LD; individual callability; genome-wide invariant SFS; demographic/clinical ancestry claims',
                  source_calls='pinned scikit-allel1.3.13 canonical source; vcftools0.1.17 named legacy lane')
    if executable is not None:
        result['vcftools'] = legacy(spec, admission, output / 'vcftools', executable)
    else:
        raise ValueError('complete first source acceptance requires the admitted vcftools executable')
    if 'independent_truth' in spec:
        check_casebook_results(output, spec['independent_truth'])
        result['independent_casebook'] = 'hand-counted SFS/window identities and exact binary-representable pi/Dxy/Hudson examples matched'
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['verify-admission', 'run'])
    parser.add_argument('--inputs', type=Path, required=True)
    parser.add_argument('--specification', type=Path, required=True)
    parser.add_argument('--admission', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, default=ROOT / 'assets/popgen-first-runtime-20261101.json')
    parser.add_argument('--output', type=Path)
    parser.add_argument('--vcftools', type=Path)
    args = parser.parse_args()
    spec = json.loads(args.specification.read_text())
    admission = json.loads(args.admission.read_text())
    if args.command == 'verify-admission':
        verify_admission(args.inputs, spec, admission)
        return
    if args.output is None or args.vcftools is None:
        parser.error('complete run requires --output and --vcftools')
    args.output.mkdir(parents=True, exist_ok=False)
    save(args.output / 'request.json', dict(argv=sys.argv, cwd=str(Path.cwd()),
         producer_sha256=inputs.sha256(Path(__file__)), inputs_sha256=inputs.sha256(Path(inputs.__file__))))
    started, before = time.perf_counter(), resource.getrusage(resource.RUSAGE_SELF)
    captured = []
    try:
        with warnings.catch_warnings(record=True) as captured:
            warnings.simplefilter('always')
            result = run(args.inputs, spec, admission, json.loads(args.runtime.read_text()), args.output, args.vcftools)
        save(args.output / 'result.json', result)
    except Exception:
        (args.output / 'failure.txt').write_text(traceback.format_exc())
        raise
    finally:
        save(args.output / 'warnings.json', [dict(category=warning.category.__name__, message=str(warning.message),
                                                 filename=warning.filename, lineno=warning.lineno) for warning in captured])
        after = resource.getrusage(resource.RUSAGE_SELF)
        save(args.output / 'observed-process.json', dict(elapsed_seconds=time.perf_counter() - started,
             user_seconds=after.ru_utime - before.ru_utime, system_seconds=after.ru_stime - before.ru_stime,
             maxrss_kib=after.ru_maxrss, profile='descriptive inner observation; primary future timer is external fresh-process parse-to-export wall/CPU/RSS; no cold-I/O guarantee'))
        save(args.output / 'output-manifest.json', {str(path.relative_to(args.output)): dict(size=path.stat().st_size, sha256=inputs.sha256(path))
                                                   for path in sorted(args.output.rglob('*')) if path.is_file()})


if __name__ == '__main__':
    main()
