# Survey: proteomics / structure domain

Verified 2026-05-30. **Honest assessment: this is the THINNEST domain** — 1 crate exists
(`rsomics-pdb-chain`). Mass-spec is out of scope; structure tools are a small tractable set.

## Mass-spec peptide search → OUT OF SCOPE
Comet / MSGF+ / MaxQuant / FragPipe(MSFragger) / DIA-NN — all require specialized spectrum
IO (mzML/mzXML/RAW/WIFF) + scoring engines (cross-correlation, matched-ion, decoy-FDR) that
are multi-person-year projects with no viable Rust path. Correctly classified adopt/skip. No
rsomics crate should be planned before a hypothetical Layer-A `rsomics-spectrum-io` +
`rsomics-ms-score` foundation exists (not on the horizon).

## Structure analysis — the tractable set

| op | crate | status |
|---|---|---|
| PDB chain extract/split (ATOM/HETATM chain field) | `rsomics-pdb-chain` | ✓ (bespoke line parser, not pdbtbx; no mmCIF) |
| PDB/mmCIF parsing foundation | — | gap → adopt **pdbtbx** (MIT, pure Rust ①, confirmed 0.12.0) |
| DSSP secondary structure (helix/sheet/coil) | — | gap (BSD-2 upstream, ~3k LOC — **P1 tractable**) |
| FreeSASA solvent-accessible area | — | gap (MIT, geometric kernel, rayon-parallel — **P1 tractable**) |
| TM-align / US-align (TM-score structural align) | — | gap (~3k LOC clean-room — **P1**, only structural-compare gap Foldseek doesn't cover) |
| Foldseek (3Di structural search) | adopt subprocess | adopt (GPL-3 + hand-SIMD; don't rewrite) |
| ProDy NMA/GNM/ANM dynamics | `rsomics-prody` (planned P2) | gap (ndarray+nalgebra eigensolve; niche) |
| MDAnalysis MD trajectories | — | out of scope (chemfiles FFI is the adopt path) |

**Highest-value tractable adds** (each <1 week): DSSP, FreeSASA, TM-score. Once a pdbtbx-based
`rsomics-structure` foundation exists, rework `rsomics-pdb-chain` to sit on it (gains mmCIF).
Realistic domain ceiling ≈ 5–6 crates.

## Verification notes
docs/07-proteomics-structure/*.md planning consistent with above; pdbtbx 0.12.0 confirmed via
cargo search. Mass-spec out-of-scope confirmed in mass-spectrometry.md.
