# Close the first-release surface without new statistical features

Adopted from the parent's complete read of the release-scope assessment:
/Volumes/KIOXIA/Developments/tmp/deseq-release-scope-20261028.8snXeY/REPORT.md,
SHA5c104bfc906b07cbd3894c430883c24dfbbc96e18e4051574227fe1a120c50c2.
Base is clean private main ef72e3c1302ad45feb368c4b959661b5de20a120.

The first release supports one coherent CLI workflow: run/results/transform/
inspect, schema5, named-design Wald fitting, normalization/dispersion/Cook/BH/
filtering and fitted non-blind VST/trend queries. LRT/shrinkage/rlog/extra
adapters/renderers and a stable Rust analysis facade remain absent. This is
the existing dossier slice, not a new product or public foundation.

Make all implementation modules crate-private and expose only a documented
binary bootstrap, run_cli. Main calls that bootstrap. Do not hide a still-public
module tree with doc(hidden), and do not invent an opaque analysis API merely
to complete this cleanup. The existing mutable solver/model/cache structs
remain internal. Rehome implementation-inspecting tests inside the crate;
retain true executable lifecycle tests as integration tests reading the public
JSON/table contract rather than importing private types. Preserve all existing
test intent; add no per-helper campaign or blanket dead-code suppression.

The development-only stage_diagnostics example imports internal solver APIs
and is not a first-release command. Retain its exact bytes in external evidence
and remove it from the active product surface; it is already recoverable from
the committed base and immutable verified green6 archive. This is source
cleanup, not deletion of historical fixtures, inputs or user data.

Supply complete project MIT/Apache-2.0 license texts matching current metadata,
reusing the team's existing standard license assets; preserve both locfit
notices. README must describe the intended supported CLI/no stable Rust model
API, but still say private/not release-ready while gates remain. Keep version,
publish=false, dependencies, bundle schema, math, algorithms and CLI behavior
unchanged. No new runtime state/flags/output or product capability is added.

One guarded fmt/tests/strict all-target Clippy and parent full scoped diff
review. Then freeze one final snapshot and run the existing24-workflow four
native gate; the already accepted green6 proof stays immutable. The two-case
current-feature performance harness is prepared but not dispatched. Rebind it
to the final accepted binary BEFORE measuring; this changes no input, source,
numeric/trial/decision contract and avoids timing an intermediate release
surface twice. Registry authorization and final metadata/publication remain
separate; no release is authorized by this plan alone.
