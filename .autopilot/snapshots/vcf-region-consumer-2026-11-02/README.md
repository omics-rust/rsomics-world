# VCF shared region-file consumer recovery increment

Root-adopted private seven-file increment, not a source release or whole dirty
checkout snapshot. Apply only over the174-body validated-stream candidate
identified by before-manifestSHA
4b6b695c7438be2c16028e12fe16d48d19c30f3d82e638557725e27603496ee1.
That preceding increment is retained in
`../vcf-naive-validated-stream-2026-11-02/`; neither increment applies to owning
HEAD682942cfa69768dc3a127a8544f2f07213b704ea alone.

The seven patches retain exact externally frozen bytes, including original
diff headers/context. Their destinations are Cargo.lock, Cargo.toml,
src/commands/variant.rs, src/regions.rs, new src/regions/file.rs,
tests/concat_compat.rs and new tests/regions_file_cli.rs. Strip only diff path
prefixes when restoring into a separately admitted checkout; do not stage the
inherited dirty owner wholesale. Source/index/modes and the base must be
verified before recovery.

`concat-compat.rs.patch` includes the one authorized post-review P2 test-stage
correction. The source packet also retains the original failing patch/logs,
review snapshot and correction-only delta; no failed evidence was overwritten.
The six other final patches are byte-identical to their reviewed versions.

Final176-body manifestSHA
2ceca46765c96f7a456010b398fc905424ab6b02b2680e47eae141c93e36fe53;
mode-manifestSHA
c40f15155b5a7168092bd823f490fba50635451da48a1b6b12f8c0dc79973a67.
Exactly5 prior bodies change,169 remain unchanged and2 are added0644; all174
previous modes and the protected reader/naive/concatCLI bodies remain exact.

Staged diff whitespace checks flag13 single-space empty context lines inside
the exact saved patches, plus one context-line EOF warning. These are patch
format bytes, not new source trailing whitespace; retain their frozen identity.
Owning source and tracked Markdown whitespace checks pass separately.

Source evidence:
`/Volumes/KIOXIA/Developments/tmp/vcf-region-consumer-wave-20261102.pAZRxI`.
Final REPORTSHAcb7ca2d9c02c69cd9cf114c4b9a274887af6189ae263a48e3441d0a4305154c4;
72-payload manifestSHAa44c5e9592f8fadc575ba5e9f6adb30b744eae13a11b38407e678185182f3ddd.
One reviewSHA0f530b8acaac38fb0dfc480d3e4dc38caaf0f12e2fbde8d740fcb2cc178ad5ee.
Root reads the report/commands/review/P2 delta, checks all72 payloads and all176
live bodies, and directly reconciles full/explicit test result lines.

Format/strict Clippy pass. Debug/release each452pass/0fail/64ignored; explicit
concat7/filter3/view4 pass each profile, with50 other ignored cases unexecuted.
No scientific recapture, owning stage/commit, public delivery, whole-concat
performance/native-four or pending newline/tied-order acceptance is implied.
See `../../state/vcf-concat-compatibility-2026-11-02.md` for all review rulings.
