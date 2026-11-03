# Phylo0.2 source and native delivery

Plan `docs/plans/2026-11-03-phylo-delivery.md`.
Accepted implementation784cef7cd42640880a1d24f35954edea54008fdb.
Owning delivery candidate6624fef40dcffbec47a1ac77f86056bf832f551d is pushed,
clean; actual native4 run41589534278 is in progress, not adopted yet.
No new Rust production behavior or dependency version is changed in this
delivery candidate. Cargo.lock changes only the product0.1→0.2 version.
The new public inference variant is delivered with a0.2 source version,
not by overwriting the immutable0.1registry archive.

## Local candidate and notices

Packet `/Volumes/KIOXIA/Developments/tmp/phylo-delivery-20261103.DngggIJ3`.
Root reads existing seq carrier, complete new helper and actual callers.
Physical-external BED guard verifies USB disk12s1,27GiB free, all tool/cache/
tmp paths and actual resolved external Cargo target before each execution.
The target stays the verified shared `cargo-target/rsomics-bed`; no unverified
override or project path dependency is introduced.

The scratch notice collector consumes actual Cargo metadata and four-target
normal/build graphs, verifies each registry archive SHA256 against Cargo.lock,
and copies111original texts for54unique packages without normalization.
Each target has53enabled packages. `licenses/provenance.json` pins exact
archive/text identities and target membership. Original license whitespace is
preserved with a license-only Git whitespace attribute, not edited away.
The notices scope is Cargo normal/build dependencies, not a broader legal
or dynamic-system-library sufficiency assertion.

Local package64610 exits0:152members,150original files, normalized manifest
and metadata verified; candidate source99090B SHA256
f80e42cb92d5778e948df5dc5d02ed17c6e96b7ebb1775cfa25758da0e594414.
This is explicitly dirty/local candidate VCS784cef7, not a clean release.
Source/install/lifecycle94577 exits0. Fresh cargo install uses the normalized
crate and a new external install root;15calls cover seven operations plus
NJ/UPGMA/WPGMA.13succeed; the NJ-only override and two-taxon WPGMA failures
exit2and1 and preserve the prior output.

The initial whole-delivery review (astra/high, read-only) finds no Critical
or Important code/packaging defect. It identifies a broad receipt label
`full_goldens=True`. Root narrows it to seven explicitly byte-compared output
names plus the separate Hommola selected-field/null-count claim; no larger
test matrix. The reviewed helper executes the same15call lifecycle once
against the installed candidate and exits0. Original candidate package and
bundle precede this receipt-only refinement and remain untouched; clean CI
must qualify the actual final helper, not relabel the earlier candidate.

Bundle/fmt/Clippy/full suite25093 exits0. Debug and release each have17library
and4CLI tests,0failures/ignored, plus both benchmark smoke programs. Ruby's
standard YAML parser accepts the workflow. First staged whitespace check
finds one extra helper EOF blank; root removes that blank before commit.

## Final native gate and retained boundaries

CI uses all four actual native hosts, checks uname/Rust host/target, retains
original fmt/Clippy/docs/package/test/oracle/install/lifecycle logs and actual
source/native carriers. Source package bodies/modes, normalized metadata,
enabled dependency inventory and installed binary provenance are checked
before bundle generation. Linux oracle pins trimAl source
d637091abe33595775f40480970d1a18d87a7bcb.

Review declined actual executions/native artifact completeness/public install/
publication/new-binary performance/older-OS portability. Root closes only the
local executions above. Remaining native, public delivery and release claims
need their actual outputs. No registry authentication retry or publication.

Before any0.2build, root copies and fully compares the old measured executable
to `phylo-wpgma-performance-20261103.sEuLzo9O/measured-native`, SHA256
4a53cffab9503280fbcab521137a8088ed3c849050b43e97f9cf1b79f971f6e3.
The old mutable Cargo output path may now contain another build; the canonical
performance receipt adds the retained original path without changing the
historical execution identity or assigning its measurement to a new carrier.
