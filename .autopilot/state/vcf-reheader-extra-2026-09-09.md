# Reheader BGZF extra-subfield compatibility

Status: expected-red run `34293512685` fails the intended reheader step on
all four native targets. Source-reviewed repair snapshot
`vcf-reheader-extra-fix-2026-09-09` is ready for full four-native execution.

World red head is `540a8094ef6b724a074f722cec4c9473bc8d2462`; control CI
`34293413259` passed. The first Linux ARM assertion was read before source
edits at KIOXIA `vcf-reheader-first-red-ViSxVG/linux-aarch64.log`: 18 groups
pass, one fails, all 36 legal cases report ordinary-gzip rejection. Artifact
identity and complete per-target evidence are being independently audited.

The repair changes exactly four source files. Existing frame parsing owns
the BC/XLEN contract; the classifier no longer requires a six-byte extra
field. A bounded flate2 single-member inflater checks CRC/ISIZE, decoded
size and unconsumed member bytes. A private decoded reader replaces the
compressed passthrough for BCF so no fixed-header Noodles BGZF decoder remains
in this reheader path. VCF header inflation shares the helper. All CLI red
test bytes are unchanged; six additional unit groups await execution.

Review found no remote-verification blocker. Per-frame decoding/buffering
changes need performance evidence. VCF tail CRC checking is not broadened,
and canonical EOF enforcement still depends on consuming BCF to actual EOF.
Separate source-only follow-ups are tiny-frame split magic and malformed BCF
early logical EOF; neither is claimed reproduced or fixed here.

The August concat audit item 7 observed an inconsistent BGZF boundary:
`format/bgzf.rs` parses extra subfields structurally, while reheader detection
requires XLEN=6 and `BC` first. Pinned SAMv1 BGZF specification explicitly
allows other subfields around `BC`. Reading Noodles BGZF 0.49 also shows
fixed-18-byte header/XLEN=6 assumptions in its frame decoder; correcting the
classifier alone cannot close this bug.

The candidate preserves all 17 prior reheader CLI tests, then adds 36 valid
cases and four malformed guards in two test groups. It covers VCF and BCF,
BC-before/after/both extra placement, optional leading noncanonical empty
frame, file/FD0 stdin input, and BCF zero/two compression workers. Independent
gzip decoding checks fixture payload preservation; decoded-format magic,
sample names, complete variant body, JSON encoding and named-output
transactions are asserted. BCF headers cross a frame boundary. Guards that
pass only because the old classifier rejects every extended header do not
prove downstream CRC/subfield validation; preserve them through the fix.

Source snapshot has 174 files and differs only by appended reheader tests
from the corrected sparse-oracle candidate. That other full run is
`34293201835`, world `e2f13662f2a19c0c308274f39b717bbf46e16319`, with
control CI `34293102709` passed. Do not confuse its test-only empty-index
layout correction with this reheader behavior change.

After the repair's exact-head control CI, run full four-native and pinned
oracle checks, including ordinary-gzip diagnostics and error/output contracts.
Keep BGZF framing product-local; cross-product foundation promotion still
requires concrete BAM/VCF consumer tests and representative no-regression
performance evidence. No local builds or product binaries while the boot
container remains above 80%; all local scratch and evidence are external.
