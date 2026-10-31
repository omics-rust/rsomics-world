#!/usr/bin/env python3
"""Generate deterministic input-only SAM, molecule, transcript and peak witnesses."""

import argparse
import csv
import hashlib
import json
from pathlib import Path
import platform
import random


SEED = 20261031
CONTIGS = {'chrA': 10000000, 'chrB': 10000000}
SAMPLES = {'T1': 100000, 'T2': 100000, 'C1': 60000, 'C2': 60000}


def save_json(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + '\n')


def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1048576), b''):
            result.update(block)
    return result.hexdigest()


def annotation(root):
    gtf = []
    gff = ['##gff-version 3', '##sequence-region chrA 1 10000000',
           '##sequence-region chrB 1 10000000']
    for i in range(256):
        chrom = 'chrA' if i < 128 else 'chrB'
        local = i % 128
        start = 20000 + local * 60000 if local != 127 else 8497802
        end = start + 2198
        strand = '+' if i % 2 == 0 else '-'
        if local == 127:
            strand = '+' if chrom == 'chrA' else '-'
        gene = 'g%03d' % i
        txs = [gene + '.t1'] + ([gene + '.t2'] if i < 64 else [])
        def emit(rows, kind, left, right, attrs, phase='.'):
            rows.append('\t'.join(map(str, [chrom, 'peak-witness', kind, left + 1, right,
                                           '.', strand, phase, attrs])))
        emit(gtf, 'gene', start, end, 'gene_id "' + gene + '";')
        emit(gff, 'gene', start, end, 'ID=' + gene)
        for tx in txs:
            attrs = 'gene_id "' + gene + '"; transcript_id "' + tx + '";'
            emit(gtf, 'transcript', start, end, attrs)
            emit(gff, 'mRNA', start, end, 'ID=' + tx + ';Parent=' + gene)
            for exon in range(3):
                left, right = start + exon * 1000, start + exon * 1000 + 198
                rank = exon + 1 if strand == '+' else 3 - exon
                emit(gtf, 'exon', left, right, attrs + ' exon_number "' + str(rank) + '";')
                if i % 8:
                    cds_left = left + (48 if exon == 0 else 0)
                    cds_right = right - (48 if exon == 2 else 0)
                    emit(gtf, 'CDS', cds_left, cds_right, attrs, '0')
                    emit(gff, 'CDS', cds_left, cds_right,
                         'ID=' + tx + '.c' + str(exon) + ';Parent=' + tx, '0')
        for exon in range(3):
            emit(gff, 'exon', start + exon * 1000, start + exon * 1000 + 198,
                 'ID=' + gene + '.e' + str(exon) + ';Parent=' + ','.join(txs))
    (root / 'annotation.gtf').write_text('\n'.join(gtf) + '\n')
    (root / 'annotation.gff3').write_text('\n'.join(gff) + '\n')


def diagnostic_rows(root):
    rows = [
        ['chrA', 8700100, 8700125, 'duplicate-display', 0, '+'],
        ['chrA', 8700100, 8700125, 'duplicate-display', 0, '+'],
        ['chrA', 8700000, 8700075, 'same-geometry-A', 0, '+'],
        ['chrA', 8700000, 8700075, 'same-geometry-B', 0, '+'],
    ]
    distances = ['', '', '', '']
    for distance in [299, 300, 301, 2999, 3000, 3001]:
        rows.append(['chrA', 8500000 + distance - 1, 8500000 + distance,
                     'downstream-plus-' + str(distance), 0, '+'])
        distances.append(distance)
        rows.append(['chrB', 8497802 - distance, 8497803 - distance,
                     'downstream-minus-' + str(distance), 0, '-'])
        distances.append(distance)
    while len(rows) < 48:
        i = len(rows)
        rows.append(['chrA', 8700000 + (i - 16) * 40, 8700000 + (i - 16) * 40 + 100,
                     'panel-' + str(i), 0, '+' if i % 2 == 0 else '-'])
        distances.append('')
    rows[-1] = ['chrB', 9900000, 9900100, 'reserved-zero', 0, '+']
    (root / 'diagnostic.bed').write_text(''.join('\t'.join(map(str, row)) + '\n' for row in rows))
    with (root / 'peak-rows.tsv').open('w', newline='') as stream:
        writer = csv.writer(stream, delimiter='\t', lineterminator='\n')
        writer.writerow(['row_key', 'row_ordinal', 'display_id', 'distance_from_feature_end'])
        for i, (row, distance) in enumerate(zip(rows, distances)):
            writer.writerow(['diagnostic:%03d' % i, i, row[3], distance])
    (root / 'missing-contig-diagnostic.bed').write_text('missing\t0\t100\tmissing-contig\t0\t+\n')


def sam_record(name, flag, chrom, start, mate, tlen, mapq=60, cigar='75M'):
    return [name, flag, chrom, start + 1, mapq, cigar, '=', mate + 1, tlen,
            'A' * 75, 'I' * 75, 'RG:Z:' + name.split('-')[0]]


def molecules(sample):
    rng = random.Random(SEED + list(SAMPLES).index(sample))
    for i in range(SAMPLES[sample]):
        chrom = 'chrA' if i % 2 == 0 else 'chrB'
        length = rng.randrange(180, 261)
        if sample.startswith('T') and i < 40000:
            locus = (i // 2) % 200
            center = 100000 + locus * 40000
            start = center + rng.randrange(-90, 91) - length // 2
            kind = 'enriched'
        else:
            start = rng.randrange(1000, 8300000)
            kind = 'diffuse'
        right = start + length - 75
        name = sample + '-core-%06d' % i
        reads = [sam_record(name, 99, chrom, start, right, length),
                 sam_record(name, 147, chrom, right, start, -length)]
        yield name, kind, chrom, start, start + length, reads, (i // 400) % 2
    for i in range(22):
        start = 8700000 + i * 400
        length = 260
        right = start + length - 75
        name = sample + '-panel-%02d' % i
        quality = [19, 20, 21][i % 3]
        flags = [99, 147]
        kind, cigar = 'ordinary', '75M'
        if i == 3:
            flags, kind = [1123, 1171], 'duplicate'
        elif i == 4:
            flags, kind = [611, 659], 'qc-fail'
        elif i == 5:
            flags, kind = [97, 145], 'improper'
        elif i == 6:
            cigar, kind = '30M50N45M', 'split'
            length, right = 310, start + 235
        elif i == 21:
            flags, kind, length = [73, 133], 'orphan', 0
            right = start
        reads = [sam_record(name, flags[0], 'chrA', start, right, length, quality, cigar),
                 sam_record(name, flags[1], 'chrA', right, start, -length, quality)]
        if i == 21:
            reads[1][5] = '*'
        if i in [7, 8]:
            extra = 256 if i == 7 else 2048
            reads += [sam_record(name, 99 | extra, 'chrA', start, right, length, quality),
                      sam_record(name, 147 | extra, 'chrA', right, start, -length, quality)]
            kind = 'secondary' if i == 7 else 'supplementary'
        yield name, kind, 'chrA', start, start + (length or 75), reads, i % 2


def generate(output):
    root = Path(output)
    root.mkdir(parents=True, exist_ok=False)
    (root / 'genome.sizes').write_text('chrA\t10000000\nchrB\t10000000\n')
    annotation(root)
    diagnostic_rows(root)
    header = '@HD\tVN:1.6\tSO:unsorted\n@SQ\tSN:chrA\tLN:10000000\n@SQ\tSN:chrB\tLN:10000000\n'
    for sample in SAMPLES:
        with (root / (sample + '.pe.sam')).open('w') as pe, \
             (root / (sample + '.se.sam')).open('w') as se, \
             (root / (sample + '.fragments.bed')).open('w') as fragments, \
             (root / (sample + '.molecules.tsv')).open('w', newline='') as ledger:
            pe.write(header + '@RG\tID:' + sample + '\tSM:' + sample + '\n')
            se.write(header + '@RG\tID:' + sample + '\tSM:' + sample + '\n')
            writer = csv.writer(ledger, delimiter='\t', lineterminator='\n')
            writer.writerow(['sample', 'molecule_key', 'kind', 'chrom', 'start', 'end',
                             'alignment_records', 'se_projection_mate'])
            for name, kind, chrom, start, end, reads, mate in molecules(sample):
                for read in reads:
                    pe.write('\t'.join(map(str, read)) + '\n')
                projected = list(reads[mate])
                projected[1] &= 16 | 4 | 512 | 1024
                projected[6:9] = ['*', 0, 0]
                se.write('\t'.join(map(str, projected)) + '\n')
                writer.writerow([sample, name, kind, chrom, start, end, len(reads), mate + 1])
                if all(read[1] & 2 and not read[1] & (4 | 8 | 512 | 1024) for read in reads[:2]):
                    fragments.write('\t'.join(map(str, [chrom, start, end, name])) + '\n')
    routes = {
        'call': {'SE': [s + '.se.sam' for s in SAMPLES], 'PE': [s + '.pe.sam' for s in SAMPLES],
                 'hosted_conversion': 'samtools view -> coordinate sort -> index; retain original SAM/ledger'},
        'call-sparse': {'pooled_fragments': ['T1', 'T2'], 'control_fragments': ['C1', 'C2'],
                        'fragment_files': [s + '.fragments.bed' for s in SAMPLES],
                        'selection': 'one proper mapped primary same-contig pair/molecule; exclude duplicate or QC-fail in either mate; no MAPQ cutoff',
                        'derivation': 'LC_ALL=C sort -k1,1 -k2,2n -k3,3n -> genomecov -bg -g genome.sizes',
                        'selection_is_oracle_input_only': True},
        'annotate': {'gene_models': ['annotation.gtf', 'annotation.gff3'], 'peak_rows': 'peak-rows.tsv'},
        'count': {'samples': ['T1', 'T2'], 'peak_rows': 'peak-rows.tsv', 'unit': 'alignment'},
        'downstream_rows': {'sources': ['complete-M2-narrowPeak', 'complete-six-SEACR-BED', 'diagnostic.bed'],
                            'key': 'source-run/row-ordinal', 'deduplicate': False}}
    save_json(root / 'input-routes.json', routes)
    files = {str(path.relative_to(root)): {'sha256': digest(path), 'size': path.stat().st_size}
             for path in sorted(root.iterdir())}
    if sum(row['size'] for row in files.values()) >= 2**31:
        raise ValueError('input corpus exceeds the frozen 2GiB cap')
    manifest = {'schema': 1, 'seed': SEED, 'generator_sha256': digest(Path(__file__)),
                'generator_python': platform.python_version(),
                'contigs': CONTIGS, 'molecules': {s: n + 22 for s, n in SAMPLES.items()},
                'operations': ['call', 'call-sparse', 'annotate', 'count'], 'files': files,
                'status': 'input-only; no upstream outputs or BAM/index validity asserted'}
    save_json(root / 'input-manifest.json', manifest)
    return manifest


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    manifest = generate(args.output)
    print(json.dumps({'files': len(manifest['files']), 'molecules': manifest['molecules'],
                      'seed': manifest['seed'], 'status': manifest['status']}, sort_keys=True))

