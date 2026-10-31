"""Deterministic compact engineering inputs, independent of PLINK and pgenlib."""

from fractions import Fraction
import hashlib
import json
from pathlib import Path


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write_json(path, value):
    Path(path).write_text(json.dumps(value, sort_keys=True, indent=2, allow_nan=False) + '\n')


def model():
    samples = [dict(fid='0', iid=f'S{i:04d}', sid='0',
                    pat='0' if i < 64 else 'S0000', mat='0' if i < 64 else 'S0001',
                    sex=0 if i in (7, 69) else 1 + i % 2) for i in range(72)]
    patterns = {'x': [0, 1, 1, 2], 'y': [1, 0, 2, 1], 'z': [0, 1, 2, 1],
                'copy': [0, 1, 1, 2], 'opposite': [2, 1, 1, 0]}
    baseline = []
    for i in range(64):
        name = list(patterns)[i] if i < 5 else f'v{i:02d}'
        pattern = patterns.get(name, [0, 1, 1, 2] if i % 3 or 5 <= i <= 7 else [0, 0, 1, 1])
        gt = [pattern[s % 4] if s < 64 else 2 for s in range(72)]
        if i == 10:
            gt = [None] * 72
        elif i == 11:
            gt = [0] * 72
        elif i == 12:
            gt = [1] * 72
        elif i == 13:
            gt = [1 if s == 0 else 0 for s in range(72)]
        elif i in (14, 15):
            for s in range(8 if i == 14 else 16):
                gt[s] = None
        if 20 <= i < 40:
            gt[63] = None
        chrom = '1' if i < 40 else '2'
        pos = (600100 + i) if i < 5 else (1100099 + i - 5 if i < 8 else 1 if i == 8 else 600000 + i * 1000)
        baseline.append(dict(chrom=chrom, pos=pos, id=name, ref='A', alt=['G'], gt=gt))
    baseline.sort(key=lambda row: (int(row['chrom']), row['pos']))
    rich = []
    for i in range(24):
        chrom = '1' if i < 16 else 'X' if i < 20 else 'Y' if i < 22 else 'MT'
        pos = [2699520, 2699521, 154931043, 154931044][i - 16] if 16 <= i < 20 else 1000 + i
        alt = ['C', 'G', 'T'] if i == 2 else ['G']
        gt = []
        for s in range(72):
            if i == 2:
                call = ['0/2', '1/2', '2/2', './.'][s % 4]
            elif chrom in ('Y', 'MT') or (chrom == 'X' and s % 2 == 0):
                call = ['0', '1', '0/1', '.'][s % 4]
            else:
                call = ['0|1', '1|0', '0/1', '0/0', '1/1', './.'][s % 6]
            gt.append(call)
        rich.append(dict(chrom=chrom, pos=pos, id=f'r{i:02d}', ref='A', alt=alt, gt=gt))
    ds_values = ['0', '0.09', '0.1', '0.1001', '0.5', '0.8999', '0.9', '1',
                 '1.1', '1.1001', '1.5', '1.9', '2', '.']
    dosage = []
    for i in range(24):
        dosage.append(dict(chrom='1' if i < 12 else '2', pos=100 + i * 1000,
                           id=f'd{i:02d}', ref='A', alt=['G'],
                           gt=['./.' if ds_values[(s + i) % len(ds_values)] == '.' else '0/0' for s in range(72)],
                           ds=[ds_values[(s + i) % len(ds_values)] for s in range(72)]))
    return dict(schema=1, samples=samples, baseline=baseline, rich=rich, dosage=dosage,
                seed=20261031, expected_origin='independent engineering truth, not source output')


def correlation_squared(left, right):
    pairs = [(Fraction(x), Fraction(y)) for x, y in zip(left, right) if x is not None and y is not None]
    n = len(pairs)
    sx, sy = sum(x for x, _ in pairs), sum(y for _, y in pairs)
    vx = n * sum(x*x for x, _ in pairs) - sx*sx
    vy = n * sum(y*y for _, y in pairs) - sy*sy
    return None if not vx or not vy else (n * sum(x*y for x, y in pairs) - sx*sy)**2 / (vx*vy)


def check_truth(truth):
    samples = truth['samples']
    keys = [(s['fid'], s['iid'], s['sid']) for s in samples]
    if len(samples) != 72 or len(set(keys)) != 72:
        raise ValueError('truth sample identity is not unique and complete')
    founders = [i for i, s in enumerate(samples) if s['pat'] == s['mat'] == '0']
    if len(founders) != 64 or len(truth['baseline']) != 64 or len(truth['rich']) != 24:
        raise ValueError('truth cohort shape/founder contract differs')
    rows = {v['id']: v for v in truth['baseline']}
    x = [rows['x']['gt'][s] for s in founders]
    counts = [x.count(g) for g in (0, 1, 2)]
    actual = {label: str(correlation_squared(x, [rows[name]['gt'][s] for s in founders]))
              for label, name in [('x_y', 'y'), ('x_z', 'z'), ('x_copy', 'copy')]}
    if counts != [16, 32, 16] or actual != {'x_y': '0', 'x_z': '1/4', 'x_copy': '1'}:
        raise ValueError('independent balanced-founder sentinel truth differs')
    for row in truth['baseline']:
        if len(row['gt']) != 72 or any(g not in (0, 1, 2, None) for g in row['gt']):
            raise ValueError('invalid baseline hardcall shape/value')
    return dict(samples=72, founders=64, baseline_variants=64, rich_variants=24,
                sentinel_genotypes=counts, founder_ld=actual,
                scope='balanced unfiltered 64-founder view; not arbitrary filtered F')


def write_vcf(path, rows, samples, dosage=None, halfcall=False, invalid=False):
    header = ['##fileformat=VCFv4.3', '##contig=<ID=1>', '##contig=<ID=2>',
              '##contig=<ID=X>', '##contig=<ID=Y>', '##contig=<ID=MT>',
              '##FORMAT=<ID=GT,Number=1,Type=String,Description="Genotype">']
    if dosage:
        header.append('##FORMAT=<ID=DS,Number=A,Type=Float,Description="Alternate dosage">')
    if dosage == 'HDS':
        header.append('##FORMAT=<ID=HDS,Number=.,Type=Float,Description="Haplotype dosages">')
    lines = header + ['#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\t' + '\t'.join(s['iid'] for s in samples)]
    for row_idx, row in enumerate(rows):
        calls = []
        for s, value in enumerate(row['gt']):
            gt = './.' if value is None else ('0/0', '0/1', '1/1')[value] if isinstance(value, int) else value
            if halfcall and row_idx == s == 0:
                gt = '0/.'
            if invalid and row_idx == s == 0:
                gt = '0/8'
            if dosage:
                ds = row['ds'][s]
                gt += ':' + ds
                if dosage == 'HDS':
                    if ds == '.':
                        gt += ':.,.'
                    else:
                        val = Fraction(ds)
                        a = min(val, Fraction(1))
                        gt += ':' + f'{float(a):.8f},{float(val-a):.8f}'
            calls.append(gt)
        lines.append('\t'.join([row['chrom'], str(row['pos']), row['id'], row['ref'], ','.join(row['alt']),
                                '.', 'PASS', '.', 'GT' + (':DS:HDS' if dosage == 'HDS' else ':DS' if dosage else '')] + calls))
    path.write_text('\n'.join(lines) + '\n')


def generate(output):
    output = Path(output)
    output.mkdir(parents=True, exist_ok=False)
    truth = model()
    checked = check_truth(truth)
    samples, baseline = truth['samples'], truth['baseline']
    write_json(output / 'truth.json', truth)
    write_json(output / 'scalar-truth.json', checked)
    psam = '#FID\tIID\tSID\tPAT\tMAT\tSEX\n' + ''.join(
        '\t'.join(str(s[k]) for k in ('fid', 'iid', 'sid', 'pat', 'mat', 'sex')) + '\n' for s in samples)
    (output / 'baseline.psam').write_text(psam)
    (output / 'sex.tsv').write_text('#FID\tIID\tSEX\n' + ''.join(f"0\t{s['iid']}\t{s['sex']}\n" for s in samples))
    (output / 'baseline.fam').write_text(''.join(f"0 {s['iid']} {s['pat']} {s['mat']} {s['sex']} -9\n" for s in samples))
    (output / 'baseline.bim').write_text(''.join(f"{v['chrom']} {v['id']} 0 {v['pos']} G A\n" for v in baseline))
    encoded = bytearray(b'\x6c\x1b\x01')
    for row in baseline:
        for start in range(0, 72, 4):
            byte = 0
            for offset, value in enumerate(row['gt'][start:start+4]):
                byte |= {0: 3, 1: 2, 2: 0, None: 1}[value] << (2 * offset)
            encoded.append(byte)
    (output / 'baseline.bed').write_bytes(encoded)
    write_vcf(output / 'baseline.vcf', baseline, samples)
    write_vcf(output / 'rich.vcf', truth['rich'], samples)
    phaseflip = (output / 'rich.vcf').read_text().replace('0|1', 'PHASESWAP').replace('1|0', '0|1').replace('PHASESWAP', '1|0')
    (output / 'phaseflip.vcf').write_text(phaseflip)
    write_vcf(output / 'ds.vcf', truth['dosage'], samples, 'DS')
    write_vcf(output / 'hds.vcf', truth['dosage'], samples, 'HDS')
    write_vcf(output / 'halfcall.vcf', baseline, samples, halfcall=True)
    write_vcf(output / 'invalid-allele.vcf', baseline, samples, invalid=True)
    multi_dosage = (output / 'rich.vcf').read_text().splitlines()
    converted = []
    for line in multi_dosage:
        if line.startswith('#CHROM'):
            converted.append('##FORMAT=<ID=DS,Number=A,Type=Float,Description="Alternate dosages">')
        if not line.startswith('#'):
            fields = line.split('\t')
            fields[8] = 'GT:DS'
            suffix = ':' + ','.join(['0.1'] * len(fields[4].split(',')))
            fields[9:] = [gt + suffix for gt in fields[9:]]
            line = '\t'.join(fields)
        converted.append(line)
    (output / 'unsupported-multiallelic-dosage.vcf').write_text('\n'.join(converted) + '\n')
    bad_cardinality = (output / 'ds.vcf').read_text().replace('0/0:0\t', '0/0:0,1\t', 1)
    (output / 'invalid-cardinality.vcf').write_text(bad_cardinality)
    ids = lambda subset: '#FID\tIID\tSID\n' + ''.join(f"0\t{s['iid']}\t0\n" for s in subset)
    (output / 'keep.id').write_text(ids(samples[:71]))
    (output / 'remove.id').write_text(ids([samples[70]]))
    (output / 'founders64.id').write_text(ids(samples[:64]))
    for n in (49, 64, 65, 72):
        (output / f'keep{n}.id').write_text(ids(samples[:n]))
    (output / 'selection.ids').write_text('\n'.join(v['id'] for v in baseline) + '\n')
    (output / 'exclusion.ids').write_text('v63\n')
    (output / 'ld-pair.ids').write_text('x\nz\n')
    write_json(output / 'manifest.json', dict(schema=1,
        origin='prepared synthetic engineering fixture; not oracle output',
        generator_sha256=digest(__file__), independent_truth=checked,
        files={p.name: digest(p) for p in sorted(output.iterdir()) if p.is_file()}))
    return truth
