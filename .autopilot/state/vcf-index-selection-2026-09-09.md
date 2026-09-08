# VCF index-selection and native-path verification

Status: index selection and byte-preserving path repair pass the full
four-native source-snapshot diagnostic gate, with red/green evidence retained.
No VCF product commit, publication or performance completion is claimed.

## Source and execution identity

The owning checkout remains dirty on
`682942cfa69768dc3a127a8544f2f07213b704ea`, unpublished version 0.6.0. All source
edits stay in `/Volumes/KIOXIA/Documents/omics-rust/rsomics-vcf`. Its Git index
and HEAD were not changed. The four unrelated control-plane untracked files
were preserved.

Physical boot APFS occupancy was rechecked at 94.53%, so no local Rust build,
test, benchmark, package or dependency download ran. The control plane instead
captures immutable, 171-file source archives for manual GitHub-hosted native
CI. Archive SHA-256 is checked before extraction; every source digest is
checked before and after tests. Cargo, rustup, targets and scratch use isolated
runner-temporary directories. The workflow has read-only permissions, no
persisted checkout credentials, no registry secret reference and no publish
step. This is source-snapshot evidence, not product exact-HEAD release CI.

| Snapshot | Control-plane commit | Audit run | Result |
|---|---|---|---|
| `vcf-concat-index-2026-09-09` | `180948d68fbc2af1fa85c37ce198927ba36b1a03` | `34277691071` | Full four-native success |
| `vcf-index-paths-red-portable-2026-09-09` | `5f886722d2e10263c5ff6393058c1fb6253c5144` | `34278711477` | Exact expected assertion failures |
| `vcf-index-paths-fix-2026-09-09` | `8a49a6549787ce738faa357fe9a4e21aa6a0bc84` | `34279567563` | Full four-native success |

The snapshots and per-file manifests are tracked under `.autopilot/snapshots/`.
The first workflow commit `4067bcc5` had invalid job-level `runner` context
references: GitHub rejected dispatch and run `34277278107` failed at parsing.
No product test ran in that attempt. The corrected workflow initializes paths
inside runner steps; this failed setup is not test evidence.

## Original repair: first green evidence

Run `34277691071` passes the seven original index-selection groups plus all
ordinary tests on Linux and macOS, each on x86_64 and aarch64. Every native
profile has 408 passing tests and 61 deliberately ignored oracle tests, in
both debug and release. Linux x86_64 additionally passes formatting, strict
Clippy, benchmark Bash syntax, package verification, and all 61 oracle tests
in each profile across 11 suites. The five existing concat oracle groups pass;
they do not constitute the dossier's still-incomplete concat matrix.

All four artifact API digests and sizes, ZIP integrity, extracted bytes, source
manifests, Cargo.lock, resolved root package, Rust 1.91 identity, source
immutability logs and raw result counts were independently checked. Four raw
job logs were retained. Verified permanent evidence:
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/vcf-index-selection-2026-09-09/baseline-34277691071/`.
Its KIOXIA capture is `vcf-baseline-34277691071-pojPWw`; the permanent copy
matches it recursively.

## Additional path defect and test-first repair

Independent source review found that the resolver newly reused a helper that
constructs sidecar paths through `input.display()`. The helper already broke
non-UTF-8 default index creation; reusing it also spread that defect to indexed
queries, whose previous noodles reader appended suffixes using `OsString`.

The portable red snapshot changes only `tests/index_selection_cli.rs` from the
first snapshot. Linux adds real byte-path lookup and creation fixtures; both
Unix classes add an exact in-memory path-byte assertion and dangling-preferred
TBI rejection with a valid CSI. The Linux query cases rename an already indexed
ASCII fixture and its sidecar, separating lookup behavior from index creation.
An unrelated lossy-name sidecar must not be selected. macOS does not attempt
to create invalid-UTF-8 APFS filenames. An earlier fixture draft that would
have done so was caught in review and never dispatched or committed.

Run `34278711477` fails exactly the three byte-path groups on both Linux
architectures (eight pass), and the filesystem-free byte-path group on both
macOS architectures (eight pass). Logs show byte 255 changed into bytes 239,
191, 189. The dangling-link and original regressions pass. All four artifact
digests, extracted bytes, source checks and exact failure names were verified;
raw job logs are retained. Verified permanent evidence:
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/vcf-index-selection-2026-09-09/red-34278711477/`.
The KIOXIA capture is `vcf-red-34278711477-dVVJzb`; the permanent copy matches.

Only after these observed failures did `default_output_path` change to append
the dot and suffix directly to the input's `OsString`. The fixed snapshot
changes only `src/index.rs` relative to red; all tests and the other 170 files
are identical. No new public item, dependency or policy is introduced.
Independent review approves full diagnostic verification, not release.

As-tested repair fingerprints:

- `src/index.rs`: `d2cc0e7124f18985d11e75d6aaea8cea3bfa03bce8b6191ee23ff8d10ed5c6ab`
- `src/regions.rs`: `2fd2a88b9033ea4a5f271affba58dd1897ff376aac73ff793fe07f0ee2bb023a`
- `src/concat/ligate.rs`: `0043c5e6c11427910835b6c5ae73daa46519581a0e2e4c318fb2428387973d3b`
- `tests/index_selection_cli.rs`: `920a1444679845fe7dce98e6de339712a70c9e5aa3a234ad694d28af4882f41a`

## Fixed source: full green evidence

Run `34279567563` passes all four native jobs. Both Linux architectures pass
11 focused groups and 412 ordinary tests per debug/release profile. Both macOS
architectures pass nine focused groups and 410 ordinary tests per profile.
The difference is exactly the two Linux filesystem tests. Each ordinary
profile still leaves 61 oracle tests ignored; Linux x86_64 explicitly runs all
61 in each profile across 11 suites, with no ignored or empty oracle suite.
Formatting, strict Clippy, Bash harness syntax and package verification pass.

All four API artifact digests and sizes, ZIP integrity, extracted file bytes,
source manifests, before/after source checks, Rust 1.91 identity, root package,
Cargo.lock and raw result counts were independently verified. All four raw
job logs are retained. The immutable red tests are unchanged in this green
snapshot. Verified permanent evidence:
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/vcf-index-selection-2026-09-09/green-34279567563/`.
Its KIOXIA capture is `vcf-green-34279567563-UkdG7e`; the permanent copy matches
recursively. These results replace the earlier pending fix status, not the
remaining concat performance or publication gates.

## Remaining gates

Indexed ingestion and writer repairs now continue in
`vcf-indexed-ingestion-2026-09-09.md`; its observed reds and revised per-contig
chunk-union design supersede the initial region-at-a-time proposal.

1. Do not publish frozen `682942c` unchanged: it retains the preexisting lossy
   default-index path helper. Transfer the fix into a separately verified
   backport or a verified superseding product release. Registry authorization
   is still a separate, unresolved publication gate.
2. Continue the accepted concat repair plan: caller-thread indexed merging,
   many-input thread/descriptor/error evidence, two-pass fully validated naive
   mode, the missing compatibility matrix, many-sample ligation performance,
   and the reheader BGZF extra-field detection regression.
3. Keep BGZF extraction behind the completed VCF/BAM consumer and performance
   gate. No new Layer A crate or speculative API is justified here.
4. Only the final owning-product commit and exact-head release checks can make
   the complete concat slice publishable. Existing source assets remain intact.
