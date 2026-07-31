# Survey: protein-structure analysis

Verified 2026-07-31. The canonical boundary and asset dispositions are in the
[`rsomics-structure` dossier](../10-products/structure.md).

## Product boundary

The historical source pool contains nine operation-sized products, not one
thin chain utility. They collapse into one coordinate-analysis product:

| Operation | Historical source | Target |
|---|---|---|
| chain/model inspection, extraction, and split | `rsomics-pdb-chain` | `rsomics-structure inspect`, `select` |
| global geometric descriptors | `rsomics-pdb-geometry` | `geometry` |
| phi, psi, omega, and chi torsions | `rsomics-pdb-dihedrals` | `dihedrals` |
| atom and residue contacts | `rsomics-pdb-contacts` | `contacts` |
| half-sphere exposure and contact number | `rsomics-pdb-hsexposure` | `exposure` |
| fixed-correspondence least-squares fit | `rsomics-pdb-superpose` | `superpose` |
| solvent-accessible surface area | `rsomics-freesasa` | `sasa` |
| DSSP secondary structure | `rsomics-dssp` | `secondary` |
| pairwise TM-score structural alignment | `rsomics-tm-align` | `align` |

The product adopts `pdbtbx` as the candidate external PDB/mmCIF IO dependency.
The existing `rsomics-pdb-core` fixed-column parser has only one target-product
consumer after consolidation and is internalized rather than retained as a
public foundation.

## Upstream coverage

| Upstream contract | Product coverage | Decision |
|---|---|---|
| wwPDB PDB and PDBx/mmCIF coordinate semantics | shared input, selection, and output model | first-release prerequisite |
| Biopython Bio.PDB geometry, NeighborSearch, torsions, HSE, Superimposer | `geometry`, `contacts`, `dihedrals`, `exposure`, `superpose` | first release after current differential and performance gates |
| FreeSASA 2.1.3 Lee-Richards, Shrake-Rupley, classifier and reports | `sasa` | later complete slice |
| DSSP 4.5.8 secondary structure and annotations | `secondary` | later complete slice |
| TM-align monomeric protein alignment | `align` | later complete slice |
| broader US-align modes | later `align` modes | no placeholder surface |
| Foldseek database structural search | none | adopt externally; do not bundle a GPL subprocess |
| PyMOL, ChimeraX, Mol* visualization | none | out of scope |
| MDAnalysis trajectories and ProDy dynamics | none | separate future workflow, not implied by coordinate analysis |

Fixed-correspondence superposition and structural alignment remain separate
operations. DSSP accessibility and torsion fields identify their DSSP method
and do not silently replace the dedicated SASA or torsion contracts.

## Evidence correction

The historical code is useful but its strongest README claims exceed its
release evidence:

- DSSP's live test accepts aggregate residue agreement as low as 98% and skips
  when `mkdssp` is unavailable.
- TM-align permits TM-score deviation `0.02`, RMSD deviation `0.6 Å`, and six
  residues of aligned-length difference against a 20220412 binary.
- FreeSASA has strong committed numerical fixtures, but the Python-binding
  snapshot and current C release need separate version provenance.
- all historical CI is Ubuntu-only and many benchmarks use tiny PDB fixtures.

The reconstructed product retains algorithms, goldens, and benchmark seeds,
then reruns them against pinned current oracles and representative structures.

## Adjacent proteomics scope

Mass-spectrometry search, quantification, structure prediction, docking,
visualization, MD trajectories, and database-scale structural search are not
subcommands of `rsomics-structure`. Their data models, dependencies, licenses,
and user workflows are materially different.
