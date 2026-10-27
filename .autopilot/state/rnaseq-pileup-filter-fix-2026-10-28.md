# Shared pileup filtered-record regression

## Current state

Producer main is now93986e17f4c3a62716b0b6434737c3f46368478a, still
unpublished0.9.1. It adds the actual HTSlib mapped/proper-pair guard to overlap
removal; a placed-unmapped same-name record must not erase a pending mate.
The parameterized regression first fails old code; all27 units, three actual
samtools oracle modes, strict Clippy and format then pass in the clean external
archive. A scoped source review passes. No public item or allocation changes.
Exact-head four-native CI40588249664 is SUCCESS on all four native classes;
earlier1bc989e acceptance (40579417757 SUCCESS) is retained history, not the
new head's gate. Owning main and inherited lock hash are rechecked.

Publish40583646680 at1bc989e is terminal FAILURE: packaging passes but registry
upload returns403 authentication failed. Fresh official registry API still
reports0.9.0 after this failure. Do not claim0.9.1 published or reuse the revoked
plaintext token. The user has been asked to update the existing organization
Actions secret CARGO_REGISTRY_TOKEN without sending a token in chat. Hold
registry publication/adoption only; implementation, reviews and CI continue.
Recheck registry and exact-head CI before any retry with the corrected secret.

Read-only secret metadata check: selected visibility, updated2026-09-08;
failed publisher logs confirm a nonempty masked secret reached the command.
This is not simply an absent environment variable. No value is read, copied,
printed or changed; do not change secret visibility or invent token scopes.

RNA-seq and BAM consumers temporarily pin the exact93986e Git revision for
private verification; replace those Git pins with the immutable registry
release before product publication. The inherited pileup Cargo.lock remains
untracked and byte-identical (0086aa04b6bb4e1d75c7ac1d60410679f94fefd74a04ed8a7570d9bd0ef6530a).

BAM temporary adoption now has5 release mpileup tests passing, including the
new same-name filtered-pair regression; selected lock update changes only the
product version and pileup source. Evidence is
/Volumes/KIOXIA/Developments/tmp/bam-pileup-adoption-20261028.kwLtWn/.
Strict all-target/all-feature Clippy and rustfmt now pass. The independent
BAM hosted-storage guards are committed/pushed at
debc170f44c434089237fc51a8c2c20009cf48ca; exact-head CI40592301741 is SUCCESS
on all four native classes.
That committed concern uses existing registry dependencies; local Git-pinned
pileup adoption/regression/version files are still uncommitted pending the
producer's registry publication. Do not conflate its CI with an adopted release.

The existing30x/250x synthetic engine/partial-BAQ benchmark is refreshed with
one warmup and three measured runs per mode. All commands/counts/process
ownership are retained in pileup-name-expiry-20261028.gFeTN4/benchmark-result.json.
Mac arm64 medians30x .02/.02s and250x .25/.26s (normal/partial); median peak
process RSS2048000/2031616/2146304/2179072bytes. This loaded development-host
projection benchmark is not a paired-cache or upstream-replacement advantage
claim. Performance decision: publish the API-preserving correctness patch
with existing hot-path evidence/no new performance promise; require the actual
complete consumer performance gate before the new RNA product is published.

The1bc989e complete global RNA report removes all original13 TIN/18-bin
differences. Its remaining2 TIN/6-bin differences are a product query-context
contract, not evidence that the shared engine still disagrees with its actual
samtools input stream. Actual per-query source/native equals R; whole-chr10
samtools equals global native. Correct sorted source bounds are confirmed in
rnaseq-qc-remaining-chr10-2_vxlhi5/sorted-bounds-result.json. The RNA-specific
framing implementation now passes the complete original source comparison at
one/four workers locally, with byte-identical reports. Its final93986e boundary
check, new four-native gate and formal performance decision remain separate.
See docs/plans/2026-10-28-rnaseq-query-contexts.md; publication remains held.

## Diagnosis history

The first fix below is real but does not explain the original whole-data
differences. A complete9810963-record Mac debug consumer rerun at the exact
424d157d producer still has the same18 coverage-bin/13 TIN differences.
Raw report SHA4f22e35ddebfc18536e6d15e1af1cce263b83b5ea4e3160e4f3c739f2fdcdba1,
owned process and complete differences are retained at
`/Volumes/KIOXIA/Developments/tmp/rnaseq-whole-pileup-fix-20261028.3wsZIg/`.
All9 consumer tests pass, and a release build completes with binary SHA
69325a596922bd938ddcd312bd0cafe01c8b2f9f55f0f4bff0c1a8bacaea024f.
These are compatibility diagnostics, not performance acceptance.

A small original-data region now reproduces the worst TIN difference without
whole-chromosome context. Original region BAM SHA
5b31083b4bafb840d6db92ddb75ab847b2411a84d14e45ecce463ba72f99e872
and source/native point evidence are at
`/Volumes/KIOXIA/Developments/tmp/rnaseq-qc-worst-gene-i0x54zxh/`.
Native/source differ by−1 across exactly31 positions130519210..130519240.
Only sampled position130519221 differs (native3/source4); scoring the native
depths at the actual source positions reproduces native TIN34.63553779228541.
The six same-QName proper36M alignments start at164/169,184/189,204/209
relative to130519000. HTSlib expires any old proper record by QName, clearing
even a newer pending node's entry. Rust's index-only compaction retained that
newer entry and incorrectly performed one additional mate adjustment.

Actual primary HTSlib1.24 sam.c is retained in that directory, SHA
d7fe7a634234c7d1b00c95055cf64c07b59afb4e5fe119c6c3213d880b822a29.
A bounded source review confirms its M/N cursor is not the difference. The
six-record shifted regression is RED on424d157d (depth2 versus source3).
Logs/clean archive are under
`/Volumes/KIOXIA/Developments/tmp/pileup-name-expiry-20261028.gFeTN4/`, leaving
the inherited owning Cargo.lock untouched.

Producer main1bc989e25a42ce6c0c76e3572b41d4da283e74f2 now matches
name-keyed expiry. Exact-head four-native CI40579417757 is in progress.
Final local27 unit tests and the existing actualsamtools1.24 bytewise oracle
pass all three modes (overlap off/on and ordinary orphan/flag filtering),
as do strict all-target/all-feature Clippy1.91 and standalone Rust1.91 format
checks on all Rust sources. Tests run from the byte-matching external archive;
the inherited owning Cargo.lock hash remains unchanged. A generic cargo-fmt
shim selected an unconfigured rustup instead; no install/default change was
made, and absolute existing Rust1.91 rustfmt completed the format check.

One fresh release
review finds its interaction with rejected-record watermarks: filtering must
not advance HTSlib cache lifecycle. A separate private emission_watermark now
advances only admitted records; last_seen still validates sorted order of
every mapped record. A compact five-record regression and a third filtered
mode in the existing actualsamtools oracle cover that interaction. The scoped
review confirms that issue is resolved without another source/API/allocation
finding. Final current-head native CI, repaired whole comparison and
publication remain pending; earlier424d CI is not this new patch's gate.

Whole RNA-seq run40558683154 failed real coverage/TIN compatibility; its
complete original evidence is retained in rnaseq-qc-real-performance state.
The first diagnosis now demonstrates an actual shared-library defect, not
an accepted relaxation of the RSeQC contract. Whole performance/publication
remain held until the repaired consumer matches all original outputs.

The public pileup0.9.0 engine removes pending overlap state by read name when
any nonzero-span record is filtered. An intervening anomalous alignment with
the same name therefore prevents valid proper mates from combining their
low qualities. The source callback skips that filtered alignment without
removing the pending proper mate. Root reproduces this with three36M reads:
flags99/97/147, starts100/105/110, qualities8, proper-mate template length46.
At BQ13 the source retains one combined-Q16 entry across26 overlap bases;
the accepted native RNA binary loses them. Source100-point coverage sum13,
native0. The complete SAM/BAM/BED/oldbinary/native-output/source-output are at
`/Volumes/KIOXIA/Developments/tmp/rnaseq-overlap-filter-LOkOKo/`.

An independent existing samtools1.24 experiment on the worst real gene
NR_024457.2 yields the original source TIN34.1786444814696 in both its window
and whole-chr12 contexts (maximum emitted locus depth5), not the native
34.63553779228533. This does not establish the cause of every whole-data
row, but weakens a query-context/version-only explanation. Raw owned
process/point-depth evidence is at
`/Volumes/KIOXIA/Developments/tmp/rnaseq-qc-samtools-window-vspsr_ad/`.

One compact foundation regression fails old code exactly: all26 expected
Q16 overlap entries are absent. The worker changes only engine.rs/tests.rs:
filtered nonzero-span records return without clearing pending overlap;
zero-span/unmapped/depth-cap removal remains unchanged. Exact regression,
25 unit tests, one actual samtools1.24 compatibility test and format pass.
Strict all-target/all-feature Clippy also passes using the existing absolute
Rust1.91 cargo-clippy and driver, not ambient Homebrew1.97. No install/cache
deletion is needed.
Inherited untracked pileup Cargo.lock SHA
0086aa04b6bb4e1d75c7ac1d60410679f94fefd74a04ed8a7570d9bd0ef6530a
is preserved and never staged.

Owning main commit424d157d74c4fcf31356a45047cff11f2c21a9eb contains the
small fix, regression, version0.9.1 and external hosted-build storage setup.
Exact-head CI40572161578 is SUCCESS on Linux/macOS x86_64/aarch64, including
debug/release tests on each class and Linux strict Clippy/package/actual
samtools compatibility. Publication remains pending complete consumer checks.
The private RNA candidate temporarily pins this exact Git revision while
registry0.9.0 remains unchanged; restore a registry0.9.1 dependency only after
the corrected producer is published. No path dependency is introduced.

Two concrete consumer regressions are authored in rsomics-rnaseq-qc coverage
and rsomics-bam mpileup. They keep the same filtered/proper-pair quality
contract. No public item or source-policy API is added. Next finish review/
native gates for the small foundation fix, validate the repaired complete
RNA report against retained original outputs, and adopt the patch in both
owning consumers. Do not claim all13 TIN differences repaired before that.

## Prepared probe disposition

The broader13-row context-probe script/workflow/tests/plan were authored but
never dispatched. They are removed from the active control plane after a
complete byte-verified recoverable external copy at
`/Volumes/KIOXIA/Developments/tmp/rnaseq-pileup-probe-prepared-archive-yupwPg/`.
verified-files.json records all four original identities. This preserves a
fallback if repaired whole-data comparison still disagrees, without launching
another broader campaign before testing the demonstrated small defect.
No user file, original source evidence or fixture is deleted.
