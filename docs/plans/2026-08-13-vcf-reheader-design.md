# `rsomics-vcf reheader` design

Status: approved product design; implementation not started.

## Purpose and boundary

`reheader` is the next `rsomics-vcf` operation and the complete scope of the
0.5 release. It replaces a VCF or BCF header, synchronizes contigs from a FASTA
index, or renames samples without changing the variant payload. It remains a
subcommand of the existing product rather than reviving
`rsomics-vcf-reheader` or adding another public foundation.

The compatibility oracle is bcftools 1.24 `reheader`, its official manual, and
the 1.24 `reheader.c` implementation. The VCF 4.1 through 4.5 and BCF2
specifications remain the format authority. The historical
`rsomics-vcf-reheader` revision
`e25a2942b13b912fefc21e739d3f10876a59ac74` supplies parsing and fixture seeds
only. Its text-only whole-file implementation, direct destination truncation,
standalone help surface, and conditional compatibility tests are discarded.

`setgt` follows in a later release. It has a larger rule and expression
contract and does not need to share a release gate with a structurally
independent header operation.

## Stable command contract

The command accepts one variant input, defaulting to standard input, and
requires at least one edit:

- `-H, --header FILE` replaces the complete header;
- `-f, --fai FILE` replaces the contig set and lengths from a FASTA index;
- `-n, --samples-list LIST` supplies comma-separated sample names;
- `-N, --samples-file FILE` reads either replacement names or old-to-new
  pairs.

The edit order is header replacement, FAI synchronization, then sample
renaming. Later edits therefore operate on the header produced by earlier
ones. The product reserves `-h` for the unified `rsomics-help` surface, so the
upstream header short option becomes `-H`.

`-o, --output FILE` selects a destination and defaults to standard output.
The command preserves the input encoding: plain VCF remains plain VCF, BGZF
VCF remains BGZF VCF, raw BCF remains raw BCF, and BGZF BCF remains BGZF BCF.
Format conversion stays in `view`; `reheader` does not expose `-O`.

`--threads INT` is available only for BCF. A nonzero value on VCF fails as an
invalid configuration instead of being accepted without effect. The ignored
upstream `-T, --temp-prefix` and general verbosity levels are absent.

Global `--json` follows the existing product rule: variant output must use a
named file so the JSON result can use standard output. The summary reports the
input encoding, which edit classes ran, and before-and-after contig and sample
counts. It does not invent a record count for VCF paths that intentionally
copy the body without decoding it.

## Header edits

A replacement header must contain one valid `##fileformat` line and one valid
`#CHROM` line. The fixed columns, FORMAT column, and sample columns must be
structurally consistent. Replacement input is normalized to one LF terminator
per header line and exactly one final LF before the body.

FAI synchronization:

- parses the first two tab-separated FAI columns as contig name and `u64`
  length;
- rejects empty names, invalid lengths, duplicate names, and unreadable or
  empty input;
- updates the lengths of retained contigs;
- removes existing contigs absent from the FAI;
- appends FAI-only contigs in FAI order immediately before `#CHROM`;
- preserves non-contig header lines and retained contig metadata other than
  length.

A sample list replaces every sample positionally and must contain exactly the
current number of samples. A sample file may contain the same positional list
or two-column old-to-new pairs. Pair parsing supports backslash-escaped
whitespace. Unknown source names, duplicate sources, duplicate final names,
empty names, extra fields, conflicting pairs, and attempts to add samples to a
sites-only header fail. This deliberately replaces the upstream warning on a
positional count mismatch with a nonzero error because a header/body sample
cardinality mismatch is not a usable artifact.

## Format paths

Plain VCF uses a raw header path. It validates and edits only the header, then
copies the existing record bytes in order. Record line endings and payload
bytes remain unchanged. Ordinary gzip VCF is rejected with an instruction to
convert it to a supported format because it cannot use the BGZF block-copy
path.

BGZF VCF reads and decompresses blocks only until the first body byte. It
writes the edited header plus the uncompressed remainder of that block,
flushes the new leading BGZF block, and copies all remaining compressed blocks
unchanged, including the existing EOF marker. The existing `noodles-bgzf`
reader and writer expose the required buffered remainder, `flush`,
`into_inner`, and underlying buffered reader; no custom BGZF parser or new
dependency is needed.

BCF records refer to numeric header dictionaries, so neither raw nor BGZF BCF
can use a body block copy. The BCF path parses the original and edited headers,
retains the original numeric indices for structured contig, FILTER, INFO, and
FORMAT IDs that survive, and appends new IDs after the retained maps. It then
streams records through the existing typed BCF reader and writer while
checking that every referenced numeric ID resolves to the same retained name
and kind. Removed definitions still used by a record, moved identities,
invalid sample cardinality, malformed dictionaries, and truncated records
fail before commit.

## Transactions and errors

Named output uses `rsomics-common::AtomicFile` and
`reject_output_alias`. Header, FAI, and sample files are also included in alias
checks where applicable. The destination is committed only after input EOF,
output flush or BGZF/BCF finish, and transaction finalization succeed. A
pre-existing destination therefore survives every parse, compatibility,
compression, write, broken-pipe, finish, or sync failure.

Standard output cannot be transactional, but all edits and cross-header checks
that do not require record traversal complete before the first output byte.
Production errors propagate through the existing top-level exit and JSON
contract. No parse, I/O, dictionary, or finalization error is downgraded to a
warning.

## Product structure

The implementation adds private product modules only:

```text
src/
├── reheader.rs
└── reheader/
    ├── header.rs
    ├── samples.rs
    ├── fai.rs
    ├── vcf.rs
    └── bcf.rs
```

`src/commands/reheader.rs` owns only CLI conversion, common-layer output, and
summary delivery. `header` owns composition and validation, `samples` and
`fai` own their narrow input models, `vcf` owns raw and BGZF body preservation,
and `bcf` owns stable dictionary construction and record checks. Existing
format detection, typed BCF values, `AtomicFile`, alias rejection, JSON exit
mapping, and `rsomics-help` styling are reused at their current boundaries.

No Layer A API is added. Header dictionary preservation and VCF/BCF body
strategies are format-product policy with only one product consumer.

## Verification and release gate

Tests are written before each behavior group. The local gate covers:

- CLI help, conflicts, required edits, stdin, stdout, JSON separation, aliases,
  and nonzero VCF threads;
- header replacement, edit composition order, CRLF input normalization,
  sites-only headers, malformed or duplicate header structure, and empty
  input;
- FAI updates, removals, additions, order, large lengths, duplicates, and
  malformed rows;
- positional sample replacement, pair replacement, escaped whitespace,
  duplicate names, unknown sources, count mismatch, and malformed files;
- plain and BGZF VCF body preservation, ordinary gzip rejection, multi-block
  headers, headers ending at a block boundary, incomplete BGZF, and exactly
  one valid EOF path;
- raw and BGZF BCF dictionary stability, retained and added definitions,
  definitions removed while still referenced, mixed INFO/FILTER/FORMAT IDs,
  contig changes, sample edits, malformed records, and truncated input;
- named-output rollback on read, write, finish, and compatibility failures.

The compatibility matrix compares every declared successful output and every
declared exit decision against bcftools 1.24 across plain VCF, BGZF VCF, raw
BCF, BGZF BCF, named input, and standard input. Expected fail-loud divergences
are asserted separately rather than normalized away. BCF outputs are compared
as normalized header dictionaries and typed records; VCF body preservation is
also checked by raw digest.

The performance gate uses representative large-header and many-sample files.
Plain VCF and BGZF VCF must show a strict throughput or resource advantage over
bcftools 1.24 on the block-copy hot path. BCF measurements record the cost of
dictionary-safe streaming without inheriting the historical dirty-tree 2.88x
claim. Every run records revision, versions, machine, input and output hashes,
flags, repeated timing distribution, and peak RSS.

Publication requires formatting, strict Clippy, debug and release tests,
package verification, the complete pinned oracle, representative performance
evidence, a fresh public-API and hot-path review, and exact-head native CI on
Linux and macOS for both `x86_64` and `aarch64`. Only then may 0.5.0 publish;
`setgt` remains absent from help and documentation until its own complete gate.
