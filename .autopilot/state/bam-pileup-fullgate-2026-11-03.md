# BAM existing pileup adoption candidate

Do not add commands or revive micro-crates: actual BAM already implements37
commands. Owner remainsdebc170f44c434089237fc51a8c2c20009cf48ca with the same
three inherited dirty paths: Cargo.toml/Cargo.lock/tests/mpileup.rs. No owner
edit, staging or commit is performed during this local qualification.

Reuse `rnaseq-pileup-filter-fix-2026-10-28.md`; producer is93986e17 and current
RNA consumer is39f65576, not the older historical RNA head. Two concrete
consumers use the same engine; no new foundation or public API is needed.

Frozen packet:
`/Volumes/KIOXIA/Developments/tmp/bam-pileup-fullgate-20261103.PdkzqR`.
Candidate patch SHA2565a786a7417d1d207e59dc803875c03968ef0d7a9d9a830347f9de01b9f82900c;
source archive1453832B SHA256
7364b2d42c8bc360bbd4585a846a9bd094289e7dff5184fef0aca578a0eef8d1.
494 source bodies and complete three-path delta are frozen; source/index and
producer inherited lock0086aa04...6530a remain unchanged.

Builder records28 successful commands. Full debug/release each have467 passed,
zero failed and64 defined ignored tests. The23 existing explicit CI oracle
commands have101 passing executions/zero failed or ignored; this is not101
distinct cases or proof that every ignored test ran. Fmt/strict Clippy pass.
194 dependency manifests are physically external; actual cached93986 source's
16 Git bodies match producer. These are local gates, not native-four acceptance.

Installed samtools/HTSlib1.24 and bedtools2.31.1 are used. Only two missing
official tag1.24 large_pos DATA fixtures are downloaded after root authorization;
both match existing dossier SHA256. A byte-identical real samtools copy plus
the test's required relative data layout is installed externally, without
wrapper, expected-output change, toolchain rebuild or source-code download.
Cargo stays offline; HTS reference/cache routes are external, inherited
REF_CACHE/REF_PATH absent. A preflight plist-pipe error is retained/corrected.

## Fresh review and integration blocker

One fresh concern review finds no runtime correctness defect, but an important
packaging blocker: Cargo.toml Git+version0.9.1 normalizes to registry0.9.1 when
the unchanged CI executes cargo package --locked. Root's fresh public sparse
index read at https://index.crates.io/rs/om/rsomics-pileup contains no0.9.1;
latest0.9.0 is non-yanked, checksum
beae57811926a88c28ee0de8923dc7edb8d87ffe9c0afba9b47bd771a279f39e.
Thus this is not merely a cargo-publish hold: normal candidate package CI
cannot pass yet. Do not dispatch known-failing CI, remove package verification,
lower the dependency or commit this as registry-ready adoption.

Root ruling: preserve frozen candidate and dirty owner; wait for verified
immutable producer0.9.1 publication, then adopt the registry dependency/lock
and run the unchanged package/consumer/exact-native gates. Do not ask for a
token in chat or retry unchanged failed credentials. Other portfolio work
continues, especially the qualified PLINK candidate's real performance gate.

Review exclusions remain explicit: no fresh BAM four-native/performance/RSS or
complete RNA/call acceptance; no exhaustive re-audit of unchanged commands;
RSeQC-specific split oracle is outside these existing CI samtools commands.
The independent reviewer inspected retained tests, did not execute them.
