import argparse
import csv
import gzip
import hashlib
import json
import math
from pathlib import Path

ABS = 2.0 ** -44
REL = 2.0 ** -40
ADMISSION_KEYS = ('schema', 'original_samples', 'sample_ids', 'sample_indices', 'population_order',
    'population_indices', 'max_alt_number', 'gt_shape', 'filter_counts', 'selected_gt_stream_sha256',
    'record_stream_sha256', 'contigs')
NATIVE_ADMISSION_KEYS = set(ADMISSION_KEYS) | {'files', 'legacy'}
SCIENCE_NAMES = ('sites.tsv.gz', 'windows.tsv.gz', 'region.json', 'sfs.json',
    'fst-jackknife.json', 'jackknife-membership.tsv.gz')
LEGACY_NAMES = ('site-pi.sites.pi', 'window-pi.windowed.pi', 'tajima-d.Tajima.D',
    'weir-fst.weir.fst', 'window-weir-fst.windowed.weir.fst')
PRODUCER_FILES = {'runtime.json', 'request.json', 'run-profile.json', 'output-manifest.json', 'result.json',
    'input-admission.json', 'filter-counts.json', 'legacy-provenance.json'}
POLICY = dict(canonical_abs_floor=ABS, canonical_relative=REL,
    canonical_rule='abs(native-source)<=max(2^-44,2^-40*abs(source))',
    integer_string_boolean_identity='exact', nonfinite='NaN class and signed Inf exact; NaN/zero signs portable',
    legacy='parsed source-format finite decimal exact', native_1_4='decompressed scientific payload exact',
    producer_exceptions=['request argv/cwd/root paths', 'actual producer/build/runtime identities',
        'worker/timing/resource/output byte observations', 'input path/basename not bytes/hash',
        'virtual legacy root/materialized=false', 'actual per-producer output carrier hashes/size',
        'oracle-only Python runtime lock/import/warnings and real subprocess diagnostics'])

def digest(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()

def read(path):
    assert path.is_file() and not path.is_symlink(), path
    return json.loads(path.read_bytes())

def classify(value):
    value = float(value)
    return 'nan' if math.isnan(value) else '+inf' if value == math.inf else '-inf' if value == -math.inf else 'finite'

class Comparison:
    def __init__(self):
        self.cells = 0
        self.max_abs = 0.0
        self.max_scaled = 0.0

    def numeric(self, got, expected, where, legacy=False):
        self.cells += 1
        assert classify(got) == classify(expected), (where, got, expected, 'nonfinite class')
        if classify(expected) != 'finite':
            return
        got, expected = float(got), float(expected)
        error = abs(got - expected)
        bound = 0.0 if legacy else max(ABS, REL * abs(expected))
        self.max_abs = max(self.max_abs, error)
        if bound:
            self.max_scaled = max(self.max_scaled, error / bound)
        assert error <= bound, (where, got, expected, error, bound)

    def value(self, got, expected, where):
        if isinstance(expected, float) or (isinstance(expected, str) and expected in ('NaN', '+Inf', '-Inf')):
            self.numeric(got, expected, where)
        elif isinstance(expected, dict):
            assert isinstance(got, dict) and set(got) == set(expected), (where, 'object keys')
            for key, value in expected.items():
                self.value(got[key], value, where + '/' + key)
        elif isinstance(expected, list):
            assert isinstance(got, list) and len(got) == len(expected), (where, 'array length')
            for index, value in enumerate(expected):
                self.value(got[index], value, where + '/' + str(index))
        else:
            self.cells += 1
            assert type(got) is type(expected) and got == expected, (where, got, expected)

    def table(self, got_path, expected_path, name, legacy=False):
        def stream(path):
            return gzip.open(path, 'rt', newline='') if name.endswith('.gz') else path.open(newline='')
        with stream(got_path) as got, stream(expected_path) as expected:
            left, right = csv.reader(got, delimiter='\t'), csv.reader(expected, delimiter='\t')
            header = next(right); assert next(left) == header, (name, 'header')
            count = 0
            for row in right:
                actual = next(left, None)
                assert actual is not None and len(actual) == len(row) == len(header), (name, count, 'row width/count')
                for column, a, b in zip(header, actual, row):
                    where = f'{name}/{count}/{column}'
                    if name.endswith('sites.tsv.gz'):
                        if column in ('alts',) or column.endswith('_AC') or column in ('wc_a', 'wc_b', 'wc_c'):
                            self.value(json.loads(a), json.loads(b), where)
                        elif column.endswith('_mpd') or column in ('hudson_num', 'hudson_den'):
                            self.numeric(a, b, where)
                        else:
                            assert a == b, (where, a, b); self.cells += 1
                    elif name.endswith('windows.tsv.gz'):
                        if column in ('start1', 'stop1') or column.endswith(('_records', '_accessible_bp')):
                            assert a == b, (where, a, b); self.cells += 1
                        else:
                            self.numeric(a, b, where)
                    elif name.endswith('jackknife-membership.tsv.gz'):
                        assert a == b, (where, a, b); self.cells += 1
                    elif legacy:
                        numeric = column in ('PI', 'TajimaD', 'WEIR_AND_COCKERHAM_FST', 'WEIGHTED_FST', 'MEAN_FST')
                        if numeric:
                            self.numeric(a, b, where, legacy=True)
                        else:
                            assert a == b, (where, a, b); self.cells += 1
                count += 1
            assert next(left, None) is None, (name, 'extra row')
            return count

def compare(packet, prefix, native, twin=None):
    source_map = read(packet / 'tar-verification.json')['files']
    source = lambda name: packet / source_map[prefix + '/' + name]['extracted_path']
    source_json = lambda name: read(source(name))
    source_manifest = source_json('output-manifest.json')
    for name, expected in source_manifest.items():
        entry = source_map[prefix + '/' + name]
        assert entry['bytes'] == expected['size'] and entry['sha256'] == expected['sha256']
    manifest = read(native / 'output-manifest.json')
    for name, expected in manifest.items():
        path = native / name
        assert path.is_file() and not path.is_symlink() and path.stat().st_size == expected['size']
        assert digest(path) == expected['sha256'], name
    actual = {path.relative_to(native).as_posix() for path in native.rglob('*') if path.is_file()}
    assert actual == set(manifest) | {'output-manifest.json'}, 'native manifest closure'
    admission = source_json('input-admission.json'); got_admission = read(native / 'input-admission.json')
    assert set(got_admission) == NATIVE_ADMISSION_KEYS
    comparison = Comparison()
    for key in ADMISSION_KEYS:
        comparison.value(got_admission[key], admission[key], 'admission/' + key)
    identities = lambda values: sorted((value['sha256'], value.get('size', value.get('bytes'))) for value in values.values())
    assert identities(got_admission['files']) == identities(admission['files']), 'input body identities'
    got_legacy = got_admission['legacy']; expected_legacy = admission['legacy']
    assert set(got_legacy) == {'files', 'records', 'input_gt_stream_sha256', 'original_record_profile', 'materialized'}
    assert got_legacy['materialized'] is False
    for key in ('files', 'records', 'input_gt_stream_sha256', 'original_record_profile'):
        comparison.value(got_legacy[key], expected_legacy[key], 'legacy/' + key)
    comparison.value(read(native / 'filter-counts.json'), source_json('filter-counts.json'), 'filters')
    result = source_json('result.json'); got_result = read(native / 'result.json')
    assert set(got_result) == {'contigs', 'selected_gt_stream_sha256', 'record_stream_sha256'}
    legacy_provenance = read(native / 'legacy-provenance.json')
    assert legacy_provenance == dict(backend='native-rust', upstream='vcftools-0.1.17', derivative=got_legacy,
        operations=['site-pi', 'window-pi', 'tajima-d', 'weir-fst', 'window-weir-fst'])
    runtime = read(native / 'runtime.json'); request = read(native / 'request.json')
    assert runtime['implementation'] == 'native-rust' and runtime['product'] == 'rsomics-popgen'
    assert isinstance(runtime['threads'], int) and runtime['threads'] > 0
    assert set(request) == {'argv', 'cwd'} and isinstance(request['argv'], list) and isinstance(request['cwd'], str)
    profile = read(native / 'run-profile.json')
    source_spec = packet / source_map['casebook/specification.json' if prefix == 'casebook-source' else 'controller/assets/popgen-first-input-20261101.json']['extracted_path']
    spec = read(source_spec)
    assert profile == dict(population_order=spec['population_order'], window_bp=spec['window_bp'],
        jackknife_variants=spec['jackknife_variants'], legacy_vcftools=True,
        canonical='scikit-allel-1.3.13-neutral', gt='complete-diploid-or-fully-missing',
        sites='PASS-P-accessible-literal-uppercase-SNV', aa='unique-nucleotide-before-pipe-no-fallback')
    for key in ('selected_gt_stream_sha256', 'record_stream_sha256'):
        assert got_result[key] == result[key]
    assert len(got_result['contigs']) == len(result['contigs'])
    counts = {}; expected_population = set(PRODUCER_FILES)
    for got_contig, contig in zip(got_result['contigs'], result['contigs']):
        assert set(got_contig) == set(contig)
        assert got_contig['output_files'] == {name: manifest[contig['chrom'] + '/' + name] for name in SCIENCE_NAMES}
        for key in ('chrom', 'retained_rows', 'window_rows', 'allele_counts_sha256', 'allele_counts_identity'):
            comparison.value(got_contig[key], contig[key], 'result/' + contig['chrom'] + '/' + key)
        for suffix in SCIENCE_NAMES:
            name = contig['chrom'] + '/' + suffix; expected_population.add(name)
            if suffix.endswith('.json'):
                comparison.value(read(native / name), source_json(name), name)
            else:
                counts[name] = comparison.table(native / name, source(name), name)
            if twin:
                first = gzip.open(native / name, 'rb').read() if name.endswith('.gz') else (native / name).read_bytes()
                other = gzip.open(twin / name, 'rb').read() if name.endswith('.gz') else (twin / name).read_bytes()
                assert first == other, ('native1/4', name)
    for suffix in LEGACY_NAMES:
        name = 'vcftools/' + suffix; expected_population.add(name)
        counts[name] = comparison.table(native / name, source(name), name, legacy=True)
        if twin:
            assert (native / name).read_bytes() == (twin / name).read_bytes(), ('native1/4', name)
    assert actual == expected_population, ('native file population', sorted(actual ^ expected_population))
    if twin:
        for name in ('input-admission.json', 'run-profile.json', 'filter-counts.json', 'legacy-provenance.json'):
            assert read(native / name) == read(twin / name), ('native1/4 metadata', name)
    return dict(status='passed', policy=POLICY, source_prefix=prefix, native=str(native),
        native_twin=str(twin) if twin else None, table_rows=counts, compared_cells=comparison.cells,
        max_finite_absolute_difference=comparison.max_abs, max_canonical_budget_fraction=comparison.max_scaled)

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--packet', type=Path, required=True)
    parser.add_argument('--source-prefix', choices=['casebook-source', 'real-source'], required=True)
    parser.add_argument('--native', type=Path, required=True)
    parser.add_argument('--twin', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    report = compare(args.packet, args.source_prefix, args.native, args.twin)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps(report))
