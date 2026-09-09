# BCF logical record boundaries and BGZF prefix classification

Status: source findings only; consumer regressions are being prepared. No
production repair or compatibility/performance claim. This follows the
separate extended-BGZF reheader repair run `34294546912`; its frozen archive
does not include these new tests.

## Contracts and evidence

The [official VCF 4.5 / BCF 2.2 specification](https://samtools.github.io/hts-specs/VCFv4.5.pdf#page=41),
printed version `e821e4f` dated 2026-02-25, defines the shared record length
over mandatory site fields. Zero shared bytes cannot encode a valid record;
an empty file has no record bytes after its header. The distinction must
survive raw input, compression and frame boundaries.

Pinned Noodles BCF 0.88 `src/io/reader/record.rs:13-15` returns zero when the
decoded `l_shared` is zero. Its `read_site_length` uses the same zero value
for physical EOF and four actual zero bytes. Product `format::Reader` and
reheader's BCF loop currently trust this as EOF. A malformed record can
therefore plausibly truncate processing while permitting success. The result
has not yet been observed in a product process.

Naive concat currently performs a separate complete frame-validation pass,
then typed inspection, then raw copying. Removing its separate frame pass
before fixing this logical EOF ambiguity could additionally skip later CRC,
ISIZE, EOF and trailing-byte checks. Blindly draining after a typed EOF would
not reject the malformed record itself and is not an acceptable fix.

Separately, reheader classifies from the first nonempty inflated frame.
BCF magic split after byte one or two is a legal transport segmentation but
appears liable to misclassification. This requires a positive product test,
including leading/intermediate empty data frames and stdin, before changing
classification. It is not the XLEN/extra-subfield defect already observed.

## Execution plan

1. Freeze tests with both real private consumers: reheader and concat.
   Cover zero-length records at the first/later boundary, with and without
   subsequent records, raw/BGZF input, and valid header-only controls. Named
   destinations must survive failure. Naive preflight must emit no stdout;
   ordinary streaming output does not promise rollback of earlier records.
2. Observe the actual four-native failures before production edits. Keep the
   reheader split-magic case distinct from malformed logical EOF cases.
3. Introduce a reader-private checked BCF record boundary if the hypothesis
   is confirmed. Only actual decoded EOF may end iteration; Noodles zero
   returned after available record bytes is an input error. Preserve partial
   reads and Interrupted behavior. Share this within VCF, not as Layer A.
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
