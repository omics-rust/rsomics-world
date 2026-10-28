#!/usr/bin/env python3
"""Capture a pinned, hosted coordinate-selection and geometry source workflow."""

import argparse
from collections import deque
import contextlib
import hashlib
import importlib
import importlib.metadata
import inspect
import json
import math
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import traceback
import urllib.request
import warnings


BIO_COMMIT = '7a9c76cce8c6a58db791be2b12a135af210cedf2'
GEMMI_COMMIT = '6b9910a51c3f41bda204012bed9329aa01ba2442'
VERSIONS = {'biopython': '1.87', 'gemmi': '0.7.3', 'numpy': '2.3.4', 'threadpoolctl': '3.6.0'}
SEED_SHA256 = '1d22709c635f6427cb5da4843a3e98225329c35cc653d4a51a9c471f999752ec'
BIO_FILES = ('Bio/PDB/Entity.py', 'Bio/PDB/Atom.py', 'Bio/PDB/Residue.py', 'Bio/PDB/Chain.py',
    'Bio/PDB/Model.py', 'Bio/PDB/PDBParser.py', 'Bio/PDB/StructureBuilder.py', 'Bio/PDB/MMCIFParser.py',
    'Bio/PDB/MMCIF2Dict.py', 'Bio/PDB/PDBIO.py', 'Bio/PDB/Selection.py', 'Bio/PDB/vectors.py',
    'Bio/PDB/Polypeptide.py', 'Bio/PDB/HSExposure.py', 'Bio/PDB/NeighborSearch.py',
    'Bio/PDB/ic_data.py', 'Bio/PDB/kdtrees.c', 'Bio/SVDSuperimposer/__init__.py',
    'Bio/Data/IUPACData.py', 'Bio/Data/PDBData.py', 'LICENSE.rst', 'Tests/PDB/1A8O.cif')
GEMMI_FILES = ('python/cif.cpp', 'python/read.cpp', 'include/gemmi/cif.hpp',
               'include/gemmi/cifdoc.hpp', 'include/gemmi/to_cif.hpp', 'LICENSE.txt')
THREADS = ('OMP_NUM_THREADS', 'OPENBLAS_NUM_THREADS', 'MKL_NUM_THREADS', 'BLIS_NUM_THREADS',
           'VECLIB_MAXIMUM_THREADS', 'NUMEXPR_NUM_THREADS')
GROMACS_GYRATE = ('https://raw.githubusercontent.com/gromacs/gromacs/'
    '820db9e499e4edc7f0dc734e6baeb0e9c3f87859/src/gromacs/trajectoryanalysis/modules/gyrate.cpp')


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def encode(value):
    if isinstance(value, float) and not math.isfinite(value):
        return {'nonfinite': 'nan' if math.isnan(value) else '+inf' if value > 0 else '-inf'}
    if isinstance(value, dict):
        return {str(key): encode(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [encode(item) for item in value]
    if hasattr(value, 'tolist'):
        return encode(value.tolist())
    return value


def save(path, value):
    with Path(path).open('x') as stream:
        json.dump(encode(value), stream, indent=2, allow_nan=False)
        stream.write('\n')


def observe(output, request, callback):
    output.mkdir(parents=True)
    save(output / 'request.json', request)
    with (output / 'stdout.txt').open('w') as stdout, (output / 'stderr.txt').open('w') as stderr, \
            contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr), warnings.catch_warnings(record=True) as seen:
        warnings.simplefilter('always')
        try:
            record = {'status': 'returned', 'result': encode(callback())}
        except Exception as error:
            record = {'status': 'raised', 'exception': {'type': type(error).__name__,
                'module': type(error).__module__, 'message': str(error), 'traceback': traceback.format_exc()}}
        record['warnings'] = [{'type': type(item.message).__name__, 'message': str(item.message),
                              'filename': item.filename, 'lineno': item.lineno} for item in seen]
    save(output / 'observation.json', record)
    if record['status'] == 'raised' and record['exception']['type'] not in request.get('allowed_exceptions', []):
        raise RuntimeError(f'required source call failed: {output}')
    return record


def adversarial_cif():
    header = '''data_integrated_contract
_entry.id integrated_contract
_contract_state.unknown ?
_contract_state.inapplicable .
_contract_state.literal_unknown '?'
_contract_state.literal_inapplicable '.'
_contract_state.literal_spaces 'literal with spaces'
loop_
_entity.id
_entity.type
1 polymer
2 non-polymer
3 water
loop_
_entity_poly.entity_id
_entity_poly.type
1 'polypeptide(L)'
loop_
_entity_poly_seq.entity_id
_entity_poly_seq.num
_entity_poly_seq.mon_id
_entity_poly_seq.hetero
1 1 ALA n
1 2 GLY n
1 3 SER y
1 3 CYS y
1 4 ALA n
1 5 ALA n
loop_
_struct_asym.id
_struct_asym.entity_id
LABEL_LONG_CHAIN 1
LIGAND_CHAIN 2
WATER_CHAIN 3
loop_
_atom_site.group_PDB
_atom_site.id
_atom_site.type_symbol
_atom_site.label_atom_id
_atom_site.label_alt_id
_atom_site.label_comp_id
_atom_site.label_asym_id
_atom_site.label_entity_id
_atom_site.label_seq_id
_atom_site.pdbx_PDB_ins_code
_atom_site.Cartn_x
_atom_site.Cartn_y
_atom_site.Cartn_z
_atom_site.occupancy
_atom_site.B_iso_or_equiv
_atom_site.auth_seq_id
_atom_site.auth_comp_id
_atom_site.auth_asym_id
_atom_site.auth_atom_id
_atom_site.pdbx_PDB_model_num
'''
    residues = [
        ('ALA', 1, 10, '?', '.', 1., [('N', -1, 0, .1), ('C', 1, 1, 0)]),
        ('ALA', 1, 10, '?', 'A', .3, [('CA', 0, 0, 0)]),
        ('ALA', 1, 10, '?', 'B', .7, [('CA', 10, 0, 0)]),
        ('ALA', 1, 10, '?', 'B', .5, [('CB', 0, -1, 1)]),
        ('ALA', 1, 10, '?', 'A', .5, [('CB', 0, -1, -1)]),
        ('GLY', 2, 10, 'A', '.', 1., [('N', 2, 1, 0), ('CA', 3, 0, 0), ('C', 4, 1, 0)]),
        ('SER', 3, 11, '?', 'A', .8, [('N', 5, 1, 0), ('CA', 6, 0, 0), ('C', 7, 1, 0),
                                   ('CB', 6, 1, 1), ('OG', 7, 2, 1)]),
        ('CYS', 3, 11, '?', 'B', .2, [('N', 5, 1, 0), ('CA', 6, 0, 0), ('C', 7, 1, 0),
                                   ('CB', 6, 1, 1), ('SG', 7, 2, 1)]),
        ('ALA', 5, 15, '?', '.', 1., [('N', 19, 1, .1), ('CA', 20, 0, 0), ('C', 21, 1, .2)]),
    ]
    rows = []
    for model in (3, 7):
        for name, label_seq, author_seq, insertion, alt, occupancy, atoms in residues:
            for atom, x, y, z in atoms:
                element = atom[0] if atom[0] in 'NOS' else 'C'
                row = ['ATOM', f'atom{len(rows)+1:03}', element, atom, alt, name, 'LABEL_LONG_CHAIN',
                    '1', str(label_seq), insertion, f'{x:.6f}', f'{y:.6f}', f'{z:.6f}', str(occupancy),
                    '12.5', str(author_seq), name, 'AUTHOR_LONG_CHAIN', atom, str(model)]
                rows.append(' '.join(row))
    rows.extend([
        "HETATM '?' C C1 ? LIG LIGAND_CHAIN 2 . ? 30.000001 1.5 2.0 . ? . LIG '?' C1 3",
        "HETATM '.' O O . HOH WATER_CHAIN 3 . . 33.000002 2.5 3.0 ? . ? HOH '.' O 3",
    ])
    return header + '\n'.join(rows) + '\n#\n'


def prepare_inputs(seed, output):
    manifest = json.loads((seed / 'manifest.json').read_text())
    archive = seed / manifest['archive']
    if manifest['archive_sha256'] != SEED_SHA256 or sha256(archive) != SEED_SHA256:
        raise ValueError('historical seed archive digest differs')
    output.mkdir(parents=True)
    historical = output / 'historical'
    with tarfile.open(archive) as source:
        members = source.getmembers()
        if {item.name for item in members} != set(manifest['files']):
            raise ValueError('historical seed membership differs')
        for item in members:
            name = Path(item.name)
            if not item.isfile() or name.is_absolute() or '..' in name.parts:
                raise ValueError('unexpected historical archive member')
            contents = source.extractfile(item).read()
            if hashlib.sha256(contents).hexdigest() != manifest['files'][item.name]['sha256']:
                raise ValueError(f'historical fixture digest differs: {item.name}')
            destination = historical / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(contents)
    authored = output / 'integrated-contract.cif'
    authored.write_text(adversarial_cif())
    save(output / 'historical-provenance.json', manifest)
    save(output / 'authored-provenance.json', {'input': authored.name, 'sha256': sha256(authored),
        'origin': 'authored integrated contract fixture, not an experimental structure',
        'raw_only_rows': ['ligand/water null author sequence and occupancy; literal null-looking IDs'],
        'analysis_selection': {'model_number': '3', 'label_asym_id': 'LABEL_LONG_CHAIN'},
        'cases': ['dual IDs', 'long chain IDs', 'nonnumeric atom IDs', 'model order', 'residue variants',
                  'highest occupancy and first-seen ties', 'all-CA peptide connectivity', 'insertion/gaps',
                  'missing atoms', 'ligand/water', 'absent/unknown/inapplicable/quoted literal fields']})
    return manifest


def fetch(url, destination):
    with urllib.request.urlopen(url, timeout=120) as response, destination.open('xb') as output:
        shutil.copyfileobj(response, output)
    return {'url': url, 'sha256': sha256(destination), 'size_bytes': destination.stat().st_size}


def source_members(archive, prefix, names, destination):
    records = {}
    with tarfile.open(archive) as bundle:
        for name in names:
            member = bundle.getmember(prefix + '/' + name)
            if not member.isfile():
                raise ValueError(f'required regular source member absent: {name}')
            target = destination / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(bundle.extractfile(member).read())
            records[name] = sha256(target)
    return records


def qualify(sources, output):
    output.mkdir()
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    prefix = Path(sys.prefix).resolve(strict=True)
    if not prefix.is_relative_to(runner):
        raise ValueError('scientific environment escapes hosted runner-temp')
    import numpy as np
    import Bio
    from threadpoolctl import threadpool_info

    packages = {}
    for name, version in VERSIONS.items():
        distribution = importlib.metadata.distribution(name)
        location = Path(distribution.locate_file('')).resolve(strict=True)
        if distribution.version != version or not location.is_relative_to(prefix):
            raise ValueError(f'unqualified installed package: {name}')
        packages[name] = {'version': distribution.version, 'location': str(location)}
    bio_source = sources / 'biopython'
    gemmi_source = sources / 'gemmi'
    pins = {
        'biopython': {'commit': BIO_COMMIT, 'archive_sha256': sha256(sources / 'biopython.tar.gz'),
            'files': source_members(sources / 'biopython.tar.gz', 'biopython-' + BIO_COMMIT, BIO_FILES, bio_source)},
        'gemmi': {'commit': GEMMI_COMMIT, 'archive_sha256': sha256(sources / 'gemmi.tar.gz'),
            'files': source_members(sources / 'gemmi.tar.gz', 'gemmi-' + GEMMI_COMMIT, GEMMI_FILES, gemmi_source)},
    }
    compared = {}
    for relative in BIO_FILES:
        if not relative.startswith('Bio/') or not relative.endswith('.py'):
            continue
        installed = Path(Bio.__file__).parent / relative.removeprefix('Bio/')
        if sha256(installed) != pins['biopython']['files'][relative]:
            raise ValueError(f'installed Biopython body differs from pinned source: {relative}')
        compared[relative] = sha256(installed)
    with urllib.request.urlopen('https://pypi.org/pypi/numpy/' + VERSIONS['numpy'] + '/json', timeout=60) as response:
        metadata = json.load(response)
    save(output / 'numpy-pypi.json', metadata)
    sdist, = [record for record in metadata['urls'] if record['packagetype'] == 'sdist']
    archive = sources / sdist['filename']
    numpy_source = fetch(sdist['url'], archive)
    if numpy_source['sha256'] != sdist['digests']['sha256']:
        raise ValueError('NumPy archive differs from exact PyPI release digest')
    numpy_files = ('numpy/_core/fromnumeric.py', 'numpy/lib/_function_base_impl.py', 'numpy/linalg/_linalg.py')
    pins['numpy'] = {'version': VERSIONS['numpy'], 'archive': numpy_source,
        'files': source_members(archive, 'numpy-' + VERSIONS['numpy'], numpy_files, sources / 'numpy')}
    for relative in numpy_files:
        installed = Path(np.__file__).parent / relative.removeprefix('numpy/')
        if sha256(installed) != pins['numpy']['files'][relative]:
            raise ValueError(f'installed NumPy body differs from released source: {relative}')
        compared[relative] = sha256(installed)
    reference = fetch(GROMACS_GYRATE, sources / 'gromacs-2025.3-gyrate.cpp')
    pins['radius_of_gyration_definition'] = {'source': reference,
        'documentation': 'https://manual.gromacs.org/documentation/2025.3/onlinehelp/gmx-gyrate.html',
        'contract': 'mass mode uses sum(m_i*distance_to_weighted_center^2)/sum(m_i), then sqrt',
        'not_parity': 'rsomics retained NumPy profile counts unpacked alternate positions, uses element masses, '
                      'raw Angstrom coordinates and float32 reductions without PBC/topology; GROMACS uses '
                      'trajectory/topology selection and mass/PBC settings. No GROMACS package or run.'}
    functions = output / 'public-methods'
    functions.mkdir()
    public = ('Bio.PDB.Entity.Entity.center_of_mass', 'Bio.PDB.PDBParser.PDBParser.get_structure',
        'Bio.PDB.MMCIFParser.MMCIFParser.get_structure', 'Bio.PDB.MMCIF2Dict.MMCIF2Dict',
        'Bio.PDB.PDBIO.PDBIO.save', 'Bio.PDB.Polypeptide.PPBuilder.build_peptides',
        'Bio.PDB.Polypeptide.CaPPBuilder.build_peptides', 'Bio.PDB.Polypeptide.Polypeptide.get_phi_psi_list',
        'Bio.PDB.vectors.calc_dihedral', 'Bio.PDB.NeighborSearch.NeighborSearch.search_all',
        'Bio.PDB.HSExposure.HSExposureCA', 'Bio.PDB.HSExposure.HSExposureCB', 'Bio.PDB.HSExposure.ExposureCN',
        'Bio.SVDSuperimposer.SVDSuperimposer.run')
    methods = {}
    for dotted in public:
        pieces = dotted.split('.')
        for size in range(len(pieces) - 1, 0, -1):
            try:
                value = importlib.import_module('.'.join(pieces[:size]))
            except ModuleNotFoundError as error:
                if error.name != '.'.join(pieces[:size]):
                    raise
                continue
            for attribute in pieces[size:]:
                value = getattr(value, attribute)
            break
        body = inspect.getsource(value)
        file = functions / (dotted + '.py')
        file.write_text(body)
        methods[dotted] = {'source_file': inspect.getsourcefile(value), 'body_sha256': sha256(file)}
    binaries = {}
    for name in ('numpy._core._multiarray_umath', 'numpy.linalg._umath_linalg', 'Bio.PDB.kdtrees', 'gemmi.gemmi_ext'):
        module = importlib.import_module(name)
        binaries[name] = {'file': module.__file__, 'sha256': sha256(module.__file__)}
    with (output / 'numpy-config.txt').open('w') as stream, contextlib.redirect_stdout(stream):
        np.show_config()
    pools = threadpool_info()
    if any(os.environ.get(name) != '1' for name in THREADS) or any(pool['num_threads'] != 1 for pool in pools):
        raise ValueError('source runtime is not single threaded')
    runtime = {'python': sys.version, 'executable': sys.executable, 'prefix': str(prefix),
        'host': platform.uname()._asdict(), 'packages': packages, 'source_pins': pins,
        'installed_python_body_comparisons': compared, 'compiled_runtime': binaries,
        'compiled_identity_scope': 'released wheels SHA-bound separately; compiled code not rebuilt from source here',
        'public_methods': methods, 'thread_environment': {name: os.environ[name] for name in THREADS},
        'threadpools': pools, 'numpy_float32_epsilon': float(np.finfo(np.float32).eps),
        'numpy_float64_epsilon': float(np.finfo(np.float64).eps),
        'native_numeric_bands': 'not selected here; preserve source dtypes/results for pre-native contract adoption'}
    save(output / 'runtime.json', runtime)
    return bio_source


def read_structure(path, auth_chains=True, auth_residues=True):
    from Bio.PDB import MMCIFParser, PDBParser

    parser = (MMCIFParser(auth_chains=auth_chains, auth_residues=auth_residues, QUIET=False)
              if path.suffix == '.cif' else PDBParser(PERMISSIVE=True, QUIET=False))
    return parser.get_structure('source', str(path))


def residue_key(residue):
    chain = residue.get_parent()
    model = chain.get_parent()
    return [model.id, model.serial_num, chain.id, *residue.id, residue.get_resname()]


def atom_record(atom):
    return {'residue': residue_key(atom.get_parent()), 'name': atom.get_name(), 'fullname': atom.get_fullname(),
        'serial': atom.get_serial_number(), 'altloc': atom.get_altloc(), 'occupancy': atom.get_occupancy(),
        'element': atom.element, 'mass': atom.mass, 'bfactor': atom.get_bfactor(),
        'coord': atom.get_coord(), 'coord_dtype': str(atom.get_coord().dtype)}


def unpacked_atoms(model):
    return [atom for chain in model for residue in chain.get_unpacked_list() for atom in residue.get_unpacked_list()]


def selected_atoms(model, ca_only=False):
    from Bio.PDB.Polypeptide import is_aa

    return [atom for chain in model for residue in chain if is_aa(residue, standard=True)
            for atom in residue if atom.element not in ('H', 'D') and (not ca_only or atom.name == 'CA')]


def inspect_structure(structure):
    models = []
    for model in structure:
        models.append({'index': model.id, 'serial': model.serial_num, 'chains': [chain.id for chain in model],
            'selected_atoms': [atom_record(atom) for atom in model.get_atoms()],
            'unpacked_atoms': [atom_record(atom) for atom in unpacked_atoms(model)],
            'residue_variants': [{'residue': residue_key(residue), 'selected_name': residue.get_resname(),
                'all_names': residue.disordered_get_id_list()} for chain in model for residue in chain
                if residue.is_disordered() == 2]})
    return {'methods': ['Structure.get_models', 'Chain.get_unpacked_list', 'Residue.get_unpacked_list'],
        'identity_scope': 'Biopython author OR label view, not preservation of both namespaces', 'models': models}


def cif_fields(path):
    import gemmi
    from Bio.PDB.MMCIF2Dict import MMCIF2Dict

    document = gemmi.cif.read_file(str(path))
    block = document.sole_block()
    bio = MMCIF2Dict(str(path))
    categories = {}
    for category in block.get_mmcif_category_names():
        columns = block.get_mmcif_category(category, raw=True)
        captured = {}
        for column, values in columns.items():
            key = category + column
            decoded = [value if gemmi.cif.is_null(value) else gemmi.cif.as_string(value) for value in values]
            if bio[key] != decoded:
                raise ValueError(f'independent CIF field readers disagree: {key}')
            captured[column] = [{'raw': value, 'decoded': text,
                'kind': 'unknown' if value == '?' else 'inapplicable' if value == '.' else
                        'quoted-text' if value[:1] in ("'", '"', ';') else 'bare-text'}
                for value, text in zip(values, decoded)]
        categories[category] = captured
    return {'methods': ['Bio.PDB.MMCIF2Dict.MMCIF2Dict', 'gemmi.cif.read_file',
        'Block.get_mmcif_category(raw=True)', 'gemmi.cif.is_null', 'gemmi.cif.as_string'],
        'categories': categories, 'absent_fields': [tag for tag in ('_atom_site.pdbx_formal_charge',
            '_contract_state.absent') if tag not in bio], 'raw_precision': 'input token strings, not float32 Structure'}


def cif_select(path, output, model_number, label_chain):
    import gemmi

    document = gemmi.cif.read_file(str(path))
    block = document.sole_block()
    raw = block.get_mmcif_category('_atom_site.', raw=True)
    keep = [index for index in range(len(raw['id']))
            if gemmi.cif.as_string(raw['pdbx_PDB_model_num'][index]) == model_number
            and gemmi.cif.as_string(raw['label_asym_id'][index]) == label_chain]
    if not keep:
        raise ValueError('declared CIF selection retained no rows')
    selected = {name: [values[index] for index in keep] for name, values in raw.items()}
    block.set_mmcif_category('_atom_site.', selected, raw=True)
    anisotrop = block.get_mmcif_category('_atom_site_anisotrop.', raw=True)
    if anisotrop:
        retained_ids = set(selected['id'])
        rows = [index for index, atom_id in enumerate(anisotrop['id']) if atom_id in retained_ids]
        if rows:
            block.set_mmcif_category('_atom_site_anisotrop.',
                {name: [values[index] for index in rows] for name, values in anisotrop.items()}, raw=True)
        else:
            block.find_mmcif_category('_atom_site_anisotrop.').erase()
    document.write_file(str(output))
    reloaded = gemmi.cif.read_file(str(output)).sole_block().get_mmcif_category('_atom_site.', raw=True)
    if reloaded != selected:
        raise ValueError('CIF selection/reload changed atom-site token identities or values')
    return {'method': 'Gemmi raw category selection/write_file/reload, not Biopython Structure serialization',
        'source_row_indices': keep, 'model_number': model_number, 'label_asym_id': label_chain,
        'retained_atom_site': selected, 'output_sha256': sha256(output),
        'metadata_scope': 'coordinate witness: other original categories retained unchanged; no full-deposition repair claim'}


def pdb_select(structure, output, model_index, chain_id):
    from Bio.PDB import PDBIO, Select

    class ChainSelect(Select):
        def accept_model(self, model):
            return model.id == model_index

        def accept_chain(self, chain):
            return chain.id == chain_id

    writer = PDBIO()
    writer.set_structure(structure)
    writer.save(str(output), select=ChainSelect(), preserve_atom_numbering=True)
    return {'method': 'Bio.PDB.PDBIO.save', 'model_index': model_index, 'chain_id': chain_id,
        'preserve_atom_numbering': True, 'output_sha256': sha256(output),
        'reload': inspect_structure(read_structure(output)),
        'scope': 'PDB-representable identity only; not a dual-author/label mmCIF preservation oracle'}


def historical_python(historical, product, constant='ORACLE_PY'):
    path = historical / ('rsomics-pdb-' + product) / 'tests/compat.rs'
    match = re.search(r'const ' + constant + r': &str = r#"(.*?)"#;', path.read_text(), re.DOTALL)
    if match is None:
        raise ValueError(f'retained source expression missing: {path}')
    return path, match.group(1)


def geometry(model, historical, flags=()):
    import numpy as np

    source, body = historical_python(historical, 'geometry')
    expression = body[body.index('def walk(entity):'):]
    namespace = {'np': np, 'model': model, 'flags': set(flags), 'sys': sys, 'deque': deque}
    exec(compile(expression, str(source) + ':retained-numpy-expression', 'exec'), namespace)
    fields = {name: namespace[name] for name in ('com', 'cen', 'rg', 'rgg', 'total_mass', 'bmin', 'bmax', 'bdim')}
    return {'methods': ['Entity.center_of_mass(geometric=False)', 'Entity.center_of_mass(geometric=True)',
            'exact retained team NumPy geometry expression; not a Bio.PDB radius-of-gyration API'],
        'source_file': str(source), 'source_sha256': sha256(source),
        'expression_sha256': hashlib.sha256(expression.encode()).hexdigest(), 'filter_flags': flags,
        'bio_unfiltered_center_of_mass': model.center_of_mass(),
        'bio_unfiltered_centroid': model.center_of_mass(geometric=True),
        'filtered_reference_fields': fields, 'coords_dtype': str(namespace['coords'].dtype),
        'masses_dtype': str(namespace['masses'].dtype), 'center_dtype': str(namespace['com'].dtype),
        'atoms': [atom_record(atom) for atom in namespace['atoms']],
        'radius_profile': 'unpacked conformers, element mass weighting/sum weights, float32, Angstrom, no PBC'}


def dihedrals(model):
    from Bio.Data.PDBData import protein_letters_3to1
    from Bio.PDB.ic_data import ic_data_sidechains
    from Bio.PDB.Polypeptide import PPBuilder, is_aa
    from Bio.PDB.vectors import calc_dihedral

    peptides = PPBuilder().build_peptides(model, aa_only=1)
    backbone = {}
    for peptide in peptides:
        values = peptide.get_phi_psi_list()
        for index, (residue, (phi, psi)) in enumerate(zip(peptide, values)):
            previous = peptide[index - 1] if index else None
            omega_atoms = [previous['CA'], previous['C'], residue['N'], residue['CA']] if previous is not None \
                and all(name in previous for name in ('CA', 'C')) and all(name in residue for name in ('N', 'CA')) else None
            backbone[tuple(residue_key(residue))] = {'phi_radians': phi, 'psi_radians': psi,
                'omega_radians': calc_dihedral(*(atom.get_vector() for atom in omega_atoms)) if omega_atoms else None,
                'omega_atoms': [atom_record(atom) for atom in omega_atoms] if omega_atoms else None}
    rows = []
    for residue in model.get_residues():
        if not is_aa(residue, standard=True):
            continue
        key = residue_key(residue)
        definitions = [item for item in ic_data_sidechains.get(protein_letters_3to1[residue.resname], ())
                       if len(item) == 5 and item[-1].startswith('chi')]
        chis = {}
        for definition in definitions:
            names, name = definition[:4], definition[4]
            missing = [atom for atom in names if atom not in residue]
            chis[name] = {'atom_names': names, 'missing_atoms': missing,
                'radians': None if missing else calc_dihedral(*(residue[atom].get_vector() for atom in names))}
        rows.append({'residue': key, 'backbone': backbone.get(tuple(key)), 'sidechain': chis,
            'unavailable_reason': None if tuple(key) in backbone else 'not in source PPBuilder peptide',
            'precision': 'Atom.get_vector promotes parsed float32 coordinates to Vector float64'})
    return {'methods': ['PPBuilder(radius=1.8).build_peptides(aa_only=1)', 'Polypeptide.get_phi_psi_list',
        'Bio.PDB.vectors.calc_dihedral', 'Bio.PDB.ic_data.ic_data_sidechains'],
        'peptides': [[residue_key(residue) for residue in peptide] for peptide in peptides], 'rows': rows,
        'missing_contract': 'null backbone/angles remain unavailable; nonfinite source return is explicitly encoded'}


def contacts(model, cutoff=4., level='A', ca_only=False, min_seq_sep=0):
    from Bio.PDB import NeighborSearch

    atoms = selected_atoms(model, ca_only)
    search = NeighborSearch(atoms)
    pairs = search.search_all(cutoff, level=level)
    rows = []
    for left, right in pairs:
        a, b = (left.get_parent(), right.get_parent()) if level == 'A' else (left, right)
        if min_seq_sep and a.get_parent().id == b.get_parent().id and abs(a.id[1] - b.id[1]) < min_seq_sep:
            continue
        values = [atom_record(left), atom_record(right)] if level == 'A' else [residue_key(left), residue_key(right)]
        rows.append(sorted(values, key=lambda value: json.dumps(encode(value), sort_keys=True)))
    rows.sort(key=lambda row: json.dumps(encode(row), sort_keys=True))
    return {'method': 'Bio.PDB.NeighborSearch.search_all', 'level': level, 'cutoff': cutoff,
        'distance_rule': '<= cutoff, source KDTree', 'ca_only': ca_only, 'min_author_seq_separation': min_seq_sep,
        'selected_atoms': [atom_record(atom) for atom in atoms], 'kdtree_coord_dtype': str(search.coords.dtype),
        'pairs': rows, 'pair_count': len(rows)}


def exposure(model, radius=13., offset=0):
    from Bio.PDB.HSExposure import HSExposureCA, HSExposureCB, ExposureCN
    from Bio.PDB.Polypeptide import CaPPBuilder

    peptides = CaPPBuilder().build_peptides(model, aa_only=1)
    HSExposureCA(model, radius=radius, offset=offset)
    HSExposureCB(model, radius=radius, offset=offset)
    ExposureCN(model, radius=radius, offset=offset)
    keys = ('EXP_HSE_A_U', 'EXP_HSE_A_D', 'EXP_HSE_B_U', 'EXP_HSE_B_D', 'EXP_CN', 'EXP_CB_PCB_ANGLE')
    rows = []
    for residue in model.get_residues():
        ca = residue['CA'] if 'CA' in residue else None
        variants = ca.disordered_get_list() if ca is not None and ca.is_disordered() == 2 else [ca] if ca is not None else []
        rows.append({'residue': residue_key(residue), 'properties': {key: residue.xtra.get(key) for key in keys},
            'selected_CA': atom_record(ca) if ca is not None else None,
            'all_CA_connectivity_candidates': [atom_record(atom) for atom in variants],
            'missing_properties': [key for key in keys if key not in residue.xtra],
            'unavailable_contract': 'source property-map absence: peptide/atom/direction/terminus dependent, never zero-filled'})
    return {'methods': ['CaPPBuilder(radius=4.3).build_peptides(aa_only=1)', 'HSExposureCA', 'HSExposureCB', 'ExposureCN'],
        'radius': radius, 'offset': offset, 'distance_rule': 'strict < radius; directional angle strict < pi/2',
        'connectivity': 'any CA conformer pair; calculation uses selected CA; peptide-local offset, inter-peptide neighbors',
        'glycine': 'source pseudo-CB rotates N -120 degrees about CA to C',
        'peptides': [[residue_key(residue) for residue in peptide] for peptide in peptides], 'rows': rows}


def superpose(reference, moving, reference_ids, moving_ids):
    import numpy as np
    from Bio.SVDSuperimposer import SVDSuperimposer

    fixed = np.asarray(reference, dtype=np.float64)
    mobile = np.asarray(moving, dtype=np.float64)
    fit = SVDSuperimposer()
    fit.set(fixed, mobile)
    fit.run()
    rotation, translation = fit.get_rotran()
    return {'methods': ['SVDSuperimposer.set', 'SVDSuperimposer.run', 'get_rotran', 'get_rms', 'get_transformed'],
        'reference_ids': reference_ids, 'moving_ids': moving_ids, 'reference': fixed, 'moving': mobile,
        'input_dtype': str(fixed.dtype), 'rotation': rotation, 'translation': translation, 'rmsd': fit.get_rms(),
        'transformed': fit.get_transformed(), 'determinant': np.linalg.det(rotation),
        'reference_centered_rank': int(np.linalg.matrix_rank(fixed - fixed.mean(axis=0))),
        'convention': 'moving @ rotation + translation, proper rotation; fixed correspondence, no alignment',
        'rank_deficient_comparison': 'transformed points and optimum RMSD, not a unique rotation matrix'}


def raw_pdb_ca(path):
    points, ids = [], []
    for index, line in enumerate(path.read_text().splitlines()):
        if line.startswith('ENDMDL'):
            break
        if line.startswith('ATOM  ') and line[12:16].strip() == 'CA':
            points.append([float(line[start:start + 8]) for start in (30, 38, 46)])
            ids.append({'line_index': index, 'serial': line[6:11], 'chain': line[21:22],
                        'residue': line[22:27], 'name': line[12:16], 'altloc': line[16:17]})
    return points, ids


def lifecycle(path, output, historical):
    import gemmi

    output.mkdir(parents=True)
    structure = read_structure(path)
    observe(output / 'inspect', {'operation': 'inspect', 'input': str(path), 'sha256': sha256(path)},
            lambda: inspect_structure(structure))
    first = next(iter(structure))
    if path.suffix == '.cif':
        observe(output / 'raw-cif-inspect', {'operation': 'raw fields and independent reader agreement'}, lambda: cif_fields(path))
        raw = gemmi.cif.read_file(str(path)).sole_block().get_mmcif_category('_atom_site.', raw=True)
        number = gemmi.cif.as_string(raw['pdbx_PDB_model_num'][0])
        chains = list(dict.fromkeys(gemmi.cif.as_string(chain) for chain, model in zip(raw['label_asym_id'], raw['pdbx_PDB_model_num'])
                                   if gemmi.cif.as_string(model) == number))
        for index, chain in enumerate(chains):
            selected = output / f'split-{index}.cif'
            observe(output / f'select-{index}', {'operation': 'select/split/reload', 'model_number': number,
                'label_asym_id': chain}, lambda chain=chain, selected=selected: cif_select(path, selected, number, chain))
        selected = output / 'split-0.cif'
        observe(output / 'selected-raw-cif', {'operation': 'selected raw fields and independent reader agreement'}, lambda: cif_fields(selected))
    else:
        for index, chain in enumerate(first):
            selected = output / f'split-{index}.pdb'
            observe(output / f'select-{index}', {'operation': 'select/split/reload', 'chain': chain.id},
                lambda chain=chain, selected=selected: pdb_select(structure, selected, first.id, chain.id))
        selected = output / 'split-0.pdb'
    model = next(iter(read_structure(selected)))
    observe(output / 'selected-inspect', {'operation': 'reload selected structure', 'path': str(selected)},
            lambda: inspect_structure(read_structure(selected)))
    for operation, function in (('geometry', lambda: geometry(model, historical)), ('dihedrals', lambda: dihedrals(model)),
                                ('contacts', lambda: contacts(model)), ('exposure', lambda: exposure(model))):
        observe(output / operation, {'operation': operation, 'selected_input': str(selected),
            'profile': 'operation-specific source view, not one forced global atom list'}, function)
    ca = selected_atoms(model, ca_only=True)
    fixed = [atom.coord for atom in ca]
    moved = [atom.coord + [1., -2., 3.] for atom in ca]
    ids = [atom_record(atom) for atom in ca]
    observe(output / 'superpose', {'operation': 'superpose', 'correspondence': 'selected CA identity, translated copy',
        'input_profile': 'parsed float32 coordinates widened to float64'}, lambda: superpose(fixed, moved, ids, ids))


def retained_references(historical, output):
    output.mkdir()
    cases = [('geometry', '1ubq.pdb', [], '1ubq.geom.tsv'), ('geometry', '1a2p.pdb', [], '1a2p.geom.tsv'),
        ('geometry', '3nir.pdb', [], '3nir.geom.tsv'),
        ('geometry', '3nir.pdb', ['--no-hydrogens', '--no-hetatm'], '3nir.heavy.geom.tsv'),
        ('geometry', '3nir.pdb', ['--ca-only'], '3nir.ca.geom.tsv'),
        ('dihedrals', '1crn.pdb', [], '1crn.dihedrals.tsv'), ('dihedrals', 'altloc.pdb', [], 'altloc.dihedrals.tsv'),
        ('contacts', '1crn.pdb', ['4.0', 'atom'], '1crn.atom_4.0.tsv'),
        ('contacts', '1crn.pdb', ['8.0', 'residue'], '1crn.residue_8.0.tsv'),
        ('contacts', '1crn.pdb', ['8.0', 'residue', '--ca-only', '--min-seq-sep=2'], '1crn.residue_ca_8.0_sep2.tsv'),
        ('contacts', '1crn.pdb', ['6.703862694776753', 'atom', '--ca-only'], '1crn.atom_ca_boundary.tsv'),
        ('contacts', 'altloc.pdb', ['3.0', 'atom', '--ca-only'], 'altloc.atom_ca_3.0.tsv'),
        ('hsexposure', '1crn.pdb', ['12'], '1crn.r12.expected.tsv'),
        ('hsexposure', '1crn.pdb', ['13'], '1crn.r13.expected.tsv'),
        ('hsexposure', '4hhb.pdb', ['13'], '4hhb.r13.expected.tsv'),
        ('hsexposure', 'altloc.pdb', ['10'], 'altloc.r10.expected.tsv'),
        ('hsexposure', 'altloc.pdb', ['13'], 'altloc.r13.expected.tsv')]
    for product, fixture, flags, golden in cases:
        root = historical / ('rsomics-pdb-' + product)
        if product == 'contacts':
            original = root / 'tests/contacts_oracle.py'
            body = original.read_text()
        else:
            original, body = historical_python(historical, product, 'ORACLE' if product == 'hsexposure' else 'ORACLE_PY')
        script = output / (product + '-retained-oracle.py')
        if not script.exists():
            script.write_text(body)
        name = product + '-' + golden
        def invoke(script=script, root=root, fixture=fixture, flags=flags, golden=golden, original=original):
            command = [sys.executable, '-B', str(script), str(root / 'tests/golden' / fixture), *flags]
            result = subprocess.run(command, text=True, capture_output=True, timeout=180)
            if result.returncode:
                print(result.stdout)
                print(result.stderr, file=sys.stderr)
                raise RuntimeError(f'retained required source oracle exited {result.returncode}: {command}')
            expected = root / 'tests/golden' / golden
            return {'command': command, 'source_sha256': sha256(original), 'extracted_script_sha256': sha256(script),
                'input_sha256': sha256(root / 'tests/golden' / fixture), 'golden_sha256': sha256(expected),
                'stdout': result.stdout, 'stderr': result.stderr, 'retained_golden': expected.read_text(),
                'byte_equal_to_historical_golden': result.stdout == expected.read_text(),
                'status_scope': 'current source observation; any old numeric/profile mismatch retained, not a native tolerance gate'}
        observe(output / name, {'operation': 'reexecute retained historical source oracle',
            'profile': 'legacy comparison asset, not canonical peptide continuity or raw identity preservation'}, invoke)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--seed', type=Path, required=True)
    parser.add_argument('--sources', type=Path, required=True)
    args = parser.parse_args(argv)
    runner = Path(os.environ['RUNNER_TEMP']).resolve(strict=True)
    if not args.output.resolve().is_relative_to(runner) or not args.sources.resolve().is_relative_to(runner):
        parser.error('source workflow outputs and package sources must remain in hosted RUNNER_TEMP')
    args.output.mkdir(parents=True)
    status = {'schema': 'rsomics.structure.first-workflow-source.v1', 'status': 'running', 'excluded': ['sasa', 'dssp', 'align'],
        'not_claimed': ['native parser fitness', 'native numeric compatibility', 'performance advantage', 'native4 tests']}
    try:
        historical_manifest = prepare_inputs(args.seed, args.output / 'inputs')
        bio_source = qualify(args.sources, args.output / 'qualification')
        inputs = args.output / 'inputs'
        public_input = inputs / '1A8O.cif'
        shutil.copyfile(bio_source / 'Tests/PDB/1A8O.cif', public_input)
        contents = public_input.read_bytes()
        if hashlib.sha1(b'blob ' + str(len(contents)).encode() + b'\0' + contents).hexdigest() != \
                '375333cb63cfca2772f883be1085fd00d78bee1b':
            raise ValueError('pinned public mmCIF bytes differ from source Git blob')
        save(inputs / 'public-mmcif-provenance.json', {'path': public_input.name, 'sha256': sha256(public_input),
            'source_url': f'https://raw.githubusercontent.com/biopython/biopython/{BIO_COMMIT}/Tests/PDB/1A8O.cif',
            'source_commit': BIO_COMMIT, 'source_git_blob': '375333cb63cfca2772f883be1085fd00d78bee1b',
            'reuse': 'admit and reuse these artifact bytes for native comparisons; no moving latest input'})
        historical = inputs / 'historical'
        lifecycle(historical / 'rsomics-pdb-geometry/tests/golden/1ubq.pdb', args.output / 'pdb-workflow', historical)
        lifecycle(public_input, args.output / 'mmcif-workflow', historical)
        lifecycle(historical / 'rsomics-pdb-chain/tests/golden/small.pdb', args.output / 'small-pdb-workflow', historical)
        authored = inputs / 'integrated-contract.cif'
        observe(args.output / 'integrated-raw-io', {'operation': 'raw CIF identity and token contract'}, lambda: cif_fields(authored))
        observe(args.output / 'integrated-structure-boundary', {'operation': 'Biopython author-residue Structure boundary',
            'allowed_exceptions': ['ValueError', 'PDBConstructionException'],
            'scope': 'raw-only ligand/water null IDs/occupancy; actual constructor outcome, not a preservation oracle'},
            lambda: inspect_structure(read_structure(authored)))
        chosen = inputs / 'integrated-selected-polymer.cif'
        observe(args.output / 'integrated-raw-selection', {'operation': 'select representable analysis population without rewriting fields'},
            lambda: cif_select(authored, chosen, '3', 'LABEL_LONG_CHAIN'))
        lifecycle(chosen, args.output / 'integrated-selected-workflow', historical)
        def cutoff_boundary():
            model = next(iter(read_structure(chosen)))
            return {'contacts': contacts(model, cutoff=3., level='A', ca_only=True),
                    'exposure': exposure(model, radius=3.)}
        observe(args.output / 'integrated-cutoff-boundary', {'operation': 'same selected CA population at exact 3 Angstrom boundary',
            'profiles': ['contacts inclusive <=', 'HSE strict <; CA peptide membership uses all conformers']}, cutoff_boundary)
        observe(args.output / 'integrated-label-residue-view', {'operation': 'alternative Biopython label chain/residue constructor',
            'scope': 'separate selected-population view, never simultaneous dual-ID preservation'},
            lambda: inspect_structure(read_structure(chosen, auth_chains=False, auth_residues=False)))
        observe(args.output / 'unrepresentable-pdb-boundary', {'operation': 'PDBIO rejects long chain IDs',
            'allowed_exceptions': ['PDBIOException'], 'scope': 'PDB representation limit, not mmCIF data loss'},
            lambda: pdb_select(read_structure(chosen), args.output / 'unrepresentable.pdb', 0, 'AUTHOR_LONG_CHAIN'))
        retained_references(historical, args.output / 'retained-reference-profiles')
        geometry_model = next(iter(read_structure(historical / 'rsomics-pdb-geometry/tests/golden/3nir.pdb')))
        for name, flags in (('all-conformers', ()), ('heavy', ('--no-hetatm', '--no-hydrogens')), ('CA', ('--ca-only',))):
            observe(args.output / ('geometry-3nir-' + name), {'operation': 'geometry source precision and unpacked conformer view',
                'filter_flags': flags}, lambda flags=flags: geometry(geometry_model, historical, flags))
        for product, fixture in (('dihedrals', 'altloc.pdb'), ('hsexposure', 'altloc.pdb'), ('hsexposure', '4hhb.pdb')):
            model = next(iter(read_structure(historical / ('rsomics-pdb-' + product) / 'tests/golden' / fixture)))
            operation = dihedrals if product == 'dihedrals' else exposure
            observe(args.output / (product + '-' + fixture), {'operation': 'typed current-source view over retained real fixture'},
                    lambda model=model, operation=operation: operation(model))
        fit_root = historical / 'rsomics-pdb-superpose/tests/golden'
        for name, fixed, moving, golden in (('main', 'ref.pdb', 'mov.pdb', 'main.golden.tsv'),
                                            ('reflection', 'refl_ref.pdb', 'refl_mov.pdb', 'refl.golden.tsv')):
            points, ids = raw_pdb_ca(fit_root / fixed)
            moved, mids = raw_pdb_ca(fit_root / moving)
            observe(args.output / ('fit-' + name), {'operation': 'SVDSuperimposer', 'input_profile': 'raw decimal PDB float64 CA',
                'retained_golden_sha256': sha256(fit_root / golden), 'retained_golden': (fit_root / golden).read_text()},
                lambda points=points, moved=moved, ids=ids, mids=mids: superpose(points, moved, ids, mids))
        observe(args.output / 'fit-rank-deficient', {'operation': 'SVDSuperimposer explicit collinear boundary'},
            lambda: superpose([[0., 0., 0.], [1., 0., 0.], [2., 0., 0.]],
                [[1., 2., 3.], [2., 2., 3.], [3., 2., 3.]], ['p0', 'p1', 'p2'], ['q0', 'q1', 'q2']))
        def degenerate_torsion():
            from Bio.PDB.vectors import Vector, calc_dihedral
            return {'radians': calc_dihedral(*(Vector(float(i), 0., 0.) for i in range(4))),
                    'contract': 'actual nonfinite/exception observation, never fabricated zero'}
        observe(args.output / 'undefined-torsion', {'operation': 'calc_dihedral collinear boundary',
            'allowed_exceptions': ['ZeroDivisionError', 'ValueError']}, degenerate_torsion)
        status.update(status='source-workflow-returned', historical_assets=list(historical_manifest['assets']))
    except Exception as error:
        status.update(status='failed', exception={'type': type(error).__name__, 'message': str(error),
                                                 'traceback': traceback.format_exc()})
        raise
    finally:
        save(args.output / 'run.json', status)


if __name__ == '__main__':
    main()
