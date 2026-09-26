# VCF final CI receipts (evidence preservation only)

No product source, test, build, rerun, dependency, publication, or repository
state was changed for this receipt. The two 2026-09-09 runs are distinct
snapshots; green indexed-BCF evidence does not cover the later naive-BGZF test.

| Snapshot / run | Exact world head | Terminal outcome and actual coverage |
|---|---|---|
| Corrected sparse-indexed-BCF fixture `34299936099` | `5e300ee1e0309beaf1cf783fc5415386f6183cb5` | Success on Linux/macOS x86_64/aarch64. All four jobs completed focused groups and debug/release suites. Library: 282 passed in each profile/target. Focused selection: 12 passed on Linux, 10 on macOS; raw matrix: 16 malformed indexed BCF commands rejected and eight positive controls accepted per target. Focused concat 38, index 13, resources 3, writer 11, reheader 21 passed per target. Linux x86_64 alone completed formatting, strict Clippy, package, and pinned compatibility-oracle steps; those steps were intentionally skipped on the other three targets. |
| Naive extended-BGZF expected-red `34300381682` | `57a0ca26dd8fa3bb31f5875ed72d21f1a7fca46a` | Failure on all four native jobs, solely in focused concat CLI (38 passed, one failed per target). The appended group completed 32 legal extended-frame cases and four canonical controls per target: all 32 legal cases were rejected as invalid BGZF, while the four controls succeeded. Other focused groups passed. Debug/release, static/package, and oracles were intentionally excluded by the regressions-only dispatch; this is **not** a full-suite green or a repaired implementation. |

GitHub Actions run and jobs APIs were checked after terminal completion. The
green pre-existing capture and newly downloaded red run JSON, jobs JSON, logs,
and four original artifact ZIPs are preserved at:

- `/Volumes/Zane's HDD/rsomics-fixtures/evidence/vcf-index-selection-2026-09-09/indexed-bcf-fixture-fix-34299936099/`
- `/Volumes/Zane's HDD/rsomics-fixtures/evidence/vcf-index-selection-2026-09-09/vcf-naive-extra-red-capture-34300381682/`

Each durable tree recursively matches its KIOXIA scratch capture. All eight
artifact ZIPs and the red run-logs ZIP pass `unzip -t` CRC checks. ZIP entries
match extracted file counts (green: 17/50/17/17; red: 15 each); every extracted
`files.sha256` matches its ZIP member. Each of the eight manifests contains
174 source paths, and all pre/post-test source-verification logs report 174
`OK` entries and match byte-for-byte. The source tarballs themselves are not
inside the downloaded CI artifacts: the raw job logs show GitHub's successful
SHA-256 checks of the source tarball and manifest before extraction, followed
by the 174 per-file checks. Thus the tarball checks are CI-observed, not a
local rehash of retained tarballs.

| Run | Snapshot tarball SHA-256 verified in CI | Retained manifest SHA-256 |
|---|---|---|
| Green | `fc9dcba16792b8496fa3d36ecb1334aa09fb6bd1a6df28909e9c25e216cd92b2` | `1375e24d03f0aa6c274f54e608c6f3a525161bfcc7bcaf9a5301f9c944714b77` |
| Red | `d183f02759bb34d75dfa63e4260828d8f21b772306c5cf6e67e5b6849be8d328` | `7d5649d0b15ff6f4e120d811eb90c9e8820f2926073e43bb633bffe54e38a9b8` |

Original artifact ZIP SHA-256, in Linux aarch64, Linux x86_64, macOS aarch64,
macOS x86_64 order:

- Green: `10a5dec4ce0ba8b6bdd085b63f684d70d925fc854330fbccad9ce35ec69d75ff`, `eec5f1286d6bc3da9636ddb380f620d5dc56340fdd92b12fa3cf38ff9e6d1105`, `345947680ae5c790246d6d0555d31dc29efdaac46afeb0c69af6b9e2f6ac8d99`, `0f2d0fffc7d1516f13438f63561f2ab79ec940dc153865bd0764f0a62628d0df`.
- Red: `3958aa208ad05ea41bef78e5b0a0d652fa22eab416c0d27620503fc92190ff8d`, `d9a7cd5a36330f3b6ed131208fef106f99ef11adb821761d1e58252f010f0cbd`, `1978eca2e8cabac1079562bb31ef1df23f8e72912cb23af4b90b92c8c338f293`, `9ea521701cd6499b4530ae81d393d082895da425552a1451ec7a5d8ba7dc455c`.
- Red raw run-logs ZIP: `0a3c1ef1100ed19932e88689ddcdb7b180f4e90879c224e63bde9b3381c16187`.

Raw red assertions are in each artifact's `concat-cli.log`, with the complete
GitHub job logs retained separately. The reported rejection is consistent
with the fixed-header inflater source finding; it cannot by itself isolate
the separate large-header magic-sniff issue. The green indexed slice uses one
contig/chunk/frame; neither run resolves broader indexed cases, naive
two-pass design, newline policy, performance, or publication gates. Keep the
inherited dirty `vcf-naive-preflight-2026-09-09.md` untouched.

The controller independently rechecked both retained run heads, all eight
artifact SHA-256 values and ZIP CRCs, the eight 174-path source manifests, and
the four green/four red raw concat summaries on 2026-09-26. All matched this
receipt. The local inspection did not execute product binaries.
