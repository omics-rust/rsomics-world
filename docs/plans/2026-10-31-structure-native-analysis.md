# Structure: complete native coordinate-analysis workflow

Adopt the read-only contract REPORT.md at
/Volumes/KIOXIA/Developments/tmp/structure-native-analysis-contract-20261031.0cVmk4/,
SHA72292f704098d485ced5aa7c5f79ce2dba9a1240fdcf31419a80e8a64d5e251a.
Parent reads the report, actual retained geometry/PPBuilder/vector/HSE/fit
source bodies and the existing raw spine. Start from private owning main
d7309d563be58e82dd5e943e6372ce29f0607834; no new public coordinate foundation.

The full first-release target remains seven commands: inspect, select/split,
geometry, dihedrals, contacts, exposure and superpose. SASA/DSSP/alignment are
later. The current two file commands are a genuine private slice, not a
replacement for the remaining product. Do not narrow the dossier to what
already exists or advertise an unfinished command.

## Shared identities, selections and read/write capability

Reuse one raw document/source, immutable input-local row keys, model order,
resolved borrowed field handles and compact finite coordinates. Do not copy
full identity maps per atom or introduce a second parser. Private analysis
views group residue/variant/logical-atom slots in source hierarchy order.
Raw author/label/null/literal states stay available for output identities.
Analysis names follow Bio MMCIFParser's label atom/residue names even when
chains/residue numbers use author IDs. Never invent absent author numbers.

Extend the same selection for remaining dossier entity/residue/atom, H/D,
hetero/polymer and conformer controls. Namespace filters address actual raw
fields; require a namespace only when that filter/split uses it. Do not build
a query language or infer CIF polymer status from the standard-AA list.
Use retained entity_poly/struct_asym metadata where needed; PDB has no invented
label/entity namespace. Explicit unsupported selectors fail, not empty stubs.

The same PDB reader must retain ANISOU/CONECT ranges/kinds for read-only
coordinate analyses. Original3nir/1a2p/4hhb/1crn fixtures remain byte-unchanged;
these operations do not consume tensors or deposited bonds. Selection/writing
still rejects unqualified linked-record editing before committing output.
No record stripping, separate permissive parser or deposition-repair claim.
SIGATM/SIGUIJ have no admitted fixture and are not silently qualified.

## Real operation-specific views

Geometry unpacks residue variants and all atom conformers in source order,
without occupancy weighting. Selected-analysis views choose strictly greater
occupancy, first-seen ties, and source last-added residue variant. PPBuilder
uses first compatible N/C altloc pair with f32 Atom-distance<1.8; store its
overrides locally, never mutate the raw or another operation's view. CaPPBuilder
tests any CA-conformer pair at f32 distance<4.3 but calculates from selected CA.

Use the reviewed mass table/Bio inference rules and actual chi definitions;
unknown mass is an explicit mass-analysis error, not carbon/zero. Geometry
coordinates/mass weights and NumPy centers/Rg are f32; total mass sums original
f64 element masses. Rg is the retained team NumPy expression, not a Bio public
Rg API or verified GROMACS result. No generic f64 predicate/reduction replaces
the distinct source precision contracts.

Contacts use selected standard-AA heavy atoms by the admitted default, optional
CA filtering, f32 inputs widened to f64 and inclusive squared-distance cutoff.
Residue pairs deduplicate only after atom hits and remove same-residue pairs.
Keep model boundaries and actual author-number separation semantics.
HSE-alpha/beta use widened-f64 vectors, strict norm<radius and angle<pi/2;
CN uses f32 Atom subtraction/dot/sqrt. Radius scalar conversion follows each
pinned source dtype. Peptide-local offsets and inter-peptide neighbors differ
from numeric-author residue separation. Missing properties are explicit, not0.

Private spatial indexing uses occupied cells/O(n) index memory, conservative
candidate enumeration and exact operation-specific final predicates. No dense
box allocation or unconditional quadratic fallback. Parallelize independent
queries/occupied-cell work with local buffers and deterministic final ordering;
the output itself can be quadratic. Linear geometry reductions keep the source
order/profile; do not schedule one job per tiny3x3 fit or promise scaling.

Proper fit uses explicit equal nonempty point correspondences, no alignment or
serial-keyed guessing. Distinguish raw f64 point inputs from structure-derived
parsed-f32 views explicitly. Compute H=moving.T*reference after centering,
ordered convergent SVD, R=U*Vt with reflection correction and t=ref_mean-mov_mean*R.
Convention is moving@R+t. Accept source-valid one/two/collinear fits; compare
paired transformed points/RMSD rather than an arbitrary underidentified R.
Use only nalgebra0.34.2 std/fixed Matrix3, bounded try_new failure propagation;
no BLAS installation or inherited handwritten SVD. Coordinate-file transform
writing is absent until the raw writer update path is independently qualified.

## Held torsion adapter and fixed comparisons

Implement ordinary PPBuilder phi/psi/omega and named chi mathematics through
the same view, with missing atoms/gaps/termini explicitly represented. Native
public representation of collinear torsion remains a separate unanswered user
choice: source returns -pi plus warnings. A private typed degenerate outcome
does not decide whether the public command will return unavailable or source
value+warning. Keep that CLI adapter absent until resolved, no placeholder or
temporary success/error policy. This holds only that adapter, not other work;
the complete seven-command release remains unearned until it is resolved.

Freeze comparisons before any native analysis candidate executes:

- Exact identities/populations/peptides/pairs/HSE integers/missing classes.
- Geometry f32 centers/Rg: abs5e-5+rel2e-6, distinguishing f32 reduction effects
  from a double-precision rewrite; bbox minima/maxima/dimensions exact at f32.
  Total mass: abs1e-9+rel1e-12. No inherited20ppm percentage gate.
- Defined torsions: abs1e-8+rel1e-10 radians. HSE pseudo-CB angle:
  abs1e-10+rel1e-10. Degenerate policy is not tested as chosen native parity.
- F64 fit RMSD/paired transformed points/full-rank R and t:
  abs1e-10+rel1e-10; orthogonality/determinant abs1e-10. No unique-R requirement
  in underidentified cases. No bands change after seeing a mismatch.

Use admitted source40644150133/corede8e5f1c..., all four actual whole chains,
3nir conformer/geometry profiles, altloc/cutoff fixtures and three fit classes.
Retain all17 historical tables as value/precision assets, not an old CLI
boundary. Record their existing printed quantization before comparison; actual
full-precision observations take precedence. No local scientific imports,
source refit/rerun, fixture repair or new per-helper oracle campaign.

## Execution and handoff

Worker owns this one private product and external evidence only. Parent owns
world/Git/index/review/native4/performance/publication. Existing help/common
contracts, shared ThreadArgs, atomic named outputs and only public run_cli stay.
Preserve failures/source/binary/command hashes and notices; comments remain rare.
Develop with compact integrated workflow checks, then one whole-feature review,
fmt/strict Clippy/complete suite. Strict representative throughput OR resource
advantage and exact-head four-native tests remain later release gates. Reuse
the source KDTree workflow, not artificially slow pair loops, for performance.
Every build/test/dependency operation uses a fresh correct-owner guard, external
Cargo home/target/TMP/toolchain1.91 and bootAPFS<80%. No install/delete/publish,
public item, B→B dependency, historical repository edits or empty future modules.
