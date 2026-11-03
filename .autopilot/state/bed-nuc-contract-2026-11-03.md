# BED nucleotide-profile source contract

Source inventory and oracle evidence only; no native nuc implementation,
dependency change, performance qualification or public command is claimed.

## Existing shared reader

`rsomics-seqio::IndexedFasta::{open,len,fetch}` is already public in0.6.0.
Root reads the complete adapter and concrete CALL consumer at
`rsomics-call/src/run.rs:493` onward. The cached registry archive SHA256
18923c9db5cb7ee3429681c889fd40a5c207a9f0782ae5859e5446d58eaa817d
has clean VCSbf8c2c8eac4e8f44907587527bfd5f7f808de97e. Adapter source in
archive and clone both hash
dca3ba9dee1cb8208f91914edd32eb936629f6a32ec467ed5095ed4cc84566c8.
Fresh public sparse-index lookup confirms the same checksum and not-yanked0.6.
CALL is the current concrete consumer; BED nuc/getfasta is a concrete next
consumer, not justification for another public reference crate or speculative
API. The adapter uses noodles' buffered-reader enum, which supports plain
files and indexed BGZF; this is separate from sequential gzip support.

## Actual source behavior

Root reads complete BEDTools2.31.1 nuc core/Main and shared Fasta.cpp from the
705ccfdf archive SHA256
1db4e57c1899aede13ef516c61c9b5819cf2790994a556dbb28972488f9946e6.
Core69e00c75073c7aeef3120e26ad51dfff1073bdeca8c12c7416f9e516b7da5f8e;
Main86c18e89827614d9ecccb90db22c01f9b1c8bc9fdfa2e1ddbcc16ab4fc0711ad;
Fastae39149d85ae2c3e511a202350c58611f530a2f101d1bf6d152d42689e8592401.

One operation reports original BED plus AT/GC fractions, A/C/G/T/N/other counts
and length, optionally extracted sequence and literal overlapping-pattern count.
Strand mode reverse-complements only explicit minus; other/missing strands use
forward sequence. It is not BED12 block splitting or RNA-output mode. U/u
counts as T; IUPAC/unknown bytes count as other unless one of those base classes.
Percent denominator includes all bases, binary32 division printed to6decimals.
Pattern matching is case-sensitive by default, optional ASCII case-insensitive;
empty pattern counts sequence positions, not zero. Keep input order/duplicates.
The column header reflects original field count and sequence/pattern options,
appearing before processing the first valid BED row, even if it is skipped.

Points are explicitly skipped with diagnostics, not expanded for profiling.
Missing chromosomes and reference-overrun rows are intentionally warned/skipped
with exit0. File/parse/index/query errors remain separate fatal conditions.
`fai_load` auto-creates an absent FAI. Shared Fasta.cpp stores fullHeader but
uses ordinary fai_load/name lookup; do not promise full-header lookup from help.

Historical nuc asset3e20a5e4f659 is refactor-then-merge: retain useful pattern,
IUPAC complement, formatting and fixture logic after source comparison, but
replace private FAI/whole-reference loading with existing shared access. Its U
classification and empty-pattern result differ from the actual source above.
No historical parser, silent skip or memory claim is accepted wholesale.

## Six complete real workflows

Packet `/Volumes/KIOXIA/Developments/tmp/bed-nuc-witness-20261103.sqZVfQlJ`.
Root reads full5592B helper/reference/two BED inputs and rehashes them before
one guarded execution. Helper SHA256
552b16d48004c71b2111c8253cc81e6d37a86a6ed8cb8e74447e8c5ff71ba8f3.
Physical USB disk12s1, external Cargo/Rustup/target/cache/tmp and28GiB free
are verified. Existing source binary1691808B SHA256
fc994e1fcfdce8ce8591630d71f21fe67e2275c2a8d967dc0e3dfb4cfa4677a3.
Exactly six10s/core0/32KiB-stream calls, packet8MiB/free8GiB; no retry or native
build. Default, strand/sequence/AaA, same case-insensitive, empty pattern,
names default and names/fullHeader all exit0. Root reads every complete output/
diagnostic and independently replays argv/status/helper/input/tool/output pins
and each generated-FAI snapshot. The first call creates FAI; subsequent body
stays identical, SHA256
95d03bd6d5ad838196ae9cc3acaca563cb36c3b1f512dcbc4d32d080e4b68b85.
Five ordinary rows are retained; two points, absent reference and overrun warn
and skip. Empty pattern counts12/8/12/12/12. AaA has two sensitive versus four
insensitive matches in AaAaAaUu. Default/fullHeader names-probe stdout and
stderr are byte-identical; full-name query is skipped in both. ORACLE SHA256
9b99fdfe3775973e4b5810ff8b45972389af4a8e3dcb8e2c24873556ba2e6f07.

## Affected policy awaiting a decision

An async question requests whether missing-reference/overrun rows should be
fatal under fail-fast/transactional output or preserve upstream warning/skip.
This is an intentional upstream contract, not a proven bug. Point handling
remains its observed source rule. No native lookup/output API is locked before
that choice. Existing reader reuse and ordinary nucleotide contract are clear.

## Indexed-reader integration correction

Root inspects complete cached noodles-fasta0.61.0 indexed-reader builder and
BufReader enum plus noodles-bgzf0.47.0 indexed builder. The File type parameter
does not imply plain-only access: `.gz`/`.bgz` paths select the BGZF variant,
requiring existing `.fai` and `.gzi`; other extensions select uncompressed.
This is extension-based dispatch, not generic gzip support or auto-indexing.
The earlier plain-file-only interpretation was incorrect.

On Nov3 root runs the existing seqio `cargo test --locked --offline --test
indexed_fasta` once under the owning BED physical-external guard, from the
clean seqio bf8c2c8 tree. Guard verifies USB disk12s1,27GiB free, all external
tool/cache/tmp paths and resolved external BED target directory. Actual
session3592 exits0:4passed,0failed/ignored, including real BGZF writer plus
FAI/GZI extraction, plain cross-line/cache queries, invalid ranges/names and
duplicate-name rejection. No source edits, dependency downloads or new API.
This is local MacARM debug coverage of that existing integration test only,
not a new all-platform/full-suite/performance qualification. BED consumer
still needs its integrated coverage and explicit reference policy.
