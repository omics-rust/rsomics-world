import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))


class PopgenSourcePreparationTest(unittest.TestCase):
    def test_connected_casebook_admits_lossless_shapes_and_rejects_invalid_genotypes(self):
        producer = ROOT / 'scripts/popgen_first_workflow_oracle.py'
        self.assertTrue(producer.is_file(), 'connected popgen source producer is absent')
        import popgen_first_workflow_inputs as inputs
        spec = importlib.util.spec_from_file_location('popgen_oracle', producer)
        oracle = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(oracle)
        self.assertFalse({'numpy', 'allel', 'dask'}.intersection(sys.modules))
        with tempfile.TemporaryDirectory(prefix='popgen-preparation-', dir=os.environ['TMPDIR']) as directory:
            destination = Path(directory)
            case = inputs.casebook(destination / 'casebook')
            case = json.loads((destination / 'casebook/specification.json').read_text())
            admission = inputs.admit(destination / 'casebook', case)
            self.assertEqual(admission['sample_ids'], ['Y1', 'C1', 'Y2', 'C2'])
            self.assertEqual(admission['population_indices'], {'YRI': [0, 2], 'CEU': [1, 3]})
            self.assertEqual(admission['max_alt_number'], 4)
            self.assertEqual(admission['gt_shape'], [18, 4, 2])
            self.assertEqual(admission['contigs']['a']['mask_codes'], {'P': 29, 'Q': 2})
            self.assertEqual(admission['contigs']['b']['mask_codes'], {'P': 10, 'Q': 13})
            self.assertEqual(admission['filter_counts'],
                             {'retained': 13, 'not_pass': 1, 'not_literal_snv': 2, 'inaccessible': 2})
            self.assertEqual(oracle.read_options(admission)['alt_number'], 4)
            self.assertEqual(oracle.read_options(admission)['numbers'],
                             {'calldata/GT': 2, 'variants/AA': 1})
            self.assertEqual(oracle.read_options(admission)['types']['calldata/GT'], 'genotype/i2')
            self.assertEqual(oracle.resolve_ancestral('t|||', ['A', 'T']), (1, 'matched'))
            self.assertEqual(oracle.resolve_ancestral('A|||', ['A', 'T']), (0, 'matched'))
            self.assertEqual(oracle.resolve_ancestral('N|||', ['A', 'T']), (-1, 'non_nucleotide'))
            self.assertEqual(oracle.resolve_ancestral('G|||', ['A', 'T']), (-1, 'allele_mismatch'))
            self.assertEqual(oracle.resolve_ancestral('.', ['A', 'T']), (-1, 'missing'))
            self.assertEqual(oracle.resolve_ancestral('A|||', ['A', 'A']), (-1, 'ambiguous'))
            plan = oracle.legacy_commands(destination / 'legacy', '/runtime/bin/vcftools', 10)
            self.assertEqual([row['name'] for row in plan],
                             ['site-pi', 'window-pi', 'tajima-d', 'weir-fst', 'window-weir-fst'])
            self.assertTrue(all('--vcf' in row['argv'] for row in plan))
            self.assertIn('--fst-window-size', plan[-1]['argv'])
            original = (destination / 'casebook/casebook.vcf').read_text()
            for genotype in ['0/.', '0/1/1', '0', '0/9']:
                (destination / 'casebook/casebook.vcf').write_text(original.replace('0/1:10', genotype + ':10', 1))
                with self.assertRaisesRegex(ValueError, 'GT'):
                    inputs.admit(destination / 'casebook', case)
            (destination / 'casebook/casebook.vcf').write_text(original)
            reread = inputs.admit(destination / 'casebook', case)
            self.assertEqual(reread['selected_gt_stream_sha256'], admission['selected_gt_stream_sha256'])
            legacy = destination / 'legacy'
            legacy.mkdir()
            derivative = inputs.prepare_legacy(destination / 'casebook', case, admission, legacy)
            admission['legacy'] = derivative
            self.assertEqual(derivative['records'], 10)
            self.assertEqual((legacy / 'YRI.txt').read_text(), 'Y1\nY2\n')
            self.assertEqual((legacy / 'CEU.txt').read_text(), 'C1\nC2\n')
            rows = [line.split('\t') for line in (legacy / 'eligible.vcf').read_text().splitlines()
                    if not line.startswith('#')]
            self.assertTrue(all(len(row) == 13 and ',' not in row[4] for row in rows))
            self.assertFalse(any('.' in value.split(':')[0] for row in rows for value in row[9:]))
            (destination / 'admission.json').write_text(json.dumps(admission))
            oracle.verify_admission(destination / 'casebook', case, admission)
            (destination / 'casebook/casebook.vcf').write_text(original + '\n')
            with self.assertRaisesRegex(ValueError, 'identity'):
                oracle.verify_admission(destination / 'casebook', case, admission)
        runtime = json.loads((ROOT / 'assets/popgen-first-runtime-20261101.json').read_text())
        self.assertEqual(oracle.validate_runtime_lock(runtime), None)
        self.assertEqual(set(runtime['python_packages']),
                         {'scikit-allel', 'numpy', 'dask', 'click', 'cloudpickle', 'fsspec',
                          'packaging', 'partd', 'pyyaml', 'toolz', 'locket'})
        self.assertFalse({'numpy', 'allel', 'dask'}.intersection(sys.modules))


if __name__ == '__main__':
    unittest.main()
