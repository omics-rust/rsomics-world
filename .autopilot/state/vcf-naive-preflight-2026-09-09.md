# Naive concat validated two-pass preflight

Status: reviewed test-only snapshot `vcf-naive-extra-red-2026-09-09` is ready
for expected-red execution. No two-pass production edits have been made.
The separately verified nonindexed BCF framing repair `34296650518` supplies
the checked logical EOF contract. Indexed fixture correction `34299936099`
is still under full verification; its source snapshot excludes this test.

## Current source findings

Naive inspection reads/inflates every frame, then reopens and parses every
typed record, then reopens for raw copying after all inputs pass. Its local
inflater uses Noodles' fixed-size BGZF header despite its structural reader
accepting arbitrary extra fields. The ordinary typed reader also sniffs magic
from only its current compressed buffer slice. A valid large extra field can
exceed that slice before decoded magic becomes available. These are source
findings; the new CLI matrix has not yet produced runtime evidence.

One appended group has 32 legal extra-field cases and four canonical controls
across VCF/BCF, first/second inputs, before/after BC, header/body frames and
stdout/named outputs. Large cases have a 9,000-byte extra payload. Four data
frames split the magic and place five body bytes in the header-boundary frame.
Independent per-member/whole-stream decoding, explicit header/dictionary/body
checks and raw output prefix/tail identity establish the fixture contract.
Initial fixed-header rejection can mask the large-buffer sniff issue; do not
attribute its cause from that red alone. Snapshot README records exact hashes
and coverage limits.

## Implementation after observed red

Add a private `Reader::open_bgzf` backed by a private generic constructor for
tests. Feed buffered input into the existing validated `DecodedReader`, consume
and replay at most 16 decoded magic bytes, and require exact BCF 2.2 or the
existing VCF fileformat prefix. Retry Interrupted and handle short reads.
Reuse existing uncompressed-stream `Inner::BcfRaw`/`Inner::Vcf`, header/schema
and typed record methods; do not add another parser or public format type.
Leave ordinary `Reader::open` unchanged in this bounded repair.

Replace naive's frame-only pass plus ordinary open with this validated
constructor and the existing complete typed loop. Derive its local Encoding
from the reader, retain header/dictionary equality, and remove the redundant
inspection implementation. Migrate rather than discard its split-magic test.
Use the shared private `inflate_frame` for the boundary-copy operation.
Keep the final raw-copy pass, output I/O classification and terminal EOF.
All inputs must reach validated logical/physical EOF before any output byte;
blind draining after a malformed BCF record is not equivalent.

Add constructor tests for short/Interrupted reads, invalid/incomplete magic,
unsupported BCF version, long empty prefixes and large extra fields. Add late
CRC/ISIZE, missing/repeated EOF, trailing-byte and typed-invalid record guards
on the second input, checking both empty stdout and preserved named output.
Retain zero-length, mixed-format, dictionary and header-only controls. Check
headers ending exactly on a frame boundary, mid-frame body and empty later
inputs. A final VCF #CHROM line without newline needs explicit coverage because
typed parsing and boundary copying currently accept different endings.

After unchanged consumer tests pass, run full four-native profiles and pinned
oracles, then benchmark against both the current three-pass implementation
and bcftools 1.24. Report complete per-mode CPU/RSS/I/O and many-sample ligation
evidence separately. No speed, all-input memory bound or TOCTOU guarantee is
implied by reducing passes. Existing read-side error wrappers remain distinct
from the writer's I/O-kind preservation.

No Layer A item is needed; this shares code inside one real product. Keep
captures external, explicitly disable external diff and verify patch replay.
Product HEAD/index and publication gates remain unchanged.
