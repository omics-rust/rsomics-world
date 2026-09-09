# Reheader BGZF extra-subfield compatibility

Status: source-reviewed expected-red snapshot `vcf-reheader-extra-red-2026-09-09`
ready for focused four-native execution. No production changes yet.

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

After observing an actual assertion failure, repair the complete private
read path, preserve ordinary-gzip conversion diagnostics and all existing
error/output contracts, then run full four-native and pinned-oracle checks.
Keep BGZF framing product-local; cross-product foundation promotion still
requires concrete BAM/VCF consumer tests and representative no-regression
performance evidence. No local builds or product binaries while the boot
container remains above 80%; all local scratch and evidence are external.
