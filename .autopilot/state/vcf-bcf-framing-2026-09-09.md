# BCF logical record boundaries and BGZF prefix classification

Status: expected-red run `34295859204` fails the intended concat/reheader
steps on all four native targets. The Linux ARM assertions were read before
production edits. Reviewed repair `vcf-bcf-framing-fix-2026-09-09` passes
full four-native verification as `34296650518`, world
`cb4fb108147b0518f8e758bf2915d8201cc90850`; control CI `34296576628` passed.
This follows the separately verified extended
BGZF reheader repair `34294546912`; that green archive excludes these tests.

The repair's full artifacts are independently verified: all four ZIP API
digests/sizes/CRCs, 101 extracted files, before/after 174-source lists, exact
heads, native Rust 1.91 and shared dependency identities. Each debug/release
profile passes 445 ordinary tests on Linux or 443 on macOS, with 62 ignored;
all three new reader unit groups pass on every target/profile. Focused concat
38, reheader 21, index 13, selection 11 Linux/9 macOS, resources three and
writer 11 groups pass. Every target rejects all 48 malformed consumer cases,
accepts all 16 split-magic cases and passes 12 header-only controls, with the
unchanged tests enforcing output preservation. Linux x86 passes all 62 pinned
oracle tests per profile, format, strict Clippy, harness syntax and packaging.
Sparse empty-tail queries remain correct; the separately recorded bcftools
`--all` SIGSEGV divergence is unchanged.

Permanent repair evidence is
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/vcf-index-selection-2026-09-09/bcf-framing-fix-34296650518/`.
All 112 retained files (4,501,966 bytes) match external scratch recursively;
inventory SHA-256 is `1f3f11a7008dc13d2dd6ef862808a4d84cfc1d5525bd581b0a82b355fd7ba291`.
This validates the frozen repair, not subsequent worktree edits or performance.
The [indexed BCF follow-up](vcf-indexed-bcf-framing-2026-09-09.md) now records
separate expected-red consumer evidence and a pending indexed repair.

Red world head is `86869341898cf2b4932c7ebd91571c44c6ff39bd`; control CI
`34295758040` passed. First raw assertions are at KIOXIA
`vcf-bcf-framing-first-red-bcCYpy/linux-aarch64.log`. Concat has 37 pass/one
fail: all 24 malformed invocations return success, all 12 named destinations
are replaced and four naive stdout cases emit output. Reheader has 19 pass/
two fail: all 24 malformed invocations succeed/replace output and all 16
legal split-magic inputs fail. Both consumers' six header-only controls pass.
All overwritten outputs are synthetic test assets inside temporary fixture
directories. Full debug/release/oracles were intentionally skipped. Complete
artifact identity and per-target case evidence are independently verified:
all four ZIP API digests/sizes/CRCs, 60 extracted files, 174-source before/after
lists, exact heads, native Rust 1.91, shared lock and 110 external package
identities. Every target independently shows the same 48 malformed successes,
16 split failures and 12 successful controls, including output checks.

Permanent evidence is
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/vcf-index-selection-2026-09-09/bcf-framing-red-34295859204/`.
All 71 retained files (3,392,495 bytes) match external scratch recursively;
inventory SHA-256 is `c0fcdc204a667064ebcbc855e4dbbeac06c7528d4d253e586299c30b84546dd5`.

The red changes only two appended test files across 174 source entries. Concat adds
24 malformed cases and six header-only controls; reheader adds 24 malformed
cases, six header-only controls and 16 split-magic cases. All prior test bytes
are retained. Snapshot README records identities and exact boundaries.

The repair changes four source files and neither CLI test. A shared private
checked BCF read peeks actual decoded EOF, retries Interrupted and rejects a
Noodles zero return after available bytes. Buffered compressed/raw consumers
supply the contract without a public API. Reheader accumulates a three-byte
magic across frames before replaying its full raw prefix. Three new unit
groups cover EOF/zero/partial lengths, helper I/O kinds and short/interrupted
valid reads; all pass in both profiles on all four native targets. Their
helper-level contract does not establish end-to-end I/O classification.

Decoded buffering adds per-reader memory and a per-record buffered check;
measure rather than assume its performance. Raw-prefix accumulation over
arbitrarily many empty frames remains unbounded. The existing top-level input
wrapper still reclassifies some I/O errors; only the helper's kind preservation
is covered by the new unit contract. Naive's current integrity pass is unchanged.
The existing `reheader-vs-bcftools.sh` and 0.5 performance record cover only
plain/BGZF VCF, not BCF reheader. They are also calibrated for macOS time and
`/Volumes` paths. A representative BCF gate on the permitted native runner
must be added before claiming performance for the changed BCF path.

## Contracts and evidence

The [official VCF 4.5 / BCF 2.2 specification](https://samtools.github.io/hts-specs/VCFv4.5.pdf#page=41),
printed version `e821e4f` dated 2026-02-25, defines the shared record length
over mandatory site fields. Zero shared bytes cannot encode a valid record;
an empty file has no record bytes after its header. The distinction must
survive raw input, compression and frame boundaries.

Pinned Noodles BCF 0.88 `src/io/reader/record.rs:13-15` returns zero when the
decoded `l_shared` is zero. Its `read_site_length` uses the same zero value
for physical EOF and four actual zero bytes. Before this repair, product
`format::Reader` and reheader's BCF loop trusted this as EOF. A malformed
record could therefore truncate processing while permitting success, as
confirmed by the expected-red consumer tests above.

The current regression slice covers nonindexed `format::Reader` and reheader.
Indexed concat's `regions::QueryReader::Bcf` and the dependency's indexed-view
query loop also trust zero return values and need explicit chunk/index
fixtures before being included in any fixed-scope claim. Index construction
is different: `index/bcf_record.rs` reads the eight-byte length prefix and
rejects shared blocks shorter than 24 bytes; do not label that path affected
by the same zero-as-EOF ambiguity without contradictory evidence.

Naive concat currently performs a separate complete frame-validation pass,
then typed inspection, then raw copying. Removing its separate frame pass
before fixing this logical EOF ambiguity could additionally skip later CRC,
ISIZE, EOF and trailing-byte checks. Blindly draining after a typed EOF would
not reject the malformed record itself and is not an acceptable fix.

Separately, before the repair reheader classified from the first nonempty inflated frame.
BCF magic split after byte one or two is a legal transport segmentation but
was misclassified in all 16 new positive cases, including leading/intermediate
empty frames and stdin. It is not the XLEN/extra-subfield defect already fixed.

## Execution plan

1. Retain the frozen tests with both real private consumers: reheader and concat.
   Cover zero-length records at the first/later boundary, with and without
   subsequent records, raw/BGZF input, and valid header-only controls. Named
   destinations must survive failure. Naive preflight must emit no stdout;
   ordinary streaming output does not promise rollback of earlier records.
2. Finish all-target raw artifact verification for the observed failures.
   Keep split-magic rejection distinct from malformed logical EOF acceptance.
3. Run full four-native and pinned-oracle checks on the reviewed repair.
   Only actual decoded EOF may end iteration; Noodles zero returned after
   available bytes is an input error. Preserve partial reads/Interrupted
   and the unchanged CLI tests. Share this within VCF, not as Layer A.
4. After that correctness gate, add a typed reader over the validated BGZF
   decoder for naive inspection. Combine structural/inflate and typed checks
   into one full preflight, retain raw-copy pass two, and unify header-boundary
   inflation with the same helper. Do not weaken header/schema/dictionary,
   CRC/size/EOF, output transaction or writer-error contracts.
5. Expand extended-frame, corrupt-frame, short-read, sparse-dictionary and
   writer-fault tests before claiming the two-pass path. Then measure complete
   per-mode throughput, RSS and I/O including many-sample ligation. Decoder
   allocation behavior also remains a performance-review concern.

No new public item, dependency or speculative family is needed. Product HEAD
and index remain unchanged. All scratch is external; boot occupancy above
80% continues to prohibit local Rust/product execution.
