# Index reference workflow implementation plan

> Execution is inline under the adopted source-preflight design. One connected preparation test, then real hosted source execution; no helper review/test ladder. Root reviews and dispatches the exact committed head.

**Goal:** extend the existing index product with coherent FASTA/BGZF→FAI/GZI→ordered multi-region FASTA retrieval+SAM dictionary, qualifying the real source contract before native implementation.

**Architecture:** keep resource preparation/access in `rsomics-index`. Reuse common/help and existing product BGZF framing; new scanners, FAI/region/query/dict policy remain product-private. No new owner, product boundary or public foundation API.

**Current delivery:** source casebook41255333228 and optimized owner030be20895cb7806f78b2923e1c1ac3afa05cd96 exact four-native CI41272452578/1 are accepted. Fresh unchanged full-reference run41273328984/1 now passes correctness and both predeclared throughput/scoped process-memory routes after complete original recovery; its distinct accepted receipt is index-optimized-reference-real-performance-2026-11-01.json. BGZF serial bulk still loses7.3388%; all CPU/scaling losses remain visible. Baseline owner7f2d4fd throughput failure/memory benefit stays historical and is not borrowed. Publication and the older missing-raw gate remain pending. No local samtools execution occurred.

## Fixed source and behavior

Target samtools 1.24 (`dc71c7274044d1050ccb64901731373ec7e915b6`) and HTSlib 1.24 (`4b705e4fada8ee2b6b15746f725ee8ac51631803`), not latest. SAMv1 reference is `c101c797106d8a1006b9b06f75e8bcc6062cf81e`.

Primary bindings:

- [samtools faidx manual](https://www.htslib.org/doc/1.24/samtools-faidx.html), [dict manual](https://www.htslib.org/doc/1.24/samtools-dict.html).
- [fixed faidx CLI source](https://github.com/samtools/samtools/blob/dc71c7274044d1050ccb64901731373ec7e915b6/faidx.c), [dict source](https://github.com/samtools/samtools/blob/dc71c7274044d1050ccb64901731373ec7e915b6/dict.c).
- [fixed FAI source](https://github.com/samtools/htslib/blob/4b705e4fada8ee2b6b15746f725ee8ac51631803/faidx.c), [format manual](https://github.com/samtools/htslib/blob/4b705e4fada8ee2b6b15746f725ee8ac51631803/faidx.5), [BGZF source](https://github.com/samtools/htslib/blob/4b705e4fada8ee2b6b15746f725ee8ac51631803/bgzf.c), [region parser](https://github.com/samtools/htslib/blob/4b705e4fada8ee2b6b15746f725ee8ac51631803/hts.c#L4003).
- [SAMv1 names/MD5/BGZF](https://github.com/samtools/hts-specs/blob/c101c797106d8a1006b9b06f75e8bcc6062cf81e/SAMv1.tex).

The public command tree is `faidx build`, `faidx query`, and `dict` inside existing index. FASTA only for the initial indexed profile; FASTQ/fqidx, reverse complement/strand labels, alternate-locus dictionaries, compressed query output/output indexing and remote discovery remain excluded. Ordinary gzip is supported for sequential dict, not indexed random access.

Preserve first-token names/descriptions, sequence case, input order, LF/CRLF physical widths, checked wide logical byte offsets and five-field FAI. Names must be nonempty and unique; dict additionally needs SAM-safe SN. No lossy text replacement or donor saturating subtraction. Indexable sequence lines have consistent stride except the short final line; dict MD5 has no line-width restriction. SAM MD5 strips bytes outside ASCII 33–126 and uppercases ASCII, not just whitespace stripping.

CLI regions use source 1-based inclusive→internal half-open ranges, whole/open-ended ranges, comma thousands separators and colon/braced-name disambiguation. Preserve literal request strings as output headers, region-file requests before positional requests, order and duplicates. Default wrapping follows input LINEBASES, not a universal 60/80; `-n 0` is unwrapped. Source observations qualify final-no-LF FAI width rather than silently fixing it. Native explicit build/query will require existing indexes during query, unlike source implicit FAI_CREATE; no hidden query writes. Explicit zero/malformed/overflowing coordinates reject rather than saturate; source clamping/warnings for well-formed out-of-bounds positive intervals remain.

New faidx reading accepts valid empty BGZF members and embedded EOF members; physical EOF ends the stream. Keep canonical writing. Existing index `FrameTracker`/`GziIndex::scan` incorrectly treat the first EOF as terminal, and `GziIndex::read`'s compressed-gap≤64KiB assumption rejects legal intervening empty members. Reuse BC parsing and CRC-backed decode, not those strict policies. FAI construction decodes once while collecting logical offsets/nonempty GZI anchors. Current code's static compatibility gap is recorded separately from its old bgzip/tabix scope/evidence; no broad rewrite is authorized by this source-only step.

`dict -A` uses chr/mitochondrial name combinations, not donor ordinal numbers. Preserve HD/tag order and exact explicit URI; default file URI is canonical path, stdin gets none. Retain the source's actual diagnostics and return codes, including negative observations, without defining failed calls as expected success or changing policy afterward.

Named native outputs will use common recoverable same-parent staging, force/path-alias checks and `AtomicFile::commit_all` for FAI/GZI. This is not crash-atomic publication of an entire set. Query streams bounded chunks and ordered worker output, not a whole-chromosome Vec. Common `ThreadArgs` is local to query (total workers; source `-@` counts extra); no fake parallel dict/build flags. Help and JSON output reuse existing index/common rules; JSON data output requires a named destination.

## Actual historical routes and foundations

- fasta-index donor `fb86ed124f069141b2934641f1291a8adc83a4be`: refactor FAI/region/fetch intent; retain tiny two.fa/goldens as tests; discard whole-file/lossy parsing, per-region reopen, unsafe coordinates, duplicate dict and invalid benchmark invocation. FAI formatting/offset equation are small direct seeds.
- bam-dict donor `e3dd9a007965f9dd741882690d973b9e51044bf3`: direct seed is bulk-buffered MD5 update; refactor scanner/options/tag ordering and replace whitespace-only filter; retain fixture/option matrix; discard ordinal aliases/direct output/permissive names and tiny-fixture speed claims. Preserve its inherited untracked lock.
- Existing common call sites: index `src/bgzip/workflow.rs`, rnaseq-qc `src/output.rs:60–87`, liftover `src/cli.rs:205`; existing help parsers in index/liftover; ThreadArgs already in rnaseq-qc. No new foundation items.
- Actual seqio `IndexedFasta` has call `src/run.rs:493–501` consumer but is plain File/default-index only, without explicit FAI/BGZF/raw-offset scanning. Its public readers collect complete records. Do not expand it solely for this new contract. Ordinary-gzip dict can privately adapt existing external flate2 1.1.9/zlib-rs; this is future owner work, not this source controller.

## This source-preflight implementation

Files: `scripts/index_reference_source.py`, `.github/workflows/index-reference-source.yml`, `scripts/test_index_reference_source.py`, this plan. Reuse existing process/download/hash/JSON wrappers rather than adding a controller framework. The script prepares inputs using only stdlib on Mac; compilation and source commands require the real manual Linux x86_64 GHA environment.

Runtime is immutable GCC14.3 Bookworm digest `a689e29bc3adf4663ef9a141d23081252764d1319c63f591a027bd6fd676f4c1`, with actual image config/compiler/ELF/library/package observation. Official samtools 1.24 archive: 9,491,012B, SHA `89b2a440123eeaa400392ce1736e7d60ce9041843027d76819753c5a8246bfdd`; HTSlib: 5,004,265B, SHA `28a8de191381c7a97a35675ceac76fa1ea95e7b678d6a2e9d600a7874e4077de`. Admit exact official release API/body metadata and fixed core-source anchors before building.

Build official libdeflate 1.25 source (186,474B, SHA `fed5cd22f00f30cc4c2e5329f94e2b8a901df9fa45ee255cb70e2b0b42344477`) using portable official CMake3.31.8 (55,012,213B, SHA `630615d8e98ac33eba7fbe472626dff5c899c85af3c024585ae109166a6909d0`). No apt/sudo/system install. Use a session prefix, libdeflate's original CMake options, HTSlib `--with-libdeflate`, and samtools `--without-curses --with-htslib=VERIFIED_SOURCE`. Admit actually loaded libdeflate and versions/configure output before science. Official source/binary archives, build logs, sources, machine/ELF/library identities, all calls and outputs stay in a fresh RUNNER_TEMP tree.

One connected >64KiB casebook includes exact team donor inputs and a >262kb multi-contig reference: wrap101, LF/CRLF, final-no-LF, mixed case, colon/braced names and all four M/MT/chrM/chrMT names. Source builds FAI/GZI on plain and actual source-bgzip data, queries default/wrap0/four-worker and explicit source sidecars, reads valid empty/embedded-EOF BGZF, produces dictionaries/metadata flag matrix and records a few malformed/missing/CRC observations. It cross-checks equivalent source representations and source sidecars; no native code exists yet. Inputs/outputs are authored synthetic source fixtures, not provider reference or clinical data; ordinary public artifact retention is sufficient.

The job is manual-only, contents-read, four CPUs with a bounded memory allocation, and no source data beyond small engineering fixtures. Preserve original exceptions/nonzero results; always finalize/upload available originals without converting an earlier failure to green. A sibling TAR preserves executable modes and symbolic-link metadata instead of uploading a flattened runtime directory. Capacity admission is scoped to this small runtime/casebook, not the future real-reference budget. Record limits honestly; no global RSS or hard-loss/timeout zero-loss claim.

- [x] ONE preparation test RED: missing controller was the observed intended failure.
- [x] Implement source-only controller/workflow; the same preparation test passed without local upstream execution. The final delivery report records the exact verification and SHA identities; source runtime/casebook are not yet run.
- [x] Root review/commit/exact-head control CI and actual hosted source41255333228; full original verification is recorded in index-reference-source state.
- [x] Inspect actual source output, then authorize the coherent native merge. All54 selected regular source fixtures and strict-input differences are recorded; no post-hoc scientific relaxation.
- [x] Root whole-slice review and one independent review; correct the observed gzip stdin short-read failure. Commit/push owner7f2d4fd; exact four-native qualification remains pending.

## Later complete real measurement, not run here

The full provider v37 reference candidate has actual HEAD 892,331,003 gzip bytes; pinned provider FAI has 84 contigs and 3,101,804,739 bases (SHA `04aa12dcb4e7131d9c7361082bfd02f8415ee11e956bf358a339f0d3c54180ab`). Its last offset implies ≥3,153,506,518 plain bytes, not measured decompression. Actual body SHA/size/compression flavor/terms remain admission work. No provider body is acquired now.

Later use that complete reference without tiling/subsampling, identical plain/BGZF bytes for both implementations, acquisition/hash/preparation outside both local-access timers. Time full FAI/GZI build→10,000 dispersed 1kb plus boundary regions→complete chromosome1→plain/BGZF dict, fresh processes and retained outputs. First observed is first-touch, not proven cold; one warmup and five alternating paired trials, plus four-total-worker query comparison. Record exact outputs, per-process wall/CPU/RSS distribution and full input/source/binary/compiler/backend identity. Require a strict useful throughput/resource advantage; no inherited micro-crate or missing historical index evidence claim.

Owner baseline was clean `41b161a7dac7eb3700208f025a6be6005c917002`. Parent has now completed the narrow external guard/doc repair at clean `c7a8f8d52c00feb405a5c968f48d425668dd3219`; actual new `scripts/guarded-cargo.sh` was read, with physical mount/device/free-path checks, offline locked Cargo target resolution and override rejection. Parent reports exact four-native CI `41253010155` success. No Rust/Cargo.lock changes were part of that repair. Keep old 13-lane harness identity and missing-raw/publication hold intact; this source-only preflight does not renew them. All Mac project files/cache/tmp remain physically external; unrelated boot occupancy is not a stop. No cleanup, new public API/crate, native build, registry action or publication is part of this delivery.
