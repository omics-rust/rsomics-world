# Popgen connected neutral-variation source workflow

Root reads the complete403-line joint dossier,93-line PLAN,83-line REPORT,
585-line READ_BINDINGS and actual provider mask/AA documentation at
popgen-source-preflight-design-20261101.5BrCFq. Parent verifies all97 delivered
files. PLAN SHA c97f05a3f838d2a5cc20f12fd0d01f692586a641471f9e9d727e95bb5c154f4c;
READ_BINDINGS2d2ee140950f9607bb792bb8b24176e64da73ca8e81d644f77613e1c7d6d12e5.
This adopts source-controller/casebook/runtime-lock preparation only. No owner
crate, native implementation, public foundation or scientific result is created.

## Input and scientific contract

Use the complete GRCh37 chr22,1-51304566 inclusive, and every panel-labelled
YRI108/CEU99 person,207 total, preserving VCF order. Provider base is
https://ftp.1000genomes.ebi.ac.uk/vol1/ftp/ . Required objects are:

- release/20130502/ALL.chr22.phase3_shapeit2_mvncall_integrated_v5b.20130502.genotypes.vcf.gz
  (205612353B) and original .tbi (36251B), both bodies still unacquired;
- release/20130502/supporting/accessible_genome_masks/StrictMask/20140520.chr22.strict_mask.fasta.gz
  (627527B), body still unacquired;
- release/20130502/integrated_call_samples_v3.20130502.ALL.panel
  (55156B, SHA b4023dc6ee2d62ee89c8d4d347db4d348e65518d66d346574cdae7a4bbd76858);
- technical/reference/human_g1k_v37.fasta.fai and the original VCF INFO/AA.

P-coded mask bases define project accessibility, NOT per-person callability.
Record all mask codes, exclusions and accessible_bp. AA is the original inferred
annotation; resolve only a unique nucleotide allele match, never substitute REF,
case-derived confidence or a mismatched GRCh38 ancestral library. Preserve raw
AA and reason counts. No true biological-independence/diversity/ancestry claim.

One parsed VCF and checked sample/population model feed all operations. Derive
actual ALT width before read_vcf; do not truncate multiallelic records. The
real diploid profile retains PASS accessible literal SNVs, including multiallelic
SNVs; indels/symbolic/spanning deletions are counted exclusions. Reject invalid
GT indices/partial or non-diploid calls rather than silently recoding them.

Preserve all site pi/Dxy/fixed/Hudson/WC components and AN/AC; full-contig pi,
theta/Tajima/Dxy/fixed; one shared100000bp inclusive window array and complete
window rows; complete folded/unfolded1D+2D SFS; and FST point/SE/vb/vj outputs.
No site-ratio averaging, negative-FST clamping or finite replacement for NaN.
SFS uses relevant complete-call biallelic records with fixed216/198 chromosomes,
no projection and no invented genome-wide invariant bin0. Unfolding additionally
requires usable AA. The source joint-folding and10000-variant jackknife profiles,
including omitted residual jackknife tail versus all-row point estimate, remain
explicit rather than being relabelled physical-block or demographic uncertainty.

## Exact oracle and preparation

Pin scikit-allel1.3.13/f9e17f71b395695d3d99c58f58c76a06c4ad8e12,
NumPy1.26.4, dask2024.9.0 and a complete hashed wheel-only dependency closure.
Use the already bound official CPython3.12.10 standalone authority in the
immutable GCC14.3-bookworm image a689e29bc3adf4663ef9a141d23081252764d1319c63f591a027bd6fd676f4c1;
actual interpreter/import/library checks still must pass. Bind vcftools0.1.17/
1c53c3c73be141103069965403e655536dda9c87 release source/compiler/executable;
configure/make/install only inside the future hosted task prefix, never sudo.
Its named biallelic complete-call derivative/legacy bins remain separate from
the canonical accessibility/window estimators. Preserve all component terms.

Implement one source program and ONE connected engineering casebook/test,
not a helper/reviewer ladder. Preserve complete source outputs/provenance and
GT/count stream identities without a redundant giant raw-human-GT JSON export.
First actual source acceptance uses the whole real input plus that casebook.
First-touch wall is not guaranteed cold I/O; later native/source comparisons
include full parse-to-identical logical exports at1/4 CPU budgets, one warmup
then five alternating pairs, without diagnostic serialization in one timer.

Runtime/data/privacy/finalization capacity must be admitted before dispatch.
Private encrypted hosted retention is being evaluated separately; do not upload
plaintext cohort originals, manifests, exceptions or scientific logs, or require
an unadmitted private host by assumption. The proposed8GiB RAM/16GiB retained
cap must include verified finalization overhead before selecting free-space
admission. No acquisition/build/science dispatch is authorized by preparation.
Small metadata/source or cached wheel downloads may use the external guard;
no scientific import or upstream build runs on the Mac.

Reuse selected old diversity/FST/SFS/core code only after real source acceptance;
internalize policy and duplicate readers, retain useful assets, never revive
micro-crates or change inherited locks/target directories. Existing common/help
remain the concrete foundations. No selection/admixture/PBS/LD placeholders.
