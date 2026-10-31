import csv
import hashlib
import importlib.util
import itertools
import json
import os
from pathlib import Path
import re
import tempfile
import unittest


SCRIPT = Path(__file__).with_name('peak_first_workflow_inputs.py')


class PeakFirstInputsTest(unittest.TestCase):
    def test_shared_molecules_transcripts_and_row_identity_reach_all_four_routes(self):
        self.assertTrue(SCRIPT.is_file(), 'shared peak input generator is absent')
        spec = importlib.util.spec_from_file_location('peak_inputs', SCRIPT)
        generator = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(generator)
        with tempfile.TemporaryDirectory(prefix='peak-input-check-', dir=os.environ['TMPDIR']) as directory:
            root = Path(directory)
            first = generator.generate(root / 'first')
            second = generator.generate(root / 'second')
            self.assertEqual(first, second, 'fixed seed must reproduce every input hash')
            self.assertEqual(first['seed'], 20261031)
            self.assertEqual(first['contigs'], {'chrA': 10000000, 'chrB': 10000000})
            self.assertEqual(first['molecules'], {'T1': 100022, 'T2': 100022, 'C1': 60022, 'C2': 60022})
            self.assertEqual(first['operations'], ['call', 'call-sparse', 'annotate', 'count'])
            self.assertLess(sum(row['size'] for row in first['files'].values()), 2**31)
            for name, record in first['files'].items():
                data = (root / 'first' / name).read_bytes()
                self.assertEqual(hashlib.sha256(data).hexdigest(), record['sha256'])
                self.assertEqual(len(data), record['size'])
            pe_total = se_total = 0
            panel_flags = set()
            mapqs = set()
            for sample, core_count in [('T1', 100000), ('T2', 100000), ('C1', 60000), ('C2', 60000)]:
                ledger = (root / 'first' / (sample + '.molecules.tsv')).read_text().splitlines()
                self.assertEqual(len(ledger), core_count + 23)
                ledger_rows = iter(csv.DictReader(ledger, delimiter='\t'))
                with (root / 'first' / (sample + '.pe.sam')).open() as stream:
                    self.assertEqual(next(stream).strip(), '@HD\tVN:1.6\tSO:unsorted')
                    records = (line.rstrip().split('\t') for line in stream if not line.startswith('@'))
                    groups = 0
                    for name, group in itertools.groupby(records, key=lambda row: row[0]):
                        reads = list(group)
                        molecule = next(ledger_rows)
                        self.assertEqual((molecule['sample'], molecule['molecule_key'], int(molecule['alignment_records'])),
                                         (sample, name, len(reads)))
                        groups += 1
                        primary = [r for r in reads if not int(r[1]) & (256 | 2048)]
                        self.assertEqual(len(primary), 2)
                        self.assertEqual({int(r[1]) & (64 | 128) for r in primary}, {64, 128})
                        for read in reads:
                            pe_total += 1
                            self.assertEqual(len(read), 12)
                            flag, pos, quality, mate_pos, tlen = map(int, [read[1], read[3], read[4], read[7], read[8]])
                            panel_flags.add(flag)
                            mapqs.add(quality)
                            if not flag & 4:
                                cigar = re.findall(r'(\d+)([MIDNSHP=X])', read[5])
                                self.assertEqual(''.join(n + op for n, op in cigar), read[5])
                                self.assertEqual(sum(int(n) for n, op in cigar if op in 'MIS=X'), len(read[9]))
                                self.assertTrue(1 <= pos <= 10000000)
                            self.assertEqual(len(read[9]), len(read[10]))
                            self.assertEqual(read[6], '=')
                            other = primary[1] if read is primary[0] else primary[0]
                            if not flag & (256 | 2048):
                                self.assertEqual(mate_pos, int(other[3]))
                            if flag & 2 and not flag & (256 | 2048):
                                end = pos - 1 + sum(int(n) for n, op in re.findall(r'(\d+)([MIDNSHP=X])', read[5]) if op in 'MDN=X')
                                other_end = int(other[3]) - 1 + sum(int(n) for n, op in re.findall(r'(\d+)([MIDNSHP=X])', other[5]) if op in 'MDN=X')
                                span = max(end, other_end) - min(pos - 1, int(other[3]) - 1)
                                self.assertEqual(abs(tlen), span)
                        self.assertEqual(sum(int(r[8]) for r in primary), 0)
                    self.assertEqual(groups, core_count + 22)
                    self.assertIsNone(next(ledger_rows, None))
                with (root / 'first' / (sample + '.se.sam')).open() as stream:
                    strands = []
                    by_contig = {'chrA': [], 'chrB': []}
                    for line in stream:
                        if line.startswith('@'):
                            continue
                        read = line.rstrip().split('\t')
                        se_total += 1
                        self.assertEqual(int(read[1]) & (1 | 64 | 128), 0)
                        if '-core-' in read[0]:
                            strands.append(bool(int(read[1]) & 16))
                            by_contig[read[2]].append(bool(int(read[1]) & 16))
                    self.assertEqual(sum(strands), core_count // 2)
                    self.assertEqual({chrom: sum(values) for chrom, values in by_contig.items()},
                                     {'chrA': core_count // 4, 'chrB': core_count // 4})
                fragment_path = root / 'first' / (sample + '.fragments.bed')
                self.assertTrue(fragment_path.is_file(), 'SEACR must receive fragments from the same molecule family')
                fragments = fragment_path.read_text().splitlines()
                self.assertEqual(len(fragments), core_count + 18)
                self.assertEqual(len({line.split('\t')[3] for line in fragments}), core_count + 18)
            self.assertEqual((pe_total, se_total), (640192, 320088))
            self.assertTrue({19, 20, 21}.issubset(mapqs))
            self.assertTrue(any(flag & 1024 for flag in panel_flags))
            self.assertTrue(any(flag & 512 for flag in panel_flags))
            self.assertTrue(any(flag & 256 for flag in panel_flags))
            self.assertTrue(any(flag & 2048 for flag in panel_flags))
            gtf_edges = set()
            with (root / 'first' / 'annotation.gtf').open() as stream:
                for line in stream:
                    fields = line.rstrip().split('\t')
                    self.assertEqual(len(fields), 9)
                    self.assertTrue(1 <= int(fields[3]) <= int(fields[4]) <= 10000000)
                    if fields[2] == 'CDS':
                        self.assertEqual((int(fields[4]) - int(fields[3]) + 1) % 3, 0)
                        self.assertEqual(fields[7], '0')
                    if fields[2] == 'exon':
                        transcript = re.search(r'transcript_id "([^"]+)"', fields[8])[1]
                        gtf_edges.add((transcript, fields[0], fields[3], fields[4], fields[6]))
            gff_edges = set()
            genes = transcripts = shared = 0
            with (root / 'first' / 'annotation.gff3').open() as stream:
                for line in stream:
                    if line.startswith('#'):
                        continue
                    fields = line.rstrip().split('\t')
                    attrs = dict(part.split('=', 1) for part in fields[8].split(';'))
                    genes += fields[2] == 'gene'
                    transcripts += fields[2] == 'mRNA'
                    if fields[2] == 'exon':
                        parents = attrs['Parent'].split(',')
                        shared += len(parents) > 1
                        for transcript in parents:
                            gff_edges.add((transcript, fields[0], fields[3], fields[4], fields[6]))
            self.assertEqual((genes, transcripts, len(gtf_edges)), (256, 320, 960))
            self.assertGreater(shared, 0)
            self.assertEqual(gtf_edges, gff_edges)
            rows = (root / 'first' / 'diagnostic.bed').read_text().splitlines()
            self.assertEqual(len(rows), 48)
            self.assertEqual(rows[0], rows[1], 'exact duplicate BED rows must remain')
            self.assertEqual(rows[2].split('\t')[:3], rows[3].split('\t')[:3])
            self.assertNotEqual(rows[2].split('\t')[3], rows[3].split('\t')[3])
            with (root / 'first' / 'peak-rows.tsv').open() as stream:
                keys = list(csv.DictReader(stream, delimiter='\t'))
            self.assertEqual(len({r['row_key'] for r in keys}), 48)
            self.assertEqual([r['row_ordinal'] for r in keys], list(map(str, range(48))))
            self.assertEqual({int(r['distance_from_feature_end']) for r in keys if r['distance_from_feature_end']},
                             {299, 300, 301, 2999, 3000, 3001})
            route = json.loads((root / 'first' / 'input-routes.json').read_text())
            self.assertEqual(route['count']['samples'], ['T1', 'T2'])
            self.assertEqual(route['call-sparse']['pooled_fragments'], ['T1', 'T2'])
            self.assertEqual(route['annotate']['peak_rows'], 'peak-rows.tsv')
            self.assertNotIn('expected_outputs', route)


if __name__ == '__main__':
    unittest.main()
