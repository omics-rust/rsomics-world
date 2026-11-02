# PLINK VCF UTF-8 record-boundary repair

Status: source integrated as1906a48b8e6e70bf2478c8e3daf1a44c38c56572;
new native/source, performance and publication acceptance remain pending.
Plan: `docs/plans/2026-11-03-plink-vcf-utf8.md`.
Owner: `/Volumes/KIOXIA/Documents/omics-rust/rsomics-plink`.

Actual previous-wave input failure and locked noodles-vcf0.90.0 source show
that read_field validates each I/O chunk independently. A valid multibyte ID
split across chunks errors before LD execution. Root/worker source inspection
agrees on a narrow repair: read one reusable raw record line then feed its
complete slice to the unchanged decoder, retaining builder/header/BGZF paths.

Metadata will retain maximum raw record width privately for reader admission.
Account raw-line, reused-record and serializer transient storage before reader
allocation and during live workspace reporting. PGEN-only prepared-reader
qualification must not be broadened to VCF. Detect second-pass growth before
unreserved allocation, preserve metadata equality checks and invalid-UTF-8
errors. This does not claim to close the preexisting whole metadata-scan peak
budget gap or provide a new public record-size policy.

The implementation worker owns only the planned source/test delta. Root owns
whole-feature review, source integration and new combined native/source and
performance gates. Prior native green11 qualifies499693cf only; scheduling
9cc22cac and this repair still need their own combined qualification.

Actual small consumer RED: plain8KiB boundary and explicitly split BGZF block
both fail incompleteUTF8 at ID index8037, exit101. Retained fixture
`/Volumes/KIOXIA/Developments/tmp/plink-vcf-utf8-d4XX7r`; implementation/log
packet `/Volumes/KIOXIA/Developments/tmp/plink-vcf-utf8-20261103.GbD08J`.
The same regression exercises metadata, literal phased genotype calls,
make/PGEN reopen, CRLF/final EOF and invalid-UTF8 failure.

Root permits the one additional stats::visit_batches pre-allocation estimate
callsite: VCF make filters use it, so it must reserve the newly observed
record buffers before dataset.reader(). No statistical policy change.

## Integration

Frozen five-file diff SHA256
351683792465db123f2e9412bb9f056eb06687fb40d638a1e1ff466f8de441fe.
Worker full debug/release each pass13 library/3 integration tests with no
failures/ignored; strict all-target/all-feature Clippy and fmt exit0. Root
matches all5 source hashes, reads the whole delta and performs guarded full
debug run50543: exit0,13 library/3 integration, zero failures/ignored.
The guard confirms actual external disk12s1,36GiB and all resolved paths.

One fresh whole-feature review reads full files and affected consumers and
finds no actionable Critical/Important/Minor issue. Root rules on its declined
items: native/source and measured performance/publication remain future gates;
conservative workspace estimates are not exact allocator/RSS or universal
budget proofs; existing metadata-first-pass admission and broader source/header
identity limitations are not represented as repaired. No further helper review.

The concrete UTF-8 source defect is repaired and its retained RED/GREEN case
passes locally; the publication hold for this defect still awaits qualification
of the combined source. Next immutable capture is green12, combining metadata,
complete output jobs and this repair without changing scientific graph scope.

Green12 capture is prepared from clean1906a48b, all306 Git bodies and modes.
Root independently replays every archive member against Git and all snapshot
checksums. Archive543618 bytes SHA256
817b2fe649e552ef3df8726ea7ceb8a49e7215746b11b415b68627fe83280ce2;
manifest0d89553fd2879a68b5f9cd5aab8c57838f58e23298307f8df610abcf4537ecfd.
All3 existing preparation tests pass under the external guard. The retained
workflow graph remains123 reports/66 worker calls; new complete Rust population
is13 library/3 integration, and older snapshot requirements remain unchanged.
Exact-head control CI and actual combined native/source execution are next.
