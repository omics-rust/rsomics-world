# Existing grouped-output alias contract correction

Peak's single whole-product review traced a real data-loss defect through
published common0.12.3: two initially absent sibling names differing only in
case could both pass prospective-path checks on APFS and overwrite one another
at grouped publication. The existing Peak alignment-count workflow actually
reproduces the failure; raw review-output-alias-red.log is retained under
/Volumes/KIOXIA/Developments/tmp/peak-whole-native-integration-20261101.JmnuFX/.

Only existing common src/atomic.rs changes. The NotFound branch compares
physical output-parent identity and probes the original two leaf names in an
isolated temporary directory on that filesystem. No guessed ASCII-only fold
or blanket rejection of truly distinct case-sensitive names. Cleanup failures
propagate; existing actual-file/hard-link/symlink/rollback behavior remains.
No new public item, dependency, product boundary or version is introduced.

Actual concrete consumers are Peak OutputGroup::commit/AtomicFile::commit_all
and LIMMA artifact publication/write_atomic_pair. This is source-call-site
evidence, not a claim that pinned published consumers already use new code.
Peak retains its private preflight until normal registry adoption is possible;
there is no unpublished/path dependency or registry version substitution.

Parent reads the full patch/report, applies only rustfmt, then strict all-target
Clippy and the full48-test foundation suite pass locally. The single existing
transaction test now checks both pair/group entry points against the actual
filesystem's case behavior. Formatting's initial unformatted diff is retained,
not presented as a compile or functional failure.

Commit6f760e2fcf2ca11f10306bbfa43dc8aca77de8dd is pushed to owning main.
Exact-head CI41229575777 succeeds: lint/package and native tests on Linux
x86_64/aarch64, macOS x86_64/aarch64. No public release/publish workflow runs.
The registry's immutable0.12.3 is unchanged; a future independent new version
and authenticated publication are required before products may upgrade.

Source packet /Volumes/KIOXIA/Developments/tmp/common-atomic-case-alias-20261101.XIr3op/
keeps original agent diff/report plus actual parent fmt/Clippy/test logs. All
build/target/rustup/TMP/cache paths were verified physically external. Protected
stats Cargo.lock, unrelated world edits and other products remain untouched.
