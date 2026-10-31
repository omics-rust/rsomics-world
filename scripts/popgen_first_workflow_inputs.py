#!/usr/bin/env python3
"""Prepare the connected popgen casebook and admit original VCF inputs."""

import argparse
from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path
import re
import struct
from typing import Iterator, NamedTuple


MASK_CODES = frozenset(b'PNLHZQ0')
BASES = frozenset('ACGT')


class Record(NamedTuple):
    ordinal: int
    chrom: str
    pos: int
    ref: str
    alts: list[str]
    passed: bool
    aa: str
    gt: list[list[int]]
    columns: list[str]


def save(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + '\n')


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1048576), b''):
            digest.update(block)
    return digest.hexdigest()


def text_stream(path: Path):
    return gzip.open(path, 'rt', encoding='utf-8', newline='') if path.suffix == '.gz' else path.open(encoding='utf-8', newline='')


def input_paths(root: Path, spec: dict) -> dict[str, Path]:
    resolved = root.resolve(strict=True)
    paths = {name: resolved / entry['filename'] for name, entry in spec['files'].items()}
    if any(not path.resolve(strict=True).is_relative_to(resolved) for path in paths.values()):
        raise ValueError('input path escapes admitted root')
    return paths


def population_panel(path: Path, populations: dict[str, int]) -> dict[str, str]:
    selected = {}
    seen = set()
    with text_stream(path) as stream:
        if stream.readline().split() != ['sample', 'pop', 'super_pop', 'gender']:
            raise ValueError('population panel schema')
        for line in stream:
            fields = line.split()
            if len(fields) != 4 or fields[0] in seen:
                raise ValueError('population panel identity/cardinality')
            seen.add(fields[0])
            if fields[1] in populations:
                selected[fields[0]] = fields[1]
    if Counter(selected.values()) != populations:
        raise ValueError('population panel membership differs')
    return selected


def load_masks(paths: dict[str, Path], spec: dict) -> dict[str, bytearray]:
    lengths = {}
    with text_stream(paths['reference_fai']) as stream:
        for line in stream:
            fields = line.split('\t')
            if len(fields) != 5 or fields[0] in lengths:
                raise ValueError('reference FAI schema/identity')
            lengths[fields[0]] = int(fields[1])
    result = {}
    for chrom, contig in spec['contigs'].items():
        if lengths.get(chrom) != contig['length']:
            raise ValueError('reference contig length differs: ' + chrom)
        with text_stream(paths[contig['mask_file']]) as stream:
            header = stream.readline().rstrip()
            if not header.startswith('>') or header[1:].split()[0] not in contig['mask_header_aliases']:
                raise ValueError('mask FASTA contig differs: ' + chrom)
            sequence = bytearray()
            for line in stream:
                if line.startswith('>'):
                    raise ValueError('mask FASTA contains an extra contig')
                sequence.extend(line.strip().encode('ascii'))
        if len(sequence) != contig['length'] or set(sequence) - MASK_CODES:
            raise ValueError('mask length/code differs: ' + chrom)
        result[chrom] = sequence
    return result


def vcf_header(path: Path) -> tuple[list[str], list[str]]:
    meta = []
    with text_stream(path) as stream:
        for line in stream:
            if line.startswith('##'):
                meta.append(line.rstrip('\r\n'))
            elif line.startswith('#CHROM\t'):
                fields = line.rstrip('\r\n').split('\t')
                if fields[:9] != ['#CHROM', 'POS', 'ID', 'REF', 'ALT', 'QUAL', 'FILTER', 'INFO', 'FORMAT']:
                    raise ValueError('VCF header schema')
                samples = fields[9:]
                if len(samples) != len(set(samples)):
                    raise ValueError('duplicate VCF sample identity')
                if not any(row.startswith('##FORMAT=<ID=GT,Number=1,Type=String,') for row in meta):
                    raise ValueError('GT schema must be Number=1 Type=String')
                if not any(row.startswith('##INFO=<ID=AA,Number=1,Type=String,') for row in meta):
                    raise ValueError('AA schema must be Number=1 Type=String')
                return meta, samples
            else:
                raise ValueError('VCF header missing or malformed')
    raise ValueError('VCF sample header missing')


def genotype(value: str, alt_count: int) -> list[int]:
    if not re.fullmatch(r'(?:[0-9]+|\.)[/|](?:[0-9]+|\.)', value):
        raise ValueError('GT is not diploid: ' + value)
    alleles = re.split(r'[/|]', value)
    if (alleles[0] == '.') != (alleles[1] == '.'):
        raise ValueError('partial-missing GT: ' + value)
    result = [-1 if allele == '.' else int(allele) for allele in alleles]
    if any(allele > alt_count for allele in result):
        raise ValueError('GT allele index exceeds ALT: ' + value)
    return result


def records(path: Path, selected_indices: list[int], sample_count: int,
            contigs: dict) -> Iterator[Record]:
    rank = {chrom: index for index, chrom in enumerate(contigs)}
    previous = (-1, 0)
    ordinal = 0
    with text_stream(path) as stream:
        for line in stream:
            if line.startswith('#'):
                continue
            fields = line.rstrip('\r\n').split('\t')
            if len(fields) != sample_count + 9:
                raise ValueError('VCF record cardinality')
            chrom, pos = fields[0], int(fields[1])
            if chrom not in contigs or not 1 <= pos <= contigs[chrom]['length']:
                raise ValueError('VCF coordinate outside admitted contig')
            coordinate = (rank[chrom], pos)
            if coordinate < previous:
                raise ValueError('VCF contig/position order differs')
            previous = coordinate
            alts = [] if fields[4] == '.' else fields[4].split(',')
            if not fields[3] or any(not allele for allele in alts):
                raise ValueError('empty VCF allele')
            formats = fields[8].split(':')
            if len(formats) != len(set(formats)) or 'GT' not in formats:
                raise ValueError('VCF FORMAT GT missing/duplicated')
            index = formats.index('GT')
            calls = []
            for sample in selected_indices:
                values = fields[9 + sample].split(':')
                if len(values) > len(formats) or index >= len(values):
                    raise ValueError('VCF FORMAT/GT cardinality')
                calls.append(genotype(values[index], len(alts)))
            ancestral = [entry[3:] for entry in fields[7].split(';') if entry.startswith('AA=')]
            if len(ancestral) > 1:
                raise ValueError('duplicate INFO/AA')
            yield Record(ordinal, chrom, pos, fields[3], alts, fields[6] == 'PASS',
                         ancestral[0] if ancestral else '', calls, fields)
            ordinal += 1


def record_identity(digest, ordinal: int, chrom: str, pos: int, ref: str,
                    alts: list[str], passed: bool, aa: str) -> None:
    row = [ordinal, chrom, pos, ref, alts, passed, aa]
    digest.update((json.dumps(row, separators=(',', ':'), ensure_ascii=True) + '\n').encode('ascii'))


def gt_identity(digest, calls: list[list[int]]) -> None:
    values = [allele for call in calls for allele in call]
    digest.update(struct.pack('<' + 'h' * len(values), *values))


def exclusion(record: Record, masks: dict[str, bytearray]) -> str:
    if not record.passed:
        return 'not_pass'
    if record.ref not in BASES or not record.alts or any(allele not in BASES for allele in record.alts):
        return 'not_literal_snv'
    if masks[record.chrom][record.pos - 1] != ord('P'):
        return 'inaccessible'
    return 'retained'


def admit(root: Path, spec: dict) -> dict:
    paths = input_paths(root, spec)
    panel = population_panel(paths['panel'], spec['populations'])
    meta, all_samples = vcf_header(paths['vcf'])
    if not set(panel).issubset(all_samples):
        raise ValueError('panel sample absent from VCF')
    ids = [sample for sample in all_samples if sample in panel]
    indices = [all_samples.index(sample) for sample in ids]
    masks = load_masks(paths, spec)
    digest = hashlib.sha256()
    identities_digest = hashlib.sha256()
    counts = Counter(dict(retained=0, not_pass=0, not_literal_snv=0, inaccessible=0))
    contig_rows = Counter()
    total = maximum = 0
    for record in records(paths['vcf'], indices, len(all_samples), spec['contigs']):
        maximum = max(maximum, len(record.alts))
        if maximum > 32767:
            raise ValueError('GT int16 profile cannot represent ALT width')
        gt_identity(digest, record.gt)
        record_identity(identities_digest, *record[:7])
        counts[exclusion(record, masks)] += 1
        contig_rows[record.chrom] += 1
        total += 1
    if total == 0 or set(contig_rows) != set(spec['contigs']):
        raise ValueError('complete input contig records absent')
    identities = {name: dict(filename=spec['files'][name]['filename'], size=path.stat().st_size,
                             sha256=sha256(path)) for name, path in paths.items()}
    for name, identity in identities.items():
        expected = spec['files'][name]
        if expected.get('size') is not None and expected['size'] != identity['size']:
            raise ValueError('provider input size differs: ' + name)
        if expected.get('sha256') is not None and expected['sha256'] != identity['sha256']:
            raise ValueError('provider input SHA256 differs: ' + name)
    return dict(schema=1, profile=spec['profile'], files=identities,
                specification_sha256=hashlib.sha256(json.dumps(spec, sort_keys=True, separators=(',', ':')).encode()).hexdigest(),
                sample_ids=ids, original_samples=len(all_samples), sample_indices=indices,
                population_order=spec['population_order'],
                population_indices={pop: [i for i, sample in enumerate(ids) if panel[sample] == pop]
                                    for pop in spec['population_order']},
                selected_gt_stream_sha256=digest.hexdigest(), max_alt_number=max(1, maximum),
                record_stream_sha256=identities_digest.hexdigest(),
                gt_shape=[total, len(ids), 2], filter_counts=dict(counts),
                gt_identity_profile='all original rows; selected sample order; signed int16 little-endian row/sample/ploidy bytes; phase omitted; original bytes retained',
                aa_headers=[line for line in meta if line.startswith('##INFO=<ID=AA,')],
                contigs={chrom: dict(length=len(mask), rows=contig_rows[chrom],
                                     accessible_bp=mask.count(ord('P')),
                                     mask_codes={chr(code): count for code, count in sorted(Counter(mask).items())})
                         for chrom, mask in masks.items()},
                admission_io='full schema/GT scan and file hashing outside both future timers; timed source/native still reread original VCF')


def prepare_legacy(root: Path, spec: dict, admission: dict, destination: Path) -> dict:
    paths = input_paths(root, spec)
    meta, samples = vcf_header(paths['vcf'])
    masks = load_masks(paths, spec)
    selected = admission['sample_indices']
    count = 0
    with (destination / 'eligible.vcf').open('x') as stream:
        stream.write('\n'.join(meta) + '\n')
        stream.write('\t'.join(['#CHROM', 'POS', 'ID', 'REF', 'ALT', 'QUAL', 'FILTER', 'INFO', 'FORMAT',
                                *admission['sample_ids']]) + '\n')
        for record in records(paths['vcf'], selected, len(samples), spec['contigs']):
            if exclusion(record, masks) == 'retained' and len(record.alts) == 1 and all(min(gt) >= 0 for gt in record.gt):
                stream.write('\t'.join(record.columns[:9] + [record.columns[9 + i] for i in selected]) + '\n')
                count += 1
    for pop, indices in admission['population_indices'].items():
        (destination / (pop + '.txt')).write_text(''.join(admission['sample_ids'][i] + '\n' for i in indices))
    return dict(root=str(destination.resolve()), records=count,
                input_gt_stream_sha256=admission['selected_gt_stream_sha256'],
                original_record_profile='PASS P-accessible literal biallelic SNV; both populations complete; original fields and GT preserved',
                files={path.name: dict(size=path.stat().st_size, sha256=sha256(path))
                       for path in destination.iterdir() if path.is_file()})


def casebook(destination: Path) -> dict:
    destination.mkdir(parents=True, exist_ok=False)
    meta = ['##fileformat=VCFv4.2', '##contig=<ID=a,length=31>', '##contig=<ID=b,length=23>',
            '##FILTER=<ID=PASS,Description="Passed">', '##FILTER=<ID=q10,Description="Low quality">',
            '##INFO=<ID=AA,Number=1,Type=String,Description="Inferred ancestral allele">',
            '##FORMAT=<ID=GT,Number=1,Type=String,Description="Genotype">',
            '##FORMAT=<ID=DP,Number=1,Type=Integer,Description="Depth">']
    rows = [
        ('a', 2, 'A', 'T', 'PASS', 'A|||', ['0/0', '1/1', '0/0', '1/1']),
        ('a', 4, 'A', 'T', 'PASS', 't|||', ['0/1', '0/0', '1/1', '0/1']),
        ('a', 7, 'A', 'C,G', 'PASS', 'G|||', ['0/2', '1/2', '0/1', '1/1']),
        ('a', 10, 'A', 'T', 'PASS', 'A|||', ['0/1'] * 4),
        ('a', 11, 'A', 'T', 'PASS', 'A|||', ['0/1'] * 4),
        ('a', 14, 'A', 'AT', 'PASS', 'A|||', ['0/1'] * 4),
        ('a', 15, 'A', 'T', 'q10', 'A|||', ['0/1'] * 4),
        ('a', 18, 'A', 'T', 'PASS', 'N|||', ['0/0', '0/1', '0/1', '1/1']),
        ('a', 19, 'A', 'T', 'PASS', 'G|||', ['0/1', '0/1', '1/1', '0/0']),
        ('a', 20, 'A', 'T', 'PASS', '.', ['./.', '0/1', '1/1', '0/0']),
        ('a', 22, 'A', 'T', 'PASS', 'A|||', ['./.'] * 4),
        ('a', 25, 'A', 'T', 'PASS', 'A|||', ['0/0'] * 4),
        ('a', 27, 'A', 'T', 'PASS', 'T|||', ['1/1'] * 4),
        ('a', 31, 'A', 'C,G,T,<DEL>', 'PASS', 'A|||', ['0/4'] * 4),
        ('b', 2, 'A', 'T', 'PASS', 'A|||', ['0/0', '0/1', '0/1', '1/1']),
        ('b', 4, 'A', 'T', 'PASS', 'T|||', ['0/1', '1/1', '1/1', '0/1']),
        ('b', 5, 'A', 'T', 'PASS', '.', ['1/1', '0/0', '1/1', '0/0']),
        ('b', 9, 'A', 'T', 'PASS', 'A|||', ['0/1', '0/1', '0/0', '1/1'])]
    header = '#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tY1\tC1\tY2\tC2'
    data = ['\t'.join(map(str, [chrom, pos, 'v' + str(i), ref, alt, '.', filt,
                                  'AA=' + aa, 'GT:DP', *[gt + ':10' for gt in calls]]))
            for i, (chrom, pos, ref, alt, filt, aa, calls) in enumerate(rows)]
    (destination / 'casebook.vcf').write_text('\n'.join(meta + [header] + data) + '\n')
    (destination / 'panel.tsv').write_text('sample\tpop\tsuper_pop\tgender\nY1\tYRI\tAFR\tmale\nY2\tYRI\tAFR\tfemale\nC1\tCEU\tEUR\tmale\nC2\tCEU\tEUR\tfemale\n')
    (destination / 'reference.fai').write_text('a\t31\t0\t31\t32\nb\t23\t0\t23\t24\n')
    (destination / 'a.mask.fa').write_text('>a\n' + 'P' * 9 + 'QQ' + 'P' * 20 + '\n')
    (destination / 'b.mask.fa').write_text('>b\n' + 'P' * 10 + 'Q' * 13 + '\n')
    spec = dict(schema=1, profile='engineering-casebook-not-real-performance', populations={'YRI': 2, 'CEU': 2},
                population_order=['YRI', 'CEU'],
                window_bp=10, jackknife_variants=3,
                files={name: dict(filename=filename) for name, filename in
                       {'vcf': 'casebook.vcf', 'panel': 'panel.tsv', 'reference_fai': 'reference.fai',
                        'mask_a': 'a.mask.fa', 'mask_b': 'b.mask.fa'}.items()},
                contigs={name: dict(length=length, mask_file='mask_' + name, mask_header_aliases=[name])
                         for name, length in [('a', 31), ('b', 23)]})
    spec['independent_truth'] = {
        'a': {'windows': [[1, 10], [11, 20], [21, 31]],
              'folded': {'YRI': [3, 3, 0], 'CEU': [3, 4, 0]},
              'unfolded': {'YRI': [3, 1, 0, 0, 0], 'CEU': [2, 0, 0, 1, 1]},
              'joint_folded': [[3, 0, 0], [0, 3, 0], [0, 0, 0]],
              'joint_unfolded_nonzero': [[0, 0, 2], [0, 4, 1], [1, 3, 1]],
              'sites': {'2': {'YRI_mpd': 0.0, 'CEU_mpd': 0.0, 'dxy_mpd': 1.0, 'hudson_num': 1.0, 'hudson_den': 1.0},
                        '4': {'YRI_mpd': 0.5, 'CEU_mpd': 0.5, 'dxy_mpd': 0.625, 'hudson_num': 0.125, 'hudson_den': 0.625}}},
        'b': {'windows': [[1, 10], [11, 20], [21, 23]],
              'folded': {'YRI': [1, 3, 0], 'CEU': [1, 3, 0]},
              'unfolded': {'YRI': [0, 3, 0, 0, 0], 'CEU': [0, 1, 0, 2, 0]},
              'joint_folded': [[1, 0, 0], [0, 3, 0], [0, 0, 0]],
              'joint_unfolded_nonzero': [[1, 1, 1], [1, 3, 2]], 'sites': {}}}
    save(destination / 'specification.json', spec)
    return spec


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    engineering = commands.add_parser('casebook')
    engineering.add_argument('output', type=Path)
    admission = commands.add_parser('admit')
    admission.add_argument('root', type=Path)
    admission.add_argument('specification', type=Path)
    admission.add_argument('output', type=Path)
    admission.add_argument('--legacy-output', type=Path, required=True)
    args = parser.parse_args()
    if args.command == 'casebook':
        casebook(args.output)
    else:
        spec = json.loads(args.specification.read_text())
        admitted = admit(args.root, spec)
        args.legacy_output.mkdir(parents=True, exist_ok=False)
        admitted['legacy'] = prepare_legacy(args.root, spec, admitted, args.legacy_output)
        save(args.output, admitted)


if __name__ == '__main__':
    main()
