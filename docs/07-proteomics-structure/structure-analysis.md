# Structure analysis and comparison

> Coordinate-file inspection, geometric analysis, solvent accessibility,
> secondary structure, superposition, and pairwise structural alignment.

The canonical implementation boundary is the
[`rsomics-structure` product dossier](../10-products/structure.md). This page
records the wider upstream landscape.

## Included product scope

`rsomics-structure` is one Layer B product over a shared coordinate, selection,
and geometry model:

| Upstream capability | Product operation | State |
|---|---|---|
| PDB and PDBx/mmCIF inspection and semantic selection | `inspect`, `select` | first-release scope |
| Bio.PDB-style descriptors, torsions, contacts, HSE, and fixed correspondence | `geometry`, `dihedrals`, `contacts`, `exposure`, `superpose` | first-release scope |
| FreeSASA Lee-Richards and Shrake-Rupley | `sasa` | later complete slice |
| DSSP 4 secondary-structure assignment | `secondary` | later complete slice |
| TM-align monomeric protein structural alignment | `align` | later complete slice |
| broader US-align modes | later `align` modes | separately gated |

The nine historical operation repositories are implementation assets for these
subcommands. They are not public package boundaries.

## Coordinate IO

PDBx/mmCIF is the current structural-biology representation and is first-class.
Legacy PDB remains supported, but its one-character chain field and record
limits cannot define the internal model.

[`pdbtbx` 0.12.0](https://docs.rs/pdbtbx/0.12.0/pdbtbx/) is the candidate
external Rust PDB/mmCIF reader and writer. It is adopted only after round-trip
tests show that the rsomics model can preserve author and label identifiers,
models, entities, conformers, insertion codes, missing values, and required
numeric fields. Product-local adapters own selection and upstream compatibility
policy.

`rsomics-pdb-core` is not a public foundation: its historical dependents all
collapse into one target product. Useful parser behavior and tests are
internalized or replaced.

## Analysis oracles

### Biopython Bio.PDB

[Biopython 1.87](https://biopython.org/docs/latest/Tutorial/chapter_pdb.html)
provides the current comparison surface for atom/residue hierarchy, neighbor
search, HSE, torsions, and fixed-correspondence superposition. Compatibility is
operation-specific: alternate-location selection, coordinate precision,
distance inclusivity, transform orientation, and output ordering are explicit.

### FreeSASA

[FreeSASA 2.1.3](https://github.com/mittinatten/freesasa/releases/tag/2.1.3)
is the current C release reviewed for `sasa`. Its official CLI supports
Lee-Richards and Shrake-Rupley, ProtOr and other classifiers, configurable
probe/resolution/threads, PDB and mmCIF input, and absolute and relative output.
The historical Rust kernels and fixtures remain valuable, but must be
revalidated against the pinned binary and its exact parser/profile.

### DSSP

[`mkdssp` 4.5.8](https://github.com/PDB-REDO/dssp/releases/tag/v4.5.8) is the
current reviewed oracle. DSSP 4 writes annotation into mmCIF by default and
retains legacy DSSP output only when the structure fits. A 98% aggregate
secondary-structure match is not a release compatibility contract; every
declared field and accepted deviation needs exact-build evidence.

### TM-align and US-align

[US-align](https://zhanggroup.org/US-align/help/) generalizes TM-align to
proteins, nucleic acids, oligomers, complexes, circular permutations, and
multiple structures. The initial `align` scope remains monomeric protein
alignment. Broader modes are not implied by accepting PDB/mmCIF input.

The historical TM-align implementation is an algorithm and benchmark seed. Its
current loose test bounds and small related-fold set do not justify an exact
compatibility claim. A release pins the actual current oracle binary, checksum,
mode, input selection, normalization, correspondence, scores, and tolerance.

## Adjacent tools

| Tool family | Decision |
|---|---|
| PyMOL, ChimeraX, Mol* | use externally for visualization |
| Foldseek | use externally for database-scale structural search; do not bundle as a product subcommand |
| MDAnalysis and trajectory formats | separate workflow and data model |
| ProDy normal-mode analysis | separate stateful dynamics workflow if later justified |
| structure prediction and docking | covered by their own domain pages |

No FFI or subprocess wrapper is added merely to make the product appear broad.
An unfinished operation stays absent from public help and release
documentation.
