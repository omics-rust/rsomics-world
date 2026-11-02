# VCF concat normal compatibility and region-file consumer

Status: observed source/native captures adopted as diagnosis and plan inputs,
not accepted tests or full concat qualification. Consumer wave plan:
`docs/plans/2026-11-02-vcf-region-files-and-normal-oracles.md`.

## Captured normal workflows

One uIrZ88 bundle completed332 commands/74 paired cases/0 timeouts. All16
encoding combinations, four BCF-G outputs and six full two-sample phase cases
match normalized complete headers/bodies. Source/tool174-body/mode identities
remain unchanged; root reads the full226-line runner/report and freshly checks
all894 final manifest entries. ManifestSHA324b2121aad8f4c1d880f113975360ff85b3db2c90c3629366a4381d337bd5a5;
root checksum logSHA429d6e533c598bf31fff27e7865ed486d71a32b20c55002a2bf18247facb7251,
stderr0 at vcf-concat-probe-root-20261102.aMJf7i.

All36 indexed inline-region cases have equal record multisets/dedup winners;
twenty differ only in same-coordinate order. Native retains A-del,B10,C10,
source gives B10,C10,A-del; exact dedup keeps the same B10 in both. One new
asynchronous policy question asks whether to preserve documented input order
(recommended) or align source order. Neither is labeled an upstream defect,
and no body-order normalization is accepted as a compatibility pass. Missing-
LF policy remains a separate pending question.

The two TSV `-R` cases fail natively before output because a whole tabbed line
is interpreted as the reference name. Root reads actual shared entry points
and primary installed manual; this is not an invalid fixture or CLI setup.
Other observed safety contrasts retain exact outputs, including upstream
QUAL=bad coercion to0/exit0. Native fail-loud behavior is preserved.

Report uIrZ88/REPORT.md SHA64a0601d348def4eb5381dd7c5ec2ae23331dedde6d427a71c7febe23db44d47;
runnerSHA36feb2c7ff0d626e812c10acdbad447643fcbc25ff08e43a10628e3be562d4fb.
The capture.log hash in the in-process inventory predates its final stdout;
the final894-file manifest is authoritative, not that one stale row.

## Focused file-format contract

A separate FI6p2q bundle, justified by the confirmed bug, executes33 commands/
28 outcomes over six normal formats plus one legacy control, with0 timeouts.
View and concat agree: source accepts TSV point/interval, BED, uppercase gzip
BED, ordinary-gzip TSV and VCF POS-only files. Native plain formats exit1 as
whole-line-reference errors; gzip exits4 as UTF-8 failure. Legacy colon-file
selection works natively and is rejected upstream. No parser fix or repeated
old bundle is hidden in this capture.

Report FI6p2q/REPORT.md SHA58c92d160fde1625563c8a3538eedc935a5f5b2c32d06a32dba7af3c66e3d4a6;
runnerSHA97278217b99b7bf7ec3094d1b324792bd5e7ccc8673bbf0dd71bf57a7f1bd2b4.
These observations establish the normal coordinate conversion, not INFO/END,
symbolic/gVCF or tabix-streaming region-file guarantees.

## Existing Layer A component, not another parser

Root reads complete intervals0.4.0 parser and real Call/CNV consumer adapters.
Clean sourcecda29c88b67882b63e1710a22aa480e905d4423f matches cached published
parser bytes. Reuse its gzip/BED/VCF/tabular normalization; VCF handles only
typed Noodles conversion, its legacy selector grammar and selection policy.
No new foundation/API is needed. A source-only non-UTF-8 suffix-inference
finding is retained for later foundation follow-up before public delivery;
do not claim it runtime-qualified or duplicate its parser to avoid it.

The planned wave is unambiguous shared-consumer repair plus observed normal
workflow oracles, not a request to implement every hypothetical corner case.
Both public-policy holds, performance, broader resource/integration and owning
native-four/delivery gates remain separate. No owning stage/commit/publication
or new test/production code has occurred for this wave yet.
