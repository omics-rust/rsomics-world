import importlib.util
import hashlib
import resource
import io
import gzip
import json
import os
from pathlib import Path
import sys
import subprocess
import tempfile
import tarfile
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))


class PopgenSourcePreparationTest(unittest.TestCase):
    def test_connected_casebook_admits_lossless_shapes_and_rejects_invalid_genotypes(self):
        self.assertTrue((ROOT / 'scripts/popgen_hosted_source.py').is_file(),
                        'manual hosted source integration is absent')
        self.assertTrue((ROOT / 'scripts/popgen_whole_real.py').is_file(),
                        'complete same-machine whole-real workflow is absent')
        import popgen_whole_real as whole
        policy = whole.load_policy()
        planned = whole.schedule(policy)
        self.assertEqual([(row['workers'], row['warmup'], row['engine']) for row in planned],
            [(1, True, 'native'), (1, True, 'source'),
             (1, False, 'native'), (1, False, 'source'),
             (1, False, 'source'), (1, False, 'native'),
             (1, False, 'native'), (1, False, 'source'),
             (1, False, 'source'), (1, False, 'native'),
             (1, False, 'native'), (1, False, 'source'),
             (4, True, 'source'), (4, True, 'native'),
             (4, False, 'source'), (4, False, 'native'),
             (4, False, 'native'), (4, False, 'source'),
             (4, False, 'source'), (4, False, 'native'),
             (4, False, 'native'), (4, False, 'source'),
             (4, False, 'source'), (4, False, 'native')])
        self.assertEqual(whole.retention_partition(policy), 16693329920)
        self.assertEqual(policy['limits']['sampled_tree_rss_bytes'], 8589934592)
        self.assertNotIn('address_space_bytes', policy['limits'])
        self.assertFalse(policy['limits']['sampled_rss_is_hard_peak'])
        self.assertEqual(hashlib.sha256((ROOT / 'assets/popgen-whole-real-compare-20261101.py').read_bytes()).hexdigest(),
                         'd14f08cee17b395ff6460692c166d4740f352ea9ee6c33115449ce778d0177d9')
        whole.validate_workflow(ROOT / '.github/workflows/popgen-whole-real.yml')
        with self.assertRaisesRegex((ValueError, FileNotFoundError), 'receipt'):
            whole.read_native_receipt(ROOT / '.autopilot/oracles/absent-popgen-native.json')
        adopted = whole.read_native_receipt(ROOT / '.autopilot/oracles/popgen-native-2026-11-01.json')
        self.assertEqual(adopted['run_id'], 41265985145)
        self.assertEqual(adopted['binary_sha256'], '2800d57d0508a4979dccb850688bc475c58c1cedb588427c1f5ba0fa05bb0525')
        before_address_limit = resource.getrlimit(resource.RLIMIT_AS)
        whole.child_setup(None)
        self.assertEqual(resource.getrlimit(resource.RLIMIT_AS), before_address_limit)
        import popgen_hosted_source as hosted
        producer = ROOT / 'scripts/popgen_first_workflow_oracle.py'
        self.assertTrue(producer.is_file(), 'connected popgen source producer is absent')
        import popgen_first_workflow_inputs as inputs
        spec = importlib.util.spec_from_file_location('popgen_oracle', producer)
        oracle = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(oracle)
        self.assertFalse({'numpy', 'allel', 'dask'}.intersection(sys.modules))
        with tempfile.TemporaryDirectory(prefix='popgen-preparation-', dir=os.environ['TMPDIR']) as directory:
            destination = Path(directory)
            bootstrap_environment = dict(GITHUB_ACTIONS='true', GITHUB_REPOSITORY='omics-rust/rsomics-world',
                GCC_VERSION='14.3.0', POPGEN_CONTAINER_DIGEST=oracle.IMAGE,
                GITHUB_SHA='5addd6108799a500094880751af1b66556e8b1e4', GITHUB_RUN_ID='41269279456',
                GITHUB_RUN_ATTEMPT='1', RUNNER_TEMP=str(destination), GITHUB_ENV=str(destination / 'bootstrap.env'))
            with patch.dict(os.environ, bootstrap_environment, clear=True), \
                 patch.object(hosted.platform, 'system', return_value='Linux'), \
                 patch.object(hosted.platform, 'machine', return_value='x86_64'), \
                 patch.object(sys, 'argv', ['popgen_whole_real.py', 'init', '--native-receipt',
                     str(ROOT / '.autopilot/oracles/popgen-native-2026-11-01.json')]):
                self.assertEqual(whole.main(), 0)
                bootstrap = whole.paths()
                written_environment = dict(line.split('=', 1) for line in (destination / 'bootstrap.env').read_text().splitlines())
                for name in hosted.CACHE_NAMES:
                    self.assertEqual(Path(os.environ[name]), bootstrap['private'] / 'cache' / name)
                    self.assertEqual(written_environment[name], os.environ[name])
                controller = bootstrap['private'] / 'controller'
                copied = json.loads((controller / 'identities.json').read_text())
                self.assertEqual(len(copied), 11)
                for name, expected in copied.items():
                    self.assertEqual((controller / name).read_bytes(), (ROOT / name).read_bytes())
                    self.assertEqual(whole.identity(controller / name), expected)
            session = hosted.new_session(destination)
            fresh_source = destination / 'fresh-source'
            fresh_source.mkdir()
            (fresh_source / 'body.txt').write_bytes(b'actual fresh body\n')
            inputs.save(fresh_source / 'output-manifest.json', {'body.txt':
                {'size': 18, 'sha256': hashlib.sha256(b'actual fresh body\n').hexdigest()}})
            view = whole.source_view(destination / 'views/first', fresh_source, ROOT / 'assets/popgen-first-input-20261101.json')
            index = json.loads((view / 'tar-verification.json').read_text())
            self.assertEqual(index['kind'], 'fresh-source-output-view')
            self.assertFalse(index['archive_authentication_claim'])
            self.assertEqual(Path(index['files']['real-source/body.txt']['extracted_path']).read_bytes(), b'actual fresh body\n')
            (fresh_source / 'body.txt').write_bytes(b'changed body\n')
            with self.assertRaisesRegex(ValueError, 'body'):
                whole.source_view(destination / 'views/changed', fresh_source, ROOT / 'assets/popgen-first-input-20261101.json')
            self.assertEqual(session['private'].stat().st_mode & 0o777, 0o700)
            transient = session['private'] / 'transient.tmp'
            transient.write_bytes(b'x')
            original_lstat = Path.lstat
            def disappearing_metadata(path, *args, **kwargs):
                if path == transient:
                    transient.unlink()
                return original_lstat(path, *args, **kwargs)
            with patch.object(Path, 'lstat', disappearing_metadata):
                self.assertEqual(hosted.retained_size(session['private']), 0)
            self.assertFalse(session['cipher'].is_relative_to(session['private']))
            self.assertEqual(set(hosted.public_outputs(session['session'])), {'payload.tar.age', 'receipt.json'})
            capacity = hosted.retention_plan()
            self.assertGreater(capacity['initial_free_bytes'], 40 * 2**30)
            self.assertEqual(capacity['retained_limit_bytes'], 16 * 2**30)
            with self.assertRaisesRegex(ValueError, 'casebook'):
                hosted.require_casebook(session['private'])
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
            fast = destination / 'lightweight-legacy'
            fast.mkdir()
            with patch.object(inputs, 'records', side_effect=AssertionError('independent GT reparse')):
                fast_derivative = whole.write_legacy(destination / 'casebook', case, admission,
                    {0, 1, 7, 8, 11, 12, 14, 15, 16, 17}, fast)
            self.assertEqual(fast_derivative['records'], 10)
            for name in ('eligible.vcf', 'YRI.txt', 'CEU.txt'):
                self.assertEqual((fast / name).read_bytes(), (legacy / name).read_bytes())
            rows = [line.split('\t') for line in (legacy / 'eligible.vcf').read_text().splitlines()
                    if not line.startswith('#')]
            self.assertTrue(all(len(row) == 13 and ',' not in row[4] for row in rows))
            self.assertFalse(any('.' in value.split(':')[0] for row in rows for value in row[9:]))
            (destination / 'admission.json').write_text(json.dumps(admission))
            oracle.verify_admission(destination / 'casebook', case, admission)
            (destination / 'casebook/casebook.vcf').write_text(original + '\n')
            with self.assertRaisesRegex(ValueError, 'identity'):
                oracle.verify_admission(destination / 'casebook', case, admission)
            inventory = hosted.archive_inventory(destination / 'casebook')
            buffer = io.BytesIO()
            hosted.write_tar(destination / 'casebook', inventory, buffer)
            buffer.seek(0)
            with tarfile.open(fileobj=buffer, mode='r:') as archive:
                self.assertEqual(archive.extractfile('casebook.vcf').read(),
                                 (destination / 'casebook/casebook.vcf').read_bytes())
            carrier = destination / 'prepared.tar.gz'
            carrier.write_bytes(gzip.compress(buffer.getvalue()))
            hosted.extract_archive(carrier, destination / 'extracted', 2**20)
            self.assertEqual((destination / 'extracted/casebook.vcf').read_bytes(),
                             (destination / 'casebook/casebook.vcf').read_bytes())
            external = destination / 'outside.txt'
            external.write_text('must not enter the archive')
            (destination / 'casebook/escape').symlink_to(external)
            with self.assertRaisesRegex(ValueError, 'symlink'):
                hosted.archive_inventory(destination / 'casebook')
            workflow = ROOT / '.github/workflows/popgen-first-workflow-oracle.yml'
            unsafe_workflow = destination / 'unsafe.yml'
            unsafe_workflow.write_text(workflow.read_text().replace('/cipher/receipt.json', '/private/controller.stderr'))
            with self.assertRaisesRegex(ValueError, 'receipt|plaintext'):
                hosted.validate_workflow(unsafe_workflow)
            unsafe_workflow.write_text((ROOT / '.github/workflows/popgen-whole-real.yml').read_text().replace('/cipher/receipt.json', '/private/controller.stderr'))
            with self.assertRaisesRegex(ValueError, 'receipt|ciphertext'):
                whole.validate_workflow(unsafe_workflow)
        runtime = json.loads((ROOT / 'assets/popgen-first-runtime-20261101.json').read_text())
        self.assertEqual(oracle.validate_runtime_lock(runtime), None)
        self.assertEqual(set(runtime['python_packages']),
                         {'scikit-allel', 'numpy', 'dask', 'click', 'cloudpickle', 'fsspec',
                          'packaging', 'partd', 'pyyaml', 'toolz', 'locket'})
        self.assertFalse({'numpy', 'allel', 'dask'}.intersection(sys.modules))
        self.assertEqual(hosted.validate_workflow(ROOT / '.github/workflows/popgen-first-workflow-oracle.yml'), None)
        records = []
        for workers in (1, 4):
            for pair in range(1, 6):
                records.extend([dict(workers=workers, pair=pair, warmup=False, engine='native',
                    wall_seconds=80.0, total_cpu_seconds=102.0, kernel_maxrss_kib=100.0),
                    dict(workers=workers, pair=pair, warmup=False, engine='source',
                    wall_seconds=100.0, total_cpu_seconds=100.0, kernel_maxrss_kib=100.0)])
        self.assertTrue(whole.advantage(records, policy)['material_advantage_observed'])
        for row in records:
            if row['workers'] == 4 and row['engine'] == 'native':
                row['wall_seconds'] = 100.0
        self.assertFalse(whole.advantage(records, policy)['material_advantage_observed'])
        self.assertTrue(whole.advantage(records, policy)['root_original_source_adoption_required'])
        lines = ((ROOT / '.github/workflows/popgen-first-workflow-oracle.yml').read_text() + '\n' +
                 (ROOT / '.github/workflows/popgen-whole-real.yml').read_text()).splitlines()
        for index, line in enumerate(lines):
            if line != '        run: |':
                continue
            body = []
            for current in lines[index + 1:]:
                if current and not current.startswith('          '):
                    break
                body.append(current[10:] if current else '')
            checked = subprocess.run(['bash', '-n'], input='\n'.join(body) + '\n', text=True, capture_output=True)
            self.assertEqual(checked.returncode, 0, checked.stderr)


if __name__ == '__main__':
    unittest.main()
