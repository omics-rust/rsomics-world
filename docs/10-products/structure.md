# Protein-structure product dossier

Status: boundary, upstream-operation, and historical-source audit complete.
The target repository does not yet exist.

## Portfolio decision

Retain one product, `rsomics-structure`, for coordinate-file inspection,
selection, geometric analysis, solvent accessibility, secondary-structure
assignment, superposition, and pairwise structural alignment.

The nine historical binaries are operation-sized fragments of this product.
They share the same coordinate records, atom and residue identities, model and
conformer selection, spatial geometry, output conventions, and installation
identity. They are modules and subcommands, not nine public crates.

The upstream contracts reviewed on 2026-07-31 are:

| Contract | Reviewed source | Role |
|---|---|---|
| Coordinate formats | [wwPDB PDB format 3.3](https://www.wwpdb.org/documentation/file-format-content/format33/v3.3.html) and [PDBx/mmCIF dictionaries](https://mmcif.wwpdb.org/) | format semantics |
| Rust coordinate IO | [`pdbtbx` 0.12.0](https://docs.rs/pdbtbx/0.12.0/pdbtbx/) | external PDB/mmCIF parser and writer candidate |
| Biopython structure analysis | [Biopython 1.87 Bio.PDB guide](https://biopython.org/docs/latest/Tutorial/chapter_pdb.html) | geometry, contacts, HSE, dihedral, and fixed-correspondence superposition oracle |
| DSSP | [`mkdssp` 4.5.8](https://github.com/PDB-REDO/dssp/releases/tag/v4.5.8) and the [DSSP 4 format description](https://pdb-redo.eu/dssp/about) | secondary-structure and annotation oracle |
| FreeSASA | [FreeSASA 2.1.3](https://github.com/mittinatten/freesasa/releases/tag/2.1.3) and its [CLI contract](https://freesasa.github.io/doxygen/CLI.html) | SASA algorithms, classifiers, and output oracle |
| Structural alignment | [US-align instructions](https://zhanggroup.org/US-align/help/) and the original [TM-align site](https://zhanggroup.org/TM-align/) | current alignment surface and exact-binary oracle |

The GitHub organization has no live `rsomics-structure` repository. The local
source pool contains the nine operation repositories and `rsomics-pdb-core` at
the revisions below. No implementation is published merely because the name is
allowlisted.

## Boundary and operation map

```text
rsomics-structure inspect
rsomics-structure select
rsomics-structure geometry
rsomics-structure dihedrals
rsomics-structure contacts
rsomics-structure exposure
rsomics-structure superpose
rsomics-structure sasa
rsomics-structure secondary
rsomics-structure align
```

| User contract | Target surface | Release decision |
|---|---|---|
| models, chains, entities, residues, atoms, conformers, and format summary | `inspect` | first release |
| chain/model/entity/residue/atom selection and split output | `select` | first release; semantic PDB and mmCIF output |
| mass, center, centroid, radius of gyration, bounding box, and counts | `geometry` | first release |
| backbone and side-chain torsions | `dihedrals` | first release |
| atom or residue contacts under a declared selection and distance rule | `contacts` | first release |
| HSE-alpha, HSE-beta, and contact number | `exposure` | first release |
| Kabsch fit over an explicit atom correspondence | `superpose` | first release |
| Lee-Richards and Shrake-Rupley SASA, classifiers, absolute and relative reports | `sasa` | later complete slice after current FreeSASA differential |
| DSSP 4 secondary-structure and declared annotation fields | `secondary` | later complete slice after exact 4.5.8 differential |
| monomeric protein structural alignment and TM-score | `align` | later complete slice after current US-align/TM-align differential |
| nucleic-acid, oligomer, complex, circular-permutation, and multiple alignment | later `align` modes | only as independently complete US-align-compatible slices |

`superpose` and `align` are not duplicates. `superpose` takes a caller-supplied
correspondence and minimizes RMSD. `align` searches for residue correspondence
and optimizes a structural score. The interface and output keep those contracts
separate.

`secondary` may report torsions and accessibility supplied by its DSSP
algorithm, but those fields are not silently substituted for the independently
configured `dihedrals` or `sasa` operations. Each output identifies its method
and selection policy.

## Coordinate and selection model

PDB is a legacy fixed-column representation with one-character chain IDs and
size limits. PDBx/mmCIF is a first-class input and output, not a later parser
plug-in. The internal model preserves:

- model number and stable file order;
- both author and label chain, residue, and sequence identifiers where mmCIF
  provides them;
- entity identity, polymer type, residue name, insertion code, hetero status,
  atom name, element, serial or atom ID, coordinates, occupancy, B factor, and
  alternate-location label;
- explicit missing, unknown, and inapplicable mmCIF values rather than
  collapsing them into empty strings;
- the distinction between biological selection policy and file representation.

Every operation receives one typed selection plan. The plan declares model,
chain/entity, polymer, atom, residue, hydrogen, heteroatom, and alternate-
location handling. Defaults may use an upstream compatibility profile, but the
resolved policy is recorded in machine-readable output.

The historical tools disagree legitimately:

| Historical behavior | Operations using it |
|---|---|
| keep every alternate position | Biopython-style global geometry |
| choose the highest-occupancy disordered atom | contacts, dihedrals, HSE, and Biopython-style superposition |
| keep blank or a named alternate label | DSSP and TM-align profiles |
| FreeSASA's input selection | SASA compatibility profile |

These become named policies over one model. They are not four parsers.
`pdbtbx` is adopted as an external dependency if adversarial PDB/mmCIF
round-trip tests confirm the required identities and values. Product-local
adapters own rsomics compatibility policy. A thin public wrapper around
`pdbtbx` would add versioning without a second product consumer.

`select` does not pretend that line filtering is structure serialization. PDB
input may offer an explicit record-preserving profile when the selection can be
represented without semantic repair. Normal output is written from the typed
model, includes required container records, and fails when the requested
structure cannot be represented in PDB; mmCIF has no one-character-chain
fallback.

## Internal module map

```text
src/
├── input/
│   ├── model.rs
│   ├── read.rs
│   ├── select.rs
│   └── write.rs
├── geometry/
│   ├── spatial.rs
│   ├── transform.rs
│   └── torsion.rs
├── operations/
│   ├── inspect.rs
│   ├── select.rs
│   ├── geometry.rs
│   ├── contacts.rs
│   ├── exposure.rs
│   ├── superpose.rs
│   ├── sasa.rs
│   ├── secondary.rs
│   └── align.rs
├── cli.rs
├── lib.rs
└── main.rs
```

The spatial index, vector math, residue assembly, mass table, atom
correspondence, and selection logic are product-internal shared modules. They
remain narrow typed APIs rather than a generic geometry framework.
`rsomics-help` supplies the single help, version, diagnostics, completion, and
output contract. `rsomics-common` supplies only product-independent error and
execution primitives.

## Historical asset disposition

| Historical asset | Audited revision | Disposition |
|---|---|---|
| `rsomics-pdb-chain` | `91ee3115f9467ef215962e44d4d232e329d2cb3d` | refactor then merge into `inspect` and `select`; preserve basic fixtures, replace the PDB-only line parser and production `expect()` |
| `rsomics-pdb-geometry` | `efc99601e1f9e6b5c449d491134c262309310441` | refactor then merge into `geometry`; preserve Biopython mass and altloc fixtures |
| `rsomics-pdb-dihedrals` | `91b1d7c6cc8182261d3466dfa227c277be715e95` | refactor then merge into `dihedrals`; preserve torsion definitions and numerical goldens |
| `rsomics-pdb-contacts` | `e79e2ae68bf5b7439228573deefdb4e6bc6bd5eb` | refactor then merge into `contacts`; retain the cell-list kernel and byte-level Biopython goldens |
| `rsomics-pdb-hsexposure` | `2f52a2e42df262ea713190664d5cec0573169076` | refactor then merge into `exposure`; retain CaPPBuilder, pseudo-C-beta, boundary, and integer-count fixtures |
| `rsomics-pdb-superpose` | `b09b5f12e8580d0d95ddb35b3b423fd868a20cfc` | refactor then merge into `superpose`; retain the right-multiplying transform convention and reflection cases |
| `rsomics-dssp` | `723668553d88c0605fd6fc6bfb8371e5bb170aba` | algorithm, fixture, and benchmark seed for later `secondary`; do not carry forward the 98% compatibility gate |
| `rsomics-freesasa` | `53fe8beaa06de9c7124342adc67e415b5c7425dc` | refactor then merge into later `sasa`; retain both kernels, ProtOr data, RSA fixtures, and current-source provenance |
| `rsomics-tm-align` | `5b4bc6b5b2c6093b8ef74a22ee18656b35a17376` | algorithm and performance seed for later `align`; current loose oracle thresholds cannot qualify a release |
| `rsomics-pdb-core` | `7452a78326384fc9307e5234f40cc2362d9fb2be` | internalize parser semantics and adversarial tests; replace its public crate boundary and evaluate `pdbtbx` for IO |

All ten repositories were clean at audit time. Historical Git identity remains
in the merge record; none of the deleted micro-crate repositories is revived.

## Existing implementation strengths

- The historical sources contain real algorithms rather than empty command
  shells: cell-list contacts, HSE, Jacobi-SVD Kabsch, Lee-Richards and
  Shrake-Rupley SASA, a DSSP implementation, and an iterative TM-score
  alignment.
- Geometry, dihedral, contacts, HSE, and superposition have committed real-PDB
  goldens derived from Biopython 1.87.
- FreeSASA has committed absolute and relative-area fixtures, including
  classifier behavior.
- Several kernels already have Criterion microbenchmarks and origin records
  suitable as migration evidence.

## Existing implementation gaps

- Every input is PDB-only. The current DSSP, FreeSASA, and US-align contracts
  accept or produce mmCIF, and modern structures may not fit legacy PDB.
- `rsomics-pdb-core` parses only ATOM/HETATM records from the first model into
  a flat `Vec<RawAtom>`. Each operation then rebuilds another private residue,
  chain, atom, vector, and selection representation.
- Chain IDs are `char`; author/label identity, entities, polymer type, assembly,
  missing mmCIF values, multi-character IDs, and safe serialization are absent.
- Live oracle tests usually return success when the oracle is missing. Release
  CI therefore proves only committed snapshots unless an explicit oracle job
  installs and pins the upstream.
- The DSSP differential accepts 98% residue agreement while its README claims
  approximately 99.96% and describes specific mismatches as acceptable.
  `mkdssp` 4.5.8 is newer than the 4.5.5 reference in the historical claim.
- The TM-align gate permits a TM-score difference of `0.02`, RMSD difference
  of `0.6 Å`, and aligned-length difference of six residues. Its committed
  oracle is TMalign 20220412, and only three related-fold cases support the
  strong compatibility and speed claims.
- The FreeSASA committed absolute-area golden was generated through Python
  bindings identified as 2.2.1, while the current C release is 2.1.3.
  Algorithm, classifier, parser, and output-profile versions need independent
  provenance.
- Benchmarks often use 1CRN or another tiny fixture. The DSSP timing is one
  Apple M2 observation dominated partly by dictionary startup; TM-align results
  are reported from another machine without tracked raw distributions.
- All historical CI is Ubuntu-only. None meets the four-native-platform gate.
- Three operations bypass `rsomics-help`; CLI help, diagnostics, output,
  threads, and failure behavior are inconsistent.
- Source comments are uneven and often narrate the algorithm or audit history.
  Migration retains only public contracts and non-obvious stable invariants.

## First release slice

The first release is a complete coordinate-selection and geometric-analysis
workflow:

1. PDB and PDBx/mmCIF input with explicit model, chain/entity, altloc,
   hydrogen, heteroatom, and polymer selection;
2. `inspect` and `select` with valid PDB/mmCIF output and transactional split
   output;
3. `geometry`, `dihedrals`, `contacts`, `exposure`, and `superpose` over the
   same resolved selection model;
4. TSV and JSON schemas with stable identities, units, transform conventions,
   missing-value semantics, and deterministic ordering;
5. uniform `rsomics-help` behavior and non-zero failure on malformed,
   unrepresentable, or inconsistent input.

`sasa`, `secondary`, and `align` remain absent from public help and release
documentation until their complete oracle and performance gates pass. They
are not placeholder subcommands.

## Compatibility and performance gates

The first release must pass:

1. adversarial PDB and mmCIF fixtures covering multiple models, long and
   colliding chain IDs, author/label disagreement, insertion codes, negative
   residue numbers, alternate conformers, partial occupancy, microheterogeneity,
   modified residues, ligands, water, hydrogen/deuterium, unknown elements,
   missing values, and truncated records;
2. round trips through `pdbtbx`, Biopython, and an independent mmCIF reader for
   every declared identity and numeric field;
3. exact operation-specific Biopython 1.87 differentials, with documented
   numeric tolerances derived from coordinate precision rather than broad
   pass percentages;
4. output-validation tests showing selected and split structures reload with
   the intended hierarchy and no partial files remain after failure;
5. representative small, large, multimeric, highly disordered, and mmCIF-only
   structures, not only microbench fixtures;
6. wall time, CPU time, peak RSS, input/output bytes, version, machine, flags,
   warmups, repetitions, and output equivalence for every hot path;
7. a strict throughput or resource-use advantage over the corresponding
   Biopython workflow on at least one first-release hot path without a material
   regression in the rest;
8. native Linux and macOS CI on both `x86_64` and `aarch64`.

Later slices add:

- FreeSASA 2.1.3 binary differentials for both algorithms, ProtOr and supported
  classifiers, unknown atoms, PDB/mmCIF parsing, absolute and relative output,
  probe, resolution, and thread settings;
- `mkdssp` 4.5.8 exact-build differentials for every declared DSSP field and
  PDB/mmCIF path, with every accepted deviation identified by residue and rule;
- current downloadable US-align and TM-align differentials over same-fold,
  remote-fold, unrelated, length-asymmetric, repetitive, multichain, modified-
  residue, PDB, and mmCIF inputs. Alignment correspondence and scores must meet
  a reviewed scientific tolerance; “different fold” is not permission for an
  unbounded heuristic mismatch.

## Foundation decision and exclusions

No public foundation is added.

- `rsomics-pdb-core` has one target-product consumer and is internalized.
- `pdbtbx` is an external dependency, not renamed and republished.
- vector, Kabsch, torsion, cell-list, and residue-assembly code remains
  product-local until a second named product has concrete call sites and tests.
- `rsomics-common` and `rsomics-help` are the only public rsomics foundations
  required by the first slice.

Explicitly excluded from this product:

- GUI visualization such as PyMOL, ChimeraX, and Mol*;
- structure prediction and docking;
- MD trajectory formats, periodic-boundary analysis, and normal-mode workflows;
- database-scale structural search and a bundled Foldseek subprocess;
- automatic downloads, biological-assembly reconstruction, and file-format
  conversion not required by a stable operation;
- generic molecular-geometry or structure-IO public crates without a second
  product consumer.
