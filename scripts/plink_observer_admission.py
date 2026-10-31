"""One hosted-only capacity change; no genotype policy or decoding change."""

import hashlib
from pathlib import Path

import plink_first_workflow_oracle as oracle
import plink_source_on_native as source
from plink_candidate import check, load, save, tree_manifest


ORIGINAL_SOURCE = '399a00c85e1678b1f88c093a0ca9dea55d8d5b099c84036391bf61ea8441bf61'


def widen(body):
    check(hashlib.sha256(body).hexdigest() == ORIGINAL_SOURCE, 'original observer source identity differs')
    before = b'variants.size() > 1000 || samples.size() > 1000'
    check(body.count(before) == 1, 'unique observer capacity expression absent')
    return body.replace(before, b'variants.size() > 500000 || samples.size() > 1000')


def admit(root, runtime):
    import plink_real_performance as real
    root = real.hosted(root)
    directory = root / 'evidence/observer-admission'; directory.mkdir()
    restored = Path(runtime['restored']); native = Path(runtime['native'])
    source_root = root / 'observer-source'
    proof = source.verify_restore(root / 'acquisition/source/original/original.tar', source_root,
        source.CAPTURE_SHA, 6360, lambda name: name.startswith('runtime/source/2.0/include/') or
        name in ('runtime/source/2.0/COPYING', 'runtime/source/2.0/COPYING.LESSER', 'controller/plink_pgen_capture.cc'))
    save(directory / 'restoration.json', proof)
    original = source_root / 'controller/plink_pgen_capture.cc'
    widened = directory / 'plink_pgen_capture_500000.cc'
    widened.write_bytes(widen(original.read_bytes()))
    spec = load(restored / 'controller/plink-first-runtime-20261031.json')
    oracle.validate_spec(spec)
    check(spec['source']['helper_flags'] == ['-std=c++17', '-O2', '-ffp-contract=off', '-pthread'], 'captured observer flags differ')
    base = source_root / 'runtime/source/2.0'
    binary = directory / 'plink_pgen_capture_500000'
    command = ['/usr/local/bin/g++', *spec['source']['helper_flags'], '-I' + str(base / 'include'), str(widened),
               *[str(base / name) for name in spec['source']['helper_sources']], '-o', str(binary)]
    before = tree_manifest(source_root)
    oracle.require(directory / 'compiler', ['/usr/local/bin/g++', '--version'])
    oracle.require(directory / 'build', command, directory, 1800)
    real.admit_elf(binary, directory / 'elf', probe_version=False)
    binding = load(native / 'generations-release.json'); graph = native / binding['graph']
    total = 0; observations = []
    for entry in binding['generations']:
        prefix = graph / entry['prefix']
        name = f"{entry['workers']}-{entry['lane']}-{entry['stage']}"
        old = directory / (name + '.old.jsonl.gz'); new = directory / (name + '.new.jsonl.gz')
        a = real.capture_raw(restored / source.ENGINES['helper'][0], prefix, old, gt_only=False, root=root)
        b = real.capture_raw(binary, prefix, new, old, gt_only=False, root=root)
        check(a['decompressed_sha256'] == b['decompressed_sha256'] and a['decompressed_bytes'] == b['decompressed_bytes'],
              'old/new compact observer raw byte streams differ')
        total += b['cells']; observations.append(dict(name=name, old=a, widened=b))
    check(len(observations) == 36 and total == 74878 and before == tree_manifest(source_root),
          'complete original compact observer equivalence or source immutability failed')
    save(directory / 'admission.json', dict(status='capacity_only_observer_admitted', old_binary_sha256=source.ENGINES['helper'][1],
        old_source_sha256=ORIGINAL_SOURCE, new_source_sha256=oracle.sha256(widened), new_binary_sha256=oracle.sha256(binary),
        sample_cap=1000, variant_cap=500000, actual_compact_cells=total, observations=observations,
        image=oracle.IMAGE, compiler_argv=command, upstream_engine_rebuilt=False, genotype_policy_changed=False,
        public_distribution=False))
    return binary
